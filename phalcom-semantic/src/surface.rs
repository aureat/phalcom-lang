//! Semantic declaration and module surface definitions.

use crate::dispatch::CallableSignature;
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use crate::types::evidence::TypeKnowledge;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};
use phalcom_common::selector::Selector;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Source-mutable or artifact-frozen map backing used by member surfaces.
/// Frozen entries retain the same ordered read contract without rebuilding a
/// native hash table during artifact import.
#[derive(Clone, Debug)]
pub struct SurfaceMap<K, V> {
    source: HashMap<K, V>,
    frozen: Option<Arc<BudgetedSortedMap<K, V>>>,
}

impl<K, V> Default for SurfaceMap<K, V> {
    fn default() -> Self {
        Self { source: HashMap::new(), frozen: None }
    }
}

impl<K: Eq + Hash + Ord, V> SurfaceMap<K, V> {
    pub fn from_frozen_entries_with_allocation_budget(
        mut entries: Vec<(K, V)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, FrozenSurfaceMapError> {
        if entries.is_empty() {
            return Ok(Self::default());
        }
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let entries = BudgetedSortedMap::from_sorted_entries(entries).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => FrozenSurfaceMapError::DuplicateKey,
            SortedMapEntriesError::OutOfOrder => unreachable!("entries were sorted before frozen construction"),
        })?;
        let frozen = allocations.arc(entries).map_err(FrozenSurfaceMapError::Allocation)?;
        Ok(Self { source: HashMap::new(), frozen: Some(frozen) })
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + Ord + ?Sized,
    {
        self.source.get(key).or_else(|| self.frozen.as_ref().and_then(|entries| entries.get(key)))
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + Ord + ?Sized,
    {
        self.source.get_mut(key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        assert!(
            !self.frozen.as_ref().is_some_and(|entries| entries.contains_key(&key)),
            "cannot overwrite a frozen member-surface entry"
        );
        self.source.insert(key, value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + Ord + ?Sized,
    {
        self.source.remove(key)
    }

    pub fn len(&self) -> usize {
        self.source.len() + self.frozen.as_ref().map_or(0, |entries| entries.len())
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.frozen.as_ref().is_none_or(|entries| entries.is_empty())
    }

    pub(crate) fn is_frozen(&self) -> bool {
        self.frozen.is_some()
    }

    pub fn iter(&self) -> SurfaceMapIter<'_, K, V> {
        match &self.frozen {
            Some(entries) => SurfaceMapIter::Frozen(entries.as_entries().iter()),
            None => SurfaceMapIter::Source(self.source.iter()),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }
}

pub enum SurfaceMapIter<'a, K, V> {
    Source(std::collections::hash_map::Iter<'a, K, V>),
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for SurfaceMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(key, value)| (key, value)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Source(iter) => iter.size_hint(),
            Self::Frozen(iter) => iter.size_hint(),
        }
    }
}

impl<K, V> ExactSizeIterator for SurfaceMapIter<'_, K, V> {}

impl<K, V, Q> std::ops::Index<&Q> for SurfaceMap<K, V>
where
    K: Eq + Hash + Ord + std::borrow::Borrow<Q>,
    Q: Hash + Eq + Ord + ?Sized,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("member-surface entry is present")
    }
}

impl<'a, K: Eq + Hash + Ord, V> IntoIterator for &'a SurfaceMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = SurfaceMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub enum FrozenSurfaceMapError {
    Allocation(AllocationError),
    DuplicateKey,
}

/// Published member surface for a specific dispatch side (instance or class).
#[derive(Clone, Debug, Default)]
pub struct MemberSurface {
    pub fields: SurfaceMap<String, TypeKnowledge>,
    pub field_ids: SurfaceMap<FieldId, TypeKnowledge>,
    pub fields_by_name: SurfaceMap<String, FieldId>,
    pub field_visibility: SurfaceMap<String, MemberVisibility>,
    pub callables: SurfaceMap<CallableId, TypeKnowledge>,
    pub callable_signatures: SurfaceMap<Selector, CallableSignature>,
    pub callables_by_selector: SurfaceMap<Selector, CallableId>,
    pub callable_visibility: SurfaceMap<Selector, MemberVisibility>,
}

