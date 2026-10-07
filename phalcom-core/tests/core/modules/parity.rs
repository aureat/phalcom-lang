use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer};
use phalcom_core::heap::{FamilySpec, Object, RuntimeExportRef};
use phalcom_core::modules::{ProgramCompileError, RuntimeLinkedRead};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{LinkedExportTarget, SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::SemanticWorkspaceSession;
use phalcom_semantic::identity::SemanticTargetId;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

fn location(path: &Path) -> SourceLocation {
    let path = path.canonicalize().unwrap();
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path,
    }
}

fn write_parity_package(root: &Path) {
    fs::write(root.join("package.ph"), "").unwrap();
    fs::write(root.join("main.ph"), "import .either as EitherModule\nfrom .constants import version\n").unwrap();
    fs::write(
        root.join("either.ph"),
        "data Either<L, R> {\n    @variant\n    Left(_ value: L)\n\n    @variant\n    Right(_ value: R)\n}\nexport Either\n",
    )
    .unwrap();
    fs::write(root.join("constants.ph"), "const version = \"1\"\nexport version\n").unwrap();
}

#[test]
fn exported_static_constant_materializes_for_selected_and_module_reads() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "expose .main\nexpose .constants\n").unwrap();
    fs::write(temp.path().join("constants.ph"), "const pi: Float = 3.141592653589793\nexport pi\n").unwrap();
    fs::write(
        temp.path().join("main.ph"),
        "from .constants import pi\nimport .constants as constants\nlet selected = pi\nlet qualified = constants.pi\n",
    )
    .unwrap();

    let program = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf()))
        .expect("static constant imports must compile");
    let mut vm = phalcom_core::vm::VM::new();
    vm.run_compiled(&program).expect("static constant imports must run");
    let entry = vm.module_registry.get(&program.entry).unwrap().object;
    let module = vm.heap.module(entry);
    for name in ["selected", "qualified"] {
        let symbol = vm.interner.find(name).unwrap();
        let value = module.get(symbol).unwrap();
        assert_eq!(value.as_float().unwrap().to_bits(), std::f64::consts::PI.to_bits(), "{name}");
    }
    let constants = program
        .modules
        .keys()
        .find(|id| id.path.components().last().is_some_and(|part| part.as_str() == "constants"))
        .unwrap();
    let constants_obj = vm.module_registry.get(constants).unwrap().object;
    assert!(vm.heap.module(constants_obj).name_to_slot.keys().all(|name| vm.resolve_symbol(*name) != "pi"));
}

