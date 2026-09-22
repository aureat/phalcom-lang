use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use phalcom_semantic::dispatch::{CallableParameter, CallableSignature, DispatchLookup, DispatchResult, DispatchSide};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::surface::DeclarationSurface;
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeStore};
use phalcom_semantic::types::substitution::TypeSubstitution;
use phalcom_semantic::{CheckingContext, SimpleTypeResolver};

#[test]
fn direct_and_nested_and_composite_substitution() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let decl_box = DeclarationId::new(module.clone(), "Box".into());
    let decl_list = DeclarationId::new(module.clone(), "List".into());
    let decl_int = DeclarationId::new(module.clone(), "Int".into());
    let decl_str = DeclarationId::new(module.clone(), "String".into());

    let int_ty = store.nominal_type(decl_int);
    let str_ty = store.nominal_type(decl_str);

    let arrow_1 = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let arrow_2 = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);

    let list_form = store.nominal_form(decl_list, arrow_1);
    let _box_form = store.nominal_form(decl_box.clone(), arrow_2);

    let param_t_id = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl_box.clone()), 0, "T", KindId::TYPE));
    let param_u_id = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl_box), 1, "U", KindId::TYPE));

    let t_ty = store.parameter_form(param_t_id);
    let u_ty = store.parameter_form(param_u_id);

    let mut subst = TypeSubstitution::new();
    subst.bind(param_t_id, int_ty);

    // 1. Direct T -> Int
    assert_eq!(subst.apply(&mut store, t_ty), int_ty);

    // 2. Unbound U remains U (partial substitution)
    assert_eq!(subst.apply(&mut store, u_ty), u_ty);

    // 3. Nested List<T> -> List<Int>
    let list_t = store.apply_type_form(list_form, &[t_ty]).unwrap();
    let list_int = subst.apply(&mut store, list_t);
    let expected_list_int = store.apply_type_form(list_form, &[int_ty]).unwrap();
    assert_eq!(list_int, expected_list_int);

    // 4. Tuple (T, String) -> (Int, String)
    let tuple = store.tuple(
        vec![
            TupleTypeElement { label: None, ty: t_ty },
            TupleTypeElement {
                label: Some("name".into()),
                ty: str_ty,
            },
        ]
        .into_boxed_slice(),
    );
    let subst_tuple = subst.apply(&mut store, tuple);
    let expected_tuple = store.tuple(
        vec![
            TupleTypeElement { label: None, ty: int_ty },
            TupleTypeElement {
                label: Some("name".into()),
                ty: str_ty,
            },
        ]
        .into_boxed_slice(),
    );
    assert_eq!(subst_tuple, expected_tuple);

    // 5. Record { a: T, b: String } -> { a: Int, b: String }
    let record = store.record(vec![RecordTypeField { name: "a".into(), ty: t_ty }, RecordTypeField { name: "b".into(), ty: str_ty }].into_boxed_slice());
    let subst_record = subst.apply(&mut store, record);
    let expected_record = store.record(
        vec![
            RecordTypeField { name: "a".into(), ty: int_ty },
            RecordTypeField { name: "b".into(), ty: str_ty },
        ]
        .into_boxed_slice(),
    );
    assert_eq!(subst_record, expected_record);

    // 6. Union T | String -> Int | String
    let union = store.union(&[t_ty, str_ty]);
    let subst_union = subst.apply(&mut store, union);
    let expected_union = store.union(&[int_ty, str_ty]);
    assert_eq!(subst_union, expected_union);

    // 7. Callable (T) -> List<T> -> (Int) -> List<Int>
    let callable = store.callable(CallableType {
        parameters: vec![CallableParameterType {
            label: None,
            ty: t_ty,
            rest: phalcom_ast::ast::RestMode::None,
        }]
        .into_boxed_slice(),
        return_type: list_t,
    });
    let subst_callable = subst.apply(&mut store, callable);
    let expected_callable = store.callable(CallableType {
        parameters: vec![CallableParameterType {
            label: None,
            ty: int_ty,
            rest: phalcom_ast::ast::RestMode::None,
        }]
        .into_boxed_slice(),
        return_type: expected_list_int,
    });
    assert_eq!(subst_callable, expected_callable);
}

