use phalcom_ast::parse;
use phalcom_common::modifier::ImplementationKind;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::{
    InterfaceBuilder, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ProjectUniverse, ResolvedProjectId, UnlinkedModuleInterface,
};
use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_semantic::checker::BindingConsistency;
use phalcom_semantic::checker::analysis::{
    AnalysisStatus, BindingState, BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, FlowBindingSummary, FlowStateSummary,
};
use phalcom_semantic::checker::flow::graph::{FlowGraph, FlowNodeKind};
use phalcom_semantic::checker::incident::{InternalSemanticIncident, InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
use phalcom_semantic::db::ProductFingerprint;
use phalcom_semantic::db::fingerprint::{
    callable_body_product_fingerprint, callable_signature_input_fingerprint, callable_signature_product_fingerprint, declaration_shell_input_fingerprint,
    declaration_shell_product_fingerprint, declaration_surface_input_fingerprint, declaration_surface_product_fingerprint,
    declaration_surface_source_input_fingerprint, linked_interface_input_fingerprint, linked_interface_product_fingerprint,
    module_diagnostics_product_fingerprint, semantic_component_product_fingerprint, unlinked_interface_input_fingerprint,
    unlinked_interface_product_fingerprint,
};
use phalcom_semantic::declarations::{DeclarationTypeInfo, GenericSupertypeTemplate};
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use phalcom_semantic::dispatch::{CallableSemanticKind, CallableSignature, DispatchSide};
use phalcom_semantic::explain::{ExplanationArena, ExplanationStep};
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, DiagnosticCauseId, ExpressionId, InternalSemanticIncidentId, LocalExpressionId};
use phalcom_semantic::signature::CallableSemanticSignature;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::surface::DeclarationSurface;
use phalcom_semantic::types::denotation::SemanticDenotation;
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge};
use phalcom_semantic::types::id::{KindId, TypeId, TypeParameterId};
use phalcom_semantic::types::parameter::{GenericConstraint, GenericSignature, TypeParameterOwner, TypeTerm};
use std::collections::BTreeMap;
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId {
        project: ResolvedProjectId::from_raw(1).into(),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("valid module component")]),
    }
}

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

fn declaration_shell() -> DeclarationTypeInfo {
    DeclarationTypeInfo {
        declaration: declaration("Shell"),
        form: TypeId(1),
        class_object_type: TypeId(2),
        kind: KindId::TYPE,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: None,
        generic_signature: None,
        supertype_template: None,
    }
}

fn build_interface(id: ModuleId, source: &str) -> UnlinkedModuleInterface {
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    InterfaceBuilder::build(id, ModuleKind::Module, &parsed.program).expect("interface builds")
}

fn source_surface_fingerprint(source: &str) -> phalcom_semantic::db::InputFingerprint {
    let module = module("surface_input");
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let program = Arc::new(parsed.program);
    let declaration = DeclarationId::new(module.clone(), "Api".into());
    let unit = ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program);
    let class_def = unit
        .program
        .statements
        .iter()
        .find_map(|statement| match statement {
            phalcom_ast::ast::Statement::Class(class_def) => Some(class_def),
            _ => None,
        })
        .expect("class declaration");
    declaration_surface_source_input_fingerprint(&unit, &declaration, class_def)
}

fn semantic_signature() -> CallableSemanticSignature {
    let owner = declaration("Owner");
    let selector = Selector::method("value", vec![]).expect("selector");
    let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    CallableSemanticSignature {
        callable,
        owner,
        side: DispatchSide::Instance,
        selector,
        generics: None,
        parameters: Box::new([]),
        declared_return: phalcom_semantic::DeclaredTypeFact::known(TypeTerm::Canonical(TypeId(1)), phalcom_semantic::DeclaredTypeBasis::SourceAnnotation),
        return_validation: phalcom_semantic::ReturnContractValidation::Unchecked,
        inferred_return: None,
        source: Some(SemanticSourceSpan::new(ModuleId::universe_root(), SourceRange { start: 10, end: 20 })),
        implementation: ImplementationKind::Phalcom,
        native_id: None,
        effects: EffectSpec::Unknown,
        raises: RaisesSpec::Unknown,
        flow: ReturnFlowSpec::Value,
        lifecycle: NativeLifecycleSpec::UNKNOWN,
    }
}

