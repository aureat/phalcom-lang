use phalcom_core::modules::RuntimeLinkedRead;
use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer, ProgramCompileError, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{LinkError, LinkedReadSpec, ModuleComponent, ModuleId, ModulePath, ModuleResolutionError};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

fn semantic_session_with_private_numbers_child_exposed() -> phalcom_semantic::SemanticWorkspaceSession {
    static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(40_000);

    let universe_provider = phalcom_modules::UniverseSourceProvider::new();
    let numbers = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("numbers").expect("valid module component"),
    ]));
    let numbers_source = universe_provider.source_text(&numbers).expect("canonical numbers source");
    let compatible_numbers_source = numbers_source.replacen("from .capabilities import", "expose .capabilities\nfrom .capabilities import", 1);
    assert_ne!(
        compatible_numbers_source,
        numbers_source.as_ref(),
        "test fixture must amend the private child exposure"
    );
    let universe_provider = phalcom_modules::UniverseSourceProvider::with_test_source_override(numbers, Arc::from(compatible_numbers_source))
        .expect("test-only Universe source override is valid");
    phalcom_semantic::SemanticWorkspaceSession::with_universe_source_provider(
        phalcom_semantic::identity::WorkspaceId::from_raw(NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed)),
        universe_provider,
    )
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_linker").join(name)
}

/// COMP-01 — Single-module file compiles without error
#[test]
fn comp_01_single_module_inline_compiles() {
    let source: Arc<str> = "let x = 42\n".into();
    let selection = EntrySelection::Inline(source);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile should succeed");
    assert_eq!(program.modules.len(), 1);
}

/// COMP-02 — Selective import of non-exported name fails at link time
#[test]
fn comp_02_selective_import_non_exported_fails_link() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.importer\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .exporter\nexpose .importer\n").unwrap();
    std::fs::write(src_dir.join("exporter.ph"), "class Foo {}\n").unwrap(); // not exported
    std::fs::write(src_dir.join("importer.ph"), "from .exporter import Foo\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let result = ProgramCompiler::compile_entry_selection(selection);
    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::MissingExport { ref name, .. })) if name == "Foo"),
        "expected MissingExport for Foo, got {:?}",
        result
    );
}

/// COMP-03 — Module-import without alias uses last path segment as local name
#[test]
fn comp_03_module_import_uses_last_segment() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "import universe.reflection.selector\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile should succeed");

    let (entry_id, entry_mod) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("entry"))
        .expect("entry module present");

    let linked_entry = program.linked.modules.get(entry_id).expect("linked entry present");
    assert!(linked_entry.bindings.imports.contains_key("selector"));
    assert!(
        entry_mod
            .linked_reads
            .iter()
            .any(|spec| matches!(spec, LinkedReadSpec::Module(id) if id.path.to_string().contains("selector")))
    );
}

/// COMP-04 — Builtin selective import of a real export compiles successfully
#[test]
fn comp_04_builtin_selective_import_real_export_compiles() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "from universe.errors.unsupported import unsupported\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile should succeed");

    let (entry_id, entry_mod) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("entry"))
        .expect("entry module present");

    let linked_entry = program.linked.modules.get(entry_id).expect("linked entry present");
    assert!(linked_entry.bindings.imports.contains_key("unsupported"));
    assert!(
        entry_mod
            .linked_reads
            .iter()
            .any(|spec| matches!(spec, LinkedReadSpec::Binding(sym) if &*sym.name == "unsupported"))
    );
}

/// COMP-05 — Builtin selective import of a non-exported name fails
#[test]
fn comp_05_builtin_selective_import_non_exported_fails() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "from universe.reflection import NonExistentExport\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let result = ProgramCompiler::compile_entry_selection(selection);
    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::MissingExport { ref name, .. })) if name == "NonExistentExport"),
        "expected MissingExport for NonExistentExport, got {:?}",
        result
    );
}

