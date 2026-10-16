//! Stable primary-owner roots collected before either relocation graph closes.
use super::export::register_structural_primary_roots;
use crate::declaration_type::{DeclaredTypeFact, DeclaredTypeState};
use crate::metadata::stable_identity::{to_stable_callable, to_stable_declaration, to_stable_impl, to_stable_module, to_stable_variant};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::StaticExportContext;
use crate::types::parameter::{GenericConstraint, GenericSignature, TypeTerm};
use crate::types::store::artifact::TypeExportContext;
use phalcom_compiler_artifact::semantic::type_store::{CanonicalRootFamilyTag as Family, CanonicalTypeRootKey, StableOwnerRef};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use std::collections::BTreeMap;

struct Roots<'a> {
    types: &'a mut TypeExportContext,
    owner: StableOwnerRef,
    family: Family,
    ordinal: u32,
    store: Option<&'a crate::types::store::TypeStore>,
}

pub(super) fn register_callable_signature_roots(
    types: &mut TypeExportContext,
    signature: &crate::signature::CallableSemanticSignature,
) -> Result<(), ArtifactError> {
    let mut roots = Roots::new(
        types,
        StableOwnerRef::Callable(to_stable_callable(&signature.callable)),
        Family::CallableSignature,
    );
    roots.callable_signature(signature)
}

pub(super) fn register_callable_definition_roots(
    types: &mut TypeExportContext,
    definition: &crate::impls::EffectiveCallableDefinition,
) -> Result<(), ArtifactError> {
    if definition.callable != definition.signature.callable {
        return Err(ArtifactError::LimitExceeded("callable definition key disagrees with its signature".into()));
    }
    let mut roots = Roots::new(
        types,
        StableOwnerRef::Callable(to_stable_callable(&definition.callable)),
        Family::CallableSignature,
    );
    roots.callable_signature(&definition.signature)?;
    if let Some(context) = &definition.inherent_body_context {
        roots.generic(&context.generic_signature)?;
        roots.ty(context.source_target_head)?;
        if let Some(covering) = &context.covering {
            let mut forward = covering.impl_to_decl.iter().collect::<Vec<_>>();
            forward.sort_unstable_by_key(|(parameter, _)| **parameter);
            for (&implementation, &declaration) in forward {
                roots.parameter(implementation)?;
                roots.parameter(declaration)?;
            }
            let mut inverse = covering.decl_to_impl.iter().collect::<Vec<_>>();
            inverse.sort_unstable_by_key(|(parameter, _)| **parameter);
            for (&declaration, &implementation) in inverse {
                roots.parameter(declaration)?;
                roots.parameter(implementation)?;
            }
        }
    }
    Ok(())
}

/// Registers type and row roots carried by already-selected call specializations.
/// Dynamic arguments intentionally add no proper-type root.
pub(super) fn register_callable_analysis_roots(
    types: &mut TypeExportContext,
    analysis: &crate::checker::analysis::CallableAnalysis,
    store: &crate::types::store::TypeStore,
) -> Result<(), ArtifactError> {
    let mut roots = Roots::new(
        types,
        StableOwnerRef::Callable(to_stable_callable(&analysis.callable)),
        Family::CallableAnalysis,
    );
    roots.store = Some(store);
    for expression in analysis.expressions.values() {
        roots.knowledge(&expression.knowledge)?;
        roots.analysis_status(&expression.status)?;
        if let Some(denotation) = &expression.denotation {
            roots.denotation(denotation)?;
        }
        if let Some(specialization) = &expression.call_specialization {
            for (parameter, argument) in specialization.bindings.iter() {
                let key = roots.key()?;
                let lineage = roots.types.expected_lineage();
                roots.types.register_parameter_root(lineage, key, *parameter)?;
                match argument {
                    crate::types::store::StaticArgument::Term(term) => roots.ty(*term)?,
                    crate::types::store::StaticArgument::RecordRow(row) => {
                        let key = roots.key()?;
                        let lineage = roots.types.expected_lineage();
                        roots.types.register_row_root(lineage, key, *row)?;
                    }
                    crate::types::store::StaticArgument::DynamicType => {}
                }
            }
            for conformance in specialization.conformances.iter() {
                roots.generic_call_conformance_binding(conformance)?;
            }
        }
        if let Some(capability) = &expression.conditional_capability {
            roots.inherent_impl_specialization(&capability.applicability)?;
        }
        if let Some(site) = &expression.trait_dispatch {
            roots.trait_dispatch_site(site)?;
        }
        if let Some(candidates) = &expression.trait_dispatch_candidates {
            for candidate in candidates.iter() {
                roots.trait_dispatch_candidate(candidate)?;
            }
        }
        if let Some(realization) = &expression.contextual_realization {
            roots.contextual_realization(realization)?;
        }
    }
    for binding in analysis.bindings.values() {
        if let Some(contract) = &binding.contract {
            roots.ty(contract.ty)?;
        }
        roots.knowledge(&binding.current)?;
        roots.binding_consistency(&binding.consistency)?;
        if let Some(denotation) = &binding.denotation {
            roots.denotation(denotation)?;
        }
    }
    roots.flow_state(&analysis.entry_flow)?;
    for exit in &analysis.exits.normal_returns {
        roots.knowledge(&exit.knowledge)?;
        roots.analysis_status(&exit.status)?;
        roots.flow_state(&exit.flow)?;
    }
    for exit in &analysis.exits.throws {
        roots.flow_state(exit)?;
    }
    for expansion in analysis.known_invalid_positional_expansions.values() {
        roots.ty(expansion.source_type)?;
    }
    for literal in analysis.sequence_literals.values() {
        roots.ty(literal.result_type)?;
        roots.ty(literal.element_type)?;
        if let Some(extent) = literal.extent {
            roots.ty(extent)?;
        }
        if let crate::checker::analysis::SequenceConstructionPlan::VectorIdentitySpread { extent, .. } = literal.construction {
            roots.ty(extent)?;
        }
        for expansion in literal.exact_expansions.values() {
            match expansion {
                crate::checker::analysis::SequenceExpansionResolution::TuplePositionals { extent }
                | crate::checker::analysis::SequenceExpansionResolution::VectorStorage { extent } => roots.ty(*extent)?,
            }
        }
    }
    for lane in analysis.for_iteration_lanes.values() {
        roots.for_iteration_lane(lane)?;
    }
    for expansion in analysis.positional_expansions.values() {
        roots.positional_expansion(expansion)?;
    }
    for resolution in analysis.associated_resolutions.values() {
        roots.associated_resolution(resolution)?;
    }
    for resolution in analysis.callable_reference_resolutions.values() {
        roots.callable_reference_resolution(resolution)?;
    }
    for resolution in analysis.family_applications.values() {
        roots.family_application_resolution(resolution)?;
    }
    for resolution in analysis.match_resolutions.values() {
        roots.match_resolution(resolution)?;
    }
    for resolution in analysis.pattern_site_resolutions.values() {
        roots.pattern_site_resolution(resolution)?;
    }
    for diagnostic in analysis.diagnostics.iter() {
        for guidance in &diagnostic.guidance {
            roots.diagnostic_guidance(guidance)?;
        }
    }
    for dependency in analysis.semantic_dependencies.iter() {
        if let crate::checker::analysis::SemanticDependency::GenericConformance(key) = dependency {
            roots.generic_conformance_key(key)?;
        }
    }
    for (_, node) in analysis.explanations.iter() {
        roots.explanation_step(&node.step)?;
        if let crate::explain::node::DerivationRule::GenericInstantiation { type_args } = &node.rule {
            for &ty in type_args {
                roots.ty(ty)?;
            }
        }
        for evidence in &node.evidence {
            if let crate::explain::node::EvidenceRef::TypeId(ty) = evidence {
                roots.ty(*ty)?;
            }
        }
    }
    Ok(())
}

