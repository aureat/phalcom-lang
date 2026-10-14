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
use phalcom_semantic::static_eval::{StaticEvalError, StaticEvalOutcome, StaticEvaluator};
use phalcom_semantic::static_eval::residual::{StaticResidualParameter, StaticResidualStore};
use phalcom_semantic::static_value::{StaticInt, StaticSequenceFamily, StaticValueData};
use phalcom_semantic::types::store::TypeData;
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn registered_query_projects() -> (tempfile::TempDir, phalcom_modules::ProjectUniverse) {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.ph"), "const A: Int = 1\n").unwrap();
    let mut projects = phalcom_modules::ProjectUniverse::new();
    let id = projects.load_synthetic_root("test", root.path(), "main").unwrap();
    assert_eq!(id, ResolvedProjectId::from_raw(1));
    (root, projects)
}

#[test]
fn test_literal_const() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), "const A: Int = 42\n", 1)).expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id = StaticConstantId::new(module.clone(), "A");
    let product = update.snapshot.static_constant(&id).expect("static constant A");
    assert!(product.is_successful());
    let val_ref = product.value().expect("value");
    let val = update.snapshot.static_values().get(val_ref).expect("value data");
    assert_eq!(val, &StaticValueData::Int(StaticInt::Small(42)));
}

#[test]
fn symbolic_constant_fingerprint_does_not_depend_on_residual_allocation() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(single_module_input(module.clone(), "const A: Int = 1\n", 1)).unwrap().snapshot;
    let original = snapshot.static_constant(&StaticConstantId::new(module, "A")).expect("static constant product");
    let parameter = StaticResidualParameter {
        owner: DeclarationId::new(ModuleId::universe_root(), "Input".into()),
        index: 0,
        domain: phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
    };
    let mut first_store = StaticResidualStore::new();
    let first_ref = first_store.parameter(parameter.clone(), &mut QueryBudget::default()).unwrap();
    let mut second_store = StaticResidualStore::new();
    let decoy = StaticResidualParameter { index: 1, ..parameter.clone() };
    second_store.parameter(decoy, &mut QueryBudget::default()).unwrap();
    let second_ref = second_store.parameter(parameter, &mut QueryBudget::default()).unwrap();

    let mut first = original.as_ref().clone();
    first.outcome = StaticEvalOutcome::Symbolic(first_ref);
    first.residuals = Some(Arc::new(first_store));
    first.value = None;
    let mut second = first.clone();
    second.outcome = StaticEvalOutcome::Symbolic(second_ref);
    second.residuals = Some(Arc::new(second_store));
    let (_root, projects) = registered_query_projects();
    let left = phalcom_semantic::db::fingerprint::static_constant_product_fingerprint(&first, snapshot.static_values(), snapshot.store(), &projects).unwrap();
    let right = phalcom_semantic::db::fingerprint::static_constant_product_fingerprint(&second, snapshot.static_values(), snapshot.store(), &projects).unwrap();
    assert_eq!(left, right, "equivalent residual DAGs must publish the same product fingerprint");
    first.residuals = None;
    assert!(
        phalcom_semantic::db::fingerprint::static_constant_product_fingerprint(&first, snapshot.static_values(), snapshot.store(), &projects).is_err(),
        "publication must fail closed when a symbolic product loses its residual store"
    );
}

#[test]
fn symbolic_constant_fingerprint_uses_logical_project_identity() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.ph"), "const A: Int = 1\n").unwrap();
    let other_root = tempfile::tempdir().unwrap();
    let mut first_projects = phalcom_modules::ProjectUniverse::new();
    let first_id = first_projects.load_synthetic_root("same", root.path(), "main").unwrap();
    let mut second_projects = phalcom_modules::ProjectUniverse::new();
    second_projects.load_synthetic_root("decoy", other_root.path(), "main").unwrap();
    let second_id = second_projects.load_synthetic_root("same", root.path(), "main").unwrap();
    assert_ne!(first_id, second_id);

    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(single_module_input(test_module(), "const A: Int = 1\n", 1)).unwrap().snapshot;
    let original = snapshot.static_constant(&StaticConstantId::new(test_module(), "A")).unwrap();
    let mut residuals = StaticResidualStore::new();
    let reference = residuals.parameter(
        StaticResidualParameter {
            owner: DeclarationId::new(ModuleId::universe_root(), "Input".into()),
            index: 0,
            domain: phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
        },
        &mut QueryBudget::default(),
    ).unwrap();
    let mut first = original.as_ref().clone();
    first.id = StaticConstantId::new(ModuleId::resolved(first_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()])), "A");
    first.outcome = StaticEvalOutcome::Symbolic(reference);
    first.residuals = Some(Arc::new(residuals));
    first.value = None;
    let mut second = first.clone();
    second.id = StaticConstantId::new(ModuleId::resolved(second_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()])), "A");
    let fingerprint = phalcom_semantic::db::fingerprint::static_constant_product_fingerprint;
    assert_eq!(
        fingerprint(&first, snapshot.static_values(), snapshot.store(), &first_projects).unwrap(),
        fingerprint(&second, snapshot.static_values(), snapshot.store(), &second_projects).unwrap(),
        "allocation order of ProjectUniverse IDs cannot change a symbolic root product"
    );
}

