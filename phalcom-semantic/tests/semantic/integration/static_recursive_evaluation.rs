use crate::semantic::support::{WorkspaceAnalysisFixture, WorkspaceFixture};
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::static_eval::{StaticEvalOutcome, StaticEvaluator};
use phalcom_semantic::static_value::{StaticValueData, StaticValueStore};
use phalcom_semantic::types::store::TypeStore;
use std::sync::Arc;

fn fixture() -> WorkspaceAnalysisFixture {
    WorkspaceFixture::new()
        .module(
            "main",
             "data Boxed<T> { value: T }\n\
             data Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\n\
             trait MarkerTrait {}\n\
             impl MarkerTrait for Int {}\n\
             type Grow<T: Type>: Type { Grow<Boxed<T>> }\n\
             type Identity<T: Type>: Type = T\n\
             type ApplyIdentity<T: Type>: Type = Identity<T>\n\
             type IsSubtype<T: Type>: Bool = Subtype<T, Int>\n\
             type SubtypeTrue: Bool = IsSubtype<Int>\n\
             type SubtypeFalse: Bool = IsSubtype<String>\n\
             type IsProduct<T: Type>: Bool = Conforms<T, Product>\n\
             type IsMarker<T: Type>: Bool = Conforms<T, MarkerTrait>\n\
             type ProductTrue: Bool = IsProduct<Boxed<Int>>\n\
             type ProductFalse: Bool = IsProduct<String>\n\
             type MarkerTrue: Bool = IsMarker<Int>\n\
             type MarkerFalse: Bool = IsMarker<String>\n\
             type ValueIdentity<N: Int>: Int = N + 1\n\
             type ApplyValueIdentity<N: Int>: Int = ValueIdentity<N>\n\
             type MixedIdentity<T: Type, N: Int>: Type = T\n\
             type ApplyMixed<T: Type, N: Int>: Type = MixedIdentity<T, N>\n\
             type Start: Type = Grow<Int>\n\
             type Heavy<N: Int>: Int { let a = N + 1\n let b = a + 1\n let c = b + 1\n c }\n\
             type Reuse: Int { let a = Heavy<1>\n let b = Heavy<1>\n a + b }\n\
             type Distinct: Int { let a = Heavy<1>\n let b = Heavy<2>\n a + b }\n\
             type RecursiveValue: Int { RecursiveValue<> }\n\
             type SelectedCycle: Int { let opt: Option<Int> = Option::Some(0)\n match opt {\n Option::Some(x) => RecursiveValue<>\n Option::None => 0\n } }\n\
             type UntakenCycle: Int { let opt: Option<Int> = Option::Some(0)\n match opt {\n Option::Some(x) => 0\n Option::None => RecursiveValue<>\n } }\n",
        )
        .analyze()
}

