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
             data Option<T> { Some(_ value: T) -> Option<T> None -> Option<T> }\n\
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
             type Choose<B: Bool>: Int { if B { 11 } else { 22 } }\n\
             type ChooseTrue: Int = Choose<true>\n\
             type ChooseFalse: Int = Choose<false>\n\
             type ChooseSafe<B: Bool>: Int { if B { 11 } else { RecursiveValue<> } }\n\
             type MutatingChoice<B: Bool>: Int { let mut result = 0\n if B { result = 1 } else { result = 2 }\n result }\n\
             type MatchChar<C: Char>: Int { match C { 'x' => 3\n _ => 4 } }\n\
             type CountTypes<*P: TypePack>: Int { let mut count = 0\n for _ in P { count = count + 1 }\n count }\n\
             type PairValue<N: Int>: (Int, Int) { (N, 7) }\n\
             type PartialRecord<N: Int>: { [#same]: Int, [42]: Int } { { [#same]: N, [42]: 7 } }\n\
              type PartialVariant<N: Int>: Option<Int> { Option::Some(N) }\n\
              type PartialData<N: Int>: Boxed<Int> { Boxed(value: N) }\n\
              type OptionValue: Option<Int> = Option::Some(9)\n\
             type MatchOption<O: Option<Int>>: Int { match O { Option::Some(x) => x\n Option::None => 0 } }\n\
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

    evaluator.evaluate()
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
fn open_value_condition_defers_to_the_enclosing_application() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let choose = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "Choose"))
        .expect("Choose definition");
    let body = choose.checked_body.as_ref().expect("Choose checked body");
    assert!(body.diagnostics.is_empty(), "Choose must check: {:?}", body.diagnostics);
    let parameter = StaticResidualParameter {
        owner: fixture.decl("main", "Choose"),
        index: 0,
        domain: choose.definition.signature.parameters[0].domain,
    };
    let open_parameter = residuals
        .parameter(parameter.clone(), &mut QueryBudget::default())
        .expect("open Bool parameter");
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(open_parameter)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(residual) = open else {
        panic!("open conditional must remain a semantic residual: {open:?}");
    };
    assert!(matches!(
        residuals.get(residual).expect("conditional residual"),
        phalcom_semantic::static_eval::StaticResidualData::Apply { application, result_domain }
            if application.definition == parameter.owner && *result_domain == body.result
    ));

    for (value, exact_name, expected) in [(true, "ChooseTrue", 11), (false, "ChooseFalse", 22)] {
        let bool_value = values.intern_bool(value);
        let supplied = StaticSpecializationEnvironmentKey::normalized([(parameter.clone(), StaticSpecializationArgument::Value(bool_value))], [], [])
            .expect("specialization");
        let specialized = specialize_helper(
            &fixture,
            body,
            residual,
            &supplied,
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        );
        let StaticEvalOutcome::Reduced(actual) = specialized else {
            panic!("open conditional must specialize to the selected branch: {specialized:?}");
        };
        let cold = evaluate(&fixture, exact_name, &mut QueryBudget::default(), &mut values, Arc::make_mut(&mut types));
        let StaticEvalOutcome::Reduced(cold_value) = cold else {
            panic!("cold conditional must reduce: {cold:?}");
        };
        assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());
        assert_eq!(
            values.get(actual).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(expected))
        );
    }

    let safe = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ChooseSafe"))
        .expect("ChooseSafe definition");
    let safe_body = safe.checked_body.as_ref().expect("ChooseSafe checked body");
    assert!(safe_body.diagnostics.is_empty(), "ChooseSafe must check: {:?}", safe_body.diagnostics);
    let safe_parameter = StaticResidualParameter {
        owner: fixture.decl("main", "ChooseSafe"),
        index: 0,
        domain: safe.definition.signature.parameters[0].domain,
    };
    let open_safe = residuals.parameter(safe_parameter.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        safe_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(open_safe)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(safe_residual) = open else {
        panic!("the unselected recursive branch must not execute for an open condition: {open:?}");
    };
    let true_value = values.intern_bool(true);
    let supplied = StaticSpecializationEnvironmentKey::normalized([(safe_parameter, StaticSpecializationArgument::Value(true_value))], [], []).unwrap();
    let selected = specialize_helper(
        &fixture,
        safe_body,
        safe_residual,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(selected) = selected else {
        panic!("the recursive untaken branch must remain unobserved: {selected:?}");
    };
    assert_eq!(
        values.get(selected).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(11))
    );

    let mutating = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "MutatingChoice"))
        .expect("MutatingChoice definition");
    let mutating_body = mutating.checked_body.as_ref().expect("MutatingChoice checked body");
    assert!(
        mutating_body.diagnostics.is_empty(),
        "MutatingChoice must check: {:?}",
        mutating_body.diagnostics
    );
    let mutating_parameter = StaticResidualParameter {
        owner: fixture.decl("main", "MutatingChoice"),
        index: 0,
        domain: mutating.definition.signature.parameters[0].domain,
    };
    let open_parameter = residuals.parameter(mutating_parameter.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        mutating_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(open_parameter)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(mutation_residual) = open else {
        panic!("open conditional must not commit either branch's local assignment: {open:?}");
    };
    for (value, expected) in [(true, 1), (false, 2)] {
        let argument = values.intern_bool(value);
        let supplied =
            StaticSpecializationEnvironmentKey::normalized([(mutating_parameter.clone(), StaticSpecializationArgument::Value(argument))], [], []).unwrap();
        let specialized = specialize_helper(
            &fixture,
            mutating_body,
            mutation_residual,
            &supplied,
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        );
        let StaticEvalOutcome::Reduced(actual) = specialized else {
            panic!("exactly one branch must update the local: {specialized:?}");
        };
        assert_eq!(
            values.get(actual).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(expected))
        );
    }
}

