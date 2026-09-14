use super::*;
use crate::bytecode::Bytecode;
use crate::product::ProductSlotRepr;
use crate::error::PhError;
use crate::modules::compile::{EntrySelection, ProgramCompiler};
use crate::value::Value;
use crate::vm::VM;
use phalcom_modules::DeclarationId;
use std::sync::Arc;

fn run_inline_with_mode(source: &str, mode: ProductOptimizationMode) -> Result<(VM, crate::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    vm.product_optimization_mode = mode;
    let src: Arc<str> = source.into();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(src)).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let mod_obj = vm.module_registry.get(entry_id).unwrap().object;
    Ok((vm, mod_obj))
}

#[test]
fn static_anonymous_product_lowering_is_projected_for_module_roots() {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline("const t = (1, 2)\n".into())).expect("inline program compiles");
    let module = program.modules.get(&program.entry).expect("entry module");
    let spec = module.lowering.anonymous_products.values().next().expect("static tuple lowering spec");
    assert!(matches!(spec.kind, crate::modules::semantic_lowering::AnonymousProductConstructionKind::Tuple { positional_len: 2, .. }));
}

#[test]
fn static_anonymous_products_compile_and_execute_through_program_path() {
    let source = "let trace = []\nclass Probe {\n  @class\n  mark(_ value: Int) -> Int { trace.append(value); value }\n}\nconst tuple: (Int, Int) = (Probe.mark(1), Probe.mark(2))\nconst record: #{a: Int, b: Int} = #{b: Probe.mark(3), a: Probe.mark(4)}\n";
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source.into())).expect("inline program compiles");
    let lowering = &program.modules.get(&program.entry).expect("entry module").lowering;
    assert_eq!(lowering.anonymous_products.len(), 2);
    assert!(lowering
        .anonymous_products
        .values()
        .all(|spec| spec.layout.components.iter().all(|component| component.repr == ProductSlotRepr::Value)));
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("program materializes");
    let closure = vm
        .compile_program_module_closure(&program.entry, source, &program)
        .expect("entry closure compiles");
    let code = &vm.heap.closure(closure).callable.chunk.code;
    assert!(code.iter().any(|instruction| matches!(instruction, Bytecode::BuildStaticTuple { .. })));
    assert!(code.iter().any(|instruction| matches!(instruction, Bytecode::BuildStaticRecord { .. })));

    vm.run_compiled(&program).expect("static products execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let trace = vm.heap.module(module).get(vm.interner.find("trace").expect("trace symbol")).expect("trace global");
    let trace_id = trace.as_obj().expect("trace object");
    assert_eq!(vm.heap.list(trace_id).elements(), &[Value::int(1), Value::int(2), Value::int(3), Value::int(4)]);

    let tuple = vm.heap.module(module).get(vm.interner.find("tuple").expect("tuple symbol")).expect("tuple global");
    let tuple_view = vm.tuple_view(tuple.as_obj().expect("tuple object")).expect("tuple view");
    assert_eq!(tuple_view.len(), 2);
    assert_eq!(tuple_view.get(0), Some(Value::int(1)));
    assert_eq!(tuple_view.get(1), Some(Value::int(2)));

    let record = vm.heap.module(module).get(vm.interner.find("record").expect("record symbol")).expect("record global");
    let record_view = vm.record_view(record.as_obj().expect("record object")).expect("record view");
    assert_eq!(record_view.labels().iter().map(|label| vm.interner.lookup(*label)).collect::<Vec<_>>(), vec!["b", "a"]);
    assert_eq!(record_view.get(vm.interner.find("a").expect("a symbol")), Some(Value::int(4)));
    assert_eq!(record_view.get(vm.interner.find("b").expect("b symbol")), Some(Value::int(3)));
}

#[test]
fn unprovable_anonymous_product_uses_dynamic_pack_route() {
    let source = "const values = (1, 2)\nconst tuple = (0, *values)\n";
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source.into())).expect("inline program compiles");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("program materializes");
    let closure = vm
        .compile_program_module_closure(&program.entry, source, &program)
        .expect("entry closure compiles");
    let code = &vm.heap.closure(closure).callable.chunk.code;
    assert!(code.iter().any(|instruction| matches!(instruction, Bytecode::FinishTuplePack)));
}

