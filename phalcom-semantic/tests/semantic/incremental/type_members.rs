//! LANG007.C1 incremental and cold-parity tests for type-level structural operations.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::ModuleId;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;

fn module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(807),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("type_members").unwrap()]),
    )
}

fn getter_callable(module: &ModuleId, owner: &str, name: &str) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), owner.into()),
        Selector::getter(name).expect("valid getter selector"),
        DispatchSide::Instance,
    )
}

#[test]
fn canonical_universe_intrinsic_type_members_match_evaluator_catalog() {
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_semantic::types::type_member::{IntrinsicTypeOperation, lookup_intrinsic_type_operation};

    // SemanticWorkspaceSession bootstraps these products by parsing the actual
    // canonical Universe sources and passing each TypeMemberDef through the
    // production declaration builder; this is deliberately distinct from the
    // hand-installed SimpleTypeResolver catalog used by low-level tests.
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .update(single_module_input(module(), "class Probe {}\n", 1))
        .expect("semantic test update should succeed");
    let snapshot = update.snapshot;
    let mut store = (*snapshot.store).clone();

    for operation in IntrinsicTypeOperation::ALL {
        let mut matching = snapshot
            .type_member_declarations
            .iter()
            .filter(|(_, product)| product.signature.intrinsic == Some(operation));
        let (member_id, product) = matching
            .next()
            .unwrap_or_else(|| panic!("canonical Universe source is missing {}", operation.name()));
        assert!(
            matching.next().is_none(),
            "canonical Universe source declares {} more than once",
            operation.name()
        );

        let owner = snapshot
            .declarations
            .intrinsic_type_owner(&member_id.owner)
            .unwrap_or_else(|| panic!("{} is not attached to a canonical intrinsic owner", operation.name()));
        assert_eq!(lookup_intrinsic_type_operation(owner, &member_id.selector), Some(operation));

        let (expected_parameters, expected_result) = operation.signature_kinds(&mut store);
        let actual_parameters = product
            .signature
            .parameters
            .iter()
            .map(|parameter| match &parameter.domain {
                phalcom_semantic::types::type_member::TypeMemberDomain::Kind(kind) => *kind,
                phalcom_semantic::types::type_member::TypeMemberDomain::ValueType(_) => panic!("unexpected ValueType in intrinsic kind test"),
            })
            .collect::<Vec<_>>();
        let phalcom_semantic::types::type_member::TypeMemberDomain::Kind(actual_result) = &product.signature.result_domain else {
            panic!("expected Kind result domain");
        };
        assert_eq!(actual_parameters.as_slice(), expected_parameters.as_ref(), "{} parameters", operation.name());
        assert_eq!(*actual_result, expected_result, "{} result", operation.name());
    }

    // Conversely, every actual source declaration marked `intrinsic` on an
    // intrinsic owner must map to precisely the evaluator selected by its
    // canonical owner+selector. There is no separate private evaluator catalog:
    // `IntrinsicTypeOperation::ALL` is the complete source-selectable set.
    let intrinsic_products = snapshot
        .type_member_declarations
        .values()
        .filter(|product| product.signature.implementation == ImplementationKind::Intrinsic)
        .collect::<Vec<_>>();
    for product in intrinsic_products {
        let owner = snapshot
            .declarations
            .intrinsic_type_owner(&product.id.owner)
            .expect("source intrinsic member owner must be canonical");
        let operation = lookup_intrinsic_type_operation(owner, &product.id.selector)
            .unwrap_or_else(|| panic!("source intrinsic member {} has no evaluator", product.id.selector.name()));
        assert_eq!(product.signature.intrinsic, Some(operation));
        assert!(
            IntrinsicTypeOperation::ALL.contains(&operation),
            "compiler-private operations cannot be published as source intrinsic members"
        );
    }
}