#[test]
fn symbolic_constant_fingerprint_projects_value_meaning_across_value_stores() {
    use phalcom_semantic::static_eval::residual::{StaticEvalTerm, StaticResidualApplication};

    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(single_module_input(test_module(), "const A: Int = 1\n", 1)).unwrap().snapshot;
    let original = snapshot.static_constant(&StaticConstantId::new(test_module(), "A")).unwrap();
    let (_root, projects) = registered_query_projects();
    let owner = DeclarationId::new(ModuleId::universe_root(), "Application".into());

    let mut first_values = phalcom_semantic::static_value::StaticValueStore::new();
    let first_true = first_values.intern_bool(true);
    let mut second_values = phalcom_semantic::static_value::StaticValueStore::new();
    second_values.intern_int(500);
    let second_true = second_values.intern_bool(true);
    let second_false = second_values.intern_bool(false);

    let make = |value, owner: DeclarationId| {
        let mut store = StaticResidualStore::new();
        let residual = store.apply(
            StaticResidualApplication {
                definition: owner,
                arguments: Box::new([StaticEvalTerm::Value(value)]),
                environment: Default::default(),
            },
            phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
            &mut QueryBudget::default(),
        ).unwrap();
        let mut product = original.as_ref().clone();
        product.outcome = StaticEvalOutcome::Symbolic(residual);
        product.residuals = Some(Arc::new(store));
        product.value = None;
        product
    };
    let first = make(first_true, owner.clone());
    let second = make(second_true, owner.clone());
    let different = make(second_false, owner);
    let fingerprint = phalcom_semantic::db::fingerprint::static_constant_product_fingerprint;
    let left = fingerprint(&first, &first_values, snapshot.store(), &projects).unwrap();
    assert_eq!(left, fingerprint(&second, &second_values, snapshot.store(), &projects).unwrap());
    assert_ne!(left, fingerprint(&different, &second_values, snapshot.store(), &projects).unwrap());
}

#[test]
fn symbolic_constant_fingerprint_projects_type_meaning_across_type_stores() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(single_module_input(module.clone(), "const A: Int = 1\n", 1)).unwrap().snapshot;
    let original = snapshot.static_constant(&StaticConstantId::new(module, "A")).unwrap();
    let int_decl = phalcom_semantic::core_surface::identity::CoreDeclarationIds::default().int;
    let original_int = snapshot.declarations.form(&int_decl).expect("canonical Int type");
    let mut independent = phalcom_semantic::types::store::TypeStore::new();
    independent.nominal_type(DeclarationId::new(ModuleId::universe_root(), "AllocationDecoy".into()));
    let independent_int = independent.nominal_type(int_decl);
    assert_ne!(original_int, independent_int, "two stores must use different arena IDs for the discriminator");

    let owner = DeclarationId::new(ModuleId::universe_root(), "Open".into());
    let mut first_store = StaticResidualStore::new();
    let first_ref = first_store.parameter(
        StaticResidualParameter {
            owner: owner.clone(), index: 0,
            domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(original_int),
        },
        &mut QueryBudget::default(),
    ).unwrap();
    let mut second_store = StaticResidualStore::new();
    let second_ref = second_store.parameter(
        StaticResidualParameter {
            owner, index: 0,
            domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(independent_int),
        },
        &mut QueryBudget::default(),
    ).unwrap();
    let mut first = original.as_ref().clone();
    first.outcome = StaticEvalOutcome::Symbolic(first_ref);
    first.residuals = Some(Arc::new(first_store));
    first.value = None;
    let mut second = first.clone();
    second.outcome = StaticEvalOutcome::Symbolic(second_ref);
    second.residuals = Some(Arc::new(second_store));
    let (_root, projects) = registered_query_projects();
    let values = snapshot.static_values();
    let fingerprint = phalcom_semantic::db::fingerprint::static_constant_product_fingerprint;
    assert_eq!(
        fingerprint(&first, values, snapshot.store(), &projects).unwrap(),
        fingerprint(&second, values, &independent, &projects).unwrap(),
        "equal nominal meanings must not inherit different TypeId allocations"
    );
}

