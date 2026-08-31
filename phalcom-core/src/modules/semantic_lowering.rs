//! Semantic-to-Codegen Lowering Projection (Part 4).
//!
//! Bridges the formal `SemanticSnapshot` products to compact, immutable,
//! backend-facing lowering specifications attached by `LoweringSite` keys.

use phalcom_common::range::SourceRange;
use phalcom_modules::{DeclarationId, ModuleId, SourceId};
use phalcom_semantic::associated::AssociatedMemberId;
use phalcom_semantic::checker::associated::{
    AssociatedResolution, AssociatedResolutionKind, BehavioralFamilySpec, FamilyApplicationResolution, FamilyApplicationSelection,
};
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{CallableId, ExpressionId, InvocationTargetId, VariantFieldId, VariantId};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::types::denotation::{AssociatedValueDenotation, SemanticDenotation};
use phalcom_semantic::types::family::{FamilyMemberTypeKind, FamilyOperationShape};
use phalcom_semantic::types::store::TypeData;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use thiserror::Error;

/// Classification of an AST expression site for lowering attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LoweringSiteKind {
    AssociatedLookup,
    AssociatedInvoke,
    FamilyApplication,
    Match,
}

/// Compiler-facing lowering attachment key.
/// Keyed by source identity, source range, and site kind (no semantic IDs in AST).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LoweringSite {
    pub source: SourceId,
    pub range: SourceRange,
    pub kind: LoweringSiteKind,
}

impl LoweringSite {
    pub fn new(source: SourceId, range: SourceRange, kind: LoweringSiteKind) -> Self {
        Self { source, range, kind }
    }
}

/// Executable rest lane handling mode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExecutableRestMode {
    None,
    Positional,
    Labeled,
    Complete,
}

/// Exact resolved invocation target specification.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExecutableInvocationTarget {
    Behavioral {
        lookup_owner: DeclarationId,
        callable: CallableId,
        operation: FamilyOperationShape,
        rest_mode: ExecutableRestMode,
    },
    VariantConstructor {
        variant: VariantId,
    },
}

/// Target of a member entry in an executable family descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableFamilyTarget {
    Singleton { variant: VariantId },
    Behavioral { target: ExecutableInvocationTarget },
    VariantConstructor { variant: VariantId },
}

/// One executable member entry in a frozen family descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableFamilyEntry {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub target: ExecutableFamilyTarget,
}

/// Frozen executable family descriptor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutableFamilyDescriptor {
    pub entries: Box<[ExecutableFamilyEntry]>,
}

/// Candidate for dynamic family pack invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableFamilyCandidate {
    pub operation: FamilyOperationShape,
    pub target: Option<ExecutableInvocationTarget>,
}

/// Candidate set for dynamic family pack invocation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutableFamilyCandidateSet {
    pub candidates: Box<[ExecutableFamilyCandidate]>,
}

/// Lowering specification for an associated expression site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedLoweringSpec {
    /// Canonical singleton variant load (immediate value).
    SingletonLoad { variant: VariantId },
    /// Fresh constructor case allocation.
    ConstructVariant { variant: VariantId, arity: u8 },
    /// Direct resolved behavioral call (no hierarchy walk, no dNU).
    InvokeResolvedAssociated { target: ExecutableInvocationTarget, arity: u8 },
    /// Exact behavioral member reification as BoundMethod.
    MakeResolvedBoundMethod { target: ExecutableInvocationTarget },
    /// Exact variant constructor reification as closure thunk.
    MakeVariantConstructorThunk { variant: VariantId, operation: FamilyOperationShape },
    /// Frozen whole-family capture.
    MakeAssociatedFamily { descriptor: Arc<ExecutableFamilyDescriptor> },
    /// Dynamic associated invocation over frozen candidate set.
    DynamicInvoke { candidates: Box<[ExecutableFamilyCandidate]> },
    /// Ordinary receiver-bound family capture. Runtime retains captured
    /// receiver and performs live behavioral dispatch on future invocation.
    MakeBehavioralFamily { spec: BehavioralFamilySpec },
    /// Ordinary receiver-bound direct invocation. Runtime uses normal dispatch
    /// against the expression's receiver.
    InvokeBoundBehavioral { selector: phalcom_common::selector::Selector },
}