#[test]
fn open_match_scrutinee_defers_without_losing_arm_order() {
    let fixture = fixture();
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let match_char = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "MatchChar"))
        .expect("MatchChar definition");
    let body = match_char.checked_body.as_ref().expect("MatchChar checked body");
    assert!(body.diagnostics.is_empty(), "MatchChar must check: {:?}", body.diagnostics);
    let parameter = StaticResidualParameter {
        owner: fixture.decl("main", "MatchChar"),
        index: 0,
        domain: match_char.definition.signature.parameters[0].domain,
    };
    let open_parameter = residuals.parameter(parameter.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(open_parameter)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(residual) = open else {
        panic!("open scrutinee must preserve the enclosing checked match: {open:?}");
    };
    assert!(matches!(
        residuals.get(residual).expect("open match residual"),
        phalcom_semantic::static_eval::StaticResidualData::Apply { application, result_domain }
            if application.definition == parameter.owner && *result_domain == body.result
    ));
    for (character, expected) in [('x', 3), ('z', 4)] {
        let argument = values.intern_char(character);
        let cold = StaticEvaluator::new(
            body,
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            &[StaticArgument::Value(argument)],
        )
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .with_semantic_tables_mut(
            Arc::make_mut(&mut types),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .evaluate();
        let StaticEvalOutcome::Reduced(cold_result) = cold else {
            panic!("exact match must select its first matching arm: {cold:?}");
        };
        let supplied = StaticSpecializationEnvironmentKey::normalized([(parameter.clone(), StaticSpecializationArgument::Value(argument))], [], []).unwrap();
        let specialized = specialize_helper(
            &fixture,
            body,
            residual,
            &supplied,
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        );
        let StaticEvalOutcome::Reduced(specialized_result) = specialized else {
            panic!("specialized match must select its first matching arm: {specialized:?}");
        };
        assert_eq!(values.get(cold_result).unwrap(), values.get(specialized_result).unwrap());
        assert_eq!(
            values.get(specialized_result).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(expected))
        );
    }

    let option = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "MatchOption"))
        .expect("MatchOption definition");
    let option_body = option.checked_body.as_ref().expect("MatchOption checked body");
    assert!(option_body.diagnostics.is_empty(), "MatchOption must check: {:?}", option_body.diagnostics);
    let parameter = StaticResidualParameter {
        owner: fixture.decl("main", "MatchOption"),
        index: 0,
        domain: option.definition.signature.parameters[0].domain,
    };
    let open_parameter = residuals.parameter(parameter.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        option_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(open_parameter)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(option_residual) = open else {
        panic!("an open variant match must retain the checked application: {open:?}");
    };
    let option_value = evaluate(&fixture, "OptionValue", &mut QueryBudget::default(), &mut values, Arc::make_mut(&mut types));
    let StaticEvalOutcome::Reduced(argument) = option_value else {
        panic!("canonical Option::Some value must reduce: {option_value:?}");
    };
    let cold = StaticEvaluator::new(
        option_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(argument)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_result) = cold else {
        panic!("exact canonical variant match must reduce: {cold:?}");
    };
    let supplied = StaticSpecializationEnvironmentKey::normalized([(parameter, StaticSpecializationArgument::Value(argument))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        option_body,
        option_residual,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(specialized_result) = specialized else {
        panic!("specializing the canonical variant match must select Some: {specialized:?}");
    };
    assert_eq!(values.get(specialized_result).unwrap(), values.get(cold_result).unwrap());
    assert_eq!(
        values.get(specialized_result).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(9))
    );
}

#[test]
fn iterative_static_type_pack_algorithm_uses_checked_mode_and_finite_arguments() {
    let fixture = fixture();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "CountTypes"))
        .expect("CountTypes definition");
    let body = product.checked_body.as_ref().expect("checked Pack counter");
    assert!(body.diagnostics.is_empty(), "Pack counter must check: {:?}", body.diagnostics);
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).unwrap();
    let pack = Arc::make_mut(&mut types).type_pack(Box::new([int, int, int])).unwrap();
    let mut values = StaticValueStore::new();
    let outcome = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Kinded(pack)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .evaluate();
    let StaticEvalOutcome::Reduced(value) = outcome else {
        panic!("finite TypePack iteration must count its exact entries: {outcome:?}");
    };
    assert_eq!(
        values.get(value).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(3))
    );

    let parameter_id = product.definition.signature.parameters[0].type_parameter_id.expect("kinded pack parameter");
    let open_form = Arc::make_mut(&mut types).parameter_form(parameter_id);
    let mut residuals = StaticResidualStore::new();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Kinded(open_form)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(open) = open else {
        panic!("unknown pack contents must preserve the checked application: {open:?}");
    };
    assert!(
        matches!(residuals.get(open).unwrap(), phalcom_semantic::static_eval::StaticResidualData::Apply { application, .. } if application.definition == fixture.decl("main", "CountTypes"))
    );
    let supplied = StaticSpecializationEnvironmentKey::normalized(
        [(
            StaticResidualParameter {
                owner: fixture.decl("main", "CountTypes"),
                index: 0,
                domain: product.definition.signature.parameters[0].domain,
            },
            StaticSpecializationArgument::Kinded(pack),
        )],
        [],
        [],
    )
    .unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        open,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specializing the finite TypePack must match cold evaluation: {specialized:?}");
    };
    assert_eq!(
        values.get(actual).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(3))
    );
}

