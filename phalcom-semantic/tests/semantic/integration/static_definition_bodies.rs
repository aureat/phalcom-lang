use crate::semantic::support::WorkspaceFixture;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::SymbolId;
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use phalcom_semantic::{
    SemanticWorkspaceSession,
    db::{CancellationToken, QueryBudget},
    identity::DeclarationId,
    static_eval::{StaticEvalError, StaticEvalOutcome, StaticEvaluator},
    static_value::{StaticValueData, StaticValueStore},
};
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn session_publishes_checked_static_body_with_stable_const_reads() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            "data Point { x: Int, y: Int }\ndata LeftChoice { @variant Same(_ first: Int, _ second: Int) }\ndata RightChoice { @variant Same(_ first: Int, _ second: Int) }\ndata Boxed<T> { value: T }\nconst base: Int = 2\ntype ReadBase: Int = base\ntype AddBase: Int = base + 0\ntype AddLiteral: Int = 1 + 2\ntype Choose: Int = if true { 7 } else { 9 }\ntype ChooseElse: Int = if false { 7 } else { 9 }\ntype NestedChoose: Int = if false { 0 } else { if true { 5 } else { 6 } }\ntype NestedArithmetic: Int = 1 + (2 + 3)\ntype LazyAnd: Bool = false and (true == true)\ntype EagerAnd: Bool = true and (true == true)\ntype LazyOr: Bool = true or (true == true)\ntype TupleValue: (Int, Int) { (1, 2) }\ntype RecordValue: {left: Int, right: Int} { {left: 1, right: 2} }\ntype MixedRecordValue: { [#same]: Int, [\"same\"]: Int, [42]: Int, [true]: Bool } { { [#same]: 1, [\"same\"]: 2, [42]: 3, [true]: false } }\ntype InvalidRecordKeyOrder: { [#valid]: Int } { { [1.5]: 1 + true } }\ntype SymbolValue: Symbol = #move\ntype MutableLocal: Int { let mut value = 1\n value = 2\n value }\ntype OrderedWrites: Int { let mut value = 0\n value = 1\n value = 2\n value }\ntype UninitializedRead: Int { let value: Int\n value }\ntype ImmutableReassignment: Int { let value = 1\n value = 2\n value }\ntype Literal: Int = 3\ntype Applied: Int = Literal<>\ntype Identity<N: Int>: Int { N }\ntype AppliedValue: Int = Identity<7>\ntype Mixed<T: Type, N: Int>: Int { N }\ntype AppliedMixed: Int = Mixed<Int, 9>\ntype FixedKind<N: Int>: Type = Int\ntype ForwardKind<T: Type, N: Int>: Type = T\ntype CompositeKind<T: Type, N: Int>: Type = Boxed<T>\ntype KindConsumer<T: Type>: Int { 11 }\ntype AppliedFixedKind: Int = KindConsumer<FixedKind<0>>\ntype AppliedForwardKind: Int = KindConsumer<ForwardKind<Int, 0>>\ntype AppliedCompositeKind: Int = KindConsumer<CompositeKind<Int, 0>>\ntype OuterValue: Int { Literal<> }\ntype RecursiveValue: Int { RecursiveValue<> }\ntype RecursiveA: Int { RecursiveB<> }\ntype RecursiveB: Int { RecursiveA<> }\ntype StaticPoint: Point { Point { y: 2, x: 1 } }\ntype StaticLeft: LeftChoice { LeftChoice::Same(3, 4) }\ntype StaticRight: RightChoice { RightChoice::Same(3, 4) }\n",
        )
        .analyze();
    let module = fixture.module("main").clone();
    let definition_id = fixture.decl("main", "ReadBase");
    let definition = fixture.analysis.snapshot.static_definition(&definition_id).expect("resolved static definition");
    let checked = definition.checked_body.as_ref().expect("session composes checked body");

    assert!(checked.diagnostics.is_empty(), "unexpected static body diagnostics: {:?}", checked.diagnostics);
    assert_eq!(
        checked.static_constant_reads.as_ref(),
        &[SymbolId {
            module: module.clone(),
            name: "base".into()
        }]
    );
    let add = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "AddBase"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked addition body");
    assert!(
        add.static_operations
            .values()
            .any(|resolution| resolution.operation == phalcom_semantic::static_intrinsic::StaticIntrinsicOperation::NumberAdd)
    );
    let add_literal = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "AddLiteral"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked literal addition body");
    let mut values = StaticValueStore::new();
    let reduced = StaticEvaluator::new(add_literal, &mut values, &mut QueryBudget::default(), &CancellationToken::new(), &[]).evaluate();
    let StaticEvalOutcome::Reduced(value) = reduced else {
        panic!("checked integer addition should reduce, got {reduced:?}");
    };
    assert_eq!(
        values.get(value).expect("result belongs to the evaluator value store"),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(3))
    );
    let evaluate_named = |name: &str, values: &mut StaticValueStore| {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|definition| definition.checked_body.as_ref())
            .expect("checked static definition body");
        StaticEvaluator::new(checked, values, &mut QueryBudget::default(), &CancellationToken::new(), &[]).evaluate()
    };
    let mut branch_values = StaticValueStore::new();
    let chosen = evaluate_named("Choose", &mut branch_values);
    let StaticEvalOutcome::Reduced(chosen_value) = chosen else {
        panic!("selected if branch should reduce, got {chosen:?}");
    };
    assert_eq!(
        branch_values.get(chosen_value).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(7))
    );
    for (name, expected) in [("ChooseElse", 9), ("NestedChoose", 5), ("NestedArithmetic", 6)] {
        let mut values = StaticValueStore::new();
        let result = evaluate_named(name, &mut values);
        let StaticEvalOutcome::Reduced(result) = result else {
            panic!("{name} should reduce, got {result:?}");
        };
        assert_eq!(
            values.get(result).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(expected)),
            "{name} should preserve selected-branch and operand order"
        );
    }

    let mut lazy_values = StaticValueStore::new();
    assert!(matches!(evaluate_named("LazyAnd", &mut lazy_values), StaticEvalOutcome::Reduced(_)));
    assert!(matches!(evaluate_named("LazyOr", &mut lazy_values), StaticEvalOutcome::Reduced(_)));
    assert!(matches!(
        evaluate_named("EagerAnd", &mut lazy_values),
        StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedOperation(_))
    ));

    let mut tuple_values = StaticValueStore::new();
    let tuple = evaluate_named("TupleValue", &mut tuple_values);
    let StaticEvalOutcome::Reduced(tuple) = tuple else {
        panic!("checked tuple constructor should reduce, got {tuple:?}");
    };
    assert!(matches!(tuple_values.get(tuple).unwrap(), StaticValueData::Tuple(_)));

    let mut record_values = StaticValueStore::new();
    let record = evaluate_named("RecordValue", &mut record_values);
    let StaticEvalOutcome::Reduced(record) = record else {
        panic!("checked record constructor should reduce, got {record:?}");
    };
    assert!(matches!(record_values.get(record).unwrap(), StaticValueData::Record(_)));

    let mixed_record = evaluate_named("MixedRecordValue", &mut record_values);
    let StaticEvalOutcome::Reduced(mixed_record) = mixed_record else {
        panic!("checked computed-key Record should reduce, got {mixed_record:?}");
    };
    let one = record_values.intern_int(1);
    let two = record_values.intern_int(2);
    let three = record_values.intern_int(3);
    let false_value = record_values.intern_bool(false);
    let expected_mixed_record = record_values
        .intern_record(vec![
            (phalcom_semantic::field_key::FieldKey::symbol("same"), one),
            (phalcom_semantic::field_key::FieldKey::string("same"), two),
            (phalcom_semantic::field_key::FieldKey::int(phalcom_semantic::scalar::StaticInt::from(42)), three),
            (phalcom_semantic::field_key::FieldKey::bool(true), false_value),
        ])
        .expect("expected mixed-key static Record");
    assert!(record_values.semantic_eq(mixed_record, expected_mixed_record).unwrap());

    let invalid_record_key = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "InvalidRecordKeyOrder"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked invalid-key static body");
    let invalid_key_result = StaticEvaluator::new(
        invalid_record_key,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .evaluate();
    assert!(matches!(
        invalid_key_result,
        StaticEvalOutcome::Invalid(StaticEvalError::ValueStore(message)) if message.contains("not a valid Record field key")
    ));

    let mut symbol_values = StaticValueStore::new();
    let symbol = evaluate_named("SymbolValue", &mut symbol_values);
    let StaticEvalOutcome::Reduced(symbol) = symbol else {
        panic!("checked symbol literal should reduce, got {symbol:?}");
    };
    assert!(matches!(symbol_values.get(symbol).unwrap(), StaticValueData::Symbol(_)));

    let mut mutable_values = StaticValueStore::new();
    let mutable = evaluate_named("MutableLocal", &mut mutable_values);
    let StaticEvalOutcome::Reduced(mutable) = mutable else {
        panic!("checked mutable local should reduce, got {mutable:?}");
    };
    assert_eq!(
        mutable_values.get(mutable).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(2))
    );
    let mut ordered_values = StaticValueStore::new();
    let ordered = evaluate_named("OrderedWrites", &mut ordered_values);
    let StaticEvalOutcome::Reduced(ordered) = ordered else {
        panic!("ordered local writes should reduce, got {ordered:?}");
    };
    assert_eq!(
        ordered_values.get(ordered).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(2)),
        "local statements must execute in source order"
    );

    let uninitialized = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "UninitializedRead"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked uninitialized-local body");
    let uninitialized_result = StaticEvaluator::new(
        uninitialized,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .evaluate();
    assert!(matches!(
        uninitialized_result,
        StaticEvalOutcome::Invalid(StaticEvalError::UninitializedBinding(_))
    ));

    let immutable = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ImmutableReassignment"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked immutable-reassignment body");
    assert!(!immutable.diagnostics.is_empty(), "ordinary checking rejects immutable reassignment");
    let immutable_result = StaticEvaluator::new(
        immutable,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .evaluate();
    assert!(matches!(immutable_result, StaticEvalOutcome::Invalid(_)));
    assert!(
        checked.module_binding_fingerprint.is_some(),
        "body consumes the ordinary module binding product"
    );
    let applied = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "Applied"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked static application body");
    assert!(applied.diagnostics.is_empty(), "unexpected application diagnostics: {:?}", applied.diagnostics);
    assert_eq!(applied.static_applications.len(), 1);
    let application = &applied.static_applications[0];
    assert_eq!(application.target, fixture.decl("main", "Literal"));
    assert_eq!(application.id.ordinal, 0);
    assert_eq!(
        application.id.body,
        phalcom_semantic::static_definition::StaticBodyId(fixture.decl("main", "Applied")),
        "application ordinals are qualified by the enclosing checked body, not the selected target"
    );
    assert!(application.signature.parameters.is_empty());
    assert!(application.arguments.is_empty());
    assert!(applied.dependencies.iter().any(|dependency| matches!(
        dependency,
        phalcom_semantic::checker::analysis::SemanticDependency::StaticDefinition(target)
            if *target == fixture.decl("main", "Literal")
    )));
    let mut wrong_result_application = (**applied).clone();
    let mut wrong_result_products = wrong_result_application.static_applications.to_vec();
    wrong_result_products[0].signature.result_domain = phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE);
    wrong_result_application.static_applications = wrong_result_products.into();
    let wrong_result = StaticEvaluator::new(
        &wrong_result_application,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .evaluate();
    assert_eq!(wrong_result, StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedExpression));
    let applied_value = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "AppliedValue"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked value-parameter application body");
    assert!(
        applied_value.diagnostics.is_empty(),
        "unexpected value-application diagnostics: {:?}",
        applied_value.diagnostics
    );
    assert_eq!(applied_value.static_applications.len(), 1);
    let value_application = &applied_value.static_applications[0];
    assert_eq!(value_application.target, fixture.decl("main", "Identity"));
    assert_eq!(value_application.id.ordinal, 0);
    assert_eq!(
        value_application.id.body,
        phalcom_semantic::static_definition::StaticBodyId(fixture.decl("main", "AppliedValue")),
        "overlapping application ordinals remain qualified by their actual checked-body owners"
    );
    assert_eq!(value_application.signature.parameters.len(), 1);
    assert!(matches!(
        value_application.arguments.as_ref(),
        [phalcom_semantic::static_definition::CheckedStaticApplicationArgument::Value { parameter_index: 0, .. }]
    ));
    assert!(applied_value.dependencies.iter().any(|dependency| matches!(
        dependency,
        phalcom_semantic::checker::analysis::SemanticDependency::StaticDefinition(target)
            if *target == fixture.decl("main", "Identity")
    )));
    let mut application_values = StaticValueStore::new();
    let applied_value_result = StaticEvaluator::new(applied, &mut application_values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .evaluate();
    let StaticEvalOutcome::Reduced(applied_value_result) = applied_value_result else {
        panic!("zero-argument static application should reduce: {applied_value_result:?}");
    };
    assert_eq!(
        application_values.get(applied_value_result).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(3))
    );

    let mut parameter_values = StaticValueStore::new();
    let applied_parameter_result = StaticEvaluator::new(
        applied_value,
        &mut parameter_values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .evaluate();
    let StaticEvalOutcome::Reduced(applied_parameter_result) = applied_parameter_result else {
        panic!("value-parameter static application should reduce: {applied_parameter_result:?}");
    };
    assert_eq!(
        parameter_values.get(applied_parameter_result).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(7))
    );

    let applied_mixed = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "AppliedMixed"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked mixed-parameter application body");
    let mut mixed_values = StaticValueStore::new();
    let mixed_result = StaticEvaluator::new(applied_mixed, &mut mixed_values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
        .with_type_store(fixture.analysis.snapshot.store())
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .evaluate();
    let StaticEvalOutcome::Reduced(mixed_result) = mixed_result else {
        panic!("mixed kinded/value static application should reduce: {mixed_result:?}");
    };
    assert_eq!(
        mixed_values.get(mixed_result).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(9))
    );

    let int_form = fixture
        .analysis
        .snapshot
        .declarations
        .form(&phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int))
        .expect("canonical Int TypeId");
    let mut kind_values = StaticValueStore::new();
    let fixed_body = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "FixedKind"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked independent kinded result");
    let fixed_zero = kind_values.intern_int(0);
    let fixed_result = StaticEvaluator::new(
        fixed_body,
        &mut kind_values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Value(fixed_zero)],
    )
    .with_type_store(fixture.analysis.snapshot.store())
    .evaluate();
    assert_eq!(fixed_result, StaticEvalOutcome::ReducedKinded(int_form));

    let forward_body = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ForwardKind"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked kinded parameter forwarding");
    let zero = kind_values.intern_int(0);
    let forwarded = StaticEvaluator::new(
        forward_body,
        &mut kind_values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[
            phalcom_semantic::static_eval::StaticArgument::Kinded(int_form),
            phalcom_semantic::static_eval::StaticArgument::Value(zero),
        ],
    )
    .with_type_store(fixture.analysis.snapshot.store())
    .evaluate();
    assert_eq!(
        forwarded,
        StaticEvalOutcome::ReducedKinded(int_form),
        "forwarding preserves canonical TypeId identity"
    );

    let evaluate_kinded_application = |name: &str, values: &mut StaticValueStore| {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|product| product.checked_body.as_ref())
            .expect("checked kinded application body");
        StaticEvaluator::new(checked, values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
            .with_type_store(fixture.analysis.snapshot.store())
            .with_static_definitions(fixture.analysis.snapshot.static_definitions())
            .evaluate()
    };
    let applied_fixed = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "AppliedFixedKind"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked nested kinded application");
    let consumer_application = applied_fixed
        .static_applications
        .iter()
        .find(|application| application.target == fixture.decl("main", "KindConsumer"))
        .expect("checked outer kind consumer application");
    assert!(
        matches!(
            consumer_application.arguments.as_ref(),
            [phalcom_semantic::static_definition::CheckedStaticApplicationArgument::KindedExpression { parameter_index: 0, .. }]
        ),
        "expected checked nested static application argument, got {:?}",
        consumer_application.arguments
    );
    for name in ["AppliedFixedKind", "AppliedForwardKind"] {
        let mut values = StaticValueStore::new();
        let outcome = evaluate_kinded_application(name, &mut values);
        let StaticEvalOutcome::Reduced(result) = outcome else {
            panic!("nested kinded application {name} should reduce, got {outcome:?}");
        };
        assert_eq!(
            values.get(result).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(11))
        );
    }
    let mut composite_values = StaticValueStore::new();
    assert_eq!(
        evaluate_kinded_application("AppliedCompositeKind", &mut composite_values),
        StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedExpression),
        "composite TypeForm substitution must fail closed"
    );

    let outer = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "OuterValue"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked outer application body");
    let mut nested_values = StaticValueStore::new();
    let nested_result = StaticEvaluator::new(outer, &mut nested_values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .evaluate();
    let StaticEvalOutcome::Reduced(nested_result) = nested_result else {
        panic!("nested non-recursive static application should reduce: {nested_result:?}");
    };
    assert_eq!(
        nested_values.get(nested_result).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(3))
    );

    let static_point = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "StaticPoint"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked data construction body");
    assert_eq!(static_point.static_constructions.len(), 1);
    let point_construction = static_point.static_constructions.values().next().unwrap();
    assert!(matches!(
        point_construction.target,
        phalcom_semantic::identity::InvocationTargetId::DataConstructor(_)
    ));
    assert!(point_construction.arguments.iter().any(|argument| matches!(
        &argument.payload,
        phalcom_semantic::static_definition::CheckedStaticPayloadId::DataComponent(component) if component.index == 0
    )));
    let mut point_values = StaticValueStore::new();
    let mut point_budget = QueryBudget::default();
    let point_result = StaticEvaluator::new(static_point, &mut point_values, &mut point_budget, &CancellationToken::new(), &[])
        .with_semantic_tables(
            fixture.analysis.snapshot.store(),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .evaluate();
    let StaticEvalOutcome::Reduced(point_result) = point_result else {
        panic!("checked static Data construction should reduce: {point_result:?}");
    };
    let point_info = fixture.analysis.snapshot.data_semantics.data_info(&fixture.decl("main", "Point")).unwrap();
    let point_one = point_values.intern_int(1);
    let point_two = point_values.intern_int(2);
    let expected_point = point_values
        .intern_data(
            point_info,
            point_construction.result_type,
            vec![
                (point_info.components[0].id.clone(), point_one),
                (point_info.components[1].id.clone(), point_two),
            ],
            fixture.analysis.snapshot.store(),
        )
        .unwrap();
    assert_eq!(point_result, expected_point, "labeled source order must not replace canonical component order");
    let mut bounded_point_values = StaticValueStore::new();
    let mut bounded_point_budget = QueryBudget::new(point_budget.steps_taken - 5);
    let bounded_point_result = StaticEvaluator::new(
        static_point,
        &mut bounded_point_values,
        &mut bounded_point_budget,
        &CancellationToken::new(),
        &[],
    )
    .with_semantic_tables(
        fixture.analysis.snapshot.store(),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .evaluate();
    assert!(matches!(bounded_point_result, StaticEvalOutcome::BudgetExceeded(_)));
    assert_eq!(bounded_point_values.len(), 2, "budget exhaustion occurs before the Data value node is interned");

    let evaluate_variant = |name: &str, values: &mut StaticValueStore| {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|definition| definition.checked_body.as_ref())
            .expect("checked static variant construction body");
        StaticEvaluator::new(checked, values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
            .with_semantic_tables(
                fixture.analysis.snapshot.store(),
                &fixture.analysis.snapshot.data_semantics,
                &fixture.analysis.snapshot.enum_semantics,
            )
            .evaluate()
    };
    let mut variant_values = StaticValueStore::new();
    let left_value = evaluate_variant("StaticLeft", &mut variant_values);
    let right_value = evaluate_variant("StaticRight", &mut variant_values);
    let (StaticEvalOutcome::Reduced(left_value), StaticEvalOutcome::Reduced(right_value)) = (left_value, right_value) else {
        panic!("exact static Variant constructions should reduce");
    };
    assert!(matches!(variant_values.get(left_value).unwrap(), StaticValueData::Variant(_)));
    assert!(matches!(variant_values.get(right_value).unwrap(), StaticValueData::Variant(_)));
    assert_ne!(
        left_value, right_value,
        "same-named cases in distinct enum owners keep distinct canonical identity"
    );

    let recursive = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "RecursiveValue"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked recursive application body");
    let mut recursive_values = StaticValueStore::new();
    let mut recursive_budget = QueryBudget::default();
    let recursive_cancellation = CancellationToken::new();
    let mut recursive_evaluator = StaticEvaluator::new(recursive, &mut recursive_values, &mut recursive_budget, &recursive_cancellation, &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions());
    let recursive_result = recursive_evaluator.evaluate();
    assert!(matches!(recursive_result, StaticEvalOutcome::Recursive(_)));
    assert!(recursive_evaluator.trace().iter().any(|frame| matches!(
        frame,
        phalcom_semantic::static_eval::StaticEvalTraceFrame::Application { target, .. }
            if *target == fixture.decl("main", "RecursiveValue")
    )));
    assert!(
        recursive_evaluator
            .trace()
            .iter()
            .any(|frame| matches!(frame, phalcom_semantic::static_eval::StaticEvalTraceFrame::Expression(_)))
    );
    let mutual = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "RecursiveA"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked mutually recursive application body");
    let mutual_result = StaticEvaluator::new(
        mutual,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .evaluate();
    assert!(matches!(mutual_result, StaticEvalOutcome::Recursive(_)));

    let mut exhausted_budget = QueryBudget::new(0);
    let budget_result = StaticEvaluator::new(applied, &mut StaticValueStore::new(), &mut exhausted_budget, &CancellationToken::new(), &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .evaluate();
    assert!(matches!(budget_result, StaticEvalOutcome::BudgetExceeded(_)));
    let module_bindings = fixture
        .analysis
        .snapshot
        .module_static_bindings
        .get(&module)
        .expect("published module bindings");
    let base = SymbolId { module, name: "base".into() };
    assert_eq!(
        module_bindings.get(&base).expect("canonical const symbol").phase,
        phalcom_semantic::module_bindings::ModuleBindingPhase::StaticConstant
    );
}

#[test]
fn symbolic_vector_identity_spread_stays_unreduced_before_reading_its_source() {
    let fixture = crate::semantic::support::Fixture::new(
        "class Copier<N: Int> { @class copy(_ source: Vector<Int, N>) -> Vector<Int, N> { <Int, N>[*source] } }\n\
         const source: Vector<Int, 2> = <Int, 2>[1, 2]\n\
         const copied: Vector<Int, 2> = <Int, 2>[*source]\n",
    );
    fixture.assert_no_error_diagnostics();
    let symbolic = fixture.callable("Copier", "copy", phalcom_semantic::identity::DispatchSide::Class);
    let symbolic_resolution = symbolic.sequence_literals.values().next().expect("generic checked identity spread");
    let symbolic_type = symbolic_resolution.result_type;
    let symbolic_extent = symbolic_resolution.extent.expect("symbolic Vector extent");
    assert!(matches!(
        fixture.analysis.snapshot.store.get(symbolic_extent),
        phalcom_semantic::types::store::TypeData::Parameter(_)
    ));

    let copied = fixture
        .analysis
        .snapshot
        .static_constant(&phalcom_semantic::static_constant::StaticConstantId::new(fixture.module.clone(), "copied"))
        .expect("checked constant identity spread");
    let mut checked = copied.checked.expression.clone();
    let mut resolutions = (*checked.sequence_literals).clone();
    let resolution = resolutions.values_mut().next().expect("checked constant literal");
    let source_range = *resolution.exact_expansions.keys().next().expect("checked Vector spread source range");
    resolution.result_type = symbolic_type;
    resolution.extent = Some(symbolic_extent);
    resolution.construction = phalcom_semantic::checker::SequenceConstructionPlan::VectorIdentitySpread {
        source_range,
        extent: symbolic_extent,
    };
    resolution.exact_expansions.insert(
        source_range,
        phalcom_semantic::checker::SequenceExpansionResolution::VectorStorage { extent: symbolic_extent },
    );
    checked.sequence_literals = Arc::new(resolutions);

    let mut values = StaticValueStore::new();
    let (outcome, residuals) = {
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let mut evaluator =
            StaticEvaluator::for_checked_expression(&checked, &mut values, &mut budget, &cancellation).with_type_store(&fixture.analysis.snapshot.store);
        let outcome = evaluator.evaluate();
        (outcome, evaluator.take_residuals().map(Arc::new))
    };
    let StaticEvalOutcome::Symbolic(reference) = outcome else {
        panic!("symbolic spread must retain a residual: {outcome:?}");
    };
    let mut product = copied.as_ref().clone();
    product.outcome = outcome;
    product.residuals = residuals;
    product.value = None;
    product.lkg_value = None;
    let retained = Arc::new(product);
    let clone = retained.clone();
    drop(retained);
    assert!(
        clone.residuals.as_ref().expect("published symbolic store").get(reference).is_ok(),
        "symbolic constant products must resolve residuals after evaluator teardown"
    );
    assert_eq!(
        values.len(),
        0,
        "the symbolic spread must stop before reading its source or publishing a sequence root"
    );
}

#[test]
fn static_return_control_propagates_to_root_and_stops_at_call_boundaries() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"
type ReturnEarly: Int { return 7
 999 }
type ReturnUnit: Unit { return
 7 }
type ReturnNestedIf: Int { if true { if false { return 1 } else { return 2 } }
 return 3 }
type ReturnElse: Int { if false { return 1 } else { return 4 }
 999 }
type ReturnUntaken: Bool { if true { return false } else { true and (true == true) }
 true }
type ReturnInner: Int { return 2
 999 }
type ReturnMiddle: Int { let value = ReturnInner<>
 return value + 3
 999 }
type ReturnOuter: Int { let value = ReturnMiddle<>
 value + 1 }
type ReturnKinded: Type { return Int
 String }
type ReturnKindedCaller: Type = ReturnKinded<>
type WrongKindedReturn: Type { return 1 }
type EscapedBreak: Int { break
 1 }
type EscapedContinue: Int { continue
 2 }
type BreakCallee: Int { break
 1 }
type BreakCaller: Int { BreakCallee<> }
"#,
        )
        .analyze();

    let evaluate = |name: &str, values: &mut StaticValueStore, budget: &mut QueryBudget, cancellation: &CancellationToken| {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|product| product.checked_body.as_ref())
            .unwrap_or_else(|| panic!("checked static definition {name}"));
        StaticEvaluator::new(checked, values, budget, cancellation, &[])
            .with_type_store(fixture.analysis.snapshot.store())
            .with_static_definitions(fixture.analysis.snapshot.static_definitions())
            .evaluate()
    };

    for (name, expected) in [("ReturnEarly", 7), ("ReturnNestedIf", 2), ("ReturnElse", 4), ("ReturnOuter", 6)] {
        let mut values = StaticValueStore::new();
        let outcome = evaluate(name, &mut values, &mut QueryBudget::default(), &CancellationToken::new());
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("{name} should reduce through its return path, got {outcome:?}");
        };
        assert_eq!(
            values.get(value).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(expected)),
            "{name} should preserve the selected return value"
        );
    }

    let mut unit_values = StaticValueStore::new();
    let unit = evaluate("ReturnUnit", &mut unit_values, &mut QueryBudget::default(), &CancellationToken::new());
    let StaticEvalOutcome::Reduced(unit) = unit else {
        panic!("return without an expression should return Unit, got {unit:?}");
    };
    assert_eq!(unit_values.get(unit).unwrap(), &StaticValueData::Unit);

    let mut untaken_values = StaticValueStore::new();
    let untaken = evaluate("ReturnUntaken", &mut untaken_values, &mut QueryBudget::default(), &CancellationToken::new());
    let StaticEvalOutcome::Reduced(untaken) = untaken else {
        panic!("the untaken unsupported branch should stay unevaluated, got {untaken:?}");
    };
    assert_eq!(untaken_values.get(untaken).unwrap(), &StaticValueData::Bool(false));

    let mut kinded_values = StaticValueStore::new();
    let return_kinded_caller = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ReturnKindedCaller"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked kinded caller body");
    let mut kinded_budget = QueryBudget::default();
    let kinded_cancellation = CancellationToken::new();
    let mut kinded_evaluator = StaticEvaluator::new(return_kinded_caller, &mut kinded_values, &mut kinded_budget, &kinded_cancellation, &[])
        .with_type_store(fixture.analysis.snapshot.store())
        .with_static_definitions(fixture.analysis.snapshot.static_definitions());
    let kinded = kinded_evaluator.evaluate();
    let return_kinded_body = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ReturnKinded"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked kinded return body");
    let int_type = return_kinded_body
        .expressions
        .values()
        .find_map(|analysis| match analysis.denotation.as_ref() {
            Some(
                phalcom_semantic::types::denotation::SemanticDenotation::TypeForm(form)
                | phalcom_semantic::types::denotation::SemanticDenotation::StaticTerm(form),
            ) => Some(*form),
            _ => None,
        })
        .expect("return expression has canonical type-form denotation");
    assert_eq!(kinded, StaticEvalOutcome::ReducedKinded(int_type), "trace: {:?}", kinded_evaluator.trace());

    let wrong_kinded = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "WrongKindedReturn"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked wrong-kind return body");
    assert!(
        wrong_kinded
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::TypeMismatch)
    );

    for name in ["EscapedBreak", "EscapedContinue", "BreakCaller"] {
        let outcome = evaluate(name, &mut StaticValueStore::new(), &mut QueryBudget::default(), &CancellationToken::new());
        assert_eq!(
            outcome,
            StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedExpression),
            "{name} must not escape its loop boundary"
        );
    }

    let mut bounded_values = StaticValueStore::new();
    let bounded = evaluate("ReturnEarly", &mut bounded_values, &mut QueryBudget::new(0), &CancellationToken::new());
    assert!(matches!(bounded, StaticEvalOutcome::BudgetExceeded(_)));
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let cancelled = evaluate("ReturnEarly", &mut StaticValueStore::new(), &mut QueryBudget::default(), &cancellation);
    assert_eq!(cancelled, StaticEvalOutcome::Cancelled);
}