#[test]
fn symbolic_constant_fingerprint_projects_pack_result_type_and_ordered_lanes() {
    use phalcom_semantic::static_eval::residual::StaticEvalTerm;
    use phalcom_semantic::types::store::PackTypeElement;

    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(single_module_input(test_module(), "const A: Int = 1\n", 1)).unwrap().snapshot;
    let original = snapshot.static_constant(&StaticConstantId::new(test_module(), "A")).unwrap();
    let (_root, projects) = registered_query_projects();
    let int_decl = phalcom_semantic::core_surface::identity::CoreDeclarationIds::default().int;
    let mut left_types = snapshot.store().clone();
    let left_int = Arc::make_mut(&mut left_types).nominal_type(int_decl.clone());
    let left_pack_type = Arc::make_mut(&mut left_types).pack(Box::new([
        PackTypeElement::unlabeled(left_int), PackTypeElement::labeled("first".into(), left_int),
        PackTypeElement::labeled("second".into(), left_int),
    ])).unwrap();
    let mut right_types = phalcom_semantic::types::store::TypeStore::new();
    right_types.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Decoy".into()));
    let right_int = right_types.nominal_type(int_decl);
    let right_pack_type = right_types.pack(Box::new([
        PackTypeElement::unlabeled(right_int), PackTypeElement::labeled("first".into(), right_int),
        PackTypeElement::labeled("second".into(), right_int),
    ])).unwrap();
    assert_ne!(left_pack_type, right_pack_type);

    let make = |int_type, pack_type, decoy| {
        let mut values = phalcom_semantic::static_value::StaticValueStore::new();
        if decoy { values.intern_int(999); }
        let one = values.intern_int(1);
        let two = values.intern_int(2);
        let mut residuals = StaticResidualStore::new();
        if decoy {
            residuals.parameter(StaticResidualParameter {
                owner: DeclarationId::new(ModuleId::universe_root(), "Decoy".into()), index: 0,
                domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(int_type),
            }, &mut QueryBudget::default()).unwrap();
        }
        let input = residuals.parameter(StaticResidualParameter {
            owner: DeclarationId::new(ModuleId::universe_root(), "Input".into()), index: 0,
            domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(int_type),
        }, &mut QueryBudget::default()).unwrap();
        let pack = residuals.pack(
            Box::new([StaticEvalTerm::Residual(input)]),
            Box::new([Box::<str>::from("first"), Box::<str>::from("second")]),
            Box::new([StaticEvalTerm::Value(one), StaticEvalTerm::Value(two)]),
            pack_type, &mut QueryBudget::default(),
        ).unwrap();
        let mut product = original.as_ref().clone();
        product.outcome = StaticEvalOutcome::Symbolic(pack);
        product.residuals = Some(Arc::new(residuals));
        product.value = None;
        (product, values)
    };
    let (left, left_values) = make(left_int, left_pack_type, false);
    let (right, right_values) = make(right_int, right_pack_type, true);
    let fingerprint = phalcom_semantic::db::fingerprint::static_constant_product_fingerprint;
    let expected = fingerprint(&left, &left_values, &left_types, &projects).unwrap();
    assert_eq!(expected, fingerprint(&right, &right_values, &right_types, &projects).unwrap(),
        "Pack result type, symbolic input and ordered Symbol lanes must fingerprint by meaning");
    let mut different_types = right_types.clone();
    let swapped_type = different_types.pack(Box::new([
        PackTypeElement::unlabeled(right_int), PackTypeElement::labeled("second".into(), right_int),
        PackTypeElement::labeled("first".into(), right_int),
    ])).unwrap();
    let (different, different_values) = make(right_int, swapped_type, false);
    assert_ne!(expected, fingerprint(&different, &different_values, &different_types, &projects).unwrap(),
        "a distinct checked Pack result type must change the symbolic product fingerprint");
}

#[test]
fn symbolic_constant_fingerprint_walks_deep_residual_dag_without_host_recursion() {
    use phalcom_semantic::static_eval::residual::{StaticEvalTerm, StaticResidualApplication};

    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(single_module_input(module.clone(), "const A: Int = 1\n", 1)).unwrap().snapshot;
    let original = snapshot.static_constant(&StaticConstantId::new(module, "A")).unwrap();
    let owner = DeclarationId::new(ModuleId::universe_root(), "Deep".into());
    let mut store = StaticResidualStore::new();
    let mut budget = QueryBudget::default();
    let mut current = store.parameter(
        StaticResidualParameter {
            owner: owner.clone(), index: 0,
            domain: phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
        },
        &mut budget,
    ).unwrap();
    for _ in 0..4096 {
        current = store.apply(
            StaticResidualApplication {
                definition: owner.clone(),
                arguments: Box::new([StaticEvalTerm::Residual(current)]),
                environment: Default::default(),
            },
            phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
            &mut budget,
        ).unwrap();
    }
    let mut product = original.as_ref().clone();
    product.outcome = StaticEvalOutcome::Symbolic(current);
    product.residuals = Some(Arc::new(store));
    product.value = None;
    let (_root, projects) = registered_query_projects();
    phalcom_semantic::db::fingerprint::static_constant_product_fingerprint(
        &product,
        snapshot.static_values(),
        snapshot.store(),
        &projects,
    ).expect("deep but bounded residual must fingerprint without host recursion");
}

#[test]
fn test_computed_const() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), "const A: Int = 10 + 20\n", 1)).expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id = StaticConstantId::new(module.clone(), "A");
    let product = update.snapshot.static_constant(&id).expect("static constant A");
    assert!(product.is_successful());
    let val_ref = product.value().expect("value");
    let val = update.snapshot.static_values().get(val_ref).expect("value data");
    assert_eq!(val, &StaticValueData::Int(StaticInt::Small(30)));
}