#[test]
fn iterative_static_type_pack_count_scales_with_entry_count() {
    let fixture = fixture();
    let body = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "CountTypes"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked TypePack counter");
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).unwrap();
    let mut prior_steps = None;
    for count in [10usize, 100, 1_000] {
        let pack = Arc::make_mut(&mut types).type_pack(vec![int; count].into_boxed_slice()).unwrap();
        let mut values = StaticValueStore::new();
        let mut budget = QueryBudget::new(100_000);
        let outcome = StaticEvaluator::new(body, &mut values, &mut budget, &CancellationToken::new(), &[StaticArgument::Kinded(pack)])
            .with_semantic_tables_mut(
                Arc::make_mut(&mut types),
                &fixture.analysis.snapshot.data_semantics,
                &fixture.analysis.snapshot.enum_semantics,
            )
            .evaluate();
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("{count} exact TypePack entries must reduce: {outcome:?}");
        };
        assert_eq!(
            values.get(value).unwrap(),
            &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(count as i64))
        );
        if let Some((previous_count, previous_steps)) = prior_steps {
            assert!(
                budget.steps_taken.saturating_sub(previous_steps) <= (count - previous_count) as u64 * 30,
                "Count work should grow linearly: {previous_count}/{previous_steps} versus {count}/{} steps",
                budget.steps_taken
            );
        }
        prior_steps = Some((count, budget.steps_taken));
    }
}

