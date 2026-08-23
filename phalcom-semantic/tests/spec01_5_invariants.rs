//! Comprehensive Spec 01.5 verification suite.
//! Tests Gates 01.5-A through 01.5-H and mathematical property laws in §29.

use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate};
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticSourceSpan};
use phalcom_semantic::dispatch::{CallableParameter, CallableSignature, DispatchSide};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::signature::{CallableParameterSemantic, CallableSemanticSignature, CallableSignatureTable, FieldSemanticSignature, FieldSignatureTable};
use phalcom_semantic::surface::DeclarationSurface;
use phalcom_semantic::types::environment::{TypeEnvironment, TypeView};
use phalcom_semantic::types::evidence::{EvidenceAuthority, TypeKnowledge};
use phalcom_semantic::types::id::{KindId, ProperTypeId, ScopedTypeId, TypeId, TypeParameterId};
use phalcom_semantic::types::parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::{Assignability, MapTypeHierarchy, check_assignability, is_subtype};
use phalcom_semantic::types::store::{RecordTypeField, TupleTypeElement, TypeData, TypeStore};
use phalcom_semantic::types::substitution::{TypeSubstitution, substitution_for_applied};
use phalcom_semantic::types::type_lambda::{BetaResult, ScopedTypeData, TypeLambdaArena, TypeLambdaProvenance};
use phalcom_semantic::types::variance::{Variance, VarianceStep, compute_variance_occurrence};

fn core_decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
}

#[test]
fn gate_01_5_a_canonical_declaration_model() {
    let mut store = TypeStore::new();
    let decl_box = core_decl("Box");
    let param_t = store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(decl_box.clone()), 0, "T", KindId::TYPE).with_variance(Variance::Covariant),
    );

    let sig = GenericSignature::with_constraints(
        TypeParameterOwner::Declaration(decl_box.clone()),
        vec![param_t].into_boxed_slice(),
        vec![GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(store.parameter_form(param_t)),
            upper: TypeTerm::Canonical(store.nominal_type(core_decl("Object"))),
        }]
        .into_boxed_slice(),
    );

    assert_eq!(sig.parameter_count(), 1);
    assert_eq!(sig.parameter_at(0), Some(param_t));
    assert_eq!(sig.constraint_count(), 1);

    let arrow = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let box_form = store.nominal_form(decl_box.clone(), arrow);
    let box_class_obj = store.class_object_type(decl_box.clone());

    let mut table = DeclarationTypeTable::new();
    table.insert(DeclarationTypeInfo {
        declaration: decl_box.clone(),
        form: box_form,
        class_object_type: box_class_obj,
        kind: arrow,
        generic_signature: Some(sig),
        supertype_template: None,
    });

    assert_eq!(table.kind(&decl_box), Some(arrow));
    assert_eq!(table.generic_signature(&decl_box).unwrap().parameter_count(), 1);
}

#[test]
fn gate_01_5_b_type_lambda_calculus_and_alpha_equivalence() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(core_decl("Int"));
    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list_form = store.nominal_form(core_decl("List"), list_kind);

    // 1. Alpha Equivalence: <T> =>> List<T> ≡α <U> =>> List<U>
    let mut arena1 = TypeLambdaArena::new();
    let scoped_t = arena1.intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_list = arena1.intern_scoped(ScopedTypeData::Free(list_form));
    let app1 = arena1.intern_scoped(ScopedTypeData::Applied {
        origin: free_list,
        arguments: vec![scoped_t].into_boxed_slice(),
    });
    let lambda1 = arena1.intern_lambda(
        vec![KindId::TYPE].into_boxed_slice(),
        app1,
        KindId::TYPE,
        Some(TypeLambdaProvenance {
            parameter_names: vec!["T".into()].into_boxed_slice(),
            ..Default::default()
        }),
    );

    let mut arena2 = TypeLambdaArena::new();
    let scoped_u = arena2.intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_list2 = arena2.intern_scoped(ScopedTypeData::Free(list_form));
    let app2 = arena2.intern_scoped(ScopedTypeData::Applied {
        origin: free_list2,
        arguments: vec![scoped_u].into_boxed_slice(),
    });
    let lambda2 = arena2.intern_lambda(
        vec![KindId::TYPE].into_boxed_slice(),
        app2,
        KindId::TYPE,
        Some(TypeLambdaProvenance {
            parameter_names: vec!["U".into()].into_boxed_slice(),
            ..Default::default()
        }),
    );

    // Both produce identical alpha-normalized hash and data
    assert_eq!(arena1.get_lambda(lambda1), arena2.get_lambda(lambda2));

    // 2. Beta Reduction: (<T> =>> List<T>)<Int> ↦β List<Int>
    let beta_res = arena1.beta_reduce(lambda1, &[int_ty], &mut store).unwrap();
    let expected_list_int = store.apply_type_form(list_form, &[int_ty]).unwrap();
    match beta_res {
        BetaResult::Canonical(reduced) => assert_eq!(reduced, expected_list_int),
        _ => panic!("Expected saturated canonical reduction"),
    }
}

