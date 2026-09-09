//! Adversarial audit probes. See docs/audit/type-system-2026-09-09/REPORT.md.
use crate::semantic::support::Fixture;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::relation::{MapTypeHierarchy, check_subtype_bounded, is_subtype};
use phalcom_semantic::types::store::{TupleTypeElement, TypeStore};

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
    let tuple = |a, b| Box::new([TupleTypeElement { label: None, ty: a }, TupleTypeElement { label: None, ty: b }]);
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
fn recursive_relation_charges_pair_budget() {
    let mut s = TypeStore::new();
    let mut h = MapTypeHierarchy::new();
    h.insert(decl("Int"), decl("Number"));
    let i = s.nominal(decl("Int"));
    let n = s.nominal(decl("Number"));
    let a = s.tuple(Box::new([TupleTypeElement { label: None, ty: i }]));
    let b = s.tuple(Box::new([TupleTypeElement { label: None, ty: n }]));
    let mut budget = QueryBudget {
        max_relation_pairs: 1,
        ..QueryBudget::default()
    };
    let outcome = check_subtype_bounded(&mut s, &h, a, b, &mut budget, &CancellationToken::new());
    assert!(outcome.is_budget_exceeded(), "two distinct pairs exceed limit one: {outcome:?}, {budget:?}");
}

#[test]
fn incompatible_override_must_not_certify_int() {
    let f = Fixture::new(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/audit/type-system-2026-09-09/override-return.ph"
    )));
    assert!(
        f.analysis.snapshot.has_errors(),
        "an incompatible String override must not satisfy a Base Int contract"
    );
}

#[test]
fn captured_write_must_invalidate_previous_int_fact() {
    let f = Fixture::new(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/audit/type-system-2026-09-09/captured-write.ph"
    )));
    assert!(
        f.analysis.snapshot.has_errors(),
        "executing the captured String write must prevent certifying an Int return"
    );
}

#[test]
fn applied_constructor_has_precise_static_type() {
    let f = Fixture::new(
        r#"
class Box<T> { @constructor new() {} }
class Probe { @class run() { let result = Box<Int>.new() } }
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().unwrap();
    let phalcom_semantic::types::store::TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(result) else {
        panic!("expected applied result")
    };
    assert_eq!(arguments.as_ref(), &[f.ty("Int")]);
}

#[test]
fn generic_inheritance_source_accepts_base_parameter() {
    let f = Fixture::new(
        r#"
class Base {}
class Child<T> is Base { @constructor new() {} }
class Probe {
  @class consume(_ value: Base) -> Int { return 1 }
  @class run() { let result = Probe.consume(Child<Int>.new()) }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn tuple_union_source_accepts_both_components() {
    let f = Fixture::new(
        r#"
class Probe {
  @class run() { let value: (Int | String, Int | String) = (1, 2) }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn unit_spellings_have_one_source_identity() {
    let f = Fixture::new(
        r#"
class Probe { @class run() { let a: Unit = (); let b: () = () } }
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn union_candidate_allocation_order_is_irrelevant() {
    fn probe(good_first: bool) -> bool {
        let mut s = TypeStore::new();
        let h = MapTypeHierarchy::new();
        let i = s.nominal(decl("Int"));
        let st = s.nominal(decl("String"));
        let u = s.union(&[i, st]);
        let tuple = |a, b| Box::new([TupleTypeElement { label: None, ty: a }, TupleTypeElement { label: None, ty: b }]);
        let actual = s.tuple(tuple(i, i));
        let (first, second) = if good_first { (i, st) } else { (st, i) };
        let a = s.tuple(tuple(u, first));
        let b = s.tuple(tuple(u, second));
        let expected = s.union(&[a, b]);
        is_subtype(&mut s, &h, actual, expected)
    }
    let bad_first = probe(false);
    let good_first = probe(true);
    eprintln!("allocation permutation: bad-first={bad_first}, good-first={good_first}");
    assert_eq!(bad_first, good_first);
    assert!(good_first);
}

#[test]
fn canonicalization_binder_rows_and_application_controls() {
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::types::store::RecordTypeField;
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
    let x = RecordTypeField { name: "x".into(), ty: i };
    let y = RecordTypeField { name: "y".into(), ty: st };
    let xy = s.record(Box::new([x.clone(), y.clone()]));
    assert_eq!(s.record(Box::new([y, x])), xy);
}

#[test]
fn representative_relation_matrix() {
    use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
    use phalcom_semantic::types::relation::check_assignability;
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
    for (name, actual) in &knowledge {
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
        eprintln!("MATRIX {name}: {}", row.join(" "));
    }
}

#[test]
fn extended_type_relation_controls() {
    use phalcom_common::selector::Selector;
    use phalcom_semantic::identity::{DispatchSide, VariantId};
    use phalcom_semantic::types::parameter::{SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::types::store::{CallableParameterType, CallableType, RecordTypeField};
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
    let x = RecordTypeField { name: "x".into(), ty: i };
    let r1 = s.record(Box::new([x.clone()]));
    let r2 = s.record(Box::new([x, RecordTypeField { name: "y".into(), ty: st }]));
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
        ("T <: Object", t, obj, true),
        ("List<T> <: List<T>", lt, lt, true),
        ("List<Int> <: List<Number>", li, ln, false),
        ("record xy <: record x", r2, r1, true),
        ("record x <: record xy", r1, r2, false),
        ("Number->Int <: Int->Number", c1, c2, true),
        ("ExactCase <: Enum", exact, en, true),
        ("Enum <: ExactCase", en, exact, false),
    ] {
        let result = is_subtype(&mut s, &h, a, b);
        eprintln!("RELATION {label}: {result}");
        assert_eq!(result, expected, "{label}");
    }
}
