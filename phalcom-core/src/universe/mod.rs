//! Bootstrap of the kernel class tower and installation of core primitives.
//!
//! The kernel is a cyclic graph closed through a distinct `Metaclass class`
//! row (`Metaclass.class == Metaclass class`, `(Metaclass class).class ==
//! Metaclass`; `object-model.md` §5–6). Under
//! [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md) that cycle is built
//! by **allocate-then-patch**: every class row is first allocated bare in the
//! [`Heap`] to obtain its [`ClassId`], then its `class` and `superclass`
//! handles are written in place.
//!
//! The metaclass hierarchy runs *parallel* to the instance hierarchy
//! ([ADR-0002](../../../docs/adr/accepted/0002-metaclass-tower-parallel-rule.md)):
//! `(X class).superclass == (X.superclass) class`. `Behavior`
//! ([ADR-0003](../../../docs/adr/accepted/0003-introduce-behavior-kernel-class.md)) is
//! the shared abstract superclass of `Class` and `Metaclass`, so the tower
//! closes at an 8-row apex instead of collapsing `Metaclass`/`Class` into
//! their own metaclasses (F6).

mod core_classes;
mod invariants;
mod primitives;

pub use core_classes::CoreClasses;

use crate::heap::{ClassId, Heap, ObjRef};
use crate::interner::Symbol;
use std::collections::HashMap;

/// The kernel: handles to the bootstrapped core classes.
#[derive(Debug, Clone)]
pub struct Universe {
    /// Handles to every bootstrapped core class and metaclass.
    pub classes: CoreClasses,
    /// Override-epoch flag for the `Bool`-receiver sacred selectors
    /// (`and(_)`, `or(_)`, `not()`, `ifTrue(_)`, `ifFalse(_)`,
    /// `ifTrue(_)ifFalse(_)`). `true` from bootstrap until any of them is
    /// (re)installed directly on the kernel `Bool` class, at which point the
    /// sacred-selector inliner's [`crate::bytecode::Bytecode::GuardBool`]
    /// deopts every inlined call site back to a real send
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    pub bool_sacred_pristine: bool,
    /// Override-epoch flag for the `Block`-receiver sacred selectors
    /// (`whileTrue(_)`), mirroring [`Universe::bool_sacred_pristine`] for the
    /// kernel `Block` class.
    pub block_sacred_pristine: bool,
    /// Override-epoch flag for `Number`'s `toString` getter.
    ///
    /// `true` once bootstrap (`core.ph`) has finished loading, `false` if any
    /// `.ph` or native (re)install of `toString` lands directly on the
    /// kernel `Number` class afterward — mirroring
    /// [`Universe::bool_sacred_pristine`]'s pattern
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md))
    /// but guarding [`crate::value::Value::to_display_string`]'s leaf fast
    /// path instead of the inliner.
    ///
    /// Unlike the `Bool`/`Block` sacred flags, this one must **not** start
    /// `true` in [`Universe::new`]: `core.ph` legitimately (re)installs
    /// `toString` on some leaf classes during bootstrap (e.g. `String`'s
    /// `toString => self`), which would immediately (and correctly) flip a
    /// flag seeded `true`. Instead it starts `false` and is snapshotted to
    /// `true` once by [`crate::vm::VM::new`] right after `core.ph` finishes
    /// running, so only a *post-bootstrap* (user-code) reinstall ever clears
    /// it again. Getting this backwards either kills the fast path forever
    /// (seeded `false`, never snapshotted) or reintroduces CB-6 (seeded
    /// `true`, so a legitimate bootstrap install of a container's
    /// `toString` never widens past the leaf types this flag was meant to
    /// cover — see [`crate::value::Value::to_display_string`]'s doc for why containers
    /// must never use this fast path at all).
    pub number_tostring_pristine: bool,
    /// Override-epoch flag for `Int`'s `toString` getter.
    pub int_tostring_pristine: bool,
    /// Override-epoch flag for `Float`'s `toString` getter.
    pub float_tostring_pristine: bool,
    /// Override-epoch flag for `Symbol`'s `toString` getter.
    ///
    /// Same rules as [`Universe::number_tostring_pristine`], watching the
    /// kernel `Symbol` class instead.
    pub symbol_tostring_pristine: bool,
    /// Override-epoch flag for `String`'s `toString` getter.
    ///
    /// Same rules as [`Universe::number_tostring_pristine`], watching the
    /// kernel `String` class instead. `core.ph` itself installs `String`'s
    /// `toString => self` during bootstrap — exactly the transient,
    /// bootstrap-only flip this flag's `false`-then-snapshot ordering exists
    /// to absorb.
    pub str_tostring_pristine: bool,
    /// Loaded **imported** modules keyed by canonical absolute filesystem
    /// path (U15, DEC-U15 A+A), distinct from [`VM::modules`](crate::vm::VM::modules)
    /// (which keys the singleton `core`/`main` modules by logical name).
    ///
    /// Memoizes [`VM::import_module`](crate::vm::VM::import_module): a
    /// canonical path is inserted the moment its `Module` is *allocated* —
    /// before it is compiled or run — so a re-entrant probe of the same path
    /// (a second `import` of the same file, or a cyclic import re-entering
    /// mid-load) always returns the identical [`ObjRef`], never recompiles,
    /// and never loops. There is deliberately no separate "in-progress" set:
    /// a module reached before its own top level finishes running is simply
    /// found here with a still-partially-populated
    /// [`ModuleObject`](crate::heap::ModuleObject) (some globals declared,
    /// some not yet) — the documented cyclic-import partial-init hazard (U15
    /// plan §4): a name read across the not-yet-complete edge surfaces the
    /// ordinary "undefined global" / `doesNotUnderstand` miss, not a hang or
    /// a silent duplicate.
    pub module_registry: HashMap<String, ObjRef>,
}