/// COMP-06 — Initialization order puts dependencies before dependents
#[test]
fn comp_06_chain_abc_initialization_order() {
    let selection = EntrySelection::Project(fixture_path("chain_abc"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile chain_abc");

    let order = &program.initialization_order;
    let pos_c = order
        .iter()
        .position(|id| id.path.to_string().ends_with(".c") || id.path.to_string() == "c")
        .expect("c found");
    let pos_b = order
        .iter()
        .position(|id| id.path.to_string().ends_with(".b") || id.path.to_string() == "b")
        .expect("b found");
    let pos_a = order
        .iter()
        .position(|id| id.path.to_string().ends_with(".a") || id.path.to_string() == "a")
        .expect("a found");

    assert!(pos_c < pos_b, "c must precede b");
    assert!(pos_b < pos_a, "b must precede a");
}

/// COMP-07 — Cyclic module dependency is detected and rejected
#[test]
fn comp_07_cycle_ab_detected_and_rejected() {
    let selection = EntrySelection::Project(fixture_path("cycle_ab"));
    let result = ProgramCompiler::compile_entry_selection(selection);

    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::RuntimeCycle(_)))),
        "expected RuntimeCycle error, got {:?}",
        result
    );
}

/// Semantic-only selected imports remain resolvable without executable reads,
/// while analysis publishes a finalized runtime layout.
#[test]
fn semantic_type_cycle_finalizes_without_runtime_import_slots() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"phase_links\"\nnamespace = \"phase_links\"\nversion = \"0.1.0\"\nentry = \"phase_links.main\"\n",
    )
    .unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .a\nexpose .b\nexpose .main\n").unwrap();
    std::fs::write(src_dir.join("a.ph"), "from .b import B\nclass A { _value: B }\nexport A\n").unwrap();
    std::fs::write(src_dir.join("b.ph"), "from .a import A\nclass B { _value: A }\nexport B\n").unwrap();
    std::fs::write(src_dir.join("main.ph"), "from .a import A\nlet value = A\n").unwrap();

    let analyzed = ProgramAnalyzer::analyze_entry_selection_with_session(
        EntrySelection::Project(root.to_path_buf()),
        semantic_session_with_private_numbers_child_exposed(),
    )
    .expect("mutual type imports should pass symbolic semantic linking");
    assert!(!analyzed.semantic.has_errors(), "{:#?}", analyzed.semantic.diagnostics);

    let a_id = analyzed
        .linked
        .modules
        .keys()
        .find(|id| id.path.to_string() == "a")
        .expect("module a is linked");
    let b_id = analyzed
        .linked
        .modules
        .keys()
        .find(|id| id.path.to_string() == "b")
        .expect("module b is linked");
    let main_id = analyzed
        .linked
        .modules
        .keys()
        .find(|id| id.path.to_string() == "main")
        .expect("entry module is linked");

    for (module, local_name) in [(a_id, "B"), (b_id, "A")] {
        let linked = &analyzed.linked.modules[module];
        assert!(
            !linked.bindings.imports.contains_key(local_name),
            "interface-only `{local_name}` must not have a runtime slot"
        );
        assert!(linked.linked_reads.is_empty(), "interface-only import must not create a linked read");
        assert!(
            analyzed.linked.graphs.runtime.edges_from(module).is_empty(),
            "interface-only import must not enter the runtime graph"
        );
    }

    let main = &analyzed.linked.modules[main_id];
    assert!(main.bindings.imports.contains_key("A"), "value-position import retains its runtime slot");
    assert_eq!(main.linked_reads.len(), 1);

    assert!(analyzed.linked.modules.contains_key(a_id));
    assert!(analyzed.linked.modules.contains_key(b_id));
}