#[test]
fn static_while_uses_loop_control_work_and_rejects_while_true_as_static_control() {
    let boolean_flag_loop = |name: &str, iterations: usize| {
        let flags = (0..iterations).map(|index| format!("flag_{index}")).collect::<Vec<_>>();
        let declarations = flags.iter().map(|flag| format!("  let mut {flag} = true\n")).collect::<String>();
        let condition = flags.join(" or ");
        let mut branch = format!("{last} = false", last = flags.last().expect("at least one loop flag"));
        for index in (0..iterations.saturating_sub(1)).rev() {
            branch = format!("if {} {{ {} = false }} else {{ {branch} }}", flags[index], flags[index]);
        }
        format!("type {name}: Unit {{\n{declarations}  while {condition} {{\n    {branch}\n  }}\n}}\n")
    };
    let mut source = String::from(
        r#"
type RecursiveBody: Int { RecursiveBody<> }
type WhileZero: Int { let mut n = 4
  while false { RecursiveBody<> }
  n }
type WhileBreak: Unit { while true {
  break
  RecursiveBody<>
} }
type WhileContinue: Bool {
  let mut first = true
  let mut second = true
  while first or second {
    if first { first = false
      continue }
    second = false
  }
  second }
type WhileReturn: Int { while true { return 13
  RecursiveBody<> }
  17 }
type WhileNested: Bool {
  let mut outer_first = true
  let mut outer_second = true
  while outer_first or outer_second {
    let mut inner_first = true
    let mut inner_second = true
    while inner_first or inner_second {
      if inner_first { inner_first = false } else { inner_second = false }
    }
    if outer_first { outer_first = false } else { outer_second = false }
  }
  outer_second }
type WhileForever: Unit { while true {} }
type ExplicitWhileTrue: Int { || { false }.whileTrue || { 1 } }
type WhileNonBool: Int { while 1 { break }
  2 }
"#,
    );
    source.push_str(&boolean_flag_loop("WhileShort", 2));
    source.push_str(&boolean_flag_loop("WhileLong", 64));
    let fixture = WorkspaceFixture::new().module("main", source.as_str()).analyze();

    let evaluate = |name: &str, budget: &mut QueryBudget, cancellation: &CancellationToken| {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|definition| definition.checked_body.as_ref())
            .unwrap_or_else(|| panic!("checked static definition {name}"));
        let mut values = StaticValueStore::new();
        let outcome = StaticEvaluator::new(checked, &mut values, budget, cancellation, &[])
            .with_type_store(fixture.analysis.snapshot.store())
            .with_static_definitions(fixture.analysis.snapshot.static_definitions())
            .evaluate();
        (outcome, values)
    };

    for (name, expected) in [("WhileZero", 4), ("WhileReturn", 13)] {
        let (outcome, values) = evaluate(name, &mut QueryBudget::default(), &CancellationToken::new());
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("{name} should reduce, got {outcome:?}");
        };
        assert_eq!(
            values.get(value).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(expected)),
            "unexpected result for {name}"
        );
    }

    for name in ["WhileShort", "WhileBreak"] {
        let (outcome, values) = evaluate(name, &mut QueryBudget::default(), &CancellationToken::new());
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("{name} should reduce, got {outcome:?}");
        };
        assert_eq!(values.get(value).unwrap(), &StaticValueData::Unit, "unexpected result for {name}");
    }
    for (name, expected) in [("WhileContinue", false), ("WhileNested", false)] {
        let (outcome, values) = evaluate(name, &mut QueryBudget::default(), &CancellationToken::new());
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("{name} should reduce, got {outcome:?}");
        };
        assert_eq!(values.get(value).unwrap(), &StaticValueData::Bool(expected), "unexpected result for {name}");
    }

    let mut short_budget = QueryBudget::default();
    let (short, _) = evaluate("WhileShort", &mut short_budget, &CancellationToken::new());
    assert!(matches!(short, StaticEvalOutcome::Reduced(_)));
    let mut long_budget = QueryBudget::new(1_000_000);
    let (long, _) = evaluate("WhileLong", &mut long_budget, &CancellationToken::new());
    assert!(matches!(long, StaticEvalOutcome::Reduced(_)));
    assert!(
        long_budget.steps_taken > short_budget.steps_taken,
        "longer loops must charge more Work transitions"
    );

    let mut bounded_budget = QueryBudget::new(64);
    let (bounded, _) = evaluate("WhileForever", &mut bounded_budget, &CancellationToken::new());
    assert!(matches!(bounded, StaticEvalOutcome::BudgetExceeded(_)));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let (pre_cancelled, _) = evaluate("WhileForever", &mut QueryBudget::default(), &cancellation);
    assert_eq!(pre_cancelled, StaticEvalOutcome::Cancelled);

    let checked = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "WhileForever"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked infinite-loop workload");
    let cancellation = CancellationToken::new();
    let cancel_after_start = cancellation.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        cancel_after_start.cancel();
    });
    let mut active_budget = QueryBudget::new(10_000_000);
    let mut values = StaticValueStore::new();
    let active = StaticEvaluator::new(checked, &mut values, &mut active_budget, &cancellation, &[])
        .with_type_store(fixture.analysis.snapshot.store())
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .evaluate();
    canceller.join().expect("active cancellation worker");
    assert_eq!(active, StaticEvalOutcome::Cancelled);
    assert!(active_budget.steps_taken > 0, "active cancellation must follow charged work");

    let explicit_while_true = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ExplicitWhileTrue"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked explicit whileTrue send");
    let explicit = StaticEvaluator::new(
        explicit_while_true,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .evaluate();
    assert_eq!(explicit, StaticEvalOutcome::Invalid(StaticEvalError::UnsupportedExpression));

    let non_bool = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "WhileNonBool"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked non-Bool while condition");
    assert!(
        non_bool
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::TypeMismatch)
    );
}