impl MemberSurface {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        fn clone_map<K, V>(
            map: &SurfaceMap<K, V>,
            mut clone_key: impl FnMut(&K, &mut AllocationBudget) -> Result<K, AllocationError>,
            mut clone_value: impl FnMut(&V, &mut AllocationBudget) -> Result<V, AllocationError>,
            allocations: &mut AllocationBudget,
        ) -> Result<SurfaceMap<K, V>, AllocationError>
        where
            K: Eq + Hash + Ord,
        {
            let mut entries = allocations.vector(map.len())?;
            for (key, value) in map.iter() {
                entries.push((clone_key(key, allocations)?, clone_value(value, allocations)?));
            }
            SurfaceMap::from_frozen_entries_with_allocation_budget(entries, allocations).map_err(|error| match error {
                FrozenSurfaceMapError::Allocation(error) => error,
                FrozenSurfaceMapError::DuplicateKey => AllocationError::Failed,
            })
        }

        fn clone_string(value: &String, allocations: &mut AllocationBudget) -> Result<String, AllocationError> {
            allocations.string(value)
        }

        Ok(Self {
            fields: clone_map(
                &self.fields,
                clone_string,
                |knowledge, allocations| knowledge.clone_with_allocation_budget(allocations),
                allocations,
            )?,
            field_ids: clone_map(
                &self.field_ids,
                |field, allocations| field.clone_with_allocation_budget(allocations),
                |knowledge, allocations| knowledge.clone_with_allocation_budget(allocations),
                allocations,
            )?,
            fields_by_name: clone_map(
                &self.fields_by_name,
                clone_string,
                |field, allocations| field.clone_with_allocation_budget(allocations),
                allocations,
            )?,
            field_visibility: clone_map(
                &self.field_visibility,
                clone_string,
                |visibility, _| Ok(*visibility),
                allocations,
            )?,
            callables: clone_map(
                &self.callables,
                |callable, allocations| callable.clone_with_allocation_budget(allocations),
                |knowledge, allocations| knowledge.clone_with_allocation_budget(allocations),
                allocations,
            )?,
            callable_signatures: clone_map(
                &self.callable_signatures,
                |selector, allocations| clone_selector_with_allocation_budget(selector, allocations),
                |signature, allocations| signature.clone_with_allocation_budget(allocations),
                allocations,
            )?,
            callables_by_selector: clone_map(
                &self.callables_by_selector,
                |selector, allocations| clone_selector_with_allocation_budget(selector, allocations),
                |callable, allocations| callable.clone_with_allocation_budget(allocations),
                allocations,
            )?,
            callable_visibility: clone_map(
                &self.callable_visibility,
                |selector, allocations| clone_selector_with_allocation_budget(selector, allocations),
                |visibility, _| Ok(*visibility),
                allocations,
            )?,
        })
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_field(&mut self, decl_id: Option<&DeclarationId>, side: DispatchSide, name: impl Into<String>, ty: TypeKnowledge) {
        self.add_field_with_visibility(decl_id, side, name, ty, MemberVisibility::Public);
    }

    pub fn add_field_with_visibility(
        &mut self,
        decl_id: Option<&DeclarationId>,
        side: DispatchSide,
        name: impl Into<String>,
        ty: TypeKnowledge,
        visibility: MemberVisibility,
    ) {
        let name_str = name.into();
        if let Some(id) = decl_id {
            let field_id = FieldId::new(id.clone(), name_str.clone(), side);
            self.field_ids.insert(field_id.clone(), ty.clone());
            self.fields_by_name.insert(name_str.clone(), field_id);
        }
        self.field_visibility.insert(name_str.clone(), visibility);
        self.fields.insert(name_str, ty);
    }

    pub fn get_field(&self, name: &str) -> Option<&TypeKnowledge> {
        self.fields.get(name)
    }

    pub fn get_field_id(&self, name: &str) -> Option<&FieldId> {
        self.fields_by_name.get(name)
    }

    pub fn add_callable(&mut self, decl_id: Option<&DeclarationId>, side: DispatchSide, signature: CallableSignature) {
        self.add_callable_with_visibility(decl_id, side, signature, MemberVisibility::Public);
    }

    pub fn add_callable_with_visibility(
        &mut self,
        decl_id: Option<&DeclarationId>,
        side: DispatchSide,
        signature: CallableSignature,
        visibility: MemberVisibility,
    ) {
        if let Some(id) = decl_id {
            let callable_id = CallableId::new(id.clone(), signature.selector.clone(), side);
            self.callables.insert(callable_id.clone(), signature.return_type.clone());
            self.callables_by_selector.insert(signature.selector.clone(), callable_id);
        }
        self.callable_visibility.insert(signature.selector.clone(), visibility);
        self.callable_signatures.insert(signature.selector.clone(), signature);
    }

    pub fn get_callable(&self, selector: &Selector) -> Option<&CallableSignature> {
        if let Some(sig) = self.callable_signatures.get(selector) {
            return Some(sig);
        }
        self.callable_signatures.values().find(|sig| sig.accepts_selector(selector))
    }