#[test]
fn partially_known_tuple_construction_specializes_to_the_cold_value() {
    let fixture = fixture();
    let pair = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "PairValue"))
        .expect("PairValue definition");
    let body = pair.checked_body.as_ref().expect("checked tuple construction");
    assert!(body.diagnostics.is_empty(), "PairValue must check: {:?}", body.diagnostics);
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "PairValue"),
        index: 0,
        domain: pair.definition.signature.parameters[0].domain,
    };
    let open_parameter = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(open_parameter)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(tuple) = open else {
        panic!("partially known tuple must retain its exact second element: {open:?}");
    };
    assert!(matches!(
        residuals.get(tuple).unwrap(),
        phalcom_semantic::static_eval::StaticResidualData::Tuple { elements, result_type: _ }
            if elements.len() == 2 && elements[0] == phalcom_semantic::static_eval::StaticEvalTerm::Residual(open_parameter)
                && matches!(elements[1], phalcom_semantic::static_eval::StaticEvalTerm::Value(_))
    ));
    assert_eq!(
        specialize_helper(
            &fixture,
            body,
            tuple,
            &StaticSpecializationEnvironmentKey::default(),
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        ),
        StaticEvalOutcome::Symbolic(tuple),
        "specializing without new facts must reuse the canonical tuple"
    );

    let three = values.intern_int(3);
    let cold = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(three)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_value) = cold else {
        panic!("exact tuple must reduce: {cold:?}");
    };
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal, StaticSpecializationArgument::Value(three))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        tuple,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized tuple must reduce: {specialized:?}");
    };
    assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());
}

#[test]
fn partially_known_record_preserves_canonical_keys_and_specializes() {
    let fixture = fixture();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "PartialRecord"))
        .expect("PartialRecord definition");
    let body = product.checked_body.as_ref().expect("checked Record construction");
    assert!(body.diagnostics.is_empty(), "PartialRecord must check: {:?}", body.diagnostics);
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "PartialRecord"),
        index: 0,
        domain: product.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(record) = open else {
        panic!("open Record must remain symbolic: {open:?}");
    };
    let phalcom_semantic::static_eval::StaticResidualData::Record { fields, .. } = residuals.get(record).unwrap() else {
        panic!("direct Record construction must retain its canonical fields: {:?}", residuals.get(record));
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].0, phalcom_semantic::field_key::FieldKey::symbol("same"));
    assert_eq!(fields[1].0, phalcom_semantic::field_key::FieldKey::int(42.into()));
    assert_eq!(fields[0].1, phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown));
    assert!(matches!(fields[1].1, phalcom_semantic::static_eval::StaticEvalTerm::Value(_)));
    assert_eq!(
        specialize_helper(
            &fixture,
            body,
            record,
            &StaticSpecializationEnvironmentKey::default(),
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        ),
        StaticEvalOutcome::Symbolic(record),
        "no-new-facts specialization must preserve the Record's canonical identity"
    );

    let exact = values.intern_int(3);
    let cold = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(exact)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_value) = cold else {
        panic!("cold Record must reduce: {cold:?}");
    };
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal.clone(), StaticSpecializationArgument::Value(exact))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        record,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized Record must reduce: {specialized:?}");
    };
    assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());
}

#[test]
fn partially_known_variant_preserves_constructor_and_specializes() {
    let fixture = fixture();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "PartialVariant"))
        .expect("PartialVariant definition");
    let body = product.checked_body.as_ref().expect("checked Variant construction");
    assert!(body.diagnostics.is_empty(), "PartialVariant must check: {:?}", body.diagnostics);
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "PartialVariant"),
        index: 0,
        domain: product.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(variant) = open else {
        panic!("open Variant must remain symbolic: {open:?}");
    };
    assert!(matches!(
        residuals.get(variant).unwrap(),
        phalcom_semantic::static_eval::StaticResidualData::Variant { variant, fields, .. }
            if variant.owner == fixture.decl("main", "Option")
                && fields.len() == 1 && fields[0].1 == phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown)
    ));
    assert_eq!(
        specialize_helper(
            &fixture,
            body,
            variant,
            &StaticSpecializationEnvironmentKey::default(),
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        ),
        StaticEvalOutcome::Symbolic(variant),
        "no-new-facts specialization must retain canonical Variant identity"
    );
    let exact = values.intern_int(3);
    let cold = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(exact)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_value) = cold else {
        panic!("cold Variant must reduce: {cold:?}");
    };
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal, StaticSpecializationArgument::Value(exact))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        variant,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized Variant must reduce: {specialized:?}");
    };
    assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());
}