impl Universe {
    /// Calls `push` once for every handle the kernel holds — [`CoreClasses`]'
    /// pinned tower plus the import [`Universe::module_registry`].
    ///
    /// The `universe` row of
    /// [memory-management.md §2.1](../../../docs/spec/v0.2/memory-management.md).
    /// Exhaustively destructured for the same reason as
    /// [`CoreClasses::each_handle`]: a new handle-bearing field must fail to
    /// compile rather than silently go unrooted (forge finding F6).
    ///
    /// `bool_sacred_pristine`/`block_sacred_pristine`/`number_tostring_pristine`/
    /// `int_tostring_pristine`/`float_tostring_pristine`/
    /// `symbol_tostring_pristine`/`str_tostring_pristine` are `bool`
    /// override-epoch flags
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)),
    /// not handles.
    pub fn each_handle(&self, push: &mut impl FnMut(ObjRef)) {
        let Universe {
            classes,
            bool_sacred_pristine: _,
            block_sacred_pristine: _,
            number_tostring_pristine: _,
            int_tostring_pristine: _,
            float_tostring_pristine: _,
            symbol_tostring_pristine: _,
            str_tostring_pristine: _,
            module_registry,
        } = self;
        classes.each_handle(push);
        for module in module_registry.values() {
            push(*module);
        }
    }

    /// Bootstraps the core class tower into `heap` and returns the [`Universe`].
    pub fn new(heap: &mut Heap) -> Self {
        Universe {
            classes: Self::create_core_classes(heap),
            bool_sacred_pristine: true,
            block_sacred_pristine: true,
            // Seeded `false`, not `true` — see
            // [`Universe::number_tostring_pristine`]'s doc: `core.ph` itself
            // installs some of these during bootstrap, so a flag seeded
            // `true` here would be flipped by a *legitimate* install before
            // bootstrap even finishes. [`VM::new`](crate::vm::VM::new)
            // snapshots all three to `true` once bootstrap completes.
            number_tostring_pristine: false,
            int_tostring_pristine: false,
            float_tostring_pristine: false,
            symbol_tostring_pristine: false,
            str_tostring_pristine: false,
            module_registry: HashMap::new(),
        }
    }

    /// The `Bool`-receiver sacred selectors watched by
    /// [`Universe::bool_sacred_pristine`]
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    pub const BOOL_SACRED_SELECTORS: &'static [&'static str] = &["and(_)", "or(_)", "not()", "ifTrue(_)", "ifFalse(_)", "ifTrue(_,ifFalse)"];

    /// The `Block`-receiver sacred selectors watched by
    /// [`Universe::block_sacred_pristine`]
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    pub const BLOCK_SACRED_SELECTORS: &'static [&'static str] = &["whileTrue(_)"];

    /// The `toString` getter selector watched by
    /// [`Universe::number_tostring_pristine`]/
    /// [`Universe::int_tostring_pristine`]/
    /// [`Universe::float_tostring_pristine`]/
    /// [`Universe::symbol_tostring_pristine`]/
    /// [`Universe::str_tostring_pristine`] on their respective kernel
    /// classes. A getter encodes to its bare name
    /// ([`crate::method::encode_selector`]'s `SignatureKind::Getter` arm),
    /// so this is `"toString"`, not `"toString()"`.
    pub const LEAF_TOSTRING_SELECTORS: &'static [&'static str] = &["toString"];

    /// Flags a (re)definition of `selector` directly on `class_id`, flipping
    /// the relevant override-epoch flag if it is a sacred selector on the
    /// kernel `Bool`/`Block` class.
    ///
    /// Called from the [`crate::bytecode::Bytecode::Method`] handler
    /// (`vm.rs`) every time a class body attaches a method — the only place
    /// user code can (re)install a method on a class row, whether that row
    /// is the *original* kernel `Bool`/`Block` (impossible for surface
    /// Phalcom today — there is no class-reopening syntax) or a
    /// same-named redeclaration that U5 makes *reopen* the existing row
    /// (see `compiler/lib.rs`'s `Statement::Class` handling) specifically so
    /// this deopt path is exercisable and testable
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    /// A cheap `==` on two [`ClassId`]s per method definition; not on any
    /// hot path.
    pub fn note_method_installed(&mut self, class_id: ClassId, selector: Symbol, interner: &crate::interner::Interner) {
        let name = interner.lookup(selector);
        if class_id == self.classes.bool_class && Self::BOOL_SACRED_SELECTORS.contains(&name) {
            self.bool_sacred_pristine = false;
        }
        if class_id == self.classes.block_class && Self::BLOCK_SACRED_SELECTORS.contains(&name) {
            self.block_sacred_pristine = false;
        }
        if (class_id == self.classes.number_class || class_id == self.classes.int_class) && Self::LEAF_TOSTRING_SELECTORS.contains(&name) {
            self.int_tostring_pristine = false;
        }
        if (class_id == self.classes.number_class || class_id == self.classes.float_class) && Self::LEAF_TOSTRING_SELECTORS.contains(&name) {
            self.float_tostring_pristine = false;
        }
        if class_id == self.classes.number_class && Self::LEAF_TOSTRING_SELECTORS.contains(&name) {
            self.number_tostring_pristine = false;
        }
        if class_id == self.classes.symbol_class && Self::LEAF_TOSTRING_SELECTORS.contains(&name) {
            self.symbol_tostring_pristine = false;
        }
        if class_id == self.classes.string_class && Self::LEAF_TOSTRING_SELECTORS.contains(&name) {
            self.str_tostring_pristine = false;
        }
    }

    /// Snapshots the leaf `toString` override-epoch flags
    /// ([`Universe::number_tostring_pristine`]/
    /// [`Universe::int_tostring_pristine`]/
    /// [`Universe::float_tostring_pristine`]/
    /// [`Universe::symbol_tostring_pristine`]/
    /// [`Universe::str_tostring_pristine`]) to `true`.
    ///
    /// Called exactly once, by [`crate::vm::VM::new`] immediately after
    /// `core.ph` finishes running. Bootstrap's own `.ph` reopens (e.g.
    /// `String`'s `toString => self`) may have already flipped one or more
    /// of these flags `false` via [`Universe::note_method_installed`] — that
    /// is expected and this call unconditionally clears it back to `true`,
    /// since bootstrap-time installs are the *baseline*, not an override.
    /// Only a (re)install of `toString` on `Number`/`Symbol`/`String` that
    /// happens after this call — i.e. from user code — is meant to clear a
    /// flag again.
    pub fn mark_leaf_tostring_pristine(&mut self) {
        self.number_tostring_pristine = true;
        self.int_tostring_pristine = true;
        self.float_tostring_pristine = true;
        self.symbol_tostring_pristine = true;
        self.str_tostring_pristine = true;
    }
}