fn callable_analysis() -> CallableAnalysis {
    let callable = CallableId::new(declaration("Owner"), Selector::getter("value").expect("selector"), DispatchSide::Instance);
    CallableAnalysis {
        callable,
        body_range: SourceRange { start: 0, end: 20 },
        expressions: BTreeMap::new(),
        bindings: BTreeMap::new(),
        for_iteration_lanes: Arc::new(BTreeMap::new()),
        positional_expansions: Arc::new(BTreeMap::new()),
        known_invalid_positional_expansions: Arc::new(BTreeMap::new()),
        sequence_literals: Arc::new(BTreeMap::new()),
        associated_resolutions: Arc::new(BTreeMap::new()),
        callable_reference_resolutions: Arc::new(BTreeMap::new()),
        family_applications: Arc::new(BTreeMap::new()),
        match_resolutions: Arc::new(BTreeMap::new()),
        pattern_site_resolutions: Arc::new(BTreeMap::new()),
        flow_graph: Arc::new(FlowGraph::default()),
        entry_flow: FlowStateSummary::default(),
        exits: BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::from([]),
        internal_incidents: Arc::from([]),
        explanations: Arc::new(ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from([]),
        semantic_dependencies: Arc::from([]),
        constructor_super_initializer: None,
        dependency_fingerprint: ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
        observed_import_phases: Arc::new(BTreeMap::new()),
    }
}

#[test]
fn callable_internal_failure_fingerprint_ignores_local_incident_id() {
    let mut left = callable_analysis();
    let mut right = callable_analysis();
    let left_incident = InternalSemanticIncident {
        id: InternalSemanticIncidentId(3),
        kind: InternalSemanticIncidentKind::FlowInvariantViolation,
        module: ModuleId::universe_root(),
        callable: Some(left.callable.clone()),
        expression: None,
        range: None,
        details: InternalSemanticIncidentDetails::DivergentMutability {
            binding: phalcom_semantic::identity::BindingId(7),
            left: true,
            right: false,
        },
    };
    let mut right_incident = left_incident.clone();
    right_incident.id = InternalSemanticIncidentId(9);
    left.status = CallableAnalysisStatus::InternalFailure(left_incident.id);
    right.status = CallableAnalysisStatus::InternalFailure(right_incident.id);
    left.internal_incidents = Arc::from(vec![left_incident].into_boxed_slice());
    right.internal_incidents = Arc::from(vec![right_incident].into_boxed_slice());

    assert_eq!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));

    let mut changed = right.clone();
    let mut changed_incidents = changed.internal_incidents.to_vec();
    changed_incidents[0].details = InternalSemanticIncidentDetails::DivergentMutability {
        binding: phalcom_semantic::identity::BindingId(7),
        left: false,
        right: true,
    };
    changed.internal_incidents = Arc::from(changed_incidents.into_boxed_slice());
    assert_ne!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&changed));
}

#[test]
fn unlinked_interface_product_changes_when_local_declarations_change() {
    let id = module("surface");
    let foo = build_interface(id.clone(), "class Foo {}\n");
    let bar = build_interface(id, "class Bar {}\n");

    assert_ne!(unlinked_interface_product_fingerprint(&foo), unlinked_interface_product_fingerprint(&bar));
}

#[test]
fn unlinked_interface_product_changes_when_metadata_value_changes() {
    let id = module("metadata");
    let one = build_interface(id.clone(), "@!doc(\"one\")\nclass Foo {}\n");
    let two = build_interface(id, "@!doc(\"two\")\nclass Foo {}\n");

    assert_ne!(unlinked_interface_product_fingerprint(&one), unlinked_interface_product_fingerprint(&two));
}

#[test]
fn unlinked_interface_product_ignores_range_only_source_movement() {
    let id = module("ranges");
    let compact = build_interface(id.clone(), "import dep\nclass Foo {}\nexport Foo\n");
    let shifted = build_interface(id, "\n\nimport dep\n\nclass Foo {}\nexport Foo\n");

    assert_eq!(
        unlinked_interface_product_fingerprint(&compact),
        unlinked_interface_product_fingerprint(&shifted)
    );
    assert_ne!(unlinked_interface_input_fingerprint(&compact), unlinked_interface_input_fingerprint(&shifted));
}

#[test]
fn linked_interface_product_ignores_range_only_source_movement_but_input_tracks_it() {
    let id = module("linked_ranges");
    let compact = build_interface(id.clone(), "class Foo {}\nexport Foo\n");
    let shifted = build_interface(id.clone(), "\n\nclass Foo {}\n\nexport Foo\n");
    let universe = Arc::new(ProjectUniverse::new());
    let linked_compact = ModuleLinker::new(universe.clone(), BTreeMap::from([(id.clone(), compact)]))
        .link(id.clone(), &BTreeMap::new())
        .expect("compact link");
    let linked_shifted = ModuleLinker::new(universe, BTreeMap::from([(id.clone(), shifted)]))
        .link(id.clone(), &BTreeMap::new())
        .expect("shifted link");
    let compact_interface = &linked_compact.modules[&id].interface;
    let shifted_interface = &linked_shifted.modules[&id].interface;

    assert_eq!(
        linked_interface_product_fingerprint(compact_interface),
        linked_interface_product_fingerprint(shifted_interface)
    );
    assert_ne!(
        linked_interface_input_fingerprint(compact_interface),
        linked_interface_input_fingerprint(shifted_interface)
    );
}