#[test]
fn partially_known_data_preserves_components_and_specializes() {
    let fixture = fixture();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "PartialData"))
        .expect("PartialData definition");
    let body = product.checked_body.as_ref().expect("checked Data construction");
    assert!(body.diagnostics.is_empty(), "PartialData must check: {:?}", body.diagnostics);
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "PartialData"),
        index: 0,
        domain: product.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(data) = open else {
        panic!(
            "open Data must remain symbolic: {open:?}; root: {:?}; construction: {:?}",
            body.root, body.static_constructions
        );
    };
    assert!(matches!(
        residuals.get(data).unwrap(),
        phalcom_semantic::static_eval::StaticResidualData::Data { owner, components, .. }
            if owner == &fixture.decl("main", "Boxed") && components.len() == 1
                && components[0].1 == phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown)
    ));
    assert_eq!(
        specialize_helper(
            &fixture,
            body,
            data,
            &StaticSpecializationEnvironmentKey::default(),
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        ),
        StaticEvalOutcome::Symbolic(data),
        "no-new-facts specialization must retain canonical Data identity"
    );
    let exact = values.intern_int(3);
    let cold = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(exact)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_value) = cold else {
        panic!("cold Data must reduce: {cold:?}");
    };
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal, StaticSpecializationArgument::Value(exact))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        data,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized Data must reduce: {specialized:?}");
    };
    assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());
}

#[test]
fn partially_known_pack_preserves_ordered_lanes_and_specializes() {
    let fixture = fixture();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "PairValue"))
        .expect("PairValue definition");
    let body = product.checked_body.as_ref().expect("checked static definition");
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "PairValue"),
        index: 0,
        domain: product.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let exact = values.intern_int(3);
    let seven = values.intern_int(7);
    let named = values.intern_symbol_id("named");
    let other = values.intern_symbol_id("other");
    let cold_value = values
        .intern_pack(vec![exact, seven], vec![(named, seven), (other, exact)])
        .expect("canonical Pack");
    let result_type = values
        .static_value_type(
            cold_value,
            Arc::make_mut(&mut types),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .expect("canonical proper Pack type");
    assert_eq!(types.kind_of(result_type), KindId::TYPE);
    let pack = residuals
        .pack(
            Box::new([
                phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown),
                phalcom_semantic::static_eval::StaticEvalTerm::Value(seven),
            ]),
            Box::new([Box::<str>::from("named"), Box::<str>::from("other")]),
            Box::new([
                phalcom_semantic::static_eval::StaticEvalTerm::Value(seven),
                phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown),
            ]),
            result_type,
            &mut QueryBudget::default(),
        )
        .expect("partially known Pack");
    let phalcom_semantic::static_eval::StaticResidualData::Pack {
        positionals,
        labels,
        labeled_values,
        result_type,
    } = residuals.get(pack).unwrap()
    else {
        panic!("Pack residual must retain its ordered lanes: {:?}", residuals.get(pack));
    };
    assert_eq!(types.kind_of(*result_type), KindId::TYPE);
    assert_eq!(positionals[0], phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown));
    assert_eq!(labels.as_ref(), &[Box::<str>::from("named"), Box::<str>::from("other")]);
    assert_eq!(labeled_values[1], phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown));
    assert_eq!(
        specialize_helper(
            &fixture,
            body,
            pack,
            &StaticSpecializationEnvironmentKey::default(),
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        ),
        StaticEvalOutcome::Symbolic(pack),
        "no-new-facts specialization must preserve the Pack's canonical identity"
    );

    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal, StaticSpecializationArgument::Value(exact))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        pack,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized Pack must reduce: {specialized:?}");
    };
    let StaticValueData::Pack(actual_id) = values.get(actual).unwrap() else {
        panic!("Pack value expected");
    };
    assert_eq!(values.pack_positionals(*actual_id).unwrap(), &[exact, seven]);
    let labeled = values.pack_labeled(*actual_id).unwrap();
    assert_eq!(labeled.len(), 2);
    assert_eq!(values.get_symbol(labeled[0].0), Some("named"));
    assert_eq!(values.get_symbol(labeled[1].0), Some("other"));
    assert_eq!(labeled[0].1, seven);
    assert_eq!(labeled[1].1, exact);
    assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());
}

