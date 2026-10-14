use phalcom_core::bytecode::Bytecode;
use phalcom_core::compiler::lib::CompilerError;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer, ProgramCompiler};
use phalcom_core::vm::VM;
use phalcom_modules::interface::{InterfaceBuilder, ModuleBindingSemantics};
use std::sync::Arc;

#[test]
fn const_is_static_module_surface() {
    let source = Arc::<str>::from("const Answer: Int = 42\n");
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(source.clone())).expect("static const should analyze cleanly");

    let parsed = &analyzed.sources[&analyzed.entry];
    let interface = InterfaceBuilder::build(parsed.id.clone(), parsed.kind, &parsed.program).expect("unlinked interface must build");
    let decl = interface.declarations.get("Answer").expect("Answer must be declared");
    assert_eq!(decl.semantics, ModuleBindingSemantics::StaticConstant);
}

#[test]
fn const_initializer_not_emitted_as_runtime_define() {
    let source = "const Answer: Int = 42\n";
    let mut vm = VM::new_kernel();
    let module = vm.create_module("main", "<test>");
    let closure_ref = vm.compile_closure(module, source).expect("const should compile");
    let chunk = &vm.heap.closure(closure_ref).callable.chunk;

    // DefineGlobal must not be emitted for Answer
    let defines_global = chunk.code.iter().any(|op| matches!(op, Bytecode::DefineGlobal(_)));
    assert!(!defines_global, "const initializer must not emit DefineGlobal at runtime");
}

#[test]
fn const_does_not_allocate_runtime_local() {
    let source = r#"
let mut f = || {
  const LocalConst: Int = 99
  42
}
"#;
    let mut vm = VM::new_kernel();
    let module = vm.create_module("main", "<test>");
    let closure_ref = vm.compile_closure(module, source).expect("closure with const should compile");
    let top_chunk = &vm.heap.closure(closure_ref).callable.chunk;
    let inner_closure_ref = top_chunk
        .constants
        .iter()
        .find_map(|v| {
            if let Some(obj) = v.as_obj() {
                if vm.heap.as_closure(obj).is_some() {
                    return Some(obj);
                }
            }
            None
        })
        .expect("inner closure found");
    let inner_callable = &vm.heap.closure(inner_closure_ref).callable;

    // No SetLocal or GetLocal should exist for LocalConst, and it must not be in local_names
    let has_set_local = inner_callable.chunk.code.iter().any(|op| matches!(op, Bytecode::SetLocal(_)));
    assert!(!has_set_local, "const must not emit SetLocal");
    let has_local_name = inner_callable.local_names.iter().any(|sym| vm.resolve_symbol(*sym) == "LocalConst");
    assert!(!has_local_name, "const must not allocate a runtime local slot");
}

#[test]
fn const_without_initializer_rejected() {
    let source = "const Uninit: Int\n";
    let mut vm = VM::new_kernel();
    let module = vm.create_module("main", "<test>");
    let err = vm.compile_closure(module, source).expect_err("const without initializer must fail");
    match err {
        PhError::Compile(CompilerError::ConstWithoutInitializer(name)) => {
            assert_eq!(name, "Uninit");
        }
        other => panic!("expected ConstWithoutInitializer, got: {:?}", other),
    }
}

#[test]
fn runtime_use_of_unmaterialized_static_const_fails_closed() {
    let source = r#"
const Answer: Int = 42
let mut y = Answer
"#;
    let mut vm = VM::new_kernel();
    let module = vm.create_module("main", "<test>");
    let err = vm.compile_closure(module, source).expect_err("runtime use of static const must fail closed");
    match err {
        PhError::Compile(CompilerError::StaticConstUnmaterialized(name)) => {
            assert_eq!(name, "Answer");
        }
        other => panic!("expected StaticConstUnmaterialized, got: {:?}", other),
    }
}

#[test]
fn structural_const_without_runtime_use_compiles_cleanly() {
    let source: Arc<str> = "const Pair = (1, 2)\n".into();
    let program =
        ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source)).expect("structural const without runtime use should analyze and compile");
    let mut vm = VM::new_kernel();
    vm.run_compiled(&program)
        .expect("structural const without runtime use should compile and run cleanly");
}

#[test]
fn structural_const_runtime_use_materializes_owner_value() {
    let source: Arc<str> = "const Pair = (1, 2)\nlet mut p = Pair\n".into();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source)).expect("program should analyze cleanly");
    let mut vm = VM::new_kernel();
    vm.run_compiled(&program).expect("structural const runtime read should succeed");
    let entry = program.initialization_order.last().expect("entry module");
    let module = vm.heap.module(vm.module_registry.get(entry).unwrap().object);
    let pair = vm.interner.find("Pair").unwrap();
    let read = vm.interner.find("p").unwrap();
    assert_eq!(module.globals[module.name_to_slot[&read]], module.static_constants[&pair]);
}