#[test]
fn linked_interface_product_includes_metadata_semantics() {
    let id = module("linked_metadata");
    let one = build_interface(id.clone(), "@!doc(\"one\")\nclass Foo {}\nexport Foo\n");
    let two = build_interface(id.clone(), "@!doc(\"two\")\nclass Foo {}\nexport Foo\n");
    let universe = Arc::new(ProjectUniverse::new());
    let linked_one = ModuleLinker::new(universe.clone(), BTreeMap::from([(id.clone(), one)]))
        .link(id.clone(), &BTreeMap::new())
        .expect("first link");
    let linked_two = ModuleLinker::new(universe, BTreeMap::from([(id.clone(), two)]))
        .link(id.clone(), &BTreeMap::new())
        .expect("second link");

    assert_ne!(
        linked_interface_product_fingerprint(&linked_one.modules[&id].interface),
        linked_interface_product_fingerprint(&linked_two.modules[&id].interface)
    );
}

#[test]
fn declaration_shell_product_changes_when_kind_changes() {
    let left = declaration_shell();
    let mut right = left.clone();
    right.kind = KindId::RECORD_ROW;

    assert_ne!(declaration_shell_input_fingerprint(&left), declaration_shell_input_fingerprint(&right));
    assert_ne!(declaration_shell_product_fingerprint(&left), declaration_shell_product_fingerprint(&right));
}

#[test]
fn declaration_shell_fingerprint_tracks_intrinsic_application_and_runtime_owner() {
    use phalcom_semantic::declarations::{DeclarationSemanticForm, IntrinsicRuntimeOwner, IntrinsicTypeApplication};

    let ordinary = declaration_shell();
    let mut intrinsic = ordinary.clone();
    intrinsic.binding.semantic_form = DeclarationSemanticForm::Intrinsic(IntrinsicTypeApplication::Tuple);
    assert_ne!(declaration_shell_input_fingerprint(&ordinary), declaration_shell_input_fingerprint(&intrinsic));
    assert_ne!(
        declaration_shell_product_fingerprint(&ordinary),
        declaration_shell_product_fingerprint(&intrinsic)
    );

    intrinsic.binding.runtime_owner = Some(IntrinsicRuntimeOwner::Tuple);
    assert_ne!(declaration_shell_input_fingerprint(&ordinary), declaration_shell_input_fingerprint(&intrinsic));
    assert_ne!(
        declaration_shell_product_fingerprint(&ordinary),
        declaration_shell_product_fingerprint(&intrinsic)
    );
}

#[test]
fn declaration_shell_product_uses_stable_marker_when_generic_metadata_is_absent() {
    let mut left = declaration_shell();
    left.generic_signature = Some(GenericSignature::new(
        TypeParameterOwner::Declaration(left.declaration.clone()),
        Box::new([TypeParameterId(1)]),
    ));
    let mut right = left.clone();
    right.generic_signature = Some(GenericSignature::new(
        TypeParameterOwner::Declaration(right.declaration.clone()),
        Box::new([TypeParameterId(2)]),
    ));

    assert_eq!(declaration_shell_product_fingerprint(&left), declaration_shell_product_fingerprint(&right));
}

#[test]
fn declaration_shell_product_uses_stable_marker_when_supertype_metadata_is_absent() {
    let mut left = declaration_shell();
    left.supertype_template = Some(GenericSupertypeTemplate {
        declaration: left.declaration.clone(),
        supertype: TypeId(3),
        structural_form: None,
    });
    let mut right = left.clone();
    right.supertype_template = Some(GenericSupertypeTemplate {
        declaration: right.declaration.clone(),
        supertype: TypeId(4),
        structural_form: None,
    });

    assert_eq!(declaration_shell_product_fingerprint(&left), declaration_shell_product_fingerprint(&right));
}

#[test]
fn declaration_shell_fingerprint_uses_generic_and_supertype_structure() {
    let mut left = declaration_shell();
    left.generic_signature = Some(
        GenericSignature::new(TypeParameterOwner::Declaration(left.declaration.clone()), Box::new([TypeParameterId(1)]))
            .with_parameter_kind_shapes(Box::new(["Type".into()]))
            .with_parameter_metadata(Box::new([KindId::TYPE]), Box::new([phalcom_semantic::types::variance::Variance::Invariant]))
            .with_constraint_shapes(Box::new([])),
    );
    left.supertype_template = Some(GenericSupertypeTemplate {
        declaration: left.declaration.clone(),
        supertype: TypeId(3),
        structural_form: Some("Base<Type>".into()),
    });
    let mut same = left.clone();
    same.generic_signature = Some(
        GenericSignature::new(TypeParameterOwner::Declaration(same.declaration.clone()), Box::new([TypeParameterId(99)]))
            .with_parameter_kind_shapes(Box::new(["Type".into()]))
            .with_parameter_metadata(Box::new([KindId::TYPE]), Box::new([phalcom_semantic::types::variance::Variance::Invariant]))
            .with_constraint_shapes(Box::new([])),
    );
    same.supertype_template.as_mut().unwrap().supertype = TypeId(44);
    assert_eq!(declaration_shell_product_fingerprint(&left), declaration_shell_product_fingerprint(&same));

    let mut changed = same.clone();
    changed.supertype_template.as_mut().unwrap().structural_form = Some("Other<Type>".into());
    assert_ne!(declaration_shell_product_fingerprint(&left), declaration_shell_product_fingerprint(&changed));
}

