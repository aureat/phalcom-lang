use crate::semantic::incremental::support::{multi_module_input, single_module_input};
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::linker::{ImportBindingId, LinkedReadSpec, SymbolId};
use phalcom_semantic::checker::analysis::SemanticDependency;
use phalcom_semantic::db::{CancellationToken, QueryBudget, QueryOutcome};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{DeclarationId, DispatchSide, FieldId};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::static_constant::StaticConstantId;
use phalcom_semantic::static_eval::StaticEvalOutcome;
use phalcom_semantic::static_value::{StaticInt, StaticValueData};
use phalcom_semantic::types::store::TypeData;
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

#[test]
fn test_literal_const() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), "const A: Int = 42\n", 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id = StaticConstantId::new(module.clone(), "A");
    let product = update.snapshot.static_constant(&id).expect("static constant A");
    assert!(product.is_successful());
    let val_ref = product.value().expect("value");
    let val = update.snapshot.static_values().get(val_ref).expect("value data");
    assert_eq!(val, &StaticValueData::Int(StaticInt::Small(42)));
}

#[test]
fn test_computed_const() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), "const A: Int = 10 + 20\n", 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id = StaticConstantId::new(module.clone(), "A");
    let product = update.snapshot.static_constant(&id).expect("static constant A");
    assert!(product.is_successful());
    let val_ref = product.value().expect("value");
    let val = update.snapshot.static_values().get(val_ref).expect("value data");
    assert_eq!(val, &StaticValueData::Int(StaticInt::Small(30)));
}

#[test]
fn test_constant_annotation_projects_a_reduced_constant_record_key() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const Key: Symbol = #token\nconst Value: { [Key]: Int } = { [#token]: 1 }\n";

    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(
        !update.snapshot.has_errors(),
        "computed key annotation should form after Key reduces: {:?}",
        update.snapshot.diagnostics
    );

    let value = update
        .snapshot
        .static_constant(&StaticConstantId::new(module, "Value"))
        .expect("Value constant product");
    assert!(value.is_successful());
    assert!(
        value
            .checked
            .dependencies
            .contains(&SemanticDependency::StaticConstant(StaticConstantId::new(test_module(), "Key"))),
        "the published Value product must retain its exact computed-key dependency"
    );
}

#[test]
fn test_field_signature_projects_a_cold_computed_record_key() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const Key: Symbol = #token\nclass Box {\n  _value: { [Key]: Int }\n}\n";
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let field = FieldId::new(DeclarationId::new(module, "Box".into()), "_value", DispatchSide::Instance);
    let signature = update.snapshot.field_signatures.get(&field).expect("field signature");
    let ty = signature.declared_type.to_knowledge().ty().expect("field type should be known");
    let TypeData::Record(row) = update.snapshot.store.get(ty) else {
        panic!("computed-key field annotation should publish a Record type")
    };
    assert_eq!(
        update.snapshot.store.record_row(*row).fields[0].key,
        phalcom_semantic::field_key::FieldKey::symbol("token")
    );
    assert!(session
        .db()
        .index()
        .dependencies_of(&phalcom_semantic::db::QueryKey::FieldSignature(field))
        .is_some_and(|dependencies| dependencies.iter().any(|edge| {
            edge.dependency == phalcom_semantic::db::QueryKey::StaticConstant(StaticConstantId::new(test_module(), "Key"))
        })));
}