#[test]
fn checked_pack_construction_preserves_both_lanes_from_source() {
    let fixture = WorkspaceFixture::new()
        .module("main", "type MakePack<N: Int>: Pack<***: Int> { Pack(N, 4, other: 9, named: 7) }\n")
        .analyze();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "MakePack"))
        .expect("MakePack definition");
    let body = product.checked_body.as_ref().expect("checked Pack body");
    assert!(body.diagnostics.is_empty(), "Pack construction must check: {:?}", body.diagnostics);
    assert_eq!(
        body.static_constructions.len(),
        1,
        "the ordinary checker must publish the Pack constructor lanes"
    );
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "MakePack"),
        index: 0,
        domain: product.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(pack) = open else {
        panic!("open source Pack must be symbolic: {open:?}");
    };
    let phalcom_semantic::static_eval::StaticResidualData::Pack {
        positionals,
        labels,
        labeled_values,
        result_type,
    } = residuals.get(pack).unwrap()
    else {
        panic!("checked source Pack must preserve its lanes: {:?}", residuals.get(pack));
    };
    assert_eq!(types.kind_of(*result_type), KindId::TYPE);
    assert_eq!(positionals[0], phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown));
    assert!(matches!(positionals[1], phalcom_semantic::static_eval::StaticEvalTerm::Value(_)));
    assert_eq!(labels.as_ref(), &[Box::<str>::from("other"), Box::<str>::from("named")]);
    assert!(
        labeled_values
            .iter()
            .all(|entry| matches!(entry, phalcom_semantic::static_eval::StaticEvalTerm::Value(_)))
    );
    assert_eq!(
        specialize_helper(
            &fixture,
            body,
            pack,
            &StaticSpecializationEnvironmentKey::default(),
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            Arc::make_mut(&mut types),
            &mut residuals,
        ),
        StaticEvalOutcome::Symbolic(pack),
        "the checked source Pack must retain its identity without new facts"
    );
    let exact = values.intern_int(3);
    let cold = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(exact)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_value) = cold else {
        panic!("cold Pack must reduce: {cold:?}");
    };
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal.clone(), StaticSpecializationArgument::Value(exact))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        pack,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized Pack must reduce: {specialized:?}");
    };
    let StaticValueData::Pack(id) = values.get(actual).unwrap() else {
        panic!("Pack value expected");
    };
    let id = *id;
    let four = values.intern_int(4);
    assert_eq!(values.pack_positionals(id).unwrap(), &[exact, four]);
    let labeled = values.pack_labeled(id).unwrap();
    assert_eq!(labeled.len(), 2);
    assert_eq!(values.get_symbol(labeled[0].0), Some("other"));
    assert_eq!(values.get_symbol(labeled[1].0), Some("named"));
    assert_eq!(values.get(actual).unwrap(), values.get(cold_value).unwrap());

    let mut fresh_residuals = StaticResidualStore::new();
    let fresh_unknown = fresh_residuals.parameter(formal, &mut QueryBudget::default()).unwrap();
    let mut exhausted = QueryBudget { max_residual_nodes: 0, ..Default::default() };
    let stopped = StaticEvaluator::new(
        body,
        &mut values,
        &mut exhausted,
        &CancellationToken::new(),
        &[StaticArgument::Residual(fresh_unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .with_residuals(&mut fresh_residuals)
    .evaluate();
    assert!(
        matches!(&stopped, StaticEvalOutcome::BudgetExceeded(report)
        if report.kind() == phalcom_semantic::types::outcome::BudgetKind::ResidualNodes),
        "source Pack residual construction must respect the residual-node budget: {stopped:?}"
    );
}

#[test]
fn contextual_pack_construction_rejects_mismatched_element_types() {
    let fixture = WorkspaceFixture::new()
        .module("main", "type WrongPack: Pack<***: String> { Pack(1, named: 2) }\n")
        .analyze();
    let body = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "WrongPack"))
        .and_then(|product| product.checked_body.as_ref())
        .expect("checked wrong Pack body");
    assert!(
        body.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::TypeMismatch),
        "the completed Pack must be checked against its declared element type: {:?}",
        body.diagnostics
    );
    assert!(
        body.static_constructions.is_empty(),
        "an invalid Pack must not publish a checked static constructor"
    );
}

