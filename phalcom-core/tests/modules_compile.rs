use phalcom_core::bytecode::Bytecode;
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::modules::CompileBindings;
use phalcom_core::vm::VM;
use phalcom_modules::{
    ImportBindingId, LinkedModule, LinkedModuleInterface, LinkedReadSpec, ModuleBindingLayout, ModuleId, ModuleKind, ModuleMetadata, ModulePath,
    ResolvedProjectId, SymbolId,
};
use std::collections::BTreeMap;

fn linked_module() -> LinkedModule {
    let module = ModuleId {
        project: phalcom_modules::ProjectIdentity::Resolved(ResolvedProjectId::from_raw(1)),
        path: ModulePath::root(),
    };
    let symbol = SymbolId {
        module: ModuleId {
            project: phalcom_modules::ProjectIdentity::Resolved(ResolvedProjectId::from_raw(1)),
            path: ModulePath::from_components(vec![phalcom_modules::ModuleComponent::from_identifier("settings").unwrap()]),
        },
        name: "mode".into(),
    };
    let mut imports = BTreeMap::new();
    imports.insert("mode".into(), ImportBindingId(0));
    LinkedModule {
        interface: LinkedModuleInterface {
            module,
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::new(),
            imports,
        },
        linked_reads: vec![LinkedReadSpec::Binding(symbol)],
        runtime_dependencies: Vec::new(),
    }
}

#[test]
fn linked_imports_lower_to_indexed_reads_without_path_constants() {
    let linked = linked_module();
    let bindings = CompileBindings::from_linked_module(&linked);
    let mut vm = VM::new();
    let module = vm.create_module("main", "<linked>");
    let closure = vm.compile_closure_as_with_bindings(module, "mode\n", UnitKind::File, Some(bindings)).unwrap();
    let code = &vm.heap.closure(closure).callable.chunk.code;
    assert!(code.iter().any(|opcode| matches!(opcode, Bytecode::GetLinked(0))));
    assert!(!code.iter().any(|opcode| matches!(opcode, Bytecode::GetGlobal(_))));
}

#[test]
fn linked_imports_are_immutable() {
    let linked = linked_module();
    let bindings = CompileBindings::from_linked_module(&linked);
    let mut vm = VM::new();
    let module = vm.create_module("main", "<linked>");
    let error = vm
        .compile_closure_as_with_bindings(module, "mode = 1\n", UnitKind::File, Some(bindings))
        .unwrap_err();
    assert!(error.to_string().contains("immutable"));
}
