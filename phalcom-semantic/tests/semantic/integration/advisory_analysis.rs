use std::collections::BTreeMap;
use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::advisory::{AdvisoryAgreement, AdvisoryModuleProduct, AdvisoryProductStatus, AdvisoryWorkspace, compare_expression};
use phalcom_semantic::checker::BindingConsistency;
use phalcom_semantic::checker::analysis::{AnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, SourceOwner, SourceSiteId, SourceSiteLocalId};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use phalcom_semantic::{AdvisoryConfidence, AdvisoryFact, AdvisoryOrigin, FormalFactRef, FormalFactStatus, ModuleId, ValueShape};

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([(module.clone(), linked_module)]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let unit = Arc::new(ParsedModuleUnit::new(
        module.clone(),
        ModuleKind::Module,
        None,
        Arc::from(source),
        Arc::new(parsed.program),
    ));
    SemanticWorkspaceInput {
        linked,
        sources: BTreeMap::from([(module, unit)]),
        generation,
    }
}

#[test]
fn snapshot_publishes_formal_source_and_advisory_products_together() {
    let module = module_id();
    let source = "class Box { value() -> Int { return 1 } }\nlet top = 1\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    assert!(!update.snapshot.formal_projection().is_empty());
    assert_eq!(
        update
            .snapshot
            .formal_fact_at(&module, source.find('1').expect("formal literal offset"))
            .map(|site| site.status),
        Some(FormalFactStatus::Ready)
    );
    assert!(update.snapshot.source_index().module(&module).is_some());
    assert!(update.snapshot.advisory().is_complete());

    let top_binding = update
        .snapshot
        .source_index()
        .module(&module)
        .unwrap()
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "top")
        .unwrap();
    assert_eq!(
        update.snapshot.advisory().binding(&top_binding.declaration_site).map(|fact| &fact.shape),
        Some(&ValueShape::Instance(DeclarationId::new(ModuleId::core(), "Int".into())))
    );
    let site_view = update.snapshot.semantic_site_at(&module, top_binding.declaration_range.start);
    assert_eq!(site_view.source_site.as_ref(), Some(&top_binding.declaration_site));
    assert!(site_view.advisory.is_some());

    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Box".into()),
        Selector::method("value", []).unwrap(),
        DispatchSide::Instance,
    );
    let summary = update.snapshot.advisory().callable(&callable).expect("advisory callable summary");
    assert_eq!(summary.status, AdvisoryProductStatus::Complete);
    assert_eq!(
        summary.return_fact.shape,
        ValueShape::Instance(DeclarationId::new(ModuleId::core(), "Int".into()))
    );
    assert_eq!(update.snapshot.id().workspace(), session.workspace());
}

#[test]
fn top_level_expression_sites_are_indexed_for_advisory_queries() {
    let module = module_id();
    let source = "class Product { @constructor new() { } }\nconst result = Product.new()\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));

    let offset = source.rfind("new").expect("constructor selector");
    let site = update
        .snapshot
        .source_index()
        .expression_site_at(&module, offset)
        .expect("top-level AST expression site");
    assert!(matches!(&site.id.owner, SourceOwner::Module(owner) if owner == &module));
    assert!(site.range.contains(offset));
    assert!(update.snapshot.advisory().expression(&site.id).is_some());
}

#[test]
fn formal_projection_preserves_causal_invalidity_and_binding_contracts() {
    let module = module_id();
    let source = "class Probe { @class run() { let x: Int = 42\nlet y: Missing = 42\n } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Probe".into()),
        Selector::method("run", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("run analysis");
    let x = analysis.bindings.values().find(|binding| binding.name == "x").expect("x binding");
    let x_site = update
        .snapshot
        .formal_projection()
        .get(&FormalFactRef::Binding {
            callable: callable.clone(),
            binding: x.binding,
        })
        .expect("x formal projection");
    let x_contract = x_site.contract.as_ref().expect("x contract projection");
    assert_eq!(x_contract.consistency, BindingConsistency::Validated);
    assert_eq!(x_site.causal_invalidity, phalcom_semantic::checker::CausalInvalidity::Clean);

    let y = analysis.bindings.values().find(|binding| binding.name == "y").expect("y binding");
    let y_site = update
        .snapshot
        .formal_projection()
        .get(&FormalFactRef::Binding { callable, binding: y.binding })
        .expect("y formal projection");
    assert!(matches!(y_site.causal_invalidity, phalcom_semantic::checker::CausalInvalidity::One(_)));
    assert!(y_site.contract.is_some(), "formal projection must retain annotation relation metadata");
}