#[test]
fn declaration_surface_source_input_ignores_bodies_and_field_defaults() {
    let body_one = source_surface_fingerprint(
        r#"
class Api {
  _value: Int = 1
  @class read() -> Int { 1 }
}
"#,
    );
    let body_two = source_surface_fingerprint(
        r#"
class Api {
  _value: Int = 2
  @class read() -> Int { 2 }
}
"#,
    );

    assert_eq!(body_one, body_two);
}

#[test]
fn declaration_surface_source_input_tracks_contract_annotations() {
    let int_surface = source_surface_fingerprint(
        r#"
class Api {
  @class read(_ value: Int) -> Int { value }
}
"#,
    );
    let string_surface = source_surface_fingerprint(
        r#"
class Api {
  @class read(_ value: String) -> Int { value }
}
"#,
    );

    assert_ne!(int_surface, string_surface);
}

#[test]
fn declaration_surface_source_input_tracks_accessor_generic_contracts() {
    let setter_number = source_surface_fingerprint(
        r#"
class Api {
  value<T>=(_ next: T) where T <: Number { }
}
"#,
    );
    let setter_object = source_surface_fingerprint(
        r#"
class Api {
  value<T>=(_ next: T) where T <: Object { }
}
"#,
    );
    let index_number = source_surface_fingerprint(
        r#"
class Api {
  [_ key: T]<T>=(_ value: T) where T <: Number { }
}
"#,
    );
    let index_object = source_surface_fingerprint(
        r#"
class Api {
  [_ key: T]<T>=(_ value: T) where T <: Object { }
}
"#,
    );

    assert_ne!(
        setter_number, setter_object,
        "setter generic constraints must invalidate its declaration surface"
    );
    assert_ne!(index_number, index_object, "index generic constraints must invalidate its declaration surface");
}

#[test]
fn declaration_surface_product_ignores_type_evidence_provenance_but_input_tracks_it() {
    let owner = declaration("Surface");
    let mut left = DeclarationSurface::new(Some(owner.clone()));
    left.add_field(
        DispatchSide::Instance,
        "value",
        TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::DeveloperAnnotation).with_range(SourceRange { start: 1, end: 2 }),
    );
    let mut right = DeclarationSurface::new(Some(owner));
    right.add_field(
        DispatchSide::Instance,
        "value",
        TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::DeveloperAnnotation).with_range(SourceRange { start: 101, end: 102 }),
    );

    assert_eq!(declaration_surface_product_fingerprint(&left), declaration_surface_product_fingerprint(&right));
    assert_ne!(declaration_surface_input_fingerprint(&left), declaration_surface_input_fingerprint(&right));
}

#[test]
fn declaration_surface_product_includes_callable_generic_contract() {
    let owner = declaration("GenericSurface");
    let selector = Selector::getter("value").expect("selector");
    let mut left = DeclarationSurface::new(Some(owner.clone()));
    let mut left_signature = CallableSignature::new(
        selector.clone(),
        Vec::new(),
        TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::DeveloperAnnotation),
    );
    left_signature.generics = Some(
        GenericSignature::new(TypeParameterOwner::Declaration(owner.clone()), Box::new([TypeParameterId(1)]))
            .with_parameter_kind_shapes(Box::new(["Type".into()]))
            .with_parameter_metadata(Box::new([KindId::TYPE]), Box::new([phalcom_semantic::types::variance::Variance::Invariant])),
    );
    left.add_callable(DispatchSide::Instance, left_signature);

    let mut right = DeclarationSurface::new(Some(owner.clone()));
    let mut right_signature = CallableSignature::new(selector, Vec::new(), TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::DeveloperAnnotation));
    right_signature.generics = Some(
        GenericSignature::new(TypeParameterOwner::Declaration(owner), Box::new([TypeParameterId(2)]))
            .with_parameter_kind_shapes(Box::new(["RecordFields".into()]))
            .with_parameter_metadata(
                Box::new([KindId::RECORD_ROW]),
                Box::new([phalcom_semantic::types::variance::Variance::Invariant]),
            ),
    );
    right.add_callable(DispatchSide::Instance, right_signature);

    assert_ne!(declaration_surface_product_fingerprint(&left), declaration_surface_product_fingerprint(&right));
}

#[test]
fn declaration_surface_product_includes_callable_semantic_kind() {
    let owner = declaration("KindSurface");
    let selector = Selector::getter("value").expect("selector");
    let mut ordinary = DeclarationSurface::new(Some(owner.clone()));
    ordinary.add_callable(
        DispatchSide::Instance,
        CallableSignature::new(
            selector.clone(),
            Vec::new(),
            TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::DeveloperAnnotation),
        ),
    );

    let mut constructor = DeclarationSurface::new(Some(owner));
    constructor.add_callable(
        DispatchSide::Instance,
        CallableSignature::new(selector, Vec::new(), TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::DeveloperAnnotation))
            .with_kind(CallableSemanticKind::Constructor),
    );

    assert_ne!(
        declaration_surface_product_fingerprint(&ordinary),
        declaration_surface_product_fingerprint(&constructor)
    );
}

