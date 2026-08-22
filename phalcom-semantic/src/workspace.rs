//! Semantic workspace analysis engine for whole linked programs.

use crate::checker::context::CheckingContext;
use crate::checker::declaration::{check_class_bodies, register_class_surface};
use crate::checker::statement::check_statement;
use crate::declarations::{
    DeclarationTypeInfo, bootstrap_universe_declarations,
};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId};
use crate::resolver::LinkedTypeResolver;
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::types::annotation::TypeResolver;
use crate::types::id::KindId;
use crate::types::native::register_standard_surfaces;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{Program, Statement};
use phalcom_modules::declaration::{
    DeclarationBlueprint, DeclarationKind, DeclarationRealizationError, DeclarationShellTable,
};
use phalcom_modules::graph::{SemanticEdge, SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{LinkedModule, LinkedProgram};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// Input to whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub generation: u64,
}

/// The result of whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticAnalysis {
    pub snapshot: Arc<SemanticSnapshot>,
}

/// Analyzes an entire linked workspace and returns an immutable semantic snapshot for this generation.
pub fn analyze_workspace(input: SemanticWorkspaceInput) -> SemanticAnalysis {
    // -------------------------------------------------------------------------
    // Phase A: Universe Bootstrap
    // -------------------------------------------------------------------------
    let mut store = TypeStore::new();
    let mut declarations = bootstrap_universe_declarations(&mut store, &|key| {
        DeclarationId::new(ModuleId::core(), key.name().into())
    });

    let mut hierarchy = MapTypeHierarchy::new();
    for (sub_name, super_name) in [
        ("Behavior", "Object"),
        ("Class", "Behavior"),
        ("Metaclass", "Class"),
        ("Int", "Number"),
        ("Float", "Number"),
        ("Some", "Option"),
        ("None", "Option"),
        ("True", "Bool"),
        ("False", "Bool"),
        ("BoundMethod", "Method"),
        ("Closure", "Function"),
        ("MethodFamily", "Family"),
        ("BoundMethodFamily", "Family"),
        ("MessageNotUnderstood", "Error"),
        ("CannotYieldAcrossNativeFrame", "Error"),
        ("UseAfterCloseError", "Error"),
    ] {
        hierarchy.insert(
            DeclarationId::new(ModuleId::core(), sub_name.into()),
            DeclarationId::new(ModuleId::core(), super_name.into()),
        );
    }

    // -------------------------------------------------------------------------
    // Phase B: Predeclare Every Source Declaration
    // -------------------------------------------------------------------------
    let mut shell_table = DeclarationShellTable::default();
    let mut initial_blueprints: Vec<DeclarationBlueprint> = declarations
        .iter()
        .map(|(decl_id, _)| DeclarationBlueprint {
            id: decl_id.clone(),
            kind: DeclarationKind::Class,
        })
        .collect();

    for (module_id, parsed_unit) in &input.sources {
        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                initial_blueprints.push(DeclarationBlueprint {
                    id: decl_id.clone(),
                    kind: DeclarationKind::Class,
                });
                if declarations.get(&decl_id).is_none() {
                    let form = store.nominal_type(decl_id.clone());
                    let class_obj_type = store.class_object_type(decl_id.clone());
                    declarations.insert(DeclarationTypeInfo {
                        declaration: decl_id,
                        form,
                        class_object_type: class_obj_type,
                        kind: KindId::TYPE,
                        generic_signature: None,
                    });
                }
            }
        }
    }
    shell_table.predeclare(initial_blueprints);

    // -------------------------------------------------------------------------
    // Phase C: Construct LinkedTypeResolver
    // -------------------------------------------------------------------------
    let known_declarations: HashSet<DeclarationId> = declarations
        .iter()
        .map(|(decl_id, _)| decl_id.clone())
        .collect();

    let resolver = LinkedTypeResolver::new(
        input.linked.clone(),
        known_declarations,
        ModuleId::core(),
    );

    // -------------------------------------------------------------------------
    // Phase D: Enrich Semantic Graph
    // -------------------------------------------------------------------------
    let mut semantic_graph = input.linked.graphs.semantics.clone();

    for (module_id, parsed_unit) in &input.sources {
        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                let from_node = SemanticNodeId::Declaration {
                    module: module_id.clone(),
                    name: class_def.name.clone().into(),
                };

                if let Some(ref super_ref) = class_def.superclass {
                    let members: Vec<String> =
                        super_ref.members.iter().map(|m| m.name.clone()).collect();
                    if let Some(target_decl) =
                        resolver.resolve_type_name(module_id, &super_ref.root, &members)
                    {
                        let to_node = SemanticNodeId::Declaration {
                            module: target_decl.module,
                            name: target_decl.name,
                        };
                        semantic_graph.add(SemanticEdge {
                            from: from_node.clone(),
                            to: to_node,
                            kind: SemanticEdgeKind::Superclass,
                            range: super_ref.range,
                        });
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase E: Realize Declaration Shells
    // -------------------------------------------------------------------------
    let mut diags_by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>> = BTreeMap::new();

    if let Err(err) = shell_table.realize_semantic_graph(&semantic_graph) {
        match err {
            DeclarationRealizationError::InheritanceCycle { cycle } => {
                if let Some(first_node) = cycle.first() {
                    let mod_id = match first_node {
                        SemanticNodeId::Module(m) => m.clone(),
                        SemanticNodeId::Declaration { module, .. } => module.clone(),
                    };
                    diags_by_module
                        .entry(mod_id)
                        .or_default()
                        .push(SemanticDiagnostic::error(
                            DiagnosticCode::AnnotationUnresolved,
                            format!("A class cannot extend itself: inheritance cycle detected: {cycle:?}"),
                            phalcom_common::range::SourceRange::default(),
                        ));
                }
            }
            DeclarationRealizationError::MissingShell(node) => {
                let mod_id = match &node {
                    SemanticNodeId::Module(m) => m.clone(),
                    SemanticNodeId::Declaration { module, .. } => module.clone(),
                };
                diags_by_module
                    .entry(mod_id)
                    .or_default()
                    .push(SemanticDiagnostic::error(
                        DiagnosticCode::AnnotationUnresolved,
                        format!("missing declaration shell for {node:?}"),
                        phalcom_common::range::SourceRange::default(),
                    ));
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase F: Build Hierarchy
    // -------------------------------------------------------------------------
    for (module_id, parsed_unit) in &input.sources {
        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                let class_decl =
                    DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                if let Some(ref super_ref) = class_def.superclass {
                    let members: Vec<String> =
                        super_ref.members.iter().map(|m| m.name.clone()).collect();
                    if let Some(super_decl) =
                        resolver.resolve_type_name(module_id, &super_ref.root, &members)
                    {
                        hierarchy.insert(class_decl, super_decl);
                    } else {
                        diags_by_module
                            .entry(module_id.clone())
                            .or_default()
                            .push(SemanticDiagnostic::error(
                                DiagnosticCode::AnnotationUnresolved,
                                format!("unresolved superclass `{}`", super_ref.root),
                                super_ref.range,
                            ));
                    }
                } else {
                    let obj_decl = DeclarationId::new(ModuleId::core(), "Object".into());
                    if class_decl != obj_decl {
                        hierarchy.insert(class_decl, obj_decl);
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase G: Collect Declaration Surfaces Before Bodies
    // -------------------------------------------------------------------------
    let mut dispatch = SurfaceDispatchResolver::new();
    register_standard_surfaces(
        &mut store,
        &declarations,
        &resolver,
        &ModuleId::core(),
        &mut dispatch,
    );

    for (module_id, parsed_unit) in &input.sources {
        let mut dummy_ctx = CheckingContext::new(
            &mut store,
            &hierarchy,
            &resolver,
            &declarations,
            module_id.clone(),
        );

        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                register_class_surface(&mut dummy_ctx, class_def);
            }
        }

        for (decl_id, surface) in dummy_ctx.dispatch.surfaces() {
            dispatch.register_surface(decl_id.clone(), surface.clone());
            if let Some(ty) = declarations.form(decl_id) {
                dispatch.register_type(ty, decl_id.clone());
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase H: Check Bodies
    // -------------------------------------------------------------------------
    for (module_id, parsed_unit) in &input.sources {
        let mut ctx = CheckingContext::new(
            &mut store,
            &hierarchy,
            &resolver,
            &declarations,
            module_id.clone(),
        );
        ctx.dispatch = dispatch.clone();

        for stmt in &parsed_unit.program.statements {
            match stmt {
                Statement::Class(class_def) => {
                    check_class_bodies(&mut ctx, class_def);
                }
                _ => {
                    check_statement(&mut ctx, stmt);
                }
            }
        }

        if !ctx.diagnostics.is_empty() {
            diags_by_module
                .entry(module_id.clone())
                .or_default()
                .extend(ctx.diagnostics);
        }
    }

    // -------------------------------------------------------------------------
    // Phase I & J: Freeze and Publish Immutable Snapshot
    // -------------------------------------------------------------------------
    let mut diagnostics_map = BTreeMap::new();
    for (module_id, diags) in diags_by_module {
        diagnostics_map.insert(module_id, Arc::from(diags.into_boxed_slice()));
    }

    let snapshot = Arc::new(SemanticSnapshot {
        generation: input.generation,
        store: Arc::new(store),
        sources: Arc::new(input.sources),
        surfaces: Arc::new(dispatch.surfaces().clone()),
        dispatch: Arc::new(dispatch),
        declarations: Arc::new(declarations),
        hierarchy: Arc::new(hierarchy),
        diagnostics: Arc::new(diagnostics_map),
        semantic_graph: Arc::new(semantic_graph),
    });

    SemanticAnalysis { snapshot }
}

/// Convenience helper to analyze a single module as a standalone workspace.
pub fn analyze_single_module(
    module: ModuleId,
    source: Arc<str>,
    program: Arc<Program>,
) -> SemanticAnalysis {
    let _ = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program);
    let linked_mod = LinkedModule {
        interface: phalcom_modules::interface::LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: phalcom_modules::linker::ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_mod);

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });

    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            source,
            program,
        )),
    );

    analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 0,
    })
}
