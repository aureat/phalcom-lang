use super::*;
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
fn test_exact_differential_parity_enabled_vs_disabled() {
    let src = r#"
data Point(_ x: Int, _ y: Int)
data Line(_ start: Point, _ end: Point)

class TestClass {
  @class
  run() {
    let p1 = Point(5, 10)
    let p2 = Point(15, 20)
    let line = Line(p1, p2)
    let dx = line.end.x - line.start.x
    let dy = line.end.y - line.start.y
    return dx * dx + dy * dy
  }
}

let res = TestClass.run()
"#;
    let (vm_opt, mod_opt) = run_inline_with_mode(src, ProductOptimizationMode::Enabled).expect("opt mode should succeed");
    let (vm_can, mod_can) = run_inline_with_mode(src, ProductOptimizationMode::Disabled).expect("disabled mode should succeed");

    let res_sym = vm_opt.interner.find("res").unwrap();
    let val_opt = vm_opt.heap.module(mod_opt).get(res_sym);
    let val_can = vm_can.heap.module(mod_can).get(res_sym);

    assert_eq!(val_opt, Some(Value::int(200)));
    assert_eq!(val_opt, val_can);
}