    pub fn get_callable_id(&self, selector: &Selector) -> Option<&CallableId> {
        if let Some(id) = self.callables_by_selector.get(selector) {
            return Some(id);
        }
        for (sig_sel, sig) in &self.callable_signatures {
            if sig.accepts_selector(selector) {
                return self.callables_by_selector.get(sig_sel);
            }
        }
        None
    }

    pub fn get_callable_for_selector(&self, selector: &Selector) -> Option<(&CallableId, &CallableSignature)> {
        if let Some(sig) = self.callable_signatures.get(selector) {
            let id = self.callables_by_selector.get(selector)?;
            return Some((id, sig));
        }
        for (sig_sel, sig) in &self.callable_signatures {
            if sig.accepts_selector(selector) {
                let id = self.callables_by_selector.get(sig_sel)?;
                return Some((id, sig));
            }
        }
        None
    }
}

/// Builder for a loaded immutable member surface. Every entry buffer and
/// nested identity/provenance clone is charged before it is requested.
pub(crate) struct FrozenMemberSurfaceBuilder {
    fields: Vec<(String, TypeKnowledge)>,
    field_ids: Vec<(FieldId, TypeKnowledge)>,
    fields_by_name: Vec<(String, FieldId)>,
    field_visibility: Vec<(String, MemberVisibility)>,
    callables: Vec<(CallableId, TypeKnowledge)>,
    callable_signatures: Vec<(Selector, CallableSignature)>,
    callables_by_selector: Vec<(Selector, CallableId)>,
    callable_visibility: Vec<(Selector, MemberVisibility)>,
}

impl FrozenMemberSurfaceBuilder {
    pub(crate) fn new(field_count: usize, callable_count: usize, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        Ok(Self {
            fields: allocations.vector(field_count)?,
            field_ids: allocations.vector(field_count)?,
            fields_by_name: allocations.vector(field_count)?,
            field_visibility: allocations.vector(field_count)?,
            callables: allocations.vector(callable_count)?,
            callable_signatures: allocations.vector(callable_count)?,
            callables_by_selector: allocations.vector(callable_count)?,
            callable_visibility: allocations.vector(callable_count)?,
        })
    }

    pub(crate) fn add_field(
        &mut self,
        field_id: FieldId,
        knowledge: TypeKnowledge,
        visibility: MemberVisibility,
        allocations: &mut AllocationBudget,
    ) -> Result<(), AllocationError> {
        let name = allocations.string(&field_id.name)?;
        let field_value = knowledge.clone_with_allocation_budget(allocations)?;
        self.fields.push((name, knowledge));

        let field_key = field_id.clone_with_allocation_budget(allocations)?;
        let field_value = field_value;
        self.field_ids.push((field_key, field_value));

        let name_key = allocations.string(&field_id.name)?;
        let field_value = field_id.clone_with_allocation_budget(allocations)?;
        self.fields_by_name.push((name_key, field_value));

        let visibility_key = allocations.string(&field_id.name)?;
        self.field_visibility.push((visibility_key, visibility));
        Ok(())
    }

    pub(crate) fn add_callable(
        &mut self,
        callable: CallableId,
        signature: CallableSignature,
        visibility: MemberVisibility,
        allocations: &mut AllocationBudget,
    ) -> Result<(), AllocationError> {
        let callable_key = callable.clone_with_allocation_budget(allocations)?;
        let callable_result = signature.return_type.clone_with_allocation_budget(allocations)?;
        let signature_key = clone_selector_with_allocation_budget(&signature.selector, allocations)?;
        let selector_map_key = clone_selector_with_allocation_budget(&signature.selector, allocations)?;
        let selector_visibility_key = clone_selector_with_allocation_budget(&signature.selector, allocations)?;
        let callable_value = callable.clone_with_allocation_budget(allocations)?;
        self.callables.push((callable_key, callable_result));
        self.callable_signatures.push((signature_key, signature));
        self.callables_by_selector.push((selector_map_key, callable_value));
        self.callable_visibility.push((selector_visibility_key, visibility));
        Ok(())
    }

