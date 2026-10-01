//! Adversarial subtyping, union obligation, and bound verification probes.

use phalcom_common::selector::Selector;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::identity::{DispatchSide, VariantId};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::parameter::{SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, check_assignability, check_subtype_bounded, is_subtype};
use phalcom_semantic::types::row::StructuralMemberPresence;
use phalcom_semantic::types::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeStore};

fn decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn repeated_union_obligation_is_not_recursion() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Int"), decl("Number"));
    let i = s.nominal(decl("Int"));
    let n = s.nominal(decl("Number"));
    let st = s.nominal(decl("String"));
    let u = s.union(&[n, st]);
    let tuple = |a, b| Box::new([TupleTypeElement::unlabeled(a), TupleTypeElement::unlabeled(b)]);
    let actual = s.tuple(tuple(i, i));
    let expected = s.tuple(tuple(u, u));
    assert!(is_subtype(&mut s, &h, actual, expected), "independent equal obligations must both succeed");
}

#[test]
fn applied_subclass_retains_nongeneric_superclass() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Child"), decl("Base"));
    let kind = s.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let child = s.nominal_form(decl("Child"), kind);
    let i = s.nominal(decl("Int"));
    let applied = s.apply_type_form(child, &[i]).unwrap();
    let base = s.nominal(decl("Base"));
    assert!(is_subtype(&mut s, &h, applied, base));
}

#[test]
fn applied_subclass_projects_transitive_and_generic_ancestors() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    let base = decl("Base");
    let middle = decl("Middle");
    let child = decl("Child");
    h.insert(child.clone(), middle.clone());
    h.insert(middle.clone(), base.clone());

    let kind = s.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let child_form = s.nominal_form(child, kind);
    let int = s.nominal(decl("Int"));
    let string = s.nominal(decl("String"));
    let applied_child = s.apply_type_form(child_form, &[int]).unwrap();
    let base_nominal = s.nominal(base);
    let unrelated = s.nominal(decl("Unrelated"));

    assert!(
        is_subtype(&mut s, &h, applied_child, base_nominal),
        "multi-hop nongeneric ancestor must be retained"
    );
    assert!(
        !is_subtype(&mut s, &h, applied_child, unrelated),
        "ancestry must not prove unrelated nominal targets"
    );

    let generic_base = decl("GenericBase");
    let generic_middle = decl("GenericMiddle");
    let generic_child = decl("GenericChild");
    let generic_kind = s.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let base_form = s.nominal_form(generic_base.clone(), generic_kind);
    let middle_form = s.nominal_form(generic_middle.clone(), generic_kind);
    let child_form = s.nominal_form(generic_child.clone(), generic_kind);
    let base_parameter = s.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(generic_base.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let middle_parameter = s.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(generic_middle.clone()),
        0,
        "U",
        KindId::TYPE,
    ));
    let child_parameter = s.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(generic_child.clone()),
        0,
        "V",
        KindId::TYPE,
    ));
    let middle_parameter_form = s.parameter_form(middle_parameter);
    let child_parameter_form = s.parameter_form(child_parameter);
    let base_of_middle = s.apply_type_form(base_form, &[middle_parameter_form]).unwrap();
    let middle_of_child = s.apply_type_form(middle_form, &[child_parameter_form]).unwrap();
    h.insert_template(phalcom_semantic::declarations::GenericSupertypeTemplate {
        declaration: generic_middle.clone(),
        supertype: base_of_middle,
        structural_form: None,
    });
    h.insert_template(phalcom_semantic::declarations::GenericSupertypeTemplate {
        declaration: generic_child.clone(),
        supertype: middle_of_child,
        structural_form: None,
    });

    let applied_generic_child = s.apply_type_form(child_form, &[int]).unwrap();
    let base_int = s.apply_type_form(base_form, &[int]).unwrap();
    let base_string = s.apply_type_form(base_form, &[string]).unwrap();
    assert!(
        is_subtype(&mut s, &h, applied_generic_child, base_int),
        "generic ancestry must preserve owner-aware substitution"
    );
    assert!(
        !is_subtype(&mut s, &h, applied_generic_child, base_string),
        "invariant generic ancestor arguments must remain distinct"
    );
    let _ = base_parameter;
}

