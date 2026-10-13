use crate::semantic::support::{WorkspaceAnalysisFixture, WorkspaceFixture};
use phalcom_semantic::core_surface::identity::{CoreDeclarationIds, StructuralUniverseIds};
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::member_projection::{MemberDeclarationIdentity, MemberProjection, MemberProjectionSubject};
use phalcom_semantic::static_domain::StaticValueDomain;
use phalcom_semantic::static_eval::application::{StaticSpecializationArgument, StaticSpecializationEnvironmentKey};
use phalcom_semantic::static_eval::residual::{StaticResidualParameter, StaticResidualStore};
use phalcom_semantic::static_eval::{StaticArgument, StaticEvalOutcome, StaticEvaluator, specialize_residual};
use phalcom_semantic::static_value::{StaticValueData, StaticValueStore};
use phalcom_semantic::traits::{AssociatedTypeRequirementId, TraitRef};
use phalcom_semantic::types::id::KindId;
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
    assert!(
        session_store.type_count() > prior_type_count,
        "composite specializations should be interned in the active session store"
    );
    assert_eq!(
        prior_store.type_count(),
        prior_type_count,
        "copy-on-write evaluation must leave the published snapshot unchanged"
    );
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
    let distinct = evaluate(
        &fixture,
        "Distinct",
        &mut distinct_budget,
        &mut distinct_values,
        Arc::make_mut(&mut session_store),
    );
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
    assert!(
        matches!(selected, StaticEvalOutcome::Recursive(_)),
        "selected recursive arm should enter the exact cycle: {selected:?}"
    );

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
    assert_eq!(
        values.get(value).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(0))
    );
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
    let type_parameter = mixed.definition.signature.parameters[0].type_parameter_id.expect("kinded mixed parameter");
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
    let phalcom_semantic::static_eval::StaticResidualData::Apply { application, .. } = residuals.get(mixed_application).expect("mixed Apply residual") else {
        panic!("expected mixed Apply residual");
    };
    assert_eq!(application.definition, fixture.decl("main", "MixedIdentity"));
    assert_eq!(application.arguments.len(), 2);
    assert!(matches!(application.arguments[0], phalcom_semantic::static_eval::StaticEvalTerm::Residual(_)));
    assert_eq!(application.arguments[1], phalcom_semantic::static_eval::StaticEvalTerm::Value(exact_value));

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
        matches!(
            outcome,
            StaticEvalOutcome::Invalid(phalcom_semantic::static_eval::StaticEvalError::InvalidArgumentKind { index: 0 })
        ),
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
    let parameter = product.definition.signature.parameters[0].type_parameter_id.expect("kinded proof parameter");
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
        let outcome = evaluate(&fixture, name, &mut QueryBudget::default(), &mut values, Arc::make_mut(&mut session_store));
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
    assert!(
        body.member_projections.values().any(|projection| {
            matches!(
                projection,
                phalcom_semantic::member_projection::MemberProjection {
                    member: phalcom_semantic::member_projection::MemberDeclarationIdentity::TraitAssociated { trait_ref, requirement },
                    result_domain: phalcom_semantic::static_domain::StaticValueDomain::ValueType(domain),
                    ..
                } if trait_ref.declaration == structural_ids.product && requirement.index == 1 && *domain == product_shape
            )
        }),
        "the checked body should publish the canonical Product::Shape site and result domain"
    );

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
    assert!(
        matches!(values.get(value), Ok(StaticValueData::Data(_) | StaticValueData::Variant(_))),
        "expected a reflected ProductShape value, got {:?}",
        values.get(value)
    );
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