// U-CLASSCLOSE (PDR-0001): five golden fixtures used to prove
// override-epoch/inline-cache invalidation by *reopening* a kernel class
// from `.ph` source (`Number#toString`, `Bool#and`, `Block#whileTrue`,
// `Option#match`) — no longer expressible now that classes are closed.
// Rewritten here, in-crate, driving the exact install sequence
// `Bytecode::Method`'s own handler runs (`dispatch.rs`: `add_method`, bump
// `world_version`, then `note_method_installed` for a non-static instance
// method) directly against the kernel `ClassId`s in `vm.universe.classes`,
// which needs `pub(crate) VM::world_version` — unreachable from an
// integration test, reachable from inside the crate (mirrors the two
// `ic_*` tests in `chunk.rs`, U-CLASSCLOSE §1.5/§11.2).
#[cfg(test)]
mod tests {
    use crate::error::PhResult;
    use crate::heap::Object;
    use crate::method::{MethodKind, MethodObject, SignatureKind};
    use crate::value::Value;
    use crate::vm::VM;

    /// Installs `method` under `selector` directly on `class`, exactly as
    /// `Bytecode::Method`'s non-static arm does — `add_method`, then bump
    /// `world_version`, then `note_method_installed` (sacred-selector
    /// tracking is a no-op for a selector/class pair it doesn't watch).
    fn install_kernel_method(vm: &mut VM, class: crate::heap::ClassId, selector: crate::interner::Symbol, method: crate::heap::ObjRef) {
        let before = vm.world_version;
        vm.heap.class_mut(class).add_method(selector, method);
        vm.world_version += 1;
        assert_ne!(
            vm.world_version, before,
            "world_version must actually move, or a warmed cache would stay valid against the stale method"
        );
        vm.universe.note_method_installed(class, selector, &vm.interner);
    }

