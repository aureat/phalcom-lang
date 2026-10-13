use crate::db::key::ProductFingerprint;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{DeclarationId, ImplId, ModuleId, TypeId};
use crate::impls::{ConformanceAssociatedTypePlan, RequirementSelectionTemplate};
use crate::traits::{InstantiatedTraitRequirement, TraitRef, TraitRequirementId};
use crate::types::id::TypeParameterId;
use crate::types::parameter::GenericSignature;
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
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
    /// Read-only canonical rule authority; local revisions own only new rules.
    parent: Option<Arc<DerivedConformanceRuleIndex>>,
    rules_by_trait: BTreeMap<DeclarationId, Vec<Arc<DerivedConformanceRule>>>,
    rules_by_impl: BTreeMap<ImplId, Arc<DerivedConformanceRule>>,
}

/// Borrowed, allocation-free view of canonical and workspace rules for one trait.
/// Rule iteration is parent-first, preserving deterministic insertion order.
#[derive(Clone, Copy, Debug)]
pub struct DerivedRuleTraitView<'a> {
    index: &'a DerivedConformanceRuleIndex,
    trait_declaration: &'a DeclarationId,
}

impl DerivedRuleTraitView<'_> {
    /// Number of matching canonical and local rules without materializing a slice.
    pub fn len(&self) -> usize {
        self.index.trait_rule_len(self.trait_declaration)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Exact slot in parent-first order.
    pub fn get(&self, index: usize) -> Option<&Arc<DerivedConformanceRule>> {
        self.index.trait_rule_at(self.trait_declaration, index)
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &Arc<DerivedConformanceRule>> + '_> {
        self.index.trait_rule_iter(self.trait_declaration)
    }
}

impl std::ops::Index<usize> for DerivedRuleTraitView<'_> {
    type Output = Arc<DerivedConformanceRule>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("derived conformance rule index out of bounds")
    }
}

impl<'a> IntoIterator for DerivedRuleTraitView<'a> {
    type Item = &'a Arc<DerivedConformanceRule>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.index.trait_rule_iter(self.trait_declaration)
    }
}

impl DerivedConformanceRuleIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Shares immutable canonical rule buckets instead of cloning each entry.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            ..Self::default()
        }
    }

    pub fn local_len(&self) -> usize {
        self.rules_by_impl.len()
    }

    pub fn parent_len(&self) -> usize {
        self.parent.as_ref().map_or(0, |parent| parent.len())
    }

    pub fn insert(&mut self, rule: DerivedConformanceRule) {
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.get_rule(&rule.impl_id).is_some()),
            "cannot overwrite a canonical derived conformance rule"
        );
        // Local re-publication must replace its old trait bucket membership.
        self.remove(&rule.impl_id);
        let rule_arc = Arc::new(rule);
        self.rules_by_trait
            .entry(rule_arc.trait_ref.declaration.clone())
            .or_default()
            .push(rule_arc.clone());
        self.rules_by_impl.insert(rule_arc.impl_id.clone(), rule_arc);
    }

    /// A trait-local view without copying parent buckets or merged rule slices.
    pub fn rules_for_trait<'a>(&'a self, trait_declaration: &'a DeclarationId) -> DerivedRuleTraitView<'a> {
        DerivedRuleTraitView { index: self, trait_declaration }
    }

    fn trait_rule_len(&self, trait_declaration: &DeclarationId) -> usize {
        self.parent.as_ref().map_or(0, |parent| parent.trait_rule_len(trait_declaration))
            + self.rules_by_trait.get(trait_declaration).map_or(0, Vec::len)
    }

    fn trait_rule_at(&self, trait_declaration: &DeclarationId, position: usize) -> Option<&Arc<DerivedConformanceRule>> {
        let inherited = self.parent.as_ref().map_or(0, |parent| parent.trait_rule_len(trait_declaration));
        if position < inherited {
            return self.parent.as_ref().and_then(|parent| parent.trait_rule_at(trait_declaration, position));
        }
        self.rules_by_trait.get(trait_declaration).and_then(|rules| rules.get(position - inherited))
    }

    fn trait_rule_iter<'a>(&'a self, trait_declaration: &'a DeclarationId) -> Box<dyn Iterator<Item = &'a Arc<DerivedConformanceRule>> + 'a> {
        let local = self.rules_by_trait.get(trait_declaration).into_iter().flat_map(|rules| rules.iter());
        match &self.parent {
            Some(parent) => Box::new(parent.trait_rule_iter(trait_declaration).chain(local)),
            None => Box::new(local),
        }
    }

    pub fn get_rule(&self, impl_id: &ImplId) -> Option<&Arc<DerivedConformanceRule>> {
        self.rules_by_impl.get(impl_id).or_else(|| self.parent.as_ref().and_then(|parent| parent.get_rule(impl_id)))
    }

    pub fn remove(&mut self, impl_id: &ImplId) {
        let Some(rule) = self.rules_by_impl.remove(impl_id) else {
            return;
        };
        let remove_trait_bucket = if let Some(rules) = self.rules_by_trait.get_mut(&rule.trait_ref.declaration) {
            rules.retain(|candidate| candidate.impl_id != *impl_id);
            rules.is_empty()
        } else {
            false
        };
        if remove_trait_bucket {
            self.rules_by_trait.remove(&rule.trait_ref.declaration);
        }
    }

    pub fn remove_source(&mut self, module: &ModuleId) {
        let removed = self
            .rules_by_impl
            .iter()
            .filter(|(_, rule)| &rule.source_module == module)
            .map(|(impl_id, _)| impl_id.clone())
            .collect::<Vec<_>>();
        for impl_id in removed {
            self.remove(&impl_id);
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &Arc<DerivedConformanceRule>> + '_> {
        match &self.parent {
            Some(parent) => Box::new(parent.iter().chain(self.rules_by_impl.values())),
            None => Box::new(self.rules_by_impl.values()),
        }
    }

    pub fn len(&self) -> usize {
        self.local_len() + self.parent_len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Computes the canonical semantic fingerprint for a derived conformance rule.
/// Parameter names and source ranges are deliberately abstracted to ensure
/// alpha-equivalent rules have identical fingerprints.
pub fn derived_conformance_rule_fingerprint(rule: &DerivedConformanceRule) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    rule.trait_ref.declaration.hash(&mut hasher);
    rule.trait_ref.arguments.hash(&mut hasher);
    let mut sorted_constraints = rule.generic_signature.constraints.to_vec();
    sorted_constraints.sort();
    sorted_constraints.hash(&mut hasher);
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
