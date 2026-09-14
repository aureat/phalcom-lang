use crate::semantic::support::WorkspaceFixture;
use phalcom_ast::ast::{BehaviorMember, Statement};
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::body::{BodyAnalysisContext, CallableBodyRequest};
use phalcom_semantic::checker::{CallableAnalysisStatus, analyze_callable_body};
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::identity::{CallableOwnerId, DispatchSide};
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
            BehaviorMember::Method(method) => (method.body.statements().expect("default method body"), method.range),
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
            module: fixture.module("main").clone(),
        },
        CallableBodyRequest {
            callable: callable.callable.clone(),
            body,
            body_range,
            owner_generic_signature: header.generic_signature.as_ref(),
            declared_signature: Some((&callable.callable, &callable.signature)),
            budget: QueryBudget::default(),
            cancel: &cancel,
            field_signatures: None,
            field_lifecycle: None,
            enum_semantics: None,
            data_semantics: None,
            associated_families: None,
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
