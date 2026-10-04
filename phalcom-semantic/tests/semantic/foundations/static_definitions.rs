use phalcom_ast::ast::{
    Expr, GenericConstraintSyntax, ImplementationKind, PathSegment, StaticDefinitionBody, StaticDomainSyntax, StaticParameterSyntax, StaticSymbolRef,
    TypeAnnotation, TypeAnnotationExpr, TypeDefinitionDef, VarianceSyntax, WhereClauseSyntax,
};
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::static_definition::resolve_static_definition;
use phalcom_semantic::static_domain::StaticValueDomain;
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeFormationSite};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeStore;

const RANGE: SourceRange = SourceRange { start: 0, end: 1 };

struct TestEnv {
    store: TypeStore,
    declarations: DeclarationTypeTable,
    resolver: SimpleTypeResolver,
    site: TypeFormationSite,
    module: ModuleId,
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
    let mut resolver = SimpleTypeResolver::new();
    for name in ["Int", "String", "Bool", "Symbol", "List", "Map", "Object", "Tuple"] {
        resolver.insert(name, DeclarationId::new(module.clone(), name.into()));
    }
    TestEnv {
        store,
        declarations,
        resolver,
        site: TypeFormationSite::module(module.clone()),
        module,
    }
}

fn domain_syntax(name: &str) -> StaticDomainSyntax {
    let form = TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: name.into(),
            root_range: RANGE,
            members: Vec::<PathSegment>::new(),
            range: RANGE,
        }),
        range: RANGE,
    };
    StaticDomainSyntax::new(form, RANGE)
}

fn static_param(name: &str, domain: &str) -> StaticParameterSyntax {
    StaticParameterSyntax {
        mode: None,
        mode_range: None,
        variance: VarianceSyntax::Invariant,
        name: name.into(),
        name_range: RANGE,
        domain: Some(domain_syntax(domain)),
        range: RANGE,
    }
}

fn type_form_body(name: &str) -> StaticDefinitionBody {
    let form = TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: name.into(),
            root_range: RANGE,
            members: Vec::<PathSegment>::new(),
            range: RANGE,
        }),
        range: RANGE,
    };
    StaticDefinitionBody::Expression(Box::new(Expr::TypeForm(Box::new(form))))
}

#[test]
fn static_definition_kind_parameter_retains_type_parameter_id() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let def = TypeDefinitionDef {
        name: "Identity".into(),
        name_range: RANGE,
        parameters: vec![static_param("T", "Type")],
        where_clause: None,
        result_domain: Some(domain_syntax("Type")),
        body: type_form_body("T"),
        implementation: ImplementationKind::Phalcom,
        implementation_range: None,
        range: RANGE,
    };

    let product = resolve_static_definition(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &def,
        &env.module,
        Box::new([]),
        &mut diagnostics,
    );

    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(product.definition.signature.parameters.len(), 1);
    let param = &product.definition.signature.parameters[0];
    assert_eq!(param.name, "T");
    assert_eq!(param.domain, StaticValueDomain::Kind(KindId::TYPE));
    assert!(param.type_parameter_id.is_some(), "kind parameter must have TypeParameterId");
    assert!(product.definition.signature.generic_signature.is_some());
    assert!(product.alias_info.is_some(), "Type-valued alias must have alias_info projection");
}

#[test]
fn static_definition_value_parameter_has_no_fake_type_parameter_id() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let def = TypeDefinitionDef {
        name: "SizeOf".into(),
        name_range: RANGE,
        parameters: vec![static_param("N", "Int")],
        where_clause: None,
        result_domain: Some(domain_syntax("Int")),
        body: type_form_body("Int"),
        implementation: ImplementationKind::Phalcom,
        implementation_range: None,
        range: RANGE,
    };

    let product = resolve_static_definition(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &def,
        &env.module,
        Box::new([]),
        &mut diagnostics,
    );

    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(product.definition.signature.parameters.len(), 1);
    let param = &product.definition.signature.parameters[0];
    assert_eq!(param.name, "N");
    let int_id = DeclarationId::new(env.module.clone(), "Int".into());
    let int_type = env.store.nominal_type(int_id);
    assert_eq!(param.domain, StaticValueDomain::ValueType(int_type));
    assert_eq!(param.type_parameter_id, None, "value parameter must have no TypeParameterId");
    assert_eq!(product.definition.signature.generic_signature, None);
    // Value result domain -> alias_info must be None
    assert!(product.alias_info.is_none(), "value-valued definition must not project TypeAliasInfo");
}