#[test]
fn test_static_sequence_constants_reduce_through_checked_array_and_vector_products() {
    let module = test_module();
    let source = r#"
const array_value: Array<Int> = <Int>[1, 2]
const vector_value: Vector<Int, 2> = <Int, 2>[3, 4]
const nested_value: Array<Array<Int>> = [<Int>[5], <Int>[6]]
const empty_vector: Vector<Int, 0> = []
const tuple_source: (Int, Int) = (7, 8)
const tuple_expanded: Array<Int> = [*tuple_source]
const vector_source: Vector<Int, 2> = <Int, 2>[9, 10]
const vector_expanded: Vector<Int, 2> = <Int, 2>[*vector_source]
"#;
    let update = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let family_and_elements = |name: &str| {
        let product = update
            .snapshot
            .static_constant(&StaticConstantId::new(module.clone(), name))
            .unwrap_or_else(|| panic!("missing static constant {name}"));
        assert!(product.is_successful(), "{name} did not reduce: {:?}", product.outcome);
        let StaticValueData::Sequence(id) = update.snapshot.static_values().get(product.value().unwrap()).unwrap() else {
            panic!("{name} must be a canonical StaticSequence")
        };
        let sequence = update.snapshot.static_values().get_sequence(*id).unwrap();
        let children = sequence
            .elements()
            .iter()
            .map(|child| update.snapshot.static_values().get(*child).unwrap().clone())
            .collect::<Vec<_>>();
        (sequence.family(), children)
    };

    assert_eq!(family_and_elements("array_value"), (StaticSequenceFamily::Array, vec![
        StaticValueData::Int(StaticInt::Small(1)),
        StaticValueData::Int(StaticInt::Small(2)),
    ]));
    assert_eq!(family_and_elements("vector_value"), (StaticSequenceFamily::Vector, vec![
        StaticValueData::Int(StaticInt::Small(3)),
        StaticValueData::Int(StaticInt::Small(4)),
    ]));
    let (nested_family, nested_elements) = family_and_elements("nested_value");
    assert_eq!(nested_family, StaticSequenceFamily::Array);
    assert_eq!(nested_elements.len(), 2);
    for child in nested_elements {
        let StaticValueData::Sequence(id) = child else {
            panic!("nested child must remain a canonical sequence")
        };
        assert_eq!(update.snapshot.static_values().get_sequence(id).unwrap().family(), StaticSequenceFamily::Array);
    }
    assert_eq!(family_and_elements("empty_vector"), (StaticSequenceFamily::Vector, Vec::new()));
    assert_eq!(family_and_elements("tuple_expanded"), (StaticSequenceFamily::Array, vec![
        StaticValueData::Int(StaticInt::Small(7)),
        StaticValueData::Int(StaticInt::Small(8)),
    ]));
    assert_eq!(family_and_elements("vector_expanded"), (StaticSequenceFamily::Vector, vec![
        StaticValueData::Int(StaticInt::Small(9)),
        StaticValueData::Int(StaticInt::Small(10)),
    ]));
}

#[test]
fn test_static_sequence_list_and_missing_proof_fail_closed() {
    let module = test_module();
    let update = SemanticWorkspaceSession::new().update(single_module_input(
        module.clone(),
        "const legacy: List<Int> = [1, 2]\n",
        1,
    )).expect("semantic test update should succeed");
    let product = update
        .snapshot
        .static_constant(&StaticConstantId::new(module, "legacy"))
        .expect("List constant product");
    assert_eq!(
        product.outcome,
        StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedExpression),
        "static List literals must not acquire frozen Sequence semantics"
    );
    assert!(product.value().is_none());

    let module = test_module();
    let update = SemanticWorkspaceSession::new().update(single_module_input(
        module.clone(),
        "const value: Array<Int> = <Int>[1, 2]\n",
        1,
    )).expect("semantic test update should succeed");
    let product = update
        .snapshot
        .static_constant(&StaticConstantId::new(module, "value"))
        .expect("Array constant product");
    assert!(product.is_successful(), "setup constant should reduce: {:?}", product.outcome);

    let mut missing_proof = product.checked.as_ref().unwrap().as_ref().clone();
    let mut checked_expression = missing_proof.expression.clone();
    checked_expression.sequence_literals = Arc::new(Default::default());
    missing_proof.expression = checked_expression;
    let mut values = phalcom_semantic::static_value::StaticValueStore::new();
    let outcome = StaticEvaluator::for_constant(
        &missing_proof,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
    )
    .with_type_store(update.snapshot.store())
    .evaluate();
    assert_eq!(outcome, StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedExpression));
    assert_eq!(values.len(), 0, "a missing checker product must stop before child/root publication");
}