#[test]
fn binding_attachment_incident_does_not_hide_expression_attachment() {
    let module = module_id();
    let source = "class CellNum { @constructor new(_ raw: Int) { raw } }\nlet value = CellNum.new(1)\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));

    assert!(
        !update.snapshot.source_index().incidents().is_empty(),
        "fixture should retain binding attachment incident"
    );
    let attachments = &update.snapshot.source_index().module(&module).expect("source module").attachments;
    assert!(!attachments.is_empty(), "formal callable attachment should still publish");
    assert!(attachments.values().any(|attachment| !attachment.expression_sites.is_empty()));
}

#[test]
fn advisory_query_distinguishes_missing_coverage_from_published_unknown() {
    let site = SourceSiteId {
        owner: SourceOwner::Module(ModuleId::core()),
        local: SourceSiteLocalId(1),
    };
    let unknown = AdvisoryFact::unknown().derive(AdvisoryConfidence::Heuristic, AdvisoryOrigin::Syntax(site.clone()));
    let shard = AdvisoryModuleProduct::new(
        ModuleId::core(),
        BTreeMap::from([(site.clone(), unknown.clone())]),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        AdvisoryProductStatus::Complete,
    );
    let workspace = AdvisoryWorkspace::from_parts(
        BTreeMap::from([(ModuleId::core(), Arc::new(shard))]),
        BTreeMap::new(),
        AdvisoryProductStatus::Complete,
    );

    assert_eq!(workspace.expression(&site), Some(&unknown));
    let missing = SourceSiteId {
        owner: SourceOwner::Module(ModuleId::core()),
        local: SourceSiteLocalId(2),
    };
    assert!(workspace.expression(&missing).is_none());
    assert_eq!(workspace.expression_or_unknown(&missing), AdvisoryFact::unknown());
}

#[test]
fn unchanged_advisory_shards_and_callable_summaries_are_reused() {
    let module = module_id();
    let source = "class Box { value() -> Int { return 1 } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(input(module.clone(), source, 1));
    let second = session.update(input(module.clone(), source, 2));

    let shard1 = first.snapshot.advisory().module(&module).unwrap();
    let shard2 = second.snapshot.advisory().module(&module).unwrap();
    assert!(Arc::ptr_eq(shard1, shard2));
    let source1 = first.snapshot.source_index().module_arc(&module).unwrap();
    let source2 = second.snapshot.source_index().module_arc(&module).unwrap();
    assert!(Arc::ptr_eq(&source1, &source2));

    let callable = CallableId::new(
        DeclarationId::new(module, "Box".into()),
        Selector::method("value", []).unwrap(),
        DispatchSide::Instance,
    );
    let summary1 = first.snapshot.advisory().callables.get(&callable).unwrap();
    let summary2 = second.snapshot.advisory().callables.get(&callable).unwrap();
    assert!(Arc::ptr_eq(summary1, summary2));
}