/// A protocol default may construct an adapter that depends on the protocol
/// only for conformance. The finalized runtime edge stays one-way.
#[test]
fn protocol_default_to_adapter_cycle_is_only_semantic_on_adapter_side() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"phase_iteration\"\nnamespace = \"phase_iteration\"\nversion = \"0.1.0\"\nentry = \"phase_iteration.main\"\n",
    )
    .unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .adapter\nexpose .main\nexpose .protocol\n").unwrap();
    std::fs::write(
        src_dir.join("protocol.ph"),
        "from .adapter import Adapter\ntrait Builds { build() -> Int { let adapter = Adapter.new(); 23 } }\nexport Builds\n",
    )
    .unwrap();
    std::fs::write(
        src_dir.join("adapter.ph"),
        "from .protocol import Builds\nclass Adapter {}\nimpl Builds for Adapter {}\nexport Adapter\n",
    )
    .unwrap();
    std::fs::write(src_dir.join("main.ph"), "from .adapter import Adapter\nlet result = Adapter.new().build()\n").unwrap();

    let analyzed = ProgramAnalyzer::analyze_entry_selection_with_session(
        EntrySelection::Project(root.to_path_buf()),
        semantic_session_with_private_numbers_child_exposed(),
    )
    .expect("interface-only conformance edge should not form a runtime cycle");
    let protocol_id = analyzed
        .linked
        .modules
        .keys()
        .find(|id| id.path.to_string() == "protocol")
        .expect("protocol module is linked");
    let adapter_id = analyzed
        .linked
        .modules
        .keys()
        .find(|id| id.path.to_string() == "adapter")
        .expect("adapter module is linked");
    let main_id = analyzed
        .linked
        .modules
        .keys()
        .find(|id| id.path.to_string() == "main")
        .expect("entry module is linked");

    assert!(
        analyzed
            .semantic
            .trait_headers
            .contains(&phalcom_semantic::identity::DeclarationId::new(protocol_id.clone(), "Builds".into())),
        "the adapter's interface-only import must retain the protocol's canonical trait identity"
    );
    assert!(
        analyzed
            .linked
            .graphs
            .runtime
            .edges_from(protocol_id)
            .iter()
            .any(|edge| &edge.dependency == adapter_id),
        "the default's adapter construction contributes the protocol-to-adapter runtime edge"
    );
    assert!(
        analyzed
            .linked
            .graphs
            .runtime
            .edges_from(adapter_id)
            .iter()
            .all(|edge| &edge.dependency != protocol_id),
        "adapter conformance alone does not create an adapter-to-protocol runtime edge"
    );
    let adapter_module = &analyzed.linked.modules[adapter_id];
    assert!(!adapter_module.bindings.imports.contains_key("Builds"));
    assert!(adapter_module.linked_reads.is_empty(), "the semantic-only trait import has no executable slot");
    let order = &analyzed.linked.initialization_order;
    assert!(order.iter().position(|id| id == adapter_id).unwrap() < order.iter().position(|id| id == protocol_id).unwrap());
    assert!(order.iter().position(|id| id == protocol_id).unwrap() < order.iter().position(|id| id == main_id).unwrap());
}

/// The same protocol/default → adapter path still rejects a runtime edge in
/// the reverse direction; strict cycle checking is independent of semantic
/// conformance edges.
#[test]
fn protocol_default_adapter_runtime_cycle_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"phase_iteration_cycle\"\nnamespace = \"phase_iteration_cycle\"\nversion = \"0.1.0\"\nentry = \"phase_iteration_cycle.main\"\n",
    )
    .unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .adapter\nexpose .main\nexpose .protocol\n").unwrap();
    std::fs::write(
        src_dir.join("protocol.ph"),
        "from .adapter import Adapter\ntrait Builds { build() -> Int { let adapter = Adapter.new(); 23 } }\nexport Builds\n",
    )
    .unwrap();
    std::fs::write(
        src_dir.join("adapter.ph"),
        "from .protocol import Builds\nclass Adapter {}\nimpl Builds for Adapter {}\nlet protocol = Builds\nexport Adapter\n",
    )
    .unwrap();
    std::fs::write(src_dir.join("main.ph"), "from .adapter import Adapter\nlet result = Adapter.new().build()\n").unwrap();

    let result = ProgramAnalyzer::analyze_entry_selection_with_session(
        EntrySelection::Project(root.to_path_buf()),
        semantic_session_with_private_numbers_child_exposed(),
    );
    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::RuntimeCycle(_)))),
        "a runtime value use of the protocol from the adapter must restore strict cycle rejection; got {result:?}"
    );
}

/// COMP-07b — Missing logical module is rejected during resolution.
#[test]
fn comp_07b_missing_logical_module_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "import .missing as Missing\n").unwrap();

    let result = ProgramCompiler::compile_entry_selection(EntrySelection::Project(root.to_path_buf()));
    assert!(
        matches!(result, Err(ProgramCompileError::Resolution(ModuleResolutionError::ModuleNotFound(_)))),
        "expected missing logical module to fail resolution, got {result:?}"
    );
}

/// COMP-08 — EntrySelection::Module for a standalone .ph file succeeds
#[test]
fn comp_08_standalone_module_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("standalone.ph");
    std::fs::write(&file, "let x = 1\n").unwrap();

    let selection = EntrySelection::Module(file);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("standalone compile");
    assert_eq!(program.modules.len(), 1);
}