    fn read_global(vm: &VM, module: crate::heap::ObjRef, name: &str) -> Value {
        let sym = vm
            .interner
            .find(name)
            .unwrap_or_else(|| panic!("global `{name}` was never interned — did it compile?"));
        vm.heap.module(module).get(sym).unwrap_or_else(|| panic!("global `{name}` was never bound"))
    }

    fn read_str_global(vm: &VM, module: crate::heap::ObjRef, name: &str) -> String {
        match read_global(vm, module, name) {
            Value::Obj(id) => vm.heap.string(id).as_str().to_string(),
            other => panic!("global `{name}` is not a Str: {other:?}"),
        }
    }

    #[test]
    fn kernel_number_leaf_tostring_override_flips_the_fast_path() {
        // Was strings/print_number_reopen_agrees.ph. `Value::to_display_string`
        // is the single function both `System.print` (a container's per-element
        // render) and `\(…)` interpolation route through (see its own doc) —
        // testing it directly covers both surfaces at once.
        let mut vm = VM::new();
        assert!(
            vm.universe.number_tostring_pristine,
            "post-bootstrap baseline must start pristine (VM::new calls mark_leaf_tostring_pristine last)"
        );
        assert_eq!(Value::Int(1).to_display_string(&mut vm).unwrap(), "1");

        let number_class = vm.universe.classes.number_class;
        let selector = vm.get_or_intern("toString");
        fn returns_n(vm: &mut VM, _recv: &Value, _args: &[Value]) -> PhResult<Value> {
            Ok(vm.alloc_string_value("N".to_string()))
        }
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            selector,
            SignatureKind::Getter,
            MethodKind::Primitive(returns_n),
        ))));
        install_kernel_method(&mut vm, number_class, selector, method);

        assert!(
            !vm.universe.number_tostring_pristine,
            "installing toString on Number must flip the leaf fast-path flag"
        );
        assert_eq!(
            Value::Int(1).to_display_string(&mut vm).unwrap(),
            "N",
            "to_display_string must fall back to the real toString send"
        );
    }

    #[test]
    fn kernel_bool_sacred_override_deopts_nested_iftrue() {
        // Was absence/absence_iftrue_nested_deopt_path.ph. Every sacred call
        // compiles its arms twice (perf-log F13) — inlined, and again as
        // block literals for the GuardBool fallback, itself compiled with
        // the inliner suppressed. Installing `and` (a sacred selector, not
        // ifTrue/ifFalse themselves) flips `bool_sacred_pristine` *before*
        // this closure ever runs, so every GuardBool site here takes the
        // deopt branch on its very first execution — proving the fallback's
        // nested conditionals, compiled as ordinary sends, compute the same
        // answers as the (separately-tested) fast path.
        let mut vm = VM::new();
        let bool_class = vm.universe.classes.bool_class;
        let and_selector = vm.get_or_intern("and(_)");
        fn returns_false(_vm: &mut VM, _recv: &Value, _args: &[Value]) -> PhResult<Value> {
            Ok(Value::Bool(false))
        }
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            and_selector,
            SignatureKind::Method(1),
            MethodKind::Primitive(returns_false),
        ))));
        install_kernel_method(&mut vm, bool_class, and_selector, method);
        assert!(!vm.universe.bool_sacred_pristine, "installing `and` (sacred) must flip the Bool guard flag");

        let module = vm.create_module("main", "kernel_bool_sacred_override_deopts_nested_iftrue");
        let source = "