fn evaluate(
    fixture: &WorkspaceAnalysisFixture,
    name: &str,
    budget: &mut QueryBudget,
    values: &mut StaticValueStore,
    types: &mut TypeStore,
) -> StaticEvalOutcome {
    let checked = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", name))
        .and_then(|product| product.checked_body.as_ref())
        .unwrap_or_else(|| panic!("checked static definition {name}"));
    let cancellation = CancellationToken::new();
    let mut evaluator = StaticEvaluator::new(checked, values, budget, &cancellation, &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .with_semantic_tables_mut(types, &fixture.analysis.snapshot.data_semantics, &fixture.analysis.snapshot.enum_semantics)
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .with_conformance(
            &fixture.analysis.snapshot.conformance_index,
            &fixture.analysis.snapshot.conformance_witness_plans,
            &fixture.analysis.snapshot.trait_surfaces,
            &fixture.analysis.snapshot.declarations,
        );
    let outcome = evaluator.evaluate();
    outcome
}

#[test]
fn changing_composite_arguments_reach_budget_without_mutating_prior_snapshot() {
    let fixture = fixture();
    let prior_store = Arc::clone(&fixture.analysis.snapshot.store);
    let prior_type_count = prior_store.type_count();
    let mut session_store = Arc::clone(&prior_store);
    let mut grow_budget = QueryBudget::new(160);
    let grow = evaluate(
        &fixture,
        "Start",
        &mut grow_budget,
        &mut StaticValueStore::new(),
        Arc::make_mut(&mut session_store),
    );
    assert!(
        matches!(&grow, StaticEvalOutcome::BudgetExceeded(report) if report.kind() == phalcom_semantic::types::outcome::BudgetKind::Steps),
        "changing arguments must continue until the step budget, got {grow:?}"
    );
    assert!(session_store.type_count() > prior_type_count, "composite specializations should be interned in the active session store");
    assert_eq!(prior_store.type_count(), prior_type_count, "copy-on-write evaluation must leave the published snapshot unchanged");
}

#[test]
fn exact_applications_are_memoized_but_distinct_arguments_are_not() {
    let fixture = fixture();
    let mut session_store = Arc::clone(&fixture.analysis.snapshot.store);
    let mut reuse_budget = QueryBudget::default();
    let mut reuse_values = StaticValueStore::new();
    let reuse = evaluate(&fixture, "Reuse", &mut reuse_budget, &mut reuse_values, Arc::make_mut(&mut session_store));
    let StaticEvalOutcome::Reduced(reuse) = reuse else {
        panic!("repeated exact application should reduce, got {reuse:?}");
    };
    assert_eq!(
        reuse_values.get(reuse).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(8))
    );

    let mut distinct_budget = QueryBudget::default();
    let mut distinct_values = StaticValueStore::new();
    let distinct = evaluate(&fixture, "Distinct", &mut distinct_budget, &mut distinct_values, Arc::make_mut(&mut session_store));
    let StaticEvalOutcome::Reduced(distinct) = distinct else {
        panic!("different applications should reduce independently, got {distinct:?}");
    };
    assert_eq!(
        distinct_values.get(distinct).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(9))
    );
    assert!(
        reuse_budget.steps_taken < distinct_budget.steps_taken,
        "the repeated exact subproblem should skip one body evaluation ({} vs {})",
        reuse_budget.steps_taken,
        distinct_budget.steps_taken
    );
}

#[test]
fn recursive_cycle_is_observed_only_in_the_selected_match_arm() {
    let fixture = fixture();
    let mut session_store = Arc::clone(&fixture.analysis.snapshot.store);
    let selected = evaluate(
        &fixture,
        "SelectedCycle",
        &mut QueryBudget::default(),
        &mut StaticValueStore::new(),
        Arc::make_mut(&mut session_store),
    );
    assert!(matches!(selected, StaticEvalOutcome::Recursive(_)), "selected recursive arm should enter the exact cycle: {selected:?}");

    let mut values = StaticValueStore::new();
    let untaken = evaluate(
        &fixture,
        "UntakenCycle",
        &mut QueryBudget::default(),
        &mut values,
        Arc::make_mut(&mut session_store),
    );
    let StaticEvalOutcome::Reduced(value) = untaken else {
        panic!("untaken recursive arm must not be evaluated: {untaken:?}");
    };
    assert_eq!(values.get(value).unwrap(), &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(0)));
}

