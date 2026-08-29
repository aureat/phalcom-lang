use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::db::{CancellationToken, DependencyRecorder, InputFingerprint, ProductFingerprint, QueryBudget, QueryKey, QueryValue, SemanticDb};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::{KindId, TypeId};
use phalcom_semantic::types::relation::{MapTypeHierarchy, check_assignability_bounded, check_subtype_bounded};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::{SnapshotTypeRef, TypeHierarchy};

fn decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
}

#[test]
fn proper_type_enforcement_and_kinds() {
    let mut store = TypeStore::new();
    let int_id = store.nominal_type(decl("Int"));
    let proper_int = store.proper_type(int_id).expect("Int is a proper type");
    assert_eq!(proper_int.raw(), int_id);

    // Create a constructor form (kind: Type -> Type)
    let list_arrow = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list_ctor = store.nominal_form(decl("List"), list_arrow);
    assert_eq!(store.kind_of(list_ctor), list_arrow);

    let err = store.proper_type(list_ctor).expect_err("constructor is not a proper type");
    assert_eq!(err, list_arrow);

    // ProperTypeId in TypeKnowledge
    let knowledge = TypeKnowledge::established(proper_int, EvidenceOrigin::Syntax);
    assert!(knowledge.is_known());
    assert_eq!(knowledge.ty(), Some(int_id));
    assert_eq!(knowledge.proper_ty(&store), Some(proper_int));
}

#[test]
fn formal_knowledge_keeps_status_and_origin_independent() {
    let established = TypeKnowledge::established(TypeId(7), EvidenceOrigin::ConstructorSemantics);
    assert_eq!(established.status(), Some(EvidenceStatus::Established));
    assert_eq!(established.origin(), Some(EvidenceOrigin::ConstructorSemantics));

    let assumed = TypeKnowledge::assumed(TypeId(7), EvidenceOrigin::DeveloperAnnotation);
    assert_eq!(assumed.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(assumed.origin(), Some(EvidenceOrigin::DeveloperAnnotation));

    let mapped = established.map_type(|ty| TypeId(ty.0 + 1));
    assert_eq!(mapped.ty(), Some(TypeId(8)));
    assert_eq!(mapped.status(), Some(EvidenceStatus::Established));
    assert_eq!(mapped.origin(), Some(EvidenceOrigin::ConstructorSemantics));
}

#[test]
fn cross_store_identity_distinctness() {
    let store1 = TypeStore::new();
    let store2 = TypeStore::new();
    assert_ne!(store1.id(), store2.id());

    let ref1 = SnapshotTypeRef::new(store1.id(), TypeId(1));
    let ref2 = SnapshotTypeRef::new(store2.id(), TypeId(1));
    assert_ne!(ref1, ref2);
}

#[test]
fn relation_outcomes_distinguish_terminal_states() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();

    let int_ty = store.nominal(decl("Int"));
    let str_ty = store.nominal(decl("String"));

    let mut budget = QueryBudget::default();
    let token = CancellationToken::new();

    // 1. Proven
    let proven_res = check_subtype_bounded(&mut store, &hier, int_ty, int_ty, &mut budget, &token);
    assert!(proven_res.is_proven());

    // 2. Refuted
    let refuted_res = check_subtype_bounded(&mut store, &hier, int_ty, str_ty, &mut budget, &token);
    assert!(refuted_res.is_refuted());

    // 3. Cancelled
    let cancelled_token = CancellationToken::new();
    cancelled_token.cancel();
    let cancelled_res = check_subtype_bounded(&mut store, &hier, int_ty, str_ty, &mut budget, &cancelled_token);
    assert!(cancelled_res.is_cancelled());

    // 4. BudgetExceeded
    let mut zero_budget = QueryBudget::new(0);
    let budget_res = check_subtype_bounded(&mut store, &hier, int_ty, str_ty, &mut zero_budget, &token);
    assert!(budget_res.is_budget_exceeded());

    // 5. DynamicBoundary
    let dyn_k = TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape);
    let known_k = TypeKnowledge::assumed(store.proper_type(int_ty).unwrap(), EvidenceOrigin::DeveloperAnnotation);
    let dyn_res = check_assignability_bounded(&mut store, &hier, &dyn_k, &known_k, &mut budget, &token);
    assert!(dyn_res.is_dynamic_boundary());

    // 6. Blocked
    let unk_k = TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration);
    let blocked_res = check_assignability_bounded(&mut store, &hier, &unk_k, &known_k, &mut budget, &token);
    assert!(blocked_res.is_blocked());
}