impl<'a> Roots<'a> {
    fn new(types: &'a mut TypeExportContext, owner: StableOwnerRef, family: Family) -> Self {
        Self::new_with_ordinal(types, owner, family, 100)
    }
    fn new_with_ordinal(types: &'a mut TypeExportContext, owner: StableOwnerRef, family: Family, ordinal: u32) -> Self {
        Self {
            types,
            owner,
            family,
            ordinal,
            store: None,
        }
    }
    fn key(&mut self) -> Result<CanonicalTypeRootKey, ArtifactError> {
        let ordinal = self.ordinal;
        self.ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
        Ok(CanonicalTypeRootKey {
            family_tag: self.family,
            owner: self.owner.clone(),
            product_subkey: ordinal,
        })
    }
    fn next_ordinal(&self) -> u32 {
        self.ordinal
    }
    fn ty(&mut self, ty: crate::types::id::TypeId) -> Result<(), ArtifactError> {
        let key = self.key()?;
        self.types.register_type_root(self.types.expected_lineage(), key, ty)?;
        Ok(())
    }
    fn kind(&mut self, kind: crate::types::id::KindId) -> Result<(), ArtifactError> {
        let key = self.key()?;
        self.types.register_kind_root(self.types.expected_lineage(), key, kind)?;
        Ok(())
    }
    fn domain(&mut self, domain: crate::types::id::StaticDomainId) -> Result<(), ArtifactError> {
        let key = self.key()?;
        self.types.register_domain_root(self.types.expected_lineage(), key, domain)?;
        Ok(())
    }
    fn knowledge(&mut self, knowledge: &crate::types::evidence::TypeKnowledge) -> Result<(), ArtifactError> {
        if let crate::types::evidence::TypeKnowledge::Known(evidence) = knowledge {
            self.ty(evidence.ty())?;
        }
        Ok(())
    }
    fn flow_state(&mut self, flow: &crate::checker::analysis::FlowStateSummary) -> Result<(), ArtifactError> {
        for binding in flow.bindings.values() {
            self.knowledge(&binding.knowledge)?;
            self.binding_consistency(&binding.consistency)?;
            if let Some(contract) = &binding.contract {
                self.ty(contract.ty)?;
            }
        }
        for field in flow.fields.values() {
            self.knowledge(&field.contract)?;
            self.knowledge(&field.current)?;
            if let crate::checker::flow::FieldContractValidity::Blocked(reason) = &field.validity {
                self.block_reason(reason)?;
            }
        }
        Ok(())
    }
    fn binding_consistency(&mut self, consistency: &crate::checker::binding::BindingConsistency) -> Result<(), ArtifactError> {
        if let crate::checker::binding::BindingConsistency::Blocked(reason) = consistency {
            self.block_reason(reason)?;
        }
        Ok(())
    }
    fn binding_state(&mut self, state: &crate::checker::analysis::BindingState) -> Result<(), ArtifactError> {
        if let Some(contract) = &state.contract {
            self.ty(contract.ty)?;
        }
        self.knowledge(&state.current)?;
        self.binding_consistency(&state.consistency)?;
        if let Some(denotation) = &state.denotation {
            self.denotation(denotation)?;
        }
        Ok(())
    }
    fn analysis_status(&mut self, status: &crate::checker::analysis::AnalysisStatus) -> Result<(), ArtifactError> {
        if let crate::checker::analysis::AnalysisStatus::Blocked(reason) = status {
            self.block_reason(reason)?;
        }
        Ok(())
    }
    fn block_reason(&mut self, reason: &crate::types::outcome::BlockReason) -> Result<(), ArtifactError> {
        if let crate::types::outcome::BlockReason::UnavailableSemanticOwner(owner) = reason {
            match owner {
                crate::db::QueryKey::CallableSignature(_) | crate::db::QueryKey::FieldSignature(_) => {}
                crate::db::QueryKey::GenericConformance(key) => self.generic_conformance_key(key)?,
                crate::db::QueryKey::TraitDispatchBucket(key) => {
                    if let crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant) = &key.target_family {
                        let variant_type = self.store.and_then(|store| store.variant_type_id(variant)).ok_or_else(|| {
                            ValidationError::ReferentialIntegrity("callable trait-dispatch query owner references an uninterned exact-case identity".into())
                        })?;
                        let root = self.key()?;
                        self.types.register_variant_root(self.types.expected_lineage(), root, variant_type)?;
                    }
                }
                unsupported => {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "callable block reason contains unsupported query owner kind {}",
                        unsupported.kind().as_str()
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }
    fn parameter(&mut self, parameter: crate::types::id::TypeParameterId) -> Result<(), ArtifactError> {
        let key = self.key()?;
        self.types.register_parameter_root(self.types.expected_lineage(), key, parameter)?;
        Ok(())
    }
    fn record_row(&mut self, row: crate::types::id::RecordRowId) -> Result<(), ArtifactError> {
        let key = self.key()?;
        self.types.register_row_root(self.types.expected_lineage(), key, row)?;
        Ok(())
    }
    fn trait_ref(&mut self, trait_ref: &crate::traits::TraitRef) -> Result<(), ArtifactError> {
        for &argument in &trait_ref.arguments {
            self.ty(argument)?;
        }
        Ok(())
    }
    fn generic_conformance_key(&mut self, key: &crate::generic_evidence::GenericConformanceKey) -> Result<(), ArtifactError> {
        self.term(&key.subject)?;
        self.trait_ref(&key.trait_ref)
    }
    fn semantic_dependency(&mut self, dependency: &crate::checker::analysis::SemanticDependency) -> Result<(), ArtifactError> {
        use crate::checker::analysis::SemanticDependency;
        match dependency {
            SemanticDependency::GenericConformance(key) => self.generic_conformance_key(key),
            SemanticDependency::TraitDispatchBucket(bucket) => {
                if let crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant) = &bucket.target_family {
                    let variant_type = self
                        .store
                        .and_then(|store| store.variant_type_id(variant))
                        .ok_or_else(|| ValidationError::ReferentialIntegrity("semantic dependency references an uninterned exact enum-case identity".into()))?;
                    let key = self.key()?;
                    self.types.register_variant_root(self.types.expected_lineage(), key, variant_type)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn generic_call_conformance_binding(&mut self, binding: &crate::checker::analysis::GenericCallConformanceBinding) -> Result<(), ArtifactError> {
        use crate::checker::analysis::GenericCallConformanceSource;
        self.generic_conformance_key(&binding.callee_conformance)?;
        match &binding.source {
            GenericCallConformanceSource::Exact { evidence } => {
                self.generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::Exact(evidence.clone()))
            }
            GenericCallConformanceSource::Forwarded { caller_conformance } => self.generic_conformance_key(caller_conformance),
            GenericCallConformanceSource::Derived { evidence } => {
                self.generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::Derived(evidence.clone()))
            }
            GenericCallConformanceSource::CompilerDerived { evidence } => {
                self.generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence.clone()))
            }
        }
    }
    fn type_environment(&mut self, environment: &crate::types::environment::TypeEnvironment) -> Result<(), ArtifactError> {
        let mut bindings = environment.bindings.iter().collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|(parameter, _)| **parameter);
        for (&parameter, &ty) in bindings {
            self.parameter(parameter)?;
            self.ty(ty)?;
        }
        let mut rows = environment.row_bindings.iter().collect::<Vec<_>>();
        rows.sort_unstable_by_key(|(parameter, _)| **parameter);
        for (&parameter, &row) in rows {
            self.parameter(parameter)?;
            self.record_row(row)?;
        }
        let mut dynamic = environment.dynamic_bindings.iter().copied().collect::<Vec<_>>();
        dynamic.sort_unstable();
        for parameter in dynamic {
            self.parameter(parameter)?;
        }
        if let Some(ty) = environment.self_binding {
            self.ty(ty)?;
        }
        Ok(())
    }
    fn type_substitution(&mut self, substitution: &crate::types::substitution::TypeSubstitution) -> Result<(), ArtifactError> {
        let mut bindings = substitution.bindings().iter().collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|(parameter, _)| **parameter);
        for (&parameter, &ty) in bindings {
            self.parameter(parameter)?;
            self.ty(ty)?;
        }
        Ok(())
    }
    fn for_iteration_requirement(&mut self, requirement: &crate::checker::analysis::ForIterationRequirementResolution) -> Result<(), ArtifactError> {
        use crate::checker::analysis::ForIterationRequirementResolution;
        match requirement {
            ForIterationRequirementResolution::Evidence {
                conformance,
                evidence,
                signature,
                ..
            } => {
                self.generic_conformance_key(conformance)?;
                self.generic_conformance_evidence(evidence)?;
                self.callable_signature(signature)
            }
            ForIterationRequirementResolution::AbstractRequirement { signature, .. } => self.callable_signature(signature),
        }
    }
    fn for_iteration_lane(&mut self, lane: &crate::checker::analysis::ForIterationLaneResolution) -> Result<(), ArtifactError> {
        self.ty(lane.source_type)?;
        self.ty(lane.item_type)?;
        self.ty(lane.iterator_type)?;
        self.for_iteration_requirement(&lane.iterator)?;
        self.for_iteration_requirement(&lane.next)
    }
    fn checked_static_application(
        &mut self,
        owner: &crate::static_definition::StaticBodyId,
        application: &crate::static_definition::CheckedStaticApplication,
    ) -> Result<(), ArtifactError> {
        if &application.id.body != owner {
            return Err(ValidationError::ReferentialIntegrity("checked static-application row belongs to a different body".into()).into());
        }
        self.static_definition_signature(&application.signature)?;
        for argument in application.arguments.iter() {
            if let crate::static_definition::CheckedStaticApplicationArgument::Kinded { term, .. } = argument {
                self.ty(*term)?;
            }
        }
        Ok(())
    }
    fn static_definition_signature(&mut self, signature: &crate::static_definition::StaticDefinitionSignature) -> Result<(), ArtifactError> {
        for parameter in signature.parameters.iter() {
            if let Some(parameter) = parameter.type_parameter_id {
                self.parameter(parameter)?;
            }
            self.static_value_domain(parameter.domain)?;
        }
        self.static_value_domain(signature.result_domain)?;
        self.generic(&signature.generic_signature)
    }
    fn static_value_domain(&mut self, domain: crate::static_domain::StaticValueDomain) -> Result<(), ArtifactError> {
        match domain {
            crate::static_domain::StaticValueDomain::Kind(kind) => self.kind(kind),
            crate::static_domain::StaticValueDomain::ValueType(ty) => self.ty(ty),
        }
    }
    fn trait_dispatch_site(&mut self, site: &crate::trait_dispatch::TraitDispatchSite) -> Result<(), ArtifactError> {
        use crate::trait_dispatch::TraitDispatchSite;
        match site {
            TraitDispatchSite::AbstractRequirement { .. } => Ok(()),
            TraitDispatchSite::Evidenced(selection) => self.trait_dispatch_selection(selection),
            TraitDispatchSite::Assumed(selection) => {
                self.generic_conformance_key(&selection.conformance)?;
                self.callable_signature(&selection.signature)
            }
        }
    }
    fn trait_dispatch_selection(&mut self, selection: &crate::trait_dispatch::TraitDispatchSelection) -> Result<(), ArtifactError> {
        self.ty(selection.exact_target)?;
        self.trait_ref(&selection.exact_trait_ref)?;
        self.generic_conformance_evidence(&selection.evidence)?;
        self.callable_signature(&selection.signature)?;
        self.requirement_selection(&selection.selection)?;
        for target in selection.requirement_targets.values() {
            self.requirement_selection(target)?;
        }
        Ok(())
    }
    fn trait_dispatch_candidate(&mut self, candidate: &crate::trait_dispatch::TraitEvidencedMemberCandidate) -> Result<(), ArtifactError> {
        self.ty(candidate.exact_target)?;
        self.trait_ref(&candidate.exact_trait_ref)?;
        self.generic_conformance_evidence(&candidate.evidence)?;
        self.callable_signature(&candidate.signature)?;
        self.requirement_selection(&candidate.selection)?;
        for target in candidate.requirement_targets.values() {
            self.requirement_selection(target)?;
        }
        Ok(())
    }
    fn contextual_realization(&mut self, realization: &crate::checker::analysis::ContextualRealization) -> Result<(), ArtifactError> {
        use crate::checker::analysis::{ContextualRealizationOperation, RealizationCoefficientSource};
        self.ty(realization.source_type)?;
        self.ty(realization.target_type)?;
        match &realization.operation {
            ContextualRealizationOperation::Basis { basis_type, coefficient, .. } => {
                self.ty(*basis_type)?;
                match coefficient {
                    RealizationCoefficientSource::One { scalar_type, one } => {
                        self.ty(*scalar_type)?;
                        self.realization_capability(one)?;
                    }
                    RealizationCoefficientSource::Expression { scalar_type, .. } => self.ty(*scalar_type)?,
                    RealizationCoefficientSource::IntegerLiteral { .. } | RealizationCoefficientSource::FloatLiteral { .. } => {}
                }
            }
            ContextualRealizationOperation::RealEmbedding { scalar_type, .. } => self.ty(*scalar_type)?,
        }
        self.realization_capability(&realization.realization)
    }
    fn realization_capability(&mut self, selection: &crate::checker::analysis::RealizationCapabilitySelection) -> Result<(), ArtifactError> {
        match selection {
            crate::checker::analysis::RealizationCapabilitySelection::Evidenced(selection) => self.trait_dispatch_selection(selection),
            crate::checker::analysis::RealizationCapabilitySelection::Assumed(selection) => {
                self.generic_conformance_key(&selection.conformance)?;
                self.callable_signature(&selection.signature)
            }
        }
    }
    fn match_resolution(&mut self, resolution: &crate::match_semantics::MatchResolution) -> Result<(), ArtifactError> {
        self.knowledge(&resolution.scrutinee)?;
        self.pattern_space(&resolution.initial_space)?;
        for arm in resolution.arms.iter() {
            self.pattern_resolution(&arm.pattern)?;
            self.pattern_space(&arm.reachable_space)?;
            self.pattern_space(&arm.residual_after)?;
            for binding in arm.bindings.iter() {
                self.pattern_binding(binding)?;
            }
            self.branch_proof(&arm.proof)?;
            self.knowledge(&arm.branch_result)?;
        }
        self.knowledge(&resolution.result)?;
        self.exhaustiveness(&resolution.exhaustiveness)
    }
    fn pattern_site_resolution(&mut self, resolution: &crate::match_semantics::PatternSiteResolution) -> Result<(), ArtifactError> {
        self.knowledge(&resolution.subject)?;
        self.pattern_resolution(&resolution.pattern)?;
        for binding in resolution.bindings.iter() {
            self.pattern_binding(binding)?;
        }
        Ok(())
    }
    fn pattern_resolution(&mut self, pattern: &crate::match_semantics::PatternResolution) -> Result<(), ArtifactError> {
        use crate::match_semantics::PatternResolution;
        match pattern {
            PatternResolution::Wildcard | PatternResolution::Char(_) => Ok(()),
            PatternResolution::Binding { knowledge, .. } => self.knowledge(knowledge),
            PatternResolution::Variant(variant) => {
                if let Some(reason) = &variant.blocked {
                    self.block_reason(reason)?;
                }
                for candidate in variant.candidates.iter() {
                    self.ty(candidate.exact_case)?;
                    self.branch_proof(&candidate.proof)?;
                    if let Some(instantiation) = &candidate.case_instantiation {
                        self.case_instantiation(instantiation)?;
                    }
                    for field in candidate.fields.iter() {
                        self.knowledge(&field.field_type)?;
                        if let Some(local_type) = &field.local_type {
                            self.local_type(local_type)?;
                        }
                        self.pattern_resolution(&field.child)?;
                    }
                }
                Ok(())
            }
            PatternResolution::Or(or_pattern) => {
                for alternative in or_pattern.alternatives.iter() {
                    self.pattern_resolution(alternative)?;
                }
                Ok(())
            }
            PatternResolution::Tuple(items) => {
                for item in items.iter() {
                    self.pattern_resolution(item)?;
                }
                Ok(())
            }
            PatternResolution::List(list) => {
                for item in list.prefix.iter() {
                    self.pattern_resolution(item)?;
                }
                if let Some(rest) = &list.rest {
                    self.pattern_resolution(rest)?;
                }
                Ok(())
            }
            PatternResolution::Record(fields) => {
                for field in fields.iter() {
                    self.pattern_resolution(&field.child)?;
                }
                Ok(())
            }
            PatternResolution::Map(entries) => {
                for entry in entries.iter() {
                    self.pattern_resolution(&entry.child)?;
                }
                Ok(())
            }
        }
    }
    fn pattern_binding(&mut self, binding: &crate::match_semantics::PatternBindingResolution) -> Result<(), ArtifactError> {
        self.knowledge(&binding.knowledge)?;
        if let Some(local_type) = &binding.local_type {
            self.local_type(local_type)?;
        }
        Ok(())
    }
    fn branch_proof(&mut self, proof: &crate::match_semantics::BranchProofEnvironment) -> Result<(), ArtifactError> {
        for (&parameter, &ty) in &proof.bindings {
            self.parameter(parameter)?;
            self.ty(ty)?;
        }
        for constraint in proof.equalities.iter() {
            self.type_constraint(constraint)?;
        }
        for (&parameter, local_type) in &proof.local_bindings {
            self.parameter(parameter)?;
            self.local_type(local_type)?;
        }
        for constraint in proof.local_equalities.iter() {
            self.local_constraint(constraint)?;
        }
        Ok(())
    }
    fn case_instantiation(&mut self, instantiation: &crate::types::CaseInstantiation) -> Result<(), ArtifactError> {
        for &parameter in instantiation.local_rigids.keys() {
            self.parameter(parameter)?;
        }
        for payload in instantiation.payload_types.iter() {
            self.local_type(payload)?;
        }
        self.local_type(&instantiation.result_type)?;
        for constraint in instantiation.constraints.iter() {
            self.local_constraint(constraint)?;
        }
        Ok(())
    }
    fn local_constraint(&mut self, constraint: &crate::types::rigid::LocalConstraint) -> Result<(), ArtifactError> {
        use crate::types::rigid::LocalConstraint;
        match constraint {
            LocalConstraint::Subtype { lower, upper } => {
                self.local_type(lower)?;
                self.local_type(upper)
            }
            LocalConstraint::Equivalent { left, right } => {
                self.local_type(left)?;
                self.local_type(right)
            }
        }
    }
    fn type_constraint(&mut self, constraint: &crate::types::constraint::TypeConstraint) -> Result<(), ArtifactError> {
        use crate::types::constraint::TypeConstraint;
        match constraint {
            TypeConstraint::Equal(left, right) | TypeConstraint::Subtype(left, right) => {
                self.ty(*left)?;
                self.ty(*right)
            }
            TypeConstraint::HasMember(ty, _) => self.ty(*ty),
        }
    }
    fn local_type(&mut self, local_type: &crate::types::rigid::LocalType) -> Result<(), ArtifactError> {
        use crate::types::rigid::LocalType;
        match local_type {
            LocalType::Canonical(ty) => self.ty(*ty),
            LocalType::Rigid(_) => Ok(()),
            LocalType::Applied { origin, arguments } => {
                self.local_type(origin)?;
                for argument in arguments.iter() {
                    self.local_type(argument)?;
                }
                Ok(())
            }
            LocalType::ExactCase { enum_type, .. } => self.local_type(enum_type),
            LocalType::Union(members) => {
                for member in members.iter() {
                    self.local_type(member)?;
                }
                Ok(())
            }
            LocalType::Tuple(elements) => {
                for element in elements.iter() {
                    self.local_type(&element.ty)?;
                }
                Ok(())
            }
            LocalType::Record(fields) => {
                for field in fields.iter() {
                    self.local_type(&field.ty)?;
                }
                Ok(())
            }
            LocalType::Callable { parameters, return_type } => {
                for parameter in parameters.iter() {
                    self.local_type(&parameter.ty)?;
                }
                self.local_type(return_type)
            }
        }
    }
    fn pattern_space(&mut self, space: &crate::match_semantics::PatternSpaceSummary) -> Result<(), ArtifactError> {
        use crate::match_semantics::PatternSpaceSummary;
        match space {
            PatternSpaceSummary::Opaque(ty) => self.ty(*ty),
            PatternSpaceSummary::Union(spaces) => {
                for space in spaces.iter() {
                    self.pattern_space(space)?;
                }
                Ok(())
            }
            PatternSpaceSummary::Variant { exact_case, fields, .. } => {
                self.ty(*exact_case)?;
                for field in fields.iter() {
                    self.pattern_space(field)?;
                }
                Ok(())
            }
            PatternSpaceSummary::Tuple(items) => {
                for item in items.iter() {
                    self.pattern_space(item)?;
                }
                Ok(())
            }
            PatternSpaceSummary::Blocked(reason) => self.block_reason(reason),
            PatternSpaceSummary::Empty | PatternSpaceSummary::Char(_) | PatternSpaceSummary::List => Ok(()),
        }
    }
    fn exhaustiveness(&mut self, result: &crate::match_semantics::ExhaustivenessResult) -> Result<(), ArtifactError> {
        match result {
            crate::match_semantics::ExhaustivenessResult::Missing(witnesses) => {
                for witness in witnesses.iter() {
                    self.coverage_witness(witness)?;
                }
                Ok(())
            }
            crate::match_semantics::ExhaustivenessResult::Blocked(reason) => self.block_reason(reason),
            crate::match_semantics::ExhaustivenessResult::Proven => Ok(()),
        }
    }
    fn coverage_witness(&mut self, witness: &crate::match_semantics::CoverageWitness) -> Result<(), ArtifactError> {
        use crate::match_semantics::CoverageWitness;
        match witness {
            CoverageWitness::Opaque(ty) => self.ty(*ty),
            CoverageWitness::Variant { exact_case, fields, .. } => {
                self.ty(*exact_case)?;
                for field in fields.iter() {
                    self.coverage_witness(field)?;
                }
                Ok(())
            }
            CoverageWitness::Tuple(items) => {
                for item in items.iter() {
                    self.coverage_witness(item)?;
                }
                Ok(())
            }
            CoverageWitness::Char(_) | CoverageWitness::List(_) | CoverageWitness::Wildcard => Ok(()),
        }
    }
    fn explanation_step(&mut self, step: &crate::explain::node::ExplanationStep) -> Result<(), ArtifactError> {
        use crate::explain::node::{ExplanationStep, GenericConstraintRelation};
        match step {
            ExplanationStep::Literal { ty, .. } | ExplanationStep::Declared { ty, .. } | ExplanationStep::CallableReturn { ty, .. } => self.ty(*ty),
            ExplanationStep::GenericSolution { parameter, ty, .. } => {
                self.parameter(*parameter)?;
                self.ty(*ty)
            }
            ExplanationStep::ExpressionResult { knowledge, .. }
            | ExplanationStep::BindingRead { knowledge, .. }
            | ExplanationStep::ProductComponent { result: knowledge, .. } => self.knowledge(knowledge),
            ExplanationStep::BindingContract {
                actual, contract, consistency, ..
            } => {
                self.knowledge(actual)?;
                self.binding_consistency(consistency)?;
                self.ty(*contract)
            }
            ExplanationStep::TypeRequirement { expected, .. } => self.ty(*expected),
            ExplanationStep::TypeRelation { actual, expected, outcome } => {
                self.knowledge(actual)?;
                if let crate::types::outcome::RelationOutcome::Blocked(reason) = outcome {
                    self.block_reason(reason)?;
                }
                self.ty(*expected)
            }
            ExplanationStep::CallableSelection { receiver, .. } | ExplanationStep::UnionArm { receiver, .. } => self.ty(*receiver),
            ExplanationStep::SelfTypeSpecialization { self_ty, receiver, resolved } => {
                self.ty(*self_ty)?;
                self.ty(*receiver)?;
                self.ty(*resolved)
            }
            ExplanationStep::ArgumentCheck { actual, expected, .. } => {
                self.knowledge(actual)?;
                self.ty(*expected)
            }
            ExplanationStep::MethodCall { return_ty, .. } | ExplanationStep::UnresolvedCall { return_ty, .. } => self.ty(*return_ty),
            ExplanationStep::GenericConstraint { parameter, relation, .. } => {
                self.parameter(*parameter)?;
                match relation {
                    GenericConstraintRelation::Equal(ty)
                    | GenericConstraintRelation::SubtypeOf(ty)
                    | GenericConstraintRelation::SupertypeOf(ty)
                    | GenericConstraintRelation::AssignableTo(ty) => self.ty(*ty),
                }
            }
            ExplanationStep::GenericConflict { parameter, .. } => parameter.map_or(Ok(()), |parameter| self.parameter(parameter)),
            ExplanationStep::CollectionSynthesis { element_types, result, .. } => {
                for &ty in element_types.iter() {
                    self.ty(ty)?;
                }
                self.ty(*result)
            }
            ExplanationStep::FlowRefinement { prior, refined, .. } => {
                self.knowledge(prior)?;
                self.knowledge(refined)
            }
            ExplanationStep::BranchJoin { branches, joined, .. } => {
                for knowledge in branches.iter() {
                    self.knowledge(knowledge)?;
                }
                self.knowledge(joined)
            }
            ExplanationStep::LoopJoin { inputs, joined } => {
                for knowledge in inputs.iter() {
                    self.knowledge(knowledge)?;
                }
                self.knowledge(joined)
            }
            ExplanationStep::IterationElement { iterable, element, .. } => {
                self.knowledge(iterable)?;
                self.knowledge(element)
            }
            ExplanationStep::ReturnCheck { actual, expected } => {
                self.knowledge(actual)?;
                expected.map_or(Ok(()), |ty| self.ty(ty))
            }
            ExplanationStep::CallableReturnSummary { returns, result, .. } => {
                for knowledge in returns.iter() {
                    self.knowledge(knowledge)?;
                }
                self.knowledge(result)
            }
            ExplanationStep::Subtyping { actual, expected, .. } => {
                self.ty(*actual)?;
                self.ty(*expected)
            }
            ExplanationStep::CallShape { .. }
            | ExplanationStep::CallableKind { .. }
            | ExplanationStep::UnknownBoundary { .. }
            | ExplanationStep::DynamicBoundary { .. } => Ok(()),
            ExplanationStep::InternalBlocked { reason } => self.block_reason(reason),
        }
    }
    fn diagnostic_guidance(&mut self, guidance: &crate::diagnostic::DiagnosticGuidance) -> Result<(), ArtifactError> {
        use crate::diagnostic::DiagnosticGuidance;
        match guidance {
            DiagnosticGuidance::ChangeAnnotation { ty, .. } | DiagnosticGuidance::SupplyAssignableValue { expected: ty } => self.ty(*ty),
            DiagnosticGuidance::EstablishTypeEvidence { expected, .. } => expected.map_or(Ok(()), |ty| self.ty(ty)),
            DiagnosticGuidance::ResolveGenericParameter { parameter } => self.parameter(*parameter),
            DiagnosticGuidance::UseCallableShape { .. } => Ok(()),
        }
    }
    fn semantic_diagnostic(&mut self, diagnostic: &crate::diagnostic::SemanticDiagnostic) -> Result<(), ArtifactError> {
        for guidance in &diagnostic.guidance {
            self.diagnostic_guidance(guidance)?;
        }
        Ok(())
    }
    fn positional_expansion(&mut self, expansion: &crate::checker::analysis::PositionalExpansionIterationResolution) -> Result<(), ArtifactError> {
        self.ty(expansion.source_type)?;
        self.ty(expansion.item_type)?;
        self.ty(expansion.iterator_type)?;
        self.for_iteration_requirement(&expansion.iterator)?;
        self.for_iteration_requirement(&expansion.next)
    }
    fn generic_conformance_evidence(&mut self, evidence: &crate::generic_evidence::GenericConformanceEvidence) -> Result<(), ArtifactError> {
        use crate::generic_evidence::{DerivedConformanceOrigin, GenericConformanceEvidence, NestedGenericEvidence};
        match evidence {
            GenericConformanceEvidence::Exact(evidence) => {
                self.ty(evidence.exact_target)?;
                self.trait_ref(&evidence.exact_trait_ref)?;
                self.type_environment(&evidence.impl_environment)?;
                for requirement in evidence.requirement_views.values() {
                    self.callable_signature(&requirement.signature)?;
                }
                for selection in evidence.requirements.values() {
                    self.requirement_selection(selection)?;
                }
                for binding in evidence.associated_types.values() {
                    if let crate::impls::AssociatedTypeBindingValue::Type(ty) = binding.value {
                        self.ty(ty)?;
                    }
                }
                for constraint in &evidence.trait_header_contract {
                    self.constraint(constraint)?;
                }
                if let Some(proofs) = &evidence.trait_header_conformance_proofs {
                    for (key, evidence) in proofs {
                        self.generic_conformance_key(key)?;
                        self.generic_conformance_evidence(evidence)?;
                    }
                }
                Ok(())
            }
            GenericConformanceEvidence::Assumed(evidence) => self.generic_conformance_key(&evidence.key),
            GenericConformanceEvidence::Derived(evidence) => {
                match &evidence.origin {
                    DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                        self.generic_conformance_evidence(&GenericConformanceEvidence::Exact(exact_source.clone()))?;
                    }
                    DerivedConformanceOrigin::GuardedRule {
                        rule,
                        exact_subject,
                        exact_trait_ref,
                    } => {
                        self.ty(*exact_subject)?;
                        self.trait_ref(exact_trait_ref)?;
                        self.derived_conformance_rule(rule)?;
                    }
                }
                self.type_substitution(&evidence.impl_bindings)?;
                for nested in evidence.nested.iter() {
                    if let NestedGenericEvidence::Conformance(nested) = nested {
                        self.generic_conformance_evidence(nested)?;
                    }
                }
                Ok(())
            }
            GenericConformanceEvidence::CompilerDerived(evidence) => self.generic_conformance_key(&evidence.key),
        }
    }
    fn derived_conformance_rule(&mut self, rule: &crate::derived_conformance::DerivedConformanceRule) -> Result<(), ArtifactError> {
        self.parameter(rule.target_parameter)?;
        self.ty(rule.target_template)?;
        self.trait_ref(&rule.trait_ref)?;
        self.generic(&Some(rule.generic_signature.clone()))?;
        self.associated_type_plan(&rule.associated_type_plan)?;
        for requirement in rule.requirement_views.values() {
            self.callable_signature(&requirement.signature)?;
        }
        for selection in rule.requirements.values() {
            self.requirement_selection(selection)?;
        }
        Ok(())
    }
    fn associated_type_plan(&mut self, plan: &crate::impls::ConformanceAssociatedTypePlan) -> Result<(), ArtifactError> {
        for binding in plan.bindings.values() {
            if let crate::impls::AssociatedTypeBindingValue::Type(ty) = binding.value_template {
                self.ty(ty)?;
            }
        }
        for diagnostic in &plan.diagnostics {
            for guidance in &diagnostic.guidance {
                self.diagnostic_guidance(guidance)?;
            }
        }
        Ok(())
    }
    fn conformance_completeness(&mut self, completeness: &crate::impls::ConformanceCompleteness) -> Result<(), ArtifactError> {
        if let crate::impls::ConformanceCompleteness::Blocked(reason) = completeness {
            self.block_reason(reason)?;
        }
        Ok(())
    }
    fn requirement_selection(&mut self, selection: &crate::impls::RequirementSelectionTemplate) -> Result<(), ArtifactError> {
        use crate::impls::RequirementSelectionTemplate;
        match selection {
            RequirementSelectionTemplate::ConformanceCallable { .. } | RequirementSelectionTemplate::TraitDefault { .. } => Ok(()),
            RequirementSelectionTemplate::InherentCallable { applicability, .. } => applicability
                .as_ref()
                .map_or(Ok(()), |applicability| self.inherent_impl_specialization(applicability)),
            RequirementSelectionTemplate::ConditionalInherent { candidate, fallback, .. } => {
                self.effective_inherent_witness(candidate)?;
                fallback.as_ref().map_or(Ok(()), |fallback| self.requirement_selection(fallback))
            }
            RequirementSelectionTemplate::DataComponent { specialized_type, .. } => self.ty(*specialized_type),
        }
    }
    fn effective_inherent_witness(&mut self, witness: &crate::impls::EffectiveInherentWitness) -> Result<(), ArtifactError> {
        self.callable_signature(&witness.signature)?;
        if let Some(applicability) = &witness.applicability {
            self.inherent_impl_specialization(applicability)?;
        }
        if let Some(domain) = &witness.conditional_domain {
            self.inherent_impl_domain(domain)?;
        }
        Ok(())
    }
    fn inherent_impl_specialization(&mut self, specialization: &crate::impls::InherentImplSpecialization) -> Result<(), ArtifactError> {
        self.ty(specialization.receiver)?;
        self.ty(specialization.owner_view)?;
        self.type_environment(&specialization.environment)?;
        let mut bindings = specialization.bindings.iter().collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|(parameter, _)| **parameter);
        for (&parameter, &ty) in bindings {
            self.parameter(parameter)?;
            self.ty(ty)?;
        }
        if let Some(provenance) = &specialization.conformance_provenance {
            for binding in provenance.iter() {
                self.generic_call_conformance_binding(binding)?;
            }
        }
        Ok(())
    }
    fn inherent_impl_domain(&mut self, domain: &crate::impls::InherentImplDomain) -> Result<(), ArtifactError> {
        self.ty(domain.head_type)?;
        self.generic(&domain.generic_signature)?;
        for constraint in &domain.constraints {
            self.constraint(constraint)?;
        }
        Ok(())
    }
    fn associated_resolution(&mut self, resolution: &crate::checker::associated::AssociatedResolution) -> Result<(), ArtifactError> {
        use crate::checker::associated::AssociatedResolutionKind;
        self.ty(resolution.owner_form)?;
        match &resolution.kind {
            AssociatedResolutionKind::ExactValue { value_type, .. } | AssociatedResolutionKind::DataExpansionField { value_type, .. } => self.ty(*value_type),
            AssociatedResolutionKind::ExactCallable { callable_type, .. } => self.ty(*callable_type),
            AssociatedResolutionKind::Family { family_type, members } => {
                self.ty(*family_type)?;
                for member in members.iter() {
                    self.ty(member.value_type)?;
                }
                Ok(())
            }
            AssociatedResolutionKind::StaticInvoke { result_type, .. } => self.ty(*result_type),
            AssociatedResolutionKind::StaticDomain { domain, .. } => self.domain(*domain),
            AssociatedResolutionKind::StaticTerm { domain, term, .. } => {
                self.domain(*domain)?;
                self.ty(*term)
            }
            AssociatedResolutionKind::DynamicInvoke { candidates, result_type } => {
                for candidate in candidates.iter() {
                    self.ty(candidate.value_type)?;
                }
                result_type.map_or(Ok(()), |result_type| self.ty(result_type))
            }
        }
    }
    fn callable_reference_resolution(&mut self, resolution: &crate::checker::associated::CallableReferenceResolution) -> Result<(), ArtifactError> {
        use crate::checker::associated::CallableReferenceResolutionKind;
        match &resolution.kind {
            CallableReferenceResolutionKind::BoundFamily { family_type, members, .. } => {
                self.ty(*family_type)?;
                for member in members.iter() {
                    self.ty(member.callable_type)?;
                }
                Ok(())
            }
            CallableReferenceResolutionKind::Associated(resolution) => self.associated_resolution(resolution),
        }
    }
    fn family_application_resolution(&mut self, resolution: &crate::checker::associated::FamilyApplicationResolution) -> Result<(), ArtifactError> {
        use crate::checker::associated::FamilyApplicationSelection;
        self.ty(resolution.family_type)?;
        match &resolution.selection {
            FamilyApplicationSelection::Static {
                callable_type, result_type, ..
            } => {
                self.ty(*callable_type)?;
                self.ty(*result_type)
            }
            FamilyApplicationSelection::Dynamic { candidates, result_type } => {
                for candidate in candidates.iter() {
                    self.ty(candidate.callable_type)?;
                }
                result_type.map_or(Ok(()), |result_type| self.ty(result_type))
            }
        }
    }
    fn denotation(&mut self, denotation: &crate::types::denotation::SemanticDenotation) -> Result<(), ArtifactError> {
        use crate::types::denotation::{AssociatedValueDenotation, SemanticDenotation};
        match denotation {
            SemanticDenotation::TypeForm(ty) | SemanticDenotation::StaticTerm(ty) => self.ty(*ty),
            SemanticDenotation::Kind(kind) => self.kind(*kind),
            SemanticDenotation::StaticDomain(domain) => self.domain(*domain),
            SemanticDenotation::StaticApplication(_) => Err(ArtifactError::LimitExceeded(
                "checked static-application rows are not retained by callable analysis".into(),
            )),
            SemanticDenotation::AssociatedValue(value) => match &**value {
                AssociatedValueDenotation::Exact { owner_form, .. } | AssociatedValueDenotation::Family { owner_form, .. } => self.ty(*owner_form),
            },
            SemanticDenotation::BehavioralFamily(family) => {
                self.ty(family.receiver_type)?;
                if let Some(application) = family.receiver_application {
                    self.ty(application)?;
                }
                Ok(())
            }
        }
    }
    fn term(&mut self, term: &TypeTerm) -> Result<(), ArtifactError> {
        match term {
            TypeTerm::Canonical(ty) => self.ty(*ty),
            TypeTerm::SelfType(_) => Ok(()),
            TypeTerm::Infer(_) => Err(ArtifactError::LimitExceeded("inference variable in published root".into())),
        }
    }
    fn fact(&mut self, fact: &DeclaredTypeFact) -> Result<(), ArtifactError> {
        if let DeclaredTypeState::Known(term) = &fact.state {
            self.term(term)?;
        }
        Ok(())
    }
    fn callable_signature(&mut self, signature: &crate::signature::CallableSemanticSignature) -> Result<(), ArtifactError> {
        self.generic(&signature.generics)?;
        for parameter in &signature.parameters {
            self.fact(&parameter.declared_type)?;
            match &parameter.contract {
                crate::signature::ParameterContract::Fixed(fact) => self.fact(fact)?,
                crate::signature::ParameterContract::Rest { contract, .. } => match contract {
                    crate::signature::RestValueContract::Unconstrained => {}
                    crate::signature::RestValueContract::Element(fact) => self.fact(fact)?,
                    crate::signature::RestValueContract::Lane(ty) => self.ty(*ty)?,
                    crate::signature::RestValueContract::Invalid(_) => {}
                },
            }
        }
        self.fact(&signature.declared_return)?;
        if let Some(crate::types::evidence::TypeKnowledge::Known(evidence)) = &signature.inferred_return {
            self.ty(evidence.ty())?;
        }
        Ok(())
    }
    fn constraint(&mut self, c: &GenericConstraint) -> Result<(), ArtifactError> {
        match c {
            GenericConstraint::Subtype { lower, upper } => {
                self.term(lower)?;
                self.term(upper)?;
            }
            GenericConstraint::Equivalent { left, right } => {
                self.term(left)?;
                self.term(right)?;
            }
            GenericConstraint::Conforms { subject, trait_ref } => {
                self.term(subject)?;
                for &ty in &trait_ref.arguments {
                    self.ty(ty)?;
                }
            }
        }
        Ok(())
    }
    fn generic(&mut self, g: &Option<GenericSignature>) -> Result<(), ArtifactError> {
        if let Some(g) = g {
            for &p in &g.parameters {
                let key = self.key()?;
                self.types.register_parameter_root(self.types.expected_lineage(), key, p)?;
            }
            for &k in &g.parameter_kinds {
                let key = self.key()?;
                self.types.register_kind_root(self.types.expected_lineage(), key, k)?;
            }
            for c in &g.constraints {
                self.constraint(c)?;
            }
        }
        Ok(())
    }
    fn static_data(&mut self, info: &Option<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo>) -> Result<(), ArtifactError> {
        if let Some(info) = info {
            let key = self.key()?;
            self.types.register_domain_root(self.types.expected_lineage(), key, info.domain)?;
            for &c in &info.constructors {
                let key = self.key()?;
                self.types.register_constructor_root(self.types.expected_lineage(), key, c)?;
            }
        }
        Ok(())
    }
}

