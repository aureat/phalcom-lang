use phalcom_ast::parse_source;
use phalcom_semantic::core_surface::{SourceDeclarationRecord, SourceNativeBindingRole, extract_source_declarations};
use phalcom_semantic::identity::ModuleId;

#[test]
fn test_native_enum_core_surface_extraction() {
    let source = r#"
@native
enum Maybe<T> {
    @variant None
    @variant Some(_ value: T)
}
"#;
    let program = parse_source(source, 0).expect("parse program");
    let module_id = ModuleId::universe_root();

    let declarations = extract_source_declarations(&module_id, &program);
    assert_eq!(declarations.len(), 1);

    match &declarations[0] {
        SourceDeclarationRecord::Enum(enum_rec) => {
            assert_eq!(enum_rec.name, "Maybe");
            assert_eq!(enum_rec.declaration_id.name.as_ref(), "Maybe");
            assert_eq!(enum_rec.binding_role, SourceNativeBindingRole::DeclarationImplementation);
            assert_eq!(enum_rec.variants.len(), 2);

            assert_eq!(enum_rec.variants[0].name, "None");
            assert_eq!(enum_rec.variants[0].binding_role, SourceNativeBindingRole::DeclarationImplementation);

            assert_eq!(enum_rec.variants[1].name, "Some");
            assert_eq!(enum_rec.variants[1].binding_role, SourceNativeBindingRole::DeclarationImplementation);
        }
        _ => panic!("expected enum declaration record"),
    }
}