#[test]
fn test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly() {
    let module = test_module();
    let field = FieldId::new(DeclarationId::new(module.clone(), "Box".into()), "_value", DispatchSide::Instance);
    let source_a = "const Key: Symbol = #first\nclass Box {\n  _value: { [Key]: Int }\n}\n";
    let source_b = "const Key: Symbol = #second\nclass Box {\n  _value: { [Key]: Int }\n}\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let signature_a = first.snapshot.field_signatures.get(&field).cloned().expect("field signature revision 1");

    let updated = incremental.update(single_module_input(module.clone(), source_b, 2));
    assert!(!updated.snapshot.has_errors(), "diagnostics: {:?}", updated.snapshot.diagnostics);
    let signature_b = updated.snapshot.field_signatures.get(&field).expect("field signature revision 2");
    assert_ne!(signature_a, *signature_b, "the owner signature must change with its selected key");
    let ty = signature_b.declared_type.to_knowledge().ty().expect("field type");
    let TypeData::Record(row) = updated.snapshot.store.get(ty) else {
        panic!("field annotation must remain a Record type")
    };
    assert_eq!(updated.snapshot.store.record_row(*row).fields[0].key, phalcom_semantic::field_key::FieldKey::symbol("second"));

    let surface = updated.snapshot.dispatch.get_surface(&field.owner).expect("declaration surface");
    let surfaced = surface.get_field(DispatchSide::Instance, "_value").expect("surface field");
    let TypeData::Record(surface_row) = updated.snapshot.store.get(surfaced.ty().expect("surface field type")) else {
        panic!("surface field must consume the canonical computed-key signature")
    };
    assert_eq!(updated.snapshot.store.record_row(*surface_row).fields[0].key, phalcom_semantic::field_key::FieldKey::symbol("second"));

    let cold = SemanticWorkspaceSession::new().update(single_module_input(module, source_b, 1));
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);
    let cold_signature = cold.snapshot.field_signatures.get(&field).expect("cold field signature");
    let cold_ty = cold_signature.declared_type.to_knowledge().ty().expect("cold field type");
    let TypeData::Record(cold_row) = cold.snapshot.store.get(cold_ty) else {
        panic!("cold field annotation must remain a Record type")
    };
    assert_eq!(
        updated.snapshot.store.record_row(*row).fields[0].key,
        cold.snapshot.store.record_row(*cold_row).fields[0].key,
        "incremental owner product must match cold analysis"
    );
}

#[test]
fn test_callable_signature_projects_all_parameter_and_return_record_keys() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = r#"
const Key: Symbol = #token
class Box {
  @class transform(_ value: { [Key]: Int }) -> { [Key]: Int } { value }
}
"#;
    let update = session.update(single_module_input(module.clone(), source_a, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = phalcom_semantic::identity::CallableId::new(
        DeclarationId::new(module.clone(), "Box".into()),
        Selector::method("transform", [SelectorSlot::Positional]).expect("valid selector"),
        DispatchSide::Class,
    );
    let signature = update.snapshot.callable_signatures.get(&callable).expect("callable signature");
    let signature_a = signature.clone();
    let parameter_ty = signature.parameters[0].declared_type.to_knowledge().ty().expect("parameter Record type");
    let return_ty = signature.declared_return.to_knowledge().ty().expect("return Record type");
    for ty in [parameter_ty, return_ty] {
        let TypeData::Record(row) = update.snapshot.store.get(ty) else {
            panic!("callable annotation must publish a Record type")
        };
        assert_eq!(
            update.snapshot.store.record_row(*row).fields[0].key,
            phalcom_semantic::field_key::FieldKey::symbol("token")
        );
    }
    let dependencies = session.db().index().dependencies_of(&phalcom_semantic::db::QueryKey::CallableSignature(callable.clone()));
    assert!(dependencies.is_some_and(|dependencies| {
        dependencies.iter().any(|edge| {
            edge.dependency == phalcom_semantic::db::QueryKey::StaticConstant(StaticConstantId::new(test_module(), "Key"))
        })
    }), "callable owner must depend on the selected constant: {dependencies:?}");

    let source_b = source_a.replace("#token", "#second");
    let updated = session.update(single_module_input(module.clone(), &source_b, 2));
    assert!(!updated.snapshot.has_errors(), "incremental diagnostics: {:?}", updated.snapshot.diagnostics);
    let signature_b = updated.snapshot.callable_signatures.get(&callable).expect("updated callable signature");
    assert_ne!(&signature_a, signature_b, "the callable product must republish when its key dependency changes");
    for ty in [
        signature_b.parameters[0].declared_type.to_knowledge().ty().expect("updated parameter type"),
        signature_b.declared_return.to_knowledge().ty().expect("updated return type"),
    ] {
        let TypeData::Record(row) = updated.snapshot.store.get(ty) else {
            panic!("updated callable annotation must remain a Record type")
        };
        assert_eq!(
            updated.snapshot.store.record_row(*row).fields[0].key,
            phalcom_semantic::field_key::FieldKey::symbol("second")
        );
    }

    let cold = SemanticWorkspaceSession::new().update(single_module_input(module, &source_b, 1));
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);
    let cold_signature = cold.snapshot.callable_signatures.get(&callable).expect("cold callable signature");
    let cold_ty = cold_signature.declared_return.to_knowledge().ty().expect("cold return type");
    let TypeData::Record(cold_row) = cold.snapshot.store.get(cold_ty) else {
        panic!("cold callable return must remain a Record type")
    };
    let updated_ty = signature_b.declared_return.to_knowledge().ty().expect("updated return type");
    let TypeData::Record(updated_row) = updated.snapshot.store.get(updated_ty) else {
        panic!("updated callable return must remain a Record type")
    };
    assert_eq!(
        updated.snapshot.store.record_row(*updated_row).fields[0].key,
        cold.snapshot.store.record_row(*cold_row).fields[0].key,
        "incremental callable projection must match cold analysis"
    );
}

