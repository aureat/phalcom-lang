use crate::error::PhResult;
use crate::heap::Object;
use crate::universe::Universe;
use crate::value::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

#[cfg(test)]
use std::cell::Cell;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{NativeInstallMode, VM};

#[cfg(test)]
pub(crate) static UNIVERSE_AST_COMPILATIONS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static UNIVERSE_INITIALIZER_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
thread_local! {
    static COUNT_UNIVERSE_BOOTSTRAP: Cell<bool> = const { Cell::new(false) };
}

impl VM {
    /// Creates a new VM: builds the heap, bootstraps the kernel tower, and
    /// installs the Universe package and native primitives.
    pub fn new() -> Self {
        Self::new_with_native_install_mode(NativeInstallMode::DescriptorOnly)
    }

    /// Creates a fresh VM execution kernel without native or source Universe
    /// bootstrap.
    pub fn new_kernel() -> Self {
        let interner = crate::interner::Interner::with_capacity(100);
        let mut heap = crate::heap::Heap::new();
        let universe = Universe::new(&mut heap);
        // The root fiber (ADR-0030 §1): already `Running`, no entry — its
        // live state is `VM::frames`/`stack`/`open_upvalues` from the start,
        // so this alloc is bookkeeping only, not a behavior change (D-FIB-4,
        // Phase 1 is a pure refactor).
        let current = heap.alloc(Object::Fiber(Box::new(crate::heap::FiberObject::root())));

        Self {
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
            universe_bootstrap_measurement: crate::vm::UniverseBootstrapMeasurement::default(),
            privileged_modules: std::collections::HashSet::new(),
            semantic_roots: None,
            classes: HashMap::new(),
            kernel_class_names: std::collections::HashSet::new(),
            prelude_bindings: HashMap::new(),
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
        }
    }

    /// Creates a fresh VM with the native runtime floor, without executing
    /// source-authored Universe modules.
    pub fn new_native() -> Self {
        Self::new_native_with_native_install_mode(NativeInstallMode::DescriptorOnly)
    }

    fn new_native_with_native_install_mode(native_install_mode: NativeInstallMode) -> Self {
        let mut vm = Self::new_kernel();
        Self::install_native_runtime(&mut vm, native_install_mode);
        vm
    }