#[test]
fn inheritance_cycle_terminates_and_refutes() {
    let mut store = TypeStore::new();
    let mut hier = MapTypeHierarchy::new();

    let a = decl("A");
    let b = decl("B");
    let c = decl("C");

    // A -> B -> C -> A (cycle)
    hier.insert(a.clone(), b.clone());
    hier.insert(b.clone(), c.clone());
    hier.insert(c.clone(), a.clone());

    let target = decl("Target");
    assert!(!hier.is_subclass(&a, &target));

    let t_a = store.nominal(a);
    let t_target = store.nominal(target);

    let mut budget = QueryBudget::default();
    let token = CancellationToken::new();
    let outcome = check_subtype_bounded(&mut store, &hier, t_a, t_target, &mut budget, &token);
    assert!(outcome.is_refuted());
}

#[test]
fn clean_vs_incremental_differential_equivalence() {
    let mut db = SemanticDb::new();
    let module_a = ModuleId::core();
    let key_parsed = QueryKey::ParsedModule(module_a.clone());
    let key_unlinked = QueryKey::UnlinkedInterface(module_a.clone());
    let key_linked = QueryKey::LinkedInterface(module_a.clone());

    let rev1 = db.revision();
    let val_parsed = QueryValue::from_bytes(b"parsed_v1");
    let val_unlinked = QueryValue::from_bytes(b"unlinked_v1");
    let val_linked = QueryValue::from_bytes(b"linked_v1");

    // Build initial dependencies: linked -> unlinked -> parsed
    let mut unlinked_rec = DependencyRecorder::new(key_unlinked.clone());
    unlinked_rec.record(key_parsed.clone(), ProductFingerprint::new(100));
    let mut linked_rec = DependencyRecorder::new(key_linked.clone());
    linked_rec.record(key_unlinked.clone(), ProductFingerprint::new(200));

    db.publish_ready(
        key_parsed.clone(),
        rev1,
        InputFingerprint::new(100),
        ProductFingerprint::new(100),
        val_parsed,
        [],
    )
    .unwrap();
    db.publish_ready(
        key_unlinked.clone(),
        rev1,
        InputFingerprint::new(200),
        ProductFingerprint::new(200),
        val_unlinked,
        unlinked_rec.finish(),
    )
    .unwrap();
    db.publish_ready(
        key_linked.clone(),
        rev1,
        InputFingerprint::new(300),
        ProductFingerprint::new(300),
        val_linked,
        linked_rec.finish(),
    )
    .unwrap();

    assert!(db.query_state(&key_parsed).unwrap().is_ready());
    assert!(db.query_state(&key_unlinked).unwrap().is_ready());
    assert!(db.query_state(&key_linked).unwrap().is_ready());

    // Invalidate leaf (parsed)
    let invalidated = db.invalidate([key_parsed.clone()]);
    assert!(invalidated.contains(&key_parsed));
    assert!(invalidated.contains(&key_unlinked));
    assert!(invalidated.contains(&key_linked));

    assert!(db.query_state(&key_parsed).is_none());
    assert!(db.query_state(&key_unlinked).is_none());
    assert!(db.query_state(&key_linked).is_none());

    // Recompute on rev2
    let rev2 = db.begin_revision();
    let val_parsed_v2 = QueryValue::from_bytes(b"parsed_v2");
    let mut unlinked_rec2 = DependencyRecorder::new(key_unlinked.clone());
    unlinked_rec2.record(key_parsed.clone(), ProductFingerprint::new(101));

    db.publish_ready(
        key_parsed.clone(),
        rev2,
        InputFingerprint::new(101),
        ProductFingerprint::new(101),
        val_parsed_v2,
        [],
    )
    .unwrap();
    db.publish_ready(
        key_unlinked.clone(),
        rev2,
        InputFingerprint::new(201),
        ProductFingerprint::new(201),
        QueryValue::from_bytes(b"unlinked_v2"),
        unlinked_rec2.finish(),
    )
    .unwrap();

    assert_eq!(db.query_state(&key_parsed).unwrap().revision(), Some(rev2));
    assert_eq!(db.query_state(&key_unlinked).unwrap().revision(), Some(rev2));
}