#[test]
fn applied_member_views_on_box_int() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let decl_box = DeclarationId::new(module.clone(), "Box".into());
    let decl_int = DeclarationId::new(module.clone(), "Int".into());

    let int_ty = store.nominal_type(decl_int.clone());

    let arrow_1 = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let box_form = store.nominal_form(decl_box.clone(), arrow_1);
    let box_class_obj = store.class_object_type(decl_box.clone());

    let param_t_id = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl_box.clone()), 0, "T", KindId::TYPE));
    let t_ty = store.parameter_form(param_t_id);

    let mut decls = DeclarationTypeTable::new();
    decls.insert(DeclarationTypeInfo {
        declaration: decl_box.clone(),
        form: box_form,
        class_object_type: box_class_obj,
        kind: arrow_1,
        generic_binder_header: None,
        generic_signature: Some(GenericSignature::new(
            TypeParameterOwner::Declaration(decl_box.clone()),
            vec![param_t_id].into_boxed_slice(),
        )),
        supertype_template: None,
    });
    decls.insert(DeclarationTypeInfo {
        declaration: decl_int.clone(),
        form: int_ty,
        class_object_type: store.class_object_type(decl_int),
        kind: KindId::TYPE,
        generic_binder_header: None,
        generic_signature: None,
        supertype_template: None,
    });

    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &decls, module);

    let mut surface = DeclarationSurface::new(Some(decl_box.clone()));
    // def get() -> T
    let get_sel = Selector::getter("get").unwrap();
    surface.add_callable(
        DispatchSide::Instance,
        CallableSignature::new(get_sel.clone(), Vec::new(), TypeKnowledge::assumed(t_ty, EvidenceOrigin::DeveloperAnnotation)),
    );
    // def put(value: T) -> Unit
    let put_sel = Selector::method("put", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    surface.add_callable(
        DispatchSide::Instance,
        CallableSignature::new(
            put_sel.clone(),
            vec![CallableParameter::new(
                "value",
                TypeKnowledge::assumed(t_ty, EvidenceOrigin::DeveloperAnnotation),
            )],
            TypeKnowledge::assumed(ctx.store.unit(), EvidenceOrigin::DeveloperAnnotation),
        ),
    );
    ctx.register_surface(decl_box, surface);

    let box_int_ty = ctx.store.apply_type_form(box_form, &[int_ty]).unwrap();

    // Query get on Box<Int>
    let get_res = ctx.resolve_dispatch(box_int_ty, &get_sel, DispatchLookup::Normal);
    let DispatchResult::Found(get_sig) = get_res else {
        panic!("expected found get")
    };
    assert_eq!(get_sig.return_type.ty().unwrap(), int_ty, "Box<Int>.get() must return Int");

    // Query put on Box<Int>
    let put_res = ctx.resolve_dispatch(box_int_ty, &put_sel, DispatchLookup::Normal);
    let DispatchResult::Found(put_sig) = put_res else {
        panic!("expected found put")
    };
    assert_eq!(put_sig.parameters[0].ty.ty().unwrap(), int_ty, "Box<Int>.put() must accept Int");
}

#[test]
fn applied_generic_subtyping_is_invariant_and_class_object_is_hierarchical() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let decl_box = DeclarationId::new(module.clone(), "Box".into());
    let decl_num = DeclarationId::new(module.clone(), "Number".into());
    let decl_int = DeclarationId::new(module.clone(), "Int".into());
    let decl_sub = DeclarationId::new(module.clone(), "Sub".into());
    let decl_super = DeclarationId::new(module.clone(), "Super".into());

    let num_ty = store.nominal_type(decl_num.clone());
    let int_ty = store.nominal_type(decl_int.clone());

    let arrow_1 = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let box_form = store.nominal_form(decl_box, arrow_1);

    let box_int = store.apply_type_form(box_form, &[int_ty]).unwrap();
    let box_num = store.apply_type_form(box_form, &[num_ty]).unwrap();

    let mut hierarchy = MapTypeHierarchy::new();
    hierarchy.insert(decl_int, decl_num);
    hierarchy.insert(decl_sub.clone(), decl_super.clone());

    // Int <: Number
    assert!(is_subtype(&mut store, &hierarchy, int_ty, num_ty));

    // Box<Int> is NOT a subtype of Box<Number> (invariant)
    assert!(!is_subtype(&mut store, &hierarchy, box_int, box_num));

    // ClassObject(Sub) <: ClassObject(Super)
    let sub_class_obj = store.class_object_type(decl_sub);
    let super_class_obj = store.class_object_type(decl_super);
    assert!(is_subtype(&mut store, &hierarchy, sub_class_obj, super_class_obj));
}

#[test]
fn materialization_recanonicalizes_labeled_pack_concat_and_surfaces_duplicates() {
    use phalcom_semantic::types::instantiation::{GenericInstantiation, RowMaterializationMode, TypeMaterializationError, materialize_type};
    use phalcom_semantic::types::store::LabeledTypePackElement;

    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "Packs".into()));
    let left_parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "L", KindId::LABELED_TYPE_PACK));
    let right_parameter = store.intern_type_parameter(TypeParameterData::new(owner, 1, "R", KindId::LABELED_TYPE_PACK));
    let left_form = store.parameter_form(left_parameter);
    let right_form = store.parameter_form(right_parameter);
    let concat = store.labeled_type_pack_concat(&[left_form, right_form]).unwrap();
    let int = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let str_ty = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "String".into()));
    let left = store
        .labeled_type_pack(Box::new([LabeledTypePackElement {
            label: "name".into(),
            element: int,
        }]))
        .unwrap();
    let right = store
        .labeled_type_pack(Box::new([LabeledTypePackElement {
            label: "name".into(),
            element: str_ty,
        }]))
        .unwrap();
    let mut instantiation = GenericInstantiation::default();
    instantiation.bind_type(left_parameter, left);
    instantiation.bind_type(right_parameter, right);

    assert!(matches!(
        materialize_type(&mut store, concat, &instantiation, RowMaterializationMode::PreserveUnboundStableTail),
        Err(TypeMaterializationError::Pack(_))
    ));
}

