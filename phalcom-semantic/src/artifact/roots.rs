//! Stable primary-owner roots collected before either relocation graph closes.
use super::export::register_structural_primary_roots;
use crate::declaration_type::{DeclaredTypeFact, DeclaredTypeState};
use crate::metadata::stable_identity::{to_stable_callable, to_stable_declaration, to_stable_impl, to_stable_variant};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::StaticExportContext;
use crate::types::parameter::{GenericConstraint, GenericSignature, TypeTerm};
use crate::types::store::artifact::TypeExportContext;
use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::semantic::type_store::{CanonicalRootFamilyTag as Family, CanonicalTypeRootKey, StableOwnerRef};

struct Roots<'a> {
    types: &'a mut TypeExportContext,
    owner: StableOwnerRef,
    family: Family,
    ordinal: u32,
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

/// Registers type and row roots carried by already-selected call specializations.
/// Dynamic arguments intentionally add no proper-type root.
pub(super) fn register_callable_analysis_roots(
    types: &mut TypeExportContext,
    analysis: &crate::checker::analysis::CallableAnalysis,
) -> Result<(), ArtifactError> {
    let mut roots = Roots::new(
        types,
        StableOwnerRef::Callable(to_stable_callable(&analysis.callable)),
        Family::CallableAnalysis,
    );
    for expression in analysis.expressions.values() {
        roots.knowledge(&expression.knowledge)?;
        if let Some(denotation) = &expression.denotation {
            roots.denotation(denotation)?;
        }
        let Some(specialization) = &expression.call_specialization else {
            continue;
        };
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
    }
    for binding in analysis.bindings.values() {
        if let Some(contract) = &binding.contract {
            roots.ty(contract.ty)?;
        }
        roots.knowledge(&binding.current)?;
        if let Some(denotation) = &binding.denotation {
            roots.denotation(denotation)?;
        }
    }
    roots.flow_state(&analysis.entry_flow)?;
    for exit in &analysis.exits.normal_returns {
        roots.knowledge(&exit.knowledge)?;
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
    Ok(())
}

impl<'a> Roots<'a> {
    fn new(types: &'a mut TypeExportContext, owner: StableOwnerRef, family: Family) -> Self {
        Self {
            types,
            owner,
            family,
            ordinal: 100,
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
            if let Some(contract) = &binding.contract {
                self.ty(contract.ty)?;
            }
        }
        for field in flow.fields.values() {
            self.knowledge(&field.contract)?;
            self.knowledge(&field.current)?;
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
                self.generic_conformance_key(&binding.callee_conformance)?;
                match &binding.source {
                    crate::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                        self.generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::Exact(evidence.clone()))?;
                    }
                    crate::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } => {
                        self.generic_conformance_key(caller_conformance)?;
                    }
                    crate::checker::analysis::GenericCallConformanceSource::Derived { evidence } => {
                        self.generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::Derived(evidence.clone()))?;
                    }
                    crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { evidence } => {
                        self.generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence.clone()))?;
                    }
                }
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
    for (id, analysis) in snapshot.callable_analyses.iter() {
        if id != &analysis.callable {
            return Err(ArtifactError::LimitExceeded("callable analysis table key disagrees with its owner".into()));
        }
        register_callable_analysis_roots(types, analysis)?;
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
    for (decl_id, _surface) in snapshot.surfaces.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(decl_id)), Family::SurfaceAndDispatch);
        for (_, field_sig) in snapshot.field_signatures.iter_owner(decl_id) {
            r.fact(&field_sig.declared_type)?;
        }
    }
    for (id, surface) in snapshot.trait_surfaces.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::TraitAndConformance);
        r.generic(&surface.generic_signature)?;
        if let Some(generics) = &surface.generic_signature {
            for constraint in &generics.constraints {
                if let GenericConstraint::Conforms { subject, trait_ref } = constraint {
                    if matches!(subject, TypeTerm::SelfType(_)) {
                        for &arg in &trait_ref.arguments {
                            r.ty(arg)?;
                        }
                    }
                }
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
    for (id, plan) in snapshot.conformance_witness_plans.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(id)), Family::ConstructorContract);
        r.ty(plan.target_template)?;
        r.generic(&plan.generic_signature)?;
        for &ty in &plan.trait_ref_template.arguments {
            r.ty(ty)?;
        }
        for selection in plan.requirements.values() {
            if let crate::impls::RequirementSelectionTemplate::DataComponent { specialized_type, .. } = selection {
                r.ty(*specialized_type)?;
            }
        }
        for template in plan.associated_type_plan.bindings.values() {
            if let crate::impls::AssociatedTypeBindingValue::Type(ty) = template.value_template {
                r.ty(ty)?;
            }
        }
    }
    for (_, product) in snapshot.static_definitions.iter() {
        if let Some(body) = &product.checked_body {
            if let crate::static_definition::StaticDefinitionBodyRepr::Operational(template) = &body.root {
                super::operational::register_operational_template_roots(template, statics, types)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::register_callable_analysis_roots;
    use crate::checker::analysis::{BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, FlowStateSummary};
    use crate::identity::{BindingId, BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, FieldId, LocalExpressionId, ModuleId};
    use crate::types::denotation::SemanticDenotation;
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
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
    fn callable_analysis_denotation_types_are_roots_before_graph_closure() {
        let owner = DeclarationId::new(ModuleId::universe_root(), "Rooted".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let mut store = TypeStore::new();
        let denotation_only_type = store.nominal_type(owner);
        let expression_id = ExpressionId::new(BodyId(4), LocalExpressionId(2));
        let mut analysis = callable_analysis(callable);
        analysis.expressions.insert(
            expression_id,
            ExpressionAnalysis::ready(
                expression_id,
                SourceRange { start: 1, end: 2 },
                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
            )
            .with_denotation(Some(SemanticDenotation::TypeForm(denotation_only_type))),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        analysis.expressions.insert(
            expression_id,
            ExpressionAnalysis::ready(
                expression_id,
                SourceRange { start: 2, end: 5 },
                TypeKnowledge::established(expression_type, EvidenceOrigin::Flow),
            ),
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        analysis.bindings.insert(
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
        );

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
            analysis.expressions.insert(
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
            );
        }

        let mut types = TypeExportContext::new(&store);
        register_callable_analysis_roots(&mut types, &analysis).unwrap();
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
        analysis.expressions.insert(
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
        );

        let mut types = TypeExportContext::new(&TypeStore::new());
        assert!(
            register_callable_analysis_roots(&mut types, &analysis).is_err(),
            "static applications stay blocked until checked rows have a retained, body-qualified owner"
        );
    }
}
