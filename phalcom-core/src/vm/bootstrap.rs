use crate::error::PhResult;
use crate::heap::CORE_MODULE_NAME;
use crate::heap::{ClassId, Object};
use crate::universe::Universe;
use crate::value::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use super::{NativeInstallMode, VM};

impl VM {
    /// Creates a new VM: builds the heap, bootstraps the kernel tower, and
    /// installs the core module and native primitives.
    pub fn new() -> Self {
        Self::new_with_native_install_mode(NativeInstallMode::DescriptorOnly)
    }

    /// Creates a VM with an explicit native installation path.
    pub fn new_with_native_install_mode(native_install_mode: NativeInstallMode) -> Self {
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
            compiler_internal_dispatch_depth: 0,
            native_method_contexts: Vec::new(),
            interner,
            reflection_cache: crate::modules::ReflectionCache::new(),
            start_time: Instant::now(),
            module_registry: crate::modules::ModuleRegistry::new(),
            typing_registry: crate::typing::RuntimeTypingRegistry::new(),
            runtime_roots: None,
            privileged_modules: std::collections::HashSet::new(),
            semantic_roots: crate::vm::SemanticRoots {
                unsupported: Value::nil(),
                ellipsis: Value::nil(),
                ordering_class: ClassId::default(),
            },
            classes: HashMap::new(),
            kernel_class_names: std::collections::HashSet::new(),
            prelude_names: std::collections::HashSet::new(),
            universe,
            next_frame_generation: 0,
            // The root fiber's `seq` is hardcoded to 1 (`FiberObject::root`); the
            // first spawned fiber gets 2 (traceback implementation spec §6).
            next_fiber_seq: 2,
            world_version: 0,
            open_upvalues: BTreeMap::new(),
            ready_queue: std::collections::VecDeque::new(),
            temp_roots: Vec::new(),
            field_layouts: HashMap::new(),
            class_parents: HashMap::new(),
            sealed_classes: HashMap::new(),
            checking: std::collections::HashSet::new(),
            compile_mode: crate::compiler::attributes::CompileMode::Debug,
            strip_contract_metadata: false,
            unit_kind: crate::compiler::lib::UnitKind::File,
            trace_core: false,
            trace_format_json: false,
            trace_fibers: false,
            native_selector: None,
            native_class: None,
            resources: crate::resource::ResourceTable::new(),
            strict_resources: false,
            numeric_policy: crate::value::NumericPolicy::standard(),
            adt_registry: crate::adt::RuntimeAdtRegistry::new(),

            #[cfg(feature = "fiber-pool")]
            fiber_pool: Vec::new(),
        };

        // Bootstrap core module and primitive methods
        vm.install_core();

        // Source/native preflight runs before native installation. The parsed
        // units retained by this index are the same units compiled below, so
        // verification and execution cannot observe different source text.
        let source_index = crate::native::NativeSourceIndex::build().expect("canonical universe source must parse");
        let descriptors = crate::native::PRIMITIVES.iter().collect::<Vec<_>>();
        crate::native::verify_native_contracts(&source_index, &descriptors).expect("canonical native source contracts must verify");