#[test]
fn test_static_sequence_budget_and_cancellation_do_not_publish_roots() {
    let module = test_module();
    let update = SemanticWorkspaceSession::new().update(single_module_input(
        module.clone(),
        "const value: Array<Int> = <Int>[1, 2, 3, 4]\n",
        1,
    )).expect("semantic test update should succeed");
    let product = update
        .snapshot
        .static_constant(&StaticConstantId::new(module, "value"))
        .expect("Array constant product");
    assert!(product.is_successful(), "setup constant should reduce: {:?}", product.outcome);

    let mut values = phalcom_semantic::static_value::StaticValueStore::new();
    let mut budget = QueryBudget::new(0);
    let bounded = StaticEvaluator::for_constant(product.checked.as_deref().unwrap(), &mut values, &mut budget, &CancellationToken::new())
        .with_type_store(update.snapshot.store())
        .evaluate();
    assert!(matches!(bounded, StaticEvalOutcome::BudgetExceeded(_)));
    assert_eq!(values.len(), 0, "budget exhaustion must precede sequence-root publication");

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled_values = phalcom_semantic::static_value::StaticValueStore::new();
    let cancelled = StaticEvaluator::for_constant(
        product.checked.as_deref().unwrap(),
        &mut cancelled_values,
        &mut QueryBudget::default(),
        &cancellation,
    )
    .with_type_store(update.snapshot.store())
    .evaluate();
    assert_eq!(cancelled, StaticEvalOutcome::Cancelled);
    assert_eq!(cancelled_values.len(), 0, "cancellation must not publish a sequence root");
}

#[test]
fn test_unconstrained_empty_static_sequence_remains_diagnostic_after_cutover() {
    let module = test_module();
    let update = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), "const empty = []\n", 1)).expect("semantic test update should succeed");
    let diagnostics = update.snapshot.diagnostics.get(&module).expect("diagnostics for underconstrained empty literal");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::GenericInferenceUnderconstrained),
        "an empty static literal without result evidence must remain underconstrained: {diagnostics:?}"
    );
    if let Some(product) = update.snapshot.static_constant(&StaticConstantId::new(module, "empty")) {
        assert!(!product.is_successful(), "underconstrained empty literal must not publish a static value");
    }
}

#[test]
fn test_bare_static_sequence_defaults_to_exact_vector() {
    let module = test_module();
    let update = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), "const values = [11, 12, 13]\n", 1)).expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let product = update
        .snapshot
        .static_constant(&StaticConstantId::new(module, "values"))
        .expect("static constant values");
    assert!(product.is_successful(), "static Vector should reduce: {:?}", product.outcome);
    let value = update
        .snapshot
        .static_values()
        .get(product.value().expect("Vector value"))
        .expect("static sequence value");
    let StaticValueData::Sequence(id) = value else {
        panic!("bare static bracket must publish a canonical Vector sequence")
    };
    let sequence = update.snapshot.static_values().get_sequence(*id).expect("Vector sequence");
    assert_eq!(sequence.family(), StaticSequenceFamily::Vector);
    assert_eq!(sequence.elements().len(), 3);
    let (_, arguments) = update.snapshot.store.applied_nominal_parts(sequence.result_type()).expect("applied Vector type");
    let TypeData::ConstInt(extent) = update.snapshot.store.get(arguments[1]) else {
        panic!("bare static Vector must retain its exact extent")
    };
    assert_eq!(extent.to_bigint(), 3.into());
}

#[test]
fn test_static_bracket_support_does_not_authorize_collection_constructors() {
    for (name, source) in [
        ("Array", "const value: Array<Int> = Array<Int>.new()\n"),
        ("Vector", "const value: Vector<Int, 0> = Vector<Int, 0>.new()\n"),
    ] {
        let module = test_module();
        let update = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
        let product = update.snapshot.static_constant(&StaticConstantId::new(module, "value"));
        assert!(
            update.snapshot.has_errors() || product.is_none_or(|product| !product.is_successful()),
            "{name}.new must not become statically authorized through bracket support"
        );
        if let Some(product) = product {
            assert!(product.value().is_none(), "{name}.new must not publish a static sequence");
        }
    }
}

#[test]
fn test_constant_annotation_projects_a_reduced_constant_record_key() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const Key: Symbol = #token\nconst Value: { [Key]: Int } = { [#token]: 1 }\n";

    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
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
            .as_ref()
            .unwrap()
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
    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
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
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&phalcom_semantic::db::QueryKey::FieldSignature(field))
            .is_some_and(|dependencies| dependencies
                .iter()
                .any(|edge| { edge.dependency == phalcom_semantic::db::QueryKey::StaticConstant(StaticConstantId::new(test_module(), "Key")) }))
    );
}

