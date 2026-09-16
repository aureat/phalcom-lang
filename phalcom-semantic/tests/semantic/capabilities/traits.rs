use crate::semantic::support::WorkspaceFixture;
use phalcom_ast::ast::{BehaviorMember, Statement};
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::body::{BodyAnalysisContext, CallableBodyRequest};
use phalcom_semantic::checker::{CallableAnalysisStatus, analyze_callable_body};
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::identity::{CallableOwnerId, DispatchSide, ImplId, ImplLocalId};
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::{TraitRef, TraitRefFormationError, TraitRequirementId};

#[test]
fn generic_trait_publishes_declaration_owned_header_without_nominal_type_entry() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Display<T> where T <: Object {
  render(_ value: T) -> T
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Display");
    let header = fixture.analysis.snapshot.trait_headers.get(&declaration).expect("trait header");
    let signature = header.generic_signature.as_ref().expect("generic trait signature");
    assert_eq!(signature.owner, TypeParameterOwner::Declaration(declaration.clone()));
    assert_eq!(signature.parameters.len(), 1);
    assert!(
        fixture.analysis.snapshot.declarations.get(&declaration).is_none(),
        "traits must not enter nominal type metadata"
    );
    assert_eq!(
        fixture
            .analysis
            .snapshot
            .source_index
            .declaration_source(&declaration)
            .map(|source| source.kind),
        Some(phalcom_semantic::source_index::SourceDeclarationKind::Trait)
    );

    let shard = fixture
        .analysis
        .snapshot
        .semantic_structure_shards
        .get(fixture.module("main"))
        .expect("semantic shard");
    assert!(
        shard
            .declarations
            .iter()
            .any(|blueprint| blueprint.id == declaration && blueprint.kind == phalcom_modules::DeclarationKind::Trait)
    );
}

#[test]
fn trait_ref_forms_non_generic_and_generic_contracts_with_canonical_arguments() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Marker {
  mark()
}

trait Display<T> where T <: Int {
  render(_ value: T) -> T
}
"#,
        )
        .analyze();

    let marker = fixture.decl("main", "Marker");
    let display = fixture.decl("main", "Display");
    let core = CoreDeclarationIds::default();
    let int = fixture.analysis.snapshot.declarations.form(&core.int).expect("Int type form");
    let mut store = (*fixture.analysis.snapshot.store).clone();

    let marker_ref = TraitRef::form(
        &mut store,
        &fixture.analysis.snapshot.trait_headers,
        fixture.analysis.snapshot.hierarchy.as_ref(),
        marker.clone(),
        &[],
    )
        .expect("non-generic trait reference");
    let display_ref = TraitRef::form(
        &mut store,
        &fixture.analysis.snapshot.trait_headers,
        fixture.analysis.snapshot.hierarchy.as_ref(),
        display.clone(),
        &[int],
    )
        .expect("generic trait reference");

    assert_eq!(marker_ref, TraitRef::new(marker, Box::<[phalcom_semantic::TypeId]>::default()));
    assert_eq!(display_ref, TraitRef::new(display, vec![int].into_boxed_slice()));
}

#[test]
fn trait_ref_rejects_wrong_category_arity_kind_and_constraint() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"class Concrete {
}

trait Display<T> where T <: Int {
  render(_ value: T) -> T
}
"#,
        )
        .analyze();
    let display = fixture.decl("main", "Display");
    let concrete = fixture.decl("main", "Concrete");
    let core = CoreDeclarationIds::default();
    let int = fixture.analysis.snapshot.declarations.form(&core.int).expect("Int type form");
    let list = fixture.analysis.snapshot.declarations.form(&core.list).expect("List type form");
    let string = fixture.analysis.snapshot.declarations.form(&core.string).expect("String type form");

    let form = |declaration, arguments: &[phalcom_semantic::TypeId]| {
        let mut store = (*fixture.analysis.snapshot.store).clone();
        TraitRef::form(
            &mut store,
            &fixture.analysis.snapshot.trait_headers,
            fixture.analysis.snapshot.hierarchy.as_ref(),
            declaration,
            arguments,
        )
    };

    assert!(matches!(form(concrete, &[]), Err(TraitRefFormationError::NotTrait(_))));
    assert!(matches!(
        form(display.clone(), &[]),
        Err(TraitRefFormationError::Arity { expected: 1, actual: 0, .. })
    ));
    assert!(matches!(
        form(display.clone(), &[int, int]),
        Err(TraitRefFormationError::Arity { expected: 1, actual: 2, .. })
    ));
    assert!(matches!(form(display.clone(), &[list]), Err(TraitRefFormationError::Kind { index: 0, .. })));
    let constraint_result = form(display, &[string]);
    assert!(
        matches!(constraint_result, Err(TraitRefFormationError::ConstraintUnsatisfied { index: 0 })),
        "unexpected constraint result: {constraint_result:?}"
    );
}

#[test]
fn trait_requirement_identity_is_stable_across_default_presence_and_separate_from_source_callable() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Display {
  render(_ value: Int) -> Int
}
"#,
        )
        .analyze();
    let display = fixture.decl("main", "Display");
    let selector = Selector::method("render", [SelectorSlot::Positional]).expect("render selector");
    let bodyless = TraitRequirementId::new(display.clone(), selector.clone(), DispatchSide::Instance);
    let bodyful = TraitRequirementId::new(display.clone(), selector, DispatchSide::Instance);
    let source = bodyless.source_callable();

    assert_eq!(bodyless, bodyful, "default presence must not alter requirement identity");
    assert_eq!(source.owner, CallableOwnerId::Declaration(display));
    assert_eq!(source.side, DispatchSide::Instance);
    assert_eq!(source.selector, bodyful.selector);
}

#[test]
fn traits_are_not_ordinary_inhabitable_type_forms() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Marker {
  mark()
}
"#,
        )
        .analyze();
    let marker = fixture.decl("main", "Marker");

    assert!(fixture.analysis.snapshot.trait_headers.contains(&marker));
    assert!(fixture.analysis.snapshot.declarations.get(&marker).is_none());
}

#[test]
fn trait_surface_publishes_all_behavior_shapes_before_default_body_analysis() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Surface {
  required(_ value: Int) -> Int
  name -> String
  value=(_ next: Int) -> Int
  [_ index: Int] -> Int
  fallback() -> Int { 1 }
}

"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Surface");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");

    assert_eq!(surface.members.len(), 5);
    for (selector, default_present) in [
        (Selector::method("required", [SelectorSlot::Positional]).unwrap(), false),
        (Selector::getter("name").unwrap(), false),
        (Selector::setter("value").unwrap(), false),
        (Selector::subscript_get([SelectorSlot::Positional]).unwrap(), false),
        (Selector::method("fallback", []).unwrap(), true),
    ] {
        let member = surface.get_by_selector(&selector, DispatchSide::Instance).expect("trait surface member");
        assert_eq!(member.default_present, default_present);
        assert_eq!(member.callable, member.requirement.source_callable());
        assert_eq!(member.signature.callable, member.callable);
        let analysis = fixture.analysis.snapshot.callable_analyses.get(&member.callable);
        assert_eq!(analysis.is_some(), default_present, "only bodyful trait members receive default analysis");
        if let Some(analysis) = analysis {
            assert!(analysis.diagnostics.is_empty(), "default body diagnostics: {:?}", analysis.diagnostics);
        }
    }
    assert!(
        fixture.analysis.snapshot.surfaces.get(&declaration).is_none(),
        "trait surface must not enter ordinary dispatch surfaces"
    );
}

