//! Callable-local frozen proof graph and retained match/pattern authority.

use super::*;
use crate::semantic::type_store::FieldKeyV1;

record!(CallableProofScopeV1 { parent: Option<u32> });
tagged!(CallableRigidOriginV1 {
    0 => VariantParameter { variant: StableVariantRef, parameter: ArtifactTypeParameterId },
    1 => Synthetic,
});
record!(CallableRigidV1 {
    scope: u32,
    kind: ArtifactKindId,
    origin: CallableRigidOriginV1
});
record!(CallableProofGraphV1 { owner: StableCallableRef, scopes: Vec<CallableProofScopeV1>, rigids: Vec<CallableRigidV1> });

tagged!(CallableLocalTypeV1 {
    0 => Canonical { ty: ArtifactTypeId },
    1 => Rigid { rigid: u32 },
    2 => Applied { origin: Box<CallableLocalTypeV1>, arguments: Vec<CallableLocalTypeV1> },
    3 => ExactCase { variant: StableVariantRef, enum_type: Box<CallableLocalTypeV1> },
    4 => Union { members: Vec<CallableLocalTypeV1> },
    5 => Tuple { elements: Vec<CallableLocalTupleElementV1> },
    6 => Record { fields: Vec<CallableLocalRecordFieldV1> },
    7 => Callable { parameters: Vec<CallableLocalParameterV1>, return_type: Box<CallableLocalTypeV1> },
});
record!(CallableLocalTupleElementV1 { label: Option<Box<str>>, ty: CallableLocalTypeV1 });
record!(CallableLocalRecordFieldV1 {
    key: FieldKeyV1,
    ty: CallableLocalTypeV1
});
record!(CallableLocalParameterV1 { label: Option<Box<str>>, ty: CallableLocalTypeV1, rest: u8 });
tagged!(CallableLocalConstraintV1 {
    0 => Subtype { lower: CallableLocalTypeV1, upper: CallableLocalTypeV1 },
    1 => Equivalent { left: CallableLocalTypeV1, right: CallableLocalTypeV1 },
});
tagged!(CallableProofConstraintV1 {
    0 => Equal { left: ArtifactTypeId, right: ArtifactTypeId },
    1 => Subtype { lower: ArtifactTypeId, upper: ArtifactTypeId },
    2 => HasMember { ty: ArtifactTypeId, selector: ExactSelectorV1 },
});
record!(CallableProofBindingV1 {
    parameter: ArtifactTypeParameterId,
    ty: ArtifactTypeId
});
record!(CallableLocalBindingV1 {
    parameter: ArtifactTypeParameterId,
    ty: CallableLocalTypeV1
});
record!(CallableBranchProofV1 {
    bindings: Vec<CallableProofBindingV1>, equalities: Vec<CallableProofConstraintV1>,
    local_bindings: Vec<CallableLocalBindingV1>, local_equalities: Vec<CallableLocalConstraintV1>,
});
record!(CallableCaseRigidV1 {
    parameter: ArtifactTypeParameterId,
    rigid: u32
});
record!(CallableCaseInstantiationV1 {
    variant: StableVariantRef, scope: u32, local_rigids: Vec<CallableCaseRigidV1>,
    payload_types: Vec<CallableLocalTypeV1>, result_type: CallableLocalTypeV1, constraints: Vec<CallableLocalConstraintV1>,
});