#[test]
fn source_type_members_publish_checked_signatures_without_changing_namespace_selection() {
    use phalcom_semantic::associated::{AssociatedFamilyKind, AssociatedMemberId};
    use phalcom_semantic::types::type_member::{
        TypeMemberApplicationError, TypeMemberArgument, TypeMemberDeclarationId, TypeMemberSelector, validate_type_member_arguments,
    };
    use std::sync::Arc;

    let module = module();
    let source_a =
        "type data Shape { Scalar }\ntype data Other { Point }\nclass Geometry {\n  type Transform<S: Shape>: Shape\n  type Preserve<T: Shape>: Shape\n}\n";
    let source_b =
        "type data Shape { Scalar }\ntype data Other { Point }\nclass Geometry {\n  type Transform<S: Shape>: Other\n  type Preserve<T: Shape>: Shape\n}\n";
    let mut session = SemanticWorkspaceSession::new();

    let initial = session
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);
    let owner = DeclarationId::new(module.clone(), "Geometry".into());
    let transform = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Transform"));
    let preserve = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Preserve"));
    let first_transform = initial.snapshot.type_member_declarations.get(&transform).expect("published Transform");
    let first_preserve = initial.snapshot.type_member_declarations.get(&preserve).expect("published Preserve");
    let shape_decl = DeclarationId::new(module.clone(), "Shape".into());
    let shape_form = initial.snapshot.declarations.form(&shape_decl).expect("Shape declaration form");
    let shape = initial.snapshot.store.kind_of(shape_form);
    assert_eq!(
        first_transform.signature.parameters[0].domain,
        phalcom_semantic::types::type_member::TypeMemberDomain::Kind(shape)
    );
    assert_eq!(
        first_transform.signature.result_domain,
        phalcom_semantic::types::type_member::TypeMemberDomain::Kind(shape)
    );
    let wrong_domain_form = initial.snapshot.declarations.form(&owner).expect("ordinary Type-domain owner form");
    let mut store = (*initial.snapshot.store).clone();
    assert!(matches!(
        validate_type_member_arguments(&mut store, &first_transform.signature, &[TypeMemberArgument::Type(wrong_domain_form)]),
        Err(TypeMemberApplicationError::ArgumentDomainMismatch { index: 0, expected, .. }) if expected == phalcom_semantic::types::type_member::TypeMemberDomain::Kind(shape)
    ));

    let surface = initial.snapshot.associated_surfaces.surfaces.get(&owner).expect("Geometry associated surface");
    let family = surface
        .families
        .get(&phalcom_common::selector::SelectorBase::Named("Transform".into()))
        .expect("Transform family");
    assert_eq!(family.kind, AssociatedFamilyKind::TypeMember);
    assert_eq!(family.members.as_ref(), &[AssociatedMemberId::TypeMember(transform.clone())]);

    let updated = session
        .update(single_module_input(module, source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let second_transform = updated.snapshot.type_member_declarations.get(&transform).expect("updated Transform");
    let second_preserve = updated.snapshot.type_member_declarations.get(&preserve).expect("retained Preserve");
    assert!(
        !Arc::ptr_eq(first_transform, second_transform),
        "changed signature must recompute its declaration product"
    );
    assert!(Arc::ptr_eq(first_preserve, second_preserve), "unrelated signature should be reused");
    let updated_family = updated
        .snapshot
        .associated_surfaces
        .surfaces
        .get(&owner)
        .unwrap()
        .families
        .get(&phalcom_common::selector::SelectorBase::Named("Transform".into()))
        .unwrap();
    assert_eq!(family.kind, updated_family.kind);
    assert_eq!(family.members, updated_family.members);
}

#[test]
fn source_type_member_projection_is_available_in_field_annotations() {
    use phalcom_semantic::types::store::TypeData;
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};

    let module = module();
    let source = "class Geometry {\n  type Wrap<T: Type>: Type\n}\nclass Consumer<T: Type> {\n  _value: Geometry::Wrap<T>\n}\n";
    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");

    assert!(
        !result.snapshot.has_errors(),
        "source member projection should be valid in a field annotation: {:?}",
        result.snapshot.diagnostics
    );
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let field = phalcom_semantic::identity::FieldId::new(consumer, "_value", phalcom_semantic::identity::DispatchSide::Instance);
    let field_signature = result.snapshot.field_signatures.get(&field).expect("published Consumer.value signature");
    let projection = field_signature
        .declared_type
        .canonical_type()
        .and_then(|field_type| match result.snapshot.store.get(field_type) {
            TypeData::TypeMemberProjection(projection) => Some(projection),
            _ => None,
        })
        .expect("field type should preserve its source type-member projection");
    let expected_member = TypeMemberDeclarationId::new(DeclarationId::new(module.clone(), "Geometry".into()), TypeMemberSelector::named("Wrap"));
    assert_eq!(projection.source_declaration.as_ref(), Some(&expected_member));
}

#[test]
fn source_type_member_projection_is_available_in_alias_annotations() {
    use phalcom_semantic::types::store::TypeData;
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};

    let module = module();
    let source = "class Geometry {\n  type Wrap<T: Type>: Type\n}\ntype Wrapped = Geometry::Wrap<Int>\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");

    assert!(
        !result.snapshot.has_errors(),
        "alias projection should resolve: {:#?}",
        result.snapshot.diagnostics
    );
    let alias = result
        .snapshot
        .type_aliases
        .get(&DeclarationId::new(module.clone(), "Wrapped".into()))
        .expect("published alias");
    let TypeData::TypeMemberProjection(projection) = result.snapshot.store.get(alias.form) else {
        panic!(
            "alias should preserve its source member projection, got {:?}",
            result.snapshot.store.get(alias.form)
        );
    };
    assert_eq!(
        projection.source_declaration.as_ref(),
        Some(&TypeMemberDeclarationId::new(
            DeclarationId::new(module, "Geometry".into()),
            TypeMemberSelector::named("Wrap")
        ))
    );
}

#[test]
fn source_type_member_projection_is_available_in_generic_header_annotations() {
    let module = module();
    let source = "class Geometry {\n  type Wrap<T: Type>: Type\n}\nclass Consumer<T: Type> where T == Geometry::Wrap<Int> {}\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");

    assert!(
        !result.snapshot.has_errors(),
        "generic-header projection should resolve through the published surface: {:#?}",
        result.snapshot.diagnostics
    );
}