#[test]
fn test_constant_annotation_key_cycle_is_diagnosed_as_a_static_constant_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const A: Symbol = B\nconst B: { [A]: Int } = { [#token]: 1 }\n";

    let update = session.update(single_module_input(module.clone(), source, 1));
    let diagnostics = update.snapshot.diagnostics.get(&module).expect("cycle diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::StaticConstantCycle),
        "computed-key formation cycle must use the static constant cycle path: {diagnostics:?}"
    );
    assert_eq!(
        update
            .snapshot
            .static_constant(&StaticConstantId::new(module, "B"))
            .expect("B constant product")
            .outcome,
        StaticEvalOutcome::Recursive
    );
}

#[test]
fn test_transitive_constant_formation_cycle_retains_computed_key_cause() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const A: Int = B\nconst B: { [C]: Int } = { [#token]: 1 }\nconst C: Symbol = A\n";

    let update = session.update(single_module_input(module.clone(), source, 1));
    let diagnostics = update.snapshot.diagnostics.get(&module).expect("cycle diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::StaticConstantCycle && diagnostic.message.contains("computed Record key")
        }),
        "the transitive mixed cycle must retain its computed-key formation cause: {diagnostics:?}"
    );
    for name in ["A", "B", "C"] {
        assert_eq!(
            update
                .snapshot
                .static_constant(&StaticConstantId::new(module.clone(), name))
                .expect("cycle constant product")
                .outcome,
            StaticEvalOutcome::Recursive,
            "{name} must not retain a successful current product"
        );
    }
}

#[test]
fn test_imported_constant_projects_a_computed_record_key_with_remote_identity() {
    let provider = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("provider").unwrap()]),
    );
    let consumer = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("consumer").unwrap()]),
    );
    let mut input = multi_module_input(
        vec![
            (provider.clone(), "const Key: Symbol = #token\n".to_owned()),
            (consumer.clone(), "const Value: { [Key]: Int } = { [#token]: 1 }\n".to_owned()),
        ],
        1,
    );
    let mut linked = (*input.linked).clone();
    let consumer_module = linked.modules.get_mut(&consumer).expect("consumer module");
    consumer_module.bindings.imports.insert("Key".into(), ImportBindingId(0));
    consumer_module.linked_reads.push(LinkedReadSpec::Binding(SymbolId {
        module: provider.clone(),
        name: "Key".into(),
    }));
    consumer_module.runtime_dependencies.push(provider.clone());
    input.linked = Arc::new(linked);

    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input);
    assert!(
        !update.snapshot.has_errors(),
        "an imported constant must project through its canonical remote symbol: {:?}",
        update.snapshot.diagnostics
    );

    let value = update
        .snapshot
        .static_constant(&StaticConstantId::new(consumer.clone(), "Value"))
        .expect("consumer Value constant");
    assert!(value.is_successful());
    assert!(value.checked.dependencies.contains(&SemanticDependency::StaticConstant(StaticConstantId::new(provider, "Key"))));
}