/// MAT-01 — Module export slots are populated after run_compiled
#[test]
fn mat_01_export_slots_populated() {
    let mut vm = VM::new();
    let option_module = vm
        .module_registry
        .iter()
        .find(|(id, _)| id.to_string() == "universe:option.option")
        .expect("canonical Option module materialized")
        .1
        .object;
    let none_sym = vm.interner.intern("None");
    assert_eq!(
        vm.heap.module(option_module).get(none_sym),
        Some(Value::none()),
        "canonical None binding remains immediate absence"
    );
    let option_sym = vm.interner.intern("Option");
    assert_eq!(
        vm.heap.module(option_module).get(option_sym),
        Some(Value::obj(vm.universe.classes.option_class)),
        "source reopen preserves the native Option class identity"
    );
    let is_none_sym = vm.interner.intern("isNone");
    assert!(
        vm.heap.class(vm.universe.classes.option_class).methods.contains_key(&is_none_sym),
        "source Option reopen installs its behavior on the canonical class"
    );
    assert!(vm.heap.module(option_module).metadata.is_some(), "canonical module metadata is installed");

    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (exporter_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("exporter"))
        .expect("exporter module present");

    let exporter_obj = vm.module_registry.get(exporter_id).unwrap().object;
    let widget_sym = vm.interner.intern("Widget");
    let widget_val = vm.heap.module(exporter_obj).get(widget_sym).expect("Widget slot exists");
    assert!(!widget_val.is_none(), "Widget slot must not be None");
    assert!(widget_val.is_obj(), "Widget slot must be an object");

    let (importer_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("importer"))
        .expect("importer module present");
    let importer_obj = vm.module_registry.get(importer_id).unwrap().object;
    let linked_read = vm
        .heap
        .module(importer_obj)
        .linked_reads
        .first()
        .copied()
        .expect("finalized Widget linked read");
    let linked_value = match linked_read {
        RuntimeLinkedRead::Module(module) => Value::obj(module),
        RuntimeLinkedRead::Binding(binding) => vm.heap.module(binding.module).get_by_slot(binding.slot as usize).expect("linked binding value"),
        RuntimeLinkedRead::StaticConstant(reference) => vm.heap.module(reference.module).static_constants[&reference.name],
    };
    assert_eq!(linked_value, widget_val, "finalized user linked read preserves the exported class identity");

    let temp = tempfile::tempdir().expect("temporary availability-contract project");
    let source_dir = temp.path().join("src");
    std::fs::create_dir_all(&source_dir).expect("create project source directory");
    std::fs::write(
        temp.path().join("project.toml"),
        "[project]\nname = \"availability_probe\"\nnamespace = \"availability_probe\"\nversion = \"0.1.0\"\nsource = \"src\"\nentry = \"availability_probe.main\"\n",
    )
    .expect("write project manifest");
    std::fs::write(source_dir.join("package.ph"), "expose .main\n").expect("write package source");
    std::fs::write(
        source_dir.join("main.ph"),
        "from universe.errors.unsupported import unsupported\nlet observed = unsupported\nexport observed\n",
    )
    .expect("write Universe-importing user module");
    let user_program = ProgramCompiler::compile_entry_selection(EntrySelection::Project(temp.path().to_path_buf()))
        .expect("compile user project against fully initialized Universe contract");
    vm.run_compiled(&user_program).expect("run Universe-importing user project");
    let (user_module_id, _) = user_program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().ends_with("main"))
        .expect("user main module present");
    let user_module = vm.module_registry.get(user_module_id).unwrap().object;
    let (unsupported_module_id, unsupported_module) = vm
        .module_registry
        .iter()
        .find(|(id, _)| id.to_string() == "universe:errors.unsupported")
        .expect("canonical exported binding module materialized");
    let unsupported_module_id = unsupported_module_id.clone();
    let unsupported_module = unsupported_module.object;
    let unsupported_target = phalcom_modules::RuntimeRequirementTarget::Binding {
        module: unsupported_module_id.clone(),
        name: "unsupported".into(),
    };
    let source_requirement = user_program
        .linked
        .runtime_requirements()
        .iter()
        .find(|requirement| &requirement.importer == user_module_id && requirement.provenance.target == unsupported_target)
        .expect("finalized user requirement retains exact source binding provenance");
    assert_eq!(
        source_requirement.provenance.product,
        phalcom_modules::RuntimeRequirementProduct::SourceInitializedValue,
        "authored source binding requires its initializer value"
    );
    assert!(
        !user_program
            .linked
            .graphs
            .runtime
            .edges_from(user_module_id)
            .iter()
            .any(|edge| edge.dependency == unsupported_module_id),
        "fully initialized Universe proof removes only the eager ordering edge"
    );
    let unsupported_read = vm
        .heap
        .module(user_module)
        .linked_reads
        .iter()
        .find_map(|read| match read {
            RuntimeLinkedRead::Binding(binding) if binding.module == unsupported_module => Some(*binding),
            _ => None,
        })
        .expect("finalized exported Universe binding linked read remains allocated");
    let unsupported_value = vm
        .heap
        .module(user_module)
        .get(vm.interner.intern("observed"))
        .expect("user module observed binding is populated");
    assert_eq!(
        vm.heap.module(unsupported_read.module).get_by_slot(unsupported_read.slot as usize),
        Some(unsupported_value),
        "post-bootstrap exact proof preserves the exported Universe value in its linked read"
    );
}