let n = 3
let r = (n < 1).ifTrue(|| { \"one\" }, ifFalse: || {
  (n < 2).ifTrue(|| { \"two\" }, ifFalse: || {
    (n < 3).ifTrue(|| { \"three\" }, ifFalse: || {
      (n < 4).ifTrue(|| { \"four\" }, ifFalse: || { \"big\" })
    })
  })
})
let someCheck = (n < 4).ifTrue(|| { (n < 2).ifTrue(|| { \"inner\" }) }).isSome
let noneCheck = (n < 2).ifTrue(|| { (n < 4).ifTrue(|| { \"inner\" }) }).isNone
";
        let closure = vm.compile_closure(module, source).expect("compiles");
        vm.run_in_module(module, closure).expect("runs on the deopt path");

        assert_eq!(read_str_global(&vm, module, "r"), "four");
        assert_eq!(read_global(&vm, module, "someCheck"), Value::Bool(true));
        assert_eq!(read_global(&vm, module, "noneCheck"), Value::Bool(true));
    }

    #[test]
    fn kernel_bool_sacred_override_deopts_some_lift() {
        // Was absence/absence_iftrue_some_lift_deopt_path.ph. Same
        // deopt-forcing setup as the nested case, proving the untaken-arm
        // Some-lift (WrapSome on the fast path) still holds through the
        // primitive path (`bool_if_true`/`bool_if_false`) once deopted.
        let mut vm = VM::new();
        let bool_class = vm.universe.classes.bool_class;
        let and_selector = vm.get_or_intern("and(_)");
        fn returns_false(_vm: &mut VM, _recv: &Value, _args: &[Value]) -> PhResult<Value> {
            Ok(Value::Bool(false))
        }
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            and_selector,
            SignatureKind::Method(1),
            MethodKind::Primitive(returns_false),
        ))));
        install_kernel_method(&mut vm, bool_class, and_selector, method);
        assert!(!vm.universe.bool_sacred_pristine);

        let module = vm.create_module("main", "kernel_bool_sacred_override_deopts_some_lift");
        let source = "
let a = true.ifTrue || { 42 }.isSome
let b = false.ifTrue || { 42 }.isSome
let c = false.ifTrue || { 42 }.isNone
let d = true.ifTrue || { }.isSome
";
        let closure = vm.compile_closure(module, source).expect("compiles");
        vm.run_in_module(module, closure).expect("runs on the deopt path");

        assert_eq!(read_global(&vm, module, "a"), Value::Bool(true));
        assert_eq!(read_global(&vm, module, "b"), Value::Bool(false));
        assert_eq!(read_global(&vm, module, "c"), Value::Bool(true));
        assert_eq!(read_global(&vm, module, "d"), Value::Bool(true));
    }

    #[test]
    fn kernel_block_sacred_override_deopts_inline_while_true() {
        // Was control-flow/control_flow_inline_override_honored.ph. `Block`'s
        // receiver is always a compiler-materialized block literal (static
        // type, GuardBlock doesn't peek it) — the only thing that can go
        // stale is whether whileTrue itself was redefined. Deliberately
        // returns a Number no correct loop could ever produce, so "the
        // override's return value came through" is unambiguous.
        let mut vm = VM::new();
        let block_class = vm.universe.classes.block_class;
        let while_true_selector = vm.get_or_intern("whileTrue(_)");
        fn returns_sentinel(_vm: &mut VM, _recv: &Value, _args: &[Value]) -> PhResult<Value> {
            Ok(Value::Int(-999))
        }
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            while_true_selector,
            SignatureKind::Method(1),
            MethodKind::Primitive(returns_sentinel),
        ))));
        install_kernel_method(&mut vm, block_class, while_true_selector, method);
        assert!(
            !vm.universe.block_sacred_pristine,
            "installing whileTrue (sacred) must flip the Block guard flag"
        );

        let module = vm.create_module("main", "kernel_block_sacred_override_deopts_inline_while_true");
        let source = "
let i = 0
let r = || { i < 3 }.whileTrue || { i = i + 1 }
";
        let closure = vm.compile_closure(module, source).expect("compiles");
        vm.run_in_module(module, closure).expect("runs on the deopt path");

        assert_eq!(
            read_global(&vm, module, "r"),
            Value::Int(-999),
            "the inlined whileTrue site must deopt to the real send and honor the override, not the fast loop"
        );
    }

    #[test]
    fn kernel_option_match_override_reroutes_every_combinator() {
        // Was absence/absence_combinators_route_through_match.ph. Every
        // `Option` combinator (isSome/isNone/ifNone/orElse) is derived
        // purely over the match(some:none:) eliminator in core.ph — no
        // combinator peeks at a variant tag. `match` is not a sacred
        // selector (no inliner, no pristine flag): this is ordinary
        // world_version-based dispatch invalidation on a kernel class,
        // proving the *routing* property, not a deopt guard.
        let mut vm = VM::new();
        let baseline_module = vm.create_module("main", "kernel_option_match_baseline");
        let baseline_source = "
let baselineSome = Some.new(1).isSome
let baselineNone = None.isNone
";
        let baseline_closure = vm.compile_closure(baseline_module, baseline_source).expect("compiles");
        vm.run_in_module(baseline_module, baseline_closure).expect("runs");
        assert_eq!(read_global(&vm, baseline_module, "baselineSome"), Value::Bool(true));
        assert_eq!(read_global(&vm, baseline_module, "baselineNone"), Value::Bool(true));

        let option_class = vm.universe.classes.option_class;
        let match_selector = vm.get_or_intern("match(some,none)");
        fn always_takes_none_arm(vm: &mut VM, _recv: &Value, args: &[Value]) -> PhResult<Value> {
            crate::primitive::block::block_call(vm, &args[1], &[])
        }
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            match_selector,
            SignatureKind::Method(2),
            MethodKind::Primitive(always_takes_none_arm),
        ))));
        // `match` is not a sacred selector — no `note_method_installed` call
        // needed, but `add_method` + the `world_version` bump alone (not
        // `install_kernel_method`, which also calls it) still busts any
        // warmed cache in core.ph's own combinator bodies.
        let before = vm.world_version;
        vm.heap.class_mut(option_class).add_method(match_selector, method);
        vm.world_version += 1;
        assert_ne!(vm.world_version, before);

        let module = vm.create_module("main", "kernel_option_match_override_reroutes_every_combinator");
        let source = "
let someIsSome = Some.new(1).isSome
let someIsNone = Some.new(1).isNone
let routed = Some.new(1).orElse || { Some.new(9) }.match(some: |v| { v }, none: || { -1 })
";
        let closure = vm.compile_closure(module, source).expect("compiles");
        vm.run_in_module(module, closure).expect("runs");

        assert_eq!(
            read_global(&vm, module, "someIsSome"),
            Value::Bool(false),
            "a real Some forced through the none: arm must report itself absent"
        );
        assert_eq!(read_global(&vm, module, "someIsNone"), Value::Bool(true));
        assert_eq!(
            read_global(&vm, module, "routed"),
            Value::Int(-1),
            "the fixture's own trailing explicit .match(...) call must also route through the override"
        );
    }
}