#[test]
fn associated_type_surface_uses_trait_owned_identity_and_exact_binding_evidence() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterable {
  type Item
}

class Box<T> {
}

class Use {
  take(_ box: Box<Int>)
  take_text(_ box: Box<String>)
}

impl<T> Iterable for Box<T> {
  type Item = T
}
"#,
        )
        .analyze();
    assert!(!fixture.analysis.snapshot.has_errors(), "associated binding diagnostics: {:?}", fixture.analysis.snapshot.diagnostics);

    let trait_declaration = fixture.decl("main", "Iterable");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&trait_declaration).expect("trait surface");
    assert_eq!(surface.associated_types.len(), 1);
    let requirement = surface.associated_type_by_name("Item").expect("Item requirement");
    assert_eq!(requirement.requirement.owner, trait_declaration);
    assert_eq!(requirement.requirement.index, 0);
    assert_eq!(requirement.kind, phalcom_semantic::types::id::KindId::TYPE);
    assert!(surface.members.is_empty(), "associated types are not callable requirements");

    let plan = fixture.analysis.snapshot.conformance_witness_plans.values().next().expect("conformance plan");
    assert!(plan.associated_type_plan.failures.is_empty());
    assert_eq!(plan.associated_type_plan.bindings.len(), 1);

    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int form");
    let string = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().string).expect("String form");
    let use_decl = fixture.decl("main", "Use");
    let take = fixture
        .analysis
        .snapshot
        .surfaces()
        .get(&use_decl)
        .expect("Use surface")
        .instance
        .get_callable(&Selector::method("take", [SelectorSlot::Positional]).expect("take selector"))
        .expect("take signature");
    let target = take.parameters[0].ty.ty().expect("Box<Int> type");
    let take_text = fixture
        .analysis
        .snapshot
        .surfaces()
        .get(&use_decl)
        .expect("Use surface")
        .instance
        .get_callable(&Selector::method("take_text", [SelectorSlot::Positional]).expect("take_text selector"))
        .expect("take_text signature");
    let text_target = take_text.parameters[0].ty.ty().expect("Box<String> type");
    let trait_ref = TraitRef::new(trait_declaration, Vec::new().into_boxed_slice());
    let evidence = fixture.analysis.snapshot.conformance_evidence_for(target, &trait_ref).expect("exact evidence");
    let binding = evidence.associated_types.get(&requirement.requirement).expect("exact Item binding");
    assert_eq!(binding.value, int);
    let text_evidence = fixture.analysis.snapshot.conformance_evidence_for(text_target, &trait_ref).expect("exact String evidence");
    assert_eq!(text_evidence.associated_types[&requirement.requirement].value, string);
}

#[test]
fn associated_type_binding_validation_is_part_of_conformance_completeness() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterable {
  type Item
}

class Box {
}

impl Iterable for Box {
}
"#,
        )
        .analyze();
    let plan = fixture.analysis.snapshot.conformance_witness_plans.values().next().expect("conformance plan");
    let phalcom_semantic::impls::ConformanceCompleteness::Incomplete { failures } = &plan.completeness else {
        panic!("missing associated binding must be incomplete: {:?}", plan.completeness);
    };
    assert!(failures.iter().any(|failure| {
        matches!(failure, phalcom_semantic::impls::ConformanceFailure::AssociatedType(failure) if failure.reason.contains("no conformance binding"))
    }));
    let missing = failures.iter().find_map(|failure| {
        let phalcom_semantic::impls::ConformanceFailure::AssociatedType(failure) = failure else { return None };
        (failure.kind == phalcom_semantic::impls::AssociatedTypeBindingFailureKind::Missing).then_some(failure)
    }).expect("missing associated binding failure");
    assert_eq!(missing.source.module, fixture.decl("main", "Box").module);
    assert!(fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeMissing));
    assert!(fixture.analysis.snapshot.conformance_evidence_for(
        fixture.analysis.snapshot.declarations.form(&fixture.decl("main", "Box")).expect("Box form"),
        &TraitRef::new(fixture.decl("main", "Iterable"), Vec::new().into_boxed_slice()),
    ).is_none());
}

#[test]
fn associated_type_binding_is_rejected_in_inherent_impls() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"class Box {
}

impl Box {
  type Item = Int
}
"#,
        )
        .analyze();
    assert!(fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeInherentUnsupported));
}

#[test]
fn associated_type_names_are_trait_local_and_duplicate_declarations_are_rejected() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait First {
  type Item
  type Item
}

trait Second {
  type Item
}
"#,
        )
        .analyze();
    let first = fixture.analysis.snapshot.trait_surfaces.get(&fixture.decl("main", "First")).expect("First surface");
    let second = fixture.analysis.snapshot.trait_surfaces.get(&fixture.decl("main", "Second")).expect("Second surface");
    assert_eq!(first.associated_types.len(), 1);
    assert_eq!(second.associated_types.len(), 1);
    assert_ne!(
        first.associated_type_by_name("Item").expect("First Item").requirement,
        second.associated_type_by_name("Item").expect("Second Item").requirement
    );
    assert!(fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeDuplicate));
}

#[test]
fn associated_type_binding_negative_matrix_is_deterministic() {
    let duplicate = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterable {
  type Item
}

class Box {
}

impl Iterable for Box {
  type Item = Int
  type Item = String
}
"#,
        )
        .analyze();
    assert!(duplicate
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeDuplicate));

    let unknown = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterable {
  type Item
}

class Box {
}

impl Iterable for Box {
  type Extra = Int
}
"#,
        )
        .analyze();
    assert!(unknown
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeUnknown));

    let invalid = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterable {
  type Item
}

class Box {
}

impl Iterable for Box {
  type Item = Missing
}
"#,
        )
        .analyze();
    assert!(invalid
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeBindingInvalid));
}

#[test]
fn direct_field_delegation_negative_matrix_is_deterministic() {
    let missing = WorkspaceFixture::new()
        .module("main", "class Counter { count via _missing }\n")
        .analyze();
    assert!(missing
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::DelegationFieldNotFound));

    let untyped = WorkspaceFixture::new()
        .module("main", "class Counter { _count\n count via _count }\n")
        .analyze();
    assert!(untyped
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::DelegationFieldTypeRequired));

    let immutable = WorkspaceFixture::new()
        .module("main", "class Counter { const _count: Int\n count=(_) via _count }\n")
        .analyze();
    assert!(immutable
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::DelegationSetterRequiresMutableField));

    let superclass = WorkspaceFixture::new()
        .module(
            "main",
            "class Base { mut _count: Int }\nclass Child is Base { count via _count }\n",
        )
        .analyze();
    assert!(superclass
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::DelegationFieldNotFound));
}