#[test]
fn static_evaluator_resource_work_scales_and_cancellation_is_cooperative() {
    let long_body = (0..4_000).map(|index| format!("  let value_{index} = {index}\n")).collect::<String>() + "  value_3999\n";
    let mut source = String::from(
        "type SequenceShort: Int { let a = 1\n a }\n\
         type SequenceLong: Int { let a = 1\n let b = 2\n let c = 3\n let d = 4\n let e = 5\n let f = 6\n let g = 7\n let h = 8\n h }\n\
         type DeepConditional: Int { if true { if true { if true { if true { 1 } else { 2 } } else { 3 } } else { 4 } } else { 5 } }\n\
         type CallLeaf: Int = 1\n\
         type CallInner: Int { let value = CallLeaf<>\n value + 1 }\n\
         type CallOuter: Int { let value = CallInner<>\n value + CallLeaf<> }\n\
         type ReturnBeforeCall: Int { return 9\n CallOuter<> }\n\
         type CancelLong: Int {\n",
    );
    source.push_str(&long_body);
    source.push_str("}\n");

    let fixture = WorkspaceFixture::new().module("main", source.as_str()).analyze();
    let evaluate = |name: &str, budget: &mut QueryBudget, cancellation: &CancellationToken| {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|definition| definition.checked_body.as_ref())
            .unwrap_or_else(|| panic!("checked static definition {name}"));
        let mut values = StaticValueStore::new();
        let mut evaluator = StaticEvaluator::new(checked, &mut values, budget, cancellation, &[])
            .with_type_store(fixture.analysis.snapshot.store())
            .with_static_definitions(fixture.analysis.snapshot.static_definitions());
        let outcome = evaluator.evaluate();
        (outcome, budget.steps_taken)
    };

    let (short, short_steps) = evaluate("SequenceShort", &mut QueryBudget::default(), &CancellationToken::new());
    assert!(matches!(short, StaticEvalOutcome::Reduced(_)));
    let (long, long_steps) = evaluate("SequenceLong", &mut QueryBudget::default(), &CancellationToken::new());
    assert!(matches!(long, StaticEvalOutcome::Reduced(_)));
    assert!(
        long_steps > short_steps,
        "longer statement sequences must charge more work: {long_steps} <= {short_steps}"
    );

    let (deep, deep_steps) = evaluate("DeepConditional", &mut QueryBudget::default(), &CancellationToken::new());
    assert!(matches!(deep, StaticEvalOutcome::Reduced(_)));
    let (nested, nested_steps) = evaluate("CallOuter", &mut QueryBudget::default(), &CancellationToken::new());
    assert!(matches!(nested, StaticEvalOutcome::Reduced(_)));
    assert!(nested_steps > short_steps, "nested static applications must charge their work");
    assert!(deep_steps > short_steps, "nested conditional work must charge its selected path");

    let (early, early_steps) = evaluate("ReturnBeforeCall", &mut QueryBudget::default(), &CancellationToken::new());
    assert!(matches!(early, StaticEvalOutcome::Reduced(_)));
    assert!(early_steps < nested_steps, "early return must skip later static application work");

    let checked = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "CancelLong"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked cancellation workload");
    let cancellation = CancellationToken::new();
    let cancel_after_start = cancellation.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        cancel_after_start.cancel();
    });
    let mut values = StaticValueStore::new();
    let mut budget = QueryBudget::new(100_000);
    let outcome = StaticEvaluator::new(checked, &mut values, &mut budget, &cancellation, &[])
        .with_type_store(fixture.analysis.snapshot.store())
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .evaluate();
    canceller.join().expect("cancellation worker");
    assert_eq!(outcome, StaticEvalOutcome::Cancelled);
    assert!(budget.steps_taken > 0, "cancellation must be observed during active evaluation");
}