#[test]
fn exported_static_constants_keep_canonical_identity_across_reexports_and_gc() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("package.ph"),
        "expose .main\nexpose .source\nexpose .facade\nexpose .facade2\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("source.ph"),
        "const pi: Float = 3.141592653589793\nconst message: String = \"persistent constant\"\nconst huge: Int = 999999999999999999999999999999\nexport pi, message, huge\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("facade.ph"),
        "export pi as tau, message as greeting, huge as magnitude from .source\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("facade2.ph"),
        "export tau as circle, greeting as words, magnitude as size from .facade\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("main.ph"),
        "import .source as source\nimport .facade2 as facade2\nfrom .source import pi as selectedPi\nfrom .facade2 import circle as selectedCircle\nlet selected = selectedPi\nlet selectedAlias = selectedCircle\nlet qualified = source.pi\nlet transitive = facade2.circle\nlet reflected = facade2.__export__(#circle).value\nlet message = facade2.words\nlet huge = facade2.size\n",
    )
    .unwrap();

    let program =
        phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf()))
            .expect("re-export package must compile");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("static constants materialize once at canonical owners");

    let find_module = |suffix: &str| {
        program
            .modules
            .keys()
            .find(|id| id.path.components().last().is_some_and(|part| part.as_str() == suffix))
            .expect("module in compiled package")
    };
    let source_id = find_module("source");
    let facade_id = find_module("facade");
    let facade2_id = find_module("facade2");
    let source_obj = vm.module_registry.get(source_id).unwrap().object;
    let facade_obj = vm.module_registry.get(facade_id).unwrap().object;
    let facade2_obj = vm.module_registry.get(facade2_id).unwrap().object;

    let source = vm.heap.module(source_obj);
    let pi = vm.interner.find("pi").unwrap();
    let message = vm.interner.find("message").unwrap();
    let huge = vm.interner.find("huge").unwrap();
    let pi_ref = match source.export(pi).unwrap() {
        RuntimeExportRef::StaticConstant(reference) => reference,
        other => panic!("expected static pi export, got {other:?}"),
    };
    let message_ref = match source.export(message).unwrap() {
        RuntimeExportRef::StaticConstant(reference) => reference,
        other => panic!("expected static message export, got {other:?}"),
    };
    let huge_ref = match source.export(huge).unwrap() {
        RuntimeExportRef::StaticConstant(reference) => reference,
        other => panic!("expected static huge export, got {other:?}"),
    };
    assert_eq!(pi_ref.module, source_obj);
    assert_eq!(message_ref.module, source_obj);
    assert_eq!(huge_ref.module, source_obj);
    assert_eq!(source.static_constants.len(), 3, "one value per canonical source constant");
    assert!(source.name_to_slot.keys().all(|name| ![pi, message, huge].contains(name)), "const values have no global slots");

    for (module, exported_name, expected) in [
        (facade_obj, "tau", pi_ref),
        (facade2_obj, "circle", pi_ref),
        (facade_obj, "greeting", message_ref),
        (facade2_obj, "words", message_ref),
        (facade_obj, "magnitude", huge_ref),
        (facade2_obj, "size", huge_ref),
    ] {
        let name = vm.interner.find(exported_name).unwrap();
        assert_eq!(vm.heap.module(module).export(name), Some(RuntimeExportRef::StaticConstant(expected)), "{exported_name}");
    }

    let string_value = vm.heap.module(source_obj).static_constants[&message_ref.name];
    let big_value = vm.heap.module(source_obj).static_constants[&huge_ref.name];
    let string_obj = string_value.as_obj().expect("String constant is heap-backed");
    let big_obj = big_value.as_obj().expect("LargeInt constant is heap-backed");
    assert!(matches!(vm.heap.get(string_obj), Object::Str(_)));
    assert!(matches!(vm.heap.get(big_obj), Object::LargeInt(_)));
    vm.force_gc();
    assert!(vm.heap.try_get(string_obj).is_some(), "owner cache roots the String across GC");
    assert!(vm.heap.try_get(big_obj).is_some(), "owner cache roots the LargeInt across GC");

    vm.run_compiled(&program).expect("constant reads, reflection, and re-exports run");
    let main_id = find_module("main");
    let main_obj = vm.module_registry.get(main_id).unwrap().object;
    let read = |name: &str| -> Value {
        let symbol = vm.interner.find(name).unwrap();
        let module = vm.heap.module(main_obj);
        module.get(symbol).unwrap()
    };
    for name in ["selected", "selectedAlias", "qualified", "transitive", "reflected"] {
        assert_eq!(read(name).as_float().unwrap().to_bits(), std::f64::consts::PI.to_bits(), "{name}");
    }
    assert_eq!(vm.heap.as_string(read("message").as_obj().unwrap()).unwrap().as_str(), "persistent constant");
    let large = vm.heap.as_large_int(read("huge").as_obj().unwrap()).unwrap();
    assert_eq!(large.to_string(), "999999999999999999999999999999");
    let dependencies = phalcom_core::primitive::reflection::module_dependencies(&mut vm, &Value::obj(facade2_obj), &[])
        .expect("Module.dependencies reflection");
    let dependency_values = vm.tuple_view_value(dependencies).expect("dependencies tuple").positionals();
    assert!(!dependency_values.is_empty(), "reflection reports runtime dependencies");
    let mut found_source_dependency = false;
    for dependency in dependency_values {
        let dependency_module = vm.heap.module_dependency(dependency.as_obj().unwrap()).module;
        found_source_dependency |= dependency_module == source_obj;
    }
    assert!(found_source_dependency, "Module.dependencies reports the canonical source owner");

    let main = vm.heap.module(main_obj);
    let selected = main.linked_reads.iter().find_map(|read| match read {
        RuntimeLinkedRead::StaticConstant(reference) if reference.name == pi_ref.name => Some(*reference),
        _ => None,
    });
    assert_eq!(selected, Some(pi_ref), "selected import retains canonical owner and name");
}

#[test]
fn universe_math_implementation_children_are_not_exposed() {
    for path in ["universe.math.float_kernel", "universe.math.integer"] {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.ph"), "expose .main\n").unwrap();
        fs::write(temp.path().join("main.ph"), format!("import {path}\n")).unwrap();
        let error = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf()))
            .expect_err("internal math child must not be externally traversable");
        assert!(error.to_string().contains("not exposed"), "{path}: {error}");
    }
}