#[test]
fn open_parameter_reads_and_applications_produce_normalized_residuals() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = phalcom_semantic::static_eval::StaticResidualStore::new();

    let identity = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "Identity"))
        .expect("Identity definition");
    let identity_parameter = identity.definition.signature.parameters[0]
        .type_parameter_id
        .expect("kinded Identity parameter");
    let identity_argument = Arc::make_mut(&mut types).parameter_form(identity_parameter);
    let identity_body = identity.checked_body.as_ref().expect("Identity checked body");
    let mut identity_budget = QueryBudget::default();
    let identity_cancellation = CancellationToken::new();
    let mut identity_evaluator = StaticEvaluator::new(
        identity_body,
        &mut values,
        &mut identity_budget,
        &identity_cancellation,
        &[phalcom_semantic::static_eval::StaticArgument::Kinded(identity_argument)],
    )
    .with_type_store_mut(Arc::make_mut(&mut types))
    .with_residuals(&mut residuals);
    let identity_outcome = identity_evaluator.evaluate();
    let StaticEvalOutcome::Symbolic(parameter) = identity_outcome else {
        panic!("an open parameter read must remain symbolic, got {identity_outcome:?}");
    };
    assert!(matches!(
        residuals.get(parameter).expect("parameter residual"),
        phalcom_semantic::static_eval::StaticResidualData::Parameter { parameter: data }
            if data.owner == fixture.decl("main", "Identity") && data.index == 0
                && data.domain == phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE)
    ));

    let apply = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ApplyValueIdentity"))
        .expect("ApplyValueIdentity definition");
    let apply_body = apply.checked_body.as_ref().expect("ApplyValueIdentity checked body");
    let value_domain = apply.definition.signature.parameters[0].domain;
    let open_value = residuals
        .parameter(
            phalcom_semantic::static_eval::StaticResidualParameter {
                owner: fixture.decl("main", "ApplyValueIdentity"),
                index: 0,
                domain: value_domain,
            },
            &mut QueryBudget::default(),
        )
        .expect("open value parameter residual");
    let outcome = StaticEvaluator::new(
        apply_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Residual(open_value)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(application) = outcome else {
        panic!("an open generic application must become Apply residual, got {outcome:?}");
    };
    let repeated = StaticEvaluator::new(
        apply_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Residual(open_value)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_residuals(&mut residuals)
    .evaluate();
    assert!(
        matches!(repeated, StaticEvalOutcome::Symbolic(repeated) if repeated == application),
        "repeated open applications should normalize to the same residual: {repeated:?}"
    );
    let phalcom_semantic::static_eval::StaticResidualData::Apply {
        application: data,
        result_domain,
    } = residuals.get(application).expect("application residual")
    else {
        panic!("expected normalized Apply residual, got {:?}", residuals.get(application));
    };
    assert_eq!(data.definition, fixture.decl("main", "ValueIdentity"));
    assert_eq!(data.arguments.len(), 1);
    assert_eq!(data.arguments[0], phalcom_semantic::static_eval::StaticEvalTerm::Residual(open_value));
    assert_eq!(*result_domain, apply.definition.signature.result_domain);

    let mixed = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ApplyMixed"))
        .expect("ApplyMixed definition");
    let mixed_body = mixed.checked_body.as_ref().expect("ApplyMixed checked body");
    let type_parameter = mixed.definition.signature.parameters[0]
        .type_parameter_id
        .expect("kinded mixed parameter");
    let type_argument = Arc::make_mut(&mut types).parameter_form(type_parameter);
    let exact_value = values.intern_int(5);
    let outcome = StaticEvaluator::new(
        mixed_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[
            phalcom_semantic::static_eval::StaticArgument::Kinded(type_argument),
            phalcom_semantic::static_eval::StaticArgument::Value(exact_value),
        ],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(mixed_application) = outcome else {
        panic!("partially known arguments should remain in an Apply residual, got {outcome:?}");
    };
    let phalcom_semantic::static_eval::StaticResidualData::Apply { application, .. } =
        residuals.get(mixed_application).expect("mixed Apply residual")
    else {
        panic!("expected mixed Apply residual");
    };
    assert_eq!(application.definition, fixture.decl("main", "MixedIdentity"));
    assert_eq!(application.arguments.len(), 2);
    assert!(matches!(application.arguments[0], phalcom_semantic::static_eval::StaticEvalTerm::Residual(_)));
    assert_eq!(
        application.arguments[1],
        phalcom_semantic::static_eval::StaticEvalTerm::Value(exact_value)
    );

    let wrong_domain = residuals
        .parameter(
            phalcom_semantic::static_eval::StaticResidualParameter {
                owner: fixture.decl("main", "WrongDomain"),
                index: 0,
                domain: phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
            },
            &mut QueryBudget::default(),
        )
        .expect("wrong-domain residual");
    let outcome = StaticEvaluator::new(
        apply_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Residual(wrong_domain)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_residuals(&mut residuals)
    .evaluate();
    assert!(
        matches!(outcome, StaticEvalOutcome::Invalid(phalcom_semantic::static_eval::StaticEvalError::InvalidArgumentKind { index: 0 })),
        "invalid residual domain must fail closed: {outcome:?}"
    );
}

#[test]
fn open_proof_goals_are_preserved_as_typed_proof_residuals() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "IsSubtype"))
        .expect("IsSubtype definition");
    let parameter = product.definition.signature.parameters[0]
        .type_parameter_id
        .expect("kinded proof parameter");
    let argument = Arc::make_mut(&mut types).parameter_form(parameter);
    let body = product.checked_body.as_ref().expect("IsSubtype checked body");
    let mut values = StaticValueStore::new();
    let mut residuals = phalcom_semantic::static_eval::StaticResidualStore::new();
    let outcome = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Kinded(argument)],
    )
    .with_type_store_mut(Arc::make_mut(&mut types))
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(residual) = outcome else {
        panic!("an unresolved generic proof should remain a Proof residual, got {outcome:?}");
    };
    assert!(matches!(
        residuals.get(residual).expect("proof residual"),
        phalcom_semantic::static_eval::StaticResidualData::Proof {
            goal: phalcom_semantic::checker::proof_predicate::ProofPredicateGoal::Subtype { .. },
            result_type: _,
            ..
        }
    ));

    for name in ["IsProduct", "IsMarker"] {
        let definition = fixture
            .analysis
            .snapshot
            .static_definition(&fixture.decl("main", name))
            .expect("generic conformance definition");
        let parameter = definition.definition.signature.parameters[0]
            .type_parameter_id
            .expect("generic conformance parameter");
        let argument = Arc::make_mut(&mut types).parameter_form(parameter);
        let body = definition.checked_body.as_ref().expect("generic conformance body");
        let outcome = StaticEvaluator::new(
            body,
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            &[phalcom_semantic::static_eval::StaticArgument::Kinded(argument)],
        )
        .with_semantic_tables_mut(
            Arc::make_mut(&mut types),
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
        .with_residuals(&mut residuals)
        .evaluate();
        let StaticEvalOutcome::Symbolic(residual) = outcome else {
            panic!("open {name} proof must remain residual, got {outcome:?}");
        };
        assert!(matches!(
            residuals.get(residual).expect("generic conformance proof residual"),
            phalcom_semantic::static_eval::StaticResidualData::Proof {
                goal: phalcom_semantic::checker::proof_predicate::ProofPredicateGoal::Conforms { .. },
                ..
            }
        ));
    }
}

