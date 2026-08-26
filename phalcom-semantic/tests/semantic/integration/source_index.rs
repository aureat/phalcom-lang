use phalcom_ast::parser::parse;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_semantic::checker::analysis::{AnalysisStatus, BindingState, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, FlowStateSummary};
use phalcom_semantic::checker::flow::graph::FlowGraph;
use phalcom_semantic::db::ProductFingerprint;
use phalcom_semantic::explain::ExplanationArena;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, FieldId, SourceOwner, SourceSiteId, SourceSiteLocalId, SourceSiteRef};
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};
use phalcom_semantic::{
    FormalFactRef, FormalSemanticProjection, ModuleId, OccurrenceIndex, OccurrenceKind, OccurrenceRole, SemanticOccurrence, SemanticRevision, SemanticTargetId,
    SnapshotId, SourceBindingKind, SourceIndexContext, SourceNameResolution, SourceSemanticIndex, TypeStoreId, WorkspaceId, build_source_scope_index,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
}

#[test]
fn canonical_targets_survive_revision_changes() {
    let owner = declaration("Widget");
    let callable = CallableId::new(owner.clone(), Selector::getter("value").unwrap(), DispatchSide::Instance);
    let field = FieldId::new(owner.clone(), "value", DispatchSide::Instance);

    let first = (
        SemanticTargetId::Declaration(owner.clone()),
        SemanticTargetId::Callable(callable.clone()),
        SemanticTargetId::Field(field.clone()),
    );
    let second = (
        SemanticTargetId::Declaration(owner),
        SemanticTargetId::Callable(callable),
        SemanticTargetId::Field(field),
    );

    assert_eq!(first, second);
}

#[test]
fn source_site_ref_rejects_stale_snapshot() {
    let store = TypeStoreId::from_raw(7);
    let first_snapshot = SnapshotId::new(WorkspaceId::from_raw(1), SemanticRevision::from_raw(1), store);
    let second_snapshot = SnapshotId::new(WorkspaceId::from_raw(1), SemanticRevision::from_raw(2), store);
    let site = SourceSiteId {
        owner: SourceOwner::Module(ModuleId::core()),
        local: SourceSiteLocalId(3),
    };
    let site_ref = SourceSiteRef::new(first_snapshot, site.clone());

    assert_eq!(site_ref.resolve_for(first_snapshot), Some(&site));
    assert_eq!(site_ref.resolve_for(second_snapshot), None);
}

#[test]
fn source_site_identity_is_owner_qualified() {
    let callable = CallableId::new(declaration("Widget"), Selector::getter("value").unwrap(), DispatchSide::Instance);
    let module_site = SourceSiteId {
        owner: SourceOwner::Module(ModuleId::core()),
        local: SourceSiteLocalId(0),
    };
    let callable_site = SourceSiteId {
        owner: SourceOwner::Callable(callable),
        local: SourceSiteLocalId(0),
    };

    assert_ne!(module_site, callable_site);
    assert_eq!(SourceRange { start: 4, end: 9 }.len(), 5);
}

#[test]
fn lexical_scope_preserves_source_order_and_nested_shadowing() {
    let source = "let value = 1\nclass Sample {\n  method(value) { value }\n}\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let index = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let use_offset = source.rfind("value }").expect("method use") + 1;
    let method_scope = index.scope_at(use_offset);
    let visible = index.visible_bindings_at(use_offset);

    assert_eq!(visible[0].name.as_ref(), "value");
    assert_eq!(visible[0].scope, method_scope);
    assert!(
        index
            .bindings
            .values()
            .any(|binding| binding.name.as_ref() == "value" && binding.scope == index.root)
    );
    assert!(matches!(
        index.resolve_name(method_scope, "value", use_offset),
        SourceNameResolution::Binding(site) if site == visible[0].declaration_site
    ));
    assert!(matches!(
        index.resolve_name(index.root, "Sample", 0),
        SourceNameResolution::Target(SemanticTargetId::Declaration(_))
    ));
}