#[test]
fn test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly() {
    let module = test_module();
    let field = FieldId::new(DeclarationId::new(module.clone(), "Box".into()), "_value", DispatchSide::Instance);
    let source_a = "const Key: Symbol = #first\nclass Box {\n  _value: { [Key]: Int }\n}\n";
    let source_b = "const Key: Symbol = #second\nclass Box {\n  _value: { [Key]: Int }\n}\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let signature_a = first.snapshot.field_signatures.get(&field).cloned().expect("field signature revision 1");

    let updated = incremental.update(single_module_input(module.clone(), source_b, 2)).expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "diagnostics: {:?}", updated.snapshot.diagnostics);
    let signature_b = updated.snapshot.field_signatures.get(&field).expect("field signature revision 2");
    assert_ne!(signature_a, *signature_b, "the owner signature must change with its selected key");
    let ty = signature_b.declared_type.to_knowledge().ty().expect("field type");
    let TypeData::Record(row) = updated.snapshot.store.get(ty) else {
        panic!("field annotation must remain a Record type")
    };
    assert_eq!(
        updated.snapshot.store.record_row(*row).fields[0].key,
        phalcom_semantic::field_key::FieldKey::symbol("second")
    );

    let surface = updated.snapshot.dispatch.get_surface(&field.owner).expect("declaration surface");
    let surfaced = surface.get_field(DispatchSide::Instance, "_value").expect("surface field");
    let TypeData::Record(surface_row) = updated.snapshot.store.get(surfaced.ty().expect("surface field type")) else {
        panic!("surface field must consume the canonical computed-key signature")
    };
    assert_eq!(
        updated.snapshot.store.record_row(*surface_row).fields[0].key,
        phalcom_semantic::field_key::FieldKey::symbol("second")
    );

    let cold = SemanticWorkspaceSession::new().update(single_module_input(module, source_b, 1)).expect("semantic test update should succeed");
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
    let update = session.update(single_module_input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
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
    let dependencies = session
        .db()
        .index()
        .dependencies_of(&phalcom_semantic::db::QueryKey::CallableSignature(callable.clone()));
    assert!(
        dependencies.is_some_and(|dependencies| {
            dependencies
                .iter()
                .any(|edge| edge.dependency == phalcom_semantic::db::QueryKey::StaticConstant(StaticConstantId::new(test_module(), "Key")))
        }),
        "callable owner must depend on the selected constant: {dependencies:?}"
    );

    let source_b = source_a.replace("#token", "#second");
    let updated = session.update(single_module_input(module.clone(), &source_b, 2)).expect("semantic test update should succeed");
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

    let cold = SemanticWorkspaceSession::new().update(single_module_input(module, &source_b, 1)).expect("semantic test update should succeed");
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
fn test_inherent_impl_callable_signature_projects_parameter_and_return_record_keys() {
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Box".into());
    let callable = phalcom_semantic::identity::CallableId::new(
        owner.clone(),
        Selector::method("echo", [SelectorSlot::Positional]).expect("valid selector"),
        DispatchSide::Instance,
    );
    let source_a = r#"
const Key: Symbol = #first
class Box {}
impl Box {
  echo(_ value: { [Key]: Int }) -> { [Key]: Int } { value }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);

    let assert_key = |store: &phalcom_semantic::types::store::TypeStore, ty: phalcom_semantic::types::id::TypeId, expected: &str| {
        let TypeData::Record(row) = store.get(ty) else {
            panic!("inherent impl annotation should publish a Record type")
        };
        assert_eq!(store.record_row(*row).fields[0].key, phalcom_semantic::field_key::FieldKey::symbol(expected));
    };
    let signature_a = first.snapshot.callable_signatures.get(&callable).expect("inherent callable signature").clone();
    assert_key(
        &first.snapshot.store,
        signature_a.parameters[0].declared_type.to_knowledge().ty().expect("parameter Record type"),
        "first",
    );
    assert_key(
        &first.snapshot.store,
        signature_a.declared_return.to_knowledge().ty().expect("return Record type"),
        "first",
    );
    let dependencies = session
        .db()
        .index()
        .dependencies_of(&phalcom_semantic::db::QueryKey::CallableSignature(callable.clone()));
    assert!(
        dependencies.is_some_and(|dependencies| {
            dependencies
                .iter()
                .any(|edge| edge.dependency == phalcom_semantic::db::QueryKey::StaticConstant(StaticConstantId::new(module.clone(), "Key")))
        }),
        "inherent callable owner must retain its exact static-constant dependency: {dependencies:?}"
    );
    let surface_signature = first
        .snapshot
        .dispatch
        .get_surface(&owner)
        .and_then(|surface| surface.get_callable(DispatchSide::Instance, &callable.selector))
        .expect("effective surface must include the accepted inherent callable");
    assert_key(
        &first.snapshot.store,
        surface_signature.parameters[0].ty.ty().expect("surface parameter type"),
        "first",
    );
    assert_key(&first.snapshot.store, surface_signature.return_type.ty().expect("surface return type"), "first");

    let source_b = source_a.replace("#first", "#second");
    let updated = session.update(single_module_input(module.clone(), &source_b, 2)).expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "incremental diagnostics: {:?}", updated.snapshot.diagnostics);
    let signature_b = updated
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("updated inherent callable signature");
    assert_ne!(signature_a, *signature_b, "the owner signature must change with its selected key");
    assert_key(
        &updated.snapshot.store,
        signature_b.parameters[0]
            .declared_type
            .to_knowledge()
            .ty()
            .expect("updated parameter Record type"),
        "second",
    );
    assert_key(
        &updated.snapshot.store,
        signature_b.declared_return.to_knowledge().ty().expect("updated return Record type"),
        "second",
    );

    let cold = SemanticWorkspaceSession::new().update(single_module_input(module, &source_b, 1)).expect("semantic test update should succeed");
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);
    let cold_signature = cold.snapshot.callable_signatures.get(&callable).expect("cold inherent callable signature");
    for (store, signature) in [(&updated.snapshot.store, signature_b), (&cold.snapshot.store, cold_signature)] {
        assert_key(store, signature.declared_return.to_knowledge().ty().expect("return Record type"), "second");
    }
}

