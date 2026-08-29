use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_native_meta::{EffectSpec, ImplementationKind, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_semantic::BlockReason;
use phalcom_semantic::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, FlowStateSummary};
use phalcom_semantic::checker::flow::graph::FlowGraph;
use phalcom_semantic::db::ProductFingerprint;
use phalcom_semantic::explain::ExplanationArena;
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, DiagnosticCauseId, ExpressionId, LocalExpressionId, ModuleId};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_semantic::{
    AdvisoryPresenter, CallableParameterSemantic, CallablePresentation, CallableSemanticSignature, DispatchSide, FormalPresentation, FormalSiteId,
    SemanticPresentationIndex, TypePresenter, TypeStore,
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
}

#[test]
fn advisory_presenter_formats_canonical_runtime_shapes() {
    let int = declaration("Int");
    let string = declaration("String");
    let int_shape = phalcom_semantic::ValueShape::Instance(int.clone());
    let string_shape = phalcom_semantic::ValueShape::Instance(string);

    assert_eq!(AdvisoryPresenter::present_shape(&int_shape), "Int");
    assert_eq!(
        AdvisoryPresenter::present_shape(&phalcom_semantic::ValueShape::ClassObject(int.clone())),
        "Int class"
    );
    assert_eq!(
        AdvisoryPresenter::present_shape(&phalcom_semantic::ValueShape::Tuple(Arc::from([int_shape.clone(), string_shape.clone()]))),
        "(Int, String)"
    );
    assert_eq!(
        AdvisoryPresenter::present_shape(&phalcom_semantic::ValueShape::ExactList(Arc::from([int_shape.clone(), int_shape.clone()]))),
        "List<Int>"
    );
    assert_eq!(
        AdvisoryPresenter::present_shape(&phalcom_semantic::ValueShape::record([
            ("value", int_shape.clone()),
            ("name", string_shape.clone()),
        ])),
        "#{name: String, value: Int}"
    );
    assert_eq!(
        AdvisoryPresenter::present_shape(&phalcom_semantic::ValueShape::bounded_union([int_shape, string_shape])),
        "Int | String"
    );
}

#[test]
fn callable_presentation_joins_canonical_signature_and_source_kind() {
    let owner = declaration("Owner");
    let selector = Selector::method("value", vec![]).expect("selector");
    let callable = phalcom_semantic::CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    let mut store = TypeStore::new();
    let int = store.nominal(declaration("Int"));
    let signature = CallableSemanticSignature {
        callable: callable.clone(),
        owner,
        side: DispatchSide::Instance,
        selector,
        generics: None,
        parameters: vec![CallableParameterSemantic::new(
            phalcom_semantic::CallableParameterId::new(callable.clone(), 0),
            "value",
            phalcom_semantic::DeclaredTypeFact::known(
                phalcom_semantic::types::TypeTerm::Canonical(int),
                phalcom_semantic::DeclaredTypeBasis::SourceAnnotation,
            ),
        )]
        .into_boxed_slice(),
        declared_return: phalcom_semantic::DeclaredTypeFact::known(
            phalcom_semantic::types::TypeTerm::Canonical(int),
            phalcom_semantic::DeclaredTypeBasis::SourceAnnotation,
        ),
        inferred_return: None,
        source: None,
        implementation: ImplementationKind::Source,
        native_id: None,
        effects: EffectSpec::Unknown,
        raises: RaisesSpec::Unknown,
        flow: ReturnFlowSpec::Value,
        lifecycle: NativeLifecycleSpec::UNKNOWN,
    };

    let presentation = CallablePresentation::from_signature(&signature, None, &TypePresenter::new(&store));
    assert_eq!(presentation.callable, callable);
    assert_eq!(presentation.selector, "value()");
    assert_eq!(presentation.owner_name, "Owner".into());
    assert_eq!(presentation.parameters[0].type_, FormalPresentation::Known("Int".into()));
    assert_eq!(presentation.return_type, FormalPresentation::Known("Int".into()));
    assert_eq!(presentation.documentation, None);
}

#[test]
fn type_presenter_formats_canonical_formal_shapes() {
    let mut store = TypeStore::new();
    let int = store.nominal(declaration("Int"));
    let string = store.nominal(declaration("String"));
    let list_kind = store.arrow_kind(Box::new([phalcom_semantic::KindId::TYPE]), phalcom_semantic::KindId::TYPE);
    let list_form = store.nominal_form(declaration("List"), list_kind);
    let list_int = store.apply_type_form(list_form, &[int]).expect("List<Int> kind-checks");
    let union = store.union(&[string, int]);

    let presenter = TypePresenter::new(&store);
    assert_eq!(presenter.present_type(int), "Int");
    assert_eq!(presenter.present_type(list_int), "List<Int>");
    assert_eq!(presenter.present_type(union), "Int | String");
    assert_eq!(
        presenter.present_knowledge(&TypeKnowledge::established(int, EvidenceOrigin::Flow)),
        FormalPresentation::Known("Int".into())
    );
    assert_eq!(
        presenter.present_knowledge(&TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)),
        FormalPresentation::Dynamic
    );
    assert_eq!(
        presenter.present_knowledge(&TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)),
        FormalPresentation::Unknown
    );
    let blocked = ExpressionAnalysis::ready(
        ExpressionId::new(BodyId(1), LocalExpressionId(2)),
        SourceRange { start: 8, end: 9 },
        TypeKnowledge::established(int, EvidenceOrigin::Flow),
    )
    .with_status(AnalysisStatus::Blocked(BlockReason::RecursiveFixpoint));
    assert_eq!(presenter.present_expression(&blocked), FormalPresentation::Blocked);
}

