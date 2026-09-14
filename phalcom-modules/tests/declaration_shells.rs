use phalcom_common::range::SourceRange;
use phalcom_modules::{
    DeclarationBlueprint, DeclarationId, DeclarationKind, DeclarationRealizationError, DeclarationShellTable, ModuleComponent, ModuleId, ModulePath,
    ProjectIdentity, ResolvedProjectId, SemanticEdge, SemanticEdgeKind, SemanticGraph,
};

fn module(project: u32, name: &str) -> ModuleId {
    ModuleId {
        project: ProjectIdentity::Resolved(ResolvedProjectId::from_raw(project)),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    }
}

fn decl(module: ModuleId, name: &str) -> DeclarationId {
    DeclarationId { module, name: name.into() }
}

#[test]
fn mutual_semantic_references_realize_after_all_shells_exist() {
    let a = decl(module(1, "a"), "A");
    let b = decl(module(1, "b"), "B");
    let mut table = DeclarationShellTable::default();
    table.predeclare([
        DeclarationBlueprint {
            id: a.clone(),
            kind: DeclarationKind::Class,
        },
        DeclarationBlueprint {
            id: b.clone(),
            kind: DeclarationKind::Class,
        },
    ]);
    let mut graph = SemanticGraph::default();
    graph.add(SemanticEdge {
        from: a.semantic_node(),
        to: b.semantic_node(),
        kind: SemanticEdgeKind::TypeReference,
        range: SourceRange::default(),
    });
    graph.add(SemanticEdge {
        from: b.semantic_node(),
        to: a.semantic_node(),
        kind: SemanticEdgeKind::TypeReference,
        range: SourceRange::default(),
    });
    table.realize_semantic_graph(&graph).unwrap();
    assert_eq!(table.realized_ids().len(), 2);
}

#[test]
fn trait_declaration_kind_is_a_distinct_shell_category() {
    let trait_id = decl(module(3, "contracts"), "Display");
    let mut table = DeclarationShellTable::default();
    table.predeclare([DeclarationBlueprint {
        id: trait_id.clone(),
        kind: DeclarationKind::Trait,
    }]);

    let shell = table.get(&trait_id.semantic_node()).expect("trait shell");
    assert_eq!(shell.blueprint.kind, DeclarationKind::Trait);
}

#[test]
fn inheritance_cycle_is_rejected_even_inside_a_semantic_scc() {
    let a = decl(module(2, "a"), "A");
    let b = decl(module(2, "b"), "B");
    let mut table = DeclarationShellTable::default();
    table.predeclare([
        DeclarationBlueprint {
            id: a.clone(),
            kind: DeclarationKind::Class,
        },
        DeclarationBlueprint {
            id: b.clone(),
            kind: DeclarationKind::Class,
        },
    ]);
    let mut graph = SemanticGraph::default();
    graph.add(SemanticEdge {
        from: a.semantic_node(),
        to: b.semantic_node(),
        kind: SemanticEdgeKind::Superclass,
        range: SourceRange::default(),
    });
    graph.add(SemanticEdge {
        from: b.semantic_node(),
        to: a.semantic_node(),
        kind: SemanticEdgeKind::Superclass,
        range: SourceRange::default(),
    });
    assert!(matches!(
        table.realize_semantic_graph(&graph),
        Err(DeclarationRealizationError::InheritanceCycle { .. })
    ));
}