#[test]
fn test_constant_key_edit_republishes_only_its_annotation_consumer() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source1 = "const Key: Symbol = #first\nconst Value: { [Key]: Int } = { [#first]: 1 }\nconst Other: Int = 7\n";
    let update1 = session.update(single_module_input(module.clone(), source1, 1));
    assert!(!update1.snapshot.has_errors(), "diagnostics: {:?}", update1.snapshot.diagnostics);
    let value_id = StaticConstantId::new(module.clone(), "Value");
    let other_id = StaticConstantId::new(module.clone(), "Other");
    let value1 = update1.snapshot.static_constant(&value_id).cloned().expect("Value revision 1");
    let other1 = update1.snapshot.static_constant(&other_id).cloned().expect("Other revision 1");

    let source2 = "const Key: Symbol = #other\nconst Value: { [Key]: Int } = { [#first]: 1 }\nconst Other: Int = 7\n";
    let update2 = session.update(single_module_input(module, source2, 2));
    let value2 = update2.snapshot.static_constant(&value_id).expect("Value revision 2");
    let other2 = update2.snapshot.static_constant(&other_id).expect("Other revision 2");
    assert!(!Arc::ptr_eq(&value1, value2), "Value must be republished after its selected key changes");
    assert!(Arc::ptr_eq(&other1, other2), "unrelated constant must remain reusable");

    let expected = value2.checked.result_type.expect("Value declared type");
    let TypeData::Record(row_id) = update2.snapshot.store.get(expected) else {
        panic!("Value annotation must remain a Record type")
    };
    assert_eq!(
        update2.snapshot.store.record_row(*row_id).fields[0].key,
        phalcom_semantic::field_key::FieldKey::symbol("other")
    );
}

#[test]
fn test_same_result_key_source_edit_revalidates_its_annotation_consumer() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let value_id = StaticConstantId::new(module.clone(), "Value");
    let source1 = "const Key: Symbol = #stable\nconst Value: { [Key]: Int } = { [#stable]: 1 }\n";
    let update1 = session.update(single_module_input(module.clone(), source1, 1));
    assert!(!update1.snapshot.has_errors(), "diagnostics: {:?}", update1.snapshot.diagnostics);
    let value1 = update1.snapshot.static_constant(&value_id).cloned().expect("Value revision 1");

    // Parenthesizing the initializer changes its checked source product while
    // preserving the reduced FieldKey. The exact dependency must be
    // revalidated without disturbing unrelated products.
    let source2 = "const Key: Symbol = (#stable)\nconst Value: { [Key]: Int } = { [#stable]: 1 }\n";
    let update2 = session.update(single_module_input(module.clone(), source2, 2));
    assert!(!update2.snapshot.has_errors(), "diagnostics: {:?}", update2.snapshot.diagnostics);
    let value2 = update2.snapshot.static_constant(&value_id).expect("Value revision 2");
    assert!(
        !Arc::ptr_eq(&value1, value2),
        "the annotation owner must revalidate a selected key even when its reduced FieldKey is unchanged"
    );
    assert!(value2.checked.dependencies.contains(&SemanticDependency::StaticConstant(StaticConstantId::new(module.clone(), "Key"))));

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(
        module.clone(),
        "const Key: Symbol = (#stable)\nconst Value: { [Key]: Int } = { [#stable]: 1 }\n",
        1,
    ));
    assert_eq!(update2.snapshot.diagnostics.get(&module), cold.snapshot.diagnostics.get(&module));
    let cold_value = cold.snapshot.static_constant(&value_id).expect("cold Value");
    assert!(value2.is_successful() && cold_value.is_successful());
    assert_eq!(
        update2.snapshot.static_values().get(value2.value().expect("incremental value")),
        cold.snapshot.static_values().get(cold_value.value().expect("cold value"))
    );
    let incremental_type = value2.checked.result_type.expect("incremental Value type");
    let cold_type = cold_value.checked.result_type.expect("cold Value type");
    let (TypeData::Record(incremental_row), TypeData::Record(cold_row)) =
        (update2.snapshot.store.get(incremental_type), cold.snapshot.store.get(cold_type))
    else {
        panic!("both Value annotations must form Record types")
    };
    assert_eq!(update2.snapshot.store.record_row(*incremental_row), cold.snapshot.store.record_row(*cold_row));
}