/// Register every reference used by the current primary projection in stable owner order.
/// Required owner adapters extend this pass together with their wire projection.
pub fn register_semantic_world_roots(
    snapshot: &SemanticSnapshot,
    types: &mut TypeExportContext,
    statics: &mut StaticExportContext,
) -> Result<(), ArtifactError> {
    register_structural_primary_roots(
        types,
        &snapshot.data_semantics,
        &snapshot.enum_semantics,
        &snapshot.declarations,
        &snapshot.hierarchy,
        &snapshot.type_aliases,
    )?;
    for (id, info) in snapshot.declarations.iter() {
        Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::GenericSignature).generic(&info.generic_signature)?;
    }
    for (id, info) in snapshot.data_semantics.data_decls.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::DataContract);
        r.generic(&info.generic_signature)?;
        r.static_data(&info.static_data)?;
        for c in &info.components {
            r.fact(&c.declared_type)?;
        }
        for p in &info.constructor.parameters {
            r.fact(&p.declared_type)?;
        }
        r.ty(info.constructor.result_type_template)?;
        for c in &info.shape_template.contributions {
            match c {
                crate::data_semantics::DataProductContribution::Component(c) => r.fact(&c.declared_type)?,
                crate::data_semantics::DataProductContribution::Expansion(e) => r.fact(&e.operand)?,
            }
        }
    }
    for (id, info) in snapshot.enum_semantics.enums.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::EnumContract);
        r.generic(&info.generic_signature)?;
        r.static_data(&info.static_data)?;
        r.ty(info.default_result_type)?;
    }
    for (id, info) in snapshot.enum_semantics.variants.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Variant(to_stable_variant(id)), Family::VariantContract);
        for f in &info.fields {
            r.fact(&f.declared_type)?;
        }
        r.ty(info.result_type_template)?;
        r.ty(info.exact_case_template)?;
        // Case bindings are keyed by stable binder identity, not native parameter IDs.
        let mut bindings = info.case_environment.bindings.iter().collect::<Vec<_>>();
        bindings.sort_by_key(|(p, _)| {
            let p = snapshot.store.type_parameter(**p);
            (p.owner.clone(), p.index)
        });
        for (&p, &ty) in bindings {
            let key = r.key()?;
            r.types.register_parameter_root(r.types.expected_lineage(), key, p)?;
            r.ty(ty)?;
        }
        for c in &info.case_environment.equalities {
            r.constraint(c)?;
        }
        if let Some(c) = &info.constructor {
            r.generic(&c.generic_signature)?;
            for p in &c.parameters {
                r.fact(&p.declared_type)?;
            }
            r.ty(c.result_type_template)?;
            r.ty(c.exact_case_template)?;
        }
    }
    for (id, sig) in snapshot.callable_signatures.iter() {
        if &sig.callable != id {
            return Err(ArtifactError::LimitExceeded("callable signature table key disagrees with its owner".into()));
        }
        register_callable_signature_roots(types, sig)?;
    }
    for (id, definition) in snapshot.callable_definitions.iter() {
        if id != &definition.callable {
            return Err(ArtifactError::LimitExceeded("callable definition table key disagrees with its owner".into()));
        }
        register_callable_definition_roots(types, definition)?;
    }
    for (id, analysis) in snapshot.callable_analyses.iter() {
        if id != &analysis.callable {
            return Err(ArtifactError::LimitExceeded("callable analysis table key disagrees with its owner".into()));
        }
        register_callable_analysis_roots(types, analysis, &snapshot.store)?;
    }
    for (_owner, requirements) in snapshot.enum_requirements.requirements.iter() {
        for requirement in requirements.iter() {
            register_callable_signature_roots(types, &requirement.signature)?;
        }
    }
    for (id, head) in snapshot.conformance_index.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(id)), Family::TraitAndConformance);
        r.ty(head.target_head)?;
        r.generic(&head.generic_signature)?;
        for &ty in &head.trait_ref.arguments {
            r.ty(ty)?;
        }
    }
    for (decl_id, surface) in snapshot.surfaces.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(decl_id)), Family::SurfaceAndDispatch);
        for (_, field_sig) in snapshot.field_signatures.iter_owner(decl_id) {
            r.fact(&field_sig.declared_type)?;
        }
        for side in [crate::identity::DispatchSide::Instance, crate::identity::DispatchSide::Class] {
            let members = surface.surface(side);
            for knowledge in members.callables.values().chain(members.fields.values()) {
                r.knowledge(knowledge)?;
            }
        }
    }
    for (id, surface) in snapshot.trait_surfaces.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::TraitAndConformance);
        r.generic(&surface.generic_signature)?;
        for (_, member) in surface.iter() {
            r.callable_signature(&member.signature)?;
        }
        for associated in surface.associated_types.values() {
            match associated.domain {
                crate::types::type_member::TypeMemberDomain::Kind(kind) => r.kind(kind)?,
                crate::types::type_member::TypeMemberDomain::ValueType(ty) => r.ty(ty)?,
            }
        }
        for diagnostic in &surface.diagnostics {
            for guidance in &diagnostic.guidance {
                match guidance {
                    crate::diagnostic::DiagnosticGuidance::ChangeAnnotation { ty, .. }
                    | crate::diagnostic::DiagnosticGuidance::SupplyAssignableValue { expected: ty }
                    | crate::diagnostic::DiagnosticGuidance::EstablishTypeEvidence { expected: Some(ty), .. } => r.ty(*ty)?,
                    crate::diagnostic::DiagnosticGuidance::ResolveGenericParameter { parameter } => r.parameter(*parameter)?,
                    crate::diagnostic::DiagnosticGuidance::UseCallableShape { .. }
                    | crate::diagnostic::DiagnosticGuidance::EstablishTypeEvidence { expected: None, .. } => {}
                }
            }
        }
    }
    for (target, members) in snapshot.dispatch.conditional_members().iter() {
        for domain in members.members.iter().map(|member| &member.domain) {
            let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(&domain.impl_id)), Family::SurfaceAndDispatch);
            r.ty(domain.head_type)?;
            r.generic(&domain.generic_signature)?;
            for constraint in &domain.constraints {
                r.constraint(constraint)?;
            }
        }
        for member in &members.members {
            let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(&member.impl_id)), Family::SurfaceAndDispatch);
            r.callable_signature(&member.signature_template)?;
        }
        match target {
            crate::impls::InherentImplTarget::Declaration(declaration) => {
                let _ = to_stable_declaration(declaration);
            }
            crate::impls::InherentImplTarget::ExactEnumCase(variant) => {
                let _ = to_stable_variant(variant);
            }
        }
    }
    for (id, product) in snapshot.type_member_declarations.iter() {
        let owner = phalcom_type_meta::identity::StableTypeMemberRef {
            owner: to_stable_declaration(&id.owner),
            selector: id.selector.name().into(),
        };
        let mut r = Roots::new(types, StableOwnerRef::TypeMember(owner), Family::TypeMemberAndDomain);
        for parameter in &product.signature.parameters {
            match parameter.domain {
                crate::types::type_member::TypeMemberDomain::Kind(kind) => {
                    let key = r.key()?;
                    r.types.register_kind_root(r.types.expected_lineage(), key, kind)?;
                }
                crate::types::type_member::TypeMemberDomain::ValueType(ty) => r.ty(ty)?,
            }
        }
        match product.signature.result_domain {
            crate::types::type_member::TypeMemberDomain::Kind(kind) => {
                let key = r.key()?;
                r.types.register_kind_root(r.types.expected_lineage(), key, kind)?;
            }
            crate::types::type_member::TypeMemberDomain::ValueType(ty) => r.ty(ty)?,
        }
    }
    let mut associated_domain_ordinal = 100u32;
    for (id, product) in snapshot.associated_static_domains.iter() {
        let owner = StableOwnerRef::Declaration(to_stable_declaration(&id.owner));
        let key = CanonicalTypeRootKey {
            family_tag: Family::TypeMemberAndDomain,
            owner: owner.clone(),
            product_subkey: associated_domain_ordinal,
        };
        associated_domain_ordinal = associated_domain_ordinal
            .checked_add(1)
            .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
        types.register_domain_root(types.expected_lineage(), key, product.domain)?;
        for &constructor in &product.constructors {
            let key = CanonicalTypeRootKey {
                family_tag: Family::TypeMemberAndDomain,
                owner: owner.clone(),
                product_subkey: associated_domain_ordinal,
            };
            associated_domain_ordinal = associated_domain_ordinal
                .checked_add(1)
                .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
            types.register_constructor_root(types.expected_lineage(), key, constructor)?;
        }
    }
    for (domain, catalog) in snapshot.static_case_catalogs.iter() {
        if domain != &catalog.domain {
            return Err(ArtifactError::LimitExceeded("static-case catalog table key disagrees with its owner".into()));
        }
        let owner = match domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => declaration,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner,
        };
        if let crate::identity::StaticDomainDeclarationId::Nominal(declaration) = domain {
            let domain_id = snapshot
                .store
                .static_domain_ids()
                .find(|candidate| {
                    matches!(snapshot.store.static_domain(*candidate), Some(crate::types::store::StaticDomainData::Nominal { declaration: found }) if found == declaration)
                })
                .ok_or_else(|| ValidationError::ReferentialIntegrity("static-case catalog has no interned nominal domain root".into()))?;
            let key = CanonicalTypeRootKey {
                family_tag: Family::TypeMemberAndDomain,
                owner: StableOwnerRef::Declaration(to_stable_declaration(owner)),
                product_subkey: associated_domain_ordinal,
            };
            associated_domain_ordinal = associated_domain_ordinal
                .checked_add(1)
                .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
            types.register_domain_root(types.expected_lineage(), key, domain_id)?;
        }
    }
    for (case, signature) in snapshot.static_case_signatures.iter() {
        if case != &signature.id {
            return Err(ArtifactError::LimitExceeded("static-case signature table key disagrees with its owner".into()));
        }
        let owner = match &case.domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => declaration,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner,
        };
        for &kind in signature.parameter_kinds.iter() {
            let key = CanonicalTypeRootKey {
                family_tag: Family::TypeMemberAndDomain,
                owner: StableOwnerRef::Declaration(to_stable_declaration(owner)),
                product_subkey: associated_domain_ordinal,
            };
            associated_domain_ordinal = associated_domain_ordinal
                .checked_add(1)
                .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
            types.register_kind_root(types.expected_lineage(), key, kind)?;
        }
    }
    for (id, plan) in snapshot.conformance_witness_plans.iter() {
        if id != &plan.impl_id {
            return Err(ArtifactError::LimitExceeded("conformance witness table key disagrees with its owner".into()));
        }
        let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(id)), Family::ConstructorContract);
        r.store = Some(&snapshot.store);
        r.ty(plan.target_template)?;
        r.generic(&plan.generic_signature)?;
        r.trait_ref(&plan.trait_ref_template)?;
        for requirement in plan.requirement_views.values() {
            r.callable_signature(&requirement.signature)?;
        }
        for selection in plan.requirements.values() {
            r.requirement_selection(selection)?;
        }
        r.associated_type_plan(&plan.associated_type_plan)?;
        r.conformance_completeness(&plan.completeness)?;
        for diagnostic in &plan.diagnostics {
            for guidance in &diagnostic.guidance {
                r.diagnostic_guidance(guidance)?;
            }
        }
    }
    for rule in snapshot.derived_conformance_rules.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(&rule.impl_id)), Family::TraitAndConformance);
        r.store = Some(&snapshot.store);
        r.derived_conformance_rule(rule)?;
    }
    for (_, product) in snapshot.static_definitions.iter() {
        if let Some(program) = &product.canonical_program {
            let owner = StableOwnerRef::Declaration(to_stable_declaration(&program.owner.0));
            let next_ordinal = super::operational::register_operational_template_roots(program, statics, types)?;
            let mut roots = Roots::new_with_ordinal(types, owner, Family::StaticDefAndBinding, next_ordinal);
            for application in program.applications.iter() {
                roots.static_definition_signature(&application.signature)?;
                for argument in application.arguments.iter() {
                    if let crate::static_definition::CanonicalStaticApplicationArgument::KindedTerm { term, .. } = argument {
                        roots.ty(*term)?;
                    }
                }
            }
            roots.static_definition_signature(&product.definition.signature)?;
            for dependency in program.dependencies.iter() {
                roots.semantic_dependency(dependency)?;
            }
            for diagnostic in product.diagnostics.iter() {
                roots.semantic_diagnostic(diagnostic)?;
            }
        } else if let Some(body) = &product.checked_body {
            let owner = StableOwnerRef::Declaration(to_stable_declaration(&body.owner.0));
            let mut next_ordinal = 0;
            if let crate::static_definition::StaticDefinitionBodyRepr::Operational(program) = &body.root {
                next_ordinal = super::operational::register_operational_template_roots(program, statics, types)?;
            }
            let mut roots = Roots::new_with_ordinal(types, owner, Family::StaticDefAndBinding, next_ordinal);
            roots.store = Some(&snapshot.store);
            for application in body.static_applications.iter() {
                roots.checked_static_application(&body.owner, application)?;
            }
            for diagnostic in product.diagnostics.iter() {
                roots.semantic_diagnostic(diagnostic)?;
            }
        }
    }
    let mut static_constant_ordinals = BTreeMap::<_, u32>::new();
    let mut static_constants = snapshot.static_constants.iter().collect::<Vec<_>>();
    static_constants.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    for (id, product) in static_constants {
        if id != &product.id {
            return Err(ArtifactError::LimitExceeded(
                "static constant table key disagrees with its product owner".into(),
            ));
        }
        let crate::static_eval::StaticEvalOutcome::Reduced(value) = &product.outcome else {
            return Err(ArtifactError::LimitExceeded("canonical static constants require Reduced outcomes".into()));
        };
        if product.value != Some(*value)
            || product
                .checked
                .as_ref()
                .is_some_and(|checked| crate::static_constant::StaticConstantAuthority::from_checked(checked) != product.authority)
        {
            return Err(ArtifactError::LimitExceeded(
                "static constant persistent authority disagrees with its source product".into(),
            ));
        }
        let stable_module = to_stable_module(&id.0.module);
        let ordinal = static_constant_ordinals.entry(stable_module.clone()).or_default();
        let base = *ordinal;
        let after_values = base
            .checked_add(3)
            .ok_or_else(|| ArtifactError::LimitExceeded("static constant root ordinal overflow".into()))?;
        let owner = StableOwnerRef::Module(stable_module);
        let root_key = |product_subkey| CanonicalTypeRootKey {
            family_tag: Family::StaticValue,
            owner: owner.clone(),
            product_subkey,
        };
        if let Some(ty) = product.authority.result_type {
            types.register_type_root(types.expected_lineage(), root_key(base), ty)?;
        }
        statics.register_root(root_key(base + 1), *value)?;
        if let Some(value) = product.lkg_value {
            statics.register_root(root_key(base + 2), value)?;
        }
        *ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| ArtifactError::LimitExceeded("static constant root ordinal overflow".into()))?;
        let mut roots = Roots::new_with_ordinal(types, owner.clone(), Family::StaticValue, after_values);
        roots.store = Some(&snapshot.store);
        for dependency in product.authority.dependencies.iter() {
            roots.semantic_dependency(dependency)?;
        }
        for diagnostic in product.diagnostics.iter() {
            roots.semantic_diagnostic(diagnostic)?;
        }
        *ordinal = roots.next_ordinal();
    }
    let mut module_bindings = snapshot.module_static_bindings.iter().collect::<Vec<_>>();
    module_bindings.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    for (module, index) in module_bindings {
        if module != index.module() {
            return Err(ArtifactError::LimitExceeded("module static-binding index key disagrees with its owner".into()));
        }
        let mut roots = Roots::new(types, StableOwnerRef::Module(to_stable_module(module)), Family::StaticDefAndBinding);
        roots.store = Some(&snapshot.store);
        for (_, fact) in index.iter() {
            if fact.state.binding != fact.source_binding || fact.source_body.module() != &fact.symbol.module {
                return Err(ArtifactError::LimitExceeded(
                    "module static-binding fact has an inconsistent binding or symbol owner".into(),
                ));
            }
            roots.binding_state(&fact.state)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Family, Roots, StableOwnerRef, register_callable_analysis_roots, to_stable_declaration};
    use crate::checker::analysis::{BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, FlowStateSummary};
    use crate::identity::{
        BindingId, BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, FieldId, ImplLocalId, LocalExpressionId, ModuleId, VariantId,
    };
    use crate::types::denotation::SemanticDenotation;
    use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
    use crate::types::store::TypeStore;
    use crate::types::store::artifact::TypeExportContext;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::Selector;
    use std::sync::Arc;

    fn callable_analysis(callable: CallableId) -> CallableAnalysis {
        CallableAnalysis {
            callable,
            body_range: SourceRange::default(),
            body_source_owner: None,
            expressions: Default::default(),
            bindings: Default::default(),
            binding_origins: Default::default(),
            for_iteration_lanes: Arc::new(Default::default()),
            positional_expansions: Arc::new(Default::default()),
            known_invalid_positional_expansions: Arc::new(Default::default()),
            sequence_literals: Arc::new(Default::default()),
            associated_resolutions: Arc::new(Default::default()),
            callable_reference_resolutions: Arc::new(Default::default()),
            family_applications: Arc::new(Default::default()),
            match_resolutions: Arc::new(Default::default()),
            pattern_site_resolutions: Arc::new(Default::default()),
            flow_graph: Arc::new(crate::checker::flow::graph::FlowGraph::default()),
            entry_flow: FlowStateSummary::default(),
            exits: BodyExitFacts {
                normal_returns: Vec::new(),
                throws: Vec::new(),
                unreachable: false,
            },
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
            diagnostics: Arc::from([]),
            internal_incidents: Arc::from([]),
            explanations: Arc::new(crate::explain::ExplanationArena::default()),
            return_explanation: None,
            dependencies: Arc::from([]),
            semantic_dependencies: Arc::from([]),
            constructor_super_initializer: None,
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status: CallableAnalysisStatus::Complete,
        }
    }

    fn callable_signature(callable: CallableId, owner: &DeclarationId, return_type: crate::types::id::TypeId) -> crate::signature::CallableSemanticSignature {
        crate::signature::CallableSemanticSignature {
            owner: owner.clone(),
            side: callable.side,
            selector: callable.selector.clone(),
            callable,
            generics: None,
            parameters: Box::new([]),
            declared_return: crate::declaration_type::DeclaredTypeFact::known(
                crate::types::parameter::TypeTerm::Canonical(return_type),
                crate::declaration_type::DeclaredTypeBasis::SourceAnnotation,
            ),
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
            native_id: None,
            effects: phalcom_native_meta::EffectSpec::Unknown,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            flow: phalcom_native_meta::ReturnFlowSpec::Value,
            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
        }
    }

    #[test]
    fn callable_blocked_query_owner_roots_cover_expression_binding_flow_pattern_and_explanation_paths() {
        use crate::checker::analysis::{AnalysisStatus, BindingState, FlowBindingSummary, FlowFieldSummary};
        use crate::explain::node::{DerivationRule, EvidenceRef, ExplanationStep};
        use crate::generic_evidence::GenericConformanceKey;
        use crate::match_semantics::{
            ExhaustivenessResult, MatchResolution, PatternResolution, PatternSpaceSummary, ResolvedVariantPattern, VariantSelectorConstraint,
        };
        use crate::types::outcome::BlockReason;
        use crate::types::parameter::TypeTerm;

        let callable_owner = DeclarationId::new(ModuleId::universe_root(), "BlockedOwnerRoot".into());
        let callable = CallableId::method(callable_owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let subject = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "BlockedSubjectRoot".into()));
        let argument = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "BlockedTraitArgumentRoot".into()));
        let key = crate::db::QueryKey::GenericConformance(GenericConformanceKey::new(
            TypeTerm::Canonical(subject),
            crate::traits::TraitRef::new(
                DeclarationId::new(ModuleId::universe_root(), "BlockedTraitRoot".into()),
                vec![argument].into_boxed_slice(),
            ),
        ));
        let reason = BlockReason::UnavailableSemanticOwner(key);
        let blocked = || reason.clone();
        let exit_subject = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "BlockedExitSubjectRoot".into()));
        let exit_argument = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "BlockedExitTraitArgumentRoot".into()));
        let exit_reason = BlockReason::UnavailableSemanticOwner(crate::db::QueryKey::GenericConformance(GenericConformanceKey::new(
            TypeTerm::Canonical(exit_subject),
            crate::traits::TraitRef::new(
                DeclarationId::new(ModuleId::universe_root(), "BlockedExitTraitRoot".into()),
                vec![exit_argument].into_boxed_slice(),
            ),
        )));
        let expression_id = ExpressionId::new(BodyId(8), LocalExpressionId(0));
        let mut analysis = callable_analysis(callable);

        let mut expression = ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 1, end: 2 },
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
        );
        expression.status = AnalysisStatus::Blocked(blocked());
        analysis.expressions.insert_source(expression_id, expression).unwrap();

        let binding_id = BindingId(4);
        let mut binding = BindingState::new(
            binding_id,
            "local",
            SourceRange { start: 3, end: 4 },
            None,
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
            false,
        );
        let exact_case = VariantId::new(callable_owner.clone(), Selector::getter("BlockedCase").unwrap());
        let exact_case_type = store.intern_variant_identity(exact_case.clone());
        binding.consistency = crate::checker::binding::BindingConsistency::Blocked(BlockReason::UnavailableSemanticOwner(
            crate::db::QueryKey::TraitDispatchBucket(crate::trait_dispatch::TraitDispatchBucketKey {
                target_family: crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(exact_case),
                selector: Selector::getter("blocked").unwrap(),
                side: DispatchSide::Instance,
            }),
        ));
        analysis.bindings.insert_source(binding_id, binding).unwrap();

        analysis.exits.normal_returns.push(crate::checker::analysis::NormalReturnFact {
            knowledge: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
            flow: FlowStateSummary::default(),
            status: crate::checker::analysis::AnalysisStatus::Blocked(exit_reason),
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            direct_result_dependency: None,
        });

        analysis
            .entry_flow
            .bindings
            .insert_source(
                binding_id,
                FlowBindingSummary {
                    knowledge: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    contract: None,
                    consistency: crate::checker::binding::BindingConsistency::Blocked(blocked()),
                    mutable: false,
                    initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
                },
            )
            .unwrap();
        let field = FieldId::new(callable_owner.clone(), "field", DispatchSide::Instance);
        analysis
            .entry_flow
            .fields
            .insert_source(
                field,
                FlowFieldSummary {
                    contract: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    current: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    initialization: crate::checker::flow::FieldInitialization::DefinitelyInitialized,
                    validity: crate::checker::flow::FieldContractValidity::Blocked(blocked()),
                    causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
                },
            )
            .unwrap();

        let variant = ResolvedVariantPattern {
            owner: None,
            family: None,
            owner_candidates: Box::new([]),
            selector: VariantSelectorConstraint::WholeFamily,
            candidates: Box::new([]),
            blocked: Some(blocked()),
        };
        let mut match_resolutions = crate::checker::analysis_map::AnalysisMap::default();
        match_resolutions
            .insert_source(
                expression_id,
                MatchResolution {
                    expression: expression_id,
                    scrutinee: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    initial_space: PatternSpaceSummary::Blocked(blocked()),
                    arms: vec![crate::match_semantics::MatchArmResolution {
                        arm_index: 0,
                        pattern: PatternResolution::Variant(variant),
                        reachable_space: PatternSpaceSummary::Empty,
                        residual_after: PatternSpaceSummary::Empty,
                        bindings: Box::new([]),
                        proof: Default::default(),
                        usefulness: crate::match_semantics::PatternUsefulness::Useful,
                        branch_result: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    }]
                    .into_boxed_slice(),
                    result: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    exhaustiveness: ExhaustivenessResult::Blocked(blocked()),
                },
            )
            .unwrap();
        analysis.match_resolutions = Arc::new(match_resolutions);

        let mut explanations = crate::explain::ExplanationArena::new();
        explanations
            .alloc_full(
                ExplanationStep::InternalBlocked { reason: blocked() },
                DerivationRule::InternalBlocked,
                EvidenceStatus::Established,
                EvidenceOrigin::Flow,
                vec![EvidenceRef::TypeId(subject), EvidenceRef::TypeId(argument)],
                Vec::new(),
            )
            .unwrap();
        analysis.explanations = Arc::new(explanations);

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(subject).is_ok(), "generic blocked-owner subject must be rooted");
        assert!(types.ty(argument).is_ok(), "generic blocked-owner trait argument must be rooted");
        assert!(types.ty(exit_subject).is_ok(), "normal-return blocked-owner subject must be rooted");
        assert!(types.ty(exit_argument).is_ok(), "normal-return blocked-owner trait argument must be rooted");
        assert!(types.variant(exact_case_type).is_ok(), "exact-case dispatch owner identity must be rooted");
    }

    #[test]
    fn callable_analysis_denotation_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "Rooted".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let denotation_only_type = store.nominal_type(owner);
        let expression_id = ExpressionId::new(BodyId(4), LocalExpressionId(2));
        let mut analysis = callable_analysis(callable);
        analysis
            .expressions
            .insert_source(
                expression_id,
                ExpressionAnalysis::ready(
                    expression_id,
                    SourceRange { start: 1, end: 2 },
                    TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                )
                .with_denotation(Some(SemanticDenotation::TypeForm(denotation_only_type))),
            )
            .unwrap();

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(
            types.ty(denotation_only_type).is_ok(),
            "type roots reachable only through callable denotations must be registered before graph closure"
        );
    }

    #[test]
    fn callable_analysis_expression_knowledge_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "KnowledgeRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let expression_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpressionKnowledgeRoot".into()));
        let expression_id = ExpressionId::new(BodyId(7), LocalExpressionId(3));
        let mut analysis = callable_analysis(callable);
        analysis
            .expressions
            .insert_source(
                expression_id,
                ExpressionAnalysis::ready(
                    expression_id,
                    SourceRange { start: 2, end: 5 },
                    TypeKnowledge::established(expression_type, EvidenceOrigin::Flow),
                ),
            )
            .unwrap();

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(
            types.ty(expression_type).is_ok(),
            "known expression types must be registered before callable-analysis graph closure"
        );
    }

    #[test]
    fn callable_analysis_binding_type_facts_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "BindingRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let contract_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "BindingContractRoot".into()));
        let current_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "BindingCurrentRoot".into()));
        let binding = BindingId(12);
        let range = SourceRange { start: 1, end: 5 };
        let mut analysis = callable_analysis(callable);
        analysis
            .bindings
            .insert_source(
                binding,
                crate::checker::analysis::BindingState::new_with_contract(
                    binding,
                    "value",
                    range,
                    Some(crate::checker::binding::BindingContract {
                        ty: contract_type,
                        origin: crate::checker::binding::BindingContractOrigin::SourceAnnotation,
                        source: Some(range),
                    }),
                    TypeKnowledge::established(current_type, EvidenceOrigin::Flow),
                    None,
                    false,
                    crate::checker::flow::InitializationState::DefinitelyInitialized,
                ),
            )
            .unwrap();

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(contract_type).is_ok(), "binding contracts must be roots");
        assert!(types.ty(current_type).is_ok(), "binding flow knowledge must be roots");
    }

    #[test]
    fn callable_analysis_flow_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "FlowRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let entry_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "EntryFlowRoot".into()));
        let flow_binding_contract = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "FlowBindingContractRoot".into()));
        let flow_binding_current = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "FlowBindingCurrentRoot".into()));
        let contract_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "FieldContractRoot".into()));
        let current_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "FieldCurrentRoot".into()));
        let field = FieldId::new(owner.clone(), "value", DispatchSide::Instance);
        let binding = BindingId(4);
        let mut analysis = callable_analysis(callable);
        analysis.entry_flow = FlowStateSummary {
            bindings: [(
                binding,
                crate::checker::analysis::FlowBindingSummary {
                    knowledge: TypeKnowledge::established(flow_binding_current, EvidenceOrigin::Flow),
                    contract: Some(crate::checker::binding::BindingContract {
                        ty: flow_binding_contract,
                        origin: crate::checker::binding::BindingContractOrigin::CallableParameter,
                        source: None,
                    }),
                    consistency: crate::checker::binding::BindingConsistency::Validated,
                    mutable: false,
                    initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
                },
            )]
            .into_iter()
            .collect(),
            fields: [(
                field.clone(),
                crate::checker::analysis::FlowFieldSummary {
                    contract: TypeKnowledge::established(contract_type, EvidenceOrigin::DeclarationSemantics),
                    current: TypeKnowledge::established(current_type, EvidenceOrigin::Flow),
                    initialization: crate::checker::flow::FieldInitialization::DefinitelyInitialized,
                    validity: crate::checker::flow::FieldContractValidity::Validated,
                    causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
                },
            )]
            .into_iter()
            .collect(),
            fact_count: 1,
        };
        analysis.exits.normal_returns.push(crate::checker::analysis::NormalReturnFact {
            knowledge: TypeKnowledge::established(entry_type, EvidenceOrigin::Flow),
            flow: FlowStateSummary::default(),
            status: crate::checker::analysis::AnalysisStatus::Ready,
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            direct_result_dependency: None,
        });

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(entry_type).is_ok(), "normal-exit knowledge must be rooted");
        assert!(types.ty(flow_binding_contract).is_ok(), "flow binding contracts must be rooted");
        assert!(types.ty(flow_binding_current).is_ok(), "flow binding knowledge must be rooted");
        assert!(types.ty(contract_type).is_ok(), "field flow contracts must be rooted");
        assert!(types.ty(current_type).is_ok(), "field current flow knowledge must be rooted");
    }

    #[test]
    fn callable_analysis_sequence_resolution_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "SequenceRootOwner".into());
        let callable = CallableId::method(owner, Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let result_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "SequenceResultRoot".into()));
        let element_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "SequenceElementRoot".into()));
        let extent_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "SequenceExtentRoot".into()));
        let construction_extent = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "SequenceConstructionExtentRoot".into()));
        let expansion_extent = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "SequenceExpansionExtentRoot".into()));
        let invalid_expansion_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "InvalidExpansionRoot".into()));
        let expression = ExpressionId::new(BodyId(8), LocalExpressionId(1));
        let invalid_expression = ExpressionId::new(BodyId(8), LocalExpressionId(2));
        let mut analysis = callable_analysis(callable);
        analysis.sequence_literals = Arc::new(
            [(
                expression,
                crate::checker::analysis::SequenceLiteralResolution {
                    source_expression: expression,
                    source_range: SourceRange { start: 1, end: 8 },
                    family: crate::checker::analysis::SequenceLiteralFamily::Vector,
                    result_type,
                    element_type,
                    extent: Some(extent_type),
                    construction: crate::checker::analysis::SequenceConstructionPlan::VectorIdentitySpread {
                        source_range: SourceRange { start: 2, end: 7 },
                        extent: construction_extent,
                    },
                    exact_expansions: [(
                        SourceRange { start: 3, end: 6 },
                        crate::checker::analysis::SequenceExpansionResolution::VectorStorage { extent: expansion_extent },
                    )]
                    .into_iter()
                    .collect(),
                },
            )]
            .into_iter()
            .collect(),
        );
        analysis.known_invalid_positional_expansions = Arc::new(
            [(
                invalid_expression,
                crate::checker::analysis::KnownInvalidPositionalExpansion {
                    source_expression: invalid_expression,
                    source_range: SourceRange { start: 9, end: 12 },
                    source_type: invalid_expansion_type,
                },
            )]
            .into_iter()
            .collect(),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(result_type).is_ok(), "sequence result types must be rooted");
        assert!(types.ty(element_type).is_ok(), "sequence element types must be rooted");
        assert!(types.ty(extent_type).is_ok(), "sequence extent types must be rooted");
        assert!(types.ty(construction_extent).is_ok(), "sequence construction-plan types must be rooted");
        assert!(types.ty(expansion_extent).is_ok(), "exact spread extent types must be rooted");
        assert!(
            types.ty(invalid_expansion_type).is_ok(),
            "known-invalid expansion receiver types must be rooted"
        );
    }

    #[test]
    fn callable_analysis_associated_resolution_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "AssociatedRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let owner_form = store.nominal_type(owner.clone());
        let callable_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "AssociatedCallableTypeRoot".into()));
        let member = crate::associated::AssociatedMemberId::DataConstructor(crate::identity::DataConstructorId::new(owner.clone()));
        let expression = ExpressionId::new(BodyId(9), LocalExpressionId(1));
        let mut analysis = callable_analysis(callable);
        analysis.associated_resolutions = Arc::new(
            [(
                expression,
                crate::checker::associated::AssociatedResolution {
                    owner_form,
                    lookup_owner: owner.clone(),
                    family: None,
                    kind: crate::checker::associated::AssociatedResolutionKind::ExactCallable {
                        member,
                        target: crate::identity::InvocationTargetId::DataConstructor(crate::identity::DataConstructorId::new(owner)),
                        callable_type,
                    },
                },
            )]
            .into_iter()
            .collect(),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(owner_form).is_ok(), "associated lookup owner forms must be rooted");
        assert!(types.ty(callable_type).is_ok(), "associated selected callable types must be rooted");
    }

    #[test]
    fn callable_analysis_reference_and_family_application_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "FamilyRootOwner".into());
        let callable = CallableId::method(owner, Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let reference_family_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ReferenceFamilyRoot".into()));
        let application_family_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ApplicationFamilyRoot".into()));
        let callable_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ApplicationCallableRoot".into()));
        let result_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ApplicationResultRoot".into()));
        let reference_expression = ExpressionId::new(BodyId(10), LocalExpressionId(1));
        let application_expression = ExpressionId::new(BodyId(10), LocalExpressionId(2));
        let mut analysis = callable_analysis(callable);
        analysis.callable_reference_resolutions = Arc::new(
            [(
                reference_expression,
                crate::checker::associated::CallableReferenceResolution {
                    kind: crate::checker::associated::CallableReferenceResolutionKind::BoundFamily {
                        family_type: reference_family_type,
                        spec: crate::checker::associated::BehavioralFamilySpec::Exact(Selector::method("run", []).unwrap()),
                        members: Box::new([]),
                    },
                },
            )]
            .into_iter()
            .collect(),
        );
        analysis.family_applications = Arc::new(
            [(
                application_expression,
                crate::checker::associated::FamilyApplicationResolution {
                    family_type: application_family_type,
                    kind: crate::checker::associated::FamilyApplicationKind::Behavioral,
                    selection: crate::checker::associated::FamilyApplicationSelection::Static {
                        operation: crate::types::family::FamilyOperationShape::getter(),
                        target: None,
                        callable_type,
                        result_type,
                    },
                },
            )]
            .into_iter()
            .collect(),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(reference_family_type).is_ok(), "bound callable-reference family types must be rooted");
        assert!(types.ty(application_family_type).is_ok(), "applied family types must be rooted");
        assert!(types.ty(callable_type).is_ok(), "family selected callable types must be rooted");
        assert!(types.ty(result_type).is_ok(), "family application result types must be rooted");
    }

    #[test]
    fn callable_analysis_iteration_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "IterationRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let source_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "IterationSourceRoot".into()));
        let item_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "IterationItemRoot".into()));
        let iterator_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "IteratorTypeRoot".into()));
        let next_result = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "NextResultRoot".into()));
        let conformance_subject = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "IterationConformanceSubjectRoot".into()));
        let trait_argument = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "IterationTraitArgumentRoot".into()));
        let expansion_source = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpansionSourceRoot".into()));
        let expansion_item = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpansionItemRoot".into()));
        let expansion_iterator = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpansionIteratorRoot".into()));
        let expansion_next_result = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpansionNextResultRoot".into()));
        let expression = ExpressionId::new(BodyId(11), LocalExpressionId(1));
        let expansion_expression = ExpressionId::new(BodyId(11), LocalExpressionId(2));
        let requirement = |selector| crate::traits::TraitRequirementId {
            owner: owner.clone(),
            selector,
            side: DispatchSide::Instance,
        };
        let iterator_requirement = requirement(Selector::method("iterator", []).unwrap());
        let conformance = crate::generic_evidence::GenericConformanceKey::new(
            crate::types::parameter::TypeTerm::Canonical(conformance_subject),
            crate::traits::TraitRef::new(owner.clone(), vec![trait_argument].into_boxed_slice()),
        );
        let mut analysis = callable_analysis(callable);
        analysis.for_iteration_lanes = Arc::new(
            [(
                expression,
                crate::checker::analysis::ForIterationLaneResolution {
                    source_expression: expression,
                    source_range: SourceRange { start: 1, end: 4 },
                    source_type,
                    item_type,
                    iterator_type,
                    iterator: crate::checker::analysis::ForIterationRequirementResolution::Evidence {
                        conformance: conformance.clone(),
                        evidence: crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(Arc::new(
                            crate::generic_evidence::CompilerDerivedConformanceEvidence {
                                key: conformance,
                                kind: crate::generic_evidence::CompilerDerivedConformanceKind::OrderedProduct,
                                classification: crate::structural::StructuralClassification::OrderedProduct(crate::structural::OrderedProductForm::Tuple),
                            },
                        )),
                        requirement: iterator_requirement,
                        signature: callable_signature(
                            CallableId::method(owner.clone(), Selector::method("iterator", []).unwrap(), DispatchSide::Instance),
                            &owner,
                            iterator_type,
                        ),
                    },
                    next: crate::checker::analysis::ForIterationRequirementResolution::AbstractRequirement {
                        requirement: requirement(Selector::method("next", []).unwrap()),
                        signature: callable_signature(
                            CallableId::method(owner.clone(), Selector::method("next", []).unwrap(), DispatchSide::Instance),
                            &owner,
                            next_result,
                        ),
                    },
                },
            )]
            .into_iter()
            .collect(),
        );
        analysis.positional_expansions = Arc::new(
            [(
                expansion_expression,
                crate::checker::analysis::PositionalExpansionIterationResolution {
                    source_expression: expansion_expression,
                    source_range: SourceRange { start: 5, end: 8 },
                    source_type: expansion_source,
                    item_type: expansion_item,
                    iterator_type: expansion_iterator,
                    iterator: crate::checker::analysis::ForIterationRequirementResolution::AbstractRequirement {
                        requirement: requirement(Selector::method("iterator", []).unwrap()),
                        signature: callable_signature(
                            CallableId::method(owner.clone(), Selector::method("iterator", []).unwrap(), DispatchSide::Instance),
                            &owner,
                            expansion_iterator,
                        ),
                    },
                    next: crate::checker::analysis::ForIterationRequirementResolution::AbstractRequirement {
                        requirement: requirement(Selector::method("next", []).unwrap()),
                        signature: callable_signature(
                            CallableId::method(owner.clone(), Selector::method("next", []).unwrap(), DispatchSide::Instance),
                            &owner,
                            expansion_next_result,
                        ),
                    },
                },
            )]
            .into_iter()
            .collect(),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [
            source_type,
            item_type,
            iterator_type,
            next_result,
            expansion_source,
            expansion_item,
            expansion_iterator,
            expansion_next_result,
            conformance_subject,
            trait_argument,
        ] {
            assert!(types.ty(ty).is_ok(), "callable iteration source, requirement, and result types must be rooted");
        }
    }

    #[test]
    fn callable_analysis_generic_conformance_evidence_roots_each_variant() {
        use crate::derived_conformance::DerivedConformanceRule;
        use crate::generic_evidence::{
            AssumedConformanceEvidence, DerivedConformanceEvidence, DerivedConformanceOrigin, GenericConformanceEvidence, GenericConformanceKey,
        };
        use crate::impls::{ConformanceAssociatedTypePlan, ConformanceEvidence};
        use crate::types::environment::TypeEnvironment;
        use crate::types::id::KindId;
        use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
        use std::collections::BTreeMap;

        let owner = DeclarationId::new(ModuleId::universe_root(), "GenericEvidenceRootOwner".into());
        let module = owner.module.clone();
        let mut store = TypeStore::new();
        let exact_target = store.nominal_type(DeclarationId::new(module.clone(), "ExactEvidenceTargetRoot".into()));
        let exact_trait_argument = store.nominal_type(DeclarationId::new(module.clone(), "ExactEvidenceTraitArgumentRoot".into()));
        let assumed_subject = store.nominal_type(DeclarationId::new(module.clone(), "AssumedEvidenceSubjectRoot".into()));
        let assumed_trait_argument = store.nominal_type(DeclarationId::new(module.clone(), "AssumedEvidenceTraitArgumentRoot".into()));
        let derived_subject = store.nominal_type(DeclarationId::new(module.clone(), "DerivedEvidenceSubjectRoot".into()));
        let derived_target_template = store.nominal_type(DeclarationId::new(module.clone(), "DerivedEvidenceTargetTemplateRoot".into()));
        let derived_rule_argument = store.nominal_type(DeclarationId::new(module.clone(), "DerivedEvidenceRuleArgumentRoot".into()));
        let derived_exact_trait_argument = store.nominal_type(DeclarationId::new(module.clone(), "DerivedEvidenceExactTraitArgumentRoot".into()));
        let exact_impl = crate::identity::ImplId::new(module.clone(), ImplLocalId(1));
        let exact = Arc::new(ConformanceEvidence {
            source_impl: exact_impl,
            exact_target,
            exact_trait_ref: crate::traits::TraitRef::new(
                DeclarationId::new(module.clone(), "ExactEvidenceTraitRoot".into()),
                vec![exact_trait_argument].into_boxed_slice(),
            ),
            impl_environment: TypeEnvironment::new(),
            requirement_views: Default::default(),
            requirements: Default::default(),
            associated_types: Default::default(),
            trait_header_contract: Box::new([]),
            trait_header_conformance_proofs: Some(Default::default()),
            fingerprint: crate::db::ProductFingerprint::default(),
        });
        let assumed = crate::generic_evidence::GenericConformanceEvidence::Assumed(AssumedConformanceEvidence {
            key: GenericConformanceKey::new(
                TypeTerm::Canonical(assumed_subject),
                crate::traits::TraitRef::new(
                    DeclarationId::new(module.clone(), "AssumedEvidenceTraitRoot".into()),
                    vec![assumed_trait_argument].into_boxed_slice(),
                ),
            ),
            owner: TypeParameterOwner::Declaration(DeclarationId::new(module.clone(), "AssumedEvidenceOwner".into())),
        });
        let derived_impl = crate::identity::ImplId::new(module.clone(), ImplLocalId(2));
        let parameter_owner = TypeParameterOwner::Impl(derived_impl.clone());
        let parameter = store.intern_type_parameter(TypeParameterData::new(parameter_owner.clone(), 0, "T", KindId::TYPE));
        let rule_trait = crate::traits::TraitRef::new(
            DeclarationId::new(module.clone(), "DerivedEvidenceRuleTraitRoot".into()),
            vec![derived_rule_argument].into_boxed_slice(),
        );
        let exact_trait = crate::traits::TraitRef::new(
            DeclarationId::new(module.clone(), "DerivedEvidenceExactTraitRoot".into()),
            vec![derived_exact_trait_argument].into_boxed_slice(),
        );
        let rule = Arc::new(DerivedConformanceRule::new(
            derived_impl.clone(),
            module.clone(),
            parameter,
            derived_target_template,
            rule_trait,
            GenericSignature::new(parameter_owner, vec![parameter].into_boxed_slice()),
            ConformanceAssociatedTypePlan::empty(derived_impl),
            BTreeMap::new(),
            BTreeMap::new(),
            crate::diagnostic::SemanticSourceSpan::new(module, SourceRange { start: 7, end: 9 }),
        ));
        let derived = GenericConformanceEvidence::Derived(Arc::new(DerivedConformanceEvidence {
            origin: DerivedConformanceOrigin::GuardedRule {
                rule,
                exact_subject: derived_subject,
                exact_trait_ref: exact_trait,
            },
            impl_bindings: Default::default(),
            nested: Box::new([]),
        }));

        let mut types = TypeExportContext::new(&store);
        let mut roots = Roots::new(&mut types, StableOwnerRef::Declaration(to_stable_declaration(&owner)), Family::CallableAnalysis);
        roots.generic_conformance_evidence(&GenericConformanceEvidence::Exact(exact)).unwrap();
        roots.generic_conformance_evidence(&assumed).unwrap();
        roots.generic_conformance_evidence(&derived).unwrap();
        drop(roots);
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [
            exact_target,
            exact_trait_argument,
            assumed_subject,
            assumed_trait_argument,
            derived_subject,
            derived_target_template,
            derived_rule_argument,
            derived_exact_trait_argument,
        ] {
            assert!(types.ty(ty).is_ok(), "generic evidence must root nested type {ty:?}");
        }
        assert!(types.parameter(parameter).is_ok(), "derived evidence must root its rule parameter");
    }

    #[test]
    fn callable_analysis_call_conformance_roots_all_source_variants() {
        use crate::checker::analysis::{CallSpecialization, ExpressionAnalysis, GenericCallConformanceBinding, GenericCallConformanceSource};
        use crate::generic_evidence::{
            CompilerDerivedConformanceEvidence, CompilerDerivedConformanceKind, DerivedConformanceEvidence, DerivedConformanceOrigin, GenericConformanceKey,
        };
        use crate::impls::ConformanceEvidence;
        use crate::types::environment::TypeEnvironment;
        use crate::types::parameter::TypeTerm;

        let owner = DeclarationId::new(ModuleId::universe_root(), "CallConformanceRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let callee_subject = store.nominal_type(DeclarationId::new(owner.module.clone(), "CalleeSubjectRoot".into()));
        let callee_argument = store.nominal_type(DeclarationId::new(owner.module.clone(), "CalleeTraitArgumentRoot".into()));
        let caller_subject = store.nominal_type(DeclarationId::new(owner.module.clone(), "CallerSubjectRoot".into()));
        let caller_argument = store.nominal_type(DeclarationId::new(owner.module.clone(), "CallerTraitArgumentRoot".into()));
        let exact_target = store.nominal_type(DeclarationId::new(owner.module.clone(), "ExactEvidenceTargetRoot".into()));
        let exact_argument = store.nominal_type(DeclarationId::new(owner.module.clone(), "ExactEvidenceTraitArgumentRoot".into()));
        let derived_target = store.nominal_type(DeclarationId::new(owner.module.clone(), "DerivedEvidenceTargetRoot".into()));
        let derived_argument = store.nominal_type(DeclarationId::new(owner.module.clone(), "DerivedEvidenceTraitArgumentRoot".into()));
        let compiler_subject = store.nominal_type(DeclarationId::new(owner.module.clone(), "CompilerDerivedSubjectRoot".into()));
        let compiler_argument = store.nominal_type(DeclarationId::new(owner.module.clone(), "CompilerDerivedTraitArgumentRoot".into()));
        let trait_owner = |name: &str| DeclarationId::new(owner.module.clone(), name.into());
        let conformance_key = |subject, trait_owner: DeclarationId, argument| {
            GenericConformanceKey::new(
                TypeTerm::Canonical(subject),
                crate::traits::TraitRef::new(trait_owner, vec![argument].into_boxed_slice()),
            )
        };
        let callee_conformance = conformance_key(callee_subject, trait_owner("CalleeTraitRoot"), callee_argument);
        let caller_conformance = conformance_key(caller_subject, trait_owner("CallerTraitRoot"), caller_argument);
        let exact_evidence = Arc::new(ConformanceEvidence {
            source_impl: crate::identity::ImplId::new(owner.module.clone(), ImplLocalId(41)),
            exact_target,
            exact_trait_ref: crate::traits::TraitRef::new(trait_owner("ExactEvidenceTraitRoot"), vec![exact_argument].into_boxed_slice()),
            impl_environment: TypeEnvironment::new(),
            requirement_views: Default::default(),
            requirements: Default::default(),
            associated_types: Default::default(),
            trait_header_contract: Box::new([]),
            trait_header_conformance_proofs: Some(Default::default()),
            fingerprint: crate::db::ProductFingerprint::default(),
        });
        let derived_source = Arc::new(ConformanceEvidence {
            source_impl: crate::identity::ImplId::new(owner.module.clone(), ImplLocalId(42)),
            exact_target: derived_target,
            exact_trait_ref: crate::traits::TraitRef::new(trait_owner("DerivedEvidenceTraitRoot"), vec![derived_argument].into_boxed_slice()),
            impl_environment: TypeEnvironment::new(),
            requirement_views: Default::default(),
            requirements: Default::default(),
            associated_types: Default::default(),
            trait_header_contract: Box::new([]),
            trait_header_conformance_proofs: Some(Default::default()),
            fingerprint: crate::db::ProductFingerprint::default(),
        });
        let derived_evidence = Arc::new(DerivedConformanceEvidence {
            origin: DerivedConformanceOrigin::ConditionalSource { exact_source: derived_source },
            impl_bindings: Default::default(),
            nested: Box::new([]),
        });
        let compiler_key = conformance_key(compiler_subject, trait_owner("CompilerDerivedTraitRoot"), compiler_argument);

        let expression_id = ExpressionId::new(BodyId(42), LocalExpressionId(3));
        let mut expression = ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 5, end: 12 },
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
        );
        expression.call_specialization = Some(CallSpecialization {
            callable: callable.clone(),
            bindings: Box::new([]),
            conformances: vec![
                GenericCallConformanceBinding {
                    callee_conformance: callee_conformance.clone(),
                    source: GenericCallConformanceSource::Exact { evidence: exact_evidence },
                },
                GenericCallConformanceBinding {
                    callee_conformance: callee_conformance.clone(),
                    source: GenericCallConformanceSource::Forwarded { caller_conformance },
                },
                GenericCallConformanceBinding {
                    callee_conformance: callee_conformance.clone(),
                    source: GenericCallConformanceSource::Derived { evidence: derived_evidence },
                },
                GenericCallConformanceBinding {
                    callee_conformance,
                    source: GenericCallConformanceSource::CompilerDerived {
                        evidence: Arc::new(CompilerDerivedConformanceEvidence {
                            key: compiler_key,
                            kind: CompilerDerivedConformanceKind::Product,
                            classification: crate::structural::StructuralClassification::Unsupported,
                        }),
                    },
                },
            ]
            .into_boxed_slice(),
        });

        let mut analysis = callable_analysis(callable);
        analysis.expressions.insert_source(expression_id, expression).unwrap();
        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [
            callee_subject,
            callee_argument,
            caller_subject,
            caller_argument,
            exact_target,
            exact_argument,
            derived_target,
            derived_argument,
            compiler_subject,
            compiler_argument,
        ] {
            assert!(types.ty(ty).is_ok(), "call conformance keys and evidence sources must root {ty:?}");
        }
    }

    #[test]
    fn checked_static_application_roots_follow_checked_body_identity() {
        use crate::static_definition::{CheckedStaticApplication, CheckedStaticApplicationArgument, StaticBodyId, StaticDefinitionSignature, StaticParameter};
        use crate::static_domain::StaticValueDomain;
        use crate::types::parameter::GenericParameterMode;

        let body_owner = DeclarationId::new(ModuleId::universe_root(), "StaticApplicationBodyRootOwner".into());
        let target = DeclarationId::new(ModuleId::universe_root(), "StaticApplicationTargetRoot".into());
        let mut store = TypeStore::new();
        let parameter_value_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "StaticApplicationParameterRoot".into()));
        let kinded_argument = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "StaticApplicationKindedArgumentRoot".into()));
        let result_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "StaticApplicationResultRoot".into()));
        let type_parameter_owner = crate::types::parameter::TypeParameterOwner::Declaration(target.clone());
        let type_parameter = store.intern_type_parameter(crate::types::parameter::TypeParameterData::new(
            type_parameter_owner.clone(),
            0,
            "T",
            crate::types::id::KindId::TYPE,
        ));
        let signature = StaticDefinitionSignature {
            parameters: vec![
                StaticParameter {
                    name: "value".into(),
                    domain: StaticValueDomain::ValueType(parameter_value_type),
                    mode: GenericParameterMode::Single,
                    index: 0,
                    type_parameter_id: None,
                    range: SourceRange { start: 1, end: 2 },
                },
                StaticParameter {
                    name: "T".into(),
                    domain: StaticValueDomain::Kind(store.kind_of(kinded_argument)),
                    mode: GenericParameterMode::Single,
                    index: 1,
                    type_parameter_id: Some(type_parameter),
                    range: SourceRange { start: 3, end: 4 },
                },
            ]
            .into_boxed_slice(),
            result_domain: StaticValueDomain::ValueType(result_type),
            generic_signature: Some(crate::types::parameter::GenericSignature::new(
                type_parameter_owner,
                vec![type_parameter].into_boxed_slice(),
            )),
            where_clause: None,
        };
        let body = StaticBodyId(body_owner.clone());
        let application = CheckedStaticApplication {
            id: crate::static_definition::StaticApplicationId {
                body: body.clone(),
                ordinal: 0,
            },
            expression: ExpressionId::new(BodyId(24), LocalExpressionId(3)),
            source_range: SourceRange { start: 5, end: 10 },
            target,
            signature,
            arguments: vec![
                CheckedStaticApplicationArgument::Value {
                    parameter_index: 0,
                    expression: ExpressionId::new(BodyId(24), LocalExpressionId(2)),
                },
                CheckedStaticApplicationArgument::Kinded {
                    parameter_index: 1,
                    term: kinded_argument,
                },
            ]
            .into_boxed_slice(),
        };

        let mut types = TypeExportContext::new(&store);
        let mut roots = Roots::new(
            &mut types,
            StableOwnerRef::Declaration(to_stable_declaration(&body_owner)),
            Family::StaticDefAndBinding,
        );
        let wrong_body = StaticBodyId(DeclarationId::new(ModuleId::universe_root(), "DifferentStaticBodyOwner".into()));
        assert!(roots.checked_static_application(&wrong_body, &application).is_err());
        roots.checked_static_application(&body, &application).unwrap();
        drop(roots);
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [parameter_value_type, kinded_argument, result_type] {
            assert!(types.ty(ty).is_ok(), "checked-body application signature must root type {ty:?}");
        }
        assert!(
            types.kind(store.kind_of(kinded_argument)).is_ok(),
            "checked-body application parameter domain must root its kind"
        );
        assert!(
            types.parameter(type_parameter).is_ok(),
            "checked-body application signature must root its kinded parameter"
        );
    }

    #[test]
    fn callable_analysis_match_and_pattern_products_register_recursive_type_roots() {
        use crate::match_semantics::{
            BranchProofEnvironment, CoverageWitness, ExhaustivenessResult, MatchArmResolution, MatchResolution, PatternBindingResolution, PatternResolution,
            PatternSiteKind, PatternSiteResolution, PatternSpaceSummary, PatternUsefulness, ResolvedFieldPattern, ResolvedVariantCandidate,
            ResolvedVariantPattern, VariantSelectorConstraint,
        };
        use crate::types::CaseInstantiation;
        use crate::types::id::RigidScopeId;
        use crate::types::rigid::LocalType;

        let owner = DeclarationId::new(ModuleId::universe_root(), "PatternRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let declaration = |name: &str| DeclarationId::new(ModuleId::universe_root(), name.into());
        let mut store = TypeStore::new();
        let scrutinee = store.nominal_type(declaration("PatternScrutineeRoot"));
        let initial = store.nominal_type(declaration("PatternInitialSpaceRoot"));
        let reachable = store.nominal_type(declaration("PatternReachableSpaceRoot"));
        let residual = store.nominal_type(declaration("PatternResidualSpaceRoot"));
        let result = store.nominal_type(declaration("PatternMatchResultRoot"));
        let branch = store.nominal_type(declaration("PatternBranchResultRoot"));
        let exact_case = store.nominal_type(declaration("PatternExactCaseRoot"));
        let payload = store.nominal_type(declaration("PatternPayloadLocalRoot"));
        let instantiated_result = store.nominal_type(declaration("PatternInstantiationResultRoot"));
        let field_type = store.nominal_type(declaration("PatternFieldKnowledgeRoot"));
        let field_local = store.nominal_type(declaration("PatternFieldLocalRoot"));
        let witness = store.nominal_type(declaration("PatternCoverageWitnessRoot"));
        let subject = store.nominal_type(declaration("PatternSiteSubjectRoot"));
        let binding_knowledge = store.nominal_type(declaration("PatternBindingKnowledgeRoot"));
        let binding_local = store.nominal_type(declaration("PatternBindingLocalRoot"));
        let variant = crate::identity::VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        let field = crate::identity::VariantFieldId::new(variant.clone(), 0);
        let match_expression = ExpressionId::new(BodyId(18), LocalExpressionId(1));

        let pattern = PatternResolution::Variant(ResolvedVariantPattern {
            owner: Some(owner),
            family: None,
            owner_candidates: Box::new([]),
            selector: VariantSelectorConstraint::WholeFamily,
            candidates: vec![ResolvedVariantCandidate {
                variant: variant.clone(),
                exact_case,
                fields: vec![ResolvedFieldPattern {
                    field,
                    field_type: TypeKnowledge::established(field_type, EvidenceOrigin::PatternDecomposition),
                    local_type: Some(LocalType::Canonical(field_local)),
                    child: Box::new(PatternResolution::Wildcard),
                }]
                .into_boxed_slice(),
                proof: BranchProofEnvironment::default(),
                case_instantiation: Some(CaseInstantiation {
                    variant: variant.clone(),
                    scope: RigidScopeId::from_index(0),
                    local_rigids: Default::default(),
                    payload_types: vec![LocalType::Canonical(payload)].into_boxed_slice(),
                    result_type: LocalType::Canonical(instantiated_result),
                    constraints: Box::new([]),
                }),
            }]
            .into_boxed_slice(),
            blocked: None,
        });
        let mut analysis = callable_analysis(callable);
        analysis.match_resolutions = Arc::new(
            [(
                match_expression,
                MatchResolution {
                    expression: match_expression,
                    scrutinee: TypeKnowledge::established(scrutinee, EvidenceOrigin::PatternDecomposition),
                    initial_space: PatternSpaceSummary::Union(
                        vec![
                            PatternSpaceSummary::Opaque(initial),
                            PatternSpaceSummary::Variant {
                                variant: variant.clone(),
                                exact_case: reachable,
                                fields: vec![PatternSpaceSummary::Opaque(residual)].into_boxed_slice(),
                            },
                        ]
                        .into_boxed_slice(),
                    ),
                    arms: vec![MatchArmResolution {
                        arm_index: 0,
                        pattern,
                        reachable_space: PatternSpaceSummary::Opaque(reachable),
                        residual_after: PatternSpaceSummary::Opaque(residual),
                        bindings: Box::new([]),
                        proof: BranchProofEnvironment::default(),
                        usefulness: PatternUsefulness::Useful,
                        branch_result: TypeKnowledge::established(branch, EvidenceOrigin::PatternDecomposition),
                    }]
                    .into_boxed_slice(),
                    result: TypeKnowledge::established(result, EvidenceOrigin::PatternDecomposition),
                    exhaustiveness: ExhaustivenessResult::Missing(vec![CoverageWitness::Opaque(witness)].into_boxed_slice()),
                },
            )]
            .into_iter()
            .collect(),
        );
        analysis.pattern_site_resolutions = Arc::new(
            [(
                crate::match_semantics::PatternSiteKey::new(PatternSiteKind::IfLet, SourceRange { start: 10, end: 14 }),
                PatternSiteResolution {
                    kind: PatternSiteKind::IfLet,
                    range: SourceRange { start: 10, end: 14 },
                    subject: TypeKnowledge::established(subject, EvidenceOrigin::PatternDecomposition),
                    pattern: PatternResolution::Binding {
                        binding: BindingId(31),
                        name: "bound".into(),
                        knowledge: TypeKnowledge::established(binding_knowledge, EvidenceOrigin::PatternDecomposition),
                    },
                    bindings: vec![PatternBindingResolution {
                        binding: BindingId(31),
                        name: "bound".into(),
                        knowledge: TypeKnowledge::established(binding_knowledge, EvidenceOrigin::PatternDecomposition),
                        local_type: Some(LocalType::Canonical(binding_local)),
                        source: SourceRange { start: 11, end: 12 },
                    }]
                    .into_boxed_slice(),
                    usefulness: PatternUsefulness::Useful,
                },
            )]
            .into_iter()
            .collect(),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [
            scrutinee,
            initial,
            reachable,
            residual,
            result,
            branch,
            exact_case,
            payload,
            instantiated_result,
            field_type,
            field_local,
            witness,
            subject,
            binding_knowledge,
            binding_local,
        ] {
            assert!(types.ty(ty).is_ok(), "match and pattern products must root every nested canonical type: {ty:?}");
        }
    }

    #[test]
    fn callable_analysis_dispatch_and_contextual_realization_register_type_roots() {
        use crate::checker::analysis::{
            AlgebraicBasisAxis, ContextualRealization, ContextualRealizationOperation, ExpressionAnalysis, RealizationCapabilitySelection,
            RealizationCoefficientSource, RealizationEvaluationPlan,
        };
        use crate::generic_evidence::{CompilerDerivedConformanceEvidence, CompilerDerivedConformanceKind, GenericConformanceEvidence, GenericConformanceKey};
        use crate::trait_dispatch::{AssumedTraitDispatchSelection, TraitDispatchConvergenceKey, TraitDispatchSelection, TraitDispatchSite};
        use crate::types::parameter::TypeTerm;

        let owner = DeclarationId::new(ModuleId::universe_root(), "DispatchRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let declaration = |name: &str| DeclarationId::new(ModuleId::universe_root(), name.into());
        let mut store = TypeStore::new();
        let exact_target = store.nominal_type(declaration("DispatchExactTargetRoot"));
        let trait_argument = store.nominal_type(declaration("DispatchTraitArgumentRoot"));
        let conformance_subject = store.nominal_type(declaration("DispatchConformanceSubjectRoot"));
        let selected_signature_result = store.nominal_type(declaration("DispatchSelectedSignatureRoot"));
        let assumed_subject = store.nominal_type(declaration("DispatchAssumedSubjectRoot"));
        let assumed_trait_argument = store.nominal_type(declaration("DispatchAssumedTraitArgumentRoot"));
        let assumed_signature_result = store.nominal_type(declaration("DispatchAssumedSignatureRoot"));
        let source_type = store.nominal_type(declaration("RealizationSourceRoot"));
        let target_type = store.nominal_type(declaration("RealizationTargetRoot"));
        let basis_type = store.nominal_type(declaration("RealizationBasisRoot"));
        let scalar_type = store.nominal_type(declaration("RealizationScalarRoot"));
        let conditional_receiver = store.nominal_type(declaration("ConditionalCapabilityReceiverRoot"));
        let conditional_owner_view = store.nominal_type(declaration("ConditionalCapabilityOwnerViewRoot"));
        let trait_owner = declaration("RealizationTraitRoot");
        let trait_ref = crate::traits::TraitRef::new(trait_owner.clone(), vec![trait_argument].into_boxed_slice());
        let assumed_trait_ref = crate::traits::TraitRef::new(trait_owner.clone(), vec![assumed_trait_argument].into_boxed_slice());
        let requirement = crate::traits::TraitRequirementId {
            owner: trait_owner.clone(),
            selector: Selector::method("realize", []).unwrap(),
            side: DispatchSide::Class,
        };
        let selected_callable = CallableId::method(owner.clone(), requirement.selector.clone(), DispatchSide::Class);
        let impl_id = crate::identity::ImplId::new(owner.module.clone(), crate::identity::ImplLocalId(1));
        let conformance = GenericConformanceKey::new(TypeTerm::Canonical(conformance_subject), trait_ref.clone());
        let evidence = GenericConformanceEvidence::CompilerDerived(Arc::new(CompilerDerivedConformanceEvidence {
            key: conformance.clone(),
            kind: CompilerDerivedConformanceKind::OrderedProduct,
            classification: crate::structural::StructuralClassification::OrderedProduct(crate::structural::OrderedProductForm::Tuple),
        }));
        let selection = TraitDispatchSelection {
            source_impl: impl_id.clone(),
            exact_target,
            exact_trait_ref: trait_ref.clone(),
            requirement: requirement.clone(),
            callable: Some(selected_callable.clone()),
            signature: callable_signature(selected_callable.clone(), &owner, selected_signature_result),
            evidence_fingerprint: crate::db::ProductFingerprint::new(17),
            evidence: evidence.clone(),
            selection: crate::impls::RequirementSelectionTemplate::ConformanceCallable {
                callable: selected_callable.clone(),
            },
            requirement_targets: Default::default(),
        };
        let candidate = crate::trait_dispatch::TraitEvidencedMemberCandidate {
            exact_target,
            exact_trait_ref: trait_ref,
            requirement: requirement.clone(),
            source_impl: impl_id.clone(),
            evidence,
            evidence_fingerprint: crate::db::ProductFingerprint::new(18),
            selection: crate::impls::RequirementSelectionTemplate::ConformanceCallable {
                callable: selected_callable.clone(),
            },
            signature: callable_signature(selected_callable.clone(), &owner, selected_signature_result),
            requirement_targets: Default::default(),
            convergence: TraitDispatchConvergenceKey::Inherent(selected_callable.clone()),
        };
        let assumed = AssumedTraitDispatchSelection {
            conformance: GenericConformanceKey::new(TypeTerm::Canonical(assumed_subject), assumed_trait_ref),
            requirement,
            signature: callable_signature(selected_callable.clone(), &owner, assumed_signature_result),
        };
        let expression_id = ExpressionId::new(BodyId(20), LocalExpressionId(3));
        let mut expression = ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 3, end: 5 },
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
        );
        expression.trait_dispatch = Some(TraitDispatchSite::Evidenced(selection.clone()));
        expression.trait_dispatch_candidates = Some(vec![candidate].into_boxed_slice());
        expression.contextual_realization = Some(Arc::new(ContextualRealization {
            source_expression: expression_id,
            source_range: SourceRange { start: 3, end: 5 },
            source_type,
            target_type,
            operation: ContextualRealizationOperation::Basis {
                basis_type,
                axis: AlgebraicBasisAxis::I,
                coefficient: RealizationCoefficientSource::One {
                    scalar_type,
                    one: RealizationCapabilitySelection::Evidenced(selection),
                },
                evaluation: RealizationEvaluationPlan::LiteralCoefficientThenAxis,
            },
            realization: RealizationCapabilitySelection::Assumed(assumed),
        }));
        expression.conditional_capability = Some(crate::dispatch::ConditionalSelectedCallCapability {
            selection: crate::dispatch::ConditionalDispatchSelection {
                impl_id: impl_id.clone(),
                callable: selected_callable,
                declaring_owner: owner.clone(),
                side: DispatchSide::Class,
            },
            applicability: crate::impls::InherentImplSpecialization {
                impl_id,
                receiver: conditional_receiver,
                owner_view: conditional_owner_view,
                bindings: Default::default(),
                environment: Default::default(),
                conformance_provenance: None,
            },
        });

        let mut analysis = callable_analysis(callable);
        analysis.expressions.insert_source(expression_id, expression).unwrap();
        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [
            exact_target,
            trait_argument,
            conformance_subject,
            selected_signature_result,
            assumed_subject,
            assumed_trait_argument,
            assumed_signature_result,
            source_type,
            target_type,
            basis_type,
            scalar_type,
            conditional_receiver,
            conditional_owner_view,
        ] {
            assert!(types.ty(ty).is_ok(), "dispatch and realization products must root nested type {ty:?}");
        }
    }

    #[test]
    fn callable_analysis_explanations_diagnostics_and_dependencies_register_type_roots() {
        use crate::diagnostic::{DiagnosticCode, DiagnosticGuidance, SemanticDiagnostic};
        use crate::explain::node::{DerivationRule, EvidenceRef, ExplanationStep};
        use crate::types::parameter::TypeTerm;

        let owner = DeclarationId::new(ModuleId::universe_root(), "ExplanationRootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let declaration = |name: &str| DeclarationId::new(ModuleId::universe_root(), name.into());
        let mut store = TypeStore::new();
        let dependency_subject = store.nominal_type(declaration("DependencySubjectRoot"));
        let dependency_trait_argument = store.nominal_type(declaration("DependencyTraitArgumentRoot"));
        let explanation_actual = store.nominal_type(declaration("ExplanationActualRoot"));
        let explanation_expected = store.nominal_type(declaration("ExplanationExpectedRoot"));
        let explanation_rule_type = store.nominal_type(declaration("ExplanationRuleTypeRoot"));
        let explanation_evidence_type = store.nominal_type(declaration("ExplanationEvidenceTypeRoot"));
        let diagnostic_guidance_type = store.nominal_type(declaration("DiagnosticGuidanceTypeRoot"));
        let key = crate::generic_evidence::GenericConformanceKey::new(
            TypeTerm::Canonical(dependency_subject),
            crate::traits::TraitRef::new(declaration("DependencyTraitRoot"), vec![dependency_trait_argument].into_boxed_slice()),
        );
        let mut arena = crate::explain::ExplanationArena::new();
        arena
            .alloc_full(
                ExplanationStep::Subtyping {
                    actual: explanation_actual,
                    expected: explanation_expected,
                    proven: false,
                },
                DerivationRule::GenericInstantiation {
                    type_args: vec![explanation_rule_type],
                },
                EvidenceStatus::Established,
                EvidenceOrigin::Flow,
                vec![EvidenceRef::TypeId(explanation_evidence_type)],
                Vec::new(),
            )
            .expect("test explanation arena is source mutable");
        let mut diagnostic = SemanticDiagnostic::error_in(
            owner.module.clone(),
            DiagnosticCode::ApplicationTooManyArguments,
            "test diagnostic",
            SourceRange { start: 1, end: 2 },
        );
        diagnostic.guidance.push(DiagnosticGuidance::ChangeAnnotation {
            range: SourceRange { start: 1, end: 2 },
            ty: diagnostic_guidance_type,
        });
        let mut analysis = callable_analysis(callable);
        analysis.explanations = Arc::new(arena);
        analysis.diagnostics = Arc::from([diagnostic]);
        analysis.semantic_dependencies = Arc::from([crate::checker::analysis::SemanticDependency::GenericConformance(key)]);

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        for ty in [
            dependency_subject,
            dependency_trait_argument,
            explanation_actual,
            explanation_expected,
            explanation_rule_type,
            explanation_evidence_type,
            diagnostic_guidance_type,
        ] {
            assert!(types.ty(ty).is_ok(), "explanations, diagnostics, and dependencies must root type {ty:?}");
        }
    }

    #[test]
    fn callable_analysis_registers_each_denotation_graph_root_before_closure() {
        use crate::checker::associated::BehavioralFamilySpec;
        use crate::types::denotation::{AssociatedValueDenotation, BehavioralFamilyDenotation, SemanticDenotation};
        use crate::types::id::KindId;
        use crate::types::store::StaticArgument;
        use std::sync::Arc;

        let owner = DeclarationId::new(ModuleId::universe_root(), "RootOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let declaration = |name: &str| DeclarationId::new(ModuleId::universe_root(), name.into());
        let mut store = TypeStore::new();

        let type_form = store.nominal_type(declaration("TypeFormRoot"));
        let static_term = store.const_int(17);
        let kind = store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
        let domain = store.nominal_static_domain(declaration("StaticDomainRoot"));

        let associated_owner = declaration("AssociatedOwnerRoot");
        let associated_owner_form = store.nominal_type(associated_owner.clone());
        let associated = AssociatedValueDenotation::exact(
            associated_owner_form,
            associated_owner.clone(),
            crate::associated::AssociatedMemberId::Variant(crate::identity::VariantId::new(associated_owner, Selector::getter("Ready").unwrap())),
            None,
        );

        let receiver_type = store.nominal_type(declaration("BehaviorReceiverRoot"));
        let application_argument = store.nominal_type(declaration("BehaviorApplicationArgument"));
        let constructor_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let receiver_form = store.nominal_form(declaration("BehaviorApplicationRoot"), constructor_kind);
        let receiver_application = store.apply_static_form(receiver_form, &[StaticArgument::Term(application_argument)]).unwrap();
        let behavioral = BehavioralFamilyDenotation {
            receiver_type,
            receiver_application: Some(receiver_application),
            spec: BehavioralFamilySpec::Exact(Selector::method("run", []).unwrap()),
            members: Arc::from([]),
        };

        let mut analysis = callable_analysis(callable);
        let denotations = [
            SemanticDenotation::TypeForm(type_form),
            SemanticDenotation::StaticTerm(static_term),
            SemanticDenotation::Kind(kind),
            SemanticDenotation::StaticDomain(domain),
            SemanticDenotation::AssociatedValue(Box::new(associated)),
            SemanticDenotation::BehavioralFamily(Box::new(behavioral)),
        ];
        for (ordinal, denotation) in denotations.into_iter().enumerate() {
            let expression_id = ExpressionId::new(BodyId(6), LocalExpressionId(ordinal as u32));
            analysis
                .expressions
                .insert_source(
                    expression_id,
                    ExpressionAnalysis::ready(
                        expression_id,
                        SourceRange {
                            start: ordinal,
                            end: ordinal + 1,
                        },
                        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    )
                    .with_denotation(Some(denotation)),
                )
                .unwrap();
        }

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &phalcom_compiler_artifact::ValidationLimits::default()).unwrap();

        assert!(types.ty(type_form).is_ok(), "TypeForm references are roots");
        assert!(types.ty(static_term).is_ok(), "StaticTerm references are type-graph roots");
        assert!(types.kind(kind).is_ok(), "Kind references are roots");
        assert!(types.domain(domain).is_ok(), "StaticDomain references are roots");
        assert!(types.ty(associated_owner_form).is_ok(), "associated captures retain their owner form");
        assert!(types.ty(receiver_type).is_ok(), "behavioral captures retain their receiver type");
        assert!(types.ty(receiver_application).is_ok(), "behavioral captures retain their optional application");
    }

    #[test]
    fn callable_analysis_static_application_denotations_remain_fail_closed() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "StaticOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut analysis = callable_analysis(callable);
        let expression_id = ExpressionId::new(BodyId(5), LocalExpressionId(1));
        analysis
            .expressions
            .insert_source(
                expression_id,
                ExpressionAnalysis::ready(
                    expression_id,
                    SourceRange { start: 3, end: 7 },
                    TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                )
                .with_denotation(Some(SemanticDenotation::StaticApplication(crate::static_definition::StaticApplicationId {
                    body: crate::static_definition::StaticBodyId(owner),
                    ordinal: 0,
                }))),
            )
            .unwrap();

        let store = TypeStore::new();
        let mut types = TypeExportContext::new(&store);
        assert!(
            register_callable_analysis_roots(&mut types, &analysis, &store).is_err(),
            "static applications stay blocked until checked rows have a retained, body-qualified owner"
        );
    }

    #[test]
    fn callable_blocked_exact_case_query_owner_rejects_missing_store_identity() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "MissingVariantRoot".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let expression_id = ExpressionId::new(BodyId(9), LocalExpressionId(0));
        let mut analysis = callable_analysis(callable);
        let variant = VariantId::new(owner, Selector::getter("Absent").unwrap());
        let mut expression = ExpressionAnalysis::ready(
            expression_id,
            SourceRange { start: 0, end: 1 },
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
        );
        expression.status = crate::checker::analysis::AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnavailableSemanticOwner(
            crate::db::QueryKey::TraitDispatchBucket(crate::trait_dispatch::TraitDispatchBucketKey {
                target_family: crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant),
                selector: Selector::getter("blocked").unwrap(),
                side: DispatchSide::Instance,
            }),
        ));
        analysis.expressions.insert_source(expression_id, expression).unwrap();

        let store = TypeStore::new();
        let mut types = TypeExportContext::new(&store);
        assert!(
            register_callable_analysis_roots(&mut types, &analysis, &store).is_err(),
            "a stable query identity absent from the source type store cannot be fabricated during export"
        );
    }
}
