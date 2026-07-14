use crate::bytecode::Bytecode;
use crate::callable::UpvalueDescriptor;
use crate::interner::Symbol;
use crate::method::{encode_selector, SignatureKind};
use crate::value::Value;
use phalcom_ast::ast::Argument;
use phalcom_common::range::SourceRange;

use super::error::CompilerError;
use super::state::Local;
use super::Compiler;

impl<'vm> Compiler<'vm> {
    /// Emits `opcode` into the current function's chunk.
    pub(crate) fn emit(&mut self, opcode: Bytecode, range: SourceRange) {
        self.functions.last_mut().unwrap().chunk.add_instruction(opcode, range);
    }

    /// Adds `value` to the current function's constant pool, returning its index.
    pub(crate) fn add_constant(&mut self, value: Value) -> u16 {
        self.functions.last_mut().unwrap().chunk.add_constant(value)
    }

    /// Emits a read of the receiver `self` for the current body.
    ///
    /// In a method/module body `self` is the frame receiver, emitted as
    /// [`Bytecode::GetSelf`]. Inside a block, `self` is not the block object —
    /// it is the enclosing method's receiver, captured as an ordinary upvalue
    /// (functions.md §2), so it resolves through [`Self::resolve_upvalue`] and is
    /// emitted as [`Bytecode::GetUpvalue`].
    pub(super) fn emit_self(&mut self, range: SourceRange) {
        if self.functions.len() > 1 {
            let self_sym = self.vm.interner.intern("self");
            if let Some(upvalue) = self.resolve_upvalue(self_sym) {
                self.emit(Bytecode::GetUpvalue(upvalue as u16), range);
                return;
            }
        }
        self.emit(Bytecode::GetSelf, range);
    }

    /// Lowers a `super.sel(args)` send to [`Bytecode::SuperSend`] (U-INH §3.4,
    /// method-lookup.md §1.14).
    ///
    /// Emits the original receiver (`self`), then the arguments, then a
    /// `SuperSend` carrying `selector_sym`, `argc`, and the **defining class
    /// name** ([`Self::current_class`]) so the VM starts the walk above that
    /// class at dispatch time (DEC-INH-B). A `super` with no enclosing class —
    /// at top level or in a free function — has no defining class to anchor the
    /// walk and is rejected with [`CompilerError::SuperOutsideMethod`].
    pub(super) fn compile_super_send(&mut self, selector_sym: Symbol, args: Vec<Argument>, argc: u8, range: SourceRange) -> Result<(), CompilerError> {
        let defining = self.current_class.ok_or(CompilerError::SuperOutsideMethod)?;
        // Class-side `super` (U-ERR-FIX SUPER-STATIC): inside a `static`
        // member, `self` is the class object, whose own class is the
        // *metaclass* — so the walk must start above the metaclass's
        // superclass, not the instance side's. Re-anchor `defining` to the
        // metaclass's own name (`"<Name>.class"`, the ADR-0002 parallel-rule
        // naming `VM::create_class` registers in `self.classes`) so the VM's
        // `Bytecode::SuperSend` handler resolves the metaclass instead of
        // the instance class.
        let defining = if self.is_static_context {
            let name = self.vm.resolve_symbol(defining).to_string();
            self.vm.interner.intern(&format!("{name}.class"))
        } else {
            defining
        };
        self.emit_self(range);
        for arg in args {
            self.compile_expr(arg.expr)?;
        }
        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
        let defining_idx = self.add_constant(Value::Symbol(defining));
        self.emit(Bytecode::SuperSend(argc, selector_idx, defining_idx), range);
        Ok(())
    }

    pub(crate) fn begin_scope(&mut self) {
        self.functions.last_mut().unwrap().scope_depth += 1;
    }

    /// Closes the innermost scope, promoting captured locals to heap cells.
    ///
    /// The function-body scope is cleaned up by [`Bytecode::Return`], which
    /// truncates the whole frame window (see the VM's `Return`/`run_until`
    /// handling). We therefore never `Pop` locals here — doing so would discard
    /// the return value sitting on top of them (the calculator-getter bug). We
    /// only emit [`Bytecode::CloseUpvalue`] for captured locals so their heap
    /// cells are promoted (`Open` -> `Closed`) before the slot is reclaimed
    /// ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md)); the VM's
    /// `Return` closes them again idempotently for the explicit-return path.
    pub(crate) fn end_scope(&mut self, range: SourceRange) {
        let func = self.functions.last_mut().unwrap();
        func.scope_depth -= 1;
        let scope_depth = func.scope_depth;
        let mut to_close = Vec::new();
        while func.num_locals > 0 && func.locals[func.num_locals - 1].depth > scope_depth {
            func.num_locals -= 1;
            let local = func.locals.pop().unwrap();
            if local.is_captured {
                to_close.push(func.num_locals as u16);
            }
        }
        for slot in to_close {
            self.emit(Bytecode::CloseUpvalue(slot), range);
        }
    }