#[test]
fn test_const_to_const_dependency() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const A: Int = 10\nconst B: Int = A + 5\n";
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a = update.snapshot.static_constant(&id_a).expect("static constant A");
    assert!(prod_a.is_successful());

    let prod_b = update.snapshot.static_constant(&id_b).expect("static constant B");
    assert!(prod_b.is_successful());
    assert!(prod_b.checked.static_constant_reads.contains(id_a.symbol()), "B should record dependency on A");

    let val_b = update.snapshot.static_values().get(prod_b.value().unwrap()).unwrap();
    assert_eq!(val_b, &StaticValueData::Int(StaticInt::Small(15)));
}

#[test]
fn test_key_discriminator_same_module_isolation() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let src1 = "const A: Int = 10\nconst B: Int = 20\n";
    let update1 = session.update(single_module_input(module.clone(), src1, 1));
    assert!(!update1.snapshot.has_errors());

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a1 = update1.snapshot.static_constant(&id_a).cloned().expect("A rev 1");
    let prod_b1 = update1.snapshot.static_constant(&id_b).cloned().expect("B rev 1");

    // Rev 2: Edit A to 15, leave B = 20 unchanged.
    let src2 = "const A: Int = 15\nconst B: Int = 20\n";
    let update2 = session.update(single_module_input(module.clone(), src2, 2));
    assert!(!update2.snapshot.has_errors());

    let prod_a2 = update2.snapshot.static_constant(&id_a).cloned().expect("A rev 2");
    let prod_b2 = update2.snapshot.static_constant(&id_b).cloned().expect("B rev 2");

    // A must recompute to 15
    assert!(!Arc::ptr_eq(&prod_a1, &prod_a2));
    let val_a2 = update2.snapshot.static_values().get(prod_a2.value().unwrap()).unwrap();
    assert_eq!(val_a2, &StaticValueData::Int(StaticInt::Small(15)));

    // B must be a cache hit (Arc::ptr_eq preserved)
    assert!(Arc::ptr_eq(&prod_b1, &prod_b2), "B should be reused from cache without recomputation");
    let val_b2 = update2.snapshot.static_values().get(prod_b2.value().unwrap()).unwrap();
    assert_eq!(val_b2, &StaticValueData::Int(StaticInt::Small(20)));
}

#[test]
fn test_transitive_invalidation_same_type() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let src1 = "const A: Int = 10\nconst B: Int = A + 1\n";
    let update1 = session.update(single_module_input(module.clone(), src1, 1));
    assert!(!update1.snapshot.has_errors());

    let _id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_b1 = update1.snapshot.static_constant(&id_b).cloned().expect("B rev 1");
    let val_b1 = update1.snapshot.static_values().get(prod_b1.value().unwrap()).unwrap();
    assert_eq!(val_b1, &StaticValueData::Int(StaticInt::Small(11)));

    // Rev 2: Edit A to 20. A's type is still Int, but its value changed.
    let src2 = "const A: Int = 20\nconst B: Int = A + 1\n";
    let update2 = session.update(single_module_input(module.clone(), src2, 2));
    assert!(!update2.snapshot.has_errors());

    let prod_b2 = update2.snapshot.static_constant(&id_b).cloned().expect("B rev 2");
    assert!(!Arc::ptr_eq(&prod_b1, &prod_b2), "B must be invalidated when A changes value");
    let val_b2 = update2.snapshot.static_values().get(prod_b2.value().unwrap()).unwrap();
    assert_eq!(val_b2, &StaticValueData::Int(StaticInt::Small(21)));
}