fn imported_const_input(provider_source: &str, generation: u64) -> (SemanticWorkspaceInput, ModuleIdPair) {
    let project = ResolvedProjectId::from_raw(42);
    let module_id =
        |name: &str| phalcom_modules::identity::ModuleId::resolved(project, ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]));
    let provider = module_id("provider");
    let consumer = module_id("consumer");
    let provider_source: Arc<str> = Arc::from(provider_source);
    let consumer_source: Arc<str> = Arc::from("from provider import base as imported\ntype ReadImported: Int = imported\n");
    let provider_program = Arc::new(phalcom_ast::parse(&provider_source, 0).program);
    let consumer_program = Arc::new(phalcom_ast::parse(&consumer_source, 0).program);
    let sources = BTreeMap::from([
        (
            provider.clone(),
            Arc::new(ParsedModuleUnit::new(
                provider.clone(),
                ModuleKind::Module,
                None,
                provider_source,
                provider_program,
            )),
        ),
        (
            consumer.clone(),
            Arc::new(ParsedModuleUnit::new(
                consumer.clone(),
                ModuleKind::Module,
                None,
                consumer_source,
                consumer_program,
            )),
        ),
    ]);
    let symbol = SymbolId {
        module: provider.clone(),
        name: "base".into(),
    };
    let export = LinkedExport {
        public_name: "base".into(),
        target: LinkedExportTarget::Binding(symbol.clone()),
        range: Default::default(),
    };
    let provider_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: provider.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([("base".into(), export)]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("base".into(), GlobalBindingId(0))]),
            imports: BTreeMap::new(),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let consumer_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::new(),
            imports: BTreeMap::from([("imported".into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Binding(symbol)],
        runtime_dependencies: vec![provider.clone()],
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::from([(provider.clone(), provider_linked), (consumer.clone(), consumer_linked)]),
        graphs: Default::default(),
        entry: consumer.clone(),
        initialization_order: vec![provider.clone(), consumer.clone()],
    });
    (SemanticWorkspaceInput::new(linked, sources, generation), ModuleIdPair { provider, consumer })
}