#[test]
fn source_type_member_projection_is_available_in_callable_parameter_and_return_annotations() {
    use phalcom_common::selector::{Selector, SelectorSlot};
    use phalcom_semantic::types::store::TypeData;
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};

    let module = module();
    let source = r#"
class Geometry {
  type Wrap<T: Type>: Type
}
class Consumer {
  transform(_ value: Geometry::Wrap<Int>) -> Geometry::Wrap<Int> { value }
}
"#;
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    assert!(
        !result.snapshot.has_errors(),
        "callable annotations should retain type-member types: {:?}",
        result.snapshot.diagnostics
    );

    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let selector = Selector::method("transform", [SelectorSlot::Positional]).expect("method selector");
    let callable = CallableId::new(consumer, selector, DispatchSide::Instance);
    let signature = result.snapshot.callable_signatures.get(&callable).expect("published transform signature");
    let expected_member = TypeMemberDeclarationId::new(DeclarationId::new(module, "Geometry".into()), TypeMemberSelector::named("Wrap"));

    for ty in [
        signature.parameters[0].declared_type.canonical_type().expect("parameter annotation"),
        signature.declared_return.canonical_type().expect("return annotation"),
    ] {
        let TypeData::TypeMemberProjection(projection) = result.snapshot.store.get(ty) else {
            panic!(
                "callable annotation should preserve the canonical projection, got {:?}",
                result.snapshot.store.get(ty)
            );
        };
        assert_eq!(projection.source_declaration.as_ref(), Some(&expected_member));
    }
}

#[test]
fn alias_application_tracks_nominal_header_domain_edits_and_matches_cold_analysis() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let source_a = "type data Shape { Scalar }\ntype data Other { Point }\ndata Tensor<T: Type, S: Shape>()\ntype V: Type = Tensor<Int, Shape::Scalar>\ntype Unrelated = Int\n";
    let source_b = "type data Shape { Scalar }\ntype data Other { Point }\ndata Tensor<T: Type, S: Other>()\ntype V: Type = Tensor<Int, Shape::Scalar>\ntype Unrelated = Int\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let unrelated = DeclarationId::new(module.clone(), "Unrelated".into());
    let unrelated_form = first.snapshot.type_aliases.get(&unrelated).expect("unrelated alias").form;
    assert!(first.snapshot.type_aliases.get(&DeclarationId::new(module.clone(), "V".into())).is_some());

    let updated = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold
        .update(single_module_input(module.clone(), source_b, 1))
        .expect("semantic test update should succeed");
    let codes =
        |result: &phalcom_semantic::session::SemanticWorkspaceUpdate| result.snapshot.all_diagnostics().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    assert_eq!(codes(&updated), codes(&cold_result), "header edit must match cold diagnostics");
    assert!(codes(&updated).contains(&DiagnosticCode::ApplicationArgumentKindMismatch));
    assert_eq!(
        updated.snapshot.type_aliases.get(&unrelated).expect("retained unrelated alias").form,
        unrelated_form
    );
}

#[test]
fn static_case_catalog_distinguishes_singleton_from_empty_constructor() {
    use phalcom_common::selector::Selector;
    use phalcom_semantic::identity::StaticDomainDeclarationId;

    let module = module();
    let source = "type data Shape { Scalar; Empty() }\ntype DefaultShape: Shape = Shape::Scalar\ntype EmptyShape: Shape = Shape::Empty()\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    assert!(!result.snapshot.has_errors(), "diagnostics: {:?}", result.snapshot.diagnostics);
    let domain = StaticDomainDeclarationId::Nominal(DeclarationId::new(module.clone(), "Shape".into()));
    let catalog = result.snapshot.static_case_catalogs.get(&domain).expect("published catalog");
    assert!(catalog.select(&Selector::getter("Scalar").unwrap()).is_some());
    assert!(catalog.select(&Selector::method("Empty", []).unwrap()).is_some());
    assert!(catalog.select(&Selector::getter("Empty").unwrap()).is_none());
    let empty = catalog.select(&Selector::method("Empty", []).unwrap()).unwrap();
    assert!(
        result.snapshot.static_case_signatures.contains_key(&empty.id),
        "zero-arity constructor still requires a checked signature"
    );
    assert!(result.snapshot.type_aliases.contains_key(&DeclarationId::new(module, "EmptyShape".into())));
}

#[test]
fn recursive_static_domain_payload_depends_on_the_domain_header_only() {
    use phalcom_common::selector::Selector;
    use phalcom_semantic::identity::StaticDomainDeclarationId;

    let module = module();
    let source = "type data Tree { Leaf; Branch(_ subtree: Tree) }\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    assert!(
        !result.snapshot.has_errors(),
        "recursive domain payload should be legal: {:?}",
        result.snapshot.diagnostics
    );
    let domain = StaticDomainDeclarationId::Nominal(DeclarationId::new(module, "Tree".into()));
    let catalog = result.snapshot.static_case_catalogs.get(&domain).expect("Tree case catalog");
    let branch = catalog
        .select(&Selector::method("Branch", [phalcom_common::selector::SelectorSlot::Positional]).unwrap())
        .unwrap();
    assert!(result.snapshot.static_case_signatures.contains_key(&branch.id));
}