/// Lowering specification for an application on a first-class family value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum FamilyApplicationLoweringSpec {
    /// Statically known operation invocation on family.
    Static {
        operation: FamilyOperationShape,
        target: Option<ExecutableInvocationTarget>,
        arity: u8,
    },
    /// Dynamic pack invocation restricted to frozen candidates.
    DynamicPack { candidates: Box<[ExecutableFamilyCandidate]> },
}

/// Canonical payload field lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantFieldLoweringSpec {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub slot: u16,
}

/// Variant lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantLoweringSpec {
    pub id: VariantId,
    pub shape: VariantShape,
    pub payload_fields: Box<[VariantFieldLoweringSpec]>,
}

/// Enum declaration lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumLoweringSpec {
    pub owner: DeclarationId,
    pub variants: Box<[VariantLoweringSpec]>,
}

/// Executable binding specification for an arm or pattern context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableBindingSpec {
    pub binding: phalcom_semantic::identity::BindingId,
    pub name: Box<str>,
    pub range: SourceRange,
}

/// Field projection for an exact variant candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableFieldProjection {
    pub field_id: VariantFieldId,
    pub slot: u16,
    pub child: ExecutablePattern,
}

/// Exact resolved variant candidate in an executable pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableVariantCandidate {
    pub variant: VariantId,
    pub fields: Box<[ExecutableFieldProjection]>,
}

/// Backend executable pattern structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutablePattern {
    Wildcard,
    Binding {
        binding_index: u32,
        name: Box<str>,
    },
    Variant {
        candidates: Box<[ExecutableVariantCandidate]>,
    },
    Or {
        alternatives: Box<[ExecutablePattern]>,
    },
    Tuple {
        elements: Box<[ExecutablePattern]>,
    },
    List {
        elements: Box<[ExecutablePattern]>,
        rest: Option<Box<ExecutablePattern>>,
    },
    Record {
        entries: Box<[(Box<str>, ExecutablePattern)]>,
    },
    Map {
        entries: Box<[(phalcom_ast::ast::MapPatternKey, ExecutablePattern)]>,
    },
}

/// Executable match arm lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableMatchArm {
    pub arm_index: u32,
    pub pattern: ExecutablePattern,
    pub bindings: Box<[ExecutableBindingSpec]>,
}

/// Match expression lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchLoweringSpec {
    pub arms: Box<[ExecutableMatchArm]>,
}

/// Complete compiled lowering semantics for a single module.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleLoweringSemantics {
    pub module: ModuleId,
    pub enums: Box<[EnumLoweringSpec]>,
    pub associated: BTreeMap<LoweringSite, AssociatedLoweringSpec>,
    pub family_values: BTreeSet<LoweringSite>,
    pub family_application_sites: BTreeSet<LoweringSite>,
    pub family_applications: BTreeMap<LoweringSite, FamilyApplicationLoweringSpec>,
    pub matches: BTreeMap<LoweringSite, MatchLoweringSpec>,
}

impl ModuleLoweringSemantics {
    pub fn new(module: ModuleId) -> Self {
        Self {
            module,
            enums: Box::new([]),
            associated: BTreeMap::new(),
            family_values: BTreeSet::new(),
            family_application_sites: BTreeSet::new(),
            family_applications: BTreeMap::new(),
            matches: BTreeMap::new(),
        }
    }
}

