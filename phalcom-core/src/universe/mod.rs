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
            symbol_tostring_pristine: false,
            str_tostring_pristine: false,
            module_registry: HashMap::new(),
        }
    }

    /// The `Bool`-receiver sacred selectors watched by
    /// [`Universe::bool_sacred_pristine`]
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    pub const BOOL_SACRED_SELECTORS: &'static [&'static str] =
        &["and(_)", "or(_)", "not()", "ifTrue(_)", "ifFalse(_)", "ifTrue(_,ifFalse)"];

    /// The `Block`-receiver sacred selectors watched by
    /// [`Universe::block_sacred_pristine`]
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    pub const BLOCK_SACRED_SELECTORS: &'static [&'static str] = &["whileTrue(_)"];

    /// The `toString` getter selector watched by
    /// [`Universe::number_tostring_pristine`]/
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
        self.symbol_tostring_pristine = true;
        self.str_tostring_pristine = true;
    }
}
