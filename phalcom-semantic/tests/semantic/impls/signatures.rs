use phalcom_common::selector::SelectorBase;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::declaration_type::DeclaredTypeState;
use phalcom_semantic::declarations::{DeclarationTypeTable, NominalDeclarationHeader};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{DeclarationId, DispatchSide, ImplId, ImplLocalId};
use phalcom_semantic::impls::build_inherent_impl_contribution;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::{TypeData, TypeStore};
use phalcom_semantic::CheckingContext;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn universe_module() -> ModuleId {
    ModuleId::universe_root()
}

fn setup_resolver() -> SimpleTypeResolver {
    let mut resolver = SimpleTypeResolver::new();
    let u = universe_module();
    resolver.insert("Int", DeclarationId::new(u.clone(), "Int".into()));
    resolver.insert("Bool", DeclarationId::new(u.clone(), "Bool".into()));
    resolver.insert("Float", DeclarationId::new(u.clone(), "Float".into()));
    resolver.insert("String", DeclarationId::new(u, "String".into()));
    resolver
}

fn register_nominal(
    store: &mut TypeStore,
    declarations: &mut DeclarationTypeTable,
    decl: DeclarationId,
    generic_signature: Option<GenericSignature>,
) {
    let header = NominalDeclarationHeader::from_signature(store, decl, generic_signature);
    declarations.insert(header.into_type_info(None));
}

#[test]
fn test_permutation_signature_canonicalization() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let pair_decl = DeclarationId::new(module.clone(), "Pair".into());
    let param_x = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 0, "X", KindId::TYPE));
    let param_y = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 1, "Y", KindId::TYPE));
    let pair_sig = GenericSignature::new(
        TypeParameterOwner::Declaration(pair_decl.clone()),
        vec![param_x, param_y].into_boxed_slice(),
    );
    register_nominal(&mut store, &mut declarations, pair_decl.clone(), Some(pair_sig));

    let mut resolver = setup_resolver();
    resolver.insert("Pair", pair_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl<A, B> Pair<B, A> {\n  first_val() -> A { }\n  second_val() -> B { }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contribution = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_stmt);
    assert_eq!(contribution.target_declaration().name.as_ref(), "Pair");
    assert_eq!(contribution.members.len(), 2);

    // first_val -> A in impl<A, B> Pair<B, A>
    // Pair<B, A> means 0th arg is B (maps to X), 1st arg is A (maps to Y)
    // Therefore return type A must be canonicalized to declaration parameter Y!
    let first_member = &contribution.members[0];
    assert!(matches!(&first_member.callable.selector.base, SelectorBase::Named(name) if name == "first_val"));
    if let DeclaredTypeState::Known(TypeTerm::Canonical(ret_ty)) = first_member.signature.declared_return.state {
        match ctx.store.get(ret_ty) {
            TypeData::Parameter(p_id) => {
                assert_eq!(*p_id, param_y, "first_val return type should be declaration parameter Y");
                let p_data = ctx.store.type_parameter(*p_id);
                assert!(matches!(p_data.owner, TypeParameterOwner::Declaration(_)), "owner must be declaration, not impl");
            }
            other => panic!("expected parameter type, got {:?}", other),
        }
    } else {
        panic!("expected known return type");
    }

    // second_val -> B in impl<A, B> Pair<B, A>
    // Return type B must be canonicalized to declaration parameter X!
    let second_member = &contribution.members[1];
    assert!(matches!(&second_member.callable.selector.base, SelectorBase::Named(name) if name == "second_val"));
    if let DeclaredTypeState::Known(TypeTerm::Canonical(ret_ty)) = second_member.signature.declared_return.state {
        match ctx.store.get(ret_ty) {
            TypeData::Parameter(p_id) => {
                assert_eq!(*p_id, param_x, "second_val return type should be declaration parameter X");
                let p_data = ctx.store.type_parameter(*p_id);
                assert!(matches!(p_data.owner, TypeParameterOwner::Declaration(_)), "owner must be declaration, not impl");
            }
            other => panic!("expected parameter type, got {:?}", other),
        }
    } else {
        panic!("expected known return type");
    }
}

#[test]
fn test_class_side_inherent_member_contribution() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let math_decl = DeclarationId::new(module.clone(), "Math".into());
    register_nominal(&mut store, &mut declarations, math_decl.clone(), None);

    let mut resolver = setup_resolver();
    resolver.insert("Math", math_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl Math {\n  @class\n  pi() -> Float { 3.14 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contribution = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_stmt);
    assert_eq!(contribution.members.len(), 1);
    assert_eq!(contribution.members[0].callable.side, DispatchSide::Class);
    assert_eq!(contribution.members[0].callable.declaration_owner().name.as_ref(), "Math");
}

#[test]
fn test_constructor_in_impl_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone(), None);

    let mut resolver = setup_resolver();
    resolver.insert("User", user_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl User {\n  @constructor\n  make() { }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contribution = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_stmt);
    assert!(contribution.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplConstructorUnsupported));
}

#[test]
fn test_bodyless_in_impl_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone(), None);

    let mut resolver = setup_resolver();
    resolver.insert("User", user_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl User {\n  name() -> String\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contribution = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_stmt);
    assert!(contribution.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplBodylessMemberUnsupported));
}