#[test]
fn callable_signature_product_includes_generics_effects_and_lifecycle() {
    let base = semantic_signature();

    let mut generic = base.clone();
    generic.generics = Some(GenericSignature::with_constraints(
        TypeParameterOwner::Callable(generic.callable.clone()),
        Box::new([TypeParameterId(1)]),
        Box::new([GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(TypeId(2)),
            upper: TypeTerm::Canonical(TypeId(3)),
        }]),
    ));
    assert_ne!(callable_signature_product_fingerprint(&base), callable_signature_product_fingerprint(&generic));

    let mut effects = base.clone();
    effects.effects = EffectSpec::Pure;
    assert_ne!(callable_signature_product_fingerprint(&base), callable_signature_product_fingerprint(&effects));

    let mut lifecycle = base.clone();
    lifecycle.lifecycle = NativeLifecycleSpec {
        since: Some("0.2"),
        deprecated_since: None,
        replacement: None,
    };
    assert_ne!(
        callable_signature_product_fingerprint(&base),
        callable_signature_product_fingerprint(&lifecycle)
    );
}

#[test]
fn callable_signature_product_ignores_source_movement_but_input_tracks_it() {
    let left = semantic_signature();
    let mut right = left.clone();
    right.source = Some(SemanticSourceSpan::new(ModuleId::universe_root(), SourceRange { start: 110, end: 120 }));

    assert_eq!(callable_signature_product_fingerprint(&left), callable_signature_product_fingerprint(&right));
    assert_ne!(callable_signature_input_fingerprint(&left), callable_signature_input_fingerprint(&right));
}

#[test]
fn callable_signature_input_tracks_source_end_when_start_is_unchanged() {
    let left = semantic_signature();
    let mut right = left.clone();
    right.source = Some(SemanticSourceSpan::new(ModuleId::universe_root(), SourceRange { start: 10, end: 21 }));

    assert_eq!(callable_signature_product_fingerprint(&left), callable_signature_product_fingerprint(&right));
    assert_ne!(callable_signature_input_fingerprint(&left), callable_signature_input_fingerprint(&right));
}