struct ModuleIdPair {
    provider: phalcom_modules::identity::ModuleId,
    consumer: phalcom_modules::identity::ModuleId,
}

#[test]
fn imported_static_body_alias_retains_canonical_const_identity() {
    let (input, modules) = imported_const_input("const base: Int = 2\nexport base\n", 1);
    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session.update(input).expect("semantic test update should succeed").snapshot;
    let read = snapshot
        .static_definition(&DeclarationId::new(modules.consumer.clone(), "ReadImported".into()))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked imported static body");
    let target = SymbolId {
        module: modules.provider,
        name: "base".into(),
    };

    assert_eq!(read.static_constant_reads.as_ref(), std::slice::from_ref(&target));
    let consumer_bindings = snapshot.module_static_bindings.get(&modules.consumer).expect("consumer binding handoff");
    assert_eq!(
        consumer_bindings
            .visible_bindings()
            .find(|(name, _)| *name == "imported")
            .map(|(_, fact)| &fact.symbol),
        Some(&target),
        "local alias keeps the linked provider identity"
    );
}

#[test]
fn provider_const_type_change_invalidates_importing_checked_body() {
    let mut session = SemanticWorkspaceSession::new();
    let (first_input, modules) = imported_const_input("const base: Int = 2\nexport base\n", 1);
    let first = session.update(first_input).expect("semantic test update should succeed").snapshot;
    let declaration = DeclarationId::new(modules.consumer.clone(), "ReadImported".into());
    let first_body = first
        .static_definition(&declaration)
        .and_then(|definition| definition.checked_body.clone())
        .expect("first checked body");

    let (second_input, _) = imported_const_input("const base: String = \"x\"\nexport base\n", 2);
    let second = session.update(second_input).expect("semantic test update should succeed").snapshot;
    let second_body = second
        .static_definition(&declaration)
        .and_then(|definition| definition.checked_body.clone())
        .expect("rechecked body");
    let symbol = SymbolId {
        module: modules.provider.clone(),
        name: "base".into(),
    };
    let first_binding = first
        .module_static_bindings
        .get(&modules.provider)
        .and_then(|bindings| bindings.get(&symbol))
        .expect("first provider binding");
    let second_binding = second
        .module_static_bindings
        .get(&modules.provider)
        .and_then(|bindings| bindings.get(&symbol))
        .expect("second provider binding");

    assert_ne!(first_binding.state.current, second_binding.state.current, "provider's checked type must change");
    assert_ne!(
        first_body.input_fingerprint, second_body.input_fingerprint,
        "provider binding facts are a checked-body input"
    );
    assert!(
        !Arc::ptr_eq(&first_body, &second_body),
        "changed provider type must replace the checked body product"
    );
}