#[test]
fn delegated_accessors_conflict_with_explicit_ordinary_accessors() {
    let getter = WorkspaceFixture::new()
        .module(
            "main",
            "class Counter { mut _count: Int\n count -> Int { _count }\n count via _count }\n",
        )
        .analyze();
    assert!(getter
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplMemberConflict));

    let setter = WorkspaceFixture::new()
        .module(
            "main",
            "class Counter { mut _count: Int\n count=(_ value: Int) { _count = value }\n mut count via _count }\n",
        )
        .analyze();
    assert!(setter
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplMemberConflict));
}

#[test]
fn trait_property_and_class_via_publish_ordinary_accessor_surface() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Counter {
  count: Int
  mut total: Int
}

class CounterImpl {
  mut _count: Int
  count via _count
  mut total via _count
}
"#,
        )
        .analyze();
    assert!(!fixture.analysis.snapshot.has_errors(), "class/property semantic update produced errors");

    let class = fixture.decl("main", "CounterImpl");
    let surface = fixture.analysis.snapshot.surfaces().get(&class).expect("class surface");
    let count = Selector::getter("count").expect("count getter");
    let total_getter = Selector::getter("total").expect("total getter");
    let total_setter = Selector::setter("total").expect("total setter");
    assert!(surface.instance.get_callable(&count).is_some());
    assert!(surface.instance.get_callable(&total_getter).is_some());
    assert!(surface.instance.get_callable(&total_setter).is_some());

    let trait_surface = fixture.analysis.snapshot.trait_surfaces.get(&fixture.decl("main", "Counter")).expect("trait surface");
    assert!(trait_surface.get_by_selector(&count, DispatchSide::Instance).is_some());
    assert!(trait_surface.get_by_selector(&total_getter, DispatchSide::Instance).is_some());
    assert!(trait_surface.get_by_selector(&total_setter, DispatchSide::Instance).is_some());
}

#[test]
fn trait_property_accepts_explicit_ordinary_getter_and_setter_witnesses() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Counter {
  mut count: Int
}

class CounterImpl {
  mut _count: Int
  count -> Int { _count }
  count=(_ value: Int) { _count = value }
}

impl Counter for CounterImpl {
}
"#,
        )
        .analyze();
    assert!(!fixture.analysis.snapshot.has_errors(), "explicit property witnesses produced errors: {:?}", fixture.analysis.snapshot.diagnostics);
    let target = fixture
        .analysis
        .snapshot
        .declarations
        .form(&fixture.decl("main", "CounterImpl"))
        .expect("CounterImpl form");
    let trait_ref = TraitRef::new(fixture.decl("main", "Counter"), Vec::new().into_boxed_slice());
    let evidence = fixture.analysis.snapshot.conformance_evidence_for(target, &trait_ref).expect("complete property conformance");
    assert_eq!(evidence.requirements.len(), 2, "getter and setter should be ordinary witness selections");
}

#[test]
fn conformance_local_via_is_rejected_without_target_private_field_access() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait CounterTrait {
  mut count: Int
}

class Counter {
  mut _count: Int
}

impl CounterTrait for Counter {
  mut count via _count
}
"#,
        )
        .analyze();
    assert!(fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::DelegationInConformanceUnsupported));
}

#[test]
fn trait_surface_reuses_actual_trait_and_member_generic_binders() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Mapper<T> {
  map<U>(_ value: T) -> U
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Mapper");
    let header = fixture.analysis.snapshot.trait_headers.get(&declaration).expect("trait header");
    let trait_parameter = header.generic_signature.as_ref().expect("trait generic signature").parameters[0];
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    let selector = Selector::method("map", [SelectorSlot::Positional]).unwrap();
    let member = surface.get_by_selector(&selector, DispatchSide::Instance).expect("map member");
    let parameter_type = member.signature.parameters[0].declared_type.canonical_type().expect("T parameter type");
    assert!(matches!(fixture.analysis.snapshot.store.get(parameter_type), phalcom_semantic::TypeData::Parameter(id) if *id == trait_parameter));
    let member_generics = member.signature.generics.as_ref().expect("member generic signature");
    assert_eq!(member_generics.owner, phalcom_semantic::TypeParameterOwner::Callable(member.callable.clone()));
    assert_eq!(member_generics.parameters.len(), 1);
}

#[test]
fn trait_associated_projection_formation_is_source_order_independent_and_nested() {
    let before = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Before {
  type Item
  get -> Option<Self::Item>
}
"#,
        )
        .analyze();
    let after = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait After {
  get -> Option<Self::Item>
  type Item
}
"#,
        )
        .analyze();

    for (fixture, trait_name) in [(&before, "Before"), (&after, "After")] {
        assert!(!fixture.analysis.snapshot.has_errors(), "projection diagnostics: {:?}", fixture.analysis.snapshot.diagnostics);
        let declaration = fixture.decl("main", trait_name);
        let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");
        let member = surface
            .get_by_selector(&Selector::getter("get").expect("getter selector"), DispatchSide::Instance)
            .expect("get requirement");
        let option = member.signature.declared_return.canonical_type().expect("Option<Self::Item> type");
        let phalcom_semantic::TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(option) else {
            panic!("expected nested Option application");
        };
        assert_eq!(arguments.len(), 1);
        let phalcom_semantic::TypeData::AssociatedProjection(projection) = fixture.analysis.snapshot.store.get(arguments[0]) else {
            panic!("expected nested associated projection");
        };
        assert_eq!(projection.trait_ref.declaration, declaration);
        assert_eq!(projection.trait_ref.arguments.len(), 0);
        assert_eq!(projection.requirement.owner, declaration);
        assert_eq!(projection.requirement.index, 0);
        assert!(matches!(fixture.analysis.snapshot.store.get(projection.subject), phalcom_semantic::TypeData::SelfType(_)));
    }
}

#[test]
fn trait_associated_projection_same_spelling_is_trait_owned() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait First {
  type Item
  get -> Self::Item
}

trait Second {
  type Item
  get -> Self::Item
}
"#,
        )
        .analyze();
    assert!(!fixture.analysis.snapshot.has_errors(), "projection diagnostics: {:?}", fixture.analysis.snapshot.diagnostics);

    let projection = |trait_name: &str| {
        let declaration = fixture.decl("main", trait_name);
        let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");
        let member = surface
            .get_by_selector(&Selector::getter("get").expect("getter selector"), DispatchSide::Instance)
            .expect("get requirement");
        let ty = member.signature.declared_return.canonical_type().expect("projection type");
        let phalcom_semantic::TypeData::AssociatedProjection(projection) = fixture.analysis.snapshot.store.get(ty) else {
            panic!("expected associated projection");
        };
        projection.clone()
    };
    let first = projection("First");
    let second = projection("Second");
    assert_ne!(first, second);
    assert_ne!(first.trait_ref.declaration, second.trait_ref.declaration);
    assert_ne!(first.requirement.owner, second.requirement.owner);
    assert_eq!(first.requirement.index, second.requirement.index);
}