#[test]
fn proof_residuals_specialize_through_structural_and_user_conformance() {
    let fixture = fixture();
    let mut session_store = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    for (name, expected) in [
        ("SubtypeTrue", true),
        ("SubtypeFalse", false),
        ("ProductTrue", true),
        ("ProductFalse", false),
        ("MarkerTrue", true),
        ("MarkerFalse", false),
    ] {
        let outcome = evaluate(
            &fixture,
            name,
            &mut QueryBudget::default(),
            &mut values,
            Arc::make_mut(&mut session_store),
        );
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("{name} should specialize its proof to Bool, got {outcome:?}");
        };
        assert_eq!(values.get(value).unwrap(), &StaticValueData::Bool(expected), "{name}");
    }
}

#[test]
fn unresolved_associated_projection_becomes_member_projection_residual() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let identity = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "Identity"))
        .expect("Identity definition");
    let identity_body = identity.checked_body.as_ref().expect("Identity checked body");

    let structural_ids = phalcom_semantic::core_surface::identity::StructuralUniverseIds::new();
    let subject = Arc::make_mut(&mut types).unit();
    let projection = Arc::make_mut(&mut types).associated_projection(
        subject,
        phalcom_semantic::traits::TraitRef::new(structural_ids.product.clone(), Vec::new()),
        phalcom_semantic::traits::AssociatedTypeRequirementId::new(structural_ids.product.clone(), 1),
    );
    assert!(matches!(
        types.get(projection),
        phalcom_semantic::types::store::TypeData::AssociatedProjection(_)
    ));

    let mut values = StaticValueStore::new();
    let mut residuals = phalcom_semantic::static_eval::StaticResidualStore::new();
    let outcome = StaticEvaluator::new(
        identity_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Kinded(projection)],
    )
    .with_type_store_mut(Arc::make_mut(&mut types))
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(residual) = outcome else {
        panic!("an unresolved associated projection must remain a member projection residual, got {outcome:?}");
    };
    assert!(matches!(
        residuals.get(residual).expect("member projection residual"),
        phalcom_semantic::static_eval::StaticResidualData::MemberProjection {
            projection: phalcom_semantic::member_projection::MemberProjection {
                subject: phalcom_semantic::member_projection::MemberProjectionSubject::Type(projected_subject),
                member: phalcom_semantic::member_projection::MemberDeclarationIdentity::TraitAssociated { trait_ref, requirement },
                ..
            },
            result_domain: phalcom_semantic::static_domain::StaticValueDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
            ..
        } if *projected_subject == subject
            && trait_ref.declaration == structural_ids.product
            && requirement.index == 1
    ));
}