#[test]
fn static_definition_mixed_parameters_and_nominal_shape_domain() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let shape_decl = DeclarationId::new(env.module.clone(), "Shape".into());
    let shape_domain = env.store.nominal_static_domain(shape_decl.clone());
    let shape_kind = env.store.static_domain_kind(shape_domain);
    env.declarations.register_intrinsic_static_domain(shape_decl.clone(), shape_kind);
    env.resolver.insert("Shape", shape_decl);

    let def = TypeDefinitionDef {
        name: "MixedDef".into(),
        name_range: RANGE,
        parameters: vec![
            static_param("N", "Int"),
            static_param("T", "Type"),
            static_param("Flag", "Bool"),
            static_param("S", "Shape"),
        ],
        where_clause: None,
        result_domain: Some(domain_syntax("Type")),
        body: type_form_body("T"),
        implementation: ImplementationKind::Phalcom,
        implementation_range: None,
        range: RANGE,
    };

    let product = resolve_static_definition(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &def,
        &env.module,
        Box::new([]),
        &mut diagnostics,
    );

    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    let params = &product.definition.signature.parameters;
    assert_eq!(params.len(), 4);

    // 0: N: Int (value)
    assert_eq!(params[0].name, "N");
    assert_eq!(params[0].type_parameter_id, None);

    // 1: T: Type (kind)
    assert_eq!(params[1].name, "T");
    assert!(params[1].type_parameter_id.is_some());

    // 2: Flag: Bool (value)
    assert_eq!(params[2].name, "Flag");
    assert_eq!(params[2].type_parameter_id, None);

    // 3: S: Shape (kind)
    assert_eq!(params[3].name, "S");
    assert!(params[3].type_parameter_id.is_some());

    // Generic signature has exactly 2 parameters (T and S)
    let gen_sig = product.definition.signature.generic_signature.as_ref().unwrap();
    assert_eq!(gen_sig.parameters.len(), 2);
    assert_eq!(params[1].type_parameter_id.unwrap(), gen_sig.parameters[0]);
    assert_eq!(params[3].type_parameter_id.unwrap(), gen_sig.parameters[1]);

    // Mixed parameters cannot project TypeAliasInfo (C3 only supports kinded parameters)
    assert!(product.alias_info.is_none());
}

#[test]
fn static_definition_diagnoses_value_parameter_in_where_clause() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let def = TypeDefinitionDef {
        name: "Constrained".into(),
        name_range: RANGE,
        parameters: vec![static_param("N", "Int"), static_param("T", "Type")],
        where_clause: Some(WhereClauseSyntax {
            constraints: vec![GenericConstraintSyntax::Subtype {
                lower: TypeAnnotation {
                    expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
                        root: "N".into(),
                        root_range: RANGE,
                        members: Vec::<PathSegment>::new(),
                        range: RANGE,
                    }),
                    range: RANGE,
                },
                upper: TypeAnnotation {
                    expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
                        root: "Int".into(),
                        root_range: RANGE,
                        members: Vec::<PathSegment>::new(),
                        range: RANGE,
                    }),
                    range: RANGE,
                },
                range: RANGE,
            }],
            range: RANGE,
        }),
        result_domain: Some(domain_syntax("Type")),
        body: type_form_body("T"),
        implementation: ImplementationKind::Phalcom,
        implementation_range: None,
        range: RANGE,
    };

    let _product = resolve_static_definition(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &def,
        &env.module,
        Box::new([]),
        &mut diagnostics,
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnsupported);
    assert!(diagnostics[0].message.contains("value parameter `N` cannot appear in type-level constraints"));
}

#[test]
fn static_definition_legacy_alias_compatibility_projection() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    // type MyInt = Int
    let def = TypeDefinitionDef {
        name: "MyInt".into(),
        name_range: RANGE,
        parameters: vec![],
        where_clause: None,
        result_domain: None, // defaults to Kind(Type)
        body: type_form_body("Int"),
        implementation: ImplementationKind::Phalcom,
        implementation_range: None,
        range: RANGE,
    };

    let product = resolve_static_definition(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &def,
        &env.module,
        Box::new([]),
        &mut diagnostics,
    );

    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(product.definition.signature.result_domain, StaticValueDomain::Kind(KindId::TYPE));
    assert!(product.alias_info.is_some(), "legacy alias must have alias_info projection");
    let info = product.alias_info.as_ref().unwrap();
    assert_eq!(info.declaration.name.as_ref(), "MyInt");
    assert_eq!(info.kind, KindId::TYPE);
    assert_eq!(info.generic_signature, None);
    let int_id = DeclarationId::new(env.module.clone(), "Int".into());
    let expected_type = env.store.nominal_type(int_id);
    assert_eq!(info.form, expected_type);
}