#[test]
fn static_evaluator_executes_match_for_and_proof_predicates() {
    let source = r#"
data Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }

trait MarkerTrait {}
impl MarkerTrait for Int {}

type MatchBasic: Int {
    let opt: Option<Int> = Option::Some(42)
    match opt {
        Option::Some(x) => x
        Option::None => 0
    }
}

type MatchTuple: Int {
    let pair = (3, 7)
    match pair {
        (a, b) => a + b
    }
}

type MatchUntakenBranch: Int {
    let opt: Option<Int> = Option::Some(10)
    match opt {
        Option::Some(x) => x
        Option::None => 0
    }
}

type ForSum: Int {
    let mut sum = 0
    for x in (1, 2, 3, 4) {
        sum = sum + x
    }
    sum
}

type ForIndex: Int {
    let mut count = 0
    for x at i in (10, 20, 30) {
        count = count + i
    }
    count
}

type ForBreak: Int {
    let mut sum = 0
    for flag in (false, false, true, false) {
        if flag {
            break
        }
        sum = sum + 1
    }
    sum
}

type ForContinue: Int {
    let mut sum = 0
    for flag in (false, true, false, false) {
        if flag {
            continue
        }
        sum = sum + 1
    }
    sum
}

type ForReturn: Int {
    for flag in (false, true, false) {
        if flag {
            return 99
        }
    }
    0
}

