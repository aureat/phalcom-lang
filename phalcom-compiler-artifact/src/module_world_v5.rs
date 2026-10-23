//! Module-world schema 5: module references are tagged local or external.
//!
//! Schema 4 and earlier name every module through a dense [`ArtifactModuleId`]
//! into the artifact's own rows, so an artifact can only describe a closed
//! world. Schema 5 adds an `external_modules` table and lets exactly the
//! positions that may name a dependency use [`ModuleRefV1`]; positions that
//! must stay local (entry, initialization order, topology, semantic row owners,
//! witness importers, bootstrap schedule) keep [`ArtifactModuleId`] so they
//! cannot express "external" (rules `ARCH-I3`, `MW5-T1`).
//!
//! Older schemas decode and then lift losslessly into this form
//! ([`ModuleWorldV5::from`]); the Universe writer keeps emitting schema 4 via
//! [`ModuleWorldV5::try_into_v4`] (ruling `D-MW5-3`).

use crate::module_world::{
    ArtifactModuleId, AvailabilityAuthorityV1, AvailabilityProductV1, AvailabilityProofV1, AvailabilityRequirementProducerV1, AvailabilityTimingV1,
    BindingV1, Decode, DeclarationV1, ExportV1, ImportV1, LinkedExportV1, MODULE_WORLD_SCHEMA_V2, MODULE_WORLD_SCHEMA_V3, MODULE_WORLD_SCHEMA_V4,
    MetadataV1, ModuleKindV1, ModuleV2, ModuleWorldV2, ModuleWorldV3, ModuleWorldV4, PortableTopologyHasher, ReferenceKindV1, ReferenceV1, RuntimeAvailabilityWitnessV1,
    RuntimeEdgeV1, RuntimeReasonV1, SemanticEdgeKindV1, SemanticEdgeV1, SemanticNodeV1, SemanticRowV1, SpanV1, TargetV1, UnlinkedTargetV1, Wire,
    availability_authority_tag, availability_product_tag, availability_timing_tag, invalid, record, source_import_binding_count, tagged,
};
use crate::{ArtifactError, BinaryReader, BinaryWriter, CodecError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};

/// Section schema number of [`ModuleWorldV5`].
pub const MODULE_WORLD_SCHEMA_V5: u32 = 5;

/// Index into [`ModuleWorldV5::external_modules`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExternalModuleIndex(pub u32);

impl Wire for ExternalModuleIndex {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.0.write(out)
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(Self(u32::read(input)?))
    }
}

/// A reference to a module that is either owned by this artifact or external.
///
/// There is deliberately no conversion that collapses both arms into one
/// integer (rule `MW5-T1`).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModuleRefV1 {
    /// A module row of this artifact.
    Local {
        /// Dense index into [`ModuleWorldV5::modules`].
        module: ArtifactModuleId,
    },
    /// A module owned by a dependency artifact.
    External {
        /// Index into [`ModuleWorldV5::external_modules`].
        module: ExternalModuleIndex,
    },
}

impl ModuleRefV1 {
    /// Returns the local module id, if this reference is local.
    pub fn local(self) -> Option<ArtifactModuleId> {
        match self {
            Self::Local { module } => Some(module),
            Self::External { .. } => None,
        }
    }

    /// Maps this reference into an extended reference space whose external
    /// rows start at `base`; `None` rejects external references.
    pub fn relocated(self, base: Option<u32>) -> Option<ArtifactModuleId> {
        match self {
            Self::Local { module } => Some(module),
            Self::External { module } => base.and_then(|base| base.checked_add(module.0)).map(ArtifactModuleId),
        }
    }

    /// Returns the external table index, if this reference is external.
    pub fn external(self) -> Option<ExternalModuleIndex> {
        match self {
            Self::External { module } => Some(module),
            Self::Local { .. } => None,
        }
    }

    /// Ordering key: local references sort before external ones.
    fn key(self) -> (u8, u32) {
        match self {
            Self::Local { module } => (0, module.0),
            Self::External { module } => (1, module.0),
        }
    }
}

impl Wire for ModuleRefV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Local { module } => {
                out.write_u8(0);
                module.write(out)
            }
            Self::External { module } => {
                out.write_u8(1);
                module.write(out)
            }
        }
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| match input.reader.read_u8()? {
            0 => Ok(Self::Local {
                module: ArtifactModuleId::read(input)?,
            }),
            1 => Ok(Self::External {
                module: ExternalModuleIndex::read(input)?,
            }),
            _ => Err(CodecError::CorruptedData("invalid ModuleRefV1 tag".into()).into()),
        })
    }
}

tagged!(TargetV2 { 0 => Module { module: ModuleRefV1 }, 1 => Binding { module: ModuleRefV1, name: String } });
tagged!(UnlinkedTargetV2 {
    0 => Local { name: String },
    1 => ReExport { path: crate::module_world::ImportPathV1, remote: String },
    2 => CanonicalDeclaration { module: ModuleRefV1, name: String },
});
record!(ExportV2 {
    exported_name: String,
    internal_name: String,
    target: UnlinkedTargetV2,
    span: SpanV1
});
record!(LinkedExportV2 {
    name: String,
    target: TargetV2,
    span: SpanV1
});
record!(ReferenceV2 {
    target: ModuleRefV1,
    kind: ReferenceKindV1,
    span: SpanV1
});
record!(RuntimeEdgeV2 {
    dependency: ModuleRefV1,
    reason: RuntimeReasonV1,
    span: SpanV1
});
tagged!(SemanticNodeV2 { 0 => Module { module: ModuleRefV1 }, 1 => Declaration { module: ModuleRefV1, name: String } });
record!(SemanticEdgeV2 {
    target: SemanticNodeV2,
    kind: SemanticEdgeKindV1,
    span: SpanV1
});
// The row node is the module the row describes and therefore stays local.
record!(SemanticRowV2 { node: SemanticNodeV1, edges: Vec<SemanticEdgeV2> });
record!(ModuleV3 limit max_modules {
    identity: StableModuleRef,
    path: Vec<String>, kind: ModuleKindV1,
    declarations: Vec<DeclarationV1>, exports: Vec<ExportV2>, imports: Vec<ImportV1>,
    exposed_children: Vec<String>, metadata: Vec<MetadataV1>,
    linked_exports: Vec<LinkedExportV2>, globals: Vec<BindingV1>, import_bindings: Vec<BindingV1>,
    linked_reads: Vec<TargetV2>, runtime_dependencies: Vec<ModuleRefV1>,
    references: Vec<ReferenceV2>, runtime_edges: Vec<RuntimeEdgeV2>,
    topology_parent: Option<ArtifactModuleId>, topology_children: Vec<ArtifactModuleId>,
    interface_fingerprint: u64, linked_interface_fingerprint: u64, dependency_fingerprint: u64,
});