#[test]
fn callable_body_product_includes_binding_state() {
    let mut left = callable_analysis();
    let binding = phalcom_semantic::BindingId(1);
    left.bindings.insert(
        binding,
        BindingState::new(
            binding,
            "value",
            SourceRange { start: 1, end: 6 },
            None,
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
            false,
        ),
    );
    let mut right = left.clone();
    right.bindings.get_mut(&binding).expect("binding").current = TypeKnowledge::established(TypeId(2), EvidenceOrigin::Flow);

    assert_ne!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_includes_expression_denotation_and_status() {
    let expression_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let mut left = callable_analysis();
    left.expressions.insert(
        expression_id,
        ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 4 },
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
        ),
    );
    let mut right = left.clone();
    let expression = right.expressions.get_mut(&expression_id).expect("expression");
    expression.denotation = Some(SemanticDenotation::TypeForm(TypeId(3)));
    expression.status = AnalysisStatus::DynamicBoundary(phalcom_semantic::DynamicReason::RuntimeReflection);

    assert_ne!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_includes_resolved_callable_identity() {
    let expression_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let mut left = callable_analysis();
    left.expressions.insert(
        expression_id,
        ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 4 },
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::CallableSignature),
        ),
    );
    let mut right = left.clone();
    right.expressions.get_mut(&expression_id).expect("expression").callable = Some(semantic_signature().callable);
    assert_ne!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_ignores_diagnostic_cause_renumbering() {
    let expression_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let mut left = callable_analysis();
    left.expressions.insert(
        expression_id,
        ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 4 },
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
        )
        .with_status(AnalysisStatus::Invalid(DiagnosticCauseId(1))),
    );
    left.expressions.get_mut(&expression_id).expect("expression").causal_invalidity =
        phalcom_semantic::checker::causal::CausalInvalidity::One(DiagnosticCauseId(1));

    let mut right = left.clone();
    right.expressions.get_mut(&expression_id).expect("expression").status = AnalysisStatus::Invalid(DiagnosticCauseId(99));
    right.expressions.get_mut(&expression_id).expect("expression").causal_invalidity =
        phalcom_semantic::checker::causal::CausalInvalidity::One(DiagnosticCauseId(99));

    assert_eq!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_includes_flow_exit_and_callable_status() {
    let left = callable_analysis();
    let mut right = left.clone();
    right.entry_flow.fact_count = 1;
    right.exits.unreachable = true;
    right.status = CallableAnalysisStatus::Partial;

    assert_ne!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_distinguishes_flow_evidence_status() {
    let binding = phalcom_semantic::identity::BindingId(9);
    let mut established = callable_analysis();
    established.entry_flow.bindings.insert(
        binding,
        FlowBindingSummary {
            knowledge: TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
            contract: None,
            consistency: BindingConsistency::Unconstrained,
            mutable: false,
            initialization: phalcom_semantic::checker::flow::InitializationState::DefinitelyInitialized,
        },
    );
    let mut assumed = established.clone();
    assumed.entry_flow.bindings.get_mut(&binding).expect("flow binding").knowledge = TypeKnowledge::assumed(TypeId(1), EvidenceOrigin::Flow);

    assert_ne!(callable_body_product_fingerprint(&established), callable_body_product_fingerprint(&assumed));
}

#[test]
fn callable_body_product_distinguishes_unknown_and_dynamic_flow() {
    let binding = phalcom_semantic::identity::BindingId(10);
    let mut unknown = callable_analysis();
    unknown.entry_flow.bindings.insert(
        binding,
        FlowBindingSummary {
            knowledge: TypeKnowledge::Unknown(phalcom_semantic::UnknownReason::NoTypeEvidence),
            contract: None,
            consistency: BindingConsistency::Unconstrained,
            mutable: false,
            initialization: phalcom_semantic::checker::flow::InitializationState::DefinitelyInitialized,
        },
    );
    let mut dynamic = unknown.clone();
    dynamic.entry_flow.bindings.get_mut(&binding).expect("flow binding").knowledge = TypeKnowledge::Dynamic(phalcom_semantic::DynamicReason::RuntimeReflection);

    assert_ne!(callable_body_product_fingerprint(&unknown), callable_body_product_fingerprint(&dynamic));
}

#[test]
fn changing_only_initialization_safety_changes_callable_product_fingerprint() {
    let binding = phalcom_semantic::identity::BindingId(11);
    let expr_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let mut definite = callable_analysis();
    definite.bindings.insert(
        binding,
        BindingState::new(
            binding,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
            true,
        ),
    );
    let mut expr = ExpressionAnalysis::ready(expr_id, SourceRange::default(), TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow));
    expr.binding_read_safety = Some(phalcom_semantic::checker::flow::BindingReadSafety::ProvenInitialized);
    expr.binding_read = Some(binding);
    definite.expressions.insert(expr_id, expr);

    let mut another_binding = definite.clone();
    another_binding.expressions.get_mut(&expr_id).expect("expr").binding_read = Some(phalcom_semantic::identity::BindingId(12));
    assert_ne!(
        callable_body_product_fingerprint(&definite),
        callable_body_product_fingerprint(&another_binding),
        "changing the resolved local binding must invalidate the callable product"
    );

    let mut runtime_check = definite.clone();
    runtime_check.bindings.get_mut(&binding).expect("binding").initialization = phalcom_semantic::checker::flow::InitializationState::MaybeInitialized;
    runtime_check.expressions.get_mut(&expr_id).expect("expr").binding_read_safety =
        Some(phalcom_semantic::checker::flow::BindingReadSafety::RuntimeCheckRequired);

    assert_ne!(callable_body_product_fingerprint(&definite), callable_body_product_fingerprint(&runtime_check));
}

#[test]
fn callable_body_product_ignores_explanation_presentation() {
    let expression_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let mut left = callable_analysis();
    let mut left_arena = ExplanationArena::new();
    let left_explanation = left_arena.alloc(
        ExplanationStep::Literal {
            expression: expression_id,
            ty: TypeId(1),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Syntax,
        Vec::new(),
    );
    left.explanations = Arc::new(left_arena);
    left.expressions.insert(
        expression_id,
        ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 4 },
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
        )
        .with_explanation(left_explanation),
    );

    let mut right = callable_analysis();
    let mut right_arena = ExplanationArena::new();
    let right_explanation = right_arena.alloc(
        ExplanationStep::Literal {
            expression: expression_id,
            ty: TypeId(2),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Syntax,
        Vec::new(),
    );
    right.explanations = Arc::new(right_arena);
    right.expressions.insert(
        expression_id,
        ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 4 },
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
        )
        .with_explanation(right_explanation),
    );

    assert_eq!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_ignores_diagnostic_details() {
    let mut left = callable_analysis();
    left.diagnostics = Arc::from(
        vec![
            SemanticDiagnostic::error_in(
                ModuleId::universe_root(),
                DiagnosticCode::TypeMismatch,
                "mismatch",
                SourceRange { start: 1, end: 2 },
            )
            .with_note("first note"),
        ]
        .into_boxed_slice(),
    );
    let mut right = left.clone();
    right.diagnostics = Arc::from(
        vec![
            SemanticDiagnostic::error_in(
                ModuleId::universe_root(),
                DiagnosticCode::TypeMismatch,
                "mismatch",
                SourceRange { start: 1, end: 2 },
            )
            .with_note("different note"),
        ]
        .into_boxed_slice(),
    );

    assert_eq!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_ignores_source_ranges_and_type_provenance() {
    let expression_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let mut left = callable_analysis();
    left.body_range = SourceRange { start: 1, end: 4 };
    left.expressions.insert(
        expression_id,
        ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 4 },
            TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow).with_range(SourceRange { start: 1, end: 4 }),
        ),
    );
    let mut right = left.clone();
    right.body_range = SourceRange { start: 101, end: 104 };
    right.expressions.get_mut(&expression_id).expect("expression").range = SourceRange { start: 101, end: 104 };
    right.expressions.get_mut(&expression_id).expect("expression").knowledge =
        TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow).with_range(SourceRange { start: 101, end: 104 });

    assert_eq!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn callable_body_product_ignores_flow_source_ranges() {
    let mut left = callable_analysis();
    let node = Arc::make_mut(&mut left.flow_graph).add_node(FlowNodeKind::Entry, SourceRange { start: 1, end: 4 });
    Arc::make_mut(&mut left.flow_graph).entry = Some(node);
    let mut right = left.clone();
    Arc::make_mut(&mut right.flow_graph).nodes.get_mut(&node).expect("flow node").range = SourceRange { start: 101, end: 104 };

    assert_eq!(callable_body_product_fingerprint(&left), callable_body_product_fingerprint(&right));
}