    /// Declares a new local named `name` in the current function.
    ///
    /// `is_mutable` records whether the binding may later be reassigned: `var`
    /// locals (and the synthetic receiver/parameter slots) pass `true`, while
    /// `let` locals pass `false` so the assignment path can reject stores to
    /// them ([ADR-0014](../../../docs/adr/accepted/0014-let-var-bindings.md)).
    pub(super) fn add_local(&mut self, name: Symbol, is_mutable: bool) {
        let func = self.functions.last_mut().unwrap();
        tracing::debug!("[Compiler] Adding local at depth {}", func.scope_depth);
        func.locals.push(Local { name, depth: func.scope_depth, is_captured: false, is_mutable });
        func.num_locals += 1;
        if func.num_locals > func.max_slots {
            func.max_slots = func.num_locals;
        }
    }

    /// Resolves `name` as a local in the current function, returning its slot.
    pub(super) fn resolve_local(&self, name: Symbol) -> Option<usize> {
        self.resolve_local_in(self.functions.len() - 1, name)
    }

    /// Resolves `name` as a local in the function at `func_idx`, returning its
    /// slot index (which doubles as the runtime stack slot within the frame).
    fn resolve_local_in(&self, func_idx: usize, name: Symbol) -> Option<usize> {
        let func = &self.functions[func_idx];
        (0..func.num_locals).rev().find(|&i| func.locals[i].name == name)
    }


    /// Resolves `name` as an upvalue captured by the current function.
    ///
    /// Walks the enclosing function-compilation states (via [`Self::functions`]
    /// indices — no aliasing borrows) marking the captured local so the
    /// enclosing frame closes it on scope exit
    /// ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md)).
    pub(super) fn resolve_upvalue(&mut self, name: Symbol) -> Option<usize> {
        self.resolve_upvalue_in(self.functions.len() - 1, name)
    }

    /// Resolves `name` as an upvalue of the function at `func_idx`, recursing
    /// into the enclosing function when the variable is itself an upvalue there.
    fn resolve_upvalue_in(&mut self, func_idx: usize, name: Symbol) -> Option<usize> {
        if func_idx == 0 {
            return None;
        }
        let enclosing = func_idx - 1;

        // 1. Resolve as a local in the enclosing function -> capture it directly.
        if let Some(slot) = self.resolve_local_in(enclosing, name) {
            self.functions[enclosing].locals[slot].is_captured = true;
            return Some(self.add_upvalue(func_idx, slot, true));
        }

        // 2. Otherwise resolve recursively as an upvalue of the enclosing
        //    function and chain through it.
        if let Some(upvalue_idx) = self.resolve_upvalue_in(enclosing, name) {
            return Some(self.add_upvalue(func_idx, upvalue_idx, false));
        }

        None
    }

    /// Records (deduplicating) an upvalue descriptor on the function at
    /// `func_idx`, returning its index in that function's upvalue list.
    fn add_upvalue(&mut self, func_idx: usize, index: usize, is_local: bool) -> usize {
        let upvalues = &mut self.functions[func_idx].upvalues;
        for (i, upval) in upvalues.iter().enumerate() {
            if upval.index == index && upval.is_local == is_local {
                return i;
            }
        }
        upvalues.push(UpvalueDescriptor { is_local, index });
        upvalues.len() - 1
    }

    /// Emits an ordinary `Invoke` send for operator selector `name` at
    /// `arity` (control-flow.md §1; U5-plan.md §4.1). Always builds the
    /// selector through [`encode_selector`] — never a hand-rolled string —
    /// so the compiler and the primitive registrations in `universe.rs`
    /// stay in lockstep (ADR-0012).
    pub(super) fn emit_operator_send(&mut self, name: &str, arity: u8, range: SourceRange) {
        let labels = vec![None; arity as usize];
        let selector = encode_selector(name, &labels, SignatureKind::Method(arity));
        let selector_sym = self.vm.interner.intern(&selector);
        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
        self.emit(Bytecode::Invoke(arity, selector_idx), range);
    }
}