#[test]
fn test_constant_annotation_key_cycle_is_diagnosed_as_a_static_constant_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const A: Symbol = B\nconst B: { [A]: Int } = { [#token]: 1 }\n";

    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
    let diagnostics = update.snapshot.diagnostics.get(&module).expect("cycle diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::StaticConstantCycle),
        "computed-key formation cycle must use the static constant cycle path: {diagnostics:?}"
    );
    assert!(
        matches!(
            update
                .snapshot
                .static_constant(&StaticConstantId::new(module, "B"))
                .expect("B constant product")
                .outcome,
            StaticEvalOutcome::Recursive(_)
        )
    );
}

#[test]
fn test_transitive_constant_formation_cycle_retains_computed_key_cause() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const A: Int = B\nconst B: { [C]: Int } = { [#token]: 1 }\nconst C: Symbol = A\n";

    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
    let diagnostics = update.snapshot.diagnostics.get(&module).expect("cycle diagnostics");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::StaticConstantCycle && diagnostic.message.contains("computed Record key") }),
        "the transitive mixed cycle must retain its computed-key formation cause: {diagnostics:?}"
    );
    for name in ["A", "B", "C"] {
        assert!(
            matches!(
                update
                    .snapshot
                    .static_constant(&StaticConstantId::new(module.clone(), name))
                    .expect("cycle constant product")
                    .outcome,
                StaticEvalOutcome::Recursive(_)
            ),
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
    let update = session.update(input).expect("semantic test update should succeed");
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
    assert!(
        value
            .checked
            .as_ref()
            .unwrap()
            .dependencies
            .contains(&SemanticDependency::StaticConstant(StaticConstantId::new(provider, "Key")))
    );
}

#[test]
fn test_constant_key_edit_republishes_only_its_annotation_consumer() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source1 = "const Key: Symbol = #first\nconst Value: { [Key]: Int } = { [#first]: 1 }\nconst Other: Int = 7\n";
    let update1 = session.update(single_module_input(module.clone(), source1, 1)).expect("semantic test update should succeed");
    assert!(!update1.snapshot.has_errors(), "diagnostics: {:?}", update1.snapshot.diagnostics);
    let value_id = StaticConstantId::new(module.clone(), "Value");
    let other_id = StaticConstantId::new(module.clone(), "Other");
    let value1 = update1.snapshot.static_constant(&value_id).cloned().expect("Value revision 1");
    let other1 = update1.snapshot.static_constant(&other_id).cloned().expect("Other revision 1");

    let source2 = "const Key: Symbol = #other\nconst Value: { [Key]: Int } = { [#first]: 1 }\nconst Other: Int = 7\n";
    let update2 = session.update(single_module_input(module, source2, 2)).expect("semantic test update should succeed");
    let value2 = update2.snapshot.static_constant(&value_id).expect("Value revision 2");
    let other2 = update2.snapshot.static_constant(&other_id).expect("Other revision 2");
    assert!(!Arc::ptr_eq(&value1, value2), "Value must be republished after its selected key changes");
    assert!(Arc::ptr_eq(&other1, other2), "unrelated constant must remain reusable");

    let expected = value2.checked.as_ref().unwrap().result_type.expect("Value declared type");
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
    let update1 = session.update(single_module_input(module.clone(), source1, 1)).expect("semantic test update should succeed");
    assert!(!update1.snapshot.has_errors(), "diagnostics: {:?}", update1.snapshot.diagnostics);
    let value1 = update1.snapshot.static_constant(&value_id).cloned().expect("Value revision 1");

    // Parenthesizing the initializer changes its checked source product while
    // preserving the reduced FieldKey. The exact dependency must be
    // revalidated without disturbing unrelated products.
    let source2 = "const Key: Symbol = (#stable)\nconst Value: { [Key]: Int } = { [#stable]: 1 }\n";
    let update2 = session.update(single_module_input(module.clone(), source2, 2)).expect("semantic test update should succeed");
    assert!(!update2.snapshot.has_errors(), "diagnostics: {:?}", update2.snapshot.diagnostics);
    let value2 = update2.snapshot.static_constant(&value_id).expect("Value revision 2");
    assert!(
        !Arc::ptr_eq(&value1, value2),
        "the annotation owner must revalidate a selected key even when its reduced FieldKey is unchanged"
    );
    assert!(
        value2
            .checked
            .as_ref()
            .unwrap()
            .dependencies
            .contains(&SemanticDependency::StaticConstant(StaticConstantId::new(module.clone(), "Key")))
    );

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(
        module.clone(),
        "const Key: Symbol = (#stable)\nconst Value: { [Key]: Int } = { [#stable]: 1 }\n",
        1,
    )).expect("semantic test update should succeed");
    assert_eq!(update2.snapshot.diagnostics.get(&module), cold.snapshot.diagnostics.get(&module));
    let cold_value = cold.snapshot.static_constant(&value_id).expect("cold Value");
    assert!(value2.is_successful() && cold_value.is_successful());
    assert_eq!(
        update2.snapshot.static_values().get(value2.value().expect("incremental value")),
        cold.snapshot.static_values().get(cold_value.value().expect("cold value"))
    );
    let incremental_type = value2.checked.as_ref().unwrap().result_type.expect("incremental Value type");
    let cold_type = cold_value.checked.as_ref().unwrap().result_type.expect("cold Value type");
    let (TypeData::Record(incremental_row), TypeData::Record(cold_row)) = (update2.snapshot.store.get(incremental_type), cold.snapshot.store.get(cold_type))
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
    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a = update.snapshot.static_constant(&id_a).expect("static constant A");
    assert!(prod_a.is_successful());

    let prod_b = update.snapshot.static_constant(&id_b).expect("static constant B");
    assert!(prod_b.is_successful());
    assert!(prod_b.checked.as_ref().unwrap().static_constant_reads.contains(id_a.symbol()), "B should record dependency on A");

    let val_b = update.snapshot.static_values().get(prod_b.value().unwrap()).unwrap();
    assert_eq!(val_b, &StaticValueData::Int(StaticInt::Small(15)));
}