#[test]
fn alias_and_static_case_signature_cycle_is_rejected_as_one_component() {
    use phalcom_semantic::diagnostic::DiagnosticCode;
    use phalcom_semantic::identity::StaticDomainDeclarationId;

    let module = module();
    let source = "type data Shape { Scalar; Wrap(_ value: Alias) }\ntype Alias: Shape = Shape::Wrap<Shape::Scalar>(_)\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    let cycles = result
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::TypeSignatureCycle)
        .collect::<Vec<_>>();
    assert_eq!(
        cycles.len(),
        1,
        "expected one mixed signature-cycle diagnostic: {:?}",
        result.snapshot.diagnostics
    );
    let domain = StaticDomainDeclarationId::Nominal(DeclarationId::new(module.clone(), "Shape".into()));
    let catalog = result
        .snapshot
        .static_case_catalogs
        .get(&domain)
        .expect("declaration catalog survives signature cycle");
    let wrap = catalog
        .cases
        .iter()
        .find(|case| case.id.selector.base == phalcom_common::selector::SelectorBase::Named("Wrap".into()))
        .unwrap();
    assert!(
        !result.snapshot.static_case_signatures.contains_key(&wrap.id),
        "cyclic checked signature must be withheld"
    );
    assert!(!result.snapshot.type_aliases.contains_key(&DeclarationId::new(module, "Alias".into())));
}

#[test]
fn associated_generic_static_case_signature_dependencies_join_the_cycle_graph() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let source = "class Container<T: Type> { type data State { Ready; Box(_ value: Alias) } }\ntype Alias: Container<Int>::State = Container<Int>::State::Box<Container<Int>::State::Ready>(_)\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module, source, 1))
        .expect("semantic test update should succeed");
    let cycles = result
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::TypeSignatureCycle)
        .collect::<Vec<_>>();
    assert_eq!(
        cycles.len(),
        1,
        "associated generic owner must select the same signature graph node: {:?}",
        result.snapshot.diagnostics
    );
}

#[test]
fn static_case_rename_invalidates_alias_and_matches_cold_analysis() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let before = "type data Shape { Scalar }\ntype DefaultShape: Shape = Shape::Scalar\n";
    let after = "type data Shape { Vector }\ntype DefaultShape: Shape = Shape::Scalar\n";
    let mut incremental = SemanticWorkspaceSession::new();
    assert!(
        !incremental
            .update(single_module_input(module.clone(), before, 1))
            .expect("semantic test update should succeed")
            .snapshot
            .has_errors()
    );
    let updated = incremental
        .update(single_module_input(module.clone(), after, 2))
        .expect("semantic test update should succeed");
    let cold = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), after, 1))
        .expect("semantic test update should succeed");
    let codes = |snapshot: &phalcom_semantic::SemanticSnapshot| snapshot.all_diagnostics().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    assert_eq!(codes(&updated.snapshot), codes(&cold.snapshot));
    assert!(codes(&updated.snapshot).contains(&DiagnosticCode::AnnotationUnresolved));
    assert!(!updated.snapshot.type_aliases.contains_key(&DeclarationId::new(module, "DefaultShape".into())));
}

#[test]
fn associated_static_case_alias_uses_domain_catalog() {
    use phalcom_common::selector::Selector;
    use phalcom_semantic::identity::{AssociatedStaticDomainDeclarationId, StaticDomainDeclarationId};

    let module = module();
    let source = "class Parser { type data State { Start; Done } }\ntype Initial = Parser::State::Start\n";
    let result = SemanticWorkspaceSession::new()
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    println!("parsed: {:?}", phalcom_ast::parse(source, 0).errors);
    println!(
        "statements: {:?}",
        result.snapshot.sources.get(&module).map(|unit| unit.program.statements.len())
    );
    assert!(!result.snapshot.has_errors(), "diagnostics: {:?}", result.snapshot.diagnostics);
    assert!(
        result.snapshot.type_aliases.contains_key(&DeclarationId::new(module.clone(), "Initial".into())),
        "aliases: {:?}; diagnostics: {:?}",
        result.snapshot.type_aliases,
        result.snapshot.diagnostics,
    );
    let domain = StaticDomainDeclarationId::Associated(AssociatedStaticDomainDeclarationId::new(DeclarationId::new(module, "Parser".into()), "State"));
    assert!(
        result
            .snapshot
            .static_case_catalogs
            .get(&domain)
            .unwrap()
            .select(&Selector::getter("Start").unwrap())
            .is_some()
    );
}

