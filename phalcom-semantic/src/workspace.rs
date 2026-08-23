//! Semantic workspace analysis engine for whole linked programs.

use crate::checker::context::CheckingContext;
use crate::checker::declaration::{check_class_bodies, register_class_surface};
use crate::checker::statement::check_statement;
use crate::declarations::{DeclarationTypeInfo, GenericSupertypeTemplate, bootstrap_universe_declarations};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId};
use crate::resolver::LinkedTypeResolver;
use crate::signature::CallableSignatureTable;
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::types::annotation::{TypeResolver, resolve_generic_signature, resolve_kind_syntax, resolve_type_annotation};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::KindId;
use crate::types::native::register_native_surfaces;
use crate::types::parameter::TypeParameterOwner;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{ClassMember, Program, Statement};
use phalcom_common::selector::Selector;
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind, DeclarationRealizationError, DeclarationShellTable};
use phalcom_modules::graph::{SemanticEdge, SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{LinkedModule, LinkedProgram};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use std::collections::{BTreeMap, HashMap, HashSet};
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
    let mut declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(ModuleId::core(), key.name().into()));

    let mut hierarchy = MapTypeHierarchy::new();
    for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
        if let Some(superclass) = relation.superclass {
            hierarchy.insert(
                DeclarationId::new(ModuleId::core(), relation.class.name().into()),
                DeclarationId::new(ModuleId::core(), superclass.name().into()),
            );
        }
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
                    let kind = if !class_def.generic_parameters.is_empty() {
                        let param_kinds: Vec<KindId> = class_def
                            .generic_parameters
                            .iter()
                            .map(|p| p.kind.as_ref().map_or(KindId::TYPE, |k| resolve_kind_syntax(&mut store, k)))
                            .collect();
                        store.arrow_kind(param_kinds.into_boxed_slice(), KindId::TYPE)
                    } else {
                        KindId::TYPE
                    };

                    let form = if kind == KindId::TYPE {
                        store.nominal_type(decl_id.clone())
                    } else {
                        store.nominal_form(decl_id.clone(), kind)
                    };
                    let class_obj_type = store.class_object_type(decl_id.clone());
                    declarations.insert(DeclarationTypeInfo {
                        declaration: decl_id,
                        form,
                        class_object_type: class_obj_type,
                        kind,
                        generic_signature: None,
                        supertype_template: None,
                    });
                }
            }
        }
    }
    shell_table.predeclare(initial_blueprints);

    // -------------------------------------------------------------------------
    // Phase C: Construct LinkedTypeResolver
    // -------------------------------------------------------------------------
    let known_declarations: HashSet<DeclarationId> = declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();

    let resolver = LinkedTypeResolver::new(input.linked.clone(), known_declarations, ModuleId::core());

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

                if let Some(super_ref) = class_def.superclass_ref() {
                    let members: Vec<String> = super_ref.members.iter().map(|m| m.name.clone()).collect();
                    if let Some(target_decl) = resolver.resolve_type_name(module_id, &super_ref.root, &members) {
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
                    diags_by_module.entry(mod_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                        mod_id,
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
                diags_by_module.entry(mod_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                    mod_id,
                    DiagnosticCode::AnnotationUnresolved,
                    format!("missing declaration shell for {node:?}"),
                    phalcom_common::range::SourceRange::default(),
                ));
            }
        }
    }

    // Publish generic signatures and supertype templates for source classes
    for (module_id, parsed_unit) in &input.sources {
        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let generic_signature = if !class_def.generic_parameters.is_empty() {
                    Some(resolve_generic_signature(
                        &mut store,
                        &declarations,
                        &resolver,
                        module_id,
                        TypeParameterOwner::Declaration(decl_id.clone()),
                        &class_def.generic_parameters,
                        class_def.where_clause.as_ref(),
                        diags_by_module.entry(module_id.clone()).or_default(),
                    ))
                } else {
                    None
                };

                let supertype_template = if let Some(super_ann) = &class_def.superclass {
                    let type_params_map = if let Some(ref sig) = generic_signature {
                        let mut map = std::collections::HashMap::new();
                        for &param_id in sig.parameters.iter() {
                            let name = store.type_parameter(param_id).name.to_string();
                            let param_form = store.parameter_form(param_id);
                            map.insert(name, param_form);
                        }
                        map
                    } else {
                        std::collections::HashMap::new()
                    };
                    let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                        parent: &resolver,
                        type_parameters: type_params_map,
                    };
                    let mut diags = Vec::new();
                    let super_k = resolve_type_annotation(&mut store, &declarations, &scoped_resolver, module_id, super_ann, &mut diags);
                    diags_by_module.entry(module_id.clone()).or_default().extend(diags);
                    if let TypeKnowledge::Known(ev) = super_k {
                        Some(GenericSupertypeTemplate {
                            declaration: decl_id.clone(),
                            supertype: ev.ty,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(info) = declarations.get(&decl_id).cloned() {
                    declarations.insert(DeclarationTypeInfo {
                        declaration: info.declaration,
                        form: info.form,
                        class_object_type: info.class_object_type,
                        kind: info.kind,
                        generic_signature,
                        supertype_template,
                    });
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase F: Build Hierarchy
    // -------------------------------------------------------------------------
    for (module_id, parsed_unit) in &input.sources {
        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                let class_decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                if let Some(super_ref) = class_def.superclass_ref() {
                    let members: Vec<String> = super_ref.members.iter().map(|m| m.name.clone()).collect();
                    if let Some(super_decl) = resolver.resolve_type_name(module_id, &super_ref.root, &members) {
                        hierarchy.insert(class_decl, super_decl);
                    } else {
                        diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                            module_id.clone(),
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
    let native_report = register_native_surfaces(&mut store, &declarations, &resolver, &ModuleId::core(), &mut dispatch)
        .expect("canonical native surface must import during semantic bootstrap");
    let mut callable_signatures = CallableSignatureTable::new();
    for (_, signature) in native_report.callable_signatures {
        callable_signatures.insert(signature);
    }

    for (module_id, parsed_unit) in &input.sources {
        let mut dummy_ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module_id.clone());

        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                register_class_surface(&mut dummy_ctx, class_def);
            }
        }

        for (decl_id, surface) in dummy_ctx.dispatch.surfaces() {
            dispatch.register_surface(decl_id.clone(), surface.clone());
            for (side, member_surface) in [
                (crate::identity::DispatchSide::Instance, &surface.instance),
                (crate::identity::DispatchSide::Class, &surface.class),
            ] {
                for (sel, sig) in &member_surface.callable_signatures {
                    let callable_id = crate::identity::CallableId::new(decl_id.clone(), sel.clone(), side);
                    if let Some(return_type) = sig.return_type.ty() {
                        let parameters = sig
                            .parameters
                            .iter()
                            .enumerate()
                            .filter_map(|(index, parameter)| {
                                let ty = parameter.ty.ty()?;
                                let mut p = crate::signature::CallableParameterSemantic::new(index as u32, parameter.local_name.clone(), ty.into());
                                if let Some(ref l) = parameter.external_label {
                                    p = p.with_label(l.clone());
                                }
                                if parameter.rest {
                                    p = p.with_rest(phalcom_ast::ast::RestMode::Positional);
                                }
                                Some(p)
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice();
                        callable_signatures.insert(crate::signature::CallableSemanticSignature {
                            callable: callable_id.clone(),
                            owner: decl_id.clone(),
                            side,
                            selector: sel.clone(),
                            generics: None,
                            parameters,
                            return_type: return_type.into(),
                            source: None,
                            implementation: phalcom_native_meta::ImplementationKind::Source,
                            native_id: None,
                            effects: phalcom_native_meta::EffectSpec::Unknown,
                            raises: phalcom_native_meta::RaisesSpec::Unknown,
                            flow: phalcom_native_meta::ReturnFlowSpec::Value,
                            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
                        });
                    }
                }
            }
            if let Some(ty) = declarations.form(decl_id) {
                dispatch.register_type(ty, decl_id.clone());
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase H: Check Bodies
    // -------------------------------------------------------------------------
    let mut callable_analyses = HashMap::new();
    for (module_id, parsed_unit) in &input.sources {
        for stmt in &parsed_unit.program.statements {
            if let Statement::Class(class_def) = stmt {
                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                    let mut map = std::collections::HashMap::new();
                    for &param_id in sig.parameters.iter() {
                        let name = store.type_parameter(param_id).name.to_string();
                        let param_form = store.parameter_form(param_id);
                        map.insert(name, param_form);
                    }
                    map
                } else {
                    std::collections::HashMap::new()
                };
                let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                    parent: &resolver,
                    type_parameters: type_params_map,
                };

                for member in &class_def.members {
                    let side = crate::checker::declaration::member_side(member);
                    let (selector_opt, body_opt, range_opt) = match member {
                        ClassMember::Method(m) => {
                            let slots = m
                                .params
                                .iter()
                                .map(|p| {
                                    if let Some(ref l) = p.label {
                                        phalcom_common::selector::SelectorSlot::Label(l.clone())
                                    } else {
                                        phalcom_common::selector::SelectorSlot::Positional
                                    }
                                })
                                .collect::<Vec<_>>();
                            (Selector::method(&m.name, slots).ok(), m.body.statements(), Some(m.range))
                        }
                        ClassMember::Getter(g) => (Selector::getter(&g.name).ok(), g.body.statements(), Some(g.range)),
                        ClassMember::Setter(s) => (Selector::setter(&s.name).ok(), s.body.statements(), Some(s.range)),
                        _ => (None, None, None),
                    };

                    if let (Some(selector), Some(body), Some(range)) = (selector_opt, body_opt, range_opt) {
                        let callable_id = crate::identity::CallableId::new(decl_id.clone(), selector, side);
                        let analysis = crate::checker::body::analyze_callable_body(
                            callable_id.clone(),
                            body,
                            range,
                            &mut store,
                            &hierarchy,
                            &scoped_resolver,
                            &declarations,
                            module_id.clone(),
                            crate::db::budget::QueryBudget::default(),
                            &crate::db::budget::CancellationToken::new(),
                        );
                        callable_analyses.insert(callable_id, Arc::new(analysis));
                    }
                }
            }
        }

        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module_id.clone());
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
            diags_by_module.entry(module_id.clone()).or_default().extend(ctx.diagnostics);
        }
    }

    // -------------------------------------------------------------------------
    // Phase I & J: Freeze and Publish Immutable Snapshot
    // -------------------------------------------------------------------------
    let mut diagnostics_map = BTreeMap::new();
    for (module_id, diags) in diags_by_module {
        diagnostics_map.insert(module_id, Arc::from(diags.into_boxed_slice()));
    }

    let snapshot = Arc::new(SemanticSnapshot::new_with_callable_analyses(
        input.generation,
        Arc::new(store),
        Arc::new(input.sources),
        Arc::new(dispatch.surfaces().clone()),
        Arc::new(dispatch),
        Arc::new(callable_signatures),
        Arc::new(declarations),
        Arc::new(hierarchy),
        Arc::new(diagnostics_map),
        Arc::new(semantic_graph),
        Arc::new(callable_analyses),
    ));

    SemanticAnalysis { snapshot }
}

/// Convenience helper to analyze a single module as a standalone workspace.
pub fn analyze_single_module(module: ModuleId, source: Arc<str>, program: Arc<Program>) -> SemanticAnalysis {
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
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, source, program)),
    );

    analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 0,
    })
}