#[test]
fn generic_trait_associated_projection_retains_abstract_trait_arguments() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Converter<Target> {
  type Output
  convert -> Self::Output
}
"#,
        )
        .analyze();
    assert!(!fixture.analysis.snapshot.has_errors(), "projection diagnostics: {:?}", fixture.analysis.snapshot.diagnostics);
    let declaration = fixture.decl("main", "Converter");
    let header = fixture.analysis.snapshot.trait_headers.get(&declaration).expect("trait header");
    let parameter = header.generic_signature.as_ref().expect("generic signature").parameters[0];
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    let member = surface
        .get_by_selector(&Selector::getter("convert").expect("getter selector"), DispatchSide::Instance)
        .expect("convert requirement");
    let ty = member.signature.declared_return.canonical_type().expect("projection type");
    let phalcom_semantic::TypeData::AssociatedProjection(projection) = fixture.analysis.snapshot.store.get(ty) else {
        panic!("expected associated projection");
    };
    let mut expected_store = (*fixture.analysis.snapshot.store).clone();
    let expected_parameter_form = expected_store.parameter_form(parameter);
    assert_eq!(projection.trait_ref.declaration, declaration);
    assert_eq!(projection.trait_ref.arguments.as_ref(), &[expected_parameter_form]);
    assert_eq!(projection.requirement.owner, declaration);
    assert_eq!(projection.requirement.index, 0);
}

#[test]
fn unknown_trait_associated_projection_reports_written_name() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait MissingAssociation {
  get -> Self::Missing
}
"#,
        )
        .analyze();
    let diagnostics = fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::AssociatedTypeUnknown)
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 1, "unexpected associated projection diagnostics: {diagnostics:?}");
    assert!(diagnostics[0].message.contains("Missing"));
    assert!(diagnostics[0].primary.range.start < diagnostics[0].primary.range.end);
}

#[test]
fn source_conformance_projection_normalizes_sibling_bindings_and_local_signatures() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Sequence {
  type Element
  type Collection
  next -> Option<Self::Element>
}

class List<T> {
}

impl<T> Sequence for List<T> {
  type Element = T
  type Collection = List<Self::Element>
  next -> Option<Self::Element> { None }
}
"#,
        )
        .analyze();
    assert!(!fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| matches!(diagnostic.code, phalcom_semantic::DiagnosticCode::AssociatedTypeUnknown | phalcom_semantic::DiagnosticCode::AssociatedTypeBindingInvalid)));

    let declaration = fixture.decl("main", "Sequence");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    let element = surface.associated_type_by_name("Element").expect("Element requirement");
    let collection = surface.associated_type_by_name("Collection").expect("Collection requirement");
    let impl_id = ImplId::new(fixture.module("main").clone(), ImplLocalId(2));
    let plan = fixture.analysis.snapshot.conformance_witness_plans.get(&impl_id).expect("conformance plan");
    let element_binding = plan.associated_type_plan.bindings.get(&element.requirement).expect("Element binding");
    let collection_binding = plan.associated_type_plan.bindings.get(&collection.requirement).expect("Collection binding");
    let phalcom_semantic::TypeData::Parameter(element_parameter) = fixture.analysis.snapshot.store.get(element_binding.value_template) else {
        panic!("expected Element = T source parameter");
    };
    assert!(matches!(
        fixture.analysis.snapshot.store.type_parameter(*element_parameter).owner,
        phalcom_semantic::TypeParameterOwner::Impl(_)
    ));
    let phalcom_semantic::TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(collection_binding.value_template) else {
        panic!("expected Collection = List<T> source application");
    };
    assert_eq!(arguments.len(), 1);
    assert_eq!(fixture.analysis.snapshot.store.get(arguments[0]), fixture.analysis.snapshot.store.get(element_binding.value_template));

    let next = phalcom_semantic::CallableId::new(
        CallableOwnerId::Conformance(impl_id.clone()),
        Selector::getter("next").expect("next selector"),
        DispatchSide::Instance,
    );
    let signature = fixture.analysis.snapshot.callable_signatures.get(&next).expect("source conformance signature");
    let option = signature.declared_return.canonical_type().expect("Option<T> source return");
    let phalcom_semantic::TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(option) else {
        panic!("expected Option<T> source return");
    };
    assert_eq!(arguments.len(), 1);
    assert_eq!(fixture.analysis.snapshot.store.get(arguments[0]), fixture.analysis.snapshot.store.get(element_binding.value_template));
    assert_eq!(plan.associated_type_plan.failures.len(), 0);
}

#[test]
fn source_conformance_projection_is_order_independent_and_cycles_are_incomplete() {
    let make_fixture = |bindings: &str| {
        WorkspaceFixture::new()
            .module(
                "main",
                format!(
                    "trait Sequence {{\n  type Element\n  type Collection\n  next -> Option<Self::Element>\n}}\n\nclass List<T> {{\n}}\n\nimpl<T> Sequence for List<T> {{\n{bindings}\n  next -> Option<Self::Element> {{ None }}\n}}\n"
                ),
            )
            .analyze()
    };
    let forward = make_fixture("  type Element = T\n  type Collection = List<Self::Element>");
    let reverse = make_fixture("  type Collection = List<Self::Element>\n  type Element = T");

    for fixture in [&forward, &reverse] {
        assert!(!fixture
            .analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| matches!(diagnostic.code, phalcom_semantic::DiagnosticCode::AssociatedTypeUnknown | phalcom_semantic::DiagnosticCode::AssociatedTypeBindingInvalid)));
        let plan = fixture.analysis.snapshot.conformance_witness_plans.values().next().expect("conformance plan");
        assert!(plan.associated_type_plan.failures.is_empty());
        assert_eq!(plan.associated_type_plan.bindings.len(), 2);
    }
    let forward_plan = forward.analysis.snapshot.conformance_witness_plans.values().next().expect("forward plan");
    let reverse_plan = reverse.analysis.snapshot.conformance_witness_plans.values().next().expect("reverse plan");
    assert_eq!(forward_plan.associated_type_plan.bindings.values().map(|binding| binding.value_template).collect::<Vec<_>>().len(), 2);
    assert_eq!(reverse_plan.associated_type_plan.bindings.values().map(|binding| binding.value_template).collect::<Vec<_>>().len(), 2);

    let cycle = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Cyclic {
  type A
  type B
}

class Box {
}

impl Cyclic for Box {
  type A = Self::B
  type B = Self::A
}
"#,
        )
        .analyze();
    let cycle_plan = cycle.analysis.snapshot.conformance_witness_plans.values().next().expect("cycle plan");
    assert!(matches!(cycle_plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Incomplete { .. }));
    assert!(cycle_plan
        .associated_type_plan
        .failures
        .iter()
        .any(|failure| failure.reason.contains("cyclic associated type binding")));
}

#[test]
fn exact_projection_normalization_consumes_published_associated_evidence() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterable {
  type Item
}

class Box<T> {
}

