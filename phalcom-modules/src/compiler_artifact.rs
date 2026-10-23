//! Projection and materialization of completed Universe module authority.
//! No source provider, interface builder, or linker is used by import.
use crate::authority::{ModuleAuthorityBuildError, ModuleAuthorityMap, ModuleAuthoritySet};
use crate::canonical_products::CanonicalExternalModuleWorld;
use crate::external_authority::ExternalModuleAuthority;
use crate::graph::*;
use crate::interface::*;
use crate::linker::*;
use crate::metadata::*;
use crate::{ModuleComponent, ModuleId, ModuleKind, ModulePath, ModuleTopology, ProjectUniverse, ResolverGeneration, TopologyNode};
use phalcom_ast::ast::{
    ExportAlias, ExportItem, ImportAlias, ImportItem, ImportPath, ImportRoot, MetadataLiteral, ModuleImportDecl, PathSegment, ReExportDecl, SelectiveImportDecl,
};
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::module_world::*;
use phalcom_compiler_artifact::module_world_v5::ModuleWorldV5;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Builds a private project-catalog candidate for importing an artifact.
///
/// Every package reference in the module world must be backed by a receipt from
/// the caller's trusted resolver and an existing persistent project. The package
/// identified by the artifact header is additionally bound to the digest of the
/// validated artifact bytes. The input graph is never mutated.
pub fn stage_artifact_project_authorities(
    projects: &ProjectUniverse,
    module_world: &ModuleWorldV5,
    package_identity: &TeloscPackageIdentity,
    artifact_digest: ArtifactDigest,
    resolver: &dyn crate::TrustedPackageResolver,
) -> Result<Arc<ProjectUniverse>, ArtifactError> {
    // A header cannot claim ownership solely through an external-project row:
    // the actual linked root/entry must belong to that exact package. Dependency
    // rows cannot grant publication authority to a foreign artifact.
    let entry = module_world
        .modules
        .get(module_world.entry.0 as usize)
        .ok_or_else(|| invalid("artifact entry module is absent"))?;
    if !entry.identity.path.is_empty() || !package_identity.matches_project(&entry.identity.project) {
        return Err(invalid("artifact header identity does not own its root entry module"));
    }
    // The current V4 linked targets are artifact-local ordinals. A package
    // importing another package must not embed that dependency as if it were
    // locally owned. Cross-artifact references require an explicit versioned
    // stable-reference wire format; otherwise the same native owner could be
    // published twice by independently validated artifact worlds.
    if matches!(package_identity, TeloscPackageIdentity::Package { .. })
        && module_world.modules.iter().any(|module| module.identity.project != entry.identity.project)
    {
        return Err(invalid(
            "package artifact embeds a foreign module owner; cross-artifact module references require a versioned schema",
        ));
    }

    let mut stable_projects = module_world.external_projects.iter().cloned().collect::<BTreeSet<_>>();
    stable_projects.extend(module_world.modules.iter().map(|module| module.identity.project.clone()));

    let artifact_package = match package_identity {
        TeloscPackageIdentity::Package { package, version } => {
            let mut matching = stable_projects.iter().filter(
                |project| matches!(project, StableProjectRef::Package { package: name, version: candidate, .. } if name == package && candidate == version),
            );
            let stable_project = matching
                .next()
                .cloned()
                .ok_or_else(|| invalid("package artifact header has no exact stable package owner in the module world"))?;
            if matching.next().is_some() {
                return Err(invalid("package artifact header matches multiple stable package owners"));
            }
            Some(stable_project)
        }
        TeloscPackageIdentity::Builtin { namespace, version } => {
            let exact = StableProjectRef::Builtin {
                namespace: namespace.clone(),
                version: version.clone(),
            };
            if !stable_projects.contains(&exact) {
                return Err(invalid("builtin artifact header has no exact stable builtin owner in the module world"));
            }
            None
        }
    };

    let mut staged = projects.clone();
    for stable_project in &stable_projects {
        match stable_project {
            StableProjectRef::Package { .. } => {
                let expected_digest = (artifact_package.as_ref() == Some(stable_project)).then_some(artifact_digest);
                staged
                    .register_trusted_package_authority(stable_project, expected_digest, resolver)
                    .map_err(|error| invalid(format!("package project authority rejected: {error}")))?;
            }
            StableProjectRef::Session { .. } => return Err(invalid("session project owner is forbidden")),
            StableProjectRef::Builtin { .. } | StableProjectRef::SourceArtifact { .. } => {
                if staged.resolve_stable_project(stable_project).is_none() {
                    return Err(invalid(format!("stable project has no exact persistent workspace binding: {stable_project:?}")));
                }
            }
        }
    }

    Ok(Arc::new(staged))
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
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
fn module_id(ids: &[ModuleId], id: ArtifactModuleId, allocations: &mut AllocationBudget) -> Result<ModuleId, ArtifactError> {
    let source = ids.get(id.0 as usize).ok_or_else(|| invalid("out-of-range artifact module reference"))?;
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
fn literal_in(value: &LiteralV1, depth: usize, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<MetadataLiteral, ArtifactError> {
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
fn metadata_in(value: &[MetadataV1], limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<ModuleMetadata, ArtifactError> {
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
fn target_in(target: &TargetV1, ids: &[ModuleId], allocations: &mut AllocationBudget) -> Result<LinkedReadSpec, ArtifactError> {
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
fn node_in(node: &SemanticNodeV1, ids: &[ModuleId], allocations: &mut AllocationBudget) -> Result<SemanticNodeId, ArtifactError> {
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
pub fn project_module_world(world: &CanonicalExternalModuleWorld, limits: &ValidationLimits) -> Result<ModuleWorldV2, ArtifactError> {
    let linked = world.linked();
    if !linked.universe.projects().is_empty() || linked.modules.keys().any(|id| !id.project.is_universe()) {
        return Err(invalid("v2 Universe projection cannot persist user project graph identities"));
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
            let identity = StableModuleRef {
                project: match &id.project {
                    crate::ProjectIdentity::Universe => StableProjectRef::Builtin {
                        namespace: "universe".into(),
                        version: "0.1.0".into(),
                    },
                    crate::ProjectIdentity::Resolved(_) => {
                        return Err(invalid("v2 Universe projection cannot persist user project graph identities"));
                    }
                    crate::ProjectIdentity::Synthetic(_) => {
                        return Err(invalid("synthetic project cannot be projected into stable artifact"));
                    }
                },
                path: id
                    .path
                    .components()
                    .iter()
                    .map(|part| part.as_str().into())
                    .collect::<Vec<Box<str>>>()
                    .into_boxed_slice(),
            };
            Ok(ModuleV2 {
                identity,
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
    let mut wire = ModuleWorldV2 {
        entry: portable_id(&ids, &linked.entry)?,
        initialization_order: linked.initialization_order.iter().map(|id| portable_id(&ids, id)).collect::<Result<_, _>>()?,
        modules,
        semantic_rows,
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    wire.refresh_portable_topology_fingerprint()?;
    Ok(wire)
}

/// Projects source and semantic products together with the finalized executable runtime graph.
pub fn project_executable_module_world(
    world: &CanonicalExternalModuleWorld,
    executable: &ExecutableLinkedProgram,
    limits: &ValidationLimits,
) -> Result<ModuleWorldV3, ArtifactError> {
    let (projected, _) = project_executable_module_world_with_ids(world, executable, limits)?;
    Ok(projected)
}

/// Projects the finalized module world plus the complete canonical execution
/// environment contract. The V4 schedule and proofs are relocated through the
/// same closed module-id map that projected V3; targets outside that closure
/// are publication errors.
pub fn project_executable_module_world_v4(
    world: &CanonicalExternalModuleWorld,
    executable: &ExecutableLinkedProgram,
    bootstrap_order: &[ModuleId],
    availability: &AvailabilityContract,
    limits: &ValidationLimits,
) -> Result<ModuleWorldV4, ArtifactError> {
    let (previous, ids) = project_executable_module_world_with_ids(world, executable, limits)?;

    let bootstrap_modules = bootstrap_order
        .iter()
        .map(|module| portable_id(&ids, module))
        .collect::<Result<Vec<_>, ArtifactError>>()?;

    let mut bootstrap_availability = availability
        .proofs()
        .map(|proof| {
            Ok(AvailabilityProofV1 {
                target: availability_target_out(&proof.target, &ids)?,
                product: availability_product_out(proof.product),
                authority: availability_authority_out(proof.authority),
                available_at: availability_timing_out(proof.available_at),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    bootstrap_availability.sort_by(AvailabilityProofV1::canonical_cmp);

    let projected = ModuleWorldV4::new(previous, bootstrap_modules, bootstrap_availability);
    projected.validate(limits)?;
    Ok(projected)
}

/// Projects package-owned modules and their references to external module
/// identities into schema 5. `stable_module_ref` is supplied by the caller so
/// this crate can project wire identities without owning semantic identity
/// policy.
pub fn project_package_module_world_v5(
    world: &CanonicalExternalModuleWorld,
    executable: &ExecutableLinkedProgram,
    external: &dyn crate::ExternalModuleAuthority,
    stable_module_ref: &dyn Fn(&ModuleId) -> StableModuleRef,
    availability: &AvailabilityContract,
    limits: &ValidationLimits,
) -> Result<ModuleWorldV5, ArtifactError> {
    use phalcom_compiler_artifact::module_world_v5 as v5;

    let linked = world.linked();
    if linked.modules.len() != executable.modules.len()
        || linked.modules.keys().any(|module| !executable.modules.contains_key(module))
        || executable.modules.keys().any(|module| !linked.modules.contains_key(module))
    {
        return Err(invalid("formal and finalized executable module closures differ"));
    }
    if linked.entry != executable.entry {
        return Err(invalid("formal and finalized executable entry modules differ"));
    }
    if linked.modules.len() > limits.max_modules {
        return Err(invalid("too many canonical modules"));
    }

    let local_ids = linked
        .modules
        .keys()
        .enumerate()
        .map(|(index, module)| {
            Ok((
                module.clone(),
                ArtifactModuleId(u32::try_from(index).map_err(|_| invalid("module index exceeds u32"))?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ArtifactError>>()?;
    let package_owner = linked.entry.project;
    if !matches!(package_owner, crate::ProjectIdentity::Resolved(_)) {
        return Err(invalid("package entry module must belong to a resolved package project"));
    }
    if local_ids.keys().any(|module| module.project != package_owner) {
        return Err(invalid("package module world contains a foreign local module owner"));
    }

    // Inventory only references that have a wire position capable of naming
    // an external module. Entry, schedule, topology, and semantic row owners
    // are checked as local-only during projection.
    let mut referenced = linked.modules.keys().cloned().collect::<BTreeSet<_>>();
    for (id, module) in &linked.modules {
        let interface = world.interfaces().get(id).ok_or_else(|| invalid("missing canonical interface"))?;
        for export in interface.exports.values() {
            if let UnlinkedExportTarget::CanonicalDeclaration { module, .. } = &export.target {
                referenced.insert(module.clone());
            }
        }
        for export in module.interface.exports.values() {
            match &export.target {
                LinkedExportTarget::Binding(symbol) => {
                    referenced.insert(symbol.module.clone());
                }
                LinkedExportTarget::Module(module) => {
                    referenced.insert(module.clone());
                }
            }
        }
        let finalized = executable.modules.get(id).ok_or_else(|| invalid("finalized executable module is missing"))?;
        for read in &finalized.linked_reads {
            collect_linked_read_modules(read, &mut referenced);
        }
        referenced.extend(finalized.runtime_dependencies.iter().cloned());
        for edge in linked.graphs.references.edges_from(id) {
            referenced.insert(edge.to.clone());
        }
        for edge in linked.graphs.runtime.edges_from(id) {
            referenced.insert(edge.dependency.clone());
        }
    }
    for node in linked.graphs.semantics.nodes() {
        for edge in linked.graphs.semantics.edges_from(&node) {
            collect_semantic_node_modules(&edge.to, &mut referenced);
        }
    }
    for witness in executable.runtime_availability_witnesses() {
        if let Some(target) = &witness.requirement.read_target {
            collect_linked_read_modules(target, &mut referenced);
        }
        referenced.insert(witness.requirement.dependency.clone());
        referenced.insert(witness.proof.target.module().clone());
    }

    let mut stable_by_module = BTreeMap::new();
    for module in &referenced {
        if !local_ids.contains_key(module) && !external.contains(module) {
            return Err(invalid(format!("module {module} is neither owned nor provided by the dependency base")));
        }
        let stable = stable_module_ref(module);
        if stable.path.len() != module.path.components().len()
            || !stable
                .path
                .iter()
                .zip(module.path.components())
                .all(|(stable, local)| stable.as_ref() == local.as_str())
        {
            return Err(invalid(format!("stable module conversion changed the path for {module}")));
        }
        stable_by_module.insert(module.clone(), stable);
    }

    let mut owned_project = None;
    let mut owned_identities = BTreeSet::new();
    for module in local_ids.keys() {
        let stable = stable_by_module
            .get(module)
            .ok_or_else(|| invalid("owned module is absent from the stable identity map"))?;
        if owned_project.as_ref().is_some_and(|project| project != &stable.project) {
            return Err(invalid("package modules map to multiple publishing projects"));
        }
        if !matches!(&stable.project, StableProjectRef::Package { .. }) {
            return Err(invalid("package-owned modules must use a persistent package identity"));
        }
        owned_project = Some(stable.project.clone());
        if !owned_identities.insert(stable.clone()) {
            return Err(invalid("multiple owned modules map to the same stable module identity"));
        }
    }

    // External ordinals are independent of traversal and source registration
    // order. The set is minimal because it contains only wire-capable refs.
    let external_modules = stable_by_module
        .iter()
        .filter(|(module, _)| !local_ids.contains_key(*module))
        .map(|(_, stable)| stable.clone())
        .collect::<BTreeSet<_>>();
    let external_ids = external_modules
        .iter()
        .enumerate()
        .map(|(index, module)| {
            Ok((
                module.clone(),
                v5::ExternalModuleIndex(u32::try_from(index).map_err(|_| invalid("external module index exceeds u32"))?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ArtifactError>>()?;
    let mut external_project_set = external_modules.iter().map(|module| module.project.clone()).collect::<BTreeSet<_>>();
    if let Some(owned_project) = owned_project.as_ref() {
        if external_project_set.contains(owned_project) {
            return Err(invalid("package-owned and external modules have the same stable project identity"));
        }
    }

    let modules = linked
        .modules
        .iter()
        .map(|(id, module)| {
            let finalized = executable.modules.get(id).ok_or_else(|| invalid("finalized executable module is missing"))?;
            let interface = world.interfaces().get(id).ok_or_else(|| invalid("missing canonical interface"))?;
            if interface.metadata != module.interface.metadata {
                return Err(invalid("unlinked and linked metadata differ"));
            }
            let node = world.topology().nodes.get(id).ok_or_else(|| invalid("missing canonical topology node"))?;
            let identity = stable_by_module.get(id).ok_or_else(|| invalid("owned module identity is absent"))?.clone();
            let exports = interface
                .exports
                .iter()
                .map(|(name, export)| {
                    if name != &export.exported_name {
                        return Err(invalid("export key/name mismatch"));
                    }
                    Ok(v5::ExportV2 {
                        exported_name: export.exported_name.clone(),
                        internal_name: export.internal_name.clone(),
                        target: match &export.target {
                            UnlinkedExportTarget::Local(name) => v5::UnlinkedTargetV2::Local { name: name.clone() },
                            UnlinkedExportTarget::ReExport { path, remote } => v5::UnlinkedTargetV2::ReExport {
                                path: path_out(path),
                                remote: remote.clone(),
                            },
                            UnlinkedExportTarget::CanonicalDeclaration { module, name } => v5::UnlinkedTargetV2::CanonicalDeclaration {
                                module: package_module_ref(module, &local_ids, &stable_by_module, &external_ids)?,
                                name: name.clone(),
                            },
                        },
                        span: span(export.range),
                    })
                })
                .collect::<Result<Vec<_>, ArtifactError>>()?;
            let linked_exports = module
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
                    Ok(v5::LinkedExportV2 {
                        name: name.to_string(),
                        target: package_target(&read, &local_ids, &stable_by_module, &external_ids)?,
                        span: span(export.range),
                    })
                })
                .collect::<Result<Vec<_>, ArtifactError>>()?;
            let references = linked
                .graphs
                .references
                .edges_from(id)
                .iter()
                .map(|edge| {
                    if &edge.from != id {
                        return Err(invalid("reference graph source mismatch"));
                    }
                    Ok(v5::ReferenceV2 {
                        target: package_module_ref(&edge.to, &local_ids, &stable_by_module, &external_ids)?,
                        kind: reference_out(edge.kind),
                        span: span(edge.range),
                    })
                })
                .collect::<Result<Vec<_>, ArtifactError>>()?;
            let runtime_edges = executable
                .graphs
                .runtime
                .edges_from(id)
                .iter()
                .map(|edge| {
                    if &edge.importer != id {
                        return Err(invalid("finalized runtime graph source mismatch"));
                    }
                    Ok(v5::RuntimeEdgeV2 {
                        dependency: package_module_ref(&edge.dependency, &local_ids, &stable_by_module, &external_ids)?,
                        reason: runtime_out(edge.reason),
                        span: span(edge.range),
                    })
                })
                .collect::<Result<Vec<_>, ArtifactError>>()?;
            Ok(v5::ModuleV3 {
                identity,
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
                exports,
                imports: interface.imports.iter().map(import_out).collect(),
                exposed_children: interface.exposed_children.iter().map(|part| part.as_str().to_owned()).collect(),
                metadata: metadata_out(&interface.metadata, limits)?,
                linked_exports,
                globals: module
                    .bindings
                    .local_globals
                    .iter()
                    .map(|(name, index)| BindingV1 {
                        name: name.to_string(),
                        index: index.0,
                    })
                    .collect(),
                import_bindings: finalized
                    .bindings
                    .imports
                    .iter()
                    .map(|(name, index)| BindingV1 {
                        name: name.to_string(),
                        index: index.0,
                    })
                    .collect(),
                linked_reads: finalized
                    .linked_reads
                    .iter()
                    .map(|read| package_target(read, &local_ids, &stable_by_module, &external_ids))
                    .collect::<Result<_, _>>()?,
                runtime_dependencies: finalized
                    .runtime_dependencies
                    .iter()
                    .map(|dependency| package_module_ref(dependency, &local_ids, &stable_by_module, &external_ids))
                    .collect::<Result<_, _>>()?,
                references,
                runtime_edges,
                topology_parent: node
                    .parent
                    .as_ref()
                    .map(|parent| package_local_id(&local_ids, parent, "topology parent"))
                    .transpose()?,
                topology_children: world
                    .topology()
                    .children
                    .get(id)
                    .into_iter()
                    .flatten()
                    .map(|child| package_local_id(&local_ids, child, "topology child"))
                    .collect::<Result<_, _>>()?,
                interface_fingerprint: interface.fingerprint().raw(),
                linked_interface_fingerprint: module.interface.fingerprint().raw(),
                dependency_fingerprint: crate::fingerprint::linked_dependency_fingerprint(finalized).raw(),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;

    let semantic_rows = linked
        .graphs
        .semantics
        .nodes()
        .iter()
        .map(|node| {
            let row_node = package_semantic_row_node(node, &local_ids)?;
            let edges = linked
                .graphs
                .semantics
                .edges_from(node)
                .iter()
                .map(|edge| {
                    if &edge.from != node {
                        return Err(invalid("semantic graph source mismatch"));
                    }
                    Ok(v5::SemanticEdgeV2 {
                        target: package_semantic_target(&edge.to, &local_ids, &stable_by_module, &external_ids)?,
                        kind: semantic_out(edge.kind),
                        span: span(edge.range),
                    })
                })
                .collect::<Result<_, ArtifactError>>()?;
            Ok(v5::SemanticRowV2 { node: row_node, edges })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;

    let mut availability_witnesses = executable
        .runtime_availability_witnesses()
        .iter()
        .map(|witness| {
            let requirement = &witness.requirement;
            let provenance = &requirement.provenance;
            if provenance.consumer != requirement.importer || provenance.target != witness.proof.target || provenance.product != witness.proof.product {
                return Err(invalid("finalized availability witness disagrees with its runtime requirement"));
            }
            if !local_ids.contains_key(&requirement.dependency) {
                if !requirement.dependency.project.is_universe() {
                    return Err(invalid("external package availability witnesses are not supported"));
                }
                if !availability.proofs().any(|proof| proof == &witness.proof) {
                    return Err(invalid("external Universe availability witness is absent from the package image base contract"));
                }
            }
            Ok(v5::RuntimeAvailabilityWitnessV2 {
                importer: package_local_id(&local_ids, &requirement.importer, "availability witness importer")?,
                dependency: package_module_ref(&requirement.dependency, &local_ids, &stable_by_module, &external_ids)?,
                range: span(requirement.range),
                reason: runtime_out(requirement.reason),
                read_target: requirement
                    .read_target
                    .as_ref()
                    .map(|target| package_target(target, &local_ids, &stable_by_module, &external_ids))
                    .transpose()?,
                target: package_availability_target(&witness.proof.target, &local_ids, &stable_by_module, &external_ids)?,
                product: availability_product_out(witness.proof.product),
                producer: availability_producer_out(provenance.producer),
                required_at: availability_timing_out(provenance.timing),
                authority: availability_authority_out(witness.proof.authority),
                available_at: availability_timing_out(witness.proof.available_at),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    availability_witnesses.sort_by(v5::RuntimeAvailabilityWitnessV2::canonical_cmp);

    // The V5 `entry` is the artifact's ownership root (the package's
    // root module), not the program entry: loaders and the composition
    // planner both key package authority on the empty-path module, and
    // MW5 validation requires the root row to be `entry`. The executable
    // entry (`slotmap.main`, say) belongs to executable-package authority.
    let ownership_root = ModuleId {
        project: executable.entry.project.clone(),
        path: ModulePath::root(),
    };
    let mut projected = ModuleWorldV5 {
        entry: package_local_id(&local_ids, &ownership_root, "package root module")?,
        // Upstream package modules are initialized by their own artifacts
        // before this package; only owned modules are ordered here.
        initialization_order: executable
            .initialization_order
            .iter()
            .filter(|module| module.project == executable.entry.project)
            .map(|module| package_local_id(&local_ids, module, "initialization order"))
            .collect::<Result<_, _>>()?,
        modules,
        semantic_rows,
        external_projects: std::mem::take(&mut external_project_set).into_iter().collect(),
        external_modules: external_modules.into_iter().collect(),
        topology_fingerprint: 0,
        availability_witnesses,
        bootstrap_modules: vec![],
        bootstrap_availability: vec![],
    };
    projected.refresh_portable_topology_fingerprint()?;
    projected.validate(limits)?;
    Ok(projected)
}

fn collect_linked_read_modules(read: &LinkedReadSpec, modules: &mut BTreeSet<ModuleId>) {
    match read {
        LinkedReadSpec::Module(module) => {
            modules.insert(module.clone());
        }
        LinkedReadSpec::Binding(symbol) => {
            modules.insert(symbol.module.clone());
        }
    }
}

fn collect_semantic_node_modules(node: &SemanticNodeId, modules: &mut BTreeSet<ModuleId>) {
    match node {
        SemanticNodeId::Module(module) | SemanticNodeId::Declaration { module, .. } => {
            modules.insert(module.clone());
        }
    }
}

fn package_local_id(ids: &BTreeMap<ModuleId, ArtifactModuleId>, module: &ModuleId, position: &str) -> Result<ArtifactModuleId, ArtifactError> {
    ids.get(module)
        .copied()
        .ok_or_else(|| invalid(format!("{position} must reference an owned package module: {module}")))
}

fn package_module_ref(
    module: &ModuleId,
    local_ids: &BTreeMap<ModuleId, ArtifactModuleId>,
    stable_by_module: &BTreeMap<ModuleId, StableModuleRef>,
    external_ids: &BTreeMap<StableModuleRef, phalcom_compiler_artifact::module_world_v5::ExternalModuleIndex>,
) -> Result<phalcom_compiler_artifact::module_world_v5::ModuleRefV1, ArtifactError> {
    use phalcom_compiler_artifact::module_world_v5::ModuleRefV1;

    if let Some(local) = local_ids.get(module) {
        return Ok(ModuleRefV1::Local { module: *local });
    }
    let stable = stable_by_module
        .get(module)
        .ok_or_else(|| invalid(format!("external module identity is absent: {module}")))?;
    let external = external_ids
        .get(stable)
        .copied()
        .ok_or_else(|| invalid(format!("external module index is absent: {stable:?}")))?;
    Ok(ModuleRefV1::External { module: external })
}

fn package_target(
    target: &LinkedReadSpec,
    local_ids: &BTreeMap<ModuleId, ArtifactModuleId>,
    stable_by_module: &BTreeMap<ModuleId, StableModuleRef>,
    external_ids: &BTreeMap<StableModuleRef, phalcom_compiler_artifact::module_world_v5::ExternalModuleIndex>,
) -> Result<phalcom_compiler_artifact::module_world_v5::TargetV2, ArtifactError> {
    use phalcom_compiler_artifact::module_world_v5::TargetV2;

    Ok(match target {
        LinkedReadSpec::Module(module) => TargetV2::Module {
            module: package_module_ref(module, local_ids, stable_by_module, external_ids)?,
        },
        LinkedReadSpec::Binding(symbol) => TargetV2::Binding {
            module: package_module_ref(&symbol.module, local_ids, stable_by_module, external_ids)?,
            name: symbol.name.to_string(),
        },
    })
}

fn package_semantic_target(
    node: &SemanticNodeId,
    local_ids: &BTreeMap<ModuleId, ArtifactModuleId>,
    stable_by_module: &BTreeMap<ModuleId, StableModuleRef>,
    external_ids: &BTreeMap<StableModuleRef, phalcom_compiler_artifact::module_world_v5::ExternalModuleIndex>,
) -> Result<phalcom_compiler_artifact::module_world_v5::SemanticNodeV2, ArtifactError> {
    use phalcom_compiler_artifact::module_world_v5::SemanticNodeV2;

    Ok(match node {
        SemanticNodeId::Module(module) => SemanticNodeV2::Module {
            module: package_module_ref(module, local_ids, stable_by_module, external_ids)?,
        },
        SemanticNodeId::Declaration { module, name } => SemanticNodeV2::Declaration {
            module: package_module_ref(module, local_ids, stable_by_module, external_ids)?,
            name: name.to_string(),
        },
    })
}

fn package_semantic_row_node(node: &SemanticNodeId, local_ids: &BTreeMap<ModuleId, ArtifactModuleId>) -> Result<SemanticNodeV1, ArtifactError> {
    Ok(match node {
        SemanticNodeId::Module(module) => SemanticNodeV1::Module {
            module: package_local_id(local_ids, module, "semantic row node")?,
        },
        SemanticNodeId::Declaration { module, name } => SemanticNodeV1::Declaration {
            module: package_local_id(local_ids, module, "semantic row node")?,
            name: name.to_string(),
        },
    })
}

fn package_availability_target(
    target: &RuntimeRequirementTarget,
    local_ids: &BTreeMap<ModuleId, ArtifactModuleId>,
    stable_by_module: &BTreeMap<ModuleId, StableModuleRef>,
    external_ids: &BTreeMap<StableModuleRef, phalcom_compiler_artifact::module_world_v5::ExternalModuleIndex>,
) -> Result<phalcom_compiler_artifact::module_world_v5::TargetV2, ArtifactError> {
    use phalcom_compiler_artifact::module_world_v5::TargetV2;

    Ok(match target {
        RuntimeRequirementTarget::Module(module) => TargetV2::Module {
            module: package_module_ref(module, local_ids, stable_by_module, external_ids)?,
        },
        RuntimeRequirementTarget::Binding { module, name } => TargetV2::Binding {
            module: package_module_ref(module, local_ids, stable_by_module, external_ids)?,
            name: name.to_string(),
        },
    })
}

fn project_executable_module_world_with_ids(
    world: &CanonicalExternalModuleWorld,
    executable: &ExecutableLinkedProgram,
    limits: &ValidationLimits,
) -> Result<(ModuleWorldV3, BTreeMap<ModuleId, ArtifactModuleId>), ArtifactError> {
    let linked = world.linked();
    if linked.modules.len() != executable.modules.len()
        || linked.modules.keys().any(|module| !executable.modules.contains_key(module))
        || executable.modules.keys().any(|module| !linked.modules.contains_key(module))
    {
        return Err(invalid("formal and finalized executable module closures differ"));
    }
    if linked.entry != executable.entry {
        return Err(invalid("formal and finalized executable entry modules differ"));
    }
    let ids = linked
        .modules
        .keys()
        .enumerate()
        .map(|(index, module)| {
            Ok((
                module.clone(),
                ArtifactModuleId(u32::try_from(index).map_err(|_| invalid("module index exceeds u32"))?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ArtifactError>>()?;

    let mut wire = project_module_world(world, limits)?;
    for ((id, _), module_row) in linked.modules.iter().zip(wire.modules.iter_mut()) {
        let finalized = executable.modules.get(id).ok_or_else(|| invalid("finalized executable module is missing"))?;
        module_row.import_bindings = finalized
            .bindings
            .imports
            .iter()
            .map(|(name, slot)| BindingV1 {
                name: name.to_string(),
                index: slot.0,
            })
            .collect();
        module_row.linked_reads = finalized.linked_reads.iter().map(|read| target_out(read, &ids)).collect::<Result<_, _>>()?;
        module_row.runtime_dependencies = finalized
            .runtime_dependencies
            .iter()
            .map(|dependency| portable_id(&ids, dependency))
            .collect::<Result<_, _>>()?;
        module_row.runtime_edges = executable
            .graphs
            .runtime
            .edges_from(id)
            .iter()
            .map(|edge| {
                if &edge.importer != id {
                    return Err(invalid("finalized runtime graph source mismatch"));
                }
                Ok(RuntimeEdgeV1 {
                    dependency: portable_id(&ids, &edge.dependency)?,
                    reason: runtime_out(edge.reason),
                    span: span(edge.range),
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        module_row.dependency_fingerprint = crate::fingerprint::linked_dependency_fingerprint(finalized).raw();
    }
    wire.entry = portable_id(&ids, &executable.entry)?;
    wire.initialization_order = executable
        .initialization_order
        .iter()
        .map(|module| portable_id(&ids, module))
        .collect::<Result<_, _>>()?;
    wire.refresh_portable_topology_fingerprint()?;

    let mut availability_witnesses = executable
        .runtime_availability_witnesses()
        .iter()
        .map(|witness| {
            let requirement = &witness.requirement;
            let provenance = &requirement.provenance;
            if provenance.consumer != requirement.importer || provenance.target != witness.proof.target || provenance.product != witness.proof.product {
                return Err(invalid("finalized availability witness disagrees with its runtime requirement"));
            }
            Ok(RuntimeAvailabilityWitnessV1 {
                importer: portable_id(&ids, &requirement.importer)?,
                dependency: portable_id(&ids, &requirement.dependency)?,
                range: span(requirement.range),
                reason: runtime_out(requirement.reason),
                read_target: requirement.read_target.as_ref().map(|target| target_out(target, &ids)).transpose()?,
                target: availability_target_out(&witness.proof.target, &ids)?,
                product: availability_product_out(witness.proof.product),
                producer: availability_producer_out(provenance.producer),
                required_at: availability_timing_out(provenance.timing),
                authority: availability_authority_out(witness.proof.authority),
                available_at: availability_timing_out(witness.proof.available_at),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    availability_witnesses.sort_by(RuntimeAvailabilityWitnessV1::canonical_cmp);
    let projected = ModuleWorldV3::new(wire, availability_witnesses);
    projected.validate(limits)?;
    Ok((projected, ids))
}

fn availability_target_out(target: &RuntimeRequirementTarget, ids: &BTreeMap<ModuleId, ArtifactModuleId>) -> Result<TargetV1, ArtifactError> {
    match target {
        RuntimeRequirementTarget::Module(module) => target_out(&LinkedReadSpec::Module(module.clone()), ids),
        RuntimeRequirementTarget::Binding { module, name } => target_out(
            &LinkedReadSpec::Binding(SymbolId {
                module: module.clone(),
                name: name.clone(),
            }),
            ids,
        ),
    }
}

fn availability_product_out(product: RuntimeRequirementProduct) -> AvailabilityProductV1 {
    match product {
        RuntimeRequirementProduct::CanonicalIdentity => AvailabilityProductV1::CanonicalIdentity,
        RuntimeRequirementProduct::MetadataDescriptor => AvailabilityProductV1::MetadataDescriptor,
        RuntimeRequirementProduct::RuntimeObject => AvailabilityProductV1::RuntimeObject,
        RuntimeRequirementProduct::SourceInitializedValue => AvailabilityProductV1::SourceInitializedValue,
    }
}

fn availability_authority_out(authority: AvailabilityAuthority) -> AvailabilityAuthorityV1 {
    match authority {
        AvailabilityAuthority::SemanticIdentityCatalog => AvailabilityAuthorityV1::SemanticIdentityCatalog,
        AvailabilityAuthority::NativeBindingCatalog => AvailabilityAuthorityV1::NativeBindingCatalog,
        AvailabilityAuthority::NativeModuleMaterialization => AvailabilityAuthorityV1::NativeModuleMaterialization,
        AvailabilityAuthority::SemanticMetadataBundle => AvailabilityAuthorityV1::SemanticMetadataBundle,
        AvailabilityAuthority::CanonicalSourceInitializer => AvailabilityAuthorityV1::CanonicalSourceInitializer,
        AvailabilityAuthority::FullyInitializedUniverse => AvailabilityAuthorityV1::FullyInitializedUniverse,
    }
}

fn availability_timing_out(timing: RuntimeRequirementTiming) -> AvailabilityTimingV1 {
    match timing {
        RuntimeRequirementTiming::BeforeSourceExecution => AvailabilityTimingV1::BeforeSourceExecution,
        RuntimeRequirementTiming::DuringModuleInitializer => AvailabilityTimingV1::DuringModuleInitializer,
        RuntimeRequirementTiming::AfterBootstrap => AvailabilityTimingV1::AfterBootstrap,
    }
}

fn availability_producer_out(producer: RuntimeRequirementProducer) -> AvailabilityRequirementProducerV1 {
    match producer {
        RuntimeRequirementProducer::DeclarationRead => AvailabilityRequirementProducerV1::DeclarationRead,
        RuntimeRequirementProducer::ImportRead => AvailabilityRequirementProducerV1::ImportRead,
        RuntimeRequirementProducer::CompilerGeneratedRead => AvailabilityRequirementProducerV1::CompilerGeneratedRead,
        RuntimeRequirementProducer::CallableOwner => AvailabilityRequirementProducerV1::CallableOwner,
        RuntimeRequirementProducer::PatternOwner => AvailabilityRequirementProducerV1::PatternOwner,
        RuntimeRequirementProducer::ConformanceTypeOwner => AvailabilityRequirementProducerV1::ConformanceTypeOwner,
        RuntimeRequirementProducer::ContextualReceiverRead => AvailabilityRequirementProducerV1::ContextualReceiverRead,
        RuntimeRequirementProducer::MetadataTypeOwner => AvailabilityRequirementProducerV1::MetadataTypeOwner,
        RuntimeRequirementProducer::StaticConstantRead => AvailabilityRequirementProducerV1::StaticConstantRead,
    }
}

/// Loads completed canonical products directly, with no source parsing or linking.
pub fn load_module_world(wire: &ModuleWorldV5, limits: &ValidationLimits) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    load_module_world_with_allocation_budget(wire, limits, &mut allocations)
}

/// Loads module authority using the caller's cumulative load transaction.
pub fn load_module_world_with_allocation_budget(
    wire: &ModuleWorldV5,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    let wire = &lower_closed_world(wire)?;
    if wire.portable_topology_fingerprint()? != wire.topology_fingerprint {
        return Err(invalid("portable module topology fingerprint mismatch"));
    }

    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let projects = allocations.arc(ProjectUniverse::new()).map_err(allocation_error)?;
    let world = load_module_world_inner(wire, limits, &mut allocations, projects)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(world)
}

/// Loads module authority while retaining the validated caller-owned project catalog.
pub fn load_module_world_with_project_catalog_and_allocation_budget(
    wire: &ModuleWorldV5,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
    projects: Arc<ProjectUniverse>,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    let wire = &lower_closed_world(wire)?;
    // The caller-owned project catalog does not weaken the wire integrity
    // contract enforced by the self-contained module-world import path.
    if wire.portable_topology_fingerprint()? != wire.topology_fingerprint {
        return Err(invalid("portable module topology fingerprint mismatch"));
    }
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let world = load_module_world_inner(wire, limits, &mut allocations, projects)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(world)
}

/// Loads one package's module authority against already admitted prefix
/// authority (rule `MW5-M2`).
///
/// Every `External { k }` reference is relocated into an extended reference
/// space (local rows, then the external table), and each external row is
/// resolved through `projects` to a native [`ModuleId`] that `prefix` must
/// already contain. No local row is created for a foreign module: the
/// returned world owns exactly the artifact's local modules, while its edges,
/// reads and exports may point into earlier generations.
pub fn load_module_world_with_prefix_authority(
    wire: &ModuleWorldV5,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
    projects: Arc<ProjectUniverse>,
    prefix: &dyn ExternalModuleAuthority,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    if wire.portable_topology_fingerprint()? != wire.topology_fingerprint {
        return Err(invalid("portable module topology fingerprint mismatch"));
    }
    let base = u32::try_from(wire.modules.len()).map_err(|_| invalid("module count exceeds the artifact reference space"))?;
    let lowered = wire
        .relocated_v4(Some(base))
        .map(|world| world.previous.world)
        .ok_or_else(|| invalid("module world reference is outside its local and external tables"))?;
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let mut external_ids = allocations.vector::<ModuleId>(wire.external_modules.len()).map_err(allocation_error)?;
    for external in &wire.external_modules {
        let id = resolve_external_module(external, &projects, limits, &mut allocations)?;
        if !prefix.contains(&id) {
            return Err(invalid(format!("external module {id} is absent from admitted prefix authority")));
        }
        external_ids.push(id);
    }
    let world = load_module_world_inner_with_prefix(&lowered, limits, &mut allocations, projects, &external_ids, Some(prefix))?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(world)
}

/// Resolves one stable external module row to its native identity.
fn resolve_external_module(
    external: &StableModuleRef,
    projects: &ProjectUniverse,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ModuleId, ArtifactError> {
    if matches!(external.project, StableProjectRef::Session { .. }) {
        return Err(invalid("session project identity in external module table is not permitted"));
    }
    let project = projects
        .resolve_stable_project(&external.project)
        .ok_or_else(|| invalid(format!("external module project has no exact workspace binding: {:?}", external.project)))?;
    if external.path.len() > limits.max_recursion_depth {
        return Err(invalid("external module path exceeds depth limit"));
    }
    let mut components = allocations.vector::<ModuleComponent>(external.path.len()).map_err(allocation_error)?;
    for part in external.path.iter() {
        ModuleComponent::validate_identifier(part).map_err(|e| invalid(e))?;
        let boxed = allocations.boxed_str(part).map_err(allocation_error)?;
        components.push(ModuleComponent::from_owned_identifier(boxed).map_err(|_| invalid("invalid identifier"))?);
    }
    let path = ModulePath::from_components(allocations.boxed_slice(components).map_err(allocation_error)?);
    Ok(ModuleId { project, path })
}

/// Lowers a module world to the closed schema-2 form this loader reads.
///
/// External module references are not yet resolved by this loader (rule
/// `MW5-M2`); they are rejected with a typed error instead of being dropped.
fn lower_closed_world(wire: &ModuleWorldV5) -> Result<ModuleWorldV2, ArtifactError> {
    wire.try_into_v4()
        .map(|world| world.previous.world)
        .ok_or_else(|| invalid("external module references are not supported by the module-world loader"))
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::Allocation(error)
}

fn authority_error(error: ModuleAuthorityBuildError) -> ArtifactError {
    match error {
        ModuleAuthorityBuildError::Allocation(error) => allocation_error(error),
        ModuleAuthorityBuildError::Entries(_) | ModuleAuthorityBuildError::SetEntries(_) => invalid("invalid module authority entries"),
    }
}

fn load_module_world_inner(
    wire: &ModuleWorldV2,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
    universe: Arc<ProjectUniverse>,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    load_module_world_inner_with_prefix(wire, limits, allocations, universe, &[], None)
}

/// Shared loader body. Indices below `wire.modules.len()` name local rows;
/// indices past it name `external_ids` (already proven members of `prefix`).
fn load_module_world_inner_with_prefix(
    wire: &ModuleWorldV2,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
    universe: Arc<ProjectUniverse>,
    external_ids: &[ModuleId],
    prefix: Option<&dyn ExternalModuleAuthority>,
) -> Result<CanonicalExternalModuleWorld, ArtifactError> {
    // Validate external projects
    for project in &wire.external_projects {
        if matches!(project, StableProjectRef::Session { .. }) {
            return Err(invalid("session project identity in external_projects is not permitted"));
        }
    }
    for adjacent in wire.external_projects.windows(2) {
        if adjacent[0] == adjacent[1] {
            return Err(invalid(format!("duplicate external project identity: {:?}", adjacent[0])));
        }
        if adjacent[0] > adjacent[1] {
            return Err(invalid("external projects are not in canonical sorted order"));
        }
    }

    if wire.modules.is_empty() || wire.modules.len() > limits.max_modules {
        return Err(invalid("invalid canonical module count"));
    }
    let mut ids = allocations.vector::<ModuleId>(wire.modules.len()).map_err(allocation_error)?;
    for module in &wire.modules {
        if matches!(module.identity.project, StableProjectRef::Session { .. }) {
            return Err(invalid("session project identity in module identity is not permitted"));
        }
        let project = universe
            .resolve_stable_project(&module.identity.project)
            .ok_or_else(|| invalid(format!("module project has no exact workspace binding: {:?}", module.identity.project)))?;
        if module.path.len() > limits.max_recursion_depth {
            return Err(invalid("module path exceeds depth limit"));
        }
        if module.path.len() != module.identity.path.len() || !module.path.iter().zip(module.identity.path.iter()).all(|(a, b)| a.as_str() == b.as_ref()) {
            return Err(invalid(format!(
                "module path does not match stable module identity: {:?} vs {:?}",
                module.path, module.identity.path
            )));
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
        let id = ModuleId { project, path };
        if ids.last().is_some_and(|previous| previous >= &id) {
            return Err(invalid("module paths must be unique and sorted"));
        }
        ids.push(id);
    }
    let ref_count = ids.len().checked_add(external_ids.len()).ok_or_else(|| allocation_error(AllocationError::Overflow))?;
    let mut ref_ids = allocations.vector::<ModuleId>(ref_count).map_err(allocation_error)?;
    for id in ids.iter().chain(external_ids) {
        ref_ids.push(id.clone_with_allocation_budget(allocations).map_err(allocation_error)?);
    }
    let entry = module_id(&ids, wire.entry, allocations)?;
    if !entry.path.is_root() {
        return Err(invalid("canonical artifact entry must be its project root module"));
    }
    let mut interfaces_entries = allocations
        .vector::<(ModuleId, Arc<UnlinkedModuleInterface>)>(wire.modules.len())
        .map_err(allocation_error)?;
    let mut modules_entries = allocations.vector::<(ModuleId, LinkedModule)>(wire.modules.len()).map_err(allocation_error)?;
    let mut ref_graph_entries = allocations
        .vector::<(ModuleId, Vec<ReferenceEdge>)>(wire.modules.len())
        .map_err(allocation_error)?;
    let mut runtime_graph_entries = allocations
        .vector::<(ModuleId, Vec<RuntimeDependencyEdge>)>(wire.modules.len())
        .map_err(allocation_error)?;
    let mut nodes_entries = allocations.vector::<(ModuleId, TopologyNode)>(wire.modules.len()).map_err(allocation_error)?;
    let mut children_entries = allocations
        .vector::<(ModuleId, ModuleAuthoritySet<ModuleId>)>(wire.modules.len())
        .map_err(allocation_error)?;
    let mut exposed_entries = allocations
        .vector::<(ModuleId, ModuleAuthoritySet<ModuleComponent>)>(wire.modules.len())
        .map_err(allocation_error)?;

    for (index, payload) in wire.modules.iter().enumerate() {
        let id = &ids[index];
        let kind = kind_in(&payload.kind);
        if id.path.is_root() && kind != ModuleKind::Package {
            return Err(invalid("project root must be a package"));
        }
        ordered_names(payload.declarations.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.exports.iter().map(|value| value.exported_name.as_str()))?;
        ordered_names(payload.linked_exports.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.globals.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.import_bindings.iter().map(|value| value.name.as_str()))?;
        ordered_names(payload.exposed_children.iter().map(String::as_str))?;

        let mut decl_entries = allocations
            .vector::<(String, DeclarationSurface)>(payload.declarations.len())
            .map_err(allocation_error)?;
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
                UnlinkedTargetV1::Local { name } => UnlinkedExportTarget::Local(allocations.string(name).map_err(allocation_error)?),
                UnlinkedTargetV1::ReExport { path, remote } => UnlinkedExportTarget::ReExport {
                    path: path_in(path, allocations)?,
                    remote: allocations.string(remote).map_err(allocation_error)?,
                },
                UnlinkedTargetV1::CanonicalDeclaration { module, name } => UnlinkedExportTarget::CanonicalDeclaration {
                    module: module_id(&ref_ids, *module, allocations)?,
                    name: allocations.string(name).map_err(allocation_error)?,
                },
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

        let mut exposed_vec = allocations
            .vector::<ModuleComponent>(payload.exposed_children.len())
            .map_err(allocation_error)?;
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
            declarations: declarations.into(),
            exports: exports.into(),
            imports,
            exposed_children: exposed_children.clone().into(),
            metadata: unlinked_metadata,
        };
        if interface.fingerprint().raw() != payload.interface_fingerprint {
            return Err(invalid("unlinked interface fingerprint mismatch"));
        }

        let mut linked_export_entries = allocations
            .vector::<(Box<str>, LinkedExport)>(payload.linked_exports.len())
            .map_err(allocation_error)?;
        for value in &payload.linked_exports {
            let target = match target_in(&value.target, &ref_ids, allocations)? {
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
            exports: linked_exports.into(),
            metadata: linked_metadata,
        };
        if linked_interface.fingerprint().raw() != payload.linked_interface_fingerprint {
            return Err(invalid("linked interface fingerprint mismatch"));
        }

        let mut globals_entries = allocations
            .vector::<(Box<str>, GlobalBindingId)>(payload.globals.len())
            .map_err(allocation_error)?;
        for value in &payload.globals {
            let name = allocations.boxed_str(&value.name).map_err(allocation_error)?;
            globals_entries.push((name, GlobalBindingId(value.index)));
        }
        let globals = ModuleAuthorityMap::from_budgeted_sorted_entries(globals_entries, allocations).map_err(authority_error)?;

        let mut import_binding_entries = allocations
            .vector::<(Box<str>, ImportBindingId)>(payload.import_bindings.len())
            .map_err(allocation_error)?;
        for value in &payload.import_bindings {
            let name = allocations.boxed_str(&value.name).map_err(allocation_error)?;
            import_binding_entries.push((name, ImportBindingId(value.index)));
        }
        let import_bindings = ModuleAuthorityMap::from_budgeted_sorted_entries(import_binding_entries, allocations).map_err(authority_error)?;

        let mut linked_reads = allocations.vector::<LinkedReadSpec>(payload.linked_reads.len()).map_err(allocation_error)?;
        for target in &payload.linked_reads {
            linked_reads.push(target_in(target, &ref_ids, allocations)?);
        }

        let mut runtime_dependencies = allocations.vector::<ModuleId>(payload.runtime_dependencies.len()).map_err(allocation_error)?;
        for value in &payload.runtime_dependencies {
            runtime_dependencies.push(module_id(&ref_ids, *value, allocations)?);
        }

        let linked_module = LinkedModule {
            interface: linked_interface,
            bindings: ModuleBindingLayout {
                local_globals: globals.into(),
                imports: import_bindings.into(),
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
                to: module_id(&ref_ids, edge.target, allocations)?,
                kind: reference_in(&edge.kind),
                range: range(&edge.span)?,
            });
        }
        ref_graph_entries.push((id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, ref_edges));

        let mut rt_edges = allocations
            .vector::<RuntimeDependencyEdge>(payload.runtime_edges.len())
            .map_err(allocation_error)?;
        for edge in &payload.runtime_edges {
            rt_edges.push(RuntimeDependencyEdge {
                importer: id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                dependency: module_id(&ref_ids, edge.dependency, allocations)?,
                reason: runtime_in(&edge.reason),
                range: range(&edge.span)?,
                provenance: None,
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
                        child.project == id.project
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
    let mut semantic_entries = allocations
        .vector::<(SemanticNodeId, Vec<SemanticEdge>)>(wire.semantic_rows.len())
        .map_err(allocation_error)?;
    for row in &wire.semantic_rows {
        let node = node_in(&row.node, &ref_ids, allocations)?;
        let mut edges = allocations.vector::<SemanticEdge>(row.edges.len()).map_err(allocation_error)?;
        for edge in &row.edges {
            edges.push(SemanticEdge {
                from: node.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                to: node_in(&edge.target, &ref_ids, allocations)?,
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
            validate_target(target, &interfaces, prefix)?;
        }
        for export in module.interface.exports.values() {
            match &export.target {
                LinkedExportTarget::Module(_) => {}
                LinkedExportTarget::Binding(symbol) => validate_binding_target(&symbol.module, &symbol.name, &interfaces, prefix)?,
            }
        }
        if let Some(iface) = interfaces.get(&module.interface.module) {
            for export in iface.exports.values() {
                if let UnlinkedExportTarget::CanonicalDeclaration { module, name } = &export.target {
                    validate_binding_target(module, name, &interfaces, prefix)?;
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
                // A prefix generation is initialized before this artifact.
                if external_ids.contains(dependency) {
                    continue;
                }
                let dep_idx = ids.binary_search(dependency).map_err(|_| invalid("runtime graph dependency is absent"))?;
                match positions[dep_idx] {
                    Some(dep_pos) if dep_pos < pos => {}
                    _ => return Err(invalid("initializer order violates runtime dependency")),
                }
            }
        }
    }

    // Ordering evidence above proves the eager subgraph; validate all modules too.
    validate_runtime_dag(&graphs.runtime, &ids, external_ids, allocations)?;
    let source_modules =
        ModuleAuthorityMap::from_budgeted_sorted_entries(allocations.vector(0).map_err(allocation_error)?, allocations).map_err(authority_error)?;
    let topology = ModuleTopology {
        generation: ResolverGeneration(0),
        fingerprint: ModuleTopology::fingerprint_for_parts(&universe, &nodes, &exposed),
        nodes,
        source_modules,
        children,
        exposed_children: exposed,
    };
    topology.validate_completed(&universe).map_err(invalid)?;
    let linked = allocations
        .arc(LinkedProgram {
            universe,
            modules: modules.into(),
            graphs,
            entry,
            initialization_order,
        })
        .map_err(allocation_error)?;
    let topology = allocations.arc(topology).map_err(allocation_error)?;
    CanonicalExternalModuleWorld::from_parts(linked, interfaces, topology).map_err(invalid)
}
/// Iterative Kahn validation avoids recursive SCC traversal on artifact input.
fn validate_runtime_dag(
    graph: &RuntimeDependencyGraph,
    ids: &[ModuleId],
    external_ids: &[ModuleId],
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    // Edges into prefix generations cannot close a cycle inside this world.
    let local_edges = |id: &ModuleId| graph.edges_from(id).iter().filter(|edge| !external_ids.contains(&edge.dependency)).count();
    let edge_count = ids
        .iter()
        .try_fold(0_usize, |count, id| count.checked_add(local_edges(id)).ok_or(AllocationError::Overflow))
        .map_err(allocation_error)?;
    let mut edges = allocations.vector::<(usize, usize)>(edge_count).map_err(allocation_error)?;
    let mut degrees = allocations.vector::<usize>(ids.len()).map_err(allocation_error)?;
    degrees.resize(ids.len(), 0);
    for (user, id) in ids.iter().enumerate() {
        for edge in graph.edges_from(id).iter().filter(|edge| !external_ids.contains(&edge.dependency)) {
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
        Some((_, expected_components)) => parent.is_some_and(|parent| parent.project == id.project && parent.path.components() == expected_components),
    }
}

fn validate_binding_target(
    module: &ModuleId,
    name: &str,
    interfaces: &ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    prefix: Option<&dyn ExternalModuleAuthority>,
) -> Result<(), ArtifactError> {
    let interface_binding = interfaces
        .get(module)
        .or_else(|| prefix.and_then(|prefix| prefix.interface(module)))
        .is_some_and(|interface| interface.declarations.contains_key(name) || interface.exports.contains_key(name));
    let catalog_binding = module.project.is_universe()
        && phalcom_native_meta::UNIVERSE_BINDINGS.iter().any(|binding| {
            binding.name == name
                && binding
                    .key
                    .source_path()
                    .iter()
                    .copied()
                    .eq(module.path.components().iter().map(|component| component.as_str()))
        });
    if !interface_binding && !catalog_binding {
        return Err(invalid(format!("linked binding has no owning declaration: {module:?}::{name}")));
    }
    Ok(())
}

fn validate_target(
    target: &LinkedReadSpec,
    interfaces: &ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    prefix: Option<&dyn ExternalModuleAuthority>,
) -> Result<(), ArtifactError> {
    if let LinkedReadSpec::Binding(symbol) = target {
        validate_binding_target(&symbol.module, &symbol.name, interfaces, prefix)?;
    }
    Ok(())
}

impl CanonicalExternalModuleWorld {
    pub fn encode_artifact(&self, limits: &ValidationLimits) -> Result<Vec<u8>, ArtifactError> {
        project_module_world(self, limits)?.encode()
    }
    pub fn decode_artifact(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        load_module_world(&ModuleWorldV5::from(ModuleWorldV2::decode(bytes, limits)?), limits)
    }
}

#[cfg(test)]
mod allocation_materialization_tests {
    use super::{load_module_world, validate_runtime_dag};
    use crate::graph::{RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason};
    use crate::identity::{ModuleComponent, ModuleId, ModulePath};
    use phalcom_common::range::SourceRange;
    use phalcom_compiler_artifact::module_world::{
        AliasV1, ArtifactModuleId, BindingV1, ImportPathV1, ImportRootV1, ImportV1, ModuleKindV1, ModuleV2, ModuleWorldV2, SegmentV1, SpanV1,
    };
    use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
    use phalcom_type_meta::allocation::AllocationBudget;
    use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};
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
            provenance: None,
        });
    }

    fn module_wire(path: Vec<String>) -> ModuleV2 {
        let boxed_path = path.iter().map(|s| s.as_str().into()).collect::<Vec<Box<str>>>().into_boxed_slice();
        ModuleV2 {
            identity: StableModuleRef {
                project: StableProjectRef::Builtin {
                    namespace: "universe".into(),
                    version: "0.1.0".into(),
                },
                path: boxed_path,
            },
            path,
            kind: ModuleKindV1::Package,
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            exposed_children: vec![],
            metadata: vec![],
            linked_exports: vec![],
            globals: vec![],
            import_bindings: vec![],
            linked_reads: vec![],
            runtime_dependencies: vec![],
            references: vec![],
            runtime_edges: vec![],
            topology_parent: None,
            topology_children: vec![],
            interface_fingerprint: 0,
            linked_interface_fingerprint: 0,
            dependency_fingerprint: 0,
        }
    }

    fn world(modules: Vec<ModuleV2>) -> ModuleWorldV2 {
        let mut world = ModuleWorldV2 {
            entry: ArtifactModuleId(0),
            initialization_order: vec![],
            modules,
            semantic_rows: vec![],
            external_projects: vec![],
            topology_fingerprint: 0,
        };
        world
            .refresh_portable_topology_fingerprint()
            .expect("test module world topology should fingerprint");
        world
    }

    fn assert_referential_rejection(world: &ModuleWorldV2, expected: &str) {
        assert!(matches!(
            load_module_world(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(world.clone()), &ValidationLimits::default()),
            Err(ArtifactError::Validation(ValidationError::ReferentialIntegrity(message))) if message == expected
        ));
    }

    #[test]
    fn module_world_rejects_invalid_and_duplicate_module_import_identities() {
        let invalid_module = world(vec![module_wire(vec!["bad-name".into()])]);
        assert_referential_rejection(&invalid_module, "invalid character in identifier");

        let duplicate_modules = world(vec![module_wire(vec![]), module_wire(vec![])]);
        assert_referential_rejection(&duplicate_modules, "module paths must be unique and sorted");

        let mut invalid_import = module_wire(vec![]);
        invalid_import.imports.push(ImportV1::Module {
            path: ImportPathV1 {
                root: ImportRootV1::Absolute {
                    segment: SegmentV1 {
                        name: "universe".into(),
                        span: SpanV1 { start: 0, end: 0 },
                    },
                },
                segments: vec![],
                span: SpanV1 { start: 0, end: 0 },
            },
            alias: Some(AliasV1 {
                name: String::new(),
                span: SpanV1 { start: 0, end: 0 },
            }),
            span: SpanV1 { start: 0, end: 0 },
        });
        assert_referential_rejection(&world(vec![invalid_import]), "empty import alias");

        let mut duplicate_import_bindings = module_wire(vec![]);
        duplicate_import_bindings.import_bindings = vec![BindingV1 { name: "item".into(), index: 0 }, BindingV1 { name: "item".into(), index: 1 }];
        assert_referential_rejection(&world(vec![duplicate_import_bindings]), "names must be nonempty, unique, and sorted");
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
        validate_runtime_dag(&graph, &ids, &[], &mut allocations).unwrap();
        let expected = Layout::array::<(usize, usize)>(3).unwrap().size()
            + Layout::array::<usize>(ids.len()).unwrap().size()
            + Layout::array::<usize>(ids.len() + 1).unwrap().size()
            + Layout::array::<usize>(ids.len()).unwrap().size();
        assert_eq!(allocations.requested(), expected);

        add_edge(&mut graph, &ids[2], &ids[0]);
        let mut cycle_allocations = AllocationBudget::new(1024);
        assert!(validate_runtime_dag(&graph, &ids, &[], &mut cycle_allocations).is_err());
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

#[cfg(test)]
mod package_entry_authority_tests {
    use super::*;
    use phalcom_type_meta::fingerprint::Fingerprint128;

    struct NoPackageReceipts;

    impl crate::TrustedPackageResolver for NoPackageReceipts {
        fn package_authority_receipt(&self, _: &StableProjectRef) -> Option<crate::ValidatedPackageAuthorityReceipt> {
            None
        }
    }

    fn wire_with_entry_owner(project: StableProjectRef) -> ModuleWorldV2 {
        ModuleWorldV2 {
            entry: ArtifactModuleId(0),
            initialization_order: vec![],
            modules: vec![ModuleV2 {
                identity: StableModuleRef { project, path: Box::new([]) },
                path: vec![],
                kind: ModuleKindV1::Package,
                declarations: vec![],
                exports: vec![],
                imports: vec![],
                exposed_children: vec![],
                metadata: vec![],
                linked_exports: vec![],
                globals: vec![],
                import_bindings: vec![],
                linked_reads: vec![],
                runtime_dependencies: vec![],
                references: vec![],
                runtime_edges: vec![],
                topology_parent: None,
                topology_children: vec![],
                interface_fingerprint: 0,
                linked_interface_fingerprint: 0,
                dependency_fingerprint: 0,
            }],
            semantic_rows: vec![],
            external_projects: vec![],
            topology_fingerprint: 0,
        }
    }

    #[test]
    fn artifact_header_cannot_claim_foreign_root_through_project_list() {
        let actual = StableProjectRef::Package {
            package: "actual".into(),
            version: "1.0.0".into(),
            artifact_fingerprint: Fingerprint128([1; 16]),
        };
        let mut wire = wire_with_entry_owner(actual);
        // A forged header's package identity is present in external_projects
        // but does not own the actual entry/root module.
        wire.external_projects.push(StableProjectRef::Package {
            package: "forged".into(),
            version: "1.0.0".into(),
            artifact_fingerprint: Fingerprint128([2; 16]),
        });
        let header = TeloscPackageIdentity::Package {
            package: "forged".into(),
            version: "1.0.0".into(),
        };
        let error = stage_artifact_project_authorities(
            &ProjectUniverse::new(),
            &phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(wire.clone()),
            &header,
            ArtifactDigest([3; 32]),
            &NoPackageReceipts,
        )
        .unwrap_err();
        assert!(error.to_string().contains("does not own its root entry"), "{error}");
    }

    #[test]
    fn package_artifact_cannot_embed_a_second_project_owner() {
        let owner = StableProjectRef::Package {
            package: "actual".into(),
            version: "1.0.0".into(),
            artifact_fingerprint: Fingerprint128([1; 16]),
        };
        let mut wire = wire_with_entry_owner(owner.clone());
        let mut foreign = wire.modules[0].clone();
        foreign.identity.project = StableProjectRef::Package {
            package: "dependency".into(),
            version: "1.0.0".into(),
            artifact_fingerprint: Fingerprint128([2; 16]),
        };
        wire.modules.push(foreign);
        let header = TeloscPackageIdentity::Package {
            package: "actual".into(),
            version: "1.0.0".into(),
        };
        let error = stage_artifact_project_authorities(
            &ProjectUniverse::new(),
            &phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(wire.clone()),
            &header,
            ArtifactDigest([3; 32]),
            &NoPackageReceipts,
        )
        .unwrap_err();
        assert!(error.to_string().contains("embeds a foreign module owner"), "{error}");
    }

    #[test]
    fn artifact_header_cannot_claim_foreign_builtin_root() {
        let wire = wire_with_entry_owner(StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        });
        let header = TeloscPackageIdentity::Package {
            package: "ordinary".into(),
            version: "1.0.0".into(),
        };
        let error = stage_artifact_project_authorities(
            &ProjectUniverse::new(),
            &phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(wire.clone()),
            &header,
            ArtifactDigest([4; 32]),
            &NoPackageReceipts,
        )
        .unwrap_err();
        assert!(error.to_string().contains("does not own its root entry"), "{error}");
    }

    #[test]
    fn artifact_header_requires_an_entry_module() {
        let mut wire = wire_with_entry_owner(StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        });
        wire.entry = ArtifactModuleId(1);
        let header = TeloscPackageIdentity::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        };
        let error = stage_artifact_project_authorities(
            &ProjectUniverse::new(),
            &phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(wire.clone()),
            &header,
            ArtifactDigest([5; 32]),
            &NoPackageReceipts,
        )
        .unwrap_err();
        assert!(error.to_string().contains("entry module is absent"), "{error}");
    }
}