#[test]
fn tuple_and_record_exactness_ignores_representation_and_record_presentation_order() {
    let source = "const tupleA = (1, 2)\nconst tupleB = (*tupleA)\nconst recordA = #{a: 1, b: 2}\nconst recordB = #{b: 2, a: 1}\nconst nestedA = #{a: (1, 2), b: #{c: 3}}\nconst nestedB = #{b: #{c: 3}, a: (1, 2)}\nconst tupleSame = tupleA === tupleB\nconst recordSame = recordA === recordB\nconst recordHashSame = recordA.hash == recordB.hash\nconst nestedSame = nestedA === nestedB\nconst crossKind = tupleA === recordA\n";
    let (vm, module) = run_inline_with_mode(source, ProductOptimizationMode::Enabled).expect("products execute");
    let get = |name: &str| vm.heap.module(module).get(vm.interner.find(name).expect("global symbol")).expect("global value");
    assert_eq!(get("tupleSame"), Value::bool(true));
    assert_eq!(get("recordSame"), Value::bool(true));
    assert_eq!(get("recordHashSame"), Value::bool(true));
    assert_eq!(get("nestedSame"), Value::bool(true));
    assert_eq!(get("crossKind"), Value::bool(false));
}

#[test]
fn test_decision_engine_pure_cases() {
    let decl_id = DeclarationId::new(phalcom_modules::ModuleId::universe_root(), "Point".into());
    let ctor_id = phalcom_semantic::identity::DataConstructorId::new(decl_id);
    let store = phalcom_semantic::types::TypeStore::default();
    let exporter = phalcom_semantic::metadata::MetadataExporter::new(&store, None, None, None, phalcom_type_meta::header::MetadataProfile::RuntimePublic);
    let bundle = Arc::new(exporter.build_bundle(&[]).expect("metadata bundle"));

    let mut candidate = CandidateBinding {
        binding: BindingId(1),
        name: "pt".to_string(),
        range: SourceRange::new(0, 0),
        mutable: false,
        is_global: false,
        shape: Some(VirtualShapePlan {
            kind: VirtualProductKind::Data {
                constructor: ctor_id.clone(),
                construction: Arc::new(crate::modules::semantic_lowering::DataConstructionLoweringSpec {
                    constructor: ctor_id,
                    exact_type: bundle,
                    layout: crate::product::ProductLayoutSpec::empty(),
                    argument_to_component: Box::new([0, 1]),
                }),
            },
            components: Box::new([
                VirtualComponentPlan::Scalar {
                    logical_component: 0,
                    leaf_offset: 0,
                },
                VirtualComponentPlan::Scalar {
                    logical_component: 1,
                    leaf_offset: 1,
                },
            ]),
            leaf_count: 2,
        }),
        rejection_reason: None,
        summary: ProductUseSummary {
            creation_loop_depth: 0,
            projections: 2,
            exact_variant_matches: 0,
            whole_values: vec![],
            captured: false,
            reassigned: false,
            opaque_expansion: false,
        },
    };

    // 1. Normal data with projections -> Virtualize
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Enabled);
    assert!(matches!(dec, ProductOptimizationDecision::Virtualize(_)));

    // 2. Disabled mode -> Materialize
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Disabled);
    assert_eq!(dec, ProductOptimizationDecision::Materialize(MaterializationReason::OptimizationDisabled));

    // 3. Mutable binding -> Materialize
    candidate.mutable = true;
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Enabled);
    assert_eq!(dec, ProductOptimizationDecision::Materialize(MaterializationReason::MutableBinding));
    candidate.mutable = false;

    // 4. Captured binding -> Materialize
    candidate.summary.captured = true;
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Enabled);
    assert_eq!(dec, ProductOptimizationDecision::Materialize(MaterializationReason::CapturedBinding));
    candidate.summary.captured = false;

    // 5. Multiple whole value uses -> Materialize
    candidate.summary.whole_values = vec![0, 0];
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Enabled);
    assert_eq!(dec, ProductOptimizationDecision::Materialize(MaterializationReason::MultipleWholeValueUses));

    // 6. Repeated loop whole use (whole loop depth > creation loop depth) -> Materialize
    candidate.summary.whole_values = vec![1];
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Enabled);
    assert_eq!(dec, ProductOptimizationDecision::Materialize(MaterializationReason::RepeatedLoopWholeUse));

    // 7. Single whole value use at same loop depth with projections -> Virtualize (sink whole allocation)
    candidate.summary.whole_values = vec![0];
    let dec = decide_product_optimization(&candidate, ProductOptimizationMode::Enabled);
    assert!(matches!(dec, ProductOptimizationDecision::Virtualize(_)));
}

