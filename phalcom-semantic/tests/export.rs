use phalcom_semantic::identity::{DeclarationId, ModuleId};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeStore};
use phalcom_semantic::{
    CompiledCallableParam, CompiledCallableType, CompiledKindRef, CompiledRecordField, CompiledTupleElement, CompiledTypeParameterOwner, CompiledTypeRef,
    SemanticExportError, TypeParameterData, TypeParameterOwner, export_kind, export_type_form,
};

#[test]
fn export_kind_primitives_and_arrows() {
    let mut store = TypeStore::new();
    assert_eq!(export_kind(&store, KindId::TYPE), CompiledKindRef::Type);

    let arrow = store.arrow_kind(Box::new([KindId::TYPE, KindId::TYPE]), KindId::TYPE);
    assert_eq!(
        export_kind(&store, arrow),
        CompiledKindRef::Arrow {
            parameters: Box::new([CompiledKindRef::Type, CompiledKindRef::Type]),
            result: Box::new(CompiledKindRef::Type),
        }
    );
}

#[test]
fn export_type_form_nominal_applied_union_tuple_record_callable_parameter() {
    let mut store = TypeStore::new();

    // 1. Primitive types
    assert_eq!(export_type_form(&store, store.never()).unwrap(), CompiledTypeRef::Never);
    assert_eq!(export_type_form(&store, store.unit()).unwrap(), CompiledTypeRef::Unit);

    // 2. Nominal
    let decl = DeclarationId::new(ModuleId::core(), "Int".into());
    let int_ty = store.nominal_type(decl.clone());
    assert_eq!(export_type_form(&store, int_ty).unwrap(), CompiledTypeRef::Nominal(decl.clone()));

    // 3. Applied generic
    let list_decl = DeclarationId::new(ModuleId::core(), "List".into());
    let list_kind = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list_ctor = store.nominal_form(list_decl.clone(), list_kind);
    let list_int = store.apply_type_form(list_ctor, &[int_ty]).unwrap();
    assert_eq!(
        export_type_form(&store, list_int).unwrap(),
        CompiledTypeRef::Applied {
            origin: Box::new(CompiledTypeRef::Nominal(list_decl)),
            arguments: Box::new([CompiledTypeRef::Nominal(decl.clone())]),
        }
    );

    // 4. Union
    let str_decl = DeclarationId::new(ModuleId::core(), "String".into());
    let str_ty = store.nominal_type(str_decl.clone());
    let union_ty = store.union(&[int_ty, str_ty]);
    assert_eq!(
        export_type_form(&store, union_ty).unwrap(),
        CompiledTypeRef::Union(Box::new([CompiledTypeRef::Nominal(decl.clone()), CompiledTypeRef::Nominal(str_decl.clone()),]))
    );

    // 5. Tuple
    let tuple_ty = store.tuple(Box::new([
        TupleTypeElement { label: None, ty: int_ty },
        TupleTypeElement {
            label: Some("name".into()),
            ty: str_ty,
        },
    ]));
    assert_eq!(
        export_type_form(&store, tuple_ty).unwrap(),
        CompiledTypeRef::Tuple(Box::new([
            CompiledTupleElement {
                label: None,
                ty: CompiledTypeRef::Nominal(decl.clone()),
            },
            CompiledTupleElement {
                label: Some("name".into()),
                ty: CompiledTypeRef::Nominal(str_decl.clone()),
            },
        ]))
    );

    // 6. Record
    let record_ty = store.record(Box::new([
        RecordTypeField { name: "x".into(), ty: int_ty },
        RecordTypeField { name: "y".into(), ty: int_ty },
    ]));
    assert_eq!(
        export_type_form(&store, record_ty).unwrap(),
        CompiledTypeRef::Record(Box::new([
            CompiledRecordField {
                name: "x".into(),
                ty: CompiledTypeRef::Nominal(decl.clone()),
            },
            CompiledRecordField {
                name: "y".into(),
                ty: CompiledTypeRef::Nominal(decl.clone()),
            },
        ]))
    );

    // 7. Callable
    let callable_ty = store.callable(CallableType {
        parameters: Box::new([CallableParameterType {
            label: Some("arg".into()),
            ty: int_ty,
            rest: false,
        }]),
        return_type: str_ty,
    });
    assert_eq!(
        export_type_form(&store, callable_ty).unwrap(),
        CompiledTypeRef::Callable(CompiledCallableType {
            positional: Box::new([CompiledCallableParam {
                name: Some("arg".into()),
                ty: CompiledTypeRef::Nominal(decl.clone()),
            }]),
            return_type: Box::new(CompiledTypeRef::Nominal(str_decl)),
        })
    );

    // 8. Parameter
    let param_id = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl.clone()), 0, "T", KindId::TYPE));
    let param_ty = store.parameter_form(param_id);

    assert_eq!(
        export_type_form(&store, param_ty).unwrap(),
        CompiledTypeRef::Parameter {
            owner: CompiledTypeParameterOwner::Declaration(decl.clone()),
            index: 0,
        }
    );
}

#[test]
fn export_rejects_class_object() {
    let mut store = TypeStore::new();

    // ClassObject rejected as non-exportable
    let decl = DeclarationId::new(ModuleId::core(), "Point".into());
    let class_obj = store.class_object_type(decl);
    assert_eq!(
        export_type_form(&store, class_obj),
        Err(SemanticExportError::NonExportableTypeForm { form: class_obj })
    );
}