#[test]
fn source_pack_spread_uses_checked_constructor_lanes() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            "type WrapPack: Pack<***: Int> { Pack(*Pack(1, 2)) }\n\
             type CombinedPack<N: Int>: Pack<***: Int> { Pack(0, *Pack(1, 2), N, a: 3, **Pack(b: 4)) }\n",
        )
        .analyze();
    let wrap = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "WrapPack"))
        .expect("WrapPack definition");
    let wrap_body = wrap.checked_body.as_ref().expect("checked WrapPack body");
    assert!(wrap_body.diagnostics.is_empty(), "WrapPack must check: {:?}", wrap_body.diagnostics);
    assert_eq!(
        wrap_body.static_constructions.len(),
        2,
        "both inner and outer Pack constructions must be published"
    );

    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let cold_wrap = StaticEvaluator::new(wrap_body, &mut values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
        .with_semantic_tables_mut(
            Arc::make_mut(&mut types),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .evaluate();
    let StaticEvalOutcome::Reduced(wrap_val) = cold_wrap else {
        panic!("cold WrapPack must reduce: {cold_wrap:?}");
    };
    let StaticValueData::Pack(wrap_id) = values.get(wrap_val).unwrap() else {
        panic!("Pack value expected");
    };
    let pos = values.pack_positionals(*wrap_id).unwrap().to_vec();
    assert_eq!(pos.len(), 2);
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    assert_eq!(pos.as_slice(), &[one, two]);

    let combined = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "CombinedPack"))
        .expect("CombinedPack definition");
    let combined_body = combined.checked_body.as_ref().expect("checked CombinedPack body");
    assert!(combined_body.diagnostics.is_empty(), "CombinedPack must check: {:?}", combined_body.diagnostics);
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "CombinedPack"),
        index: 0,
        domain: combined.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        combined_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(pack_res) = open else {
        panic!("open CombinedPack must be symbolic: {open:?}");
    };
    let phalcom_semantic::static_eval::StaticResidualData::Pack {
        positionals,
        labels,
        labeled_values,
        ..
    } = residuals.get(pack_res).unwrap()
    else {
        panic!("expected Pack residual: {:?}", residuals.get(pack_res));
    };
    assert_eq!(positionals.len(), 4);
    assert_eq!(positionals[3], phalcom_semantic::static_eval::StaticEvalTerm::Residual(unknown));
    assert_eq!(labels.as_ref(), &[Box::<str>::from("a"), Box::<str>::from("b")]);
    assert_eq!(labeled_values.len(), 2);

    let ninety_nine = values.intern_int(99);
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal, StaticSpecializationArgument::Value(ninety_nine))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        combined_body,
        pack_res,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized CombinedPack must reduce: {specialized:?}");
    };
    let cold_combined = StaticEvaluator::new(
        combined_body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(ninety_nine)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
    .evaluate();
    let StaticEvalOutcome::Reduced(cold_val) = cold_combined else {
        panic!("cold CombinedPack must reduce: {cold_combined:?}");
    };
    assert_eq!(values.get(actual).unwrap(), values.get(cold_val).unwrap());
    let StaticValueData::Pack(comb_id) = values.get(actual).unwrap() else {
        panic!("Pack value expected");
    };
    let comb_pos = values.pack_positionals(*comb_id).unwrap().to_vec();
    let zero = values.intern_int(0);
    assert_eq!(comb_pos.as_slice(), &[zero, one, two, ninety_nine]);
}

#[test]
fn source_pack_spread_concat_and_prepend() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            "type PrependCold: Pack<***: Int> { Pack(0, *Pack(1, 2)) }\n\
             type ConcatCold: Pack<***: Int> { Pack(*Pack(1, 2), *Pack(3, 4)) }\n",
        )
        .analyze();
    let test_cold = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "ConcatCold"))
        .expect("ConcatCold definition");
    let test_body = test_cold.checked_body.as_ref().expect("checked ConcatCold body");
    assert!(test_body.diagnostics.is_empty(), "ConcatCold must check: {:?}", test_body.diagnostics);

    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let cold = StaticEvaluator::new(test_body, &mut values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .with_semantic_tables_mut(
            Arc::make_mut(&mut types),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .evaluate();
    let StaticEvalOutcome::Reduced(res_val) = cold else {
        panic!("cold ConcatCold must reduce: {cold:?}");
    };
    let StaticValueData::Pack(res_id) = values.get(res_val).unwrap() else {
        panic!("Pack value expected");
    };
    let res_pos = values.pack_positionals(*res_id).unwrap().to_vec();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let three = values.intern_int(3);
    let four = values.intern_int(4);
    assert_eq!(res_pos.as_slice(), &[one, two, three, four]);

    let prepend_def = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "PrependCold"))
        .expect("PrependCold definition");
    let prepend_body = prepend_def.checked_body.as_ref().expect("checked PrependCold body");
    assert!(prepend_body.diagnostics.is_empty(), "PrependCold must check: {:?}", prepend_body.diagnostics);
    let prepend_cold = StaticEvaluator::new(prepend_body, &mut values, &mut QueryBudget::default(), &CancellationToken::new(), &[])
        .with_static_definitions(fixture.analysis.snapshot.static_definitions())
        .with_semantic_tables_mut(
            Arc::make_mut(&mut types),
            &fixture.analysis.snapshot.data_semantics,
            &fixture.analysis.snapshot.enum_semantics,
        )
        .with_hierarchy(fixture.analysis.snapshot.hierarchy().as_ref())
        .evaluate();
    let StaticEvalOutcome::Reduced(prep_val) = prepend_cold else {
        panic!("cold PrependCold must reduce: {prepend_cold:?}");
    };
    let StaticValueData::Pack(prep_id) = values.get(prep_val).unwrap() else {
        panic!("Pack value expected");
    };
    let prep_pos = values.pack_positionals(*prep_id).unwrap().to_vec();
    let zero = values.intern_int(0);
    assert_eq!(prep_pos.as_slice(), &[zero, one, two]);
}