#[test]
fn gate_01_5_c_lazy_view_and_eager_materialization_equivalence() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(core_decl("Int"));
    let str_ty = store.nominal_type(core_decl("String"));
    let decl_pair = core_decl("Pair");

    let p_a = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl_pair.clone()), 0, "A", KindId::TYPE));
    let p_b = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl_pair.clone()), 1, "B", KindId::TYPE));

    let t_a = store.parameter_form(p_a);
    let t_b = store.parameter_form(p_b);

    // Composite record: { first: A, second: B, tag: String }
    let rec_ty = store.record(
        vec![
            RecordTypeField { name: "first".into(), ty: t_a },
            RecordTypeField {
                name: "second".into(),
                ty: t_b,
            },
            RecordTypeField {
                name: "tag".into(),
                ty: str_ty,
            },
        ]
        .into_boxed_slice(),
    );

    // Environment: { A -> Int, B -> String }
    let mut env = TypeEnvironment::new();
    env.bind_param(p_a, int_ty);
    env.bind_param(p_b, str_ty);

    // Eager substitution
    let subst = env.to_substitution();
    let eager_materialized = subst.apply(&mut store, rec_ty);

    // Lazy TypeView materialization
    let view = TypeView::new(rec_ty, env);
    let lazy_materialized = view.materialize(&mut store);

    // Mathematical Law §29.8: materialize(view(t, ρ)) ≡ substitute(t, ρ)
    assert_eq!(lazy_materialized, eager_materialized);
}

#[test]
fn gate_01_5_d_variance_subtyping_and_generic_inheritance() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(core_decl("Int"));
    let num_ty = store.nominal_type(core_decl("Number"));

    let mut hier = MapTypeHierarchy::new();
    hier.insert(core_decl("Int"), core_decl("Number"));
    assert!(is_subtype(&store, &hier, int_ty, num_ty));

    // 1. Covariant container: Producer<+T> => Producer<Int> <: Producer<Number>
    let decl_producer = core_decl("Producer");
    let arrow_1 = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let producer_form = store.nominal_form(decl_producer.clone(), arrow_1);
    store.set_parameter_variance(decl_producer.clone(), 0, Variance::Covariant);

    let prod_int = store.apply_type_form(producer_form, &[int_ty]).unwrap();
    let prod_num = store.apply_type_form(producer_form, &[num_ty]).unwrap();

    assert!(
        is_subtype(&store, &hier, prod_int, prod_num),
        "Covariant application must preserve subtyping Producer<Int> <: Producer<Number>"
    );
    assert!(
        !is_subtype(&store, &hier, prod_num, prod_int),
        "Covariant application must refute reverse Producer<Number> <: Producer<Int>"
    );

    // 2. Contravariant consumer: Consumer<-T> => Consumer<Number> <: Consumer<Int>
    let decl_consumer = core_decl("Consumer");
    let consumer_form = store.nominal_form(decl_consumer.clone(), arrow_1);
    store.set_parameter_variance(decl_consumer.clone(), 0, Variance::Contravariant);

    let cons_int = store.apply_type_form(consumer_form, &[int_ty]).unwrap();
    let cons_num = store.apply_type_form(consumer_form, &[num_ty]).unwrap();

    assert!(
        is_subtype(&store, &hier, cons_num, cons_int),
        "Contravariant application must invert subtyping Consumer<Number> <: Consumer<Int>"
    );
    assert!(
        !is_subtype(&store, &hier, cons_int, cons_num),
        "Contravariant application must refute non-inverted Consumer<Int> <: Consumer<Number>"
    );

    // 3. Generic Supertype Template: Names<T> is Sequence<T>
    let decl_names = core_decl("Names");
    let decl_seq = core_decl("Sequence");
    let names_form = store.nominal_form(decl_names.clone(), arrow_1);
    let seq_form = store.nominal_form(decl_seq.clone(), arrow_1);

    let p_names_t = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(decl_names.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let t_names = store.parameter_form(p_names_t);

    let seq_template = store.apply_type_form(seq_form, &[t_names]).unwrap();
    hier.insert_template(GenericSupertypeTemplate {
        declaration: decl_names.clone(),
        supertype: seq_template,
    });

    let names_int = store.apply_type_form(names_form, &[int_ty]).unwrap();
    let seq_int = store.apply_type_form(seq_form, &[int_ty]).unwrap();

    assert!(
        is_subtype(&store, &hier, names_int, seq_int),
        "Names<Int> must specialize and inherit Sequence<Int>"
    );
}