    pub(crate) fn finish(self, allocations: &mut AllocationBudget) -> Result<MemberSurface, FrozenSurfaceMapError> {
        Ok(MemberSurface {
            fields: SurfaceMap::from_frozen_entries_with_allocation_budget(self.fields, allocations)?,
            field_ids: SurfaceMap::from_frozen_entries_with_allocation_budget(self.field_ids, allocations)?,
            fields_by_name: SurfaceMap::from_frozen_entries_with_allocation_budget(self.fields_by_name, allocations)?,
            field_visibility: SurfaceMap::from_frozen_entries_with_allocation_budget(self.field_visibility, allocations)?,
            callables: SurfaceMap::from_frozen_entries_with_allocation_budget(self.callables, allocations)?,
            callable_signatures: SurfaceMap::from_frozen_entries_with_allocation_budget(self.callable_signatures, allocations)?,
            callables_by_selector: SurfaceMap::from_frozen_entries_with_allocation_budget(self.callables_by_selector, allocations)?,
            callable_visibility: SurfaceMap::from_frozen_entries_with_allocation_budget(self.callable_visibility, allocations)?,
        })
    }
}

fn clone_selector_with_allocation_budget(selector: &Selector, allocations: &mut AllocationBudget) -> Result<Selector, AllocationError> {
    use phalcom_common::selector::{SelectorBase, SelectorSlot};

    let base = match &selector.base {
        SelectorBase::Named(name) => SelectorBase::Named(allocations.string(name)?),
        SelectorBase::Subscript => SelectorBase::Subscript,
    };
    let mut slots = allocations.vector(selector.slots.len())?;
    for slot in selector.slots.iter() {
        slots.push(match slot {
            SelectorSlot::Positional => SelectorSlot::Positional,
            SelectorSlot::Label(label) => SelectorSlot::Label(allocations.string(label)?),
        });
    }
    Ok(Selector { base, kind: selector.kind, slots: allocations.boxed_slice(slots)? })
}

/// Source visibility published with compiler-owned member surfaces.
pub use phalcom_common::modifier::Visibility as MemberVisibility;
pub use phalcom_common::modifier::Visibility;

/// Published semantic surface for a module or class declaration containing both instance and class surfaces.
#[derive(Clone, Debug, Default)]
pub struct DeclarationSurface {
    pub id: Option<DeclarationId>,
    pub instance: MemberSurface,
    pub class: MemberSurface,
}

impl DeclarationSurface {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        Ok(Self {
            id: self.id.as_ref().map(|id| id.clone_with_allocation_budget(allocations)).transpose()?,
            instance: self.instance.clone_with_allocation_budget(allocations)?,
            class: self.class.clone_with_allocation_budget(allocations)?,
        })
    }

    pub fn new(id: Option<DeclarationId>) -> Self {
        Self {
            id,
            instance: MemberSurface::new(),
            class: MemberSurface::new(),
        }
    }

    pub fn surface(&self, side: DispatchSide) -> &MemberSurface {
        match side {
            DispatchSide::Instance => &self.instance,
            DispatchSide::Class => &self.class,
        }
    }

    pub fn surface_mut(&mut self, side: DispatchSide) -> &mut MemberSurface {
        match side {
            DispatchSide::Instance => &mut self.instance,
            DispatchSide::Class => &mut self.class,
        }
    }

    pub fn add_field(&mut self, side: DispatchSide, name: impl Into<String>, ty: TypeKnowledge) {
        let id = self.id.clone();
        self.surface_mut(side).add_field(id.as_ref(), side, name, ty);
    }

    pub fn add_field_with_visibility(&mut self, side: DispatchSide, name: impl Into<String>, ty: TypeKnowledge, visibility: MemberVisibility) {
        let id = self.id.clone();
        self.surface_mut(side).add_field_with_visibility(id.as_ref(), side, name, ty, visibility);
    }

    pub fn get_field(&self, side: DispatchSide, name: &str) -> Option<&TypeKnowledge> {
        self.surface(side).get_field(name)
    }

    pub fn get_field_id(&self, side: DispatchSide, name: &str) -> Option<&FieldId> {
        self.surface(side).get_field_id(name)
    }

    pub fn add_callable(&mut self, side: DispatchSide, signature: CallableSignature) {
        let id = self.id.clone();
        self.surface_mut(side).add_callable(id.as_ref(), side, signature);
    }

    pub fn add_callable_with_visibility(&mut self, side: DispatchSide, signature: CallableSignature, visibility: MemberVisibility) {
        let id = self.id.clone();
        self.surface_mut(side).add_callable_with_visibility(id.as_ref(), side, signature, visibility);
    }

    pub fn get_callable(&self, side: DispatchSide, selector: &Selector) -> Option<&CallableSignature> {
        self.surface(side).get_callable(selector)
    }

    pub fn get_callable_id(&self, side: DispatchSide, selector: &Selector) -> Option<&CallableId> {
        self.surface(side).get_callable_id(selector)
    }
}
