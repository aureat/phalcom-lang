use crate::db::key::ProductFingerprint;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{DeclarationId, ImplId, ModuleId, TypeId};
use crate::impls::{ConformanceAssociatedTypePlan, RequirementSelectionTemplate};
use crate::traits::{InstantiatedTraitRequirement, TraitRef, TraitRequirementId};
use crate::types::id::TypeParameterId;
use crate::types::parameter::GenericSignature;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Resolved head of a guarded generic-target conformance `impl<I> Trait for I where ...`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDerivedConformanceRuleHead {
    pub impl_id: ImplId,
    pub source_module: ModuleId,
    pub target_parameter: TypeParameterId,
    pub target_template: TypeId,
    pub trait_ref: TraitRef,
    pub generic_signature: GenericSignature,
    pub source: SemanticSourceSpan,
}

/// First-class guarded derived conformance rule product.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedConformanceRule {
    pub impl_id: ImplId,
    pub source_module: ModuleId,
    pub target_parameter: TypeParameterId,
    pub target_template: TypeId,
    pub trait_ref: TraitRef,
    pub generic_signature: GenericSignature,
    pub associated_type_plan: ConformanceAssociatedTypePlan,
    pub requirement_views: BTreeMap<TraitRequirementId, InstantiatedTraitRequirement>,
    pub requirements: BTreeMap<TraitRequirementId, RequirementSelectionTemplate>,
    pub fingerprint: ProductFingerprint,
    pub source: SemanticSourceSpan,
}

impl DerivedConformanceRule {
    pub fn new(
        impl_id: ImplId,
        source_module: ModuleId,
        target_parameter: TypeParameterId,
        target_template: TypeId,
        trait_ref: TraitRef,
        generic_signature: GenericSignature,
        associated_type_plan: ConformanceAssociatedTypePlan,
        requirement_views: BTreeMap<TraitRequirementId, InstantiatedTraitRequirement>,
        requirements: BTreeMap<TraitRequirementId, RequirementSelectionTemplate>,
        source: SemanticSourceSpan,
    ) -> Self {
        let mut rule = Self {
            impl_id,
            source_module,
            target_parameter,
            target_template,
            trait_ref,
            generic_signature,
            associated_type_plan,
            requirement_views,
            requirements,
            fingerprint: ProductFingerprint::default(),
            source,
        };
        rule.fingerprint = derived_conformance_rule_fingerprint(&rule);
        rule
    }
}

/// Deterministic index of guarded derived conformance rules, keyed by consequent trait declaration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DerivedConformanceRuleIndex {
    rules_by_trait: BTreeMap<DeclarationId, Vec<Arc<DerivedConformanceRule>>>,
    rules_by_impl: BTreeMap<ImplId, Arc<DerivedConformanceRule>>,
}

impl DerivedConformanceRuleIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, rule: DerivedConformanceRule) {
        let rule_arc = Arc::new(rule);
        self.rules_by_trait
            .entry(rule_arc.trait_ref.declaration.clone())
            .or_default()
            .push(rule_arc.clone());
        self.rules_by_impl.insert(rule_arc.impl_id.clone(), rule_arc);
    }

    pub fn rules_for_trait(&self, trait_declaration: &DeclarationId) -> &[Arc<DerivedConformanceRule>] {
        self.rules_by_trait.get(trait_declaration).map_or(&[], |v| v.as_slice())
    }

    pub fn get_rule(&self, impl_id: &ImplId) -> Option<&Arc<DerivedConformanceRule>> {
        self.rules_by_impl.get(impl_id)
    }

    pub fn remove_source(&mut self, module: &ModuleId) {
        self.rules_by_impl.retain(|impl_id, _| &impl_id.module != module);
        for rules in self.rules_by_trait.values_mut() {
            rules.retain(|rule| &rule.source_module != module);
        }
        self.rules_by_trait.retain(|_, rules| !rules.is_empty());
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<DerivedConformanceRule>> {
        self.rules_by_impl.values()
    }

    pub fn len(&self) -> usize {
        self.rules_by_impl.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules_by_impl.is_empty()
    }
}

/// Computes the canonical semantic fingerprint for a derived conformance rule.
/// Parameter names and source ranges are deliberately abstracted to ensure
/// alpha-equivalent rules have identical fingerprints.
pub fn derived_conformance_rule_fingerprint(rule: &DerivedConformanceRule) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    rule.trait_ref.declaration.hash(&mut hasher);
    rule.trait_ref.arguments.hash(&mut hasher);
    rule.generic_signature.parameters.len().hash(&mut hasher);
    rule.generic_signature.constraints.hash(&mut hasher);
    for (req, sel) in &rule.requirements {
        req.hash(&mut hasher);
        match sel {
            RequirementSelectionTemplate::ConformanceCallable { callable } => {
                0u8.hash(&mut hasher);
                callable.hash(&mut hasher);
            }
            RequirementSelectionTemplate::TraitDefault { callable } => {
                1u8.hash(&mut hasher);
                callable.hash(&mut hasher);
            }
            RequirementSelectionTemplate::InherentCallable { callable, .. } => {
                2u8.hash(&mut hasher);
                callable.hash(&mut hasher);
            }
            RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => {
                3u8.hash(&mut hasher);
                candidate.callable.hash(&mut hasher);
            }
            RequirementSelectionTemplate::DataComponent { component, .. } => {
                4u8.hash(&mut hasher);
                component.hash(&mut hasher);
            }
        }
    }
    for (req, binding) in &rule.associated_type_plan.bindings {
        req.hash(&mut hasher);
        binding.value_template.hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}