#[test]
fn alias_and_type_member_signature_cycle_is_rejected_as_one_component() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let source = "class Geometry {\n  type Wrap<T: Shape>: Shape\n}\ntype Shape = Geometry::Wrap<Shape>\n";
    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");

    let diagnostics = result.snapshot.diagnostics.get(&module).expect("module diagnostics");
    let cycles = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::TypeSignatureCycle)
        .collect::<Vec<_>>();
    assert_eq!(cycles.len(), 1, "one deterministic cycle diagnostic per component: {diagnostics:#?}");
    assert!(cycles[0].message.contains("Geometry"));
    assert!(cycles[0].message.contains("Shape"));
    assert!(!result.snapshot.type_aliases.contains_key(&DeclarationId::new(module.clone(), "Shape".into())));
    let member = phalcom_semantic::types::type_member::TypeMemberDeclarationId::new(
        DeclarationId::new(module, "Geometry".into()),
        phalcom_semantic::types::type_member::TypeMemberSelector::named("Wrap"),
    );
    assert!(!result.snapshot.type_member_declarations.contains_key(&member));
}

#[test]
fn associated_surface_rejects_type_member_collision_with_owner_behavior() {
    use phalcom_common::selector::SelectorBase;
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let source = "type data Shape { Scalar }\nclass Geometry {\n  type Transform<S: Shape>: Shape\n  Transform() -> Int { 1 }\n}\n";
    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    let owner = DeclarationId::new(module.clone(), "Geometry".into());
    let surface = result.snapshot.associated_surfaces.surfaces.get(&owner).expect("published owner surface");
    assert!(!surface.families.contains_key(&SelectorBase::Named("Transform".into())));
    assert!(
        result.snapshot.diagnostics.get(&module).is_some_and(|diagnostics| diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::EnumFamilyCategoryConflict })),
        "expected same-owner type-member collision diagnostic: {:?}",
        result.snapshot.diagnostics
    );
}

#[test]
fn associated_collision_introduction_and_removal_refresh_incrementally_with_cold_parity() {
    use phalcom_common::selector::SelectorBase;
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let source_a = "type data Shape { Scalar }\nclass Geometry {\n  type Transform<S: Shape>: Shape\n}\n";
    let source_b = "type data Shape { Scalar }\nclass Geometry {\n  type Transform<S: Shape>: Shape\n  Transform() -> Int { 1 }\n}\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let revision_a = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    let revision_b = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    let revision_c = incremental
        .update(single_module_input(module.clone(), source_a, 3))
        .expect("semantic test update should succeed");

    let owner = DeclarationId::new(module.clone(), "Geometry".into());
    let family_key = SelectorBase::Named("Transform".into());
    assert!(revision_a.snapshot.associated_surfaces.surfaces[&owner].families.contains_key(&family_key));
    assert!(!revision_b.snapshot.associated_surfaces.surfaces[&owner].families.contains_key(&family_key));
    assert!(revision_c.snapshot.associated_surfaces.surfaces[&owner].families.contains_key(&family_key));

    let codes = |result: &phalcom_semantic::session::SemanticWorkspacePublication| {
        result
            .snapshot
            .diagnostics
            .get(&module)
            .into_iter()
            .flat_map(|diagnostics| diagnostics.iter())
            .map(|diagnostic| (diagnostic.code, diagnostic.message.clone(), diagnostic.primary_range))
            .collect::<Vec<_>>()
    };
    assert!(
        codes(&revision_b)
            .iter()
            .any(|(code, _, _)| *code == DiagnosticCode::EnumFamilyCategoryConflict)
    );
    let mut cold_b = SemanticWorkspaceSession::new();
    let cold_b = cold_b
        .update(single_module_input(module.clone(), source_b, 1))
        .expect("semantic test update should succeed");
    assert_eq!(codes(&revision_b), codes(&cold_b), "collision diagnostics should match cold analysis");

    let mut cold_c = SemanticWorkspaceSession::new();
    let cold_c = cold_c
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert_eq!(codes(&revision_c), codes(&cold_c), "collision removal should match cold analysis");
    assert!(codes(&revision_c).is_empty());
}

#[test]
fn canonical_duplicate_type_member_selectors_are_not_published() {
    use phalcom_common::selector::SelectorBase;
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};

    let module = module();
    let source = "class Geometry {\n  type At<I: Int>: Type\n  type [I: Int]: Type\n}\n";
    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    let owner = DeclarationId::new(module.clone(), "Geometry".into());
    let member_id = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::Subscript);
    assert!(!result.snapshot.type_member_declarations.contains_key(&member_id));
    let surface = result.snapshot.associated_surfaces.surfaces.get(&owner).expect("published Geometry surface");
    assert!(!surface.families.contains_key(&SelectorBase::Subscript));
}