#[test]
fn same_scope_redeclaration_keeps_first_lexical_target() {
    let source = "let value = 1\nlet value = 2\nvalue\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let index = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let values = index.bindings.values().filter(|binding| binding.name.as_ref() == "value").collect::<Vec<_>>();
    assert_eq!(values.len(), 2);
    let first = values.iter().find(|binding| binding.redeclaration_of.is_none()).expect("first binding");
    let duplicate = values.iter().find(|binding| binding.redeclaration_of.is_some()).expect("duplicate binding");
    assert_eq!(duplicate.redeclaration_of.as_ref(), Some(&first.declaration_site));
    let use_offset = source.rfind("value").expect("use") + 1;
    assert!(matches!(
        index.resolve_name(index.root, "value", use_offset),
        SourceNameResolution::Binding(site) if site == first.declaration_site
    ));
    assert_eq!(first.kind, SourceBindingKind::TopLevelLet);
}

#[test]
fn compiler_scope_index_covers_closure_and_destructure_bindings() {
    let source = "let mapper = |value| value\nlet (x, y) = (1, \"hello\")\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let index = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());

    let parameter_start = source.find("value").expect("closure parameter");
    let parameter = index
        .binding_for_declaration((parameter_start..parameter_start + "value".len()).into())
        .expect("closure binding");
    assert_eq!(parameter.kind, SourceBindingKind::ClosureParameter);
    assert!(parameter.mutable);

    let destructured = index
        .bindings
        .values()
        .filter(|binding| matches!(binding.kind, SourceBindingKind::Destructure))
        .collect::<Vec<_>>();
    assert_eq!(destructured.len(), 2);
    assert!(destructured.iter().all(|binding| binding.mutable));
}

#[test]
fn imports_attach_only_to_canonical_linked_targets() {
    let source = "from .shapes import Circle\nCircle\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let shapes = ModuleId::core();
    let circle = DeclarationId::new(shapes.clone(), "Circle".into());
    let context = SourceIndexContext::default()
        .with_module(".shapes", shapes.clone())
        .with_target(shapes, "Circle", SemanticTargetId::Declaration(circle));
    let index = build_source_scope_index(ModuleId::core(), &parsed.program, &context);
    let binding = index
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "Circle")
        .expect("selective import binding");

    assert_eq!(binding.kind, SourceBindingKind::Import);
    assert!(matches!(index.target_for(&binding.declaration_site), Some(SemanticTargetId::Declaration(_))));
}

#[test]
fn for_binding_is_owned_by_loop_scope_and_is_mutable() {
    let source = "let values = [1, 2]\nfor value in values { value }\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let index = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let value = index.bindings.values().find(|binding| binding.name.as_ref() == "value").expect("for binding");

    assert_eq!(value.kind, SourceBindingKind::ForBinding);
    assert!(value.mutable);
    assert_ne!(value.scope, index.root);
}

#[test]
fn occurrence_index_selects_nested_site_and_keeps_unresolved_hint_advisory() {
    let owner = SourceOwner::Module(ModuleId::core());
    let outer = SourceSiteId {
        owner: owner.clone(),
        local: SourceSiteLocalId(10),
    };
    let inner = SourceSiteId {
        owner: owner.clone(),
        local: SourceSiteLocalId(11),
    };
    let target = SemanticTargetId::Module(ModuleId::core());
    let occurrences = vec![
        SemanticOccurrence {
            site: outer.clone(),
            range: (0..20).into(),
            kind: OccurrenceKind::Declaration,
            role: OccurrenceRole::Declaration,
            owner: owner.clone(),
            hint: None,
        },
        SemanticOccurrence {
            site: inner.clone(),
            range: (4..8).into(),
            kind: OccurrenceKind::Module,
            role: OccurrenceRole::Reference,
            owner,
            hint: Some(phalcom_semantic::OccurrenceHint::Name("missing".into())),
        },
    ];
    let index = OccurrenceIndex::new(occurrences, BTreeMap::from([(outer, target.clone())]));

    let selected = index.occurrence_at(5).expect("nested occurrence");
    assert_eq!(selected.occurrence.site, inner);
    assert!(
        selected.target.is_none(),
        "unresolved occurrence must not inherit advisory hint as exact target"
    );
    assert_eq!(
        index.occurrences_for_target(&target),
        Some(
            [SourceSiteId {
                owner: SourceOwner::Module(ModuleId::core()),
                local: SourceSiteLocalId(10)
            }]
            .as_slice()
        )
    );
}