tagged!(CallableVariantSelectorV1 {
    0 => Exact { selector: ExactSelectorV1 },
    1 => Pattern { pattern: SelectorPatternSpecV1 },
    2 => WholeFamily,
});
tagged!(CallablePatternV1 {
    0 => Wildcard,
    1 => Char { value: u32 },
    2 => Binding { binding: u32, name: Box<str>, knowledge: TypeKnowledgeV1 },
    3 => Variant { owner: Option<StableDeclarationRef>, family: Option<AssociatedFamilyIdentityV1>, owner_candidates: Vec<StableDeclarationRef>,
        selector: CallableVariantSelectorV1, candidates: Vec<CallableVariantCandidateV1>, blocked: Option<CallableBlockReasonV1> },
    4 => Or { alternatives: Vec<CallablePatternV1> },
    5 => Tuple { elements: Vec<CallablePatternV1> },
    6 => List { prefix: Vec<CallablePatternV1>, rest: Option<Box<CallablePatternV1>> },
    7 => Record { fields: Vec<CallableRecordPatternV1> },
    8 => Map { entries: Vec<CallableMapPatternV1> },
});
record!(CallableVariantCandidateV1 { variant: StableVariantRef, exact_case: ArtifactTypeId, fields: Vec<CallableFieldPatternV1>, proof: CallableBranchProofV1, case_instantiation: Option<CallableCaseInstantiationV1> });
record!(CallableFieldPatternV1 { variant: StableVariantRef, field: u32, field_type: TypeKnowledgeV1, local_type: Option<CallableLocalTypeV1>, child: Box<CallablePatternV1> });
record!(CallableRecordPatternV1 { label: Box<str>, child: Box<CallablePatternV1> });
tagged!(CallableMapPatternKeyV1 {
    0 => Symbol { value: Box<str> }, 1 => String { value: Box<str> }, 2 => Int { digits: Box<str>, radix: u32 },
});
record!(CallableMapPatternV1 { key: CallableMapPatternKeyV1, child: Box<CallablePatternV1> });
record!(CallablePatternBindingV1 { binding: u32, name: Box<str>, knowledge: TypeKnowledgeV1, local_type: Option<CallableLocalTypeV1>, source: SpanV1 });
tagged!(CallablePatternUsefulnessV1 { 0 => Useful, 1 => Redundant, 2 => Impossible });
tagged!(CallablePatternSpaceV1 {
    0 => Empty, 1 => Char { value: u32 }, 2 => Blocked { reason: CallableBlockReasonV1 }, 3 => Opaque { ty: ArtifactTypeId },
    4 => Union { members: Vec<CallablePatternSpaceV1> },
    5 => Variant { variant: StableVariantRef, exact_case: ArtifactTypeId, fields: Vec<CallablePatternSpaceV1> },
    6 => Tuple { elements: Vec<CallablePatternSpaceV1> }, 7 => List,
});
tagged!(CallableCoverageWitnessV1 {
    0 => Char { value: u32 }, 1 => Variant { variant: StableVariantRef, exact_case: ArtifactTypeId, fields: Vec<CallableCoverageWitnessV1> },
    2 => Tuple { elements: Vec<CallableCoverageWitnessV1> }, 3 => List { elements: Vec<CallableCoverageWitnessV1> },
    4 => Opaque { ty: ArtifactTypeId }, 5 => Wildcard,
});
tagged!(CallableExhaustivenessV1 { 0 => Proven, 1 => Missing { witnesses: Vec<CallableCoverageWitnessV1> }, 2 => Blocked { reason: CallableBlockReasonV1 } });
record!(CallableMatchArmV1 { arm_index: u32, pattern: CallablePatternV1, reachable_space: CallablePatternSpaceV1, residual_after: CallablePatternSpaceV1,
    bindings: Vec<CallablePatternBindingV1>, proof: CallableBranchProofV1, usefulness: CallablePatternUsefulnessV1, branch_result: TypeKnowledgeV1 });
record!(CallableMatchResolutionV1 { expression: u32, scrutinee: TypeKnowledgeV1, initial_space: CallablePatternSpaceV1, arms: Vec<CallableMatchArmV1>, result: TypeKnowledgeV1, exhaustiveness: CallableExhaustivenessV1 });
tagged!(CallablePatternSiteKindV1 { 0 => IfLet, 1 => WhileLet, 2 => ForLane });
record!(CallablePatternSiteResolutionV1 { ordinal: u32, kind: CallablePatternSiteKindV1, range: SpanV1, subject: TypeKnowledgeV1, pattern: CallablePatternV1, bindings: Vec<CallablePatternBindingV1>, usefulness: CallablePatternUsefulnessV1 });

impl CallableAnalysisRecordV1 {
    /// Checks local proof closure without allocating scratch or materializing owners.
    pub fn validate_proof_graph(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        validate_callable_proofs(None, self, limits).map(|_| ())
    }