    /// Creates a VM with an explicit native installation path.
    pub fn new_with_native_install_mode(native_install_mode: NativeInstallMode) -> Self {
        // Canonical source/native verification, linking, semantic analysis, and
        // lowering are process-shared. Runtime installation remains fresh.
        let canonical = crate::modules::canonical_universe_program().expect("canonical Universe compiler product must build");
        let mut vm = Self::new_native_with_native_install_mode(native_install_mode);

        // Compile and run the registered universe modules now that every native
        // primitive is installed: this is what actually attaches each
        // universe submodule's class-reopen (`List`, `Option`, `Some`, `System`,
        // …) to its bootstrapped kernel row. Must run after
        // `install_primitives` so a reopen can call the primitives it wraps
        // (e.g. `List.at(_:)` calling `at_(_:)`).
        vm.run_universe_modules(canonical).expect("universe modules must compile and run cleanly");
        vm.sync_universe_class_aliases();

        // Semantic roots are late-bound to the exact values exported by the
        // universe sources. No Rust replacement is valid for these identities.
        {
            let unsupported = vm
                .universe_global(&["errors", "unsupported"], "unsupported")
                .expect("universe must export canonical unsupported");
            let ellipsis = vm
                .universe_global(&["object", "ellipsis"], "ellipsis")
                .expect("universe must export canonical ellipsis");
            let ordering = vm
                .universe_global(&["object", "ordering"], "Ordering")
                .and_then(|value| value.as_obj())
                .expect("universe must export Ordering class");
            vm.semantic_roots = Some(crate::vm::SemanticRoots {
                unsupported,
                ellipsis,
                ordering_class: ordering,
            });
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
        // half needs the Universe module globals table, so it lives here rather
        // than in `verify_invariants`, which is heap-structural (`&Heap` only) and
        // cannot read module globals (U-CORE-1 spec SD-1).
        {
            let none_value = vm
                .universe_global(&["option", "option"], "None")
                .expect("None global must be bound by canonical Option module");
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

    fn install_native_runtime(vm: &mut Self, native_install_mode: NativeInstallMode) {
        let universe_sym = vm.interner.intern("universe");

        // Initialize canonical builtin 'universe' package with native bindings & exports.
        let universe_pkg = crate::modules::builtin_materialize::initialize_canonical_universe(vm).expect("canonical universe package initializes");
        vm.define_global(universe_pkg, universe_sym, Value::obj(universe_pkg)).unwrap();
        // Bind primordial classes into their canonical modules and retain root
        // aliases for source prelude compatibility.
        vm.bind_primordial_universe();
        vm.sync_universe_class_aliases();

        // Stamp the kernel `Message` class's fixed-slot count (U8,
        // method-lookup.md §2). `Message` instances are built directly in Rust
        // and carry no `.ph` field layout.
        {
            let message_class = vm.universe.classes.message_class;
            vm.heap.class_mut(message_class).field_count = 4;
        }

        // Stamp fixed layouts for Rust-built error classes. Their fields follow
        // the same inherited slot order as compiler-produced class layouts.
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
        // `CannotYieldAcrossNativeFrame < Error` has no fields beyond its
        // inherited error layout.
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

        // Resource base class field stamp (U-RESOURCE): slot 0 is packed handle.
        {
            let res_class = vm.universe.classes.resource_class;
            let handle_sym = vm.interner.intern("_handle");
            vm.heap.class_mut(res_class).field_slots.insert(handle_sym, 0);
            vm.heap.class_mut(res_class).field_count = 1;
        }

        // UseAfterCloseError < Error.
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

        // Native descriptors are the sole primitive authority. Keep the public
        // mode parameter for callers during the migration, but never reinstall
        // the retired hand-written primitive table.
        let _ = native_install_mode;
        crate::native::install::install_registered_primitives(vm).expect("registered primitives must install cleanly");
        // Typing reflection is separate from the primordial native-surface catalog.
        crate::primitive::typing::install(vm);

        // Establish the native floor for base-name dispatch. Source reopens
        // below may finalize their own rows again.
        vm.finalize_all_primordial_base_names();
    }

    fn compile_universe_module(
        &mut self,
        canonical: &crate::modules::CanonicalUniverseProgram,
        id: &phalcom_modules::ModuleId,
    ) -> PhResult<crate::heap::ObjRef> {
        let parsed = canonical
            .source_index()
            .unit(id)
            .ok_or_else(|| crate::error::RuntimeError::Internal(format!("Universe source module {id} is missing from canonical index")))?;
        let compiled = canonical
            .program()
            .modules
            .get(id)
            .ok_or_else(|| crate::error::RuntimeError::Internal(format!("Universe compiled module {id} is missing from canonical program")))?;
        let linked = canonical
            .program()
            .linked
            .modules
            .get(id)
            .ok_or_else(|| crate::error::RuntimeError::Internal(format!("Universe linked module {id} is missing from canonical program")))?;
        let module = self
            .module_registry
            .get(id)
            .ok_or_else(|| crate::error::RuntimeError::Internal(format!("Universe module {id} is not materialized")))?
            .object;

        self.heap.module_mut(module).lowering = Some(compiled.lowering.clone());
        self.materialize_linked_reads_for_module(id, compiled)?;
        let bindings = crate::modules::CompileBindings::from_linked_module(linked);
        let source_id = self.heap.module_mut(module).push_source(std::sync::Arc::new(parsed.text.to_string()));
        #[cfg(test)]
        if COUNT_UNIVERSE_BOOTSTRAP.with(Cell::get) {
            UNIVERSE_AST_COMPILATIONS.fetch_add(1, Ordering::Relaxed);
        }
        Ok(self
            .compile_ast_as_with_bindings(
                module,
                source_id,
                (*parsed.program).clone(),
                crate::compiler::lib::UnitKind::File,
                Some(bindings),
            )
            .map_err(|error| crate::error::RuntimeError::Internal(format!("failed to compile Universe module {id}: {error}")))?)
    }

    /// Compiles and runs the precomputed Universe modules in topological order.
    ///
    /// See the call site in [`Self::new`] for why this must run after
    /// [`Universe::install_primitives`].
    ///
    /// # Errors
    ///
    /// Returns any [`crate::error::PhError`] raised while compiling or executing universe modules.
    fn run_universe_modules(&mut self, canonical: &crate::modules::CanonicalUniverseProgram) -> PhResult<()> {
        self.universe_bootstrap_measurement = crate::vm::UniverseBootstrapMeasurement {
            discovered_units: canonical.source_index().units.len(),
            root_reachable_units: canonical.root_reachable().len(),
            executed_units: canonical.bootstrap_order().len(),
        };
        for id in canonical.bootstrap_order().iter() {
            let closure = self.compile_universe_module(canonical, id)?;
            let module = self
                .module_registry
                .get(id)
                .ok_or_else(|| crate::error::RuntimeError::Internal(format!("Universe module {id} is not materialized")))?
                .object;
            #[cfg(test)]
            if COUNT_UNIVERSE_BOOTSTRAP.with(Cell::get) {
                UNIVERSE_INITIALIZER_EXECUTIONS.fetch_add(1, Ordering::Relaxed);
            }
            self.run_in_module(module, closure)?;
            self.module_registry.get_mut(id).expect("bootstrapped Universe module is registered").state = crate::modules::registry::ModuleState::Initialized;
        }
        Ok(())
    }

    /// Binds primordial runtime classes to canonical Universe modules and
    /// exposes root package aliases for compatibility.
    pub fn bind_primordial_universe(&mut self) {
        let m = self.universe_root_module();
        self.runtime_roots = Some(crate::vm::RuntimeRoots { universe: m, entry: None });
        self.privileged_modules.extend(self.module_registry.iter().map(|(_, record)| record.object));

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
        // All three (`Option`/`Some`/`None`) are sealed to the Universe package at
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
        add_class!(result_class);
        add_class!(ordering_class);
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
        // those same rows under the Universe root so canonical `.ph` class
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
        // Seal `None` to the Universe package too (see the sealing note above the
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

        // The root aliases above preserve bare prelude reads. The authoritative
        // ClassKey for every primordial declaration belongs to its source module.
        for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
            let owner_id = Self::canonical_universe_module_id(binding.key);
            let owner = self.module_registry.get(&owner_id).expect("canonical Universe owner module").object;
            let name_sym = self.interner.intern(binding.name);
            let class_id = self.universe.classes.resolve(binding.key);
            self.classes.insert(crate::vm::ClassKey { module: owner, name: name_sym }, class_id);
            self.kernel_class_names.insert(name_sym);
        }

        for key in [
            phalcom_native_meta::UniverseKey::Option,
            phalcom_native_meta::UniverseKey::Some,
            phalcom_native_meta::UniverseKey::None,
        ] {
            let owner_id = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(
                key.source_path()
                    .iter()
                    .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                    .collect::<Vec<_>>(),
            ));
            let owner = self.module_registry.get(&owner_id).expect("canonical Option owner module").object;
            let name = self.interner.intern(key.name());
            self.sealed_classes.insert(crate::vm::ClassKey { module: owner, name }, m);
        }

        // The private `nil` sentinel has no surface class global: there is no
        // `Nil` name reachable from user code (Invariant 4). The `Nil` class row
        // still exists in the tower to back `Value::Nil::class`, but it is
        // internal only.
    }

    fn sync_universe_class_aliases(&mut self) {
        let root = self.universe_root_module();
        self.prelude_bindings.clear();
        for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
            let owner_id = Self::canonical_universe_module_id(binding.key);
            let owner = self.module_registry.get(&owner_id).expect("canonical Universe owner module").object;
            let name = self.interner.intern(binding.name);
            let value = if binding.key == phalcom_native_meta::UniverseKey::None {
                Value::obj(self.universe.classes.none_class)
            } else {
                self.heap
                    .module(owner)
                    .get(name)
                    .unwrap_or_else(|| Value::obj(self.universe.classes.resolve(binding.key)))
            };
            let slot = self.heap.module_mut(root).declare(name).expect("Universe root alias slot");
            self.heap.module_mut(root).set_global(slot, value).expect("Universe root alias value");
            if binding.prelude || matches!(binding.key, phalcom_native_meta::UniverseKey::Some | phalcom_native_meta::UniverseKey::None) {
                self.prelude_bindings.insert(
                    name,
                    crate::modules::BindingRef {
                        module: owner,
                        slot: u16::try_from(self.heap.module(owner).slot_of(name).expect("canonical Universe binding slot")).expect("Universe slot fits u16"),
                    },
                );
            }
        }

        // Source-authored class declarations keep their defining module
        // identity, but remain available through bare prelude reads. Record
        // only class declarations here; package children and context
        // intrinsics stay reachable through qualified module paths.
        let universe_modules = self
            .module_registry
            .iter()
            .filter_map(|(id, record)| id.project.is_universe().then_some(record.object))
            .collect::<Vec<_>>();
        let native_names = phalcom_native_meta::UNIVERSE_BINDINGS
            .iter()
            .map(|binding| (binding.name, binding.prelude))
            .collect::<HashMap<_, _>>();
        let source_bindings = self
            .classes
            .keys()
            .filter(|key| key.module != root)
            .filter(|key| universe_modules.contains(&key.module))
            .filter_map(|key| {
                let name_text = self.resolve_symbol(key.name);
                if let Some(&is_prelude) = native_names.get(name_text) {
                    if !is_prelude && !matches!(name_text, "Some" | "None") {
                        return None;
                    }
                }
                let slot = self.heap.module(key.module).slot_of(key.name)?;
                Some((
                    key.name,
                    crate::modules::BindingRef {
                        module: key.module,
                        slot: u16::try_from(slot).ok()?,
                    },
                ))
            })
            .collect::<Vec<_>>();
        for (name, binding) in source_bindings {
            self.prelude_bindings.insert(name, binding);
        }

        // The root package itself is a linked prelude target for explicit
        // `universe` reads, while its child package names are not prelude
        // bindings.
        let universe_name = self.interner.intern("universe");
        let universe_slot = self.heap.module(root).slot_of(universe_name);
        if let Some(slot) = universe_slot {
            self.prelude_bindings.insert(
                universe_name,
                crate::modules::BindingRef {
                    module: root,
                    slot: u16::try_from(slot).expect("Universe root slot fits u16"),
                },
            );
        }
    }