#[test]
fn recursive_relation_charges_pair_budget() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Int"), decl("Number"));
    let i = s.nominal(decl("Int"));
    let n = s.nominal(decl("Number"));
    let a = s.tuple(Box::new([TupleTypeElement::unlabeled(i)]));
    let b = s.tuple(Box::new([TupleTypeElement::unlabeled(n)]));
    let mut budget = QueryBudget {
        max_relation_pairs: 1,
        ..QueryBudget::default()
    };
    let outcome = check_subtype_bounded(&mut s, &h, a, b, &mut budget, &CancellationToken::new());
    assert!(outcome.is_budget_exceeded(), "two distinct pairs exceed limit one: {outcome:?}, {budget:?}");
}

#[test]
fn recursive_relation_charges_pair_budget_at_exact_limit() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Int"), decl("Number"));
    let i = s.nominal(decl("Int"));
    let n = s.nominal(decl("Number"));
    let a = s.tuple(Box::new([TupleTypeElement::unlabeled(i)]));
    let b = s.tuple(Box::new([TupleTypeElement::unlabeled(n)]));
    let mut budget = QueryBudget {
        max_relation_pairs: 1,
        ..QueryBudget::default()
    };
    let outcome = check_subtype_bounded(&mut s, &h, a, b, &mut budget, &CancellationToken::new());
    assert!(
        outcome.is_budget_exceeded(),
        "the tuple and nested nominal relation require two obligations: {outcome:?}, {budget:?}"
    );

    let mut budget = QueryBudget {
        max_relation_pairs: 2,
        ..QueryBudget::default()
    };
    let outcome = check_subtype_bounded(&mut s, &h, a, b, &mut budget, &CancellationToken::new());
    assert!(outcome.is_proven(), "exact relation-pair budget must be accepted: {outcome:?}, {budget:?}");
}

#[test]
fn recursive_relation_enforces_type_depth_budget() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Int"), decl("Number"));
    let i = s.nominal(decl("Int"));
    let n = s.nominal(decl("Number"));
    let inner_actual = s.tuple(Box::new([TupleTypeElement::unlabeled(i)]));
    let inner_expected = s.tuple(Box::new([TupleTypeElement::unlabeled(n)]));
    let actual = s.tuple(Box::new([TupleTypeElement::unlabeled(inner_actual)]));
    let expected = s.tuple(Box::new([TupleTypeElement::unlabeled(inner_expected)]));

    let mut shallow = QueryBudget {
        max_type_depth: 1,
        ..QueryBudget::default()
    };
    let exhausted = check_subtype_bounded(&mut s, &h, actual, expected, &mut shallow, &CancellationToken::new());
    assert!(
        matches!(exhausted, phalcom_semantic::types::outcome::RelationOutcome::BudgetExceeded(ref report) if report.kind == phalcom_semantic::types::outcome::BudgetKind::TypeDepth),
        "nested relation must report type-depth exhaustion: {exhausted:?}"
    );

    let mut sufficient = QueryBudget {
        max_type_depth: 2,
        ..QueryBudget::default()
    };
    let proven = check_subtype_bounded(&mut s, &h, actual, expected, &mut sufficient, &CancellationToken::new());
    assert!(proven.is_proven(), "depth at the deepest nested relation must be accepted: {proven:?}");
}

#[test]
fn union_candidate_allocation_order_is_irrelevant() {
    fn probe(good_first: bool) -> bool {
        let mut s = TypeStore::new();
        let h = MapTypeHierarchy::new();
        let i = s.nominal(decl("Int"));
        let st = s.nominal(decl("String"));
        let u = s.union(&[i, st]);
        let tuple = |a, b| Box::new([TupleTypeElement::unlabeled(a), TupleTypeElement::unlabeled(b)]);
        let actual = s.tuple(tuple(i, i));
        let (first, second) = if good_first { (i, st) } else { (st, i) };
        let a = s.tuple(tuple(u, first));
        let b = s.tuple(tuple(u, second));
        let expected = s.union(&[a, b]);
        is_subtype(&mut s, &h, actual, expected)
    }
    let bad_first = probe(false);
    let good_first = probe(true);
    assert_eq!(bad_first, good_first);
    assert!(good_first);
}