#[test]
fn iterative_source_pack_count_specializes_open_value_to_cold_result() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            "type CountPack<P: Pack<***: Int>>: Int { let mut count = 0\n for _ in P { count = count + 1 }\n count }\n",
        )
        .analyze();
    let product = fixture
        .analysis
        .snapshot
        .static_definition(&fixture.decl("main", "CountPack"))
        .expect("CountPack definition");
    let body = product.checked_body.as_ref().expect("checked Pack counter");
    assert!(body.diagnostics.is_empty(), "Pack counter must check: {:?}", body.diagnostics);
    assert!(
        body.static_iteration_lanes.values().any(|lane| matches!(
            lane.item_domain,
            phalcom_semantic::checker::static_iteration::StaticIterationItemDomain::Uniform(_)
        )),
        "an open Pack remainder must not publish a finite empty item domain"
    );
    assert!(
        !body.pattern_site_resolutions.is_empty(),
        "the loop must retain the ordinary checked pattern site"
    );
    let mut types = Arc::clone(&fixture.analysis.snapshot.store);
    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let named = values.intern_symbol_id("named");
    let pack = values.intern_pack(vec![one, two], vec![(named, one)]).unwrap();
    let cold = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Value(pack)],
    )
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .evaluate();
    let StaticEvalOutcome::Reduced(exact) = cold else {
        panic!("finite Pack iteration must reduce: {cold:?}");
    };
    assert_eq!(
        values.get(exact).unwrap(),
        &StaticValueData::Int(phalcom_semantic::static_value::StaticInt::Small(3))
    );
    let mut residuals = StaticResidualStore::new();
    let formal = StaticResidualParameter {
        owner: fixture.decl("main", "CountPack"),
        index: 0,
        domain: product.definition.signature.parameters[0].domain,
    };
    let unknown = residuals.parameter(formal.clone(), &mut QueryBudget::default()).unwrap();
    let open = StaticEvaluator::new(
        body,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        &[StaticArgument::Residual(unknown)],
    )
    .with_static_definitions(fixture.analysis.snapshot.static_definitions())
    .with_semantic_tables_mut(
        Arc::make_mut(&mut types),
        &fixture.analysis.snapshot.data_semantics,
        &fixture.analysis.snapshot.enum_semantics,
    )
    .with_residuals(&mut residuals)
    .evaluate();
    let StaticEvalOutcome::Symbolic(deferred) = open else {
        panic!("open Pack loop must defer as an application: {open:?}");
    };
    assert!(matches!(
        residuals.get(deferred).unwrap(),
        phalcom_semantic::static_eval::StaticResidualData::Apply { .. }
    ));
    let supplied = StaticSpecializationEnvironmentKey::normalized([(formal, StaticSpecializationArgument::Value(pack))], [], []).unwrap();
    let specialized = specialize_helper(
        &fixture,
        body,
        deferred,
        &supplied,
        &mut values,
        &mut QueryBudget::default(),
        &CancellationToken::new(),
        Arc::make_mut(&mut types),
        &mut residuals,
    );
    let StaticEvalOutcome::Reduced(actual) = specialized else {
        panic!("specialized Pack counter must reduce: {specialized:?}");
    };
    assert_eq!(values.get(actual).unwrap(), values.get(exact).unwrap());
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