#[test]
fn advisory_disagreement_is_unknown_for_all_non_ready_formal_states() {
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module_id(), "let value = 1\n", 1));
    let int = update
        .snapshot
        .declarations
        .form(&DeclarationId::new(ModuleId::core(), "Int".into()))
        .expect("Int declaration");
    let expression_id = phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(1));
    let advisory = AdvisoryFact::new(
        ValueShape::Instance(DeclarationId::new(ModuleId::core(), "String".into())),
        AdvisoryConfidence::Exact,
    );
    let statuses = [
        AnalysisStatus::Invalid(phalcom_semantic::identity::DiagnosticCauseId(1)),
        AnalysisStatus::Suppressed(phalcom_semantic::checker::SuppressionCause::One(phalcom_semantic::identity::DiagnosticCauseId(
            2,
        ))),
        AnalysisStatus::Blocked(phalcom_semantic::BlockReason::RecursiveFixpoint),
        AnalysisStatus::DynamicBoundary(phalcom_semantic::DynamicReason::RuntimeReflection),
        AnalysisStatus::Cancelled,
    ];

    for status in statuses {
        let formal = ExpressionAnalysis::ready(
            expression_id,
            phalcom_common::range::SourceRange { start: 0, end: 1 },
            phalcom_semantic::types::evidence::TypeKnowledge::established(int, phalcom_semantic::types::evidence::EvidenceOrigin::Syntax),
        )
        .with_status(status);
        assert_eq!(
            compare_expression(&update.snapshot.store, &formal, &advisory),
            AdvisoryAgreement::Unknown,
            "non-ready formal state must not become comparable advisory evidence"
        );
    }

    for knowledge in [
        phalcom_semantic::types::evidence::TypeKnowledge::Unknown(phalcom_semantic::types::evidence::UnknownReason::NoTypeEvidence),
        phalcom_semantic::types::evidence::TypeKnowledge::Dynamic(phalcom_semantic::types::evidence::DynamicReason::ExplicitEscape),
    ] {
        let formal = ExpressionAnalysis::ready(expression_id, phalcom_common::range::SourceRange { start: 0, end: 1 }, knowledge);
        assert_eq!(
            compare_expression(&update.snapshot.store, &formal, &advisory),
            AdvisoryAgreement::Unknown,
            "non-concrete formal knowledge must not become comparable advisory evidence"
        );
    }
}

#[test]
fn advisory_disagreement_keeps_ready_formal_product_unchanged() {
    let module = module_id();
    let source = "class Probe { @class run() { let value: Int = 1 } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));
    let callable = update
        .snapshot
        .callable_analyses
        .keys()
        .find(|callable| callable.owner.name.as_ref() == "Probe")
        .cloned()
        .expect("Probe callable");
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("formal analysis");
    let expression = analysis.expressions.values().next().expect("expression product");
    let before = expression.clone();
    let advisory = AdvisoryFact::new(
        ValueShape::Instance(DeclarationId::new(ModuleId::core(), "String".into())),
        AdvisoryConfidence::Exact,
    );
    let relation = compare_expression(&update.snapshot.store, expression, &advisory);
    assert!(matches!(relation, AdvisoryAgreement::Incomparable | AdvisoryAgreement::Unknown));
    assert_eq!(expression, &before, "advisory disagreement must not mutate formal expression product");
}

#[test]
fn advisory_parameter_transfer_reaches_tail_return_product() {
    let module = module_id();
    let source = "class Product { @constructor new() { } }\nclass Service { @constructor new() { } consume(_ value) { value } }\nconst result = Service.new().consume(Product.new())\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Service".into()),
        Selector::method("consume", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let product = DeclarationId::new(module, "Product".into());
    let summary = update.snapshot.advisory().callable(&callable).expect("consume advisory summary");
    assert!(!summary.parameters.is_empty(), "summary={summary:#?}");
    assert_eq!(summary.parameters[0].1.shape, ValueShape::Instance(product.clone()));
    assert_eq!(summary.return_fact.shape, ValueShape::Instance(product));
}

#[test]
fn advisory_parameter_transfer_converges_through_forwarding_callable() {
    let module = module_id();
    let source = "class Product { @constructor new() { } }\nclass Relay { @constructor new() { } sink(_ value) { value } forward(_ value) { sink(value) } }\nconst result = Relay.new().forward(Product.new())\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let product = DeclarationId::new(module.clone(), "Product".into());
    let relay = DeclarationId::new(module, "Relay".into());
    for name in ["sink", "forward"] {
        let callable = CallableId::new(
            relay.clone(),
            Selector::method(name, [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
            DispatchSide::Instance,
        );
        let summary = update.snapshot.advisory().callable(&callable).expect("relay advisory summary");
        assert_eq!(summary.parameters[0].1.shape, ValueShape::Instance(product.clone()));
        assert_eq!(summary.return_fact.shape, ValueShape::Instance(product.clone()));
    }
}