/// Errors occurring during semantic-to-lowering projection.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum ProjectionError {
    #[error("ambiguous lowering site attachment at {0:?}")]
    AmbiguousLoweringSiteAttachment(LoweringSite),
    #[error("missing source range for expression {0:?}")]
    MissingSourceRange(ExpressionId),
    #[error("missing variant metadata for {0:?}")]
    MissingVariantMetadata(VariantId),
    #[error("missing field layout for field {0:?} in variant {1:?}")]
    MissingFieldLayout(VariantFieldId, VariantId),
    #[error("missing pattern binding for {0:?}")]
    MissingPatternBinding(phalcom_semantic::identity::BindingId),
    #[error("pattern binding index overflow for {0}")]
    PatternBindingIndexOverflow(usize),
    #[error("arity overflow for length {0}")]
    ArityOverflow(usize),
    #[error("slot overflow for index {0}")]
    SlotOverflow(usize),
    #[error("non-proven match reached executable lowering for expression {0:?}")]
    NonProvenMatch(ExpressionId),
    #[error("ordinary behavioral resolution carried a non-behavioral target")]
    InvalidBoundBehavioralTarget,
    #[error("missing constructor metadata for variant {0:?}")]
    MissingConstructorMetadata(VariantId),
}

/// Projects formal snapshot products into an immutable `ModuleLoweringSemantics` bundle.
pub fn build_module_lowering_semantics(module: &ModuleId, snapshot: &SemanticSnapshot) -> Result<ModuleLoweringSemantics, ProjectionError> {
    let source_id = if let Some(parsed_unit) = snapshot.sources.get(module) {
        parsed_unit
            .source
            .as_ref()
            .map(|s| s.source_id.clone())
            .unwrap_or_else(|| SourceId(module.to_string().into_boxed_str()))
    } else {
        SourceId(module.to_string().into_boxed_str())
    };

    // 1. Project Enums in this module
    let mut enums = Vec::new();
    for (owner, enum_info) in &snapshot.enum_semantics.enums {
        if owner.module != *module {
            continue;
        }
        let mut variants = Vec::new();
        for variant_id in enum_info.variants.iter() {
            let vinfo = snapshot
                .enum_semantics
                .variant_info(variant_id)
                .ok_or_else(|| ProjectionError::MissingVariantMetadata(variant_id.clone()))?;
            let shape = vinfo.shape;
            let mut payload_fields = Vec::new();
            for (idx, field) in vinfo.fields.iter().enumerate() {
                let slot = u16::try_from(idx).map_err(|_| ProjectionError::SlotOverflow(idx))?;
                payload_fields.push(VariantFieldLoweringSpec {
                    id: field.id.clone(),
                    local_name: field.local_name.clone(),
                    slot,
                });
            }
            variants.push(VariantLoweringSpec {
                id: variant_id.clone(),
                shape,
                payload_fields: payload_fields.into_boxed_slice(),
            });
        }
        enums.push(EnumLoweringSpec {
            owner: owner.clone(),
            variants: variants.into_boxed_slice(),
        });
    }
    enums.sort_by(|a, b| a.owner.cmp(&b.owner));

    // 2. Project Associated Expressions & Family Applications
    let mut associated = BTreeMap::new();
    let mut family_values = BTreeSet::new();
    let mut family_application_sites = BTreeSet::new();
    let mut family_applications = BTreeMap::new();
    let mut matches = BTreeMap::new();

    for (callable_id, analysis) in snapshot.callable_analyses.iter() {
        if callable_id.owner.module() != module {
            continue;
        }

        // Associated resolutions
        for (expr_id, resolution) in analysis.associated_resolutions.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let (kind, spec) = project_associated_resolution(resolution, snapshot)?;
            let site = LoweringSite::new(source_id.clone(), range, kind);

            if associated.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            associated.insert(site, spec);
        }

        // Family-valued expressions
        for expression in analysis.expressions.values() {
            let is_associated_family = matches!(
                expression.denotation.as_ref(),
                Some(SemanticDenotation::AssociatedValue(assoc))
                    if matches!(&**assoc, AssociatedValueDenotation::Family { .. })
            );
            if is_associated_family
                && let Some(ty) = expression.knowledge.ty()
                && matches!(snapshot.store.get(ty), TypeData::Family(_))
            {
                family_values.insert(LoweringSite::new(source_id.clone(), expression.range, LoweringSiteKind::FamilyApplication));
            }
        }

        // Family applications
        for (expr_id, fam_app) in analysis.family_applications.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let spec = project_family_application(snapshot, fam_app)?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::FamilyApplication);
            family_application_sites.insert(site.clone());

            if family_applications.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            family_applications.insert(site, spec);
        }

        // 3. Project Match expressions
        for (expr_id, match_resolution) in analysis.match_resolutions.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let spec = project_match_resolution(match_resolution, snapshot)?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::Match);

            if matches.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            matches.insert(site, spec);
        }
    }

    Ok(ModuleLoweringSemantics {
        module: module.clone(),
        enums: enums.into_boxed_slice(),
        associated,
        family_values,
        family_application_sites,
        family_applications,
        matches,
    })
}