#[test]
fn test_runtime_dependency_rejection() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "let runtime_x = 10\nconst C = runtime_x\n";
    let update = session.update(single_module_input(module.clone(), source, 1));

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::TypeOnlyRuntimeValue),
        "expected TypeOnlyRuntimeValue diagnostic, got: {:?}",
        diags
    );

    let id_c = StaticConstantId::new(module.clone(), "C");
    if let Some(prod_c) = update.snapshot.static_constant(&id_c) {
        assert!(!prod_c.is_successful(), "C must not reduce successfully with runtime dependency");
    }
}

#[test]
fn test_self_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "const A: Int = A\n";
    let update = session.update(single_module_input(module.clone(), source, 1));

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::StaticConstantCycle),
        "expected StaticConstantCycle diagnostic, got: {:?}",
        diags
    );

    let id_a = StaticConstantId::new(module.clone(), "A");
    let prod_a = update.snapshot.static_constant(&id_a).expect("constant product");
    assert_eq!(prod_a.outcome, StaticEvalOutcome::Recursive);
    assert!(!prod_a.is_successful());
}

#[test]
fn test_mutual_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "const A: Int = B\nconst B: Int = A\n";
    let update = session.update(single_module_input(module.clone(), source, 1));

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::StaticConstantCycle),
        "expected StaticConstantCycle diagnostic, got: {:?}",
        diags
    );

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a = update.snapshot.static_constant(&id_a).expect("constant product A");
    let prod_b = update.snapshot.static_constant(&id_b).expect("constant product B");

    assert_eq!(prod_a.outcome, StaticEvalOutcome::Recursive);
    assert_eq!(prod_b.outcome, StaticEvalOutcome::Recursive);
}

#[test]
fn test_evaluation_controls_and_lkg() {
    let module = test_module();

    // 1. Cancelled token
    {
        let mut session = SemanticWorkspaceSession::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let res = session.update_with_budget_and_cancel(single_module_input(module.clone(), "const A: Int = 42\n", 1), QueryBudget::default(), &cancel);
        assert!(matches!(res, Err(QueryOutcome::Cancelled)));
    }

    // 2. Zero budget
    {
        let mut session = SemanticWorkspaceSession::new();
        let cancel = CancellationToken::new();
        let zero_budget = QueryBudget {
            max_steps: 0,
            ..QueryBudget::default()
        };
        let res = session.update_with_budget_and_cancel(single_module_input(module.clone(), "const A: Int = 42\n", 1), zero_budget, &cancel);
        assert!(matches!(res, Err(QueryOutcome::BudgetExceeded(_))));
    }

    // 3. LKG persistence
    {
        let mut session = SemanticWorkspaceSession::new();

        // Rev 1: Valid constant
        let update1 = session.update(single_module_input(module.clone(), "const A: Int = 42\n", 1));
        assert!(!update1.snapshot.has_errors());
        let id_a = StaticConstantId::new(module.clone(), "A");
        let prod_a1 = update1.snapshot.static_constant(&id_a).expect("A rev 1");
        assert!(prod_a1.is_successful());
        let val1 = prod_a1.value().unwrap();
        assert_eq!(prod_a1.lkg_value, Some(val1));

        // Rev 2: Introduce cycle (eval fails)
        let update2 = session.update(single_module_input(module.clone(), "const A: Int = A\n", 2));
        let prod_a2 = update2.snapshot.static_constant(&id_a).expect("A rev 2");
        assert!(!prod_a2.is_successful());
        assert_eq!(prod_a2.value, None);
        assert_eq!(prod_a2.lkg_value, Some(val1), "LKG value must persist across failed revision");
    }
}
