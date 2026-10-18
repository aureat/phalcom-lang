//! Projection and materialization of completed Universe module authority.
//! No source provider, interface builder, or linker is used by import.
use crate::authority::{ModuleAuthorityBuildError, ModuleAuthorityMap, ModuleAuthoritySet};
use crate::canonical_products::CanonicalExternalModuleWorld;
use crate::graph::*;
use crate::interface::*;
use crate::linker::*;
use crate::metadata::*;
use crate::{ModuleComponent, ModuleId, ModuleKind, ModulePath, ModuleTopology, ProjectUniverse, ResolverGeneration, TopologyFingerprint, TopologyNode};
use phalcom_ast::ast::{
    ExportAlias, ExportItem, ImportAlias, ImportItem, ImportPath, ImportRoot, MetadataLiteral, ModuleImportDecl, PathSegment, ReExportDecl, SelectiveImportDecl,
};
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::module_world::*;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::collections::BTreeMap;
use std::sync::Arc;

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

fn authority_error(error: ModuleAuthorityBuildError) -> ArtifactError {
    match error {
        ModuleAuthorityBuildError::Allocation(err) => allocation_error(err),
        _ => invalid("invalid module authority entries"),
    }
}

fn span(range: SourceRange) -> SpanV1 {
    SpanV1 {
        start: range.start as u64,
        end: range.end as u64,
    }
}
fn range(span: &SpanV1) -> Result<SourceRange, ArtifactError> {
    if span.start > span.end {
        return Err(invalid("reversed module source range"));
    }
    Ok(SourceRange {
        start: usize::try_from(span.start).map_err(|_| invalid("source start does not fit host"))?,
        end: usize::try_from(span.end).map_err(|_| invalid("source end does not fit host"))?,
    })
}
fn validate_module_id(ids: &[ModuleId], id: ArtifactModuleId) -> Result<(), ArtifactError> {
    if (id.0 as usize) < ids.len() {
        Ok(())
    } else {
        Err(invalid("out-of-range artifact module reference"))
    }
}
fn module_id(
    ids: &[ModuleId],
    id: ArtifactModuleId,
    allocations: &mut AllocationBudget,
) -> Result<ModuleId, ArtifactError> {
    let source = ids
        .get(id.0 as usize)
        .ok_or_else(|| invalid("out-of-range artifact module reference"))?;
    source.clone_with_allocation_budget(allocations).map_err(allocation_error)
}
fn portable_id(ids: &BTreeMap<ModuleId, ArtifactModuleId>, id: &ModuleId) -> Result<ArtifactModuleId, ArtifactError> {
    ids.get(id)
        .copied()
        .ok_or_else(|| invalid(format!("module {id} is outside the closed artifact world")))
}
fn kind_out(kind: ModuleKind) -> ModuleKindV1 {
    match kind {
        ModuleKind::Module => ModuleKindV1::Module,
        ModuleKind::Package => ModuleKindV1::Package,
    }
}
fn kind_in(kind: &ModuleKindV1) -> ModuleKind {
    match kind {
        ModuleKindV1::Module => ModuleKind::Module,
        ModuleKindV1::Package => ModuleKind::Package,
    }
}
fn segment_out(segment: &PathSegment) -> SegmentV1 {
    SegmentV1 {
        name: segment.name.clone(),
        span: span(segment.range),
    }
}
fn segment_in(segment: &SegmentV1, allocations: &mut AllocationBudget) -> Result<PathSegment, ArtifactError> {
    ModuleComponent::validate_identifier(&segment.name).map_err(|e| invalid(e))?;
    let name = allocations.string(&segment.name).map_err(allocation_error)?;
    Ok(PathSegment {
        name,
        range: range(&segment.span)?,
    })
}
fn path_out(path: &ImportPath) -> ImportPathV1 {
    ImportPathV1 {
        root: match &path.root {
            ImportRoot::Absolute(segment) => ImportRootV1::Absolute { segment: segment_out(segment) },
            ImportRoot::Relative { dots, range } => ImportRootV1::Relative {
                dots: *dots,
                span: span(*range),
            },
        },
        segments: path.segments.iter().map(segment_out).collect(),
        span: span(path.range),
    }
}
fn path_in(path: &ImportPathV1, allocations: &mut AllocationBudget) -> Result<ImportPath, ArtifactError> {
    let root = match &path.root {
        ImportRootV1::Absolute { segment } => ImportRoot::Absolute(segment_in(segment, allocations)?),
        ImportRootV1::Relative { dots, span } => {
            if *dots == 0 {
                return Err(invalid("relative import requires at least one dot"));
            }
            ImportRoot::Relative {
                dots: *dots,
                range: range(span)?,
            }
        }
    };
    let mut segments = allocations.vector::<PathSegment>(path.segments.len()).map_err(allocation_error)?;
    for s in &path.segments {
        segments.push(segment_in(s, allocations)?);
    }
    Ok(ImportPath {
        root,
        segments,
        range: range(&path.span)?,
    })
}
fn alias_out(alias: &ImportAlias) -> AliasV1 {
    AliasV1 {
        name: alias.name.clone(),
        span: span(alias.range),
    }
}
fn alias_in(alias: &AliasV1, allocations: &mut AllocationBudget) -> Result<ImportAlias, ArtifactError> {
    if alias.name.is_empty() {
        return Err(invalid("empty import alias"));
    }
    let name = allocations.string(&alias.name).map_err(allocation_error)?;
    Ok(ImportAlias {
        name,
        range: range(&alias.span)?,
    })
}
fn import_out(import: &ImportSurface) -> ImportV1 {
    match import {
        ImportSurface::Module(value) => ImportV1::Module {
            path: path_out(&value.path),
            alias: value.alias.as_ref().map(alias_out),
            span: span(value.range),
        },
        ImportSurface::Selective(value) => ImportV1::Selective {
            path: path_out(&value.path),
            items: value
                .items
                .iter()
                .map(|item| ImportItemV1 {
                    name: item.name.clone(),
                    name_span: span(item.name_range),
                    alias: item.alias.as_ref().map(alias_out),
                    span: span(item.range),
                })
                .collect(),
            span: span(value.range),
        },
        ImportSurface::ReExport(value) => ImportV1::ReExport {
            path: path_out(&value.path),
            items: value
                .items
                .iter()
                .map(|item| ImportItemV1 {
                    name: item.local_or_remote_name.clone(),
                    name_span: span(item.name_range),
                    alias: item.alias.as_ref().map(|alias| AliasV1 {
                        name: alias.name.clone(),
                        span: span(alias.range),
                    }),
                    span: span(item.range),
                })
                .collect(),
            span: span(value.range),
        },
    }
}
fn import_in(import: &ImportV1, allocations: &mut AllocationBudget) -> Result<ImportSurface, ArtifactError> {
    Ok(match import {
        ImportV1::Module { path, alias, span } => ImportSurface::Module(ModuleImportDecl {
            path: path_in(path, allocations)?,
            alias: alias.as_ref().map(|a| alias_in(a, allocations)).transpose()?,
            range: range(span)?,
        }),
        ImportV1::Selective { path, items, span } => {
            let mut item_vec = allocations.vector::<ImportItem>(items.len()).map_err(allocation_error)?;
            for item in items {
                let name = allocations.string(&item.name).map_err(allocation_error)?;
                let alias = item.alias.as_ref().map(|a| alias_in(a, allocations)).transpose()?;
                item_vec.push(ImportItem {
                    name,
                    name_range: range(&item.name_span)?,
                    alias,
                    range: range(&item.span)?,
                });
            }
            ImportSurface::Selective(SelectiveImportDecl {
                path: path_in(path, allocations)?,
                items: item_vec,
                range: range(span)?,
            })
        }
        ImportV1::ReExport { path, items, span } => {
            let mut item_vec = allocations.vector::<ExportItem>(items.len()).map_err(allocation_error)?;
            for item in items {
                let local_or_remote_name = allocations.string(&item.name).map_err(allocation_error)?;
                let alias = item
                    .alias
                    .as_ref()
                    .map(|alias| {
                        Ok::<_, ArtifactError>(ExportAlias {
                            name: allocations.string(&alias.name).map_err(allocation_error)?,
                            range: range(&alias.span)?,
                        })
                    })
                    .transpose()?;
                item_vec.push(ExportItem {
                    local_or_remote_name,
                    name_range: range(&item.name_span)?,
                    alias,
                    range: range(&item.span)?,
                });
            }
            ImportSurface::ReExport(ReExportDecl {
                path: path_in(path, allocations)?,
                items: item_vec,
                range: range(span)?,
            })
        }
    })
}
fn literal_out(value: &MetadataLiteral, depth: usize, limits: &ValidationLimits) -> Result<LiteralV1, ArtifactError> {
    if depth >= limits.max_recursion_depth {
        return Err(invalid("metadata nesting exceeds limit"));
    }
    Ok(match value {
        MetadataLiteral::Unit => LiteralV1::Unit,
        MetadataLiteral::Bool(value) => LiteralV1::Bool { value: *value },
        MetadataLiteral::Int(value) => LiteralV1::Int { value: value.clone() },
        MetadataLiteral::Float(value) => LiteralV1::Float { bits: value.to_bits() },
        MetadataLiteral::String(value) => LiteralV1::String { value: value.clone() },
        MetadataLiteral::Symbol(value) => LiteralV1::Symbol { value: value.clone() },
        MetadataLiteral::Tuple(values) => LiteralV1::Tuple {
            values: values.iter().map(|value| literal_out(value, depth + 1, limits)).collect::<Result<_, _>>()?,
        },
        MetadataLiteral::Record(fields) => LiteralV1::Record {
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok(LiteralFieldV1 {
                        name: name.clone(),
                        value: literal_out(value, depth + 1, limits)?,
                    })
                })
                .collect::<Result<_, ArtifactError>>()?,
        },
    })
}
fn literal_in(
    value: &LiteralV1,
    depth: usize,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<MetadataLiteral, ArtifactError> {
    if depth >= limits.max_recursion_depth {
        return Err(invalid("metadata nesting exceeds limit"));
    }
    Ok(match value {
        LiteralV1::Unit => MetadataLiteral::Unit,
        LiteralV1::Bool { value } => MetadataLiteral::Bool(*value),
        LiteralV1::Int { value } => MetadataLiteral::Int(allocations.string(value).map_err(allocation_error)?),
        LiteralV1::Float { bits } => MetadataLiteral::Float(f64::from_bits(*bits)),
        LiteralV1::String { value } => MetadataLiteral::String(allocations.string(value).map_err(allocation_error)?),
        LiteralV1::Symbol { value } => MetadataLiteral::Symbol(allocations.string(value).map_err(allocation_error)?),
        LiteralV1::Tuple { values } => {
            let mut tuple_vals = allocations.vector::<MetadataLiteral>(values.len()).map_err(allocation_error)?;
            for val in values {
                tuple_vals.push(literal_in(val, depth + 1, limits, allocations)?);
            }
            MetadataLiteral::Tuple(tuple_vals)
        }
        LiteralV1::Record { fields } => {
            let mut rec_fields = allocations.vector::<(String, MetadataLiteral)>(fields.len()).map_err(allocation_error)?;
            for field in fields {
                let name = allocations.string(&field.name).map_err(allocation_error)?;
                let lit = literal_in(&field.value, depth + 1, limits, allocations)?;
                rec_fields.push((name, lit));
            }
            MetadataLiteral::Record(rec_fields)
        }
    })
}
fn metadata_out(value: &ModuleMetadata, limits: &ValidationLimits) -> Result<Vec<MetadataV1>, ArtifactError> {
    value
        .attributes
        .iter()
        .map(|attribute| {
            Ok(MetadataV1 {
                target: match attribute.target {
                    MetadataTarget::Module => MetadataTargetV1::Module,
                    MetadataTarget::Package => MetadataTargetV1::Package,
                    MetadataTarget::Project => MetadataTargetV1::Project,
                },
                name: attribute.name.clone(),
                arguments: attribute.arguments.iter().map(|arg| literal_out(arg, 0, limits)).collect::<Result<_, _>>()?,
                span: span(attribute.range),
            })
        })
        .collect()
}
fn metadata_in(
    value: &[MetadataV1],
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ModuleMetadata, ArtifactError> {
    let mut attributes = allocations.vector::<ModuleMetadataAttribute>(value.len()).map_err(allocation_error)?;
    for attribute in value {
        let name = allocations.string(&attribute.name).map_err(allocation_error)?;
        let mut arguments = allocations.vector::<MetadataLiteral>(attribute.arguments.len()).map_err(allocation_error)?;
        for arg in &attribute.arguments {
            arguments.push(literal_in(arg, 0, limits, allocations)?);
        }
        attributes.push(ModuleMetadataAttribute {
            target: match attribute.target {
                MetadataTargetV1::Module => MetadataTarget::Module,
                MetadataTargetV1::Package => MetadataTarget::Package,
                MetadataTargetV1::Project => MetadataTarget::Project,
            },
            name,
            arguments,
            range: range(&attribute.span)?,
        });
    }
    Ok(ModuleMetadata { attributes })
}
fn target_out(target: &LinkedReadSpec, ids: &BTreeMap<ModuleId, ArtifactModuleId>) -> Result<TargetV1, ArtifactError> {
    Ok(match target {
        LinkedReadSpec::Module(module) => TargetV1::Module {
            module: portable_id(ids, module)?,
        },
        LinkedReadSpec::Binding(symbol) => TargetV1::Binding {
            module: portable_id(ids, &symbol.module)?,
            name: symbol.name.to_string(),
        },
    })
}
fn target_in(
    target: &TargetV1,
    ids: &[ModuleId],
    allocations: &mut AllocationBudget,
) -> Result<LinkedReadSpec, ArtifactError> {
    Ok(match target {
        TargetV1::Module { module } => LinkedReadSpec::Module(module_id(ids, *module, allocations)?),
        TargetV1::Binding { module, name } => {
            if name.is_empty() {
                return Err(invalid("empty linked symbol"));
            }
            LinkedReadSpec::Binding(SymbolId {
                module: module_id(ids, *module, allocations)?,
                name: allocations.boxed_str(name).map_err(allocation_error)?,
            })
        }
    })
}
fn node_out(node: &SemanticNodeId, ids: &BTreeMap<ModuleId, ArtifactModuleId>) -> Result<SemanticNodeV1, ArtifactError> {
    Ok(match node {
        SemanticNodeId::Module(module) => SemanticNodeV1::Module {
            module: portable_id(ids, module)?,
        },
        SemanticNodeId::Declaration { module, name } => SemanticNodeV1::Declaration {
            module: portable_id(ids, module)?,
            name: name.to_string(),
        },
    })
}
fn node_in(
    node: &SemanticNodeV1,
    ids: &[ModuleId],
    allocations: &mut AllocationBudget,
) -> Result<SemanticNodeId, ArtifactError> {
    Ok(match node {
        SemanticNodeV1::Module { module } => SemanticNodeId::Module(module_id(ids, *module, allocations)?),
        SemanticNodeV1::Declaration { module, name } => {
            if name.is_empty() {
                return Err(invalid("empty semantic declaration name"));
            }
            SemanticNodeId::Declaration {
                module: module_id(ids, *module, allocations)?,
                name: allocations.boxed_str(name).map_err(allocation_error)?,
            }
        }
    })
}
fn ordered_names<'a>(names: impl IntoIterator<Item = &'a str>) -> Result<(), ArtifactError> {
    let mut previous = None;
    for name in names {
        if name.is_empty() || previous.is_some_and(|prev| prev >= name) {
            return Err(invalid("names must be nonempty, unique, and sorted"));
        }
        previous = Some(name);
    }
    Ok(())
}

/// Projects completed authority. Stable Universe paths define artifact-local indices.
pub fn project_module_world(world: &CanonicalExternalModuleWorld, limits: &ValidationLimits) -> Result<ModuleWorldV1, ArtifactError> {
    let linked = world.linked();
    if !linked.universe.projects().is_empty() || linked.modules.keys().any(|id| !id.project.is_universe()) {
        return Err(invalid("v1 Universe projection cannot persist user project graph identities"));
    }
    if linked.modules.len() > limits.max_modules {
        return Err(invalid("too many canonical modules"));
    }
    let ids = linked
        .modules
        .keys()
        .enumerate()
        .map(|(index, id)| {
            Ok((
                id.clone(),
                ArtifactModuleId(u32::try_from(index).map_err(|_| invalid("module index exceeds u32"))?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ArtifactError>>()?;
    let modules = linked
        .modules
        .iter()
        .map(|(id, module)| {
            let interface = world.interfaces().get(id).ok_or_else(|| invalid("missing canonical interface"))?;
            if interface.metadata != module.interface.metadata {
                return Err(invalid("unlinked and linked metadata differ"));
            }
            let node = world.topology().nodes.get(id).ok_or_else(|| invalid("missing canonical topology node"))?;
            Ok(ModuleV1 {
                path: id.path.components().iter().map(|part| part.as_str().to_owned()).collect(),
                kind: kind_out(interface.kind),
                declarations: interface
                    .declarations
                    .iter()
                    .map(|(name, declaration)| {
                        if name != &declaration.name {
                            return Err(invalid("declaration key/name mismatch"));
                        }
                        Ok(DeclarationV1 {
                            name: name.clone(),
                            semantics: match declaration.semantics {
                                ModuleBindingSemantics::Runtime { mutable } => BindingSemanticsV1::Runtime { mutable },
                                ModuleBindingSemantics::StaticConstant => BindingSemanticsV1::StaticConstant,
                                ModuleBindingSemantics::Declaration => BindingSemanticsV1::Declaration,
                            },
                            span: span(declaration.range),
                        })
                    })
                    .collect::<Result<_, _>>()?,
                exports: interface
                    .exports
                    .iter()
                    .map(|(name, export)| {
                        if name != &export.exported_name {
                            return Err(invalid("export key/name mismatch"));
                        }
                        Ok(ExportV1 {
                            exported_name: export.exported_name.clone(),
                            internal_name: export.internal_name.clone(),
                            target: match &export.target {
                                UnlinkedExportTarget::Local(name) => UnlinkedTargetV1::Local { name: name.clone() },
                                UnlinkedExportTarget::ReExport { path, remote } => UnlinkedTargetV1::ReExport {
                                    path: path_out(path),
                                    remote: remote.clone(),
                                },
                                UnlinkedExportTarget::CanonicalDeclaration { module, name } => UnlinkedTargetV1::CanonicalDeclaration {
                                    module: portable_id(&ids, module)?,
                                    name: name.clone(),
                                },
                            },
                            span: span(export.range),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                imports: interface.imports.iter().map(import_out).collect(),
                exposed_children: interface.exposed_children.iter().map(|part| part.as_str().to_owned()).collect(),
                metadata: metadata_out(&interface.metadata, limits)?,
                linked_exports: module
                    .interface
                    .exports
                    .iter()
                    .map(|(name, export)| {
                        if name != &export.public_name {
                            return Err(invalid("linked export key/name mismatch"));
                        }
                        let read = match &export.target {
                            LinkedExportTarget::Binding(symbol) => LinkedReadSpec::Binding(symbol.clone()),
                            LinkedExportTarget::Module(module) => LinkedReadSpec::Module(module.clone()),
                        };
                        Ok(LinkedExportV1 {
                            name: name.to_string(),
                            target: target_out(&read, &ids)?,
                            span: span(export.range),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                globals: module
                    .bindings
                    .local_globals
                    .iter()
                    .map(|(name, index)| BindingV1 {
                        name: name.to_string(),
                        index: index.0,
                    })
                    .collect(),
                import_bindings: module
                    .bindings
                    .imports
                    .iter()
                    .map(|(name, index)| BindingV1 {
                        name: name.to_string(),
                        index: index.0,
                    })
                    .collect(),
                linked_reads: module.linked_reads.iter().map(|read| target_out(read, &ids)).collect::<Result<_, _>>()?,
                runtime_dependencies: module.runtime_dependencies.iter().map(|id| portable_id(&ids, id)).collect::<Result<_, _>>()?,
                references: linked
                    .graphs
                    .references
                    .edges_from(id)
                    .iter()
                    .map(|edge| {
                        if &edge.from != id {
                            return Err(invalid("reference graph source mismatch"));
                        }
                        Ok(ReferenceV1 {
                            target: portable_id(&ids, &edge.to)?,
                            kind: reference_out(edge.kind),
                            span: span(edge.range),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                runtime_edges: linked
                    .graphs
                    .runtime
                    .edges_from(id)
                    .iter()
                    .map(|edge| {
                        if &edge.importer != id {
                            return Err(invalid("runtime graph source mismatch"));
                        }
                        Ok(RuntimeEdgeV1 {
                            dependency: portable_id(&ids, &edge.dependency)?,
                            reason: runtime_out(edge.reason),
                            span: span(edge.range),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                topology_parent: node.parent.as_ref().map(|id| portable_id(&ids, id)).transpose()?,
                topology_children: world
                    .topology()
                    .children
                    .get(id)
                    .into_iter()
                    .flatten()
                    .map(|id| portable_id(&ids, id))
                    .collect::<Result<_, _>>()?,
                interface_fingerprint: interface.fingerprint().raw(),
                linked_interface_fingerprint: module.interface.fingerprint().raw(),
                dependency_fingerprint: crate::fingerprint::linked_dependency_fingerprint(module).raw(),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let semantic_rows = linked
        .graphs
        .semantics
        .nodes()
        .iter()
        .map(|node| {
            Ok(SemanticRowV1 {
                node: node_out(node, &ids)?,
                edges: linked
                    .graphs
                    .semantics
                    .edges_from(node)
                    .iter()
                    .map(|edge| {
                        if &edge.from != node {
                            return Err(invalid("semantic graph source mismatch"));
                        }
                        Ok(SemanticEdgeV1 {
                            target: node_out(&edge.to, &ids)?,
                            kind: semantic_out(edge.kind),
                            span: span(edge.range),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            })
        })
        .collect::<Result<_, ArtifactError>>()?;
    Ok(ModuleWorldV1 {
        entry: portable_id(&ids, &linked.entry)?,
        initialization_order: linked.initialization_order.iter().map(|id| portable_id(&ids, id)).collect::<Result<_, _>>()?,
        modules,
        semantic_rows,
        topology_fingerprint: world.topology().fingerprint.raw(),
    })
}

/// Loads completed canonical products directly, with no source parsing or linking.
pub fn load_module_world(wire: &ModuleWorldV1, limits: &ValidationLimits) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    load_module_world_with_allocation_budget(wire, limits, &mut allocations)
}

/// Loads module authority using the caller's cumulative load transaction.
pub fn load_module_world_with_allocation_budget(
    wire: &ModuleWorldV1,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let world = load_module_world_inner(wire, limits, &mut allocations)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(world)
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::LimitExceeded(
        match error {
            AllocationError::Exhausted => "module materialization allocation budget exceeded",
            AllocationError::Overflow => "module materialization allocation layout overflow",
            AllocationError::Failed => "module materialization allocation failed",
            AllocationError::UnaccountedBoundary => "semantic artifact operation has an unaccounted allocation boundary",
        }
        .into(),
    )
}

fn load_module_world_inner(
    wire: &ModuleWorldV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    if wire.modules.is_empty() || wire.modules.len() > limits.max_modules {
        return Err(invalid("invalid canonical module count"));
    }
    let mut ids = allocations.vector::<ModuleId>(wire.modules.len()).map_err(allocation_error)?;
    for module in &wire.modules {
        if module.path.len() > limits.max_recursion_depth {
            return Err(invalid("module path exceeds depth limit"));
        }
        let mut components = allocations.vector::<ModuleComponent>(module.path.len()).map_err(allocation_error)?;
        for part in &module.path {
            ModuleComponent::validate_identifier(part).map_err(|e| invalid(e))?;
            let boxed = allocations.boxed_str(part).map_err(allocation_error)?;
            let comp = ModuleComponent::from_owned_identifier(boxed).map_err(|_| invalid("invalid identifier"))?;
            components.push(comp);
        }
        let boxed_components = allocations.boxed_slice(components).map_err(allocation_error)?;
        let path = ModulePath::from_components(boxed_components);
        let id = ModuleId::universe(path);
        if ids.last().is_some_and(|previous| previous >= &id) {
            return Err(invalid("module paths must be unique and sorted"));
        }
        ids.push(id);
    }
    let entry = module_id(&ids, wire.entry, allocations)?;
    if entry != ModuleId::universe_root() {
        return Err(invalid("Universe artifact entry must be its root module"));
    }
    let mut interfaces_entries = allocations.vector::<(ModuleId, Arc<UnlinkedModuleInterface>)>(wire.modules.len()).map_err(allocation_error)?;
    let mut modules_entries = allocations.vector::<(ModuleId, LinkedModule)>(wire.modules.len()).map_err(allocation_error)?;
    let mut ref_graph_entries = allocations.vector::<(ModuleId, Vec<ReferenceEdge>)>(wire.modules.len()).map_err(allocation_error)?;
    let mut runtime_graph_entries = allocations.vector::<(ModuleId, Vec<RuntimeDependencyEdge>)>(wire.modules.len()).map_err(allocation_error)?;
    let mut nodes_entries = allocations.vector::<(ModuleId, TopologyNode)>(wire.modules.len()).map_err(allocation_error)?;
    let mut children_entries = allocations.vector::<(ModuleId, ModuleAuthoritySet<ModuleId>)>(wire.modules.len()).map_err(allocation_error)?;
    let mut exposed_entries = allocations.vector::<(ModuleId, ModuleAuthoritySet<ModuleComponent>)>(wire.modules.len()).map_err(allocation_error)?;

    for (index, payload) in wire.modules.iter().enumerate() {
        let id = &ids[index];
        let kind = kind_in(&payload.kind);
        if id.path.is_root() && kind != ModuleKind::Package {
            return Err(invalid("Universe root must be a package"));
        }
        ordered_names(payload.declarations.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.exports.iter().map(|value| value.exported_name.as_str()))?;
        ordered_names(payload.linked_exports.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.globals.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.import_bindings.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.exposed_children.iter().map(String::as_str))?;

        let mut decl_entries = allocations.vector::<(String, DeclarationSurface)>(payload.declarations.len()).map_err(allocation_error)?;
        for value in &payload.declarations {
            let key_name = allocations.string(&value.name).map_err(allocation_error)?;
            let surface_name = allocations.string(&value.name).map_err(allocation_error)?;
            decl_entries.push((
                key_name,
                DeclarationSurface {
                    name: surface_name,
                    semantics: match value.semantics {
                        BindingSemanticsV1::Runtime { mutable } => ModuleBindingSemantics::Runtime { mutable },
                        BindingSemanticsV1::StaticConstant => ModuleBindingSemantics::StaticConstant,
                        BindingSemanticsV1::Declaration => ModuleBindingSemantics::Declaration,
                    },
                    range: range(&value.span)?,
                },
            ));
        }
        let declarations = ModuleAuthorityMap::from_budgeted_sorted_entries(decl_entries, allocations).map_err(authority_error)?;

        let mut export_entries = allocations.vector::<(String, ExportSurface)>(payload.exports.len()).map_err(allocation_error)?;
        for value in &payload.exports {
            let key_name = allocations.string(&value.exported_name).map_err(allocation_error)?;
            let exported_name = allocations.string(&value.exported_name).map_err(allocation_error)?;
            let internal_name = allocations.string(&value.internal_name).map_err(allocation_error)?;
            let target = match &value.target {
                UnlinkedTargetV1::Local { name } => {
                    UnlinkedExportTarget::Local(allocations.string(name).map_err(allocation_error)?)
                }
                UnlinkedTargetV1::ReExport { path, remote } => UnlinkedExportTarget::ReExport {
                    path: path_in(path, allocations)?,
                    remote: allocations.string(remote).map_err(allocation_error)?,
                },
                UnlinkedTargetV1::CanonicalDeclaration { module, name } => {
                    UnlinkedExportTarget::CanonicalDeclaration {
                        module: module_id(&ids, *module, allocations)?,
                        name: allocations.string(name).map_err(allocation_error)?,
                    }
                }
            };
            export_entries.push((
                key_name,
                ExportSurface {
                    exported_name,
                    internal_name,
                    target,
                    range: range(&value.span)?,
                },
            ));
        }
        let exports = ModuleAuthorityMap::from_budgeted_sorted_entries(export_entries, allocations).map_err(authority_error)?;

        let mut imports = allocations.vector::<ImportSurface>(payload.imports.len()).map_err(allocation_error)?;
        for import in &payload.imports {
            imports.push(import_in(import, allocations)?);
        }

        let mut exposed_vec = allocations.vector::<ModuleComponent>(payload.exposed_children.len()).map_err(allocation_error)?;
        for value in &payload.exposed_children {
            ModuleComponent::validate_identifier(value).map_err(|e| invalid(e))?;
            let boxed = allocations.boxed_str(value).map_err(allocation_error)?;
            let comp = ModuleComponent::from_owned_identifier(boxed).map_err(|_| invalid("invalid identifier"))?;
            exposed_vec.push(comp);
        }
        let exposed_children = ModuleAuthoritySet::from_budgeted_sorted_entries(exposed_vec, allocations).map_err(authority_error)?;
        if kind != ModuleKind::Package && !exposed_children.is_empty() {
            return Err(invalid("ordinary module cannot expose children"));
        }

        let unlinked_metadata = metadata_in(&payload.metadata, limits, allocations)?;
        for attribute in &unlinked_metadata.attributes {
            if attribute.target
                != match kind {
                    ModuleKind::Module => MetadataTarget::Module,
                    ModuleKind::Package => MetadataTarget::Package,
                }
            {
                return Err(invalid("metadata target differs from module kind"));
            }
        }
        let linked_metadata = metadata_in(&payload.metadata, limits, allocations)?;

        let interface = UnlinkedModuleInterface {
            id: id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            kind,
            declarations,
            exports,
            imports,
            exposed_children: exposed_children.clone(),
            metadata: unlinked_metadata,
        };
        if interface.fingerprint().raw() != payload.interface_fingerprint {
            return Err(invalid("unlinked interface fingerprint mismatch"));
        }

        let mut linked_export_entries = allocations.vector::<(Box<str>, LinkedExport)>(payload.linked_exports.len()).map_err(allocation_error)?;
        for value in &payload.linked_exports {
            let target = match target_in(&value.target, &ids, allocations)? {
                LinkedReadSpec::Module(module) => LinkedExportTarget::Module(module),
                LinkedReadSpec::Binding(symbol) => LinkedExportTarget::Binding(symbol),
            };
            let key_name = allocations.boxed_str(&value.name).map_err(allocation_error)?;
            let public_name = allocations.boxed_str(&value.name).map_err(allocation_error)?;
            linked_export_entries.push((
                key_name,
                LinkedExport {
                    public_name,
                    target,
                    range: range(&value.span)?,
                },
            ));
        }
        let linked_exports = ModuleAuthorityMap::from_budgeted_sorted_entries(linked_export_entries, allocations).map_err(authority_error)?;
        if interface
            .exports
            .keys()
            .map(String::as_str)
            .ne(payload.linked_exports.iter().map(|value| value.name.as_str()))
        {
            return Err(invalid("unlinked and linked public export names disagree"));
        }
        let linked_interface = LinkedModuleInterface {
            module: id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            kind,
            exports: linked_exports,
            metadata: linked_metadata,
        };
        if linked_interface.fingerprint().raw() != payload.linked_interface_fingerprint {
            return Err(invalid("linked interface fingerprint mismatch"));
        }

        let mut globals_entries = allocations.vector::<(Box<str>, GlobalBindingId)>(payload.globals.len()).map_err(allocation_error)?;
        for value in &payload.globals {
            let name = allocations.boxed_str(&value.name).map_err(allocation_error)?;
            globals_entries.push((name, GlobalBindingId(value.index)));
        }
        let globals = ModuleAuthorityMap::from_budgeted_sorted_entries(globals_entries, allocations).map_err(authority_error)?;

        let mut import_binding_entries = allocations.vector::<(Box<str>, ImportBindingId)>(payload.import_bindings.len()).map_err(allocation_error)?;
        for value in &payload.import_bindings {
            let name = allocations.boxed_str(&value.name).map_err(allocation_error)?;
            import_binding_entries.push((name, ImportBindingId(value.index)));
        }
        let import_bindings = ModuleAuthorityMap::from_budgeted_sorted_entries(import_binding_entries, allocations).map_err(authority_error)?;

        let mut linked_reads = allocations.vector::<LinkedReadSpec>(payload.linked_reads.len()).map_err(allocation_error)?;
        for target in &payload.linked_reads {
            linked_reads.push(target_in(target, &ids, allocations)?);
        }

        let mut runtime_dependencies = allocations.vector::<ModuleId>(payload.runtime_dependencies.len()).map_err(allocation_error)?;
        for value in &payload.runtime_dependencies {
            runtime_dependencies.push(module_id(&ids, *value, allocations)?);
        }

        let linked_module = LinkedModule {
            interface: linked_interface,
            bindings: ModuleBindingLayout {
                local_globals: globals,
                imports: import_bindings,
            },
            linked_reads,
            runtime_dependencies,
        };
        if crate::fingerprint::linked_dependency_fingerprint(&linked_module).raw() != payload.dependency_fingerprint {
            return Err(invalid("private linkage fingerprint mismatch"));
        }
        let mut global_indices = allocations.vector::<bool>(payload.globals.len()).map_err(allocation_error)?;
        global_indices.resize(payload.globals.len(), false);
        for value in &payload.globals {
            let index = value.index as usize;
            if index >= payload.globals.len() || global_indices[index] || !interface.declarations.contains_key(&value.name) {
                return Err(invalid("invalid global binding layout"));
            }
            global_indices[index] = true;
        }
        let mut import_indices = allocations.vector::<bool>(payload.linked_reads.len()).map_err(allocation_error)?;
        import_indices.resize(payload.linked_reads.len(), false);
        for value in &payload.import_bindings {
            let index = value.index as usize;
            if index >= payload.linked_reads.len() || import_indices[index] || interface.declarations.contains_key(&value.name) {
                return Err(invalid("invalid import binding layout"));
            }
            import_indices[index] = true;
        }

        let mut ref_edges = allocations.vector::<ReferenceEdge>(payload.references.len()).map_err(allocation_error)?;
        for edge in &payload.references {
            ref_edges.push(ReferenceEdge {
                from: id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                to: module_id(&ids, edge.target, allocations)?,
                kind: reference_in(&edge.kind),
                range: range(&edge.span)?,
            });
        }
        ref_graph_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, ref_edges));

        let mut rt_edges = allocations.vector::<RuntimeDependencyEdge>(payload.runtime_edges.len()).map_err(allocation_error)?;
        for edge in &payload.runtime_edges {
            rt_edges.push(RuntimeDependencyEdge {
                importer: id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                dependency: module_id(&ids, edge.dependency, allocations)?,
                reason: runtime_in(&edge.reason),
                range: range(&edge.span)?,
            });
        }
        let mut rt_deps = allocations.vector::<&ModuleId>(rt_edges.len()).map_err(allocation_error)?;
        rt_deps.extend(rt_edges.iter().map(|e| &e.dependency));
        rt_deps.sort();
        rt_deps.dedup();
        if rt_deps.iter().copied().ne(linked_module.runtime_dependencies.iter()) {
            return Err(invalid("runtime graph and linked dependency list disagree"));
        }
        runtime_graph_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, rt_edges));

        let parent = payload.topology_parent.map(|value| module_id(&ids, value, allocations)).transpose()?;
        if !has_expected_topology_parent(&id, parent.as_ref()) {
            return Err(invalid("topology parent differs from logical module path"));
        }
        let node_id = id.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        nodes_entries.push((
            id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            TopologyNode {
                module_id: node_id,
                kind,
                source: None,
                project: id.project,
                parent,
            },
        ));

        let mut child_indices = allocations.vector::<u32>(payload.topology_children.len()).map_err(allocation_error)?;
        for value in &payload.topology_children {
            validate_module_id(&ids, *value)?;
            child_indices.push(value.0);
        }
        child_indices.sort_unstable();
        if child_indices.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid("duplicate topology child"));
        }
        if !child_indices.is_empty() {
            let mut child_vec = allocations.vector::<ModuleId>(child_indices.len()).map_err(allocation_error)?;
            for index in child_indices {
                child_vec.push(module_id(&ids, ArtifactModuleId(index), allocations)?);
            }
            let child_set = ModuleAuthoritySet::from_budgeted_sorted_entries(child_vec, allocations).map_err(authority_error)?;
            children_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, child_set));
        }

        if !exposed_children.is_empty() {
            exposed_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, exposed_children));
        }

        let iface_arc = allocations.arc(interface).map_err(allocation_error)?;
        interfaces_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, iface_arc));
        modules_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, linked_module));
    }

    let nodes = ModuleAuthorityMap::from_budgeted_sorted_entries(nodes_entries, allocations).map_err(authority_error)?;
    let children = ModuleAuthorityMap::from_budgeted_sorted_entries(children_entries, allocations).map_err(authority_error)?;
    let exposed = ModuleAuthorityMap::from_budgeted_sorted_entries(exposed_entries, allocations).map_err(authority_error)?;
    let interfaces = ModuleAuthorityMap::from_budgeted_sorted_entries(interfaces_entries, allocations).map_err(authority_error)?;
    let modules = ModuleAuthorityMap::from_budgeted_sorted_entries(modules_entries, allocations).map_err(authority_error)?;
    let references = ReferenceGraph::from_budgeted_sorted_entries(ref_graph_entries, allocations).map_err(authority_error)?;
    let runtime = RuntimeDependencyGraph::from_budgeted_sorted_entries(runtime_graph_entries, allocations).map_err(authority_error)?;

    for (id, node) in &nodes {
        if let Some(parent) = &node.parent {
            if nodes.get(parent).is_none_or(|node| node.kind != ModuleKind::Package) || !children.get(parent).is_some_and(|set| set.contains(id)) {
                return Err(invalid("missing package parent or parent-child edge"));
            }
        }
        if let Some(set) = children.get(id) {
            for child in set {
                if nodes.get(child).and_then(|node| node.parent.as_ref()) != Some(id) {
                    return Err(invalid("topology child parent mismatch"));
                }
            }
        }
        if let Some(set) = exposed.get(id) {
            for component in set {
                let parent_components = id.path.components();
                if !children.get(id).is_some_and(|set| {
                    set.iter().any(|child| {
                        child.project == crate::ProjectIdentity::Universe
                            && child.path.components().len() == parent_components.len() + 1
                            && child.path.components()[..parent_components.len()] == *parent_components
                            && child.path.components().last() == Some(component)
                    })
                }) {
                    return Err(invalid("exposed child is absent from topology"));
                }
            }
        }
    }

    let mut semantic_node_refs = allocations.vector::<&SemanticNodeV1>(wire.semantic_rows.len()).map_err(allocation_error)?;
    semantic_node_refs.extend(wire.semantic_rows.iter().map(|row| &row.node));
    fn semantic_node_key(node: &SemanticNodeV1) -> (ArtifactModuleId, u8, &str) {
        match node {
            SemanticNodeV1::Module { module } => (*module, 0_u8, ""),
            SemanticNodeV1::Declaration { module, name } => (*module, 1_u8, name.as_str()),
        }
    }
    semantic_node_refs.sort_unstable_by_key(|node| semantic_node_key(node));
    if semantic_node_refs
        .windows(2)
        .any(|pair| semantic_node_key(pair[0]) == semantic_node_key(pair[1]))
    {
        return Err(invalid("duplicate semantic graph row"));
    }
    let mut semantic_entries = allocations.vector::<(SemanticNodeId, Vec<SemanticEdge>)>(wire.semantic_rows.len()).map_err(allocation_error)?;
    for row in &wire.semantic_rows {
        let node = node_in(&row.node, &ids, allocations)?;
        let mut edges = allocations.vector::<SemanticEdge>(row.edges.len()).map_err(allocation_error)?;
        for edge in &row.edges {
            edges.push(SemanticEdge {
                from: node.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                to: node_in(&edge.target, &ids, allocations)?,
                kind: semantic_in(&edge.kind),
                range: range(&edge.span)?,
            });
        }
        semantic_entries.push((node, edges));
    }
    let semantics = SemanticGraph::from_budgeted_entries(semantic_entries, allocations).map_err(authority_error)?;
    let graphs = ModuleGraphs {
        references,
        semantics,
        runtime,
    };

    for module in modules.values() {
        for target in &module.linked_reads {
            validate_target(target, &interfaces)?;
        }
        for export in module.interface.exports.values() {
            match &export.target {
                LinkedExportTarget::Module(_) => {}
                LinkedExportTarget::Binding(symbol) => validate_binding_target(&symbol.module, &symbol.name, &interfaces)?,
            }
        }
        if let Some(iface) = interfaces.get(&module.interface.module) {
            for export in iface.exports.values() {
                if let UnlinkedExportTarget::CanonicalDeclaration { module, name } = &export.target {
                    validate_binding_target(module, name, &interfaces)?;
                }
            }
        }
    }

    let mut positions = allocations.vector::<Option<usize>>(ids.len()).map_err(allocation_error)?;
    positions.resize(ids.len(), None);
    let mut initialization_order = allocations.vector::<ModuleId>(wire.initialization_order.len()).map_err(allocation_error)?;
    for (pos, &value) in wire.initialization_order.iter().enumerate() {
        let index = value.0 as usize;
        if index >= ids.len() {
            return Err(invalid("out-of-range artifact module reference"));
        }
        if positions[index].is_some() {
            return Err(invalid("duplicate initializer module"));
        }
        positions[index] = Some(pos);
        initialization_order.push(module_id(&ids, value, allocations)?);
    }
    for (index, &pos) in positions.iter().enumerate() {
        if let Some(pos) = pos {
            let id = &ids[index];
            for edge in graphs.runtime.edges_from(id) {
                let dependency = &edge.dependency;
                let dep_idx = ids.binary_search(dependency).map_err(|_| invalid("runtime graph dependency is absent"))?;
                match positions[dep_idx] {
                    Some(dep_pos) if dep_pos < pos => {}
                    _ => return Err(invalid("initializer order violates runtime dependency")),
                }
            }
        }
    }

    // Ordering evidence above proves the eager subgraph; validate all modules too.
    validate_runtime_dag(&graphs.runtime, &ids, allocations)?;
    let universe = allocations.arc(ProjectUniverse::new()).map_err(allocation_error)?;
    let empty_sources = ModuleAuthorityMap::from_budgeted_sorted_entries(
        allocations.vector(0).map_err(allocation_error)?,
        allocations,
    ).map_err(authority_error)?;
    let topology = ModuleTopology {
        generation: ResolverGeneration(0),
        fingerprint: TopologyFingerprint(wire.topology_fingerprint),
        nodes,
        source_modules: empty_sources,
        children,
        exposed_children: exposed,
    };
    topology.validate_completed(&universe).map_err(invalid)?;
    let linked = allocations
        .arc(LinkedProgram {
            universe,
            modules,
            graphs,
            entry,
            initialization_order,
        })
        .map_err(allocation_error)?;
    let topology = allocations.arc(topology).map_err(allocation_error)?;
    CanonicalExternalModuleWorld::from_parts(linked, interfaces, topology).map_err(invalid)
}
/// Iterative Kahn validation avoids recursive SCC traversal on artifact input.
fn validate_runtime_dag(graph: &RuntimeDependencyGraph, ids: &[ModuleId], allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
    let edge_count = ids
        .iter()
        .try_fold(0_usize, |count, id| {
            count.checked_add(graph.edges_from(id).len()).ok_or(AllocationError::Overflow)
        })
        .map_err(allocation_error)?;
    let mut edges = allocations.vector::<(usize, usize)>(edge_count).map_err(allocation_error)?;
    let mut degrees = allocations.vector::<usize>(ids.len()).map_err(allocation_error)?;
    degrees.resize(ids.len(), 0);
    for (user, id) in ids.iter().enumerate() {
        for edge in graph.edges_from(id) {
            let dependency = ids.binary_search(&edge.dependency).map_err(|_| invalid("runtime graph dependency is absent"))?;
            degrees[user] = degrees[user].checked_add(1).ok_or(AllocationError::Overflow)?;
            edges.push((dependency, user));
        }
    }
    edges.sort_unstable();

    let offset_count = ids.len().checked_add(1).ok_or(AllocationError::Overflow)?;
    let mut offsets = allocations.vector::<usize>(offset_count).map_err(allocation_error)?;
    offsets.resize(offset_count, 0);
    for &(dependency, _) in &edges {
        offsets[dependency + 1] = offsets[dependency + 1].checked_add(1).ok_or(AllocationError::Overflow)?;
    }
    for index in 0..ids.len() {
        offsets[index + 1] = offsets[index + 1].checked_add(offsets[index]).ok_or(AllocationError::Overflow)?;
    }

    let mut ready = allocations.vector::<usize>(ids.len()).map_err(allocation_error)?;
    ready.extend(degrees.iter().enumerate().filter_map(|(index, degree)| (*degree == 0).then_some(index)));
    let mut processed = 0;
    while let Some(dependency) = ready.pop() {
        processed += 1;
        for &(_, user) in &edges[offsets[dependency]..offsets[dependency + 1]] {
            let degree = degrees.get_mut(user).ok_or_else(|| invalid("runtime graph user is absent"))?;
            *degree = degree.checked_sub(1).ok_or_else(|| invalid("runtime graph degree underflow"))?;
            if *degree == 0 {
                ready.push(user);
            }
        }
    }
    if processed != ids.len() {
        return Err(invalid("cyclic runtime dependency graph"));
    }
    Ok(())
}

fn has_expected_topology_parent(id: &ModuleId, parent: Option<&ModuleId>) -> bool {
    let components = id.path.components();
    match components.split_last() {
        None => parent.is_none(),
        Some((_, expected_components)) => parent.is_some_and(|parent| {
            parent.project == crate::ProjectIdentity::Universe && parent.path.components() == expected_components
        }),
    }
}

fn validate_binding_target(
    module: &ModuleId,
    name: &str,
    interfaces: &ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>>,
) -> Result<(), ArtifactError> {
    if !interfaces
        .get(module)
        .is_some_and(|interface| interface.declarations.contains_key(name) || interface.exports.contains_key(name))
    {
        return Err(invalid(format!("linked binding has no owning declaration: {module:?}::{name}")));
    }
    Ok(())
}

fn validate_target(target: &LinkedReadSpec, interfaces: &ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>>) -> Result<(), ArtifactError> {
    if let LinkedReadSpec::Binding(symbol) = target {
        validate_binding_target(&symbol.module, &symbol.name, interfaces)?;
    }
    Ok(())
}

impl CanonicalExternalModuleWorld {
    pub fn encode_artifact(&self, limits: &ValidationLimits) -> Result<Vec<u8>, ArtifactError> {
        project_module_world(self, limits)?.encode()
    }
    pub fn decode_artifact(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        load_module_world(&ModuleWorldV1::decode(bytes, limits)?, limits)
    }
}

#[cfg(test)]
mod allocation_materialization_tests {
    use super::validate_runtime_dag;
    use crate::graph::{RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason};
    use crate::identity::{ModuleComponent, ModuleId, ModulePath};
    use phalcom_common::range::SourceRange;
    use phalcom_type_meta::allocation::AllocationBudget;
    use std::alloc::Layout;

    fn module(name: &str) -> ModuleId {
        ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]))
    }

    fn add_edge(graph: &mut RuntimeDependencyGraph, importer: &ModuleId, dependency: &ModuleId) {
        graph.add(RuntimeDependencyEdge {
            importer: importer.clone(),
            dependency: dependency.clone(),
            range: SourceRange::default(),
            reason: RuntimeDependencyReason::WholeModuleImport,
        });
    }

    #[test]
    fn runtime_dag_validation_uses_charged_dense_scratch_and_detects_cycles() {
        let ids = [module("a"), module("b"), module("c")];
        let mut graph = RuntimeDependencyGraph::default();
        for id in &ids {
            graph.add_node(id.clone());
        }
        add_edge(&mut graph, &ids[0], &ids[1]);
        add_edge(&mut graph, &ids[0], &ids[1]);
        add_edge(&mut graph, &ids[1], &ids[2]);

        let mut allocations = AllocationBudget::new(1024);
        validate_runtime_dag(&graph, &ids, &mut allocations).unwrap();
        let expected = Layout::array::<(usize, usize)>(3).unwrap().size()
            + Layout::array::<usize>(ids.len()).unwrap().size()
            + Layout::array::<usize>(ids.len() + 1).unwrap().size()
            + Layout::array::<usize>(ids.len()).unwrap().size();
        assert_eq!(allocations.requested(), expected);

        add_edge(&mut graph, &ids[2], &ids[0]);
        let mut cycle_allocations = AllocationBudget::new(1024);
        assert!(validate_runtime_dag(&graph, &ids, &mut cycle_allocations).is_err());
        assert!(cycle_allocations.requested() > 0);
    }
}

fn reference_out(value: ReferenceKind) -> ReferenceKindV1 {
    match value {
        ReferenceKind::WholeModuleImport => ReferenceKindV1::WholeModuleImport,
        ReferenceKind::SelectiveImport => ReferenceKindV1::SelectiveImport,
        ReferenceKind::ReExport => ReferenceKindV1::ReExport,
        ReferenceKind::InterfaceOnly => ReferenceKindV1::InterfaceOnly,
    }
}
fn reference_in(value: &ReferenceKindV1) -> ReferenceKind {
    match value {
        ReferenceKindV1::WholeModuleImport => ReferenceKind::WholeModuleImport,
        ReferenceKindV1::SelectiveImport => ReferenceKind::SelectiveImport,
        ReferenceKindV1::ReExport => ReferenceKind::ReExport,
        ReferenceKindV1::InterfaceOnly => ReferenceKind::InterfaceOnly,
    }
}

fn semantic_out(value: SemanticEdgeKind) -> SemanticEdgeKindV1 {
    match value {
        SemanticEdgeKind::ModuleInterface => SemanticEdgeKindV1::ModuleInterface,
        SemanticEdgeKind::TypeReference => SemanticEdgeKindV1::TypeReference,
        SemanticEdgeKind::Superclass => SemanticEdgeKindV1::Superclass,
        SemanticEdgeKind::ProtocolReference => SemanticEdgeKindV1::ProtocolReference,
        SemanticEdgeKind::ConstraintReference => SemanticEdgeKindV1::ConstraintReference,
        SemanticEdgeKind::CallbackSignature => SemanticEdgeKindV1::CallbackSignature,
        SemanticEdgeKind::AdtReference => SemanticEdgeKindV1::AdtReference,
    }
}
fn semantic_in(value: &SemanticEdgeKindV1) -> SemanticEdgeKind {
    match value {
        SemanticEdgeKindV1::ModuleInterface => SemanticEdgeKind::ModuleInterface,
        SemanticEdgeKindV1::TypeReference => SemanticEdgeKind::TypeReference,
        SemanticEdgeKindV1::Superclass => SemanticEdgeKind::Superclass,
        SemanticEdgeKindV1::ProtocolReference => SemanticEdgeKind::ProtocolReference,
        SemanticEdgeKindV1::ConstraintReference => SemanticEdgeKind::ConstraintReference,
        SemanticEdgeKindV1::CallbackSignature => SemanticEdgeKind::CallbackSignature,
        SemanticEdgeKindV1::AdtReference => SemanticEdgeKind::AdtReference,
    }
}

fn runtime_out(value: RuntimeDependencyReason) -> RuntimeReasonV1 {
    match value {
        RuntimeDependencyReason::WholeModuleImport => RuntimeReasonV1::WholeModuleImport,
        RuntimeDependencyReason::SelectiveValueImport => RuntimeReasonV1::SelectiveValueImport,
        RuntimeDependencyReason::ReExport => RuntimeReasonV1::ReExport,
        RuntimeDependencyReason::RuntimeDeclarationReference => RuntimeReasonV1::RuntimeDeclarationReference,
    }
}
fn runtime_in(value: &RuntimeReasonV1) -> RuntimeDependencyReason {
    match value {
        RuntimeReasonV1::WholeModuleImport => RuntimeDependencyReason::WholeModuleImport,
        RuntimeReasonV1::SelectiveValueImport => RuntimeDependencyReason::SelectiveValueImport,
        RuntimeReasonV1::ReExport => RuntimeDependencyReason::ReExport,
        RuntimeReasonV1::RuntimeDeclarationReference => RuntimeDependencyReason::RuntimeDeclarationReference,
    }
}
