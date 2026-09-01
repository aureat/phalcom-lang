use phalcom_common::range::SourceRange;
use phalcom_modules::DeclarationId;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::flow::predicate::FlowPredicate;
use phalcom_semantic::checker::flow::state::FlowState;
use phalcom_semantic::checker::flow::transfer::{PredicateTransfer, apply_predicate};
use phalcom_semantic::identity::BindingId;
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

#[test]
fn test_predicate_transfer_pure_matrix() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let module = ModuleId::universe_root();

    let obj_decl = DeclarationId::new(module.clone(), "Object".into());
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module.clone(), "String".into());
    let animal_decl = DeclarationId::new(module.clone(), "Animal".into());
    let cat_decl = DeclarationId::new(module.clone(), "Cat".into());
    let dog_decl = DeclarationId::new(module.clone(), "Dog".into());

    let obj_ty = store.nominal(obj_decl.clone());
    let int_ty = store.nominal(int_decl.clone());
    let str_ty = store.nominal(str_decl.clone());
    let animal_ty = store.nominal(animal_decl.clone());
    let cat_ty = store.nominal(cat_decl.clone());
    let dog_ty = store.nominal(dog_decl.clone());

    hierarchy.insert(cat_decl, animal_decl.clone());
    hierarchy.insert(dog_decl, animal_decl.clone());
    hierarchy.insert(int_decl, obj_decl.clone());
    hierarchy.insert(str_decl, obj_decl.clone());
    hierarchy.insert(animal_decl, obj_decl);

    let int_or_str = store.union(&[int_ty, str_ty]);
    let cat_or_dog = store.union(&[cat_ty, dog_ty]);

    let b = BindingId(1);

    // 1. Unknown + authoritative is(Int) -> Established<Int>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(int_ty));
        assert_eq!(cur.status(), Some(EvidenceStatus::Established));
    }

    // 2. Dynamic + authoritative is(Int) -> Dynamic unchanged
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert_eq!(res, PredicateTransfer::Unchanged);
    }

    // 3. Assumed<Object> + authoritative is(Int) -> Established<Int>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::assumed(obj_ty, EvidenceOrigin::DeveloperAnnotation),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(int_ty));
        assert_eq!(cur.status(), Some(EvidenceStatus::Established));
    }

    // 4. Assumed<Int|String> + authoritative is(Int) -> Established<Int>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::assumed(int_or_str, EvidenceOrigin::DeveloperAnnotation),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(int_ty));
        assert_eq!(cur.status(), Some(EvidenceStatus::Established));
    }

    // 5. Assumed<Cat|Dog> + authoritative is(Animal) -> Assumed<Cat|Dog>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::assumed(cat_or_dog, EvidenceOrigin::DeveloperAnnotation),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: animal_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert_eq!(res, PredicateTransfer::Unchanged);
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(cat_or_dog));
        assert_eq!(cur.status(), Some(EvidenceStatus::Assumed));
    }

    // 6. Established<Cat|Dog> + authoritative is(Animal) -> Established<Cat|Dog>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(cat_or_dog, EvidenceOrigin::Syntax),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: animal_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert_eq!(res, PredicateTransfer::Unchanged);
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(cat_or_dog));
        assert_eq!(cur.status(), Some(EvidenceStatus::Established));
    }

    // 7. Established<String> + authoritative is(Int) -> Contradiction
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(str_ty, EvidenceOrigin::Syntax),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Contradiction { .. }));
    }

    // 8. Assumed<String> + authoritative is(Int) -> Established<Int>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::assumed(str_ty, EvidenceOrigin::DeveloperAnnotation),
            true,
        );
        let pred = FlowPredicate::IsInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(int_ty));
        assert_eq!(cur.status(), Some(EvidenceStatus::Established));
    }

    // 9. Established<Int|String> + IsNotInstance(Int) -> Established<String>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(int_or_str, EvidenceOrigin::Syntax),
            true,
        );
        let pred = FlowPredicate::IsNotInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(str_ty));
        assert_eq!(cur.status(), Some(EvidenceStatus::Established));
    }

    // 10. Assumed<Int|String> + IsNotInstance(Int) -> Assumed<String>
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::assumed(int_or_str, EvidenceOrigin::DeveloperAnnotation),
            true,
        );
        let pred = FlowPredicate::IsNotInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(cur.ty(), Some(str_ty));
        assert_eq!(cur.status(), Some(EvidenceStatus::Assumed));
    }

    // 11. Established<Int> + IsNotInstance(Int) -> Contradiction
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
            true,
        );
        let pred = FlowPredicate::IsNotInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Contradiction { .. }));
    }

    // 12. Assumed<Int> + IsNotInstance(Int) -> Unknown(InferenceConflict)
    {
        let mut state = FlowState::new();
        state.declare(
            b,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation),
            true,
        );
        let pred = FlowPredicate::IsNotInstance { binding: b, target: int_ty }.authoritative();
        let res = apply_predicate(&mut state, &pred, &mut store, &hierarchy);
        assert!(matches!(res, PredicateTransfer::Refined(_)));
        let cur = state.get_current_type(b).unwrap();
        assert_eq!(*cur, TypeKnowledge::Unknown(UnknownReason::InferenceConflict));
    }
}
