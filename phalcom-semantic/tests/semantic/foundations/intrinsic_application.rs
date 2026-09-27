use phalcom_ast::ast::{GenericArgumentSyntax, PathSegment, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeBinding, DeclarationTypeInfo, DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeFormResolution, TypeFormationSite, resolve_type_form};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{GenericBinderHeader, GenericParameterMode, GenericSignature, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::row::RecordRowTail;
use phalcom_semantic::types::store::{StaticArgument, TypeData, TypeStore};

const RANGE: SourceRange = SourceRange { start: 0, end: 1 };

struct TestEnv {
    store: TypeStore,
    declarations: DeclarationTypeTable,
    resolver: SimpleTypeResolver,
    module: ModuleId,
    site: TypeFormationSite,
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
    let mut resolver = SimpleTypeResolver::new();
    for name in ["Tuple", "Record", "Int"] {
        resolver.insert(name, DeclarationId::new(module.clone(), name.into()));
    }
    TestEnv {
        store,
        declarations,
        resolver,
        module: module.clone(),
        site: TypeFormationSite::module(module),
    }
}

fn reference(name: &str) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: name.into(),
            root_range: RANGE,
            members: Vec::<PathSegment>::new(),
            range: RANGE,
        }),
        range: RANGE,
    }
}

fn application(origin: &str, arguments: Vec<GenericArgumentSyntax>) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference(origin)),
            arguments,
            range: RANGE,
        },
        range: RANGE,
    }
}

fn install_variadic_family(env: &mut TestEnv, name: &str, kind: KindId, mode: GenericParameterMode) {
    let declaration = DeclarationId::new(env.module.clone(), name.into());
    let owner = TypeParameterOwner::Declaration(declaration.clone());
    let parameter = env
        .store
        .intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "Args", kind).with_mode(mode));
    let signature = GenericSignature::new(owner, vec![parameter].into_boxed_slice()).with_parameter_modes(vec![mode].into_boxed_slice());
    let family_kind = env.store.arrow_kind(Box::new([kind]), KindId::TYPE);
    let form = env.store.nominal_form(declaration.clone(), family_kind);
    env.declarations.insert(DeclarationTypeInfo {
        declaration: declaration.clone(),
        form,
        class_object_type: env.store.class_object_type(declaration.clone()),
        kind: family_kind,
        binding: DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(GenericBinderHeader::from_signature(&env.store, &signature)),
        generic_signature: Some(signature),
        supertype_template: None,
    });
    env.resolver.insert(name, declaration);
}

#[test]
fn zero_intrinsic_applications_select_complete_pack_and_record_row_domains() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(tuple) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &application("Tuple", vec![]),
        &mut diagnostics,
    ) else {
        panic!("Tuple<> should select an empty CompleteTypePack: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(tuple, env.store.unit());

    diagnostics.clear();
    let TypeFormResolution::Ready(record) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &application("Record", vec![]),
        &mut diagnostics,
    ) else {
        panic!("Record<> should select a closed-empty RecordFields row: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(matches!(env.store.get(record), TypeData::Record(row) if {
        let row = env.store.record_row(*row);
        row.fields.is_empty() && row.tail == RecordRowTail::Closed
    }));
}

#[test]
fn zero_user_defined_variadic_applications_retain_the_declared_argument_domains() {
    let mut env = setup();
    install_variadic_family(&mut env, "PackFamily", KindId::COMPLETE_TYPE_PACK, GenericParameterMode::CompletePack);
    install_variadic_family(&mut env, "RowFamily", KindId::RECORD_ROW, GenericParameterMode::LabeledPack);

    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(pack_family) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &application("PackFamily", vec![]),
        &mut diagnostics,
    ) else {
        panic!("zero CompleteTypePack contribution should apply: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(matches!(env.store.get(pack_family), TypeData::Applied { arguments, .. } if
        matches!(arguments.as_ref(), [StaticArgument::Term(pack)] if matches!(env.store.get(*pack), TypeData::CompleteTypePack { positionals, labeled } if {
            matches!(env.store.get(*positionals), TypeData::TypePack(elements) if elements.is_empty())
                && matches!(env.store.get(*labeled), TypeData::LabeledTypePack(elements) if elements.is_empty())
        }))
    ));

    diagnostics.clear();
    let TypeFormResolution::Ready(row_family) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &application("RowFamily", vec![]),
        &mut diagnostics,
    ) else {
        panic!("zero RecordFields contribution should apply: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(matches!(env.store.get(row_family), TypeData::Applied { arguments, .. } if {
        matches!(arguments.as_ref(), [StaticArgument::RecordRow(row)] if {
            let row = env.store.record_row(*row);
            row.fields.is_empty() && row.tail == RecordRowTail::Closed
        })
    }));
}