#[test]
fn named_at_and_subscript_have_one_canonical_source_selector_identity() {
    use phalcom_ast::ast::{ClassMember, Statement};
    use phalcom_semantic::types::type_member::TypeMemberSelector;

    let parsed = phalcom_ast::parse("class Named { type At<I: Int>: Type }\nclass Indexed { type [I: Int]: Type }\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let selectors: Vec<_> = parsed
        .program
        .statements
        .iter()
        .filter_map(|statement| {
            let Statement::Class(class) = statement else { return None };
            class.members.iter().find_map(|member| match member {
                ClassMember::TypeMember(member) => Some(TypeMemberSelector::from_syntax(&member.selector)),
                _ => None,
            })
        })
        .collect();
    assert_eq!(selectors, vec![TypeMemberSelector::Subscript, TypeMemberSelector::Subscript]);
}

#[test]
fn wrong_user_static_domain_argument_is_rejected() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = module();
    let source = "type data Shape { Scalar }\ndata Tensor<S: Shape>()\ntype Bad = Tensor<Int>\n";
    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .update(single_module_input(module, source, 1))
        .expect("semantic test update should succeed");
    assert!(result.snapshot.has_errors(), "mismatched Shape argument should be rejected");
    assert!(
        result
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ApplicationArgumentKindMismatch),
        "expected a domain mismatch diagnostic: {:?}",
        result.snapshot.diagnostics
    );
}

#[test]
fn associated_static_domain_case_edit_invalidates_and_matches_cold_analysis() {
    use std::sync::Arc;

    let module = module();
    let source_a = "data Packet(bytes: Int)\nimpl Packet { type data State { Unread; Parsed } }\nclass Geometry { type Keep<T: Type>: Type }\n";
    let source_b = "data Packet(bytes: Int)\nimpl Packet { type data State { Unread; Ready } }\nclass Geometry { type Keep<T: Type>: Type }\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);

    let packet = DeclarationId::new(module.clone(), "Packet".into());
    let state = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(packet.clone(), "State");
    let first_state = first.snapshot.associated_static_domains.get(&state).expect("initial State product");
    let geometry = DeclarationId::new(module.clone(), "Geometry".into());
    let keep =
        phalcom_semantic::types::type_member::TypeMemberDeclarationId::new(geometry, phalcom_semantic::types::type_member::TypeMemberSelector::named("Keep"));
    let first_keep = first.snapshot.type_member_declarations.get(&keep).expect("unrelated Keep product");

    let updated = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let updated_state = updated.snapshot.associated_static_domains.get(&state).expect("updated State product");
    assert!(
        !Arc::ptr_eq(first_state, updated_state),
        "case edit must recompute its associated-domain product"
    );
    assert_ne!(first_state, updated_state, "renamed case must alter the published constructor set");
    assert_eq!(
        first_keep.signature,
        updated.snapshot.type_member_declarations.get(&keep).expect("updated Keep product").signature,
        "an unrelated member's checked signature must remain unchanged"
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold
        .update(single_module_input(module.clone(), source_b, 1))
        .expect("semantic test update should succeed");
    assert!(!cold_result.snapshot.has_errors(), "cold diagnostics: {:?}", cold_result.snapshot.diagnostics);
    let constructor_facts = |snapshot: &phalcom_semantic::SemanticSnapshot,
                             product: &phalcom_semantic::db::product::AssociatedStaticDomainDeclarationProduct| {
        product
            .constructors
            .iter()
            .map(|constructor| {
                let metadata = snapshot.store.static_constructor(*constructor).expect("constructor metadata");
                (metadata.declaration.clone(), metadata.case_name.clone(), metadata.parameter_kinds.clone())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        constructor_facts(&updated.snapshot, updated_state),
        constructor_facts(
            &cold_result.snapshot,
            cold_result.snapshot.associated_static_domains.get(&state).expect("cold State product")
        ),
        "store-local constructor ids must materialize to the same stable declaration facts"
    );
    assert_eq!(
        updated.snapshot.associated_surfaces.surfaces.get(&packet),
        cold_result.snapshot.associated_surfaces.surfaces.get(&packet),
        "associated namespace must match cold analysis after the case edit"
    );
}

#[test]
fn associated_static_case_signature_edit_invalidates_and_matches_cold_analysis() {
    use phalcom_semantic::identity::{AssociatedStaticDomainDeclarationId, StaticDomainDeclarationId};
    use phalcom_semantic::static_case::StaticCaseShape;
    use std::sync::Arc;

    let module = module();
    let source_a = "type data Shape { Scalar }\ntype data Other { Point }\nclass Parser { type data State { Value(_ shape: Shape); Stay(_ shape: Shape) } }\n";
    let source_b = "type data Shape { Scalar }\ntype data Other { Point }\nclass Parser { type data State { Value(_ shape: Other); Stay(_ shape: Shape) } }\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let initial = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let domain = StaticDomainDeclarationId::Associated(AssociatedStaticDomainDeclarationId::new(
        DeclarationId::new(module.clone(), "Parser".into()),
        "State",
    ));
    let case = initial.snapshot.static_case_catalogs[&domain]
        .cases
        .iter()
        .find(|case| matches!(case.shape, StaticCaseShape::Constructor { .. }))
        .expect("Value constructor case")
        .id
        .clone();
    let first_signature = initial.snapshot.static_case_signatures.get(&case).expect("checked Value signature");
    let sibling_id = initial.snapshot.static_case_catalogs[&domain]
        .cases
        .iter()
        .find(|case| case.id.selector.base == phalcom_common::selector::SelectorBase::Named("Stay".into()))
        .expect("Stay constructor case")
        .id
        .clone();
    let sibling_signature = initial.snapshot.static_case_signatures.get(&sibling_id).expect("checked Stay signature");

    let updated = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let updated_signature = updated.snapshot.static_case_signatures.get(&case).expect("updated Value signature");
    assert!(
        !Arc::ptr_eq(first_signature, updated_signature),
        "payload kind edit must rebuild the signature product"
    );
    assert_ne!(first_signature.parameter_kinds, updated_signature.parameter_kinds, "payload domain must update");
    assert!(
        Arc::ptr_eq(
            sibling_signature,
            updated.snapshot.static_case_signatures.get(&sibling_id).expect("retained Stay signature")
        ),
        "an unrelated sibling case signature should be reused"
    );

    let cold = SemanticWorkspaceSession::new()
        .update(single_module_input(module, source_b, 1))
        .expect("semantic test update should succeed");
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);
    let cold_signature = cold.snapshot.static_case_signatures.get(&case).expect("cold Value signature");
    assert_eq!(
        updated_signature.parameter_kinds, cold_signature.parameter_kinds,
        "incremental signature must match cold analysis"
    );
}