#[test]
fn p3_dense_positional_concat_remains_flat() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "Dense".into()));

    let mut segments = Vec::new();
    for i in 0..256 {
        let param = store.intern_type_parameter(TypeParameterData::new(owner.clone(), i, format!("P{i}"), KindId::TYPE_PACK));
        segments.push(store.parameter_form(param));
    }

    let concat = store.type_pack_concat(&segments).expect("dense concat must succeed");
    match store.get(concat) {
        phalcom_semantic::types::store::TypeData::TypePackConcat(children) => {
            assert_eq!(children.len(), 256);
            for child in children.iter() {
                assert!(!matches!(store.get(*child), phalcom_semantic::types::store::TypeData::TypePackConcat(_)));
            }
        }
        other => panic!("expected TypePackConcat, got {other:?}"),
    }
}

#[test]
fn p3_dense_labeled_concat_remains_flat_and_ordered() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "Dense".into()));

    let mut segments = Vec::new();
    for i in 0..256 {
        let param = store.intern_type_parameter(TypeParameterData::new(owner.clone(), i, format!("L{i}"), KindId::LABELED_TYPE_PACK));
        segments.push(store.parameter_form(param));
    }

    let concat = store.labeled_type_pack_concat(&segments).expect("dense labeled concat must succeed");
    match store.get(concat) {
        phalcom_semantic::types::store::TypeData::LabeledTypePackConcat(children) => {
            assert_eq!(children.len(), 256);
            for child in children.iter() {
                assert!(!matches!(store.get(*child), phalcom_semantic::types::store::TypeData::LabeledTypePackConcat(_)));
            }
        }
        other => panic!("expected LabeledTypePackConcat, got {other:?}"),
    }
}

#[test]
fn p3_symbolic_labeled_transition_to_disjoint_and_duplicate() {
    use phalcom_semantic::types::instantiation::{GenericInstantiation, RowMaterializationMode, TypeMaterializationError, materialize_type};
    use phalcom_semantic::types::store::LabeledTypePackElement;

    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "Packs".into()));
    let p_left = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "L", KindId::LABELED_TYPE_PACK));
    let p_right = store.intern_type_parameter(TypeParameterData::new(owner, 1, "R", KindId::LABELED_TYPE_PACK));

    let f_left = store.parameter_form(p_left);
    let f_right = store.parameter_form(p_right);
    let concat = store.labeled_type_pack_concat(&[f_left, f_right]).unwrap();

    let int = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));

    // 1. Transition to disjoint: [a: Int] ++ [b: Int] -> [a: Int, b: Int]
    let left_disjoint = store
        .labeled_type_pack(Box::new([LabeledTypePackElement {
            label: "a".into(),
            element: int,
        }]))
        .unwrap();
    let right_disjoint = store
        .labeled_type_pack(Box::new([LabeledTypePackElement {
            label: "b".into(),
            element: int,
        }]))
        .unwrap();

    let mut inst_disjoint = GenericInstantiation::default();
    inst_disjoint.bind_type(p_left, left_disjoint);
    inst_disjoint.bind_type(p_right, right_disjoint);

    let res_disjoint = materialize_type(&mut store, concat, &inst_disjoint, RowMaterializationMode::PreserveUnboundStableTail);
    assert!(res_disjoint.is_ok());
    let materialized_ty = res_disjoint.unwrap();
    assert!(matches!(
        store.get(materialized_ty),
        phalcom_semantic::types::store::TypeData::LabeledTypePack(_)
    ));

    // 2. Transition to duplicate: [dup: Int] ++ [dup: Int] -> Error
    let left_dup = store
        .labeled_type_pack(Box::new([LabeledTypePackElement {
            label: "dup".into(),
            element: int,
        }]))
        .unwrap();
    let right_dup = store
        .labeled_type_pack(Box::new([LabeledTypePackElement {
            label: "dup".into(),
            element: int,
        }]))
        .unwrap();

    let mut inst_dup = GenericInstantiation::default();
    inst_dup.bind_type(p_left, left_dup);
    inst_dup.bind_type(p_right, right_dup);

    let res_dup = materialize_type(&mut store, concat, &inst_dup, RowMaterializationMode::PreserveUnboundStableTail);
    assert!(matches!(res_dup, Err(TypeMaterializationError::Pack(_))));
}