#[test]
fn module_diagnostics_product_includes_secondary_details() {
    let module = ModuleId::universe_root();
    let left = SemanticDiagnostic::error_in(module.clone(), DiagnosticCode::TypeMismatch, "mismatch", SourceRange { start: 1, end: 2 }).with_note("first note");
    let right =
        SemanticDiagnostic::error_in(module.clone(), DiagnosticCode::TypeMismatch, "mismatch", SourceRange { start: 1, end: 2 }).with_note("different note");

    assert_ne!(
        module_diagnostics_product_fingerprint(&module, &[left]),
        module_diagnostics_product_fingerprint(&module, &[right])
    );
}

#[test]
fn module_diagnostics_product_ignores_snapshot_local_cause_numbers() {
    let module = ModuleId::universe_root();
    let left = SemanticDiagnostic::error_in(module.clone(), DiagnosticCode::TypeMismatch, "mismatch", SourceRange { start: 1, end: 2 })
        .with_root_cause(DiagnosticCauseId(17));
    let right = SemanticDiagnostic::error_in(module.clone(), DiagnosticCode::TypeMismatch, "mismatch", SourceRange { start: 1, end: 2 })
        .with_root_cause(DiagnosticCauseId(91));

    assert_eq!(
        module_diagnostics_product_fingerprint(&module, &[left]),
        module_diagnostics_product_fingerprint(&module, &[right]),
    );
}

#[test]
fn semantic_component_product_changes_when_resolved_target_changes() {
    let exporter_a = module("exporter_a");
    let exporter_b = module("exporter_b");
    let importer = module("importer");
    let interfaces = BTreeMap::from([
        (exporter_a.clone(), build_interface(exporter_a.clone(), "class Foo {}\nexport Foo\n")),
        (exporter_b.clone(), build_interface(exporter_b.clone(), "class Foo {}\nexport Foo\n")),
        (importer.clone(), build_interface(importer.clone(), "from .target import Foo\n")),
    ]);
    let universe = Arc::new(ProjectUniverse::new());
    let linked_a = ModuleLinker::new(universe.clone(), interfaces.clone())
        .link(importer.clone(), &BTreeMap::from([((importer.clone(), ".target".to_string()), exporter_a)]))
        .expect("link to exporter A");
    let linked_b = ModuleLinker::new(universe, interfaces)
        .link(importer.clone(), &BTreeMap::from([((importer, ".target".to_string()), exporter_b)]))
        .expect("link to exporter B");

    assert_ne!(
        semantic_component_product_fingerprint(&linked_a),
        semantic_component_product_fingerprint(&linked_b)
    );
}