#[test]
fn type_presenter_formats_generic_specializations_from_canonical_products() {
    let mut store = TypeStore::new();
    let int = store.nominal(declaration("Int"));
    let box_decl = declaration("Box");
    let box_kind = store.arrow_kind(Box::new([phalcom_semantic::KindId::TYPE]), phalcom_semantic::KindId::TYPE);
    let box_form = store.nominal_form(box_decl, box_kind);
    let specialized = store.apply_type_form(box_form, &[int]).expect("Box<Int> kind-checks");
    let presenter = TypePresenter::new(&store);

    assert_eq!(presenter.present_type(specialized), "Box<Int>");
    assert_eq!(
        presenter.present_type(specialized),
        presenter.present_type(specialized),
        "unchanged formal product has stable presentation"
    );
}

#[test]
fn presentation_index_projects_formal_sites_without_reanalysis() {
    let module = ModuleId::core();
    let mut store = TypeStore::new();
    let int = store.nominal(declaration("Int"));
    let callable = CallableId::new(declaration("Demo"), Selector::getter("value").unwrap(), DispatchSide::Instance);
    let expression_id = ExpressionId::new(BodyId(1), LocalExpressionId(1));
    let expression = ExpressionAnalysis::ready(
        expression_id,
        SourceRange { start: 4, end: 7 },
        TypeKnowledge::established(int, EvidenceOrigin::Flow),
    );
    let analysis = CallableAnalysis {
        callable: callable.clone(),
        body_range: SourceRange { start: 0, end: 12 },
        expressions: BTreeMap::from([(expression_id, expression)]),
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(FlowGraph::default()),
        entry_flow: FlowStateSummary::default(),
        exits: Default::default(),
        diagnostics: Arc::from([]),
        internal_incidents: Arc::from([]),
        explanations: Arc::new(ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from([]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: ProductFingerprint::new(7),
        status: CallableAnalysisStatus::Complete,
    };

    let presenter = TypePresenter::new(&store);
    let index = SemanticPresentationIndex::from_callable_analysis(module.clone(), &analysis, &presenter);

    let callable_site = index.get(&FormalSiteId::Callable(callable.clone())).expect("callable site");
    assert_eq!(callable_site.module, module);
    assert_eq!(callable_site.range, SourceRange { start: 0, end: 12 });
    assert_eq!(callable_site.presentation, FormalPresentation::Known("Ready".into()));

    let expression_site = index
        .get(&FormalSiteId::Expression {
            callable,
            expression: expression_id,
        })
        .expect("expression site");
    assert_eq!(expression_site.presentation, FormalPresentation::Known("Int".into()));
    assert_eq!(expression_site.range, SourceRange { start: 4, end: 7 });
    assert_eq!(index.len(), 2);
}

#[test]
fn presentation_preserves_non_ready_formal_states() {
    let module = ModuleId::core();
    let store = TypeStore::new();
    let presenter = TypePresenter::new(&store);

    assert_eq!(presenter.present_callable_status(CallableAnalysisStatus::Blocked), FormalPresentation::Blocked);
    assert_eq!(
        presenter.present_callable_status(CallableAnalysisStatus::Cancelled),
        FormalPresentation::Cancelled
    );
    assert_eq!(
        presenter.present_callable_status(CallableAnalysisStatus::BudgetExceeded),
        FormalPresentation::BudgetExceeded
    );
    assert_eq!(presenter.present_callable_status(CallableAnalysisStatus::Partial), FormalPresentation::Partial);

    let expression = ExpressionAnalysis::invalid(
        ExpressionId::new(BodyId(2), LocalExpressionId(1)),
        SourceRange { start: 1, end: 2 },
        DiagnosticCauseId(1),
    );
    let callable = CallableId::new(declaration("Invalid"), Selector::getter("body").unwrap(), DispatchSide::Instance);
    let analysis = CallableAnalysis {
        callable: callable.clone(),
        body_range: SourceRange { start: 0, end: 3 },
        expressions: BTreeMap::from([(expression.id, expression)]),
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(FlowGraph::default()),
        entry_flow: FlowStateSummary::default(),
        exits: Default::default(),
        diagnostics: Arc::from([]),
        internal_incidents: Arc::from([]),
        explanations: Arc::new(ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from([]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: ProductFingerprint::new(8),
        status: CallableAnalysisStatus::Complete,
    };
    let index = SemanticPresentationIndex::from_callable_analysis(module, &analysis, &presenter);
    assert_eq!(
        index
            .get(&FormalSiteId::Expression {
                callable,
                expression: expression_id_for(&analysis)
            })
            .unwrap()
            .presentation,
        FormalPresentation::Invalid
    );
}

fn expression_id_for(analysis: &CallableAnalysis) -> phalcom_semantic::identity::ExpressionId {
    *analysis.expressions.keys().next().expect("invalid expression")
}