#[test]
fn canonicalization_binder_rows_and_application_controls() {
    let mut s = TypeStore::new();
    let i = s.nominal(decl("Int"));
    let st = s.nominal(decl("String"));
    let kind = s.arrow_kind(Box::new([KindId::TYPE, KindId::TYPE]), KindId::TYPE);
    let ctor = s.nominal_form(decl("Pair"), kind);
    let whole = s.apply_type_form(ctor, &[i, st]).unwrap();
    let partial = s.apply_type_form(ctor, &[i]).unwrap();
    assert_eq!(s.apply_type_form(partial, &[st]).unwrap(), whole);
    assert_eq!(s.apply_type_form(ctor, &[i, st]).unwrap(), whole);
    assert_ne!(s.apply_type_form(ctor, &[st, i]).unwrap(), whole);
    let p = s.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl("A")), 0, "T", KindId::TYPE));
    let q = s.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl("B")), 0, "T", KindId::TYPE));
    assert_ne!(p, q);
    let x = RecordTypeField { name: "x".into(), ty: i, presence: StructuralMemberPresence::Required };
    let y = RecordTypeField { name: "y".into(), ty: st, presence: StructuralMemberPresence::Required };
    let xy = s.record(Box::new([x.clone(), y.clone()]));
    assert_eq!(s.record(Box::new([y, x])), xy);
}

#[test]
fn representative_relation_matrix() {
    let mut s = TypeStore::new();
    let h = MapTypeHierarchy::new();
    let i = s.nominal(decl("Int"));
    let st = s.nominal(decl("String"));
    let object = s.nominal(decl("Object"));
    let forms = [("Int", i), ("String", st), ("Unit", s.unit()), ("Never", s.never()), ("Object", object)];
    let mut knowledge = forms
        .iter()
        .map(|(name, ty)| (*name, TypeKnowledge::established(*ty, EvidenceOrigin::Syntax)))
        .collect::<Vec<_>>();
    knowledge.push(("Dynamic", TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)));
    knowledge.push(("Unknown", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)));
    for (_name, actual) in &knowledge {
        let row = knowledge
            .iter()
            .map(|(_, expected)| {
                let result = check_assignability(&mut s, &h, actual, expected);
                if result.is_assignable() {
                    "P"
                } else if result.is_refuted() {
                    "R"
                } else if matches!(result, phalcom_semantic::types::relation::Assignability::DynamicBoundary(_)) {
                    "D"
                } else {
                    "B"
                }
            })
            .collect::<Vec<_>>();
        assert!(!row.is_empty());
    }
}

#[test]
fn extended_type_relation_controls() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Int"), decl("Number"));
    let i = s.nominal(decl("Int"));
    let n = s.nominal(decl("Number"));
    let st = s.nominal(decl("String"));
    let obj = s.nominal(decl("Object"));
    let owner = decl("Owner");
    let self_ty = s.self_type(SelfTypeTerm {
        owner: owner.clone(),
        side: DispatchSide::Instance,
        role: SelfRole::ReceiverValue,
    });
    let p = s.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner), 0, "T", KindId::TYPE));
    let t = s.parameter_form(p);
    let kind = s.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list = s.nominal_form(decl("List"), kind);
    let lt = s.apply_type_form(list, &[t]).unwrap();
    let li = s.apply_type_form(list, &[i]).unwrap();
    let ln = s.apply_type_form(list, &[n]).unwrap();
    let x = RecordTypeField { name: "x".into(), ty: i, presence: StructuralMemberPresence::Required };
    let r1 = s.record(Box::new([x.clone()]));
    let r2 = s.record(Box::new([x, RecordTypeField { name: "y".into(), ty: st, presence: StructuralMemberPresence::Required }]));
    let c1 = s.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(n)]),
        return_type: i,
    });
    let c2 = s.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(i)]),
        return_type: n,
    });
    let en = s.nominal(decl("Enum"));
    let variant = VariantId::new(decl("Enum"), Selector::method("Case", []).unwrap());
    let exact = s.exact_case_type(&variant, en).unwrap();
    for (label, a, b, expected) in [
        ("Self <: Self", self_ty, self_ty, true),
        ("Self <: Int", self_ty, i, false),
        ("T <: T", t, t, true),
        ("T <: Int", t, i, false),
        ("T <: Object", t, obj, false),
        ("List<T> <: List<T>", lt, lt, true),
        ("List<Int> <: List<Number>", li, ln, false),
        ("record xy <: record x", r2, r1, false),
        ("record x <: record xy", r1, r2, false),
        ("Number->Int <: Int->Number", c1, c2, true),
        ("ExactCase <: Enum", exact, en, true),
        ("Enum <: ExactCase", en, exact, false),
    ] {
        let result = is_subtype(&mut s, &h, a, b);
        assert_eq!(result, expected, "{label}");
    }
}