#[test]
fn checked_structural_projection_residualizes_open_and_reduces_exact_subjects() {
    let fixture = fixture();
    let identity = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "Identity"))
        .expect("Identity definition");
    let mut body = identity.checked_body.as_ref().expect("Identity checked body").as_ref().clone();
    let parameter = identity.definition.signature.parameters[0]
        .type_parameter_id
        .expect("Identity kinded type parameter");
    let structural_ids = phalcom_semantic::core_surface::identity::StructuralUniverseIds::new();
    let product_shape = fixture
        .analysis
        .snapshot
        .declarations
        .form(&structural_ids.product_shape)
        .expect("canonical ProductShape form");
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let parameter_form = Arc::make_mut(&mut types).parameter_form(parameter);
    let projection = phalcom_semantic::member_projection::MemberProjection {
        subject: phalcom_semantic::member_projection::MemberProjectionSubject::Type(parameter_form),
        member: phalcom_semantic::member_projection::MemberDeclarationIdentity::TraitAssociated {
            trait_ref: phalcom_semantic::traits::TraitRef::new(structural_ids.product.clone(), Vec::new()),
            requirement: phalcom_semantic::traits::AssociatedTypeRequirementId::new(structural_ids.product.clone(), 1),
        },
        arguments: Box::new([]),
        evidence: None,
        result_domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(product_shape),
    };
    let expression = *body.expressions.keys().next().expect("Identity checked expression site");
    Arc::make_mut(&mut body.member_projections).insert(expression, projection);
    body.result = phalcom_semantic::static_domain::StaticValueDomain::ValueType(product_shape);
    let body = body;
    assert!(body.member_projections.values().any(|projection| {
        matches!(
            projection,
            phalcom_semantic::member_projection::MemberProjection {
                member: phalcom_semantic::member_projection::MemberDeclarationIdentity::TraitAssociated { trait_ref, requirement },
                result_domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(domain),
                ..
            } if trait_ref.declaration == structural_ids.product && requirement.index == 1 && *domain == product_shape
        )
    }), "the checked body should publish the canonical Product::Shape site and result domain");

    let mut values = StaticValueStore::new();
    let mut residuals = phalcom_semantic::static_eval::StaticResidualStore::new();
    let open = StaticEvaluator::new(
        &body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Kinded(parameter_form)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
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
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(residual) = open else {
        panic!("an open structural projection must remain residual, got {open:?}");
    };
    assert!(matches!(
        residuals.get(residual).expect("structural member residual"),
        phalcom_semantic::static_eval::StaticResidualData::MemberProjection {
            projection: phalcom_semantic::member_projection::MemberProjection {
                subject: phalcom_semantic::member_projection::MemberProjectionSubject::Type(subject),
                member: phalcom_semantic::member_projection::MemberDeclarationIdentity::TraitAssociated { trait_ref, requirement },
                result_domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(domain),
                ..
            },
            result_domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(residual_domain),
            ..
        } if *subject == parameter_form
            && trait_ref.declaration == structural_ids.product
            && requirement.index == 1
            && *domain == product_shape
            && *residual_domain == product_shape
    ));

    let unit = Arc::make_mut(&mut types).unit();
    let exact = StaticEvaluator::new(
        &body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[phalcom_semantic::static_eval::StaticArgument::Kinded(unit)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
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
    .evaluate();
    let StaticEvalOutcome::Reduced(value) = exact else {
        panic!("an exact structural projection should reduce to a static value, got {exact:?}");
    };
    assert!(matches!(values.get(value), Ok(StaticValueData::Data(_) | StaticValueData::Variant(_))), "expected a reflected ProductShape value, got {:?}", values.get(value));
    let actual_type = values
        .static_value_type(
            value,
            Arc::make_mut(&mut types),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .expect("reflected ProductShape type");
    assert_eq!(Arc::make_mut(&mut types).kind_of(actual_type), phalcom_semantic::types::id::KindId::TYPE);
}