    /// Also checks canonical type, kind, parameter, variant, and field owners.
    pub fn validate_proof_authority(
        &self,
        type_graph: &TypeGraphV1,
        structural_contracts: &StructuralPrimaryContractsV1,
        limits: &ValidationLimits,
    ) -> Result<(), ArtifactError> {
        validate_proofs(
            Some(ProofWorld {
                type_graph,
                structural_contracts,
            }),
            self,
            limits,
        )
        .map(|_| ())
    }
}

#[derive(Clone, Copy)]
struct ProofWorld<'a> {
    type_graph: &'a TypeGraphV1,
    structural_contracts: &'a StructuralPrimaryContractsV1,
}

impl ProofWorld<'_> {
    fn validate_denotation_type(&self, ty: ArtifactTypeId) -> Result<(), ArtifactError> {
        if ty.0 as usize >= self.type_graph.types.len() {
            return Err(invalid("callable proof canonical type is absent"));
        }
        Ok(())
    }
}

/// No scratch allocation: all references are into one record's local graph.
pub(super) fn validate_callable_proofs(
    world: Option<&SemanticWorldV3>,
    record: &CallableAnalysisRecordV1,
    limits: &ValidationLimits,
) -> Result<usize, ArtifactError> {
    validate_proofs(
        world.map(|world| ProofWorld {
            type_graph: &world.type_graph,
            structural_contracts: &world.structural_contracts,
        }),
        record,
        limits,
    )
}

fn validate_proofs(world: Option<ProofWorld<'_>>, record: &CallableAnalysisRecordV1, limits: &ValidationLimits) -> Result<usize, ArtifactError> {
    let mut check = ProofValidator {
        world,
        record,
        limits,
        rows: 0,
    };
    if let Some(graph) = &record.local_proof_graph {
        if graph.owner != record.callable {
            return Err(invalid("callable proof graph belongs to another callable"));
        }
        for (index, scope) in graph.scopes.iter().enumerate() {
            check.row(0)?;
            if scope.parent.is_some_and(|parent| parent as usize >= index) {
                return Err(invalid("callable proof scope parent is absent, cyclic, or noncanonical"));
            }
        }
        for rigid in &graph.rigids {
            check.row(0)?;
            check.scope(rigid.scope)?;
            if let Some(world) = world {
                if rigid.kind.0 as usize >= world.type_graph.kinds.len() {
                    return Err(invalid("callable proof rigid kind is absent"));
                }
                if let CallableRigidOriginV1::VariantParameter { variant, parameter } = &rigid.origin {
                    check.variant(variant)?;
                    let parameter = check.parameter(*parameter)?;
                    if let Some(parameter) = parameter {
                        let constructor_owns_parameter = world
                            .structural_contracts
                            .variant_contracts
                            .iter()
                            .find(|row| &row.variant == variant)
                            .and_then(|row| row.constructor.as_ref())
                            .and_then(|constructor| constructor.generic_signature.as_ref())
                            .is_some_and(|signature| {
                                signature.parameters.contains(match &rigid.origin {
                                    CallableRigidOriginV1::VariantParameter { parameter, .. } => parameter,
                                    _ => unreachable!(),
                                })
                            });
                        if parameter.kind != rigid.kind || !constructor_owns_parameter {
                            return Err(invalid("callable proof rigid origin is not a parameter of its variant constructor"));
                        }
                    }
                }
            }
        }
    }
    if record.match_resolutions.windows(2).any(|pair| pair[0].expression >= pair[1].expression) {
        return Err(invalid("callable match expressions are not canonical"));
    }
    for resolution in &record.match_resolutions {
        check.row(0)?;
        if resolution.expression as usize >= record.expressions.len() {
            return Err(invalid("callable match references an absent expression"));
        }
        check.knowledge(&resolution.scrutinee)?;
        check.space(&resolution.initial_space, 0)?;
        for (index, arm) in resolution.arms.iter().enumerate() {
            check.row(0)?;
            if arm.arm_index as usize != index {
                return Err(invalid("callable match arms are not dense and source ordered"));
            }
            check.pattern(&arm.pattern, 0)?;
            check.space(&arm.reachable_space, 0)?;
            check.space(&arm.residual_after, 0)?;
            check.bindings(&arm.bindings)?;
            check.proof(&arm.proof, None, 0)?;
            check.knowledge(&arm.branch_result)?;
        }
        check.knowledge(&resolution.result)?;
        match &resolution.exhaustiveness {
            CallableExhaustivenessV1::Proven => {}
            CallableExhaustivenessV1::Blocked { reason } => check.reason(reason)?,
            CallableExhaustivenessV1::Missing { witnesses } => {
                for witness in witnesses {
                    check.witness(witness, 0)?;
                }
            }
        }
    }
    for (index, site) in record.pattern_site_resolutions.iter().enumerate() {
        check.row(0)?;
        if site.ordinal as usize != index || site.range.start > site.range.end {
            return Err(invalid("callable pattern sites have invalid ordinals or ranges"));
        }
        let key = |site: &CallablePatternSiteResolutionV1| {
            (
                match site.kind {
                    CallablePatternSiteKindV1::IfLet => 0,
                    CallablePatternSiteKindV1::WhileLet => 1,
                    CallablePatternSiteKindV1::ForLane => 2,
                },
                site.range.start,
                site.range.end,
            )
        };
        if index > 0 && key(&record.pattern_site_resolutions[index - 1]) >= key(site) {
            return Err(invalid("callable pattern site keys are not canonical"));
        }
        check.knowledge(&site.subject)?;
        check.pattern(&site.pattern, 0)?;
        check.bindings(&site.bindings)?;
    }
    Ok(check.rows)
}

