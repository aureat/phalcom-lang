use crate::error::PhResult;
use crate::heap::{ClassId, Object};
use crate::heap::CORE_MODULE_NAME;
use crate::universe::Universe;
use crate::value::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use super::VM;

impl VM {
    /// Creates a new VM: builds the heap, bootstraps the kernel tower, and
    /// installs the core module and native primitives.
    pub fn new() -> Self {
        let interner = crate::interner::Interner::with_capacity(100);
        let mut heap = crate::heap::Heap::new();
        let universe = Universe::new(&mut heap);
        // The root fiber (ADR-0030 §1): already `Running`, no entry — its
        // live state is `VM::frames`/`stack`/`open_upvalues` from the start,
        // so this alloc is bookkeeping only, not a behavior change (D-FIB-4,
        // Phase 1 is a pure refactor).
        let current = heap.alloc(Object::Fiber(Box::new(crate::heap::FiberObject::root())));

        let mut vm = Self {
            heap,
            frames: Vec::with_capacity(256),
            stack: Vec::with_capacity(1024),
            current,
            switch_pending: false,
            native_reentry_depth: 0,
            interner,
            start_time: Instant::now(),
            modules: HashMap::new(),
            main_module: None,
            last_imported_module: None,
            classes: HashMap::new(),
            universe,
            next_frame_generation: 0,
            world_version: 0,
            open_upvalues: BTreeMap::new(),
            ready_queue: std::collections::VecDeque::new(),
            temp_roots: Vec::new(),
            field_layouts: HashMap::new(),
            constructor_aliases: HashMap::new(),
            has_new_construct: std::collections::HashSet::new(),
            class_parents: HashMap::new(),
            sealed_classes: HashMap::new(),
            checking: std::collections::HashSet::new(),
            compile_mode: crate::compiler::attributes::CompileMode::Debug,
            strip_contract_metadata: false,
            init_selector_cache: HashMap::new(),
            variadic_selector_cache: HashMap::new(),
            #[cfg(feature = "fiber-pool")]
            fiber_pool: Vec::new(),
        };

        // Bootstrap core module and primitive methods
        vm.install_core();

        // Initialize Some class field layout (ADR-0011)
        {
            let some_class = vm.universe.classes.some_class;
            let value_sym = vm.interner.intern("_value");
            vm.heap.class_mut(some_class).field_slots.insert(value_sym, 0);
            vm.heap.class_mut(some_class).field_count = 1;
        }

        // Stamp the kernel `Message` class's fixed-slot count (U8,
        // method-lookup.md §2). Like `Some`, `Message` instances are built
        // directly in Rust ([`VM::new_message`]) — its four slots
        // (selector/name/labels/args) carry no `.ph` field layout, so the
        // count is set here rather than by the compiler's class lowering.
        {
            let message_class = vm.universe.classes.message_class;
            vm.heap.class_mut(message_class).field_count = 4;
        }

        // Stamp the `Error` root's and `MessageNotUnderstood`'s fixed-slot
        // layout (U-CORE-6, ADR-0008/ADR-0011). Like `Some`/`Message`, both
        // are built directly in Rust — `Error` has one field (`_message`,
        // slot 0); `MessageNotUnderstood < Error` inherits that slot and adds
        // one more (`_reifiedMessage`, slot 1), appended after the
        // superclass's fields per the compiler's field-offset rule
        // (`compiler/lib.rs`), keeping the two rows' slot 0 consistent.
        {
            let error_class = vm.universe.classes.error_class;
            let msg_sym = vm.interner.intern("_message");
            vm.heap.class_mut(error_class).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(error_class).field_count = 1;
        }
        {
            let mnu = vm.universe.classes.message_not_understood_class;
            let msg_sym = vm.interner.intern("_message");
            let reified_sym = vm.interner.intern("_reifiedMessage");
            vm.heap.class_mut(mnu).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(mnu).field_slots.insert(reified_sym, 1);
            vm.heap.class_mut(mnu).field_count = 2;
        }
        // `CannotYieldAcrossNativeFrame < Error` (U-FIBER, D-FIB-1): no
        // fields beyond the inherited `_message` slot 0 — mirrors `Error`
        // itself rather than adding anything.
        {
            let cynf = vm.universe.classes.cannot_yield_across_native_frame_class;
            let msg_sym = vm.interner.intern("_message");
            vm.heap.class_mut(cynf).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(cynf).field_count = 1;
        }
        Universe::install_primitives(&mut vm);

        // Finalize every kernel row's base-name index (selectors.md §3.1,
        // U16-Open) now that its native primitives are installed, so `::`
        // works against a kernel class with no `.ph` reopen (e.g. `Behavior`,
        // `Metaclass`, `Message`, `Fiber`) and not only the ones `core.ph`
        // happens to touch. A `.ph` reopen below re-finalizes its own row
        // anyway (`Bytecode::FinalizeClass`, idempotent rebuild), so this
        // pass is never stale — only ever a floor under it.
        vm.finalize_all_core_base_names();

        // Compile and run the registered core module now that every native
        // primitive is installed: this is what actually attaches each
        // `core.ph` class-reopen (`List`, `Option`, `Some`, `None`, `System`,
        // …) to its bootstrapped kernel row. Must run after
        // `install_primitives` so a reopen can call the primitives it wraps
        // (e.g. `List.at(_:)` calling `at_(_:)`). Previously `install_core`
        // only registered the source text (for diagnostics) without ever
        // compiling or executing it, so every `.ph` skeleton was inert; U-LIST
        // is the first unit whose surface protocol actually depends on a
        // reopen taking effect, which surfaced the gap.
        vm.run_core_module().expect("core module (core.ph) must compile and run cleanly");

        // Snapshot the leaf `toString` override-epoch flags now that
        // `core.ph`'s own reopens (e.g. `String`'s `toString => self`) have
        // already run and legitimately flipped some of them — see
        // `Universe::mark_leaf_tostring_pristine`'s doc for why this must
        // happen exactly here (after bootstrap, before any user code) and
        // not in `Universe::new`.
        vm.universe.mark_leaf_tostring_pristine();

        // R-INV-0.3 (global half) — the `None` **global** resolves to the shared
        // singleton *value*, not the `None` class object (ADR-0007/0010). This
        // half needs the core module (its globals table), so it lives here rather
        // than in `verify_invariants`, which is heap-structural (`&Heap` only) and
        // cannot read module globals (U-CORE-1 spec SD-1).
        {
            let core = vm.get_module_from_str(CORE_MODULE_NAME).expect("core module registered by install_core");
            let none_sym = vm.interner.intern("None");
            let none_value = vm.heap.module(core).get(none_sym).expect("None global must be bound by install_core");
            assert!(matches!(none_value, Value::Obj(id) if id == vm.universe.classes.none_singleton), "None global must resolve to the shared singleton value, not the None class");
            assert_ne!(none_value, Value::Obj(vm.universe.classes.none_class), "None global must not resolve to the None class object");
        }

        vm.universe
            .verify_invariants(&vm.heap)
            .expect("kernel invariants (object-model.md §5-6)");

        vm
    }