type ForEmpty: Int {
    let mut sum = 0
    for x in () {
        sum = sum + 1
    }
    sum
}

type EquivTrue: Bool = Equivalent<Int, Int>
type EquivFalse: Bool = Equivalent<Int, String>
type SubtypeTrue: Bool = Subtype<Int, Int>
type SubtypeFalse: Bool = Subtype<String, Int>
type ConformsMarkerTrue: Bool = Conforms<Int, MarkerTrait>
type ConformsMarkerFalse: Bool = Conforms<String, MarkerTrait>
"#;

    let fixture = WorkspaceFixture::new().module("main", source).analyze();

    let evaluate_int = |name: &str| -> i64 {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|definition| definition.checked_body.as_ref())
            .unwrap_or_else(|| panic!("missing checked static definition {name}"));
        assert!(checked.diagnostics.is_empty(), "diagnostics in {name}: {:?}", checked.diagnostics);
        let mut values = StaticValueStore::new();
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let outcome = StaticEvaluator::new(checked, &mut values, &mut budget, &cancellation, &[])
            .with_type_store(fixture.analysis.snapshot.store())
            .with_semantic_tables(
                fixture.analysis.snapshot.store(),
                &fixture.analysis.snapshot.data_semantics,
                &fixture.analysis.snapshot.enum_semantics,
            )
            .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
            .with_static_definitions(fixture.analysis.snapshot.static_definitions())
            .evaluate();
        match outcome {
            StaticEvalOutcome::Reduced(v) => match values.get(v).unwrap() {
                StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(n)) => *n,
                other => panic!("expected Small int for {name}, got {other:?}"),
            },
            other => panic!("expected Reduced for {name}, got {other:?}"),
        }
    };

    let evaluate_bool = |name: &str| -> bool {
        let checked = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .and_then(|definition| definition.checked_body.as_ref())
            .unwrap_or_else(|| panic!("missing checked static definition {name}"));
        assert!(checked.diagnostics.is_empty(), "diagnostics in {name}: {:?}", checked.diagnostics);
        let mut values = StaticValueStore::new();
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let outcome = StaticEvaluator::new(checked, &mut values, &mut budget, &cancellation, &[])
            .with_type_store(fixture.analysis.snapshot.store())
            .with_semantic_tables(
                fixture.analysis.snapshot.store(),
                &fixture.analysis.snapshot.data_semantics,
                &fixture.analysis.snapshot.enum_semantics,
            )
            .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
            .with_conformance(
                &fixture.analysis.snapshot.conformance_index,
                &fixture.analysis.snapshot.conformance_witness_plans,
                &fixture.analysis.snapshot.trait_surfaces,
                &fixture.analysis.snapshot.declarations,
            )
            .with_static_definitions(fixture.analysis.snapshot.static_definitions())
            .evaluate();
        match outcome {
            StaticEvalOutcome::Reduced(v) => match values.get(v).unwrap() {
                StaticValueData::Bool(b) => *b,
                other => panic!("expected Bool for {name}, got {other:?}"),
            },
            other => panic!("expected Reduced for {name}, got {other:?}"),
        }
    };

    // Match tests
    assert_eq!(evaluate_int("MatchBasic"), 42);
    assert_eq!(evaluate_int("MatchTuple"), 10);
    assert_eq!(evaluate_int("MatchUntakenBranch"), 10);

    // For loop tests
    assert_eq!(evaluate_int("ForSum"), 10);
    assert_eq!(evaluate_int("ForIndex"), 3);
    assert_eq!(evaluate_int("ForBreak"), 2);
    assert_eq!(evaluate_int("ForContinue"), 3);
    assert_eq!(evaluate_int("ForReturn"), 99);
    assert_eq!(evaluate_int("ForEmpty"), 0);

    // Proof predicate tests
    assert_eq!(evaluate_bool("EquivTrue"), true);
    let equiv_false = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "EquivFalse"))
        .and_then(|definition| definition.checked_body.as_ref())
        .expect("checked non-equivalent proof predicate");
    let equiv_false_outcome = StaticEvaluator::new(
        equiv_false,
        &mut StaticValueStore::new(),
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[],
    )
    .with_type_store(fixture.analysis.snapshot.store())
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .evaluate();
    assert!(
        matches!(equiv_false_outcome, StaticEvalOutcome::Symbolic(_)),
        "failure to prove type equivalence must remain symbolic, got {equiv_false_outcome:?}"
    );
    assert_eq!(evaluate_bool("SubtypeTrue"), true);
    assert_eq!(evaluate_bool("SubtypeFalse"), false);
    assert_eq!(evaluate_bool("ConformsMarkerTrue"), true);
    assert_eq!(evaluate_bool("ConformsMarkerFalse"), false);
}