/// One exact availability proof applied to a finalized runtime requirement.
///
/// Identical to [`RuntimeAvailabilityWitnessV1`] except that the dependency
/// and targets may be external. The importer is always local.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAvailabilityWitnessV2 {
    /// Importer whose eager dependency was omitted.
    pub importer: ArtifactModuleId,
    /// Runtime dependency whose product is required.
    pub dependency: ModuleRefV1,
    /// Source location of the runtime requirement.
    pub range: SpanV1,
    /// Runtime reason that produced the requirement.
    pub reason: RuntimeReasonV1,
    /// Canonical read retained for the requirement, when it has one.
    pub read_target: Option<TargetV2>,
    /// Exact target of the availability proof.
    pub target: TargetV2,
    /// Product guaranteed by the proof.
    pub product: AvailabilityProductV1,
    /// Producer of the runtime requirement.
    pub producer: AvailabilityRequirementProducerV1,
    /// Stage at which the requirement needs the product.
    pub required_at: AvailabilityTimingV1,
    /// Authority that guarantees the product.
    pub authority: AvailabilityAuthorityV1,
    /// Latest stage at which the product is guaranteed to be available.
    pub available_at: AvailabilityTimingV1,
}

impl Wire for RuntimeAvailabilityWitnessV2 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.importer.write(out)?;
        self.dependency.write(out)?;
        self.range.write(out)?;
        self.reason.write(out)?;
        self.read_target.write(out)?;
        self.target.write(out)?;
        self.product.write(out)?;
        self.producer.write(out)?;
        self.required_at.write(out)?;
        self.authority.write(out)?;
        self.available_at.write(out)
    }

    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            Ok(Self {
                importer: ArtifactModuleId::read(input)?,
                dependency: ModuleRefV1::read(input)?,
                range: SpanV1::read(input)?,
                reason: RuntimeReasonV1::read(input)?,
                read_target: Option::<TargetV2>::read(input)?,
                target: TargetV2::read(input)?,
                product: AvailabilityProductV1::read(input)?,
                producer: AvailabilityRequirementProducerV1::read(input)?,
                required_at: AvailabilityTimingV1::read(input)?,
                authority: AvailabilityAuthorityV1::read(input)?,
                available_at: AvailabilityTimingV1::read(input)?,
            })
        })
    }
}

type TargetKey<'a> = (u8, (u8, u32), &'a str);

fn target_key(target: &TargetV2) -> TargetKey<'_> {
    match target {
        TargetV2::Module { module } => (0, module.key(), ""),
        TargetV2::Binding { module, name } => (1, module.key(), name.as_str()),
    }
}

impl RuntimeAvailabilityWitnessV2 {
    /// Orders complete witness records in canonical wire order.
    ///
    /// For a lifted V1 witness this agrees with
    /// [`RuntimeAvailabilityWitnessV1::canonical_cmp`].
    pub fn canonical_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }

    #[allow(clippy::type_complexity)]
    fn sort_key(&self) -> (u32, (u8, u32), u64, u64, u8, Option<TargetKey<'_>>, TargetKey<'_>, u8, u8, u8, u8, u8) {
        let reason = match &self.reason {
            RuntimeReasonV1::WholeModuleImport => 0,
            RuntimeReasonV1::SelectiveValueImport => 1,
            RuntimeReasonV1::ReExport => 2,
            RuntimeReasonV1::RuntimeDeclarationReference => 3,
        };
        let producer = match &self.producer {
            AvailabilityRequirementProducerV1::DeclarationRead => 0,
            AvailabilityRequirementProducerV1::ImportRead => 1,
            AvailabilityRequirementProducerV1::CompilerGeneratedRead => 2,
            AvailabilityRequirementProducerV1::CallableOwner => 3,
            AvailabilityRequirementProducerV1::PatternOwner => 4,
            AvailabilityRequirementProducerV1::ConformanceTypeOwner => 5,
            AvailabilityRequirementProducerV1::ContextualReceiverRead => 6,
            AvailabilityRequirementProducerV1::MetadataTypeOwner => 7,
            AvailabilityRequirementProducerV1::StaticConstantRead => 8,
        };
        (
            self.importer.0,
            self.dependency.key(),
            self.range.start,
            self.range.end,
            reason,
            self.read_target.as_ref().map(target_key),
            target_key(&self.target),
            availability_product_tag(&self.product),
            producer,
            availability_timing_tag(&self.required_at),
            availability_authority_tag(&self.authority),
            availability_timing_tag(&self.available_at),
        )
    }
}

/// Module world whose module references may name dependency modules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleWorldV5 {
    /// Entry module (local).
    pub entry: ArtifactModuleId,
    /// Initialization schedule over owned modules only.
    pub initialization_order: Vec<ArtifactModuleId>,
    /// Module rows owned by this artifact.
    pub modules: Vec<ModuleV3>,
    /// Semantic dependency rows, one per owned node.
    pub semantic_rows: Vec<SemanticRowV2>,
    /// Projects this artifact depends on, sorted and unique.
    pub external_projects: Vec<StableProjectRef>,
    /// Dependency modules referenced by this module world, strictly sorted.
    pub external_modules: Vec<StableModuleRef>,
    /// Portable fingerprint of the module topology.
    pub topology_fingerprint: u64,
    /// Exact proofs that justify omitted eager edges.
    pub availability_witnesses: Vec<RuntimeAvailabilityWitnessV2>,
    /// Canonical bootstrap schedule (Universe artifact only).
    pub bootstrap_modules: Vec<ArtifactModuleId>,
    /// Reusable environment contract (Universe artifact only).
    pub bootstrap_availability: Vec<AvailabilityProofV1>,
}