/// MAT-02 — __module__ intrinsic resolves to the module object itself
#[test]
fn mat_02_module_intrinsic_resolves_to_self() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let mod_sym = vm.interner.intern("__module__");
    for id in program.modules.keys() {
        let obj = vm.module_registry.get(id).unwrap().object;
        let val = vm.heap.module(obj).get(mod_sym).expect("__module__ present");
        assert_eq!(val, Value::obj(obj), "__module__ must match own object reference for {id}");
    }
}

/// MAT-03 — __package__ for a root package is Some(self)
#[test]
fn mat_03_root_package_intrinsic_is_some_self() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (root_id, _) = program.modules.iter().find(|(id, _)| id.path.is_root()).expect("root package present");

    let root_obj = vm.module_registry.get(root_id).unwrap().object;
    let pkg_sym = vm.interner.intern("__package__");
    let val = vm.heap.module(root_obj).get(pkg_sym).expect("__package__ present");
    assert_eq!(val, Value::obj(root_obj).wrap_some().unwrap());
}

/// MAT-04 — __package__ for a leaf module points to nearest parent package object
#[test]
fn mat_04_leaf_package_points_to_parent() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (root_id, _) = program.modules.iter().find(|(id, _)| id.path.is_root()).expect("root package present");
    let root_obj = vm.module_registry.get(root_id).unwrap().object;

    let (exporter_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("exporter"))
        .expect("exporter present");
    let exporter_obj = vm.module_registry.get(exporter_id).unwrap().object;

    let pkg_sym = vm.interner.intern("__package__");
    let val = vm.heap.module(exporter_obj).get(pkg_sym).expect("__package__ present");
    assert_eq!(val, Value::obj(root_obj).wrap_some().unwrap());
}

/// MAT-05 — Selective import value is non-None after full run_compiled
#[test]
fn mat_05_selective_import_value_non_none() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (importer_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("importer"))
        .expect("importer present");
    let importer_obj = vm.module_registry.get(importer_id).unwrap().object;

    // Check linked_read slot 0
    let linked_read = vm.heap.module(importer_obj).linked_reads.first().copied().expect("linked read 0");
    let value = match linked_read {
        RuntimeLinkedRead::Module(m) => Value::obj(m),
        RuntimeLinkedRead::Binding(b) => vm.heap.module(b.module).get_by_slot(b.slot as usize).expect("slot value"),
        RuntimeLinkedRead::StaticConstant(reference) => vm.heap.module(reference.module).static_constants[&reference.name],
    };
    assert!(!value.is_none(), "linked read for Widget must be non-None");
    assert!(value.is_obj(), "linked read for Widget must be an object");
}

/// MAT-06 — Module-import binding resolves to the module object
#[test]
fn mat_06_module_import_binding_resolves_to_module() {
    let mut vm = VM::new();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"mod_import_test\"\nnamespace = \"mod_import_test\"\nversion = \"0.1.0\"\nentry = \"mod_import_test.importer\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .importer\n").unwrap();
    std::fs::write(src_dir.join("importer.ph"), "import universe.reflection.selector\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile succeeds");
    vm.run_compiled(&program).expect("run succeeds");

    let (importer_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("importer"))
        .expect("importer present");
    let importer_obj = vm.module_registry.get(importer_id).unwrap().object;

    let linked_read = vm.heap.module(importer_obj).linked_reads.first().copied().expect("linked read 0");
    match linked_read {
        RuntimeLinkedRead::Module(m) => {
            let expected = ModuleId::universe(ModulePath::from_components(vec![
                ModuleComponent::from_identifier("reflection").unwrap(),
                ModuleComponent::from_identifier("selector").unwrap(),
            ]));
            assert_eq!(vm.heap.module(m).id, expected);
        }
        RuntimeLinkedRead::Binding(_) => panic!("expected Module linked read"),
        RuntimeLinkedRead::StaticConstant(_) => panic!("expected Module linked read"),
    }
}