#[test]
fn associated_static_case_payload_tracks_generic_owner_kind_edit_with_cold_parity() {
    use phalcom_semantic::identity::{AssociatedStaticDomainDeclarationId, StaticDomainDeclarationId};
    use phalcom_semantic::static_case::StaticCaseShape;
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::kind::KindData;
    use std::sync::Arc;

    let module = module();
    let source_type = "type data Shape { Scalar }\nclass Container<T: Type> { type data State { Box(_ value: T) } }\n";
    let source_shape = "type data Shape { Scalar }\nclass Container<T: Shape> { type data State { Box(_ value: T) } }\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let initial = incremental
        .update(single_module_input(module.clone(), source_type, 1))
        .expect("semantic test update should succeed");
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let domain = StaticDomainDeclarationId::Associated(AssociatedStaticDomainDeclarationId::new(
        DeclarationId::new(module.clone(), "Container".into()),
        "State",
    ));
    let case = initial.snapshot.static_case_catalogs[&domain]
        .cases
        .iter()
        .find(|case| matches!(case.shape, StaticCaseShape::Constructor { .. }))
        .expect("Box constructor case")
        .id
        .clone();
    let first_signature = initial.snapshot.static_case_signatures.get(&case).expect("Type payload signature");
    assert_eq!(first_signature.parameter_kinds.as_ref(), &[KindId::TYPE]);

    let updated = incremental
        .update(single_module_input(module.clone(), source_shape, 2))
        .expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let updated_signature = updated.snapshot.static_case_signatures.get(&case).expect("Shape payload signature");
    assert!(
        !Arc::ptr_eq(first_signature, updated_signature),
        "owner header edit must rebuild the payload signature"
    );
    assert_ne!(updated_signature.parameter_kinds.as_ref(), &[KindId::TYPE]);
    assert!(matches!(
        updated.snapshot.store.get_kind(updated_signature.parameter_kinds[0]),
        KindData::StaticDomain(_)
    ));

    let cold = SemanticWorkspaceSession::new()
        .update(single_module_input(module, source_shape, 1))
        .expect("semantic test update should succeed");
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);
    let cold_signature = cold.snapshot.static_case_signatures.get(&case).expect("cold Shape payload signature");
    assert_eq!(
        updated_signature.parameter_kinds, cold_signature.parameter_kinds,
        "incremental and cold owner-kind facts must agree"
    );
}

#[test]
fn inherent_impl_static_case_signature_edit_invalidates_and_matches_cold_analysis() {
    use phalcom_semantic::identity::{AssociatedStaticDomainDeclarationId, StaticDomainDeclarationId};
    use phalcom_semantic::static_case::StaticCaseShape;

    let module = module();
    let source_a =
        "type data Shape { Scalar }\ntype data Other { Point }\ndata Packet(bytes: Int)\nimpl Packet { type data State { Value(_ shape: Shape) } }\n";
    let source_b =
        "type data Shape { Scalar }\ntype data Other { Point }\ndata Packet(bytes: Int)\nimpl Packet { type data State { Value(_ shape: Other) } }\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let domain = StaticDomainDeclarationId::Associated(AssociatedStaticDomainDeclarationId::new(
        DeclarationId::new(module.clone(), "Packet".into()),
        "State",
    ));
    let catalog = first.snapshot.static_case_catalogs.get(&domain).expect("impl-owned catalog");
    let case = catalog
        .cases
        .iter()
        .find(|case| matches!(case.shape, StaticCaseShape::Constructor { .. }))
        .expect("Value constructor")
        .id
        .clone();
    let signature = first.snapshot.static_case_signatures.get(&case).expect("initial signature");

    let updated = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let updated_signature = updated.snapshot.static_case_signatures.get(&case).expect("updated signature");
    assert_ne!(
        signature.parameter_kinds, updated_signature.parameter_kinds,
        "impl-owned payload domain must update"
    );

    let cold = SemanticWorkspaceSession::new()
        .update(single_module_input(module, source_b, 1))
        .expect("semantic test update should succeed");
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);
    assert_eq!(
        updated_signature.parameter_kinds,
        cold.snapshot.static_case_signatures.get(&case).expect("cold signature").parameter_kinds,
        "impl-owned signature must match cold analysis"
    );
}

