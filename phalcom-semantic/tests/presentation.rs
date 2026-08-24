use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_semantic::BlockReason;
use phalcom_semantic::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, FlowStateSummary};
use phalcom_semantic::checker::flow::graph::FlowGraph;
use phalcom_semantic::db::ProductFingerprint;
use phalcom_semantic::explain::ExplanationArena;
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, DiagnosticCauseId, ExpressionId, LocalExpressionId, ModuleId};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceAuthority, TypeKnowledge, UnknownReason};
use phalcom_semantic::{DispatchSide, FormalPresentation, FormalSiteId, SemanticPresentationIndex, TypePresenter, TypeStore};
use std::collections::BTreeMap;
use std::sync::Arc;

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
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
        presenter.present_knowledge(&TypeKnowledge::known(int, EvidenceAuthority::Proven)),
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
        TypeKnowledge::known(int, EvidenceAuthority::Proven),
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
        TypeKnowledge::known(int, EvidenceAuthority::Proven),
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
        explanations: Arc::new(ExplanationArena::default()),
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
        explanations: Arc::new(ExplanationArena::default()),
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