impl<T> Iterable for Box<T> {
  type Item = T
}
"#,
        )
        .analyze();
    let trait_declaration = fixture.decl("main", "Iterable");
    let target_declaration = fixture.decl("main", "Box");
    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int form");
    let box_form = fixture.analysis.snapshot.declarations.form(&target_declaration).expect("Box form");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let target = store.apply_type_form(box_form, &[int]).expect("Box<Int>");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&trait_declaration).expect("trait surface");
    let requirement = surface.associated_type_by_name("Item").expect("Item requirement");
    let trait_ref = TraitRef::new(trait_declaration, Vec::new().into_boxed_slice());
    let projection = store.associated_projection(target, trait_ref, requirement.requirement.clone());

    let mut budget = phalcom_semantic::QueryBudget::default();
    let cancel = phalcom_semantic::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );
    assert_eq!(
        phalcom_semantic::types::normalize_type(&mut store, projection, &mut context),
        phalcom_semantic::types::ProjectionNormalizationResult::Normalized(int)
    );

    let option_form = fixture
        .analysis
        .snapshot
        .declarations
        .form(&CoreDeclarationIds::default().option)
        .expect("Option type form");
    let nested = store.apply_type_form(option_form, &[projection]).expect("Option projection");
    let nested_result = phalcom_semantic::types::normalize_type(&mut store, nested, &mut context);
    let phalcom_semantic::types::ProjectionNormalizationResult::Normalized(nested) = nested_result else {
        panic!("expected nested exact projection normalization, got {nested_result:?}");
    };
    let phalcom_semantic::TypeData::Applied { arguments, .. } = store.get(nested) else {
        panic!("expected normalized Option application, got {:?}", store.get(nested));
    };
    assert_eq!(arguments.as_ref(), &[int]);
}

#[test]
fn exact_projection_integration_normalizes_parameter_and_return_witness_contracts() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Iterator {
  type Item
  next -> Option<Self::Item>
  accept(_ value: Self::Item) -> Self::Item
}

class List<T> {
}

impl<T> Iterator for List<T> {
  type Item = T
  next -> Option<T> { None }
  accept(_ value: T) -> T { value }
}
"#,
        )
        .analyze();
    let iterator = fixture.decl("main", "Iterator");
    let list = fixture.decl("main", "List");
    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int form");
    let list_form = fixture.analysis.snapshot.declarations.form(&list).expect("List form");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let target = store.apply_type_form(list_form, &[int]).expect("List<Int>");
    let trait_ref = TraitRef::new(iterator.clone(), Vec::new().into_boxed_slice());
    let evidence = match phalcom_semantic::impls::resolve_conformance_evidence_with_surfaces(
        &fixture.analysis.snapshot.conformance_index,
        &fixture.analysis.snapshot.conformance_witness_plans,
        &fixture.analysis.snapshot.trait_surfaces,
        &fixture.analysis.snapshot.declarations,
        &mut store,
        fixture.analysis.snapshot.hierarchy.as_ref(),
        target,
        &trait_ref,
    ) {
        phalcom_semantic::impls::ConformanceResolution::Proven(evidence) => evidence,
        other => panic!("expected exact Iterator evidence, got {other:?}"),
    };
    let item = fixture
        .analysis
        .snapshot
        .trait_surfaces
        .get(&iterator)
        .expect("Iterator surface")
        .associated_type_by_name("Item")
        .expect("Item requirement")
        .requirement
        .clone();
    assert_eq!(evidence.associated_types.get(&item).expect("exact Item binding").value, int);

    let surface = fixture.analysis.snapshot.trait_surfaces.get(&iterator).expect("Iterator surface");
    let next = surface
        .get_by_selector(&Selector::getter("next").expect("next selector"), DispatchSide::Instance)
        .expect("next requirement")
        .requirement
        .clone();
    let accept = surface
        .get_by_selector(
            &Selector::method("accept", vec![SelectorSlot::Positional]).expect("accept selector"),
            DispatchSide::Instance,
        )
        .expect("accept requirement")
        .requirement
        .clone();
    let next_return = evidence.requirement_views.get(&next).expect("next view").signature.declared_return.canonical_type().expect("next return");
    let phalcom_semantic::TypeData::Applied { arguments, .. } = store.get(next_return) else {
        panic!("expected normalized Option<Int>, got {:?}", store.get(next_return));
    };
    assert_eq!(arguments.as_ref(), &[int]);
    assert_eq!(
        evidence
            .requirement_views
            .get(&accept)
            .expect("accept view")
            .signature
            .parameters[0]
            .declared_type
            .canonical_type(),
        Some(int)
    );
    assert_eq!(evidence.requirement_views.get(&accept).expect("accept view").signature.declared_return.canonical_type(), Some(int));
}

#[test]
fn trait_default_can_use_abstract_associated_projection_and_exact_view_normalizes_it() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Defaulted {
  type Item
  echo(_ value: Self::Item) -> Self::Item { value }
}

class Box {
}

impl Defaulted for Box {
  type Item = Int
}
"#,
        )
        .analyze();
    assert!(!fixture.analysis.snapshot.has_errors(), "unexpected diagnostics: {:?}", fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>());
    let defaulted = fixture.decl("main", "Defaulted");
    let target = fixture
        .analysis
        .snapshot
        .declarations
        .form(&fixture.decl("main", "Box"))
        .expect("Box form");
    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int form");
    let trait_ref = TraitRef::new(defaulted.clone(), Vec::new().into_boxed_slice());
    let evidence = fixture.analysis.snapshot.conformance_evidence_for(target, &trait_ref).expect("default-backed evidence");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&defaulted).expect("Defaulted surface");
    let requirement = surface
        .get_by_selector(&Selector::method("echo", vec![SelectorSlot::Positional]).expect("echo selector"), DispatchSide::Instance)
        .expect("echo requirement")
        .requirement
        .clone();
    assert!(matches!(
        evidence.requirements.get(&requirement),
        Some(phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. })
    ));
    assert_eq!(evidence.requirement_views.get(&requirement).expect("echo view").signature.parameters[0].declared_type.canonical_type(), Some(int));
    assert_eq!(evidence.requirement_views.get(&requirement).expect("echo view").signature.declared_return.canonical_type(), Some(int));
}

#[test]
fn normalized_associated_projection_rejects_incompatible_witness() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Typed {
  type Item
  echo(_ value: Self::Item) -> Self::Item
}

class Box {
}

impl Typed for Box {
  type Item = Int
  echo(_ value: String) -> String { value }
}
"#,
        )
        .analyze();
    let plan = fixture.analysis.snapshot.conformance_witness_plans.values().next().expect("Typed conformance plan");
    assert!(matches!(
        plan.completeness,
        phalcom_semantic::impls::ConformanceCompleteness::Incomplete { ref failures }
            if failures.iter().any(|failure| matches!(failure, phalcom_semantic::impls::ConformanceFailure::Behavioral(failure) if failure.reason.contains("subtype") || failure.reason.contains("type")))
    ), "expected normalized witness incompatibility, got {:?}", plan.completeness);
}

#[test]
fn projection_normalization_modes_preserve_symbolic_source_and_unknown_states() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Abstract {
  type Item
  get -> Self::Item
}

trait Other {
  type Item
}

class Box<T> {
}