#[test]
fn occurrence_interval_queries_remain_bounded_for_large_indexes() {
    let owner = SourceOwner::Module(ModuleId::core());
    let occurrences = (0..4096)
        .map(|offset| SemanticOccurrence {
            site: SourceSiteId {
                owner: owner.clone(),
                local: SourceSiteLocalId(offset as u32),
            },
            range: (offset * 2..offset * 2 + 1).into(),
            kind: OccurrenceKind::Binding,
            role: OccurrenceRole::Read,
            owner: owner.clone(),
            hint: None,
        })
        .collect::<Vec<_>>();
    let index = OccurrenceIndex::new(occurrences, BTreeMap::new());

    assert_eq!(index.len(), 4096);
    assert_eq!(index.occurrence_at(8190).map(|view| view.occurrence.range), Some((8190..8191).into()));
    assert!(index.occurrence_at(8191).is_none());
}

#[test]
fn formal_products_attach_by_callable_and_checker_ids() {
    let source = "class Sample { method(value) { value } }\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let module = ModuleId::core();
    let scopes = build_source_scope_index(module.clone(), &parsed.program, &SourceIndexContext::default());
    let mut index = SourceSemanticIndex::from_scope_indices(BTreeMap::from([(module.clone(), scopes)]));
    let declaration = DeclarationId::new(module.clone(), "Sample".into());
    let callable = CallableId::new(
        declaration,
        Selector::method("method", vec![phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap(),
        DispatchSide::Instance,
    );
    let parameter_start = source.find("value").expect("parameter");
    let expression_start = source.rfind("value").expect("expression");
    let expression_id = phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(0));
    let binding_id = phalcom_semantic::identity::BindingId(0);
    let analysis = CallableAnalysis {
        callable: callable.clone(),
        body_range: (source.find('{').unwrap()..source.rfind('}').unwrap() + 1).into(),
        expressions: BTreeMap::from([(
            expression_id,
            ExpressionAnalysis {
                id: expression_id,
                range: (expression_start..expression_start + 5).into(),
                knowledge: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                callable: None,
                denotation: None,
                status: AnalysisStatus::Ready,
                causal_invalidity: Default::default(),
                explanation: None,
                call: None,
            },
        )]),
        bindings: BTreeMap::from([(
            binding_id,
            BindingState::new(
                binding_id,
                "value",
                (parameter_start..parameter_start + 5).into(),
                None,
                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                true,
            ),
        )]),
        flow_graph: Arc::new(FlowGraph::default()),
        entry_flow: FlowStateSummary::default(),
        exits: Default::default(),
        diagnostics: Arc::from([]),
        internal_incidents: Arc::from([]),
        explanations: Arc::new(ExplanationArena::default()),
        dependencies: Arc::from([]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: ProductFingerprint::new(1),
        status: CallableAnalysisStatus::Complete,
    };

    index.attach_formal_analysis(&module, &analysis).expect("unique formal attachment");
    let attachment = index
        .module(&module)
        .expect("module shard")
        .attachments
        .get(&callable)
        .expect("callable attachment");
    let binding_site = attachment.formal_bindings.get(&binding_id).expect("binding site");
    let expression_site = attachment.formal_expressions.get(&expression_id).expect("expression site");
    assert_eq!(
        index.source_site(binding_site).expect("binding source site").range,
        (parameter_start..parameter_start + 5).into()
    );
    assert_eq!(
        index.source_site(expression_site).expect("expression source site").range,
        (expression_start..expression_start + 5).into()
    );
    assert_eq!(
        index.occurrences_for_target(&SemanticTargetId::Binding(binding_site.clone())),
        Some([binding_site.clone()].as_slice())
    );

    let projection = FormalSemanticProjection::from_callable_analyses(&HashMap::from([(callable.clone(), Arc::new(analysis.clone()))]));
    assert_eq!(projection.len(), 3);
    assert!(matches!(
        projection.get(&FormalFactRef::Expression { callable: callable.clone(), expression: expression_id }),
        Some(site) if site.range == (expression_start..expression_start + 5).into()
    ));
    assert!(matches!(
        projection.fact_at(&module, expression_start + 1),
        Some(site) if matches!(site.fact, FormalFactRef::Expression { .. })
    ));
}