#[test]
fn test_key_discriminator_same_module_isolation() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let src1 = "const A: Int = 10\nconst B: Int = 20\n";
    let update1 = session.update(single_module_input(module.clone(), src1, 1)).expect("semantic test update should succeed");
    assert!(!update1.snapshot.has_errors());

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a1 = update1.snapshot.static_constant(&id_a).cloned().expect("A rev 1");
    let prod_b1 = update1.snapshot.static_constant(&id_b).cloned().expect("B rev 1");

    // Rev 2: Edit A to 15, leave B = 20 unchanged.
    let src2 = "const A: Int = 15\nconst B: Int = 20\n";
    let update2 = session.update(single_module_input(module.clone(), src2, 2)).expect("semantic test update should succeed");
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
    let update1 = session.update(single_module_input(module.clone(), src1, 1)).expect("semantic test update should succeed");
    assert!(!update1.snapshot.has_errors());

    let _id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_b1 = update1.snapshot.static_constant(&id_b).cloned().expect("B rev 1");
    let val_b1 = update1.snapshot.static_values().get(prod_b1.value().unwrap()).unwrap();
    assert_eq!(val_b1, &StaticValueData::Int(StaticInt::Small(11)));

    // Rev 2: Edit A to 20. A's type is still Int, but its value changed.
    let src2 = "const A: Int = 20\nconst B: Int = A + 1\n";
    let update2 = session.update(single_module_input(module.clone(), src2, 2)).expect("semantic test update should succeed");
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
    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");

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
    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::StaticConstantCycle),
        "expected StaticConstantCycle diagnostic, got: {:?}",
        diags
    );

    let id_a = StaticConstantId::new(module.clone(), "A");
    let prod_a = update.snapshot.static_constant(&id_a).expect("constant product");
    assert!(matches!(prod_a.outcome, StaticEvalOutcome::Recursive(_)));
    assert!(!prod_a.is_successful());
}

#[test]
fn test_mutual_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "const A: Int = B\nconst B: Int = A\n";
    let update = session.update(single_module_input(module.clone(), source, 1)).expect("semantic test update should succeed");

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

    assert!(matches!(prod_a.outcome, StaticEvalOutcome::Recursive(_)));
    assert!(matches!(prod_b.outcome, StaticEvalOutcome::Recursive(_)));
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
        let update1 = session.update(single_module_input(module.clone(), "const A: Int = 42\n", 1)).expect("semantic test update should succeed");
        assert!(!update1.snapshot.has_errors());
        let id_a = StaticConstantId::new(module.clone(), "A");
        let prod_a1 = update1.snapshot.static_constant(&id_a).expect("A rev 1");
        assert!(prod_a1.is_successful());
        let val1 = prod_a1.value().unwrap();
        assert_eq!(prod_a1.lkg_value, Some(val1));

        // Rev 2: Introduce cycle (eval fails)
        let update2 = session.update(single_module_input(module.clone(), "const A: Int = A\n", 2)).expect("semantic test update should succeed");
        let prod_a2 = update2.snapshot.static_constant(&id_a).expect("A rev 2");
        assert!(!prod_a2.is_successful());
        assert_eq!(prod_a2.value, None);
        assert_eq!(prod_a2.lkg_value, Some(val1), "LKG value must persist across failed revision");
    }
}