    /// Compiles and runs the registered core module (`core.ph`).
    ///
    /// See the call site in [`Self::new`] for why this must run after
    /// [`Universe::install_primitives`].
    ///
    /// # Errors
    ///
    /// Returns any [`crate::error::PhError`] raised while compiling or executing `core.ph`.
    fn run_core_module(&mut self) -> PhResult<()> {
        let module = self.get_module_from_str(CORE_MODULE_NAME).expect("core module registered by install_core");
        let source = include_str!("../../core/core.ph");
        let closure = self.compile_closure(module, source)?;
        self.run_in_module(module, closure)
    }

    /// Bootstraps the core module and exposes each kernel class as a global.
    pub fn install_core(&mut self) {
        let m = self.create_module(CORE_MODULE_NAME, "<internal core module>");
        self.register_source(CORE_MODULE_NAME, include_str!("../../core/core.ph"));
        let core_sym = self.heap.module(m).symbol();
        self.modules.insert(core_sym, m);

        macro_rules! add_class {
            ($field:ident) => {
                let class_id = self.universe.classes.$field;
                let name = self.heap.class(class_id).name.clone();
                let name_sym = self.interner.intern(&name);
                self.define_global(core_sym, name_sym, Value::Obj(class_id)).ok();
                self.classes.insert(name_sym, class_id);
            };
        }

        add_class!(object_class);
        add_class!(behavior_class);
        add_class!(class_class);
        add_class!(metaclass_class);
        add_class!(number_class);
        add_class!(string_class);
        add_class!(bool_class);
        // The boolean tower (ADR-0004): `True`/`False` are ordinary class
        // globals — unlike `None`, their global names bind to the class objects
        // (not to a singleton value), so a `class True {}` / `class False {}`
        // reopen in core.ph re-emits the identical `DefineGlobal` binding (a
        // harmless no-op) and resolves the bootstrapped `self.classes` rows.
        add_class!(true_class);
        add_class!(false_class);
        add_class!(method_class);
        add_class!(symbol_class);
        add_class!(system_class);
        add_class!(function_class);
        add_class!(block_class);
        // Absence type (ADR-0007). `Option` and `Some` are ordinary class
        // globals. `None`, however, is a *value* global bound to the shared
        // singleton — not the `None` class — so `None` in source resolves to the
        // singleton object (values-and-absence.md §3.1).
        //
        // All three (`Option`/`Some`/`None`) are sealed to the core module at
        // bootstrap (U-ANNOT-LAYOUT §3.4, `attr.sealed_violation`): user `.ph`
        // code must not extend them. This is registered directly in
        // `self.sealed_classes` here (rather than via the `@sealed` decorator)
        // because `None` has no `.ph` class reopen to carry the annotation —
        // see the singleton-binding note below.
        add_class!(option_class);
        {
            let option_sym = self.interner.intern(&self.heap.class(self.universe.classes.option_class).name.clone());
            self.sealed_classes.insert(option_sym, m);
        }
        add_class!(some_class);
        {
            let some_sym = self.interner.intern(&self.heap.class(self.universe.classes.some_class).name.clone());
            self.sealed_classes.insert(some_sym, m);
        }
        add_class!(iterable_class);
        add_class!(list_class);
        // `Map`/`Set` (ADR-0039, U-COLLTYPES Phase 1): ordinary class globals,
        // native heap arms mirroring `List`.
        add_class!(map_class);
        add_class!(set_class);
        // `Tuple` (ADR-0039, U-COLLTYPES Phase 2): ordinary class global,
        // native heap arm mirroring `List`.
        add_class!(tuple_class);
        // `Range` (ADR-0039, U-COLLTYPES Phase 3): ordinary class global,
        // native heap arm mirroring `List`.
        add_class!(range_class);
        add_class!(message_class);
        // `Error` root + `MessageNotUnderstood < Error` (U-CORE-6, ADR-0008):
        // globals only, no `.ph` reopen — an empty reopen would be harmless
        // (like `Message`'s) but is skipped as unnecessary; a reopen with a
        // body that *reads* `_message` would trip the read-before-write check.
        add_class!(error_class);
        add_class!(message_not_understood_class);
        // `Fiber` (ADR-0030, U-FIBER floor extension) + its restricted-yield
        // guard error, both ordinary class globals.
        add_class!(fiber_class);
        add_class!(cannot_yield_across_native_frame_class);
        // `Family` (selectors.md §3, U16-Open, ADR-0047): ordinary class
        // global, native heap arm mirroring `Fiber`/`List`.
        add_class!(family_class);

        // The `None` class row is *not* exposed under a class global (that name
        // is the singleton), but it must live in `self.classes` so a
        // `class None { ... }` skeleton in `core.ph` reopens this bootstrapped
        // row (compiler `Statement::Class` handling) instead of forging a fresh
        // one that would clobber the `None` global.
        let none_class = self.universe.classes.none_class;
        let none_class_name = self.heap.class(none_class).name.clone();
        let none_class_sym = self.interner.intern(&none_class_name);
        self.classes.insert(none_class_sym, none_class);
        // Seal `None` to the core module too (see the sealing note above the
        // `Option`/`Some` rows): `class MyNone extends None {}` in user code
        // must raise `attr.sealed_violation` the same as the other two.
        self.sealed_classes.insert(none_class_sym, m);

        // Bind the `None` global to the shared singleton object.
        let none_global_sym = self.interner.intern("None");
        let none_value = Value::Obj(self.universe.classes.none_singleton);
        self.define_global(core_sym, none_global_sym, none_value).ok();

        // The private `nil` sentinel has no surface class global: there is no
        // `Nil` name reachable from user code (Invariant 4). The `Nil` class row
        // still exists in the tower to back `Value::Nil::class`, but it is
        // internal only.
    }