#[test]
fn universe_math_exports_exact_families_and_rejects_non_matching_numeric_arguments() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "expose .main\n").unwrap();
    fs::write(
        temp.path().join("main.ph"),
        "import universe.math as math\nlet sqrt = math.sqrt\nlet atan2 = math.atan2\nlet log = math.log\nlet gcd = math.gcd\n",
    )
    .unwrap();

    let program = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf()))
        .expect("math Family fixture compiles");
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("math Family fixture runs");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;

    for (name, selector) in [("sqrt", "sqrt(_)"), ("atan2", "atan2(_,_)"), ("log", "log(_,base)"), ("gcd", "gcd(_,_)")] {
        let value = vm.heap.module(module).get(vm.interner.find(name).expect("exported local")).expect("Family value");
        let family = vm.heap.as_family(value.as_obj().expect("Family object")).expect("exact captured Family");
        let FamilySpec::Exact(symbol) = family.spec else {
            panic!("{name} must use an exact Family capture");
        };
        assert_eq!(vm.resolve_symbol(symbol), selector, "{name}");
    }

    for source in [
        "import universe.math as math\nmath.sqrt(4)\n",
        "import universe.math as math\nmath.ln(1)\n",
        "import universe.math as math\nmath.log(8, base: 2.0)\n",
        "import universe.math as math\nmath.gcd(12.0, 8.0)\n",
        "import universe.math as math\nmath.isqrt(81.0)\n",
    ] {
        fs::write(temp.path().join("main.ph"), source).expect("negative math fixture");
        let error = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf()))
            .expect_err("math must reject non-matching numeric arguments before runtime");
        assert!(matches!(error, ProgramCompileError::Semantic(_)), "{source}: {error}");
    }
}

#[test]
fn strict_compiler_and_workspace_publish_identical_module_products() {
    let temp = TempDir::new().unwrap();
    write_parity_package(temp.path());

    let strict = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Package(temp.path().to_path_buf())).expect("strict package analysis");

    let mut workspace = SemanticWorkspaceSession::new();
    let mutations = ["package.ph", "main.ph", "either.ph", "constants.ph"].into_iter().map(|name| {
        let path = temp.path().join(name);
        WorkspaceSourceBatchMutation::SetDiskSnapshot {
            source: location(&path),
            text: Arc::from(fs::read_to_string(&path).unwrap()),
            revision: SourceRevision(1),
            recovered_program: None,
        }
    });
    let publication = workspace.apply_module_mutations(mutations).expect("workspace package analysis");
    let workspace_modules = workspace.module_session();

    let source_id = |name: &str| SourceId(temp.path().join(name).canonicalize().unwrap().to_string_lossy().into());
    let strict_module = |name: &str| {
        strict
            .sources
            .iter()
            .find(|(_, source)| source.source.as_ref().map(|location| &location.source_id) == Some(&source_id(name)))
            .map(|(module, _)| module.clone())
            .expect("strict source module")
    };
    let strict_main = strict_module("main.ph");
    let strict_either = strict_module("either.ph");
    let strict_constants = strict_module("constants.ph");

    assert_eq!(workspace_modules.module_for_source(&source_id("main.ph")), Some(&strict_main));
    assert_eq!(workspace_modules.module_for_source(&source_id("either.ph")), Some(&strict_either));
    assert_eq!(workspace_modules.module_for_source(&source_id("constants.ph")), Some(&strict_constants));

    let strict_main_linked = &strict.linked.modules[&strict_main];
    let strict_either_read = strict_main_linked
        .linked_reads
        .iter()
        .find_map(|read| match read {
            phalcom_modules::LinkedReadSpec::Module(module) => Some(module),
            _ => None,
        })
        .expect("strict module import read");
    assert_eq!(
        workspace_modules.resolved_imports().get(&(strict_main.clone(), ".either".into())),
        Some(strict_either_read)
    );

    let strict_either_export = &strict.linked.modules[&strict_either].interface.exports["Either"].target;
    let workspace_either_export = &workspace_modules.linked().unwrap().modules[&strict_either].interface.exports["Either"].target;
    assert_eq!(strict_either_export, workspace_either_export);
    assert_eq!(
        strict_either_export,
        &LinkedExportTarget::Binding(phalcom_modules::SymbolId {
            module: strict_either.clone(),
            name: "Either".into(),
        })
    );

    let either_declaration = phalcom_semantic::DeclarationId::new(strict_either.clone(), "Either".into());
    assert!(strict.semantic.declarations().get(&either_declaration).is_some());
    assert!(publication.snapshot.declarations().get(&either_declaration).is_some());

    let version_symbol = phalcom_modules::SymbolId {
        module: strict_constants.clone(),
        name: "version".into(),
    };
    assert!(
        strict_main_linked
            .linked_reads
            .iter()
            .any(|read| matches!(read, phalcom_modules::LinkedReadSpec::Binding(symbol) if symbol == &version_symbol))
    );
    assert!(
        publication
            .snapshot
            .source_index()
            .occurrences_for_target(&SemanticTargetId::ModuleBinding(version_symbol))
            .is_some()
    );
}

#[test]
fn package_less_parity_comparator_rejects_relative_imports() {
    let temp = TempDir::new().unwrap();
    let main = temp.path().join("main.ph");
    fs::write(&main, "import .sibling as Sibling\n").unwrap();
    fs::write(temp.path().join("sibling.ph"), "class Sibling {}\n").unwrap();

    let result = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Module(main));
    assert!(matches!(
        result,
        Err(phalcom_core::modules::compile::ProgramCompileError::StandaloneImportRequiresPackageContext { .. })
    ));
}