#[test]
fn recursive_activation_shadowing() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();

    // CV-ENV-02: Recursive activation shadowing
    // Formal parameter bindings: later frames shadow earlier frames without conflict.
    let decl_grow = fixture.decl("main", "Grow");
    let param_grow = StaticResidualParameter {
        owner: decl_grow.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    let core_ids = CoreDeclarationIds::default();
    let int_type = fixture.analysis.snapshot.declarations.form(&core_ids.int).expect("Int type");

    let boxed_decl = fixture.decl("main", "Boxed");
    let ctor_kind = Arc::make_mut(&mut types).arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let boxed_origin = Arc::make_mut(&mut types).nominal_form(boxed_decl, ctor_kind);
    let boxed_int = Arc::make_mut(&mut types).apply_type_form(boxed_origin, &[int_type]).unwrap();
    let boxed_boxed_int = Arc::make_mut(&mut types).apply_type_form(boxed_origin, &[boxed_int]).unwrap();

    // Frame 0: Grow.0 = Int
    let env_frame0 =
        StaticSpecializationEnvironmentKey::normalized(vec![(param_grow.clone(), StaticSpecializationArgument::Kinded(int_type))], vec![], vec![]).unwrap();

    // Frame 1: overlays Grow.0 = Boxed<Int> (shadows Frame 0 binding)
    let env_frame1 = env_frame0
        .overlay_arguments([(param_grow.clone(), StaticSpecializationArgument::Kinded(boxed_int))])
        .expect("overlay_arguments must shadow older formal binding without conflict");
    assert_eq!(env_frame1.argument_bindings().len(), 1);
    assert_eq!(env_frame1.argument_bindings()[0].1, StaticSpecializationArgument::Kinded(boxed_int));

    // Frame 2: overlays Grow.0 = Boxed<Boxed<Int>> (shadows Frame 1 binding)
    let env_frame2 = env_frame1
        .overlay_arguments([(param_grow.clone(), StaticSpecializationArgument::Kinded(boxed_boxed_int))])
        .expect("overlay_arguments must shadow again");
    assert_eq!(env_frame2.argument_bindings().len(), 1);
    assert_eq!(env_frame2.argument_bindings()[0].1, StaticSpecializationArgument::Kinded(boxed_boxed_int));

    // CV-ENV-03: Unrelated inherited bindings preserved across inner calls
    let decl_other = fixture.decl("main", "Other");
    let param_other = StaticResidualParameter {
        owner: decl_other,
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let env_outer =
        StaticSpecializationEnvironmentKey::normalized(vec![(param_other.clone(), StaticSpecializationArgument::Kinded(int_type))], vec![], vec![]).unwrap();
    let env_inner = env_outer
        .overlay_arguments([(param_grow.clone(), StaticSpecializationArgument::Kinded(boxed_int))])
        .unwrap();
    assert_eq!(env_inner.argument_bindings().len(), 2, "unrelated outer bindings must be preserved");

    // CV-ENV-05: Repeating cycle identity
    // Calling recursive function F<A> -> F<A> cycles promptly without pollution from caller argument history
    let mut cycle_budget = QueryBudget::default();
    let cycle_outcome = evaluate(&fixture, "SelectedCycle", &mut cycle_budget, &mut values, Arc::make_mut(&mut types));
    assert!(
        matches!(cycle_outcome, StaticEvalOutcome::Recursive(_)),
        "SelectedCycle must cycle cleanly: {cycle_outcome:?}"
    );

    // CV-ENV-06: Memo identity
    // Same args + same ambient proof = memo hit (Reuse takes fewer steps than Distinct)
    let mut reuse_budget = QueryBudget::default();
    let mut reuse_values = StaticValueStore::new();
    let reuse = evaluate(&fixture, "Reuse", &mut reuse_budget, &mut reuse_values, Arc::make_mut(&mut types));
    assert!(matches!(reuse, StaticEvalOutcome::Reduced(_)));

    let mut distinct_budget = QueryBudget::default();
    let mut distinct_values = StaticValueStore::new();
    let distinct = evaluate(&fixture, "Distinct", &mut distinct_budget, &mut distinct_values, Arc::make_mut(&mut types));
    assert!(matches!(distinct, StaticEvalOutcome::Reduced(_)));
    assert!(
        reuse_budget.steps_taken < distinct_budget.steps_taken,
        "memo hit must save steps: {} vs {}",
        reuse_budget.steps_taken,
        distinct_budget.steps_taken
    );
}

fn specialize_helper(
    fixture: &WorkspaceAnalysisFixture,
    body: &phalcom_semantic::static_definition::CheckedStaticBody,
    residual: phalcom_semantic::static_eval::StaticResidualRef,
    supplied: &StaticSpecializationEnvironmentKey,
    values: &mut StaticValueStore,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    types: &mut TypeStore,
    residuals: &mut StaticResidualStore,
) -> StaticEvalOutcome {
    let mut evaluator = StaticEvaluator::new(body, values, budget, cancellation, &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .with_semantic_tables_mut(types, &fixture.analysis.snapshot.data_semantics, &fixture.analysis.snapshot.enum_semantics)
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .with_conformance(
            &fixture.analysis.snapshot.conformance_index,
            &fixture.analysis.snapshot.conformance_witness_plans,
            &fixture.analysis.snapshot.trait_surfaces,
            &fixture.analysis.snapshot.declarations,
        )
        .with_residuals(residuals);
    specialize_residual(&mut evaluator, residual, supplied)
}

#[test]
fn direct_residual_escape_and_parity() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();

    // 1. CV-ESC-01: Direct proof escape
    let is_subtype = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "IsSubtype"))
        .expect("IsSubtype definition");
    let param_t = is_subtype.definition.signature.parameters[0].type_parameter_id.expect("kinded proof parameter");
    let param_form = Arc::make_mut(&mut types).parameter_form(param_t);
    let is_subtype_body = is_subtype.checked_body.as_ref().expect("IsSubtype checked body");

    let open_outcome = StaticEvaluator::new(is_subtype_body, &mut values, &mut budget, &cancellation, &[StaticArgument::Kinded(param_form)])
        .with_type_store_mut(Arc::make_mut(&mut types))
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .with_residuals(&mut residuals)
        .evaluate();

    let StaticEvalOutcome::Symbolic(proof_res) = open_outcome else {
        panic!("open IsSubtype<T> must remain a Proof residual, got {open_outcome:?}");
    };

    // Caller frame is torn down. Now specialize proof_res with T = Int
    let core_ids = CoreDeclarationIds::default();
    let int_type = fixture.analysis.snapshot.declarations.form(&core_ids.int).expect("Int type");
    let string_type = fixture.analysis.snapshot.declarations.form(&core_ids.string).expect("String type");

    let supplied_int = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(param_t, int_type)], vec![]).unwrap();

    let spec_true = specialize_helper(
        &fixture,
        is_subtype_body,
        proof_res,
        &supplied_int,
        &mut values,
        &mut budget,
        &cancellation,
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(val_true) = spec_true else {
        panic!("specializing with T = Int should reduce, got {spec_true:?}");
    };
    let val_true_data = values.get(val_true).unwrap().clone();
    assert_eq!(val_true_data, StaticValueData::Bool(true));

    // Compare with cold exact SubtypeTrue
    let cold_true = evaluate(&fixture, "SubtypeTrue", &mut QueryBudget::default(), &mut values, Arc::make_mut(&mut types));
    let StaticEvalOutcome::Reduced(cold_val_true) = cold_true else {
        panic!("SubtypeTrue should reduce, got {cold_true:?}");
    };
    assert_eq!(val_true_data, *values.get(cold_val_true).unwrap(), "specialized and cold exact must match");

    // Specialize with T = String
    let supplied_str = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(param_t, string_type)], vec![]).unwrap();
    let spec_false = specialize_helper(
        &fixture,
        is_subtype_body,
        proof_res,
        &supplied_str,
        &mut values,
        &mut budget,
        &cancellation,
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(val_false) = spec_false else {
        panic!("specializing with T = String should reduce, got {spec_false:?}");
    };
    let val_false_data = values.get(val_false).unwrap().clone();
    assert_eq!(val_false_data, StaticValueData::Bool(false));

    let cold_false = evaluate(&fixture, "SubtypeFalse", &mut QueryBudget::default(), &mut values, Arc::make_mut(&mut types));
    let StaticEvalOutcome::Reduced(cold_val_false) = cold_false else {
        panic!("SubtypeFalse should reduce, got {cold_false:?}");
    };
    assert_eq!(
        val_false_data,
        *values.get(cold_val_false).unwrap(),
        "specialized and cold exact false must match"
    );

    // CV-ID-01: No-New-Facts identity
    let empty_supplied = StaticSpecializationEnvironmentKey::default();
    let no_facts = specialize_helper(
        &fixture,
        is_subtype_body,
        proof_res,
        &empty_supplied,
        &mut values,
        &mut budget,
        &cancellation,
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    assert_eq!(
        no_facts,
        StaticEvalOutcome::Symbolic(proof_res),
        "no-new-facts must reuse existing interned residual reference"
    );

    // 2. CV-ESC-02: Direct MemberProjection escape
    let identity = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "Identity"))
        .expect("Identity definition");
    let mut member_body = identity.checked_body.as_ref().expect("Identity checked body").as_ref().clone();
    let identity_param = identity.definition.signature.parameters[0]
        .type_parameter_id
        .expect("Identity kinded type parameter");
    let structural_ids = StructuralUniverseIds::new();
    let product_shape = fixture
        .analysis
        .snapshot
        .declarations
        .form(&structural_ids.product_shape)
        .expect("canonical ProductShape form");
    let identity_param_form = Arc::make_mut(&mut types).parameter_form(identity_param);
    let projection = MemberProjection {
        subject: MemberProjectionSubject::Type(identity_param_form),
        member: MemberDeclarationIdentity::TraitAssociated {
            trait_ref: TraitRef::new(structural_ids.product.clone(), Vec::new()),
            requirement: AssociatedTypeRequirementId::new(structural_ids.product.clone(), 1),
        },
        arguments: Box::new([]),
        evidence: None,
        result_domain: StaticValueDomain::ValueType(product_shape),
    };
    let expression = *member_body.expressions.keys().next().expect("Identity checked expression site");
    Arc::make_mut(&mut member_body.member_projections).insert(expression, projection);
    member_body.result = StaticValueDomain::ValueType(product_shape);
    let member_body = member_body;

    let open_member = StaticEvaluator::new(
        &member_body,
        &mut values,
        &mut budget,
        &cancellation,
        &[StaticArgument::Kinded(identity_param_form)],
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
    let StaticEvalOutcome::Symbolic(member_res) = open_member else {
        panic!("open structural projection must remain residual, got {open_member:?}");
    };

    // Cold exact evaluation on unit
    let unit = Arc::make_mut(&mut types).unit();
    let exact_member = StaticEvaluator::new(&member_body, &mut values, &mut budget, &cancellation, &[StaticArgument::Kinded(unit)])
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
    let StaticEvalOutcome::Reduced(cold_member_val) = exact_member else {
        panic!("exact structural projection must reduce, got {exact_member:?}");
    };
    let cold_member_data = values.get(cold_member_val).unwrap().clone();

    // Specialize escaped member_res with T = unit
    let supplied_unit = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(identity_param, unit)], vec![]).unwrap();

    let spec_member = specialize_helper(
        &fixture,
        &member_body,
        member_res,
        &supplied_unit,
        &mut values,
        &mut budget,
        &cancellation,
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(spec_member_val) = spec_member else {
        panic!("specializing escaped member projection must reduce, got {spec_member:?}");
    };
    assert_eq!(
        values.get(spec_member_val).unwrap(),
        &cold_member_data,
        "escaped member projection specialized value must match cold exact"
    );

    // CV-ID-02: Idempotence: specializing with same environment produces same result
    let spec_again = specialize_helper(
        &fixture,
        &member_body,
        member_res,
        &supplied_unit,
        &mut values,
        &mut budget,
        &cancellation,
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    assert_eq!(spec_again, StaticEvalOutcome::Reduced(spec_member_val), "specialization must be idempotent");

    // Also test CV-ESC-01 / CV-SUB-01 with open Apply residual
    let apply_def = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ApplyValueIdentity"))
        .expect("ApplyValueIdentity definition");
    let apply_body = apply_def.checked_body.as_ref().expect("ApplyValueIdentity checked body");
    let value_domain = apply_def.definition.signature.parameters[0].domain;
    let open_param = StaticResidualParameter {
        owner: fixture.decl("main", "ApplyValueIdentity"),
        index: 0,
        domain: value_domain,
    };
    let open_value = residuals.parameter(open_param.clone(), &mut QueryBudget::default()).expect("open param");
    let apply_outcome = StaticEvaluator::new(
        apply_body,
        &mut values,
        &mut QueryBudget::default(),
        &cancellation,
        &[StaticArgument::Residual(open_value)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(apply_res) = apply_outcome else {
        panic!("open ApplyValueIdentity must return Apply residual, got {apply_outcome:?}");
    };

    let val_42 = values.intern_int(42);
    let supplied_val = StaticSpecializationEnvironmentKey::normalized(vec![(open_param, StaticSpecializationArgument::Value(val_42))], vec![], vec![]).unwrap();
    let spec_apply = specialize_helper(
        &fixture,
        apply_body,
        apply_res,
        &supplied_val,
        &mut values,
        &mut budget,
        &cancellation,
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(spec_apply_val) = spec_apply else {
        panic!("specializing Apply residual with concrete value must reduce, got {spec_apply:?}");
    };
    assert_eq!(
        values.get(spec_apply_val).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(43)),
        "ApplyValueIdentity<42> should evaluate 42 + 1 = 43"
    );
}

#[test]
fn specialization_cancellation_and_budget() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let is_subtype = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "IsSubtype"))
        .expect("IsSubtype definition");
    let param_t = is_subtype.definition.signature.parameters[0].type_parameter_id.expect("kinded proof parameter");
    let param_form = Arc::make_mut(&mut types).parameter_form(param_t);
    let is_subtype_body = is_subtype.checked_body.as_ref().expect("IsSubtype checked body");
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();

    let open_outcome = StaticEvaluator::new(is_subtype_body, &mut values, &mut budget, &cancellation, &[StaticArgument::Kinded(param_form)])
        .with_type_store_mut(Arc::make_mut(&mut types))
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .with_residuals(&mut residuals)
        .evaluate();
    let StaticEvalOutcome::Symbolic(proof_res) = open_outcome else {
        panic!("open IsSubtype<T> must remain a Proof residual");
    };

    let core_ids = CoreDeclarationIds::default();
    let int_type = fixture.analysis.snapshot.declarations.form(&core_ids.int).expect("Int type");
    let supplied_int = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(param_t, int_type)], vec![]).unwrap();

    // 1. Cancellation check (CV-RES-01)
    let cancelled_token = CancellationToken::new();
    cancelled_token.cancel();
    let cancelled_outcome = {
        let mut cancelled_evaluator = StaticEvaluator::new(is_subtype_body, &mut values, &mut budget, &cancelled_token, &[]).with_residuals(&mut residuals);
        specialize_residual(&mut cancelled_evaluator, proof_res, &supplied_int)
    };
    assert_eq!(cancelled_outcome, StaticEvalOutcome::Cancelled, "cancelled token must yield Cancelled");

    // 2. Budget exhaustion check (CV-RES-01)
    let mut zero_budget = QueryBudget::new(0);
    let uncanceled_token = CancellationToken::new();
    let budget_outcome = {
        let mut budget_evaluator = StaticEvaluator::new(is_subtype_body, &mut values, &mut zero_budget, &uncanceled_token, &[]).with_residuals(&mut residuals);
        specialize_residual(&mut budget_evaluator, proof_res, &supplied_int)
    };
    assert!(
        matches!(budget_outcome, StaticEvalOutcome::BudgetExceeded(_)),
        "specialization with exhausted budget must yield BudgetExceeded, got {budget_outcome:?}"
    );
}