struct ProofValidator<'a> {
    world: Option<ProofWorld<'a>>,
    record: &'a CallableAnalysisRecordV1,
    limits: &'a ValidationLimits,
    rows: usize,
}
impl ProofValidator<'_> {
    fn row(&mut self, depth: usize) -> Result<(), ArtifactError> {
        self.rows = self.rows.checked_add(1).ok_or_else(|| invalid("callable proof row overflow"))?;
        if depth > self.limits.max_recursion_depth || self.rows > self.limits.max_semantic_entries_per_family {
            return Err(invalid("callable proof nesting or row limit exceeded"));
        }
        Ok(())
    }
    fn text(&self, text: &str) -> Result<(), ArtifactError> {
        if text.len() > self.limits.max_string_bytes {
            Err(invalid("callable proof string limit exceeded"))
        } else {
            Ok(())
        }
    }
    fn ty(&self, ty: ArtifactTypeId) -> Result<(), ArtifactError> {
        self.world.map_or(Ok(()), |world| world.validate_denotation_type(ty))
    }
    fn parameter(&self, id: ArtifactTypeParameterId) -> Result<Option<&crate::semantic::type_store::ParameterV1>, ArtifactError> {
        self.world
            .map(|world| {
                world
                    .type_graph
                    .parameters
                    .get(id.0 as usize)
                    .ok_or_else(|| invalid("callable proof parameter is absent"))
            })
            .transpose()
    }
    fn variant(&self, variant: &StableVariantRef) -> Result<(), ArtifactError> {
        if self
            .world
            .is_some_and(|world| !world.structural_contracts.variant_contracts.iter().any(|row| &row.variant == variant))
        {
            return Err(invalid("callable proof variant owner is absent"));
        }
        Ok(())
    }
    fn exact_case(&self, variant: &StableVariantRef, ty: ArtifactTypeId) -> Result<(), ArtifactError> {
        self.variant(variant)?;
        self.ty(ty)?;
        if let Some(world) = self.world {
            let matches = world.type_graph.types.get(ty.0 as usize).is_some_and(|entry| {
                matches!(&entry.form, crate::semantic::type_store::TypeV1::ExactCase { variant: id, .. }
                    if world.type_graph.variants.get(id.0 as usize) == Some(variant))
            });
            if !matches {
                return Err(invalid("callable pattern exact-case type belongs to another variant"));
            }
        }
        Ok(())
    }
    fn scope(&self, ordinal: u32) -> Result<(), ArtifactError> {
        if self
            .record
            .local_proof_graph
            .as_ref()
            .is_none_or(|graph| ordinal as usize >= graph.scopes.len())
        {
            return Err(invalid("callable proof scope is absent"));
        }
        Ok(())
    }
    fn rigid(&self, ordinal: u32, scope: Option<u32>) -> Result<(), ArtifactError> {
        let graph = self
            .record
            .local_proof_graph
            .as_ref()
            .ok_or_else(|| invalid("callable proof rigid has no owning graph"))?;
        let rigid = graph.rigids.get(ordinal as usize).ok_or_else(|| invalid("callable proof rigid is absent"))?;
        if let Some(mut current) = scope {
            let mut visible = false;
            for _ in 0..graph.scopes.len() {
                if current == rigid.scope {
                    visible = true;
                    break;
                }
                let Some(parent) = graph.scopes.get(current as usize).and_then(|scope| scope.parent) else {
                    break;
                };
                current = parent;
            }
            if !visible {
                return Err(invalid("callable proof rigid is not visible from its scope"));
            }
        }
        Ok(())
    }
    fn reason(&self, reason: &CallableBlockReasonV1) -> Result<(), ArtifactError> {
        self.world
            .map_or(Ok(()), |world| validate_callable_block_reason(reason, &world.type_graph, self.limits))
    }
    fn knowledge(&mut self, knowledge: &TypeKnowledgeV1) -> Result<(), ArtifactError> {
        if let TypeKnowledgeV1::Known { ty, provenance, .. } = knowledge {
            self.ty(*ty)?;
            for range in &provenance.ranges {
                self.row(0)?;
                if range.start > range.end {
                    return Err(invalid("callable pattern evidence range is inverted"));
                }
            }
            for text in &provenance.descriptions {
                self.row(0)?;
                self.text(text)?;
            }
        }
        if let TypeKnowledgeV1::Unknown {
            reason: UnknownReasonV1::UnresolvedName { name },
        } = knowledge
        {
            self.text(name)?;
        }
        Ok(())
    }
    fn local(&mut self, term: &CallableLocalTypeV1, scope: Option<u32>, depth: usize) -> Result<(), ArtifactError> {
        self.row(depth)?;
        match term {
            CallableLocalTypeV1::Canonical { ty } => self.ty(*ty)?,
            CallableLocalTypeV1::Rigid { rigid } => self.rigid(*rigid, scope)?,
            CallableLocalTypeV1::Applied { origin, arguments } => {
                self.local(origin, scope, depth + 1)?;
                for term in arguments {
                    self.local(term, scope, depth + 1)?;
                }
            }
            CallableLocalTypeV1::ExactCase { variant, enum_type } => {
                self.variant(variant)?;
                self.local(enum_type, scope, depth + 1)?;
            }
            CallableLocalTypeV1::Union { members } => {
                for term in members {
                    self.local(term, scope, depth + 1)?;
                }
            }
            CallableLocalTypeV1::Tuple { elements } => {
                for element in elements {
                    if let Some(label) = &element.label {
                        self.text(label)?;
                    }
                    self.local(&element.ty, scope, depth + 1)?;
                }
            }
            CallableLocalTypeV1::Record { fields } => {
                for field in fields {
                    match &field.key {
                        FieldKeyV1::Symbol { value } | FieldKeyV1::String { value } => self.text(value)?,
                        FieldKeyV1::Int { decimal } => {
                            self.text(decimal)?;
                            crate::semantic::static_value::validate_decimal(decimal)?;
                        }
                        FieldKeyV1::Bool { .. } => {}
                    }
                    self.local(&field.ty, scope, depth + 1)?;
                }
            }
            CallableLocalTypeV1::Callable { parameters, return_type } => {
                for parameter in parameters {
                    if parameter.rest > 3 {
                        return Err(invalid("callable local parameter rest mode is invalid"));
                    }
                    if let Some(label) = &parameter.label {
                        self.text(label)?;
                    }
                    self.local(&parameter.ty, scope, depth + 1)?;
                }
                self.local(return_type, scope, depth + 1)?;
            }
        }
        Ok(())
    }
    fn constraint(&mut self, constraint: &CallableLocalConstraintV1, scope: Option<u32>, depth: usize) -> Result<(), ArtifactError> {
        self.row(depth)?;
        let (left, right) = match constraint {
            CallableLocalConstraintV1::Subtype { lower, upper } => (lower, upper),
            CallableLocalConstraintV1::Equivalent { left, right } => (left, right),
        };
        self.local(left, scope, depth + 1)?;
        self.local(right, scope, depth + 1)
    }
    fn proof(&mut self, proof: &CallableBranchProofV1, scope: Option<u32>, depth: usize) -> Result<(), ArtifactError> {
        self.row(depth)?;
        if proof.bindings.windows(2).any(|pair| pair[0].parameter >= pair[1].parameter)
            || proof.local_bindings.windows(2).any(|pair| pair[0].parameter >= pair[1].parameter)
        {
            return Err(invalid("callable proof bindings are not canonical"));
        }
        for binding in &proof.bindings {
            self.row(depth)?;
            self.parameter(binding.parameter)?;
            self.ty(binding.ty)?;
        }
        for equality in &proof.equalities {
            self.row(depth)?;
            match equality {
                CallableProofConstraintV1::Equal { left, right } => {
                    self.ty(*left)?;
                    self.ty(*right)?;
                }
                CallableProofConstraintV1::Subtype { lower, upper } => {
                    self.ty(*lower)?;
                    self.ty(*upper)?;
                }
                CallableProofConstraintV1::HasMember { ty, selector } => {
                    self.ty(*ty)?;
                    validate_static_case_selector(selector, self.limits)?;
                }
            }
        }
        for binding in &proof.local_bindings {
            self.row(depth)?;
            self.parameter(binding.parameter)?;
            self.local(&binding.ty, scope, depth + 1)?;
        }
        for constraint in &proof.local_equalities {
            self.constraint(constraint, scope, depth + 1)?;
        }
        Ok(())
    }
    fn bindings(&mut self, bindings: &[CallablePatternBindingV1]) -> Result<(), ArtifactError> {
        for (index, binding) in bindings.iter().enumerate() {
            self.row(0)?;
            if binding.binding as usize >= self.record.bindings.len()
                || bindings[..index].iter().any(|other| other.binding == binding.binding)
                || binding.source.start > binding.source.end
            {
                return Err(invalid("callable pattern binding is absent, duplicated, or has an inverted range"));
            }
            self.text(&binding.name)?;
            self.knowledge(&binding.knowledge)?;
            if let Some(term) = &binding.local_type {
                self.local(term, None, 0)?;
            }
        }
        Ok(())
    }
    fn pattern(&mut self, pattern: &CallablePatternV1, depth: usize) -> Result<(), ArtifactError> {
        self.row(depth)?;
        match pattern {
            CallablePatternV1::Wildcard => {}
            CallablePatternV1::Char { value } => {
                if char::from_u32(*value).is_none() {
                    return Err(invalid("callable pattern character is invalid"));
                }
            }
            CallablePatternV1::Binding { binding, name, knowledge } => {
                if *binding as usize >= self.record.bindings.len() {
                    return Err(invalid("callable pattern binding references another owner"));
                }
                self.text(name)?;
                self.knowledge(knowledge)?;
            }
            CallablePatternV1::Variant {
                owner,
                family,
                owner_candidates,
                selector,
                candidates,
                blocked,
            } => {
                if let Some(world) = self.world {
                    for owner in owner.iter().chain(owner_candidates) {
                        if !world
                            .structural_contracts
                            .declaration_shells
                            .iter()
                            .any(|declaration| &declaration.declaration == owner)
                        {
                            return Err(invalid("callable pattern declaration owner is absent"));
                        }
                    }
                }
                if family
                    .as_ref()
                    .is_some_and(|family| owner.as_ref() != Some(&family.owner) || !matches!(&family.selector, TypeMemberSelectorV1::Named { .. }))
                {
                    return Err(invalid("callable pattern family belongs to another declaration or has an invalid selector"));
                }
                for _ in owner_candidates {
                    self.row(depth)?;
                }
                match selector {
                    CallableVariantSelectorV1::Exact { selector } => validate_static_case_selector(selector, self.limits)?,
                    CallableVariantSelectorV1::Pattern { pattern } => {
                        validate_selector_base(&pattern.base, self.limits)?;
                        validate_selector_slots(&pattern.prefix, self.limits)?;
                        validate_selector_slots(&pattern.suffix, self.limits)?;
                    }
                    CallableVariantSelectorV1::WholeFamily => {}
                }
                if let Some(reason) = blocked {
                    self.reason(reason)?;
                }
                for (index, candidate) in candidates.iter().enumerate() {
                    self.row(depth)?;
                    if candidates[..index].iter().any(|other| other.variant == candidate.variant) {
                        return Err(invalid("callable pattern variant candidate is duplicated"));
                    }
                    self.exact_case(&candidate.variant, candidate.exact_case)?;
                    if owner.as_ref().is_some_and(|owner| owner != &candidate.variant.owner) {
                        return Err(invalid("callable pattern candidate belongs to another owner"));
                    }
                    let scope = candidate.case_instantiation.as_ref().map(|case| case.scope);
                    if let Some(case) = &candidate.case_instantiation {
                        self.row(depth)?;
                        self.scope(case.scope)?;
                        if case.variant != candidate.variant || case.local_rigids.windows(2).any(|pair| pair[0].parameter >= pair[1].parameter) {
                            return Err(invalid("callable case instantiation owner or binder order is invalid"));
                        }
                        for binder in &case.local_rigids {
                            self.row(depth)?;
                            self.parameter(binder.parameter)?;
                            self.rigid(binder.rigid, Some(case.scope))?;
                            let rigid = &self.record.local_proof_graph.as_ref().expect("scope validated").rigids[binder.rigid as usize];
                            if rigid.scope != case.scope
                                || !matches!(&rigid.origin, CallableRigidOriginV1::VariantParameter { variant, parameter } if variant == &case.variant && parameter == &binder.parameter)
                            {
                                return Err(invalid("callable case binder disagrees with its rigid definition"));
                            }
                        }
                        for payload in &case.payload_types {
                            self.local(payload, scope, depth + 1)?;
                        }
                        self.local(&case.result_type, scope, depth + 1)?;
                        for constraint in &case.constraints {
                            self.constraint(constraint, scope, depth + 1)?;
                        }
                    }
                    self.proof(&candidate.proof, scope, depth + 1)?;
                    for (index, field) in candidate.fields.iter().enumerate() {
                        self.row(depth)?;
                        if field.variant != candidate.variant || candidate.fields[..index].iter().any(|other| other.field == field.field) {
                            return Err(invalid("callable field pattern owner or ordinal is invalid"));
                        }
                        if let Some(world) = self.world {
                            if !world
                                .structural_contracts
                                .variant_contracts
                                .iter()
                                .any(|variant| variant.variant == field.variant && (field.field as usize) < variant.fields.len())
                            {
                                return Err(invalid("callable field pattern is absent from its variant"));
                            }
                        }
                        self.knowledge(&field.field_type)?;
                        if let Some(term) = &field.local_type {
                            self.local(term, scope, depth + 1)?;
                        }
                        self.pattern(&field.child, depth + 1)?;
                    }
                }
            }
            CallablePatternV1::Or { alternatives } | CallablePatternV1::Tuple { elements: alternatives } => {
                for pattern in alternatives {
                    self.pattern(pattern, depth + 1)?;
                }
            }
            CallablePatternV1::List { prefix, rest } => {
                for pattern in prefix {
                    self.pattern(pattern, depth + 1)?;
                }
                if let Some(pattern) = rest {
                    self.pattern(pattern, depth + 1)?;
                }
            }
            CallablePatternV1::Record { fields } => {
                for field in fields {
                    self.text(&field.label)?;
                    self.pattern(&field.child, depth + 1)?;
                }
            }
            CallablePatternV1::Map { entries } => {
                for entry in entries {
                    match &entry.key {
                        CallableMapPatternKeyV1::Symbol { value } | CallableMapPatternKeyV1::String { value } => self.text(value)?,
                        CallableMapPatternKeyV1::Int { digits, radix } => {
                            self.text(digits)?;
                            if !(2..=36).contains(radix) || digits.is_empty() {
                                return Err(invalid("callable map pattern integer is invalid"));
                            }
                        }
                    }
                    self.pattern(&entry.child, depth + 1)?;
                }
            }
        }
        Ok(())
    }
    fn space(&mut self, space: &CallablePatternSpaceV1, depth: usize) -> Result<(), ArtifactError> {
        self.row(depth)?;
        match space {
            CallablePatternSpaceV1::Empty | CallablePatternSpaceV1::List => {}
            CallablePatternSpaceV1::Char { value } => {
                if char::from_u32(*value).is_none() {
                    return Err(invalid("callable pattern-space character is invalid"));
                }
            }
            CallablePatternSpaceV1::Blocked { reason } => self.reason(reason)?,
            CallablePatternSpaceV1::Opaque { ty } => self.ty(*ty)?,
            CallablePatternSpaceV1::Union { members } | CallablePatternSpaceV1::Tuple { elements: members } => {
                for space in members {
                    self.space(space, depth + 1)?;
                }
            }
            CallablePatternSpaceV1::Variant { variant, exact_case, fields } => {
                self.exact_case(variant, *exact_case)?;
                for space in fields {
                    self.space(space, depth + 1)?;
                }
            }
        }
        Ok(())
    }
    fn witness(&mut self, witness: &CallableCoverageWitnessV1, depth: usize) -> Result<(), ArtifactError> {
        self.row(depth)?;
        match witness {
            CallableCoverageWitnessV1::Wildcard => {}
            CallableCoverageWitnessV1::Char { value } => {
                if char::from_u32(*value).is_none() {
                    return Err(invalid("callable coverage character is invalid"));
                }
            }
            CallableCoverageWitnessV1::Opaque { ty } => self.ty(*ty)?,
            CallableCoverageWitnessV1::Tuple { elements } | CallableCoverageWitnessV1::List { elements } => {
                for witness in elements {
                    self.witness(witness, depth + 1)?;
                }
            }
            CallableCoverageWitnessV1::Variant { variant, exact_case, fields } => {
                self.exact_case(variant, *exact_case)?;
                for witness in fields {
                    self.witness(witness, depth + 1)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callable_proof_local_term_codec_preserves_nested_shapes_and_keys() {
        let leaf = CallableLocalTypeV1::Rigid { rigid: 2 };
        let term = CallableLocalTypeV1::Callable {
            parameters: vec![CallableLocalParameterV1 {
                label: Some("arg".into()),
                rest: 3,
                ty: CallableLocalTypeV1::Applied {
                    origin: Box::new(CallableLocalTypeV1::Canonical { ty: ArtifactTypeId(19) }),
                    arguments: vec![leaf.clone()],
                },
            }],
            return_type: Box::new(CallableLocalTypeV1::Tuple {
                elements: vec![CallableLocalTupleElementV1 {
                    label: None,
                    ty: CallableLocalTypeV1::Record {
                        fields: vec![
                            CallableLocalRecordFieldV1 {
                                key: FieldKeyV1::Symbol { value: "name".into() },
                                ty: leaf.clone(),
                            },
                            CallableLocalRecordFieldV1 {
                                key: FieldKeyV1::Int {
                                    decimal: "123456789012345678901234567890".into(),
                                },
                                ty: CallableLocalTypeV1::Union {
                                    members: vec![leaf, CallableLocalTypeV1::Canonical { ty: ArtifactTypeId(23) }],
                                },
                            },
                        ],
                    },
                }],
            }),
        };
        let mut writer = BinaryWriter::new();
        term.write(&mut writer).unwrap();
        let bytes = writer.into_vec();
        let mut budget = AllocationBudget::new(1 << 20);
        let mut decode = Decode {
            reader: BinaryReader::new(&bytes),
            limits: ValidationLimits::default(),
            allocations: &mut budget,
            items_left: 1000,
            depth: 0,
        };
        let restored = CallableLocalTypeV1::read(&mut decode).unwrap();
        assert_eq!(restored, term);
        assert_eq!(decode.reader.remaining(), 0);
        let mut writer = BinaryWriter::new();
        restored.write(&mut writer).unwrap();
        assert_eq!(writer.into_vec(), bytes);
        let mut short = AllocationBudget::new(budget.requested() - 1);
        let mut decode = Decode {
            reader: BinaryReader::new(&bytes),
            limits: ValidationLimits::default(),
            allocations: &mut short,
            items_left: 1000,
            depth: 0,
        };
        assert!(CallableLocalTypeV1::read(&mut decode).is_err());
    }
}