#[test]
fn test_incremental_tuple_at_edit_matches_cold_session() {
    let module = module();
    let source_a = r#"
class Worker {
  first_element -> (Int, String)[0] {
    42
  }
}
"#;
    let source_b = r#"
class Worker {
  first_element -> (Int, String)[1] {
    "hello"
  }
}
"#;

    // 1. Initial incremental run
    let mut incremental = SemanticWorkspaceSession::new();
    let res_a = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!res_a.snapshot.has_errors(), "diagnostics: {:?}", res_a.snapshot.diagnostics);

    let callable = getter_callable(&module, "Worker", "first_element");
    let sig_a = res_a.snapshot.callable_signatures.get(&callable).expect("Worker.first_element signature A");
    let ret_a = sig_a.declared_return.canonical_type().expect("ret A");
    assert_eq!(res_a.snapshot.store.format_type(ret_a), "Int");

    // 2. Incremental update
    let res_b_inc = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!res_b_inc.snapshot.has_errors(), "diagnostics B inc: {:?}", res_b_inc.snapshot.diagnostics);
    let sig_b_inc = res_b_inc
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("Worker.first_element signature B inc");
    let ret_b_inc = sig_b_inc.declared_return.canonical_type().expect("ret B inc");
    assert_eq!(res_b_inc.snapshot.store.format_type(ret_b_inc), "String");

    // 3. Cold run on source_b
    let mut cold = SemanticWorkspaceSession::new();
    let res_b_cold = cold
        .update(single_module_input(module.clone(), source_b, 1))
        .expect("semantic test update should succeed");
    assert!(!res_b_cold.snapshot.has_errors(), "diagnostics B cold: {:?}", res_b_cold.snapshot.diagnostics);
    let sig_b_cold = res_b_cold
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("Worker.first_element signature B cold");
    let ret_b_cold = sig_b_cold.declared_return.canonical_type().expect("ret B cold");

    // 4. Parity assertion: semantic type format and diagnostics match
    assert_eq!(
        res_b_inc.snapshot.store.format_type(ret_b_inc),
        res_b_cold.snapshot.store.format_type(ret_b_cold),
        "Incremental and cold return types must have identical formatted types"
    );
    assert_eq!(
        res_b_inc.snapshot.has_errors(),
        res_b_cold.snapshot.has_errors(),
        "Diagnostic error presence must match"
    );
}

#[test]
fn test_incremental_tuple_map_edit_matches_cold_session() {
    let module = module();
    let source_a = r#"
class Transformer {
  mapped_tuple -> (Int, name: String)::Map<List> {
    ([1, 2], name: ["a", "b"])
  }
}
"#;
    let source_b = r#"
class Transformer {
  mapped_tuple -> (Int, name: String)::Map<Option> {
    (Some(1), name: Some("a"))
  }
}
"#;

    // 1. Initial incremental run
    let mut incremental = SemanticWorkspaceSession::new();
    let res_a = incremental
        .update(single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!res_a.snapshot.has_errors(), "diagnostics A: {:?}", res_a.snapshot.diagnostics);

    // 2. Incremental update
    let res_b_inc = incremental
        .update(single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!res_b_inc.snapshot.has_errors(), "diagnostics B inc: {:?}", res_b_inc.snapshot.diagnostics);

    // 3. Cold run
    let mut cold = SemanticWorkspaceSession::new();
    let res_b_cold = cold
        .update(single_module_input(module.clone(), source_b, 1))
        .expect("semantic test update should succeed");
    assert!(!res_b_cold.snapshot.has_errors(), "diagnostics B cold: {:?}", res_b_cold.snapshot.diagnostics);

    let callable = getter_callable(&module, "Transformer", "mapped_tuple");
    let sig_inc = res_b_inc.snapshot.callable_signatures.get(&callable).expect("sig inc");
    let sig_cold = res_b_cold.snapshot.callable_signatures.get(&callable).expect("sig cold");

    let ret_inc = sig_inc.declared_return.canonical_type().expect("ret inc");
    let ret_cold = sig_cold.declared_return.canonical_type().expect("ret cold");

    assert_eq!(
        res_b_inc.snapshot.store.format_type(ret_inc),
        res_b_cold.snapshot.store.format_type(ret_cold),
        "Incremental and cold mapped return types must match"
    );
    assert_eq!(res_b_inc.snapshot.store.format_type(ret_inc), "(Option<Int>, name: Option<String>)");
}