impl Wire for ModuleWorldV5 {
    fn collection_limit(limits: &ValidationLimits) -> usize {
        limits.max_modules
    }

    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.entry.write(out)?;
        self.initialization_order.write(out)?;
        self.modules.write(out)?;
        self.semantic_rows.write(out)?;
        self.external_projects.write(out)?;
        self.external_modules.write(out)?;
        self.topology_fingerprint.write(out)?;
        self.availability_witnesses.write(out)?;
        self.bootstrap_modules.write(out)?;
        self.bootstrap_availability.write(out)
    }

    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            Ok(Self {
                entry: ArtifactModuleId::read(input)?,
                initialization_order: Vec::<ArtifactModuleId>::read(input)?,
                modules: Vec::<ModuleV3>::read(input)?,
                semantic_rows: Vec::<SemanticRowV2>::read(input)?,
                external_projects: Vec::<StableProjectRef>::read(input)?,
                external_modules: Vec::<StableModuleRef>::read(input)?,
                topology_fingerprint: u64::read(input)?,
                availability_witnesses: Vec::<RuntimeAvailabilityWitnessV2>::read(input)?,
                bootstrap_modules: Vec::<ArtifactModuleId>::read(input)?,
                bootstrap_availability: Vec::<AvailabilityProofV1>::read(input)?,
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Lift (V4 -> V5) and lowering (V5 -> V4)
// ---------------------------------------------------------------------------

fn local(module: ArtifactModuleId) -> ModuleRefV1 {
    ModuleRefV1::Local { module }
}

fn lift_target(target: &TargetV1) -> TargetV2 {
    match target {
        TargetV1::Module { module } => TargetV2::Module { module: local(*module) },
        TargetV1::Binding { module, name } => TargetV2::Binding {
            module: local(*module),
            name: name.clone(),
        },
    }
}

fn lower_target(target: &TargetV2, base: Option<u32>) -> Option<TargetV1> {
    Some(match target {
        TargetV2::Module { module } => TargetV1::Module { module: module.relocated(base)? },
        TargetV2::Binding { module, name } => TargetV1::Binding {
            module: module.relocated(base)?,
            name: name.clone(),
        },
    })
}

fn lift_node(node: &SemanticNodeV1) -> SemanticNodeV2 {
    match node {
        SemanticNodeV1::Module { module } => SemanticNodeV2::Module { module: local(*module) },
        SemanticNodeV1::Declaration { module, name } => SemanticNodeV2::Declaration {
            module: local(*module),
            name: name.clone(),
        },
    }
}

fn lower_node(node: &SemanticNodeV2, base: Option<u32>) -> Option<SemanticNodeV1> {
    Some(match node {
        SemanticNodeV2::Module { module } => SemanticNodeV1::Module { module: module.relocated(base)? },
        SemanticNodeV2::Declaration { module, name } => SemanticNodeV1::Declaration {
            module: module.relocated(base)?,
            name: name.clone(),
        },
    })
}

fn lift_unlinked(target: &crate::module_world::UnlinkedTargetV1) -> UnlinkedTargetV2 {
    match target {
        UnlinkedTargetV1::Local { name } => UnlinkedTargetV2::Local { name: name.clone() },
        UnlinkedTargetV1::ReExport { path, remote } => UnlinkedTargetV2::ReExport {
            path: path.clone(),
            remote: remote.clone(),
        },
        UnlinkedTargetV1::CanonicalDeclaration { module, name } => UnlinkedTargetV2::CanonicalDeclaration {
            module: local(*module),
            name: name.clone(),
        },
    }
}

fn lower_unlinked(target: &UnlinkedTargetV2, base: Option<u32>) -> Option<UnlinkedTargetV1> {
    Some(match target {
        UnlinkedTargetV2::Local { name } => UnlinkedTargetV1::Local { name: name.clone() },
        UnlinkedTargetV2::ReExport { path, remote } => UnlinkedTargetV1::ReExport {
            path: path.clone(),
            remote: remote.clone(),
        },
        UnlinkedTargetV2::CanonicalDeclaration { module, name } => UnlinkedTargetV1::CanonicalDeclaration {
            module: module.relocated(base)?,
            name: name.clone(),
        },
    })
}

impl From<ModuleV2> for ModuleV3 {
    fn from(module: ModuleV2) -> Self {
        Self {
            identity: module.identity,
            path: module.path,
            kind: module.kind,
            declarations: module.declarations,
            exports: module
                .exports
                .into_iter()
                .map(|export| ExportV2 {
                    exported_name: export.exported_name,
                    internal_name: export.internal_name,
                    target: lift_unlinked(&export.target),
                    span: export.span,
                })
                .collect(),
            imports: module.imports,
            exposed_children: module.exposed_children,
            metadata: module.metadata,
            linked_exports: module
                .linked_exports
                .into_iter()
                .map(|export| LinkedExportV2 {
                    name: export.name,
                    target: lift_target(&export.target),
                    span: export.span,
                })
                .collect(),
            globals: module.globals,
            import_bindings: module.import_bindings,
            linked_reads: module.linked_reads.iter().map(lift_target).collect(),
            runtime_dependencies: module.runtime_dependencies.into_iter().map(local).collect(),
            references: module
                .references
                .into_iter()
                .map(|reference| ReferenceV2 {
                    target: local(reference.target),
                    kind: reference.kind,
                    span: reference.span,
                })
                .collect(),
            runtime_edges: module
                .runtime_edges
                .into_iter()
                .map(|edge| RuntimeEdgeV2 {
                    dependency: local(edge.dependency),
                    reason: edge.reason,
                    span: edge.span,
                })
                .collect(),
            topology_parent: module.topology_parent,
            topology_children: module.topology_children,
            interface_fingerprint: module.interface_fingerprint,
            linked_interface_fingerprint: module.linked_interface_fingerprint,
            dependency_fingerprint: module.dependency_fingerprint,
        }
    }
}

impl ModuleV3 {
    /// Lowers this row with external references relocated past `base`
    /// (see [`ModuleRefV1::relocated`]).
    fn relocated_v2(&self, base: Option<u32>) -> Option<ModuleV2> {
        Some(ModuleV2 {
            identity: self.identity.clone(),
            path: self.path.clone(),
            kind: self.kind.clone(),
            declarations: self.declarations.clone(),
            exports: self
                .exports
                .iter()
                .map(|export| {
                    Some(ExportV1 {
                        exported_name: export.exported_name.clone(),
                        internal_name: export.internal_name.clone(),
                        target: lower_unlinked(&export.target, base)?,
                        span: export.span.clone(),
                    })
                })
                .collect::<Option<_>>()?,
            imports: self.imports.clone(),
            exposed_children: self.exposed_children.clone(),
            metadata: self.metadata.clone(),
            linked_exports: self
                .linked_exports
                .iter()
                .map(|export| {
                    Some(LinkedExportV1 {
                        name: export.name.clone(),
                        target: lower_target(&export.target, base)?,
                        span: export.span.clone(),
                    })
                })
                .collect::<Option<_>>()?,
            globals: self.globals.clone(),
            import_bindings: self.import_bindings.clone(),
            linked_reads: self.linked_reads.iter().map(|target| lower_target(target, base)).collect::<Option<_>>()?,
            runtime_dependencies: self.runtime_dependencies.iter().map(|module| module.relocated(base)).collect::<Option<_>>()?,
            references: self
                .references
                .iter()
                .map(|reference| {
                    Some(ReferenceV1 {
                        target: reference.target.relocated(base)?,
                        kind: reference.kind.clone(),
                        span: reference.span.clone(),
                    })
                })
                .collect::<Option<_>>()?,
            runtime_edges: self
                .runtime_edges
                .iter()
                .map(|edge| {
                    Some(RuntimeEdgeV1 {
                        dependency: edge.dependency.relocated(base)?,
                        reason: edge.reason.clone(),
                        span: edge.span.clone(),
                    })
                })
                .collect::<Option<_>>()?,
            topology_parent: self.topology_parent,
            topology_children: self.topology_children.clone(),
            interface_fingerprint: self.interface_fingerprint,
            linked_interface_fingerprint: self.linked_interface_fingerprint,
            dependency_fingerprint: self.dependency_fingerprint,
        })
    }
}

impl From<ModuleWorldV4> for ModuleWorldV5 {
    /// Total, lossless lift: every reference becomes local and the external
    /// table is empty (rule `MW5-L1`).
    fn from(world: ModuleWorldV4) -> Self {
        let ModuleWorldV4 {
            previous,
            bootstrap_modules,
            bootstrap_availability,
        } = world;
        let ModuleWorldV3 {
            world,
            availability_witnesses,
        } = previous;
        let ModuleWorldV2 {
            entry,
            initialization_order,
            modules,
            semantic_rows,
            external_projects,
            topology_fingerprint,
        } = world;
        Self {
            entry,
            initialization_order,
            modules: modules.into_iter().map(ModuleV3::from).collect(),
            semantic_rows: semantic_rows
                .iter()
                .map(|row| SemanticRowV2 {
                    node: row.node.clone(),
                    edges: row
                        .edges
                        .iter()
                        .map(|edge| SemanticEdgeV2 {
                            target: lift_node(&edge.target),
                            kind: edge.kind.clone(),
                            span: edge.span.clone(),
                        })
                        .collect(),
                })
                .collect(),
            external_projects,
            external_modules: Vec::new(),
            topology_fingerprint,
            availability_witnesses: availability_witnesses
                .into_iter()
                .map(|witness| RuntimeAvailabilityWitnessV2 {
                    importer: witness.importer,
                    dependency: local(witness.dependency),
                    range: witness.range,
                    reason: witness.reason,
                    read_target: witness.read_target.as_ref().map(lift_target),
                    target: lift_target(&witness.target),
                    product: witness.product,
                    producer: witness.producer,
                    required_at: witness.required_at,
                    authority: witness.authority,
                    available_at: witness.available_at,
                })
                .collect(),
            bootstrap_modules,
            bootstrap_availability,
        }
    }
}

impl From<ModuleWorldV3> for ModuleWorldV5 {
    /// Lifts a schema-3 world (no bootstrap contract).
    fn from(world: ModuleWorldV3) -> Self {
        Self::from(ModuleWorldV4::new(world, Vec::new(), Vec::new()))
    }
}

impl From<ModuleWorldV2> for ModuleWorldV5 {
    /// Lifts a schema-2 world (no witnesses, no bootstrap contract).
    fn from(world: ModuleWorldV2) -> Self {
        Self::from(ModuleWorldV3::new(world, Vec::new()))
    }
}

impl ModuleWorldV5 {
    /// Lowers to schema 4. Returns `None` iff the world contains an external
    /// reference (rule `MW5-L1`). The Universe writer keeps emitting V4
    /// through this.
    pub fn try_into_v4(&self) -> Option<ModuleWorldV4> {
        self.relocated_v4(None)
    }

    /// Lowers this world into the closed schema-4 shape, relocating every
    /// external reference `External { k }` to `ArtifactModuleId(base + k)`.
    ///
    /// With `base = Some(modules.len())` the result addresses one extended
    /// reference space: local rows first, then this world's external table.
    /// No row is created for an external module; a loader must resolve the
    /// relocated indices against prior artifact authority. `None` keeps the
    /// closed-world contract and rejects any external reference.
    pub fn relocated_v4(&self, base: Option<u32>) -> Option<ModuleWorldV4> {
        let modules = self.modules.iter().map(|module| module.relocated_v2(base)).collect::<Option<Vec<_>>>()?;
        let semantic_rows = self
            .semantic_rows
            .iter()
            .map(|row| {
                Some(SemanticRowV1 {
                    node: row.node.clone(),
                    edges: row
                        .edges
                        .iter()
                        .map(|edge| {
                            Some(SemanticEdgeV1 {
                                target: lower_node(&edge.target, base)?,
                                kind: edge.kind.clone(),
                                span: edge.span.clone(),
                            })
                        })
                        .collect::<Option<_>>()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let availability_witnesses = self
            .availability_witnesses
            .iter()
            .map(|witness| {
                Some(RuntimeAvailabilityWitnessV1 {
                    importer: witness.importer,
                    dependency: witness.dependency.relocated(base)?,
                    range: witness.range.clone(),
                    reason: witness.reason.clone(),
                    read_target: match &witness.read_target {
                        Some(target) => Some(lower_target(target, base)?),
                        None => None,
                    },
                    target: lower_target(&witness.target, base)?,
                    product: witness.product.clone(),
                    producer: witness.producer.clone(),
                    required_at: witness.required_at.clone(),
                    authority: witness.authority.clone(),
                    available_at: witness.available_at.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(ModuleWorldV4 {
            previous: ModuleWorldV3 {
                world: ModuleWorldV2 {
                    entry: self.entry,
                    initialization_order: self.initialization_order.clone(),
                    modules,
                    semantic_rows,
                    external_projects: self.external_projects.clone(),
                    topology_fingerprint: self.topology_fingerprint,
                },
                availability_witnesses,
            },
            bootstrap_modules: self.bootstrap_modules.clone(),
            bootstrap_availability: self.bootstrap_availability.clone(),
        })
    }

    /// Resolves a module reference to its stable identity.
    pub fn module_identity(&self, module: ModuleRefV1) -> Option<&StableModuleRef> {
        match module {
            ModuleRefV1::Local { module } => self.modules.get(module.0 as usize).map(|row| &row.identity),
            ModuleRefV1::External { module } => self.external_modules.get(module.0 as usize),
        }
    }

    // -----------------------------------------------------------------------
    // Source import ordinals (same contract as `ModuleWorldV2`)
    // -----------------------------------------------------------------------

    /// Returns the module-relative source-binding ordinal for one import-site binding.
    pub fn source_import_binding_ordinal(&self, module: &StableModuleRef, site: u32, local: u32) -> Result<Option<u32>, ArtifactError> {
        let Some(row) = self.modules.iter().find(|row| row.identity == *module) else {
            return Ok(None);
        };
        let mut base = 0u32;
        for (site_index, import) in row.imports.iter().enumerate() {
            let site_index = u32::try_from(site_index).map_err(|_| invalid("source import site ordinal exceeds u32"))?;
            let count = source_import_binding_count(import)?;
            if site_index == site {
                return if local < count {
                    base.checked_add(local)
                        .map(Some)
                        .ok_or_else(|| invalid("source import binding ordinal exceeds u32"))
                } else {
                    Ok(None)
                };
            }
            base = base.checked_add(count).ok_or_else(|| invalid("source import binding ordinal exceeds u32"))?;
        }
        Ok(None)
    }

    /// Decodes a module-relative source-binding ordinal into its import-site row and site-local ordinal.
    pub fn source_import_binding_at(&self, module: &StableModuleRef, ordinal: u32) -> Result<Option<(u32, u32)>, ArtifactError> {
        let Some(row) = self.modules.iter().find(|row| row.identity == *module) else {
            return Ok(None);
        };
        let mut base = 0u32;
        for (site_index, import) in row.imports.iter().enumerate() {
            let site_index = u32::try_from(site_index).map_err(|_| invalid("source import site ordinal exceeds u32"))?;
            let count = source_import_binding_count(import)?;
            let end = base.checked_add(count).ok_or_else(|| invalid("source import binding ordinal exceeds u32"))?;
            if ordinal >= base && ordinal < end {
                return Ok(Some((site_index, ordinal - base)));
            }
            base = end;
        }
        Ok(None)
    }

    /// Counts the source bindings represented by a module's ordered import rows.
    pub fn source_import_binding_count(&self, module: &StableModuleRef) -> Result<Option<u32>, ArtifactError> {
        let Some(row) = self.modules.iter().find(|row| row.identity == *module) else {
            return Ok(None);
        };
        let count = row.imports.iter().try_fold(0u32, |count, import| {
            count
                .checked_add(source_import_binding_count(import)?)
                .ok_or_else(|| invalid("source import binding count exceeds u32"))
        })?;
        Ok(Some(count))
    }

    // -----------------------------------------------------------------------
    // Portable topology fingerprint (rules MW5-F1, MW5-F2)
    // -----------------------------------------------------------------------

    fn hash_module_ref(&self, out: &mut PortableTopologyHasher, module: ModuleRefV1) -> Result<(), ArtifactError> {
        let identity = self
            .module_identity(module)
            .ok_or_else(|| invalid("module link references missing module"))?;
        out.module_identity(identity)
    }

    fn hash_target(&self, out: &mut PortableTopologyHasher, target: &TargetV2) -> Result<(), ArtifactError> {
        match target {
            TargetV2::Module { module } => {
                out.u8(0);
                self.hash_module_ref(out, *module)
            }
            TargetV2::Binding { module, name } => {
                out.u8(1);
                self.hash_module_ref(out, *module)?;
                out.text(name)
            }
        }
    }

    fn hash_node(&self, out: &mut PortableTopologyHasher, node: &SemanticNodeV2) -> Result<(), ArtifactError> {
        match node {
            SemanticNodeV2::Module { module } => {
                out.u8(0);
                self.hash_module_ref(out, *module)
            }
            SemanticNodeV2::Declaration { module, name } => {
                out.u8(1);
                self.hash_module_ref(out, *module)?;
                out.text(name)
            }
        }
    }

    /// Recomputes the artifact-scoped topology fingerprint.
    ///
    /// Module references hash through their stable identity, with no
    /// local/external tag: ownership is disjoint (`MW5-V4`), so identities
    /// cannot collide. For a world with an empty external table this equals
    /// the V4 fingerprint of the source world (`MW5-F2`).
    pub fn portable_topology_fingerprint(&self) -> Result<u64, ArtifactError> {
        let mut out = PortableTopologyHasher::new();
        out.count(self.external_projects.len())?;
        for project in &self.external_projects {
            out.project(project)?;
        }

        out.count(self.modules.len())?;
        for module in &self.modules {
            out.module_identity(&module.identity)?;
            out.u8(match module.kind {
                ModuleKindV1::Module => 0,
                ModuleKindV1::Package => 1,
            });
            match module.topology_parent {
                None => out.u8(0),
                Some(parent) => {
                    out.u8(1);
                    self.hash_module_ref(&mut out, local(parent))?;
                }
            }
            out.count(module.topology_children.len())?;
            for child in &module.topology_children {
                self.hash_module_ref(&mut out, local(*child))?;
            }
            out.count(module.exposed_children.len())?;
            for child in &module.exposed_children {
                out.text(child)?;
            }

            out.count(module.exports.len())?;
            for export in &module.exports {
                match &export.target {
                    UnlinkedTargetV2::CanonicalDeclaration { module: target, name } => {
                        out.u8(1);
                        out.text(&export.exported_name)?;
                        self.hash_module_ref(&mut out, *target)?;
                        out.text(name)?;
                    }
                    UnlinkedTargetV2::Local { .. } | UnlinkedTargetV2::ReExport { .. } => out.u8(0),
                }
            }

            out.count(module.linked_exports.len())?;
            for export in &module.linked_exports {
                out.text(&export.name)?;
                self.hash_target(&mut out, &export.target)?;
            }
            out.count(module.linked_reads.len())?;
            for read in &module.linked_reads {
                self.hash_target(&mut out, read)?;
            }

            out.count(module.runtime_dependencies.len())?;
            for dependency in &module.runtime_dependencies {
                self.hash_module_ref(&mut out, *dependency)?;
            }
            out.count(module.references.len())?;
            for reference in &module.references {
                out.u8(match reference.kind {
                    ReferenceKindV1::WholeModuleImport => 0,
                    ReferenceKindV1::SelectiveImport => 1,
                    ReferenceKindV1::ReExport => 2,
                    ReferenceKindV1::InterfaceOnly => 3,
                });
                self.hash_module_ref(&mut out, reference.target)?;
            }
            out.count(module.runtime_edges.len())?;
            for edge in &module.runtime_edges {
                out.u8(match edge.reason {
                    RuntimeReasonV1::WholeModuleImport => 0,
                    RuntimeReasonV1::SelectiveValueImport => 1,
                    RuntimeReasonV1::ReExport => 2,
                    RuntimeReasonV1::RuntimeDeclarationReference => 3,
                });
                self.hash_module_ref(&mut out, edge.dependency)?;
            }
        }

        out.count(self.semantic_rows.len())?;
        for row in &self.semantic_rows {
            let node = lift_node(&row.node);
            self.hash_node(&mut out, &node)?;
            out.count(row.edges.len())?;
            for edge in &row.edges {
                out.u8(match edge.kind {
                    SemanticEdgeKindV1::ModuleInterface => 0,
                    SemanticEdgeKindV1::TypeReference => 1,
                    SemanticEdgeKindV1::Superclass => 2,
                    SemanticEdgeKindV1::ProtocolReference => 3,
                    SemanticEdgeKindV1::ConstraintReference => 4,
                    SemanticEdgeKindV1::CallbackSignature => 5,
                    SemanticEdgeKindV1::AdtReference => 6,
                });
                self.hash_node(&mut out, &edge.target)?;
            }
        }

        Ok(out.finish())
    }

    /// Updates the stored fingerprint after constructing or modifying a wire image.
    pub fn refresh_portable_topology_fingerprint(&mut self) -> Result<(), ArtifactError> {
        self.topology_fingerprint = self.portable_topology_fingerprint()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Codec
    // -----------------------------------------------------------------------

    /// Serializes the schema-5 wire image.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(MODULE_WORLD_SCHEMA_V5);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes and validates schema 5 under a cumulative allocation budget.
    pub fn decode_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| ArtifactError::Allocation(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let world = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut section,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            let schema = input.reader.read_u32()?;
            if schema != MODULE_WORLD_SCHEMA_V5 {
                return Err(crate::error::ValidationError::IncompatibleAbi {
                    kind: "module_world_schema",
                    found: schema,
                    expected: MODULE_WORLD_SCHEMA_V5,
                }
                .into());
            }
            let world = Self::read(&mut input)?;
            if input.reader.has_remaining() {
                return Err(invalid("trailing module-world payload bytes"));
            }
            world
        };
        world.validate_with_allocation_budget(limits, &mut section)?;
        transaction.commit_child(section).map_err(ArtifactError::Allocation)?;
        Ok(world)
    }

    /// Decodes schema 5 against a plain remaining-byte budget.
    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| ArtifactError::Allocation(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let world = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| ArtifactError::Allocation(AllocationError::Overflow))?;
        Ok(world)
    }

    /// Decodes schema 5 with the default section budget.
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }

    /// Decodes any accepted module-world schema (2 through 5) and returns the
    /// schema-5 form, lifting older schemas (rule `MW5-L2`).
    pub fn decode_any_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let schema = BinaryReader::new(bytes).read_u32()?;
        match schema {
            MODULE_WORLD_SCHEMA_V5 => Self::decode_with_allocation_budget(bytes, limits, transaction),
            MODULE_WORLD_SCHEMA_V4 => ModuleWorldV4::decode_with_allocation_budget(bytes, limits, transaction).map(Self::from),
            MODULE_WORLD_SCHEMA_V3 => {
                let v3 = ModuleWorldV3::decode_with_allocation_budget(bytes, limits, transaction)?;
                Ok(Self::from(ModuleWorldV4::new(v3, Vec::new(), Vec::new())))
            }
            MODULE_WORLD_SCHEMA_V2 => {
                let v2 = ModuleWorldV2::decode_with_allocation_budget(bytes, limits, transaction)?;
                Ok(Self::from(ModuleWorldV4::new(ModuleWorldV3::new(v2, Vec::new()), Vec::new(), Vec::new())))
            }
            found => Err(crate::error::ValidationError::IncompatibleAbi {
                kind: "module_world_schema",
                found,
                expected: MODULE_WORLD_SCHEMA_V5,
            }
            .into()),
        }
    }

    // -----------------------------------------------------------------------
    // Validation
    // -----------------------------------------------------------------------

    /// Checks one module reference is in range and marks external uses.
    fn check_ref(&self, module: ModuleRefV1, what: &str, used: &mut [bool]) -> Result<(), ArtifactError> {
        match module {
            ModuleRefV1::Local { module } => {
                if (module.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!(
                        "{what} references local module {} out of bounds ({} modules)",
                        module.0,
                        self.modules.len()
                    )));
                }
            }
            ModuleRefV1::External { module } => match used.get_mut(module.0 as usize) {
                Some(slot) => *slot = true,
                None => {
                    return Err(invalid(format!(
                        "{what}: external module reference {} out of bounds ({} entries)",
                        module.0,
                        self.external_modules.len()
                    )));
                }
            },
        }
        Ok(())
    }

    fn check_target(&self, target: &TargetV2, what: &str, used: &mut [bool]) -> Result<(), ArtifactError> {
        match target {
            TargetV2::Module { module } | TargetV2::Binding { module, .. } => self.check_ref(*module, what, used),
        }
    }

    fn check_local(&self, module: ArtifactModuleId, what: &str) -> Result<(), ArtifactError> {
        if (module.0 as usize) >= self.modules.len() {
            return Err(invalid(format!(
                "{what} references local module {} out of bounds ({} modules)",
                module.0,
                self.modules.len()
            )));
        }
        Ok(())
    }

    /// Validates the world on its own: every local rule of schemas 2–4 applied
    /// to the new field types, plus `MW5-V1` through `MW5-V10`.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        let mut allocations = AllocationBudget::new(usize::MAX);
        self.validate_with_allocation_budget(limits, &mut allocations)
    }

    /// Validates the world exactly like [`Self::validate`], charging the
    /// external-module usage bitmap against `allocations` so decode-time
    /// validation stays inside the section's load ledger.
    fn validate_with_allocation_budget(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        if self.modules.is_empty() {
            return Err(invalid("module world must contain at least one module"));
        }
        if self.modules.len() > limits.max_modules {
            return Err(invalid("module count exceeds limit"));
        }
        self.check_local(self.entry, "entry module")?;
        for (i, init) in self.initialization_order.iter().enumerate() {
            self.check_local(*init, &format!("initialization order {i}"))?;
        }

        for adjacent in self.external_projects.windows(2) {
            if adjacent[0] == adjacent[1] {
                return Err(invalid(format!("duplicate external project identity: {:?}", adjacent[0])));
            }
            if adjacent[0] > adjacent[1] {
                return Err(invalid("external projects are not in canonical sorted order"));
            }
        }
        for project in &self.external_projects {
            crate::identity::validate_persistent_project_ref(project, limits)?;
        }

        // MW5-V1: strictly increasing external table.
        for adjacent in self.external_modules.windows(2) {
            if adjacent[0] >= adjacent[1] {
                return Err(invalid("external modules are not strictly sorted (MW5-V1)"));
            }
        }
        for external in &self.external_modules {
            // MW5-V2: no session project (persistent check rejects it).
            crate::identity::validate_persistent_project_ref(&external.project, limits)?;
            // MW5-V3: dependency project is declared.
            if !self.external_projects.contains(&external.project) {
                return Err(invalid(format!("external module {external:?} belongs to an undeclared project (MW5-V3)")));
            }
            // MW5-V4: disjoint ownership.
            if self.modules.iter().any(|row| row.identity.project == external.project) {
                return Err(invalid(format!("external module {external:?} is in a project owned by this artifact (MW5-V4)")));
            }
        }

        let mut used = allocations.vector::<bool>(self.external_modules.len()).map_err(ArtifactError::Allocation)?;
        used.resize(self.external_modules.len(), false);

        for (mod_idx, module) in self.modules.iter().enumerate() {
            crate::identity::validate_persistent_project_ref(&module.identity.project, limits)?;
            if module.path.len() != module.identity.path.len() || !module.path.iter().zip(module.identity.path.iter()).all(|(a, b)| a.as_str() == b.as_ref()) {
                return Err(invalid(format!(
                    "module {mod_idx} path does not match stable module identity: {:?} vs {:?}",
                    module.path, module.identity.path
                )));
            }
            if module.path.is_empty() && mod_idx != (self.entry.0 as usize) {
                return Err(invalid(format!("module {mod_idx} has empty path")));
            }
            if module.path.iter().any(|segment| segment.is_empty()) {
                return Err(invalid("empty segment in module path"));
            }
            for other in &self.modules[(mod_idx + 1)..] {
                if module.identity == other.identity {
                    return Err(invalid(format!("duplicate module identity for {:?}", module.identity)));
                }
            }

            for (exp_idx, export) in module.exports.iter().enumerate() {
                for other in &module.exports[(exp_idx + 1)..] {
                    if export.exported_name == other.exported_name {
                        return Err(invalid(format!("module {mod_idx} contains duplicate export {:?}", export.exported_name)));
                    }
                }
                if let UnlinkedTargetV2::CanonicalDeclaration { module: target, .. } = &export.target {
                    self.check_ref(*target, &format!("export in module {mod_idx}"), &mut used)?;
                }
            }
            for export in &module.linked_exports {
                self.check_target(&export.target, &format!("linked export in module {mod_idx}"), &mut used)?;
            }
            for read in &module.linked_reads {
                self.check_target(read, &format!("linked read in module {mod_idx}"), &mut used)?;
            }
            for reference in &module.references {
                self.check_ref(reference.target, &format!("reference in module {mod_idx}"), &mut used)?;
            }
            for edge in &module.runtime_edges {
                self.check_ref(edge.dependency, &format!("runtime edge in module {mod_idx}"), &mut used)?;
            }
            for dependency in &module.runtime_dependencies {
                self.check_ref(*dependency, &format!("runtime dependency in module {mod_idx}"), &mut used)?;
            }
            // MW5-V8: dependencies and edges agree across both arms.
            let dependencies_match = module
                .runtime_dependencies
                .iter()
                .all(|dependency| module.runtime_edges.iter().any(|edge| edge.dependency == *dependency))
                && module
                    .runtime_edges
                    .iter()
                    .all(|edge| module.runtime_dependencies.contains(&edge.dependency));
            if !dependencies_match {
                return Err(invalid(format!("runtime dependencies and edges disagree for module {mod_idx} (MW5-V8)")));
            }

            // MW5-V6: local-only positions.
            if let Some(parent) = module.topology_parent {
                self.check_local(parent, &format!("topology parent of module {mod_idx}"))?;
                if parent.0 as usize == mod_idx {
                    return Err(invalid(format!("module {mod_idx} has impossible self-loop as topology parent")));
                }
            }
            for child in &module.topology_children {
                self.check_local(*child, &format!("topology child of module {mod_idx}"))?;
                if child.0 as usize == mod_idx {
                    return Err(invalid(format!("module {mod_idx} has impossible self-loop as topology child")));
                }
            }
        }

        for row in &self.semantic_rows {
            self.check_local(
                match &row.node {
                    SemanticNodeV1::Module { module } | SemanticNodeV1::Declaration { module, .. } => *module,
                },
                "semantic row node",
            )?;
            for edge in &row.edges {
                match &edge.target {
                    SemanticNodeV2::Module { module } | SemanticNodeV2::Declaration { module, .. } => {
                        self.check_ref(*module, "semantic edge target", &mut used)?;
                    }
                }
            }
        }

        self.validate_witnesses(limits, &mut used)?;

        // MW5-V5: minimal external table.
        if let Some(unused) = used.iter().position(|used| !used) {
            return Err(invalid(format!(
                "external module {unused} is not referenced by this module world (MW5-V5)"
            )));
        }

        self.validate_bootstrap(limits)?;

        if self.portable_topology_fingerprint()? != self.topology_fingerprint {
            return Err(invalid("portable module topology fingerprint mismatch"));
        }
        Ok(())
    }

    fn validate_witnesses(&self, limits: &ValidationLimits, used: &mut [bool]) -> Result<(), ArtifactError> {
        if self.availability_witnesses.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("availability witness count exceeds limit"));
        }
        if self
            .availability_witnesses
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err(invalid("availability witnesses are not uniquely sorted"));
        }

        // MW5-V9
        for witness in &self.availability_witnesses {
            self.check_local(witness.importer, "availability witness importer")?;
            let importer = &self.modules[witness.importer.0 as usize];
            self.check_ref(witness.dependency, "availability witness dependency", used)?;
            self.check_target(&witness.target, "availability witness target", used)?;
            if let Some(read) = &witness.read_target {
                self.check_target(read, "availability witness read target", used)?;
            }
            let target_module = match &witness.target {
                TargetV2::Module { module } | TargetV2::Binding { module, .. } => *module,
            };
            if target_module != witness.dependency {
                return Err(invalid("availability witness target and dependency disagree"));
            }
            if let Some(read_target) = &witness.read_target {
                if read_target != &witness.target || !importer.linked_reads.contains(read_target) {
                    return Err(invalid("availability witness does not match a finalized runtime read"));
                }
            }
            let available = availability_timing_tag(&witness.available_at) <= availability_timing_tag(&witness.required_at)
                || (witness.authority == AvailabilityAuthorityV1::FullyInitializedUniverse
                    && witness.required_at == AvailabilityTimingV1::DuringModuleInitializer);
            if !available {
                return Err(invalid("availability witness is later than its runtime requirement"));
            }
            if witness.dependency.external().is_some()
                && !matches!(
                    witness.authority,
                    AvailabilityAuthorityV1::SemanticIdentityCatalog
                        | AvailabilityAuthorityV1::NativeBindingCatalog
                        | AvailabilityAuthorityV1::NativeModuleMaterialization
                        | AvailabilityAuthorityV1::FullyInitializedUniverse
                )
            {
                return Err(invalid(
                    "external availability witness uses an authority an external artifact cannot supply (MW5-V9)",
                ));
            }
        }

        // MW5-V10: every linked read is an eager edge or exactly witnessed.
        for (module_index, module) in self.modules.iter().enumerate() {
            for read in &module.linked_reads {
                let dependency = match read {
                    TargetV2::Module { module } | TargetV2::Binding { module, .. } => *module,
                };
                let importer = u32::try_from(module_index).map_err(|_| invalid("module index exceeds u32"))?;
                if !module.runtime_dependencies.contains(&dependency)
                    && !self.availability_witnesses.iter().any(|witness| {
                        witness.importer.0 == importer
                            && witness.dependency == dependency
                            && witness.read_target.as_ref() == Some(read)
                            && witness.target == *read
                    })
                {
                    return Err(invalid(format!(
                        "runtime read in module {module_index} has neither an eager edge nor an exact availability witness (MW5-V10)"
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_bootstrap(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        // MW5-V7
        if !self.external_modules.is_empty() && (!self.bootstrap_modules.is_empty() || !self.bootstrap_availability.is_empty()) {
            return Err(invalid(
                "a world with external modules must not carry a bootstrap contract (MW5-V7)",
            ));
        }
        if self.bootstrap_modules.len() > limits.max_modules {
            return Err(invalid("bootstrap module count exceeds limit"));
        }
        if self.bootstrap_availability.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("bootstrap availability proof count exceeds limit"));
        }
        let is_universe = |identity: &StableModuleRef| matches!(&identity.project, StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe");
        for (index, module) in self.bootstrap_modules.iter().enumerate() {
            let row = self
                .modules
                .get(module.0 as usize)
                .ok_or_else(|| invalid("bootstrap schedule references an out-of-bounds module"))?;
            if self.bootstrap_modules[..index].contains(module) {
                return Err(invalid("bootstrap schedule contains a duplicate module"));
            }
            if !is_universe(&row.identity) {
                return Err(invalid("bootstrap schedule contains a non-Universe module"));
            }
        }
        if self
            .bootstrap_availability
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err(invalid("bootstrap availability proofs are not uniquely sorted"));
        }
        for proof in &self.bootstrap_availability {
            let module = match &proof.target {
                TargetV1::Module { module } => *module,
                TargetV1::Binding { module, name } => {
                    if name.is_empty() {
                        return Err(invalid("bootstrap availability binding name is empty"));
                    }
                    *module
                }
            };
            let row = self
                .modules
                .get(module.0 as usize)
                .ok_or_else(|| invalid("bootstrap availability proof references an out-of-bounds module"))?;
            if !is_universe(&row.identity) {
                return Err(invalid("bootstrap availability proof targets a non-Universe module"));
            }
            use AvailabilityAuthorityV1 as Authority;
            use AvailabilityProductV1 as Product;
            use AvailabilityTimingV1 as Timing;
            let valid = matches!(
                (&proof.authority, &proof.product, &proof.available_at, &proof.target),
                (Authority::SemanticIdentityCatalog, Product::CanonicalIdentity, Timing::BeforeSourceExecution, TargetV1::Module { .. })
                    | (Authority::SemanticIdentityCatalog, Product::CanonicalIdentity, Timing::BeforeSourceExecution, TargetV1::Binding { .. })
                    | (Authority::NativeModuleMaterialization, Product::RuntimeObject, Timing::BeforeSourceExecution, TargetV1::Module { .. })
                    | (Authority::NativeBindingCatalog, Product::RuntimeObject, Timing::BeforeSourceExecution, TargetV1::Binding { .. })
                    | (Authority::FullyInitializedUniverse, Product::RuntimeObject, Timing::AfterBootstrap, TargetV1::Module { .. })
                    | (Authority::FullyInitializedUniverse, Product::SourceInitializedValue, Timing::AfterBootstrap, TargetV1::Module { .. })
                    | (Authority::FullyInitializedUniverse, Product::SourceInitializedValue, Timing::AfterBootstrap, TargetV1::Binding { .. })
                    | (Authority::FullyInitializedUniverse, Product::MetadataDescriptor, Timing::AfterBootstrap, TargetV1::Module { .. })
            );
            if !valid {
                return Err(invalid("bootstrap availability proof uses an unsupported authority/product/timing/target combination"));
            }
            let requires_bootstrap_membership =
                proof.authority == Authority::FullyInitializedUniverse && matches!(&proof.product, Product::RuntimeObject | Product::SourceInitializedValue);
            if requires_bootstrap_membership && !self.bootstrap_modules.contains(&module) {
                return Err(invalid("fully initialized Universe runtime/source proof targets a module outside the bootstrap schedule"));
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Execution authority (port of `ModuleWorldV3::validate_execution_authority`)
    // -----------------------------------------------------------------------

    /// Validates availability witnesses against the semantic runtime
    /// requirements and import phases.
    pub fn validate_execution_authority(&self, authority: &crate::semantic::world::ModuleExecutionAuthorityV1) -> Result<(), ArtifactError> {
        for requirement in &authority.runtime_requirements {
            let importer = self
                .modules
                .iter()
                .position(|module| module.identity == requirement.importer)
                .ok_or_else(|| invalid("runtime requirement importer is absent from module world"))?;
            let dependency = self
                .module_ref_of(&requirement.dependency)
                .ok_or_else(|| invalid("runtime requirement dependency is absent from module world"))?;
            let importer_id = ArtifactModuleId(u32::try_from(importer).map_err(|_| invalid("module index exceeds u32"))?);
            if !self.modules[importer].runtime_dependencies.contains(&dependency)
                && !self
                    .availability_witnesses
                    .iter()
                    .any(|witness| self.witness_matches_requirement(witness, requirement))
            {
                return Err(invalid("runtime requirement has neither an eager edge nor an exact availability witness"));
            }
            if requirement.provenance.consumer != requirement.importer || importer_id.0 as usize != importer {
                return Err(invalid("runtime requirement ownership is inconsistent with module world"));
            }
        }

        for witness in &self.availability_witnesses {
            let matches_requirement = authority
                .runtime_requirements
                .iter()
                .any(|requirement| self.witness_matches_requirement(witness, requirement));
            if !matches_requirement && !self.is_source_import_witness(witness, authority) {
                return Err(invalid(format!(
                    "availability witness has no matching runtime requirement or runtime source import: {witness:?}"
                )));
            }
        }
        Ok(())
    }

    /// Resolves a stable identity to the local or external reference naming it.
    pub fn module_ref_of(&self, identity: &StableModuleRef) -> Option<ModuleRefV1> {
        if let Some(index) = self.modules.iter().position(|module| module.identity == *identity) {
            return Some(local(ArtifactModuleId(u32::try_from(index).ok()?)));
        }
        let index = self.external_modules.binary_search(identity).ok()?;
        Some(ModuleRefV1::External {
            module: ExternalModuleIndex(u32::try_from(index).ok()?),
        })
    }

    fn witness_matches_requirement(&self, witness: &RuntimeAvailabilityWitnessV2, requirement: &crate::semantic::world::RuntimeDependencyRequirementV1) -> bool {
        use crate::module_world::{availability_producer_matches, availability_product_matches, availability_timing_matches, runtime_reason_matches};
        let Some(importer) = self.modules.get(witness.importer.0 as usize) else {
            return false;
        };
        let Some(dependency) = self.module_identity(witness.dependency) else {
            return false;
        };
        if importer.identity != requirement.importer
            || *dependency != requirement.dependency
            || witness.range != requirement.range
            || !runtime_reason_matches(&witness.reason, &requirement.reason)
            || !availability_producer_matches(&witness.producer, &requirement.provenance.producer)
            || !availability_product_matches(&witness.product, &requirement.provenance.product)
            || !availability_timing_matches(&witness.required_at, &requirement.provenance.timing)
        {
            return false;
        }
        let Some(target) = self.stable_requirement_target(&witness.target) else {
            return false;
        };
        if target != requirement.provenance.target {
            return false;
        }
        let read_target = match &witness.read_target {
            Some(target) => match self.stable_linked_read(target) {
                Some(target) => Some(target),
                None => return false,
            },
            None => None,
        };
        match (&read_target, &requirement.read_target) {
            (Some(left), Some(right)) if left == right => {}
            (None, None) => {}
            _ => return false,
        }
        true
    }

    fn stable_requirement_target(&self, target: &TargetV2) -> Option<crate::semantic::world::RuntimeRequirementTargetV1> {
        use crate::semantic::world::RuntimeRequirementTargetV1 as Target;
        match target {
            TargetV2::Module { module } => Some(Target::Module {
                module: self.module_identity(*module)?.clone(),
            }),
            TargetV2::Binding { module, name } => Some(Target::Binding {
                module: self.module_identity(*module)?.clone(),
                name: name.clone().into_boxed_str(),
            }),
        }
    }

    fn stable_linked_read(&self, target: &TargetV2) -> Option<crate::semantic::world::LinkedReadSpecV1> {
        use crate::semantic::world::LinkedReadSpecV1 as Read;
        match target {
            TargetV2::Module { module } => Some(Read::Module {
                module: self.module_identity(*module)?.clone(),
            }),
            TargetV2::Binding { module, name } => Some(Read::Binding {
                symbol: StableDeclarationRef {
                    module: self.module_identity(*module)?.clone(),
                    path: vec![name.as_str().into()].into_boxed_slice(),
                },
            }),
        }
    }

    fn is_source_import_witness(&self, witness: &RuntimeAvailabilityWitnessV2, authority: &crate::semantic::world::ModuleExecutionAuthorityV1) -> bool {
        if witness.producer != AvailabilityRequirementProducerV1::ImportRead {
            return false;
        }
        let Some(read_target) = witness.read_target.as_ref() else {
            return false;
        };
        let Some(importer) = self.modules.get(witness.importer.0 as usize) else {
            return false;
        };
        let Some(phase_product) = authority.import_phase_products.iter().find(|product| product.module == importer.identity) else {
            return false;
        };
        let mut ordinal = 0u32;
        for import in &importer.imports {
            let local_names = crate::module_world::source_import_local_names(import);
            for local_name in &local_names {
                let current = ordinal;
                ordinal = match ordinal.checked_add(1) {
                    Some(next) => next,
                    None => return false,
                };
                let Some(binding) = importer.import_bindings.iter().find(|binding| binding.name == local_name.as_str()) else {
                    continue;
                };
                if importer.linked_reads.get(binding.index as usize) != Some(read_target) {
                    continue;
                }
                let phase_runtime = phase_product.bindings.iter().find(|binding| binding.binding == current).is_some_and(|binding| {
                    phase_product.coverage == crate::semantic::world::PhaseEvidenceCoverageV1::Incomplete
                        || !binding.summary.observed
                        || binding.summary.phase == crate::semantic::world::DependencyPhaseV1::Runtime
                });
                let force_runtime = matches!(import, ImportV1::Module { .. } | ImportV1::ReExport { .. });
                if force_runtime || phase_runtime {
                    return true;
                }
            }
        }
        false
    }
}