#[test]
fn constructor_products_refresh_when_attributes_and_field_defaults_change() {
    use phalcom_common::selector::{Selector, SelectorSlot};
    use phalcom_semantic::identity::DispatchSide;
    use phalcom_semantic::session::SemanticWorkspaceSession;

    let module = module("constructor_products");
    let initial = "@construct\nclass Parent {\n  _required: Int\n  _defaulted: Int = 1\n}\nclass Child is Parent {\n  _child: Int = 3\n}\n";
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .update(super::support::single_module_input(module.clone(), initial, 1))
        .expect("semantic test update should succeed");
    let owner = DeclarationId::new(module.clone(), "Parent".into());
    let generated = CallableId::new(
        owner.clone(),
        Selector::method("new", [SelectorSlot::Label("required".into())]).unwrap(),
        DispatchSide::Class,
    );
    assert_eq!(
        first.snapshot.constructor_initializations.get(&generated).map(|product| product.origin),
        Some(phalcom_semantic::ConstructorOrigin::Construct)
    );

    let changed_default = "@construct\nclass Parent {\n  _required: Int\n  _defaulted: Int = 9\n}\nclass Child is Parent {\n  _child: Int = 3\n}\n";
    let second = session
        .update(super::support::single_module_input(module.clone(), changed_default, 2))
        .expect("semantic test update should succeed");
    assert_ne!(
        first
            .snapshot
            .constructor_initializations
            .get(&generated)
            .map(|product| product.field_input_fingerprint),
        second
            .snapshot
            .constructor_initializations
            .get(&generated)
            .map(|product| product.field_input_fingerprint),
        "editing a field declaration/default changes the canonical construction product"
    );

    let without_attribute = "class Parent {\n  _required: Int\n  _defaulted: Int = 2\n}\nclass Child is Parent {\n  _child: Int = 3\n}\n";
    let third = session
        .update(super::support::single_module_input(module.clone(), without_attribute, 3))
        .expect("semantic test update should succeed");
    assert!(
        third.snapshot.constructor_initializations.get(&generated).is_none(),
        "removed @construct path must not survive as a stale product"
    );
    let default_only = CallableId::new(owner.clone(), Selector::method("new", Vec::new()).unwrap(), DispatchSide::Class);
    assert_eq!(
        third.snapshot.constructor_initializations.get(&default_only).map(|product| product.origin),
        Some(phalcom_semantic::ConstructorOrigin::DefaultOnly)
    );
    let child_owner = DeclarationId::new(module.clone(), "Child".into());
    let child_default = CallableId::new(child_owner, Selector::method("new", Vec::new()).unwrap(), DispatchSide::Class);
    assert!(
        third.snapshot.constructor_initializations.get(&child_default).is_none(),
        "a default-only child path cannot omit the parent's required initialization lifecycle"
    );

    let no_defaults = "class Parent {}\nclass Child is Parent {\n  _child: Int = 3\n}\n";
    let fourth = session
        .update(super::support::single_module_input(module.clone(), no_defaults, 4))
        .expect("semantic test update should succeed");
    assert!(
        fourth.snapshot.constructor_initializations.get(&default_only).is_none(),
        "removing the final default must remove the default-only path"
    );
    assert_eq!(
        fourth.snapshot.constructor_initializations.get(&child_default).map(|product| product.origin),
        Some(phalcom_semantic::ConstructorOrigin::DefaultOnly),
        "removing the parent's lifecycle refreshes the child's eligible default-only path"
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold = cold
        .update(super::support::single_module_input(module, no_defaults, 1))
        .expect("semantic test update should succeed");
    assert_eq!(
        &*fourth.snapshot.constructor_initializations, &*cold.snapshot.constructor_initializations,
        "incremental constructor products must match a cold final snapshot"
    );
}

#[test]
fn parent_constructor_product_change_revalidates_explicit_super_body() {
    use phalcom_common::selector::Selector;
    use phalcom_semantic::db::QueryKey;
    use phalcom_semantic::identity::DispatchSide;
    use phalcom_semantic::session::SemanticWorkspaceSession;

    let module = module("constructor_super_dependency");
    let initial = "class ParentDefaults {\n  _base: Int = 1\n}\nclass Child is ParentDefaults {\n  @constructor new() { super.new() }\n}\n";
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .update(super::support::single_module_input(module.clone(), initial, 1))
        .expect("semantic test update should succeed");
    let parent = DeclarationId::new(module.clone(), "ParentDefaults".into());
    let parent_factory = CallableId::new(parent.clone(), Selector::method("new", Vec::new()).unwrap(), DispatchSide::Class);
    let parent_key = QueryKey::ConstructorInitialization(parent);
    let initial_field_fingerprint = first.snapshot.constructor_initializations.get(&parent_factory).unwrap().field_input_fingerprint;
    let initial_owner_fingerprint = session.db().ready_product_fingerprint(&parent_key);
    let child = CallableId::new(
        DeclarationId::new(module.clone(), "Child".into()),
        Selector::method("new", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );
    assert!(first.snapshot.callable_analyses[&child].constructor_super_initializer.is_some());
    assert!(
        session
            .db()
            .index()
            .dependents_of(&parent_key)
            .is_some_and(|dependents| dependents.contains(&QueryKey::CallableBody(child.clone()))),
        "the child body query must publish its constructor-owner dependency edge"
    );

    let changed = "class ParentDefaults {\n  _base: Int = 2\n}\nclass Child is ParentDefaults {\n  @constructor new() { super.new() }\n}\n";
    let second = session
        .update(super::support::single_module_input(module, changed, 2))
        .expect("semantic test update should succeed");
    assert_ne!(
        initial_field_fingerprint,
        second
            .snapshot
            .constructor_initializations
            .get(&parent_factory)
            .unwrap()
            .field_input_fingerprint,
        "changing a parent default changes its constructor input fingerprint"
    );
    assert_ne!(
        initial_owner_fingerprint,
        session.db().ready_product_fingerprint(&parent_key),
        "the owner query fingerprint publishes the changed constructor product"
    );
    assert!(
        session.db().revision_recomputed_keys().any(|key| key == &QueryKey::CallableBody(child.clone())),
        "a changed direct-parent construction product must recompute the subclass constructor body"
    );
    assert!(second.snapshot.callable_analyses[&child].constructor_super_initializer.is_some());
}