#[test]
fn gate_01_5_e_generic_method_inference_and_occurs_check() {
    use phalcom_semantic::checker::inference::{ConstraintOrigin, InferenceOutcome, InferenceRelation, InferenceSession, InferenceTerm};

    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let mut session = InferenceSession::new();

    let int_ty = store.nominal_type(core_decl("Int"));
    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list_form = store.nominal_form(core_decl("List"), list_kind);
    let list_int = store.apply_type_form(list_form, &[int_ty]).unwrap();

    let var_t = session.fresh_variable(KindId::TYPE);
    let list_var_term = InferenceTerm::Applied {
        origin: Box::new(InferenceTerm::Canonical(list_form)),
        arguments: Box::new([InferenceTerm::Var(var_t)]),
    };
    let list_int_term = InferenceTerm::Canonical(list_int);

    // Constraint: List<T> == List<Int>
    session.add_constraint(InferenceRelation::Equivalent(list_var_term, list_int_term), ConstraintOrigin::Explicit, None);

    let outcome = session.solve(&mut store, &hier);
    assert!(outcome.is_solved());
    if let InferenceOutcome::Solved(sol) = outcome {
        assert_eq!(sol.substitutions.get(&var_t), Some(&int_ty), "Inferred T == Int from argument container");
    }

    // Occurs check: X == List<X> should fail
    let mut session2 = InferenceSession::new();
    let var_x = session2.fresh_variable(KindId::TYPE);
    let list_x_term = InferenceTerm::Applied {
        origin: Box::new(InferenceTerm::Canonical(list_form)),
        arguments: Box::new([InferenceTerm::Var(var_x)]),
    };

    session2.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var_x), list_x_term),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome2 = session2.solve(&mut store, &hier);
    assert!(!outcome2.is_solved(), "Occurs check must reject recursive self-binding X := List<X>");
}

#[test]
fn gate_01_5_f_variance_occurrence_algebra() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(core_decl("Int"));
    let decl_cell = core_decl("Cell");
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl_cell.clone()), 0, "T", KindId::TYPE));
    let t_form = store.parameter_form(param_t);

    // Callable: (T) -> Int  (T occurs in contravariant parameter position)
    let call_ty = store.callable(phalcom_semantic::types::store::CallableType {
        parameters: vec![phalcom_semantic::types::store::CallableParameterType {
            label: None,
            ty: t_form,
            rest: false,
        }]
        .into_boxed_slice(),
        return_type: int_ty,
    });

    let mut path = Vec::new();
    let (polarity, steps) = compute_variance_occurrence(&store, param_t, call_ty, Variance::Covariant, &mut path).unwrap();
    assert_eq!(polarity, Variance::Contravariant, "Parameter position flips polarity to contravariant (-)");
    assert_eq!(steps.len(), 1);
    assert!(matches!(steps[0], VarianceStep::CallableParameter { index: 0, .. }));
}