impl<T> Abstract for Box<T> {
  type Item = T
}
"#,
        )
        .analyze();
    let abstract_declaration = fixture.decl("main", "Abstract");
    let abstract_surface = fixture.analysis.snapshot.trait_surfaces.get(&abstract_declaration).expect("Abstract surface");
    let abstract_member = abstract_surface
        .get_by_selector(&Selector::getter("get").expect("get selector"), DispatchSide::Instance)
        .expect("get requirement");
    let abstract_projection = abstract_member.signature.declared_return.canonical_type().expect("abstract projection");
    let abstract_trait_ref = match fixture.analysis.snapshot.store.get(abstract_projection) {
        phalcom_semantic::TypeData::AssociatedProjection(projection) => projection.trait_ref.clone(),
        other => panic!("expected abstract projection, got {other:?}"),
    };
    let mut abstract_store = (*fixture.analysis.snapshot.store).clone();
    let mut abstract_budget = phalcom_semantic::QueryBudget::default();
    let abstract_cancel = phalcom_semantic::CancellationToken::new();
    let mut abstract_context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::AbstractTrait { trait_ref: &abstract_trait_ref },
        &mut abstract_budget,
        &abstract_cancel,
    );
    assert_eq!(
        phalcom_semantic::types::normalize_type(&mut abstract_store, abstract_projection, &mut abstract_context),
        phalcom_semantic::types::ProjectionNormalizationResult::Symbolic(abstract_projection)
    );

    let source_plan = fixture.analysis.snapshot.conformance_witness_plans.values().next().expect("source plan");
    let source_requirement = source_plan
        .associated_type_plan
        .bindings
        .keys()
        .next()
        .expect("source associated requirement")
        .clone();
    let source_binding = source_plan.associated_type_plan.bindings.get(&source_requirement).expect("source binding");
    let source_target = source_plan.target_template;
    let source_trait_ref = source_plan.trait_ref_template.clone();
    let mut source_store = (*fixture.analysis.snapshot.store).clone();
    let source_projection = source_store.associated_projection(source_target, source_trait_ref.clone(), source_requirement);
    let mut source_budget = phalcom_semantic::QueryBudget::default();
    let source_cancel = phalcom_semantic::CancellationToken::new();
    let mut source_context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::SourceConformance {
            trait_ref: &source_trait_ref,
            target: source_target,
            plan: &source_plan.associated_type_plan,
        },
        &mut source_budget,
        &source_cancel,
    );
    assert_eq!(
        phalcom_semantic::types::normalize_type(&mut source_store, source_projection, &mut source_context),
        phalcom_semantic::types::ProjectionNormalizationResult::Normalized(source_binding.value_template)
    );

    let other_declaration = fixture.decl("main", "Other");
    let other_requirement = fixture
        .analysis
        .snapshot
        .trait_surfaces
        .get(&other_declaration)
        .expect("Other surface")
        .associated_type_by_name("Item")
        .expect("Other Item")
        .requirement
        .clone();
    let box_declaration = fixture.decl("main", "Box");
    let box_form = fixture.analysis.snapshot.declarations.form(&box_declaration).expect("Box form");
    let int = fixture.analysis.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int form");
    let mut exact_store = (*fixture.analysis.snapshot.store).clone();
    let exact_target = exact_store.apply_type_form(box_form, &[int]).expect("Box<Int>");
    let other_ref = TraitRef::new(other_declaration, Vec::new().into_boxed_slice());
    let other_projection = exact_store.associated_projection(exact_target, other_ref, other_requirement);
    let mut exact_budget = phalcom_semantic::QueryBudget::default();
    let exact_cancel = phalcom_semantic::CancellationToken::new();
    let mut exact_context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut exact_budget,
        &exact_cancel,
    );
    assert_eq!(
        phalcom_semantic::types::normalize_type(&mut exact_store, other_projection, &mut exact_context),
        phalcom_semantic::types::ProjectionNormalizationResult::Unknown(phalcom_semantic::UnknownReason::UnderconstrainedTypeVariable)
    );
}

#[test]
fn projection_normalization_propagates_cancellation_and_budget_states() {
    let mut store = phalcom_semantic::TypeStore::new();
    let ty = store.unit();
    let cancel = phalcom_semantic::CancellationToken::new();
    cancel.cancel();
    let mut budget = phalcom_semantic::QueryBudget::default();
    let trait_ref = TraitRef::new(fixture_module(), Vec::new().into_boxed_slice());
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::AbstractTrait {
            trait_ref: &trait_ref,
        },
        &mut budget,
        &cancel,
    );
    assert_eq!(
        phalcom_semantic::types::normalize_type(&mut store, ty, &mut context),
        phalcom_semantic::types::ProjectionNormalizationResult::Cancelled
    );

    let cancel = phalcom_semantic::CancellationToken::new();
    let mut budget = phalcom_semantic::QueryBudget::new(0);
    let trait_ref = TraitRef::new(fixture_module(), Vec::new().into_boxed_slice());
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::AbstractTrait { trait_ref: &trait_ref },
        &mut budget,
        &cancel,
    );
    assert!(matches!(
        phalcom_semantic::types::normalize_type(&mut store, ty, &mut context),
        phalcom_semantic::types::ProjectionNormalizationResult::BudgetExceeded(_)
    ));
}

#[test]
fn source_projection_normalization_detects_recursive_binding_cycles() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Cyclic {
  type A
  type B
}

class Box {
}

impl Cyclic for Box {
  type A = Self::B
  type B = Self::A
}
"#,
        )
        .analyze();
    let plan = fixture.analysis.snapshot.conformance_witness_plans.values().next().expect("cycle plan");
    let requirement = plan.associated_type_plan.bindings.keys().next().expect("cycle requirement").clone();
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let projection = store.associated_projection(plan.target_template, plan.trait_ref_template.clone(), requirement);
    let mut budget = phalcom_semantic::QueryBudget::default();
    let cancel = phalcom_semantic::CancellationToken::new();
    let trait_ref = plan.trait_ref_template.clone();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::SourceConformance {
            trait_ref: &trait_ref,
            target: plan.target_template,
            plan: &plan.associated_type_plan,
        },
        &mut budget,
        &cancel,
    );
    assert_eq!(
        phalcom_semantic::types::normalize_type(&mut store, projection, &mut context),
        phalcom_semantic::types::ProjectionNormalizationResult::Recursive
    );
}

#[test]
fn exact_projection_normalization_preserves_exact_enum_case_identity() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Tagged {
  type Item
}

enum Status {
  Ready
  Waiting
}

impl Tagged for Status::Ready {
  type Item = Self
}
"#,
        )
        .analyze();
    let trait_declaration = fixture.decl("main", "Tagged");
    let status_declaration = fixture.decl("main", "Status");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&trait_declaration).expect("Tagged surface");
    let requirement = surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();
    let (_, contribution) = fixture.analysis.snapshot.conformance_index.iter().next().expect("case conformance");
    let phalcom_semantic::impls::ConformanceTarget::ExactEnumCase(variant) = &contribution.target else {
        panic!("expected exact enum-case conformance");
    };
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let status = fixture.analysis.snapshot.declarations.form(&status_declaration).expect("Status type");
    let target = store.exact_case_type(variant, status).expect("Status::Ready type");
    let trait_ref = TraitRef::new(trait_declaration, Vec::new().into_boxed_slice());
    let projection = store.associated_projection(target, trait_ref, requirement);
    let mut budget = phalcom_semantic::QueryBudget::default();
    let cancel = phalcom_semantic::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );
    let normalized = match phalcom_semantic::types::normalize_type(&mut store, projection, &mut context) {
        phalcom_semantic::types::ProjectionNormalizationResult::Normalized(ty) => ty,
        other => panic!("expected exact enum-case normalization, got {other:?}"),
    };
    assert!(matches!(store.get(normalized), phalcom_semantic::TypeData::ExactCase { .. }));
    assert_eq!(normalized, target);
}