        // Populate prelude_names from UNIVERSE_BINDINGS
        for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
            if binding.prelude {
                let sym = vm.interner.intern(binding.name);
                vm.prelude_names.insert(sym);
            }
        }
        let universe_sym = vm.interner.intern("universe");
        vm.prelude_names.insert(universe_sym);
        let none_sym = vm.interner.intern("None");
        vm.prelude_names.insert(none_sym);

        // Initialize canonical builtin 'universe' package with native bindings & exports
        let universe_pkg = crate::modules::builtin_materialize::initialize_canonical_universe(&mut vm).expect("canonical universe package initializes");

        // Expose 'universe' as a global in core module
        let core_module = vm.core_module().expect("core module");
        vm.define_global(core_module, universe_sym, Value::obj(universe_pkg)).unwrap();

        // Stamp the kernel `Message` class's fixed-slot count (U8,
        // method-lookup.md §2). `Message` instances are built
        // directly in Rust ([`VM::new_message`]) — its four slots
        // (selector/name/labels/args) carry no `.ph` field layout, so the
        // count is set here rather than by the compiler's class lowering.
        {
            let message_class = vm.universe.classes.message_class;
            vm.heap.class_mut(message_class).field_count = 4;
        }

        // Stamp the `Error` root's and `MessageNotUnderstood`'s fixed-slot
        // layout (U-CORE-6, ADR-0008/ADR-0011). Like `Message`, both
        // are built directly in Rust — `Error` has one field (`_message`,
        // slot 0); `MessageNotUnderstood < Error` inherits that slot and adds
        // one more (`_reifiedMessage`, slot 1), appended after the
        // superclass's fields per the compiler's field-offset rule
        // (`compiler/lib.rs`), keeping the two rows' slot 0 consistent.
        {
            let error_class = vm.universe.classes.error_class;
            let msg_sym = vm.interner.intern("_message");
            let kind_sym = vm.interner.intern("_kind");
            let cause_sym = vm.interner.intern("_cause");
            let displaced_sym = vm.interner.intern("_displaced");
            vm.heap.class_mut(error_class).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(error_class).field_slots.insert(kind_sym, 1);
            vm.heap.class_mut(error_class).field_slots.insert(cause_sym, 2);
            vm.heap.class_mut(error_class).field_slots.insert(displaced_sym, 3);
            vm.heap.class_mut(error_class).field_count = 4;
        }
        {
            let mnu = vm.universe.classes.message_not_understood_class;
            let msg_sym = vm.interner.intern("_message");
            let kind_sym = vm.interner.intern("_kind");
            let cause_sym = vm.interner.intern("_cause");
            let displaced_sym = vm.interner.intern("_displaced");
            let reified_sym = vm.interner.intern("_reifiedMessage");
            vm.heap.class_mut(mnu).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(mnu).field_slots.insert(kind_sym, 1);
            vm.heap.class_mut(mnu).field_slots.insert(cause_sym, 2);
            vm.heap.class_mut(mnu).field_slots.insert(displaced_sym, 3);
            vm.heap.class_mut(mnu).field_slots.insert(reified_sym, 4);
            vm.heap.class_mut(mnu).field_count = 5;
        }
        // `CannotYieldAcrossNativeFrame < Error` (U-FIBER, D-FIB-1): no
        // fields beyond the inherited slots — mirrors `Error`
        // itself rather than adding anything.
        {
            let cynf = vm.universe.classes.cannot_yield_across_native_frame_class;
            let msg_sym = vm.interner.intern("_message");
            let kind_sym = vm.interner.intern("_kind");
            let cause_sym = vm.interner.intern("_cause");
            let displaced_sym = vm.interner.intern("_displaced");
            vm.heap.class_mut(cynf).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(cynf).field_slots.insert(kind_sym, 1);
            vm.heap.class_mut(cynf).field_slots.insert(cause_sym, 2);
            vm.heap.class_mut(cynf).field_slots.insert(displaced_sym, 3);
            vm.heap.class_mut(cynf).field_count = 4;
        }

        // Resource base class field stamp (U-RESOURCE): slot 0 is packed handle
        {
            let res_class = vm.universe.classes.resource_class;
            let handle_sym = vm.interner.intern("_handle");
            vm.heap.class_mut(res_class).field_slots.insert(handle_sym, 0);
            vm.heap.class_mut(res_class).field_count = 1;
        }

        // UseAfterCloseError < Error
        {
            let uace = vm.universe.classes.use_after_close_error_class;
            let msg_sym = vm.interner.intern("_message");
            let kind_sym = vm.interner.intern("_kind");
            let cause_sym = vm.interner.intern("_cause");
            let displaced_sym = vm.interner.intern("_displaced");
            vm.heap.class_mut(uace).field_slots.insert(msg_sym, 0);
            vm.heap.class_mut(uace).field_slots.insert(kind_sym, 1);
            vm.heap.class_mut(uace).field_slots.insert(cause_sym, 2);
            vm.heap.class_mut(uace).field_slots.insert(displaced_sym, 3);
            vm.heap.class_mut(uace).field_count = 4;
        }
        // Native descriptors are the sole primitive authority. Keep the
        // public mode parameter for callers during the migration, but never
        // reinstall the retired hand-written primitive table.
        let _ = native_install_mode;
        crate::native::install::install_registered_primitives(&mut vm).expect("registered primitives must install cleanly");
        // Typing reflection is a separate semantic subsystem whose classes are
        // not part of the primordial native-surface catalog.
        crate::primitive::typing::install(&mut vm);

        // Finalize every kernel row's base-name index (selectors.md §3.1,
        // U16-Open) now that its native primitives are installed, so `::`
        // works against a kernel class with no `.ph` reopen (e.g. `Behavior`,
        // `Metaclass`, `Message`, `Fiber`) and not only the ones `core.ph`
        // happens to touch. A `.ph` reopen below re-finalizes its own row
        // anyway (`Bytecode::FinalizeClass`, idempotent rebuild), so this
        // pass is never stale — only ever a floor under it.
        vm.finalize_all_core_base_names();

        // Compile and run the registered universe modules now that every native
        // primitive is installed: this is what actually attaches each
        // universe submodule's class-reopen (`List`, `Option`, `Some`, `System`,
        // …) to its bootstrapped kernel row. Must run after
        // `install_primitives` so a reopen can call the primitives it wraps
        // (e.g. `List.at(_:)` calling `at_(_:)`).
        vm.run_universe_modules(&source_index).expect("universe modules must compile and run cleanly");

        // Semantic roots are late-bound to the exact values exported by the
        // universe sources. No Rust replacement is valid for these identities.
        {
            let core = vm.core_module().expect("core module registered by install_core");
            let unsupported = vm
                .heap
                .module(core)
                .get(vm.interner.intern("unsupported"))
                .expect("universe must export canonical unsupported");
            let ellipsis = vm
                .heap
                .module(core)
                .get(vm.interner.intern("ellipsis"))
                .expect("universe must export canonical ellipsis");
            let ordering = vm
                .heap
                .module(core)
                .get(vm.interner.intern("Ordering"))
                .and_then(|value| value.as_obj())
                .expect("universe must export Ordering class");
            vm.semantic_roots = crate::vm::SemanticRoots {
                unsupported,
                ellipsis,
                ordering_class: ordering,
            };
        }

        // Populate prelude_names with universe globals for compatibility,
        // excluding types explicitly prohibited from prelude by Spec §13.1.
        {
            let non_prelude_names: std::collections::HashSet<&str> = ["Behavior", "Metaclass", "Message", "Nil"].into_iter().collect();

            let core_mod = vm.core_module().expect("core module");
            for sym in vm.heap.module(core_mod).name_to_slot.keys().copied().collect::<Vec<_>>() {
                let name = vm.resolve_symbol(sym).to_string();
                if !non_prelude_names.contains(name.as_str()) {
                    vm.prelude_names.insert(sym);
                }
            }
        }

        // Snapshot the leaf `toString` override-epoch flags now that
        // universe reopens (e.g. `String`'s `toString => self`) have
        // already run and legitimately flipped some of them — see
        // `Universe::mark_leaf_tostring_pristine`'s doc for why this must
        // happen exactly here (after bootstrap, before any user code) and
        // not in `Universe::new`.
        vm.universe.mark_leaf_tostring_pristine();

        // R-INV-0.3 (global half) — the `None` **global** resolves to the immediate
        // value, not the `None` class object (ADR-0007/0010). This
        // half needs the core module (its globals table), so it lives here rather
        // than in `verify_invariants`, which is heap-structural (`&Heap` only) and
        // cannot read module globals (U-CORE-1 spec SD-1).
        {
            let core = vm.core_module().expect("core module registered by install_core");
            let none_sym = vm.interner.intern("None");
            let none_value = vm.heap.module(core).get(none_sym).expect("None global must be bound by install_core");
            assert_eq!(none_value, Value::none(), "None global must resolve to immediate absence");
            assert_ne!(
                none_value,
                Value::obj(vm.universe.classes.none_class),
                "None global must not resolve to the None class object"
            );
        }

        vm.universe.verify_invariants(&vm.heap).expect("kernel invariants (object-model.md §5-6)");

        vm
    }

    /// Compiles and runs the universe modules in topological order.
    ///
    /// See the call site in [`Self::new`] for why this must run after
    /// [`Universe::install_primitives`].
    ///
    /// # Errors
    ///
    /// Returns any [`crate::error::PhError`] raised while compiling or executing universe modules.
    fn run_universe_modules(&mut self, source_index: &crate::native::NativeSourceIndex) -> PhResult<()> {
        let module = self.core_module().expect("core module registered by install_core");
        for parsed in &source_index.units {
            let source_id = self.heap.module_mut(module).push_source(std::sync::Arc::new(parsed.text.to_string()));
            let closure = self.compile_ast_as(module, source_id, (*parsed.program).clone(), crate::compiler::lib::UnitKind::File)?;
            self.run_in_module(module, closure)?;
        }
        Ok(())
    }

    /// Bootstraps the core module and exposes each kernel class as a global.
    pub fn install_core(&mut self) {
        let core_id = phalcom_modules::ModuleId::universe_root();
        let m = self.create_module_with_id(core_id, crate::heap::ModuleKind::Module, CORE_MODULE_NAME, "<internal core module>");
        self.runtime_roots = Some(crate::vm::RuntimeRoots { core: m, entry: None });
        self.privileged_modules.insert(m);

        macro_rules! add_class {
            ($field:ident) => {
                let class_id = self.universe.classes.$field;
                let name = self.heap.class(class_id).name.clone();
                let name_sym = self.interner.intern(&name);
                self.define_global(m, name_sym, Value::obj(class_id)).ok();
                let key = crate::vm::ClassKey { module: m, name: name_sym };
                self.classes.insert(key, class_id);
                self.kernel_class_names.insert(name_sym);
            };
        }

        add_class!(object_class);
        add_class!(behavior_class);
        add_class!(class_class);
        add_class!(metaclass_class);
        add_class!(number_class);
        add_class!(int_class);
        add_class!(float_class);
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
        add_class!(selector_class);
        add_class!(selector_pattern_class);
        add_class!(system_class);
        add_class!(function_class);
        add_class!(closure_class);
        add_class!(bound_method_class);
        add_class!(family_class);
        // The callable surface is VM-owned and closed. This protects the
        // common `Function#call...` gateway from user-defined subclasses and
        // keeps all five runtime representations on the sealed hierarchy.
        for class_id in [
            self.universe.classes.function_class,
            self.universe.classes.closure_class,
            self.universe.classes.bound_method_class,
            self.universe.classes.family_class,
            self.universe.classes.method_class,
        ] {
            let name_sym = self.interner.intern(&self.heap.class(class_id).name.clone());
            self.sealed_classes.insert(crate::vm::ClassKey { module: m, name: name_sym }, m);
        }
        // Absence type (ADR-0007/PDR-0033). `Option` and `Some` are ordinary
        // class globals. `None`, however, is a *value* global bound to the
        // immediate variant — not the `None` class — so `None` in source resolves
        // to the immediate value (values-and-absence.md §3.1).
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
            let option_key = crate::vm::ClassKey { module: m, name: option_sym };
            self.sealed_classes.insert(option_key, m);
        }
        add_class!(some_class);
        add_class!(unit_class);
        {
            let some_sym = self.interner.intern(&self.heap.class(self.universe.classes.some_class).name.clone());
            let some_key = crate::vm::ClassKey { module: m, name: some_sym };
            self.sealed_classes.insert(some_key, m);
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
        add_class!(record_class);
        // `Range` (ADR-0039, U-COLLTYPES Phase 3): ordinary class global,
        // native heap arm mirroring `List`.
        add_class!(range_class);
        // `Bytes` (PDR-0011, U-BYTES): ordinary class global, native heap arm
        // mirroring `List`; the core.ph `class Bytes` block is a stub
        // completion of this row, not a fresh class.
        add_class!(bytes_class);
        add_class!(message_class);
        add_class!(attribute_class);
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
        add_class!(package_class);
        add_class!(project_class);
        add_class!(resource_class);
        add_class!(use_after_close_error_class);

        // Typing reflection rows are allocated with the VM universe because
        // native typing primitives need them before source execution. Register
        // those same rows under the core module so canonical `.ph` class
        // presentations complete the preallocated identities instead of
        // allocating shadow classes during universe bootstrap.
        for (name, class_id) in self.universe.typing_classes.iter() {
            let name_sym = self.interner.intern(name);
            let key = crate::vm::ClassKey { module: m, name: name_sym };
            self.classes.insert(key, class_id);
            self.kernel_class_names.insert(name_sym);
        }

        // The `None` class row is *not* exposed under a class global (that name
        // is the singleton), but it must live in `self.classes` so a
        // `class None { ... }` skeleton in `core.ph` reopens this bootstrapped
        // row (compiler `Statement::Class` handling) instead of forging a fresh
        // one that would clobber the `None` global.
        let none_class = self.universe.classes.none_class;
        let none_class_name = self.heap.class(none_class).name.clone();
        let none_class_sym = self.interner.intern(&none_class_name);
        let none_class_key = crate::vm::ClassKey {
            module: m,
            name: none_class_sym,
        };
        self.classes.insert(none_class_key, none_class);
        // `None` bypasses `add_class!` (its global binds the immediate
        // value, not the class), so it must be reserved (ruling 3) here
        // explicitly rather than falling out of that macro.
        self.kernel_class_names.insert(none_class_sym);

        // `Nil` is the private tagged-value sentinel class. It gets a source
        // presentation for census parity, but must not acquire a public class
        // global when its source module is compiled.
        let nil_class = self.universe.classes.nil_class;
        let nil_class_sym = self.interner.intern(&self.heap.class(nil_class).name.clone());
        let nil_class_key = crate::vm::ClassKey {
            module: m,
            name: nil_class_sym,
        };
        self.classes.insert(nil_class_key, nil_class);
        self.kernel_class_names.insert(nil_class_sym);
        // Seal `None` to the core module too (see the sealing note above the
        // `Option`/`Some` rows): `class MyNone is None {}` in user code
        // must raise `attr.sealed_violation` the same as the other two.
        let none_class_key_sealed = crate::vm::ClassKey {
            module: m,
            name: none_class_sym,
        };
        self.sealed_classes.insert(none_class_key_sealed, m);

        // Bind the `None` global to immediate absence.
        let none_global_sym = self.interner.intern("None");
        self.define_global(m, none_global_sym, Value::none()).ok();

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
        let rows = [
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
            c.closure_class,
            c.bound_method_class,
            c.symbol_class,
            c.selector_pattern_class,
            c.module_class,
            c.package_class,
            c.project_class,
            c.system_class,
            c.option_class,
            c.some_class,
            c.none_class,
            c.unit_class,
            c.list_class,
            c.map_class,
            c.set_class,
            c.tuple_class,
            c.record_class,
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

    /// Allocates a builtin package with `logical_name` and registers it in `vm.module_registry`.
    pub fn create_builtin_package(&mut self, logical_name: &str) -> crate::heap::ObjRef {
        if logical_name == "universe" {
            let id = phalcom_modules::ModuleId::universe_root();
            if let Some(rec) = self.module_registry.get(&id) {
                return rec.object;
            }
        }
        let mut ids = phalcom_modules::SyntheticProjectIdAllocator;
        let path = phalcom_modules::ModuleComponent::from_identifier(logical_name)
            .map(|c| phalcom_modules::ModulePath::from_components(vec![c]))
            .unwrap_or_else(|_| phalcom_modules::ModulePath::root());
        let id = phalcom_modules::ModuleId::synthetic(ids.allocate(), path);
        let kind = crate::heap::ModuleKind::Package;
        self.create_module_with_id(id, kind, logical_name, &format!("<builtin:{logical_name}>"))
    }
}
