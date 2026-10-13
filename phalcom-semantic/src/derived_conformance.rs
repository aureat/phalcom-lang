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
    parent: Option<Arc<Self>>,
    rules_by_trait: BTreeMap<DeclarationId, Vec<Arc<DerivedConformanceRule>>>,
    rules_by_impl: BTreeMap<ImplId, Arc<DerivedConformanceRule>>,
}

/// Borrowed canonical-first trait bucket with indexed and iterable access.
/// Both slices remain in their owning tables; constructing the view copies no rules.
pub struct DerivedRuleView<'a> {
    parent: &'a [Arc<DerivedConformanceRule>],
    local: &'a [Arc<DerivedConformanceRule>],
}

impl DerivedRuleView<'_> {
    /// Number of rules visible across the canonical and local buckets.
    pub fn len(&self) -> usize {
        self.parent.len() + self.local.len()
    }

    /// Whether neither bucket contains a rule.
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty() && self.local.is_empty()
    }
}

impl std::ops::Index<usize> for DerivedRuleView<'_> {
    type Output = Arc<DerivedConformanceRule>;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.parent.len() {
            &self.parent[index]
        } else {
            &self.local[index - self.parent.len()]
        }
    }
}

impl<'a> IntoIterator for DerivedRuleView<'a> {
    type Item = &'a Arc<DerivedConformanceRule>;
    type IntoIter = std::iter::Chain<std::slice::Iter<'a, Arc<DerivedConformanceRule>>, std::slice::Iter<'a, Arc<DerivedConformanceRule>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.parent.iter().chain(self.local.iter())
    }
}

impl DerivedConformanceRuleIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Shares canonical rules without cloning trait buckets or impl entries.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        assert!(parent.parent.is_none(), "derived rule parent must be a canonical root");
        Self {
            parent: Some(parent),
            ..Self::default()
        }
    }

    pub fn insert(&mut self, rule: DerivedConformanceRule) {
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.get_rule(&rule.impl_id).is_some()),
            "cannot overwrite a canonical derived conformance rule"
        );
        self.remove(&rule.impl_id);
        let rule_arc = Arc::new(rule);
        self.rules_by_trait
            .entry(rule_arc.trait_ref.declaration.clone())
            .or_default()
            .push(rule_arc.clone());
        self.rules_by_impl.insert(rule_arc.impl_id.clone(), rule_arc);
    }

    pub fn rules_for_trait(&self, trait_declaration: &DeclarationId) -> DerivedRuleView<'_> {
        DerivedRuleView {
            parent: self
                .parent
                .as_ref()
                .and_then(|parent| parent.rules_by_trait.get(trait_declaration))
                .map_or(&[], Vec::as_slice),
            local: self.rules_by_trait.get(trait_declaration).map_or(&[], Vec::as_slice),
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
        self.rules_by_impl.retain(|impl_id, _| &impl_id.module != module);
        for rules in self.rules_by_trait.values_mut() {
            rules.retain(|rule| &rule.source_module != module);
        }
        self.rules_by_trait.retain(|_, rules| !rules.is_empty());
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &Arc<DerivedConformanceRule>> + '_> {
        match &self.parent {
            Some(parent) => {
                let mut canonical = parent.rules_by_impl.values().peekable();
                let mut local = self.rules_by_impl.values().peekable();
                Box::new(std::iter::from_fn(move || match (canonical.peek(), local.peek()) {
                    (Some(left), Some(right)) if left.impl_id <= right.impl_id => canonical.next(),
                    (Some(_), Some(_)) => local.next(),
                    (Some(_), None) => canonical.next(),
                    (None, Some(_)) => local.next(),
                    (None, None) => None,
                }))
            }
            None => Box::new(self.rules_by_impl.values()),
        }
    }

    /// Number of workspace-owned rules.
    pub fn local_len(&self) -> usize {
        self.rules_by_impl.len()
    }

    pub fn len(&self) -> usize {
        self.local_len() + self.parent.as_ref().map_or(0, |parent| parent.len())
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

#[cfg(test)]
mod parent_tests {
    use super::*;
    use crate::identity::ImplLocalId;
    use crate::types::parameter::TypeParameterOwner;

    fn rule(module: ModuleId, ordinal: u32, trait_declaration: DeclarationId) -> DerivedConformanceRule {
        let impl_id = ImplId::new(module.clone(), ImplLocalId(ordinal));
        DerivedConformanceRule::new(
            impl_id.clone(),
            module.clone(),
            TypeParameterId(0),
            TypeId(0),
            TraitRef::new(trait_declaration, Vec::new()),
            GenericSignature {
                owner: TypeParameterOwner::Impl(impl_id.clone()),
                parameters: Box::new([]),
                parameter_kinds: Box::new([]),
                parameter_kind_shapes: Box::new([]),
                parameter_variances: Box::new([]),
                parameter_modes: Box::new([]),
                constraint_shapes: Box::new([]),
                constraints: Box::new([]),
            },
            ConformanceAssociatedTypePlan::empty(impl_id),
            BTreeMap::new(),
            BTreeMap::new(),
            SemanticSourceSpan::new(module, phalcom_common::range::SourceRange::default()),
        )
    }

    #[test]
    fn layered_rule_queries_preserve_canonical_and_local_evidence() {
        let canonical_module = ModuleId::universe_root();
        let local_module = ModuleId::resolved(phalcom_modules::ResolvedProjectId::from_raw(99), phalcom_modules::ModulePath::root());
        let trait_id = DeclarationId::new(canonical_module.clone(), "T".into());
        let canonical_rule = rule(canonical_module.clone(), 0, trait_id.clone());
        let local_rule = rule(local_module.clone(), 1, trait_id.clone());
        let mut parent = DerivedConformanceRuleIndex::new();
        parent.insert(canonical_rule.clone());
        let parent = Arc::new(parent);
        let mut overlay = DerivedConformanceRuleIndex::with_parent(parent.clone());
        assert_eq!(overlay.rules_for_trait(&trait_id).len(), 1);
        overlay.insert(local_rule.clone());
        let ids = overlay.rules_for_trait(&trait_id).into_iter().map(|rule| rule.impl_id.clone()).collect::<Vec<_>>();
        assert_eq!(ids, vec![canonical_rule.impl_id.clone(), local_rule.impl_id.clone()]);
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(overlay.iter().map(|rule| rule.impl_id.clone()).collect::<Vec<_>>(), sorted);
        assert_eq!(overlay.local_len(), 1);
        overlay.remove_source(&canonical_module);
        assert_eq!(overlay.rules_for_trait(&trait_id).len(), 2);
        overlay.remove_source(&local_module);
        assert_eq!(overlay.rules_for_trait(&trait_id).len(), 1);
        assert_eq!(parent.rules_for_trait(&trait_id).len(), 1);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| overlay.insert(canonical_rule))).is_err());
    }
}