#[test]
fn exact_projection_normalization_preserves_ambiguity() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Tagged {
  type Item
}

class Value<T> {
}

class Marker {
}

impl<T> Tagged for Value<T> {
  type Item = T
}

impl Tagged for Value<Marker> {
  type Item = Int
}
"#,
        )
        .analyze();
    let trait_declaration = fixture.decl("main", "Tagged");
    let value_declaration = fixture.decl("main", "Value");
    let marker_declaration = fixture.decl("main", "Marker");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&trait_declaration).expect("Tagged surface");
    let requirement = surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();
    let value_form = fixture.analysis.snapshot.declarations.form(&value_declaration).expect("Value form");
    let marker = fixture.analysis.snapshot.declarations.form(&marker_declaration).expect("Marker form");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let target = store.apply_type_form(value_form, &[marker]).expect("Value<Marker>");
    let trait_ref = TraitRef::new(trait_declaration, Vec::new().into_boxed_slice());
    let projection = store.associated_projection(target, trait_ref, requirement);
    let mut budget = phalcom_semantic::QueryBudget::default();
    let cancel = phalcom_semantic::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );
    let result = phalcom_semantic::types::normalize_type(&mut store, projection, &mut context);
    assert!(matches!(result, phalcom_semantic::types::ProjectionNormalizationResult::Ambiguous(ref candidates) if candidates.len() == 2), "expected ambiguity, got {result:?}");
}

fn fixture_module() -> phalcom_semantic::DeclarationId {
    phalcom_semantic::DeclarationId::new(phalcom_semantic::ModuleId::universe_root(), "SyntheticTrait".into())
}

#[test]
fn trait_surface_reports_duplicate_selector_and_preserves_source_visibility() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Surface {
  @private hidden() -> Int
  @protected semi() -> Int
  _$internal() -> Int
  run() -> Int
  run() -> String
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Surface");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&declaration).expect("trait surface");

    assert_eq!(surface.members.len(), 4, "duplicate selectors are one contract conflict");
    assert_eq!(
        surface
            .get_by_selector(&Selector::method("hidden", []).unwrap(), DispatchSide::Instance)
            .expect("hidden member")
            .visibility,
        phalcom_semantic::MemberVisibility::Private
    );
    assert_eq!(
        surface
            .get_by_selector(&Selector::method("semi", []).unwrap(), DispatchSide::Instance)
            .expect("semi member")
            .visibility,
        phalcom_semantic::MemberVisibility::Protected
    );
    assert_eq!(
        surface
            .get_by_selector(&Selector::method("_$internal", []).unwrap(), DispatchSide::Instance)
            .expect("internal member")
            .visibility,
        phalcom_semantic::MemberVisibility::Internal
    );
    assert!(
        surface
            .get_by_selector(&Selector::method("run", []).unwrap(), DispatchSide::Instance)
            .expect("run member")
            .source
            .range
            .start
            < surface
            .get_by_selector(&Selector::method("run", []).unwrap(), DispatchSide::Instance)
            .expect("run member")
            .source
            .range
            .end
    );
    assert_eq!(
        fixture
            .analysis
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::TraitMemberConflict)
            .count(),
        1
    );
}

#[test]
fn callable_body_accepts_explicit_trait_owner_generics_without_nominal_entry() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Mapper<T> {
  map<U>(_ owner_value: T, _ member_value: U) -> U {
    let owner_typed: T = owner_value
    let member_typed: U = member_value
    member_typed
  }
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Mapper");
    let snapshot = &fixture.analysis.snapshot;
    let header = snapshot.trait_headers.get(&declaration).expect("trait header");
    let surface = snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    let callable = surface
        .get_by_selector(
            &Selector::method("map", [SelectorSlot::Positional, SelectorSlot::Positional]).unwrap(),
            DispatchSide::Instance,
        )
        .expect("map member");
    let unit = snapshot.sources.get(fixture.module("main")).expect("source unit");
    let (body, body_range) = match &unit.program.statements[0] {
        Statement::Trait(trait_def) => match &trait_def.members[0] {
            phalcom_ast::ast::TraitMember::Behavior(BehaviorMember::Method(method)) => (method.body.statements().expect("default method body"), method.range),
            member => panic!("expected trait method, got {member:?}"),
        },
        statement => panic!("expected trait declaration, got {statement:?}"),
    };
    let mut store = (*snapshot.store).clone();
    let resolver = SimpleTypeResolver::new();
    let cancel = CancellationToken::new();
    let analysis = analyze_callable_body(
        BodyAnalysisContext {
            store: &mut store,
            hierarchy: snapshot.hierarchy.as_ref(),
            resolver: &resolver,
            declarations: snapshot.declarations.as_ref(),
            dispatch: snapshot.dispatch.as_ref(),
            trait_surface: None,
            conformance_semantics: None,
            module: fixture.module("main").clone(),
        },
        CallableBodyRequest {
            callable: callable.callable.clone(),
            body,
            body_range,
            owner_generic_signature: header.generic_signature.as_ref(),
            self_type_override: None,
            declared_signature: Some((&callable.callable, &callable.signature)),
            budget: QueryBudget::default(),
            cancel: &cancel,
            field_signatures: None,
            field_lifecycle: None,
            enum_semantics: None,
            data_semantics: None,
            associated_families: None,
            conformance_semantics: None,
        },
    );

    assert!(snapshot.declarations.get(&declaration).is_none(), "trait must remain outside nominal metadata");
    assert_eq!(analysis.status, CallableAnalysisStatus::Complete);
    assert!(analysis.diagnostics.is_empty(), "trait default body diagnostics: {:?}", analysis.diagnostics);
}

