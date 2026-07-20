use crate::bytecode::Bytecode;
use crate::heap::ClassId;
use crate::value::Value;
use phalcom_common::range::SourceRange;
use std::cell::Cell;
use std::collections::HashSet;

/// One monomorphic inline-cache slot, owned by a single `Bytecode::Invoke` site.
#[derive(Debug, Clone, Copy)]
pub struct InlineCache {
    /// Receiver class the cached resolution was recorded for.
    pub class: ClassId,
    /// The resolved `MethodObject` handle.
    pub method: crate::heap::ObjRef,
    /// `VM.world_version` at record time; a mismatch means a method was
    /// (re)defined somewhere since, and the entry must be discarded.
    pub version: u64,
}

/// One global-resolution cache slot, owned by a single `Bytecode::GetGlobal` or
/// `Bytecode::SetGlobal` site.
///
/// Resolving a global name is otherwise a SipHash probe of
/// [`ModuleObject::name_to_slot`](crate::heap::ModuleObject) on *every* access,
/// plus a second probe of the core module when the name is not local (perf-log
/// F12: ~13% of `bare_send` ticks; `for`'s inner loop pays four probes per
/// iteration). This records where the name landed so repeat accesses index
/// straight into [`ModuleObject::globals`](crate::heap::ModuleObject).
#[derive(Debug, Clone, Copy)]
pub struct GlobalCache {
    /// Module the name actually resolved in — the accessing closure's own module,
    /// or the core module when it resolved through the fallback.
    pub module: crate::heap::ObjRef,
    /// Slot index within that module's `globals`.
    pub slot: usize,
    /// `globals_version` **of the accessing closure's module** (not of
    /// [`Self::module`]) at record time. A mismatch means that module declared a
    /// new name since, which may shadow this resolution, so the entry is dropped.
    pub version: u64,
}