fn project_match_resolution(
    resolution: &phalcom_semantic::match_semantics::MatchResolution,
    snapshot: &SemanticSnapshot,
) -> Result<MatchLoweringSpec, ProjectionError> {
    if !matches!(resolution.exhaustiveness, phalcom_semantic::match_semantics::ExhaustivenessResult::Proven) {
        return Err(ProjectionError::NonProvenMatch(resolution.expression));
    }

    let mut arms = Vec::with_capacity(resolution.arms.len());
    for arm in resolution.arms.iter() {
        let bindings = arm
            .bindings
            .iter()
            .map(|b| ExecutableBindingSpec {
                binding: b.binding.clone(),
                name: b.name.clone(),
                range: b.source,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let pattern = project_pattern_resolution(&arm.pattern, &arm.bindings, snapshot)?;

        arms.push(ExecutableMatchArm {
            arm_index: arm.arm_index,
            pattern,
            bindings,
        });
    }

    Ok(MatchLoweringSpec { arms: arms.into_boxed_slice() })
}

fn project_pattern_resolution(
    pattern: &phalcom_semantic::match_semantics::PatternResolution,
    bindings: &[phalcom_semantic::match_semantics::PatternBindingResolution],
    snapshot: &SemanticSnapshot,
) -> Result<ExecutablePattern, ProjectionError> {
    match pattern {
        phalcom_semantic::match_semantics::PatternResolution::Wildcard => Ok(ExecutablePattern::Wildcard),
        phalcom_semantic::match_semantics::PatternResolution::Binding { binding, name, .. } => {
            let index = bindings
                .iter()
                .position(|b| b.binding == *binding)
                .ok_or_else(|| ProjectionError::MissingPatternBinding(binding.clone()))?;
            let binding_index = u32::try_from(index).map_err(|_| ProjectionError::PatternBindingIndexOverflow(index))?;
            Ok(ExecutablePattern::Binding {
                binding_index,
                name: name.clone(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Variant(var_pat) => {
            let mut candidates = Vec::with_capacity(var_pat.candidates.len());
            for candidate in var_pat.candidates.iter() {
                let vinfo = snapshot
                    .enum_semantics
                    .variant_info(&candidate.variant)
                    .ok_or_else(|| ProjectionError::MissingVariantMetadata(candidate.variant.clone()))?;
                let mut field_projections = Vec::with_capacity(candidate.fields.len());
                for field in candidate.fields.iter() {
                    let idx = vinfo
                        .fields
                        .iter()
                        .position(|f| f.id == field.field)
                        .ok_or_else(|| ProjectionError::MissingFieldLayout(field.field.clone(), candidate.variant.clone()))?;
                    let slot = u16::try_from(idx).map_err(|_| ProjectionError::SlotOverflow(idx))?;

                    let child = project_pattern_resolution(&field.child, bindings, snapshot)?;
                    field_projections.push(ExecutableFieldProjection {
                        field_id: field.field.clone(),
                        slot,
                        child,
                    });
                }

                candidates.push(ExecutableVariantCandidate {
                    variant: candidate.variant.clone(),
                    fields: field_projections.into_boxed_slice(),
                });
            }

            Ok(ExecutablePattern::Variant {
                candidates: candidates.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Or(or_pat) => {
            let mut alts = Vec::with_capacity(or_pat.alternatives.len());
            for alt in or_pat.alternatives.iter() {
                alts.push(project_pattern_resolution(alt, bindings, snapshot)?);
            }
            Ok(ExecutablePattern::Or {
                alternatives: alts.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Tuple(elements) => {
            let mut elems = Vec::with_capacity(elements.len());
            for elem in elements.iter() {
                elems.push(project_pattern_resolution(elem, bindings, snapshot)?);
            }
            Ok(ExecutablePattern::Tuple {
                elements: elems.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::List(list_pat) => {
            let mut elems = Vec::with_capacity(list_pat.prefix.len());
            for elem in list_pat.prefix.iter() {
                elems.push(project_pattern_resolution(elem, bindings, snapshot)?);
            }
            let rest = if let Some(r) = &list_pat.rest {
                Some(Box::new(project_pattern_resolution(r, bindings, snapshot)?))
            } else {
                None
            };
            Ok(ExecutablePattern::List {
                elements: elems.into_boxed_slice(),
                rest,
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Record(fields) => {
            let mut entries = Vec::with_capacity(fields.len());
            for f in fields.iter() {
                let child = project_pattern_resolution(&f.child, bindings, snapshot)?;
                entries.push((f.label.clone(), child));
            }
            Ok(ExecutablePattern::Record {
                entries: entries.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Map(entries_pat) => {
            let mut entries = Vec::with_capacity(entries_pat.len());
            for e in entries_pat.iter() {
                let child = project_pattern_resolution(&e.child, bindings, snapshot)?;
                entries.push((e.key.clone(), child));
            }
            Ok(ExecutablePattern::Map {
                entries: entries.into_boxed_slice(),
            })
        }
    }
}

fn project_associated_resolution(
    resolution: &AssociatedResolution,
    snapshot: &SemanticSnapshot,
) -> Result<(LoweringSiteKind, AssociatedLoweringSpec), ProjectionError> {
    match &resolution.kind {
        AssociatedResolutionKind::ExactValue { member, .. } => {
            let spec = match member {
                AssociatedMemberId::Variant(v) => AssociatedLoweringSpec::SingletonLoad { variant: v.clone() },
            };
            Ok((LoweringSiteKind::AssociatedLookup, spec))
        }
        AssociatedResolutionKind::ExactCallable { target, .. } => {
            let spec = match target {
                InvocationTargetId::Behavioral(c) => AssociatedLoweringSpec::MakeResolvedBoundMethod {
                    target: ExecutableInvocationTarget::Behavioral {
                        lookup_owner: resolution.lookup_owner.clone(),
                        callable: c.clone(),
                        operation: behavioral_operation(c),
                        rest_mode: executable_rest_mode(snapshot, c),
                    },
                },
                InvocationTargetId::VariantConstructor(vc) => AssociatedLoweringSpec::MakeVariantConstructorThunk {
                    variant: vc.variant.clone(),
                    operation: variant_constructor_operation(snapshot, &vc.variant)?,
                },
            };
            Ok((LoweringSiteKind::AssociatedLookup, spec))
        }
        AssociatedResolutionKind::Family { members, .. } => {
            let mut entries = Vec::new();
            for member in members.iter() {
                let (target, member_kind) = match (&member.member, &member.target) {
                    (AssociatedMemberId::Variant(variant), None) => {
                        (ExecutableFamilyTarget::Singleton { variant: variant.clone() }, FamilyMemberTypeKind::Value)
                    }
                    (_, Some(InvocationTargetId::VariantConstructor(vc))) => (
                        ExecutableFamilyTarget::VariantConstructor { variant: vc.variant.clone() },
                        FamilyMemberTypeKind::Callable,
                    ),
                    (_, Some(InvocationTargetId::Behavioral(c))) => (
                        ExecutableFamilyTarget::Behavioral {
                            target: ExecutableInvocationTarget::Behavioral {
                                lookup_owner: resolution.lookup_owner.clone(),
                                callable: c.clone(),
                                operation: member.operation.clone(),
                                rest_mode: executable_rest_mode(snapshot, c),
                            },
                        },
                        FamilyMemberTypeKind::Callable,
                    ),
                };
                entries.push(ExecutableFamilyEntry {
                    operation: member.operation.clone(),
                    member_kind,
                    target,
                });
            }
            let desc = ExecutableFamilyDescriptor {
                entries: entries.into_boxed_slice(),
            };
            Ok((
                LoweringSiteKind::AssociatedLookup,
                AssociatedLoweringSpec::MakeAssociatedFamily { descriptor: Arc::new(desc) },
            ))
        }
        AssociatedResolutionKind::StaticInvoke { target, .. } => {
            let spec = match target {
                InvocationTargetId::VariantConstructor(vc) => {
                    let vinfo = snapshot
                        .enum_semantics
                        .variant_info(&vc.variant)
                        .ok_or_else(|| ProjectionError::MissingVariantMetadata(vc.variant.clone()))?;
                    let arity = u8::try_from(vinfo.fields.len()).map_err(|_| ProjectionError::ArityOverflow(vinfo.fields.len()))?;
                    AssociatedLoweringSpec::ConstructVariant {
                        variant: vc.variant.clone(),
                        arity,
                    }
                }
                InvocationTargetId::Behavioral(c) => {
                    let arity = u8::try_from(c.selector.slots.len()).map_err(|_| ProjectionError::ArityOverflow(c.selector.slots.len()))?;
                    AssociatedLoweringSpec::InvokeResolvedAssociated {
                        target: ExecutableInvocationTarget::Behavioral {
                            lookup_owner: resolution.lookup_owner.clone(),
                            callable: c.clone(),
                            operation: behavioral_operation(c),
                            rest_mode: executable_rest_mode(snapshot, c),
                        },
                        arity,
                    }
                }
            };
            Ok((LoweringSiteKind::AssociatedInvoke, spec))
        }
        AssociatedResolutionKind::DynamicInvoke { candidates, .. } => {
            let mut exec_candidates = Vec::new();
            for c in candidates.iter() {
                let target = c.target.as_ref().map(|t| match t {
                    InvocationTargetId::Behavioral(cid) => ExecutableInvocationTarget::Behavioral {
                        lookup_owner: resolution.lookup_owner.clone(),
                        callable: cid.clone(),
                        operation: c.operation.clone(),
                        rest_mode: executable_rest_mode(snapshot, cid),
                    },
                    InvocationTargetId::VariantConstructor(vc) => ExecutableInvocationTarget::VariantConstructor { variant: vc.variant.clone() },
                });
                exec_candidates.push(ExecutableFamilyCandidate {
                    operation: c.operation.clone(),
                    target,
                });
            }
            let spec = AssociatedLoweringSpec::DynamicInvoke {
                candidates: exec_candidates.into_boxed_slice(),
            };
            Ok((LoweringSiteKind::AssociatedInvoke, spec))
        }
        AssociatedResolutionKind::BoundBehavioralFamily { spec, .. } => Ok((
            LoweringSiteKind::AssociatedLookup,
            AssociatedLoweringSpec::MakeBehavioralFamily { spec: spec.clone() },
        )),
        AssociatedResolutionKind::BoundBehavioralInvoke { target, .. } => {
            let InvocationTargetId::Behavioral(callable) = target else {
                return Err(ProjectionError::InvalidBoundBehavioralTarget);
            };
            Ok((
                LoweringSiteKind::AssociatedInvoke,
                AssociatedLoweringSpec::InvokeBoundBehavioral {
                    selector: callable.selector.clone(),
                },
            ))
        }
    }
}

fn behavioral_operation(callable: &CallableId) -> FamilyOperationShape {
    FamilyOperationShape::new(callable.selector.kind, callable.selector.slots.clone())
}

fn variant_constructor_operation(snapshot: &SemanticSnapshot, variant: &VariantId) -> Result<FamilyOperationShape, ProjectionError> {
    let info = snapshot
        .enum_semantics
        .variant_info(variant)
        .ok_or_else(|| ProjectionError::MissingVariantMetadata(variant.clone()))?;
    let constructor = info
        .constructor
        .as_ref()
        .ok_or_else(|| ProjectionError::MissingConstructorMetadata(variant.clone()))?;
    let slots = constructor
        .parameters
        .iter()
        .map(|parameter| match &parameter.external_label {
            Some(label) => phalcom_common::selector::SelectorSlot::Label(label.to_string()),
            None => phalcom_common::selector::SelectorSlot::Positional,
        })
        .collect::<Vec<_>>();
    Ok(FamilyOperationShape::method(slots.into_boxed_slice()))
}

fn executable_rest_mode(snapshot: &SemanticSnapshot, callable: &CallableId) -> ExecutableRestMode {
    snapshot
        .callable_signatures
        .get(callable)
        .and_then(|signature| signature.parameters.iter().find(|parameter| parameter.rest != phalcom_ast::ast::RestMode::None))
        .map(|parameter| match parameter.rest {
            phalcom_ast::ast::RestMode::None => ExecutableRestMode::None,
            phalcom_ast::ast::RestMode::Positional => ExecutableRestMode::Positional,
            phalcom_ast::ast::RestMode::Labeled => ExecutableRestMode::Labeled,
            phalcom_ast::ast::RestMode::Complete => ExecutableRestMode::Complete,
        })
        .unwrap_or(ExecutableRestMode::None)
}

fn project_family_application(snapshot: &SemanticSnapshot, fam_app: &FamilyApplicationResolution) -> Result<FamilyApplicationLoweringSpec, ProjectionError> {
    match &fam_app.selection {
        FamilyApplicationSelection::Static { operation, target, .. } => {
            let exec_target = target.as_ref().map(|target| executable_invocation_target(snapshot, target, operation));
            let arity = u8::try_from(operation.slots.len()).map_err(|_| ProjectionError::ArityOverflow(operation.slots.len()))?;
            Ok(FamilyApplicationLoweringSpec::Static {
                operation: operation.clone(),
                target: exec_target,
                arity,
            })
        }
        FamilyApplicationSelection::Dynamic { candidates, .. } => {
            let exec_candidates = candidates
                .iter()
                .map(|candidate| ExecutableFamilyCandidate {
                    operation: candidate.operation.clone(),
                    target: candidate
                        .target
                        .as_ref()
                        .map(|target| executable_invocation_target(snapshot, target, &candidate.operation)),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            Ok(FamilyApplicationLoweringSpec::DynamicPack { candidates: exec_candidates })
        }
    }
}

fn executable_invocation_target(snapshot: &SemanticSnapshot, target: &InvocationTargetId, operation: &FamilyOperationShape) -> ExecutableInvocationTarget {
    match target {
        InvocationTargetId::Behavioral(c) => ExecutableInvocationTarget::Behavioral {
            lookup_owner: c.owner.declaration().clone(),
            callable: c.clone(),
            operation: operation.clone(),
            rest_mode: executable_rest_mode(snapshot, c),
        },
        InvocationTargetId::VariantConstructor(vc) => ExecutableInvocationTarget::VariantConstructor { variant: vc.variant.clone() },
    }
}