#[test]
fn trait_defaults_call_complete_abstract_surface_without_runtime_targets() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Display<T> {
  render(_ value: T) -> T
  later(_ value: T) -> T
  helper(_ value: T) -> T { value }
  render_default(_ value: T) -> T { self.render(value) }
  later_default(_ value: T) -> T { self.later(value) }
  helper_default(_ value: T) -> T { self.helper(value) }
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Display");
    let snapshot = &fixture.analysis.snapshot;
    let surface = snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    let render = surface
        .get_by_selector(&Selector::method("render", [SelectorSlot::Positional]).unwrap(), DispatchSide::Instance)
        .expect("render requirement");
    let later = surface
        .get_by_selector(&Selector::method("later", [SelectorSlot::Positional]).unwrap(), DispatchSide::Instance)
        .expect("later requirement");
    let helper = surface
        .get_by_selector(&Selector::method("helper", [SelectorSlot::Positional]).unwrap(), DispatchSide::Instance)
        .expect("helper default");

    for (name, requirement) in [("render_default", render), ("later_default", later), ("helper_default", helper)] {
        let callable = surface
            .get_by_selector(&Selector::method(name, [SelectorSlot::Positional]).unwrap(), DispatchSide::Instance)
            .expect("default member");
        let analysis = snapshot.callable_analyses.get(&callable.callable).expect("default analysis");
        assert!(analysis.diagnostics.is_empty(), "{name} diagnostics: {:?}", analysis.diagnostics);
        assert!(analysis.dependencies.iter().any(|dependency| dependency == &requirement.callable));
        assert!(
            analysis
                .expressions
                .values()
                .any(|expression| expression.callable.as_ref() == Some(&requirement.callable))
        );
        assert!(snapshot.declarations.get(&declaration).is_none(), "trait must not gain nominal metadata");
    }
}

#[test]
fn trait_defaults_preserve_abstract_self_and_use_canonical_argument_checking() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Contracts {
  make() -> Self
  accepts(_ value: Int) -> Int
  generic<U>(_ value: U) -> U
  use_self(_ value: Self) -> Self { self.make() }
  use_generic() -> Int { self.generic(1) }
  bad_argument() -> Int { self.accepts("wrong") }
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Contracts");
    let snapshot = &fixture.analysis.snapshot;
    let surface = snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    let make = surface
        .get_by_selector(&Selector::method("make", []).unwrap(), DispatchSide::Instance)
        .expect("make requirement");
    let self_type = make.signature.declared_return.canonical_type().expect("Self return");
    assert!(matches!(snapshot.store.get(self_type), phalcom_semantic::TypeData::SelfType(term) if term.owner == declaration));

    let use_self = surface
        .get_by_selector(&Selector::method("use_self", [SelectorSlot::Positional]).unwrap(), DispatchSide::Instance)
        .expect("use_self default");
    let use_self_analysis = snapshot.callable_analyses.get(&use_self.callable).expect("use_self analysis");
    assert!(
        use_self_analysis.diagnostics.is_empty(),
        "Self default diagnostics: {:?}",
        use_self_analysis.diagnostics
    );
    assert!(use_self_analysis.dependencies.iter().any(|dependency| dependency == &make.callable));
    assert!(
        use_self_analysis
            .semantic_dependencies
            .iter()
            .any(|dependency| dependency == &phalcom_semantic::checker::analysis::SemanticDependency::TraitSurface(declaration.clone()))
    );
    assert!(!use_self_analysis.semantic_dependencies.iter().any(|dependency| {
        matches!(dependency, phalcom_semantic::checker::analysis::SemanticDependency::CallableSignature(callable) if callable == &make.callable)
    }));

    let use_generic = surface
        .get_by_selector(&Selector::method("use_generic", []).unwrap(), DispatchSide::Instance)
        .expect("use_generic default");
    let use_generic_analysis = snapshot.callable_analyses.get(&use_generic.callable).expect("use_generic analysis");
    assert!(
        use_generic_analysis.diagnostics.is_empty(),
        "generic default diagnostics: {:?}",
        use_generic_analysis.diagnostics
    );

    let accepts = surface
        .get_by_selector(&Selector::method("accepts", [SelectorSlot::Positional]).unwrap(), DispatchSide::Instance)
        .expect("accepts requirement");
    let bad_argument = surface
        .get_by_selector(&Selector::method("bad_argument", []).unwrap(), DispatchSide::Instance)
        .expect("bad_argument default");
    let bad_analysis = snapshot.callable_analyses.get(&bad_argument.callable).expect("bad_argument analysis");
    assert!(bad_analysis.dependencies.iter().any(|dependency| dependency == &accepts.callable));
    assert!(
        bad_analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ArgumentMismatch),
        "argument mismatch must use the canonical call diagnostic: {:?}",
        bad_analysis.diagnostics
    );
}

#[test]
fn trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"trait Restricted {
  unknown() -> Int { self.missing() }
  field() -> Int { self.value }
  parent() -> Int { super.value() }
}
"#,
        )
        .analyze();
    let declaration = fixture.decl("main", "Restricted");
    let snapshot = &fixture.analysis.snapshot;
    let surface = snapshot.trait_surfaces.get(&declaration).expect("trait surface");
    for name in ["unknown", "field", "parent"] {
        let callable = surface
            .get_by_selector(&Selector::method(name, []).unwrap(), DispatchSide::Instance)
            .expect("restricted default");
        let analysis = snapshot.callable_analyses.get(&callable.callable).expect("restricted analysis");
        assert!(
            analysis.exits.normal_returns.iter().any(|exit| exit.knowledge.is_unknown()),
            "{name} must not prove a concrete result"
        );
        assert!(analysis.dependencies.is_empty(), "{name} must not resolve a concrete callable");
        assert_eq!(
            analysis.semantic_dependencies.as_ref(),
            &[phalcom_semantic::checker::analysis::SemanticDependency::TraitSurface(declaration.clone())],
            "{name} must depend only on its trait contract surface"
        );
    }
}

#[test]
fn trait_default_replays_external_nominal_callable_dependencies() {
    let fixture = WorkspaceFixture::new()
        .module(
            "main",
            r#"class Api {
  @class value() -> Int { 1 }
}

trait UsesApi {
  run() -> Int { Api.value() }
}
"#,
        )
        .analyze();
    let trait_declaration = fixture.decl("main", "UsesApi");
    let api_declaration = fixture.decl("main", "Api");
    let api_callable = phalcom_semantic::identity::CallableId::new(api_declaration.clone(), Selector::method("value", []).unwrap(), DispatchSide::Class);
    let run = fixture
        .analysis
        .snapshot
        .trait_surfaces
        .get(&trait_declaration)
        .expect("trait surface")
        .get_by_selector(&Selector::method("run", []).unwrap(), DispatchSide::Instance)
        .expect("run default");
    let analysis = fixture.analysis.snapshot.callable_analyses.get(&run.callable).expect("run analysis");
    assert!(analysis.diagnostics.is_empty(), "external call diagnostics: {:?}", analysis.diagnostics);
    assert!(
        analysis
            .semantic_dependencies
            .iter()
            .any(|dependency| { dependency == &phalcom_semantic::checker::analysis::SemanticDependency::DeclarationSurface(api_declaration.clone()) })
    );
    assert!(
        analysis
            .semantic_dependencies
            .iter()
            .any(|dependency| { dependency == &phalcom_semantic::checker::analysis::SemanticDependency::CallableSignature(api_callable.clone()) })
    );
    assert!(
        analysis
            .semantic_dependencies
            .iter()
            .any(|dependency| { dependency == &phalcom_semantic::checker::analysis::SemanticDependency::TraitSurface(trait_declaration.clone()) })
    );
    assert!(!analysis.semantic_dependencies.iter().any(|dependency| {
        matches!(
            dependency,
            phalcom_semantic::checker::analysis::SemanticDependency::DeclarationShell(declaration)
                if declaration.module.project == phalcom_modules::ProjectIdentity::Universe
        )
    }));
}