    /// Finalizes every primordial class row's (and its metaclass's) base-name
    /// index (selectors.md §3.1, U16-Open) right after
    /// [`Universe::install_primitives`] wires up the native floor.
    ///
    /// The list is in dependency order (each row's superclass appears
    /// earlier), matching [`Self::finalize_class_base_names`]'s precondition.
    /// A later `.ph` reopen in a Universe source (or user code) re-finalizes its own
    /// row idempotently via [`crate::bytecode::Bytecode::FinalizeClass`] —
    /// this pass only guarantees every row has *some* finalized index, even
    /// one Universe source never touches (`Behavior`, `Metaclass`, `Message`,
    /// `Fiber`, …).
    fn finalize_all_primordial_base_names(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::{UNIVERSE_AST_COMPILATIONS, UNIVERSE_INITIALIZER_EXECUTIONS, VM};
    use crate::modules::RuntimeLinkedRead;
    use std::sync::atomic::Ordering;

    struct UniverseBootstrapCounterScope;

    impl UniverseBootstrapCounterScope {
        fn enter() -> Self {
            super::COUNT_UNIVERSE_BOOTSTRAP.with(|enabled| enabled.set(true));
            Self
        }
    }

    impl Drop for UniverseBootstrapCounterScope {
        fn drop(&mut self) {
            super::COUNT_UNIVERSE_BOOTSTRAP.with(|enabled| enabled.set(false));
        }
    }

    #[test]
    fn kernel_bootstrap_has_no_native_or_source_state() {
        let vm = VM::new_kernel();

        assert!(vm.runtime_roots.is_none());
        assert!(vm.semantic_roots.is_none());
        assert!(vm.classes.is_empty());
        assert!(vm.module_registry.iter().next().is_none());
        assert_eq!(vm.universe_bootstrap_measurement(), super::super::UniverseBootstrapMeasurement::default());
    }

    #[test]
    fn native_bootstrap_has_native_floor_without_source_semantic_roots() {
        let vm = VM::new_native();

        assert!(vm.runtime_roots.is_some());
        assert!(vm.semantic_roots.is_none());
        assert!(vm.module_registry.iter().next().is_some());
        assert_eq!(vm.universe_bootstrap_measurement(), super::super::UniverseBootstrapMeasurement::default());
    }

    #[test]
    fn lower_tier_source_root_access_is_explicitly_rejected() {
        let vm = VM::new_native();
        let error = vm.require_semantic_roots().expect_err("native VM must not expose source semantic roots");

        assert!(error.to_string().contains("source-authored Universe bootstrap"));
    }

    #[test]
    fn full_vms_keep_mutable_module_state_isolated() {
        let mut first = VM::new();
        let mut second = VM::new();
        let module = first.create_module("bootstrap_isolation_probe", "<test>");
        let symbol = first.interner.intern("value");

        first
            .define_global(module, symbol, crate::value::Value::int(42))
            .expect("probe global definition");
        assert_eq!(first.heap.module(module).get(symbol), Some(crate::value::Value::int(42)));
        let second_symbol = second.interner.intern("bootstrap_isolation_probe");
        assert!(second.find_module_by_symbol(second_symbol).is_none());
    }

    #[test]
    fn full_vm_publishes_semantic_roots_only_after_source_bootstrap() {
        let vm = VM::new();

        assert!(vm.semantic_roots.is_some());
        let roots = vm.require_semantic_roots().expect("full VM owns semantic roots");
        assert!(
            roots
                .unsupported
                .same_as(&vm.universe_global(&["errors", "unsupported"], "unsupported").unwrap())
        );
    }

    #[test]
    fn canonical_linked_prefix_matches_runtime_materialization() {
        let canonical = crate::modules::canonical_universe_program().expect("canonical Universe compiler product");
        let (id, linked) = canonical
            .program()
            .linked
            .modules
            .iter()
            .find(|(_, module)| !module.bindings.imports.is_empty())
            .expect("Universe must contain a linked import");
        let expected_reads = linked.linked_reads.clone();
        let mut vm = VM::new_native();
        let _closure = vm
            .compile_universe_module(canonical, id)
            .expect("canonical module compiles with linked bindings");
        let runtime = vm.module_registry.get(id).expect("canonical runtime module").object;
        let actual_reads = vm.heap.module(runtime).linked_reads.clone();

        assert!(actual_reads.len() >= expected_reads.len());
        for (expected, actual) in expected_reads.iter().zip(actual_reads.iter()) {
            match (expected, actual) {
                (phalcom_modules::LinkedReadSpec::Module(target), RuntimeLinkedRead::Module(actual_target)) => {
                    assert_eq!(*actual_target, vm.module_registry.get(target).expect("linked module target").object);
                }
                (phalcom_modules::LinkedReadSpec::Binding(symbol), RuntimeLinkedRead::Binding(binding)) => {
                    let target = vm.module_registry.get(&symbol.module).expect("linked binding target").object;
                    let name = vm.interner.intern(&symbol.name);
                    assert_eq!(binding.module, target);
                    assert_eq!(Some(binding.slot as usize), vm.heap.module(target).slot_of(name));
                }
                (expected, actual) => panic!("linked read shape changed: {expected:?} -> {actual:?}"),
            }
        }
    }

    #[test]
    fn repeated_full_vms_reuse_global_compiler_work() {
        let _counter_scope = UniverseBootstrapCounterScope::enter();
        let canonical = crate::modules::canonical_universe_program().expect("canonical Universe compiler product");
        let modules_per_vm = canonical.bootstrap_order().len();
        let compile_before = UNIVERSE_AST_COMPILATIONS.load(Ordering::Relaxed);
        let execute_before = UNIVERSE_INITIALIZER_EXECUTIONS.load(Ordering::Relaxed);

        let _a = VM::new();
        let _b = VM::new();
        let _c = VM::new();

        assert_eq!(crate::modules::canonical_universe::SOURCE_INDEX_BUILDS.load(Ordering::Relaxed), 1);
        assert_eq!(crate::modules::canonical_universe::NATIVE_CONTRACT_VERIFICATIONS.load(Ordering::Relaxed), 1);
        assert_eq!(crate::modules::canonical_universe::CANONICAL_LINKS.load(Ordering::Relaxed), 1);
        assert_eq!(crate::modules::canonical_universe::CANONICAL_SEMANTIC_ANALYSES.load(Ordering::Relaxed), 1);
        assert_eq!(crate::modules::canonical_universe::CANONICAL_PROGRAM_PROJECTIONS.load(Ordering::Relaxed), 1);
        assert_eq!(UNIVERSE_AST_COMPILATIONS.load(Ordering::Relaxed) - compile_before, modules_per_vm * 3);
        assert_eq!(UNIVERSE_INITIALIZER_EXECUTIONS.load(Ordering::Relaxed) - execute_before, modules_per_vm * 3);
    }
}