    /// Finalizes every kernel class row's (and its metaclass's) base-name
    /// index (selectors.md §3.1, U16-Open) right after
    /// [`Universe::install_primitives`] wires up the native floor.
    ///
    /// The list is in dependency order (each row's superclass appears
    /// earlier), matching [`Self::finalize_class_base_names`]'s precondition.
    /// A later `.ph` reopen in `core.ph` (or user code) re-finalizes its own
    /// row idempotently via [`crate::bytecode::Bytecode::FinalizeClass`] —
    /// this pass only guarantees every row has *some* finalized index, even
    /// one `core.ph` never touches (`Behavior`, `Metaclass`, `Message`,
    /// `Fiber`, …).
    fn finalize_all_core_base_names(&mut self) {
        let c = self.universe.classes;
        let rows: [ClassId; 30] = [
            c.object_class,
            c.behavior_class,
            c.class_class,
            c.metaclass_class,
            c.number_class,
            c.string_class,
            c.nil_class,
            c.bool_class,
            c.true_class,
            c.false_class,
            c.method_class,
            c.function_class,
            c.block_class,
            c.symbol_class,
            c.module_class,
            c.system_class,
            c.option_class,
            c.some_class,
            c.none_class,
            c.list_class,
            c.map_class,
            c.set_class,
            c.tuple_class,
            c.range_class,
            c.message_class,
            c.error_class,
            c.message_not_understood_class,
            c.fiber_class,
            c.cannot_yield_across_native_frame_class,
            c.family_class,
        ];
        for class_id in rows {
            self.finalize_class_base_names(class_id);
            let meta_id = self.heap.class(class_id).class;
            self.finalize_class_base_names(meta_id);
        }
    }
}