/// A chunk of compiled bytecode and its associated constant values.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub spans: Vec<SourceRange>,
    /// Index into [`ModuleObject::sources`](crate::heap::ModuleObject) of the
    /// source text [`Self::spans`] point into.
    ///
    /// A module accumulates one source entry per compiled unit — one per REPL
    /// cell — so a span baked into *this* chunk must be rendered against the
    /// text that chunk was compiled from, not against whatever the module was
    /// most recently fed (U-REPL §D2, precondition 6). Debug data: read only on
    /// the error path, never in the interpreter loop.
    ///
    /// Stamped by the compiler when it finalizes a [`Callable`](crate::callable::Callable);
    /// a hand-built [`Chunk`] keeps the default `0`.
    pub source_id: u32,
    /// Parallel to `code`; only `Bytecode::Invoke` indices are ever non-`None`.
    /// Cell enables interior mutability for cache refill through a shared `&Chunk` borrow.
    pub caches: Vec<Cell<Option<InlineCache>>>,
    /// Parallel to `code`; only `Bytecode::GetGlobal`/`SetGlobal` indices are ever
    /// non-`None`. Separate from [`Self::caches`] because the two never occupy the
    /// same instruction, and a single union would pay for the wider variant at
    /// every site.
    pub gcaches: Vec<Cell<Option<GlobalCache>>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    /// Constructs a new, empty chunk.
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
            source_id: 0,
            caches: Vec::new(),
            gcaches: Vec::new(),
        }
    }

    /// Appends an instruction, keeping `caches.len() == gcaches.len() == code.len()`.
    pub fn add_instruction(&mut self, opcode: Bytecode, range: SourceRange) {
        self.code.push(opcode);
        self.spans.push(range);
        self.caches.push(Cell::new(None));
        self.gcaches.push(Cell::new(None));
    }

    /// Appends a constant and returns its index.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    /// Rewrites statically-adjacent `(GetLocal | Constant) -> Invoke` pairs into the
    /// fused [`Bytecode::InvokeLocal`] / [`Bytecode::InvokeConst`], each of which
    /// retires the pair's work in **one** dispatch instead of two (perf-log cut 008).
    ///
    /// Run once per chunk, after compilation, before the [`crate::callable::Callable`]
    /// is frozen.
    ///
    /// # The in-place rewrite, and why there is no re-layout
    ///
    /// The fused opcode replaces the *first* instruction of the pair and the original
    /// `Invoke` is left in place at `p + 1` as dead code, so `code.len()` never
    /// changes. That is what keeps this pass cheap and safe: every jump offset in the
    /// chunk stays correct, and the `ip`-indexed parallel arrays (`spans`, `caches`,
    /// `gcaches`) stay aligned with `code`. The alternative — compacting the array —
    /// would mean rewriting every branch offset and re-indexing three side tables, for
    /// the same number of saved dispatches.
    ///
    /// The dead `Invoke` costs 8 bytes of `code` and is never executed, because
    /// [`Bytecode::InvokeLocal`]/[`Bytecode::InvokeConst`] advance `ip` past it.
    ///
    /// # Why a jump target forbids the fusion
    ///
    /// The rewrite is sound only if `p + 1` is unreachable. If any branch targets the
    /// `Invoke` directly, that entry point must keep finding a real `Invoke` there —
    /// so such a pair is skipped. The fallback is simply the unfused pair, which is
    /// correct, just not fast.
    pub fn fuse_superinstructions(&mut self) {
        let targets = self.branch_targets();
        for p in 0..self.code.len().saturating_sub(1) {
            if targets.contains(&(p + 1)) {
                continue;
            }
            let Bytecode::Invoke(arity, selector) = self.code[p + 1] else { continue };
            self.code[p] = match self.code[p] {
                Bytecode::GetLocal(slot) => Bytecode::InvokeLocal(slot, arity, selector),
                Bytecode::Constant(idx) => Bytecode::InvokeConst(idx, arity, selector),
                _ => continue,
            };
        }
    }

    /// Every instruction index some branch in this chunk can jump to.
    ///
    /// A branch's offset is applied to the `ip` *already advanced past the branch
    /// itself* (`VM::apply_jump_offset`), so a branch at `b` with offset `o` targets
    /// `b + 1 + o`. Conditional branches contribute their target whether or not it is
    /// taken at runtime — this is a static, conservative set.
    fn branch_targets(&self) -> HashSet<usize> {
        self.code
            .iter()
            .enumerate()
            .filter_map(|(b, op)| {
                let offset = match *op {
                    Bytecode::Jump(o)
                    | Bytecode::JumpIfFalse(o)
                    | Bytecode::JumpIfNone(o)
                    | Bytecode::Loop(o)
                    | Bytecode::GuardBool(o)
                    | Bytecode::GuardBlock(o) => o,
                    _ => return None,
                };
                usize::try_from(b as i64 + 1 + offset as i64).ok()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_common::range::EmptySourceRange;

    fn chunk_of(code: &[Bytecode]) -> Chunk {
        let mut chunk = Chunk::new();
        for op in code {
            chunk.add_instruction(*op, EmptySourceRange);
        }
        chunk
    }

    #[test]
    fn fuses_both_pair_shapes_in_place() {
        let mut chunk = chunk_of(&[Bytecode::GetLocal(3), Bytecode::Invoke(1, 7), Bytecode::Constant(4), Bytecode::Invoke(0, 9)]);
        chunk.fuse_superinstructions();

        // The pair's head is rewritten and the `Invoke` stays put as dead code:
        // `code.len()` is unchanged, which is what keeps every jump offset and every
        // `ip`-indexed side table valid without a re-layout.
        assert_eq!(chunk.code.len(), 4);
        assert_eq!(chunk.code[0], Bytecode::InvokeLocal(3, 1, 7));
        assert_eq!(chunk.code[1], Bytecode::Invoke(1, 7));
        assert_eq!(chunk.code[2], Bytecode::InvokeConst(4, 0, 9));
        assert_eq!(chunk.code[3], Bytecode::Invoke(0, 9));
    }

    #[test]
    fn refuses_to_fuse_a_pair_whose_invoke_is_a_jump_target() {
        // `Jump(1)` at 0 lands on index 2 — the `Invoke`. Fusing would rewrite the
        // `GetLocal` at 1 and leave that entry point executing a *dead* instruction
        // with no receiver pushed. The pass must leave this pair alone.
        let mut chunk = chunk_of(&[Bytecode::Jump(1), Bytecode::GetLocal(0), Bytecode::Invoke(0, 5)]);
        chunk.fuse_superinstructions();

        assert_eq!(chunk.code[1], Bytecode::GetLocal(0), "fused a pair reachable by a jump into its Invoke");
        assert_eq!(chunk.code[2], Bytecode::Invoke(0, 5));
    }

    #[test]
    fn a_backward_loop_edge_also_pins_its_target() {
        // `Loop(-3)` at 3 targets index 1, not the `Invoke` at 2 — so this pair is
        // fusible, and the guard must not be so conservative it refuses it.
        let mut chunk = chunk_of(&[Bytecode::Pop, Bytecode::GetLocal(0), Bytecode::Invoke(0, 5), Bytecode::Loop(-3)]);
        chunk.fuse_superinstructions();

        assert_eq!(chunk.code[1], Bytecode::InvokeLocal(0, 0, 5));
    }

    // U-CLASSCLOSE §11.2: the two `ic_add_method_invalidates`/
    // `ic_override_after_caching` golden fixtures used to prove inline-cache
    // invalidation by *reopening* a class — no longer expressible from `.ph`
    // source now that classes are closed (decision 0065). Rewritten here as
    // in-crate tests driving the install path directly (`add_method` +
    // `world_version`), which needs `VM::world_version` (`pub(crate)`) —
    // unreachable from an integration test, reachable from inside the crate
    // (U-CLASSCLOSE §1.5).
    //
    // `ClassObject::add_method` does **not** bump `world_version` itself —
    // every real call site bumps it as a separate adjacent statement
    // (`primitive/mod.rs`, `dispatch.rs`'s constructor-install arm,
    // `universe/primitives.rs`). A rewrite that called `add_method` alone
    // would leave the cached entry *valid* against the stale method and pass
    // while proving the opposite of invalidation — both tests below bump
    // explicitly and assert the version actually moved before asserting
    // anything about the cache.
    fn install_and_bump(vm: &mut crate::vm::VM, class: crate::heap::ClassId, selector: crate::interner::Symbol, method: crate::heap::ObjRef) {
        let before = vm.world_version;
        vm.heap.class_mut(class).add_method(selector, method);
        vm.world_version += 1;
        assert_ne!(vm.world_version, before, "world_version must actually move, or the cache would stay valid against the stale method");
    }

    /// Finds the `Bytecode::Invoke` index in `chunk` whose selector constant
    /// is `selector`, and asserts its cache slot is already populated
    /// (warmed by the caller running the closure first).
    fn warmed_invoke_index(chunk: &Chunk, selector: crate::interner::Symbol) -> usize {
        let ip = chunk
            .code
            .iter()
            .position(|op| matches!(op, Bytecode::Invoke(_, sel_idx) if chunk.constants[*sel_idx as usize] == Value::Symbol(selector)))
            .expect("selector's Invoke site must exist in the warmed chunk");
        assert!(chunk.caches[ip].get().is_some(), "cache must be populated after three warming sends");
        ip
    }

    #[test]
    fn ic_add_method_invalidates() {
        let mut vm = crate::vm::VM::new();
        let module = vm.create_module("main", "ic_add_method_invalidates");
        let closure = vm
            .compile_closure(
                module,
                "class A {\n  val => 1\n}\nconst a = A.new()\nconst _ = a.val\nconst _ = a.val\nconst _ = a.val\n",
            )
            .expect("compiles");
        vm.run_in_module(module, closure).expect("runs");

        let val_sym = vm.get_or_intern("val");
        let a_key = crate::vm::ClassKey { module, name: vm.get_or_intern("A") };
        let class_a = *vm.classes.get(&a_key).expect("A must be registered after running its Bytecode::Class");

        let chunk = &vm.heap.closure(closure).callable.chunk;
        let ip = warmed_invoke_index(chunk, val_sym);
        let old_entry = chunk.caches[ip].get().unwrap();

        fn returns_two(_vm: &mut crate::vm::VM, _recv: &Value, _args: &[Value]) -> crate::error::PhResult<Value> {
            Ok(Value::Number(2.0))
        }
        let new_method = vm.heap.alloc(crate::heap::Object::Method(Box::new(crate::method::MethodObject::new_single(
            val_sym,
            crate::method::SignatureKind::Getter,
            crate::method::MethodKind::Primitive(returns_two),
        ))));
        install_and_bump(&mut vm, class_a, val_sym, new_method);

        let installed = *vm.heap.class(class_a).methods.get(&val_sym).expect("val must resolve on A after install");
        assert_ne!(installed, old_entry.method, "the class's installed method must have changed");
        assert_ne!(old_entry.version, vm.world_version, "the cached entry's version must no longer match — it must be treated as stale");
    }

    #[test]
    fn ic_override_after_caching() {
        let mut vm = crate::vm::VM::new();
        let module = vm.create_module("main", "ic_override_after_caching");
        let mut source = "class A {\n  get => 1\n}\nconst a = A.new()\n".to_string();
        for _ in 0..10 {
            source.push_str("const _ = a.get\n");
        }
        let closure = vm.compile_closure(module, &source).expect("compiles");
        vm.run_in_module(module, closure).expect("runs");

        let get_sym = vm.get_or_intern("get");
        let a_key = crate::vm::ClassKey { module, name: vm.get_or_intern("A") };
        let class_a = *vm.classes.get(&a_key).expect("A must be registered after running its Bytecode::Class");

        let chunk = &vm.heap.closure(closure).callable.chunk;
        let ip = warmed_invoke_index(chunk, get_sym);
        let old_entry = chunk.caches[ip].get().unwrap();

        fn returns_two(_vm: &mut crate::vm::VM, _recv: &Value, _args: &[Value]) -> crate::error::PhResult<Value> {
            Ok(Value::Number(2.0))
        }
        let new_method = vm.heap.alloc(crate::heap::Object::Method(Box::new(crate::method::MethodObject::new_single(
            get_sym,
            crate::method::SignatureKind::Getter,
            crate::method::MethodKind::Primitive(returns_two),
        ))));
        install_and_bump(&mut vm, class_a, get_sym, new_method);

        let installed = *vm.heap.class(class_a).methods.get(&get_sym).expect("get must resolve on A after install");
        assert_ne!(installed, old_entry.method, "a heavily-warmed (10 sends) cache must still see the new method installed");
        assert_ne!(old_entry.version, vm.world_version, "a heavily-warmed cache must still be busted, not just a lightly-touched one");
    }
}