#[test]
fn test_data_scalar_replacement_in_function() {
    let src = r#"
data Point(_ x: Int, _ y: Int)

class Compute {
  @class
  run() {
    let p = Point(10, 20)
    return p.x + p.y
  }
}

let res = Compute.run()
"#;
    let (vm, main_mod) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("should succeed");
    let res_sym = vm.interner.find("res").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(res_sym), Some(Value::int(30)));
}

#[test]
fn test_nested_data_scalar_replacement_in_function() {
    let src = r#"
data Point(_ x: Int, _ y: Int)
data Pair(_ first: Point, _ second: Point)

class Compute {
  @class
  run() {
    let p1 = Point(10, 20)
    let p2 = Point(30, 40)
    let pair = Pair(p1, p2)
    return pair.first.x + pair.second.y
  }
}

let res = Compute.run()
"#;
    let (vm, main_mod) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("should succeed");
    let res_sym = vm.interner.find("res").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(res_sym), Some(Value::int(50)));
}

#[test]
fn test_ephemeral_constructor_projection() {
    let src = r#"
data Point(_ x: Int, _ y: Int)

class Compute {
  @class
  run() {
    return Point(100, 200).y
  }
}

let res = Compute.run()
"#;
    let (vm, main_mod) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("should succeed");
    let res_sym = vm.interner.find("res").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(res_sym), Some(Value::int(200)));
}

#[test]
fn test_data_allocation_sinking_to_single_whole_use() {
    let src = r#"
data Point(_ x: Int, _ y: Int)

class Compute {
  @class
  identity(_ p) {
    return p
  }

  @class
  run() {
    let p = Point(10, 20)
    let sum = p.x + p.y
    let p_sink = Compute.identity(p)
    return sum + p_sink.x
  }
}

let res = Compute.run()
"#;
    let (vm, main_mod) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("should succeed");
    let res_sym = vm.interner.find("res").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(res_sym), Some(Value::int(40)));
}

#[test]
fn test_tuple_and_record_scalar_replacement_and_sinking() {
    let src = r#"
class Compute {
  @class
  identity(_ x) { return x }

  @class
  run() {
    let t = (10, 20)
    let r = #{ a: 30, b: 40 }
    let t_sink = Compute.identity(t)
    let r_sink = Compute.identity(r)
    return 100
  }
}

let res = Compute.run()
"#;
    let (vm_opt, mod_opt) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("opt mode should succeed");
    let (vm_can, mod_can) = run_inline_with_mode(src, ProductOptimizationMode::Disabled).expect("disabled mode should succeed");

    let res_sym = vm_opt.interner.find("res").unwrap();
    let val_opt = vm_opt.heap.module(mod_opt).get(res_sym);
    let val_can = vm_can.heap.module(mod_can).get(res_sym);

    assert_eq!(val_opt, Some(Value::int(100)));
    assert_eq!(val_opt, val_can);
}

#[test]
fn test_nested_mixed_product_scalar_replacement() {
    let src = r#"
data Point(_ x: Int, _ y: Int)

class Compute {
  @class
  run() {
    let pt = Point(1, 2)
    let t = (pt, 3)
    let r = #{ nested: t, flag: true }
    return 200
  }
}

let res = Compute.run()
"#;
    let (vm_opt, mod_opt) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("opt mode should succeed");
    let (vm_can, mod_can) = run_inline_with_mode(src, ProductOptimizationMode::Disabled).expect("disabled mode should succeed");

    let res_sym = vm_opt.interner.find("res").unwrap();
    let val_opt = vm_opt.heap.module(mod_opt).get(res_sym);
    let val_can = vm_can.heap.module(mod_can).get(res_sym);

    assert_eq!(val_opt, Some(Value::int(200)));
    assert_eq!(val_opt, val_can);
}
