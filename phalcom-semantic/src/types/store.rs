//! Canonical Type Store with interning and normalization.

use super::id::{KindId, TypeId, TypeParameterId};
use super::kind::KindData;
use crate::identity::DeclarationId;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TupleTypeElement {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordTypeField {
    pub name: Box<str>,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableParameterType {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableType {
    pub parameters: Box<[CallableParameterType]>,
    pub return_type: TypeId,
}

/// Structural canonical type definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeData {
    /// Canonical bottom type (inhabited by no values).
    Never,
    /// Canonical unit type (inhabited by single unit value).
    Unit,
    /// Canonical nominal class declaration type.
    Nominal {
        declaration: DeclarationId,
    },
    /// Generic type application (e.g. `List<Int>`).
    Applied {
        origin: TypeId,
        arguments: Box<[TypeId]>,
    },
    /// Flat, deduplicated, sorted union of two or more distinct types.
    Union(Box<[TypeId]>),
    /// Tuple type.
    Tuple(Box<[TupleTypeElement]>),
    /// Record / structural property map.
    Record(Box<[RecordTypeField]>),
    /// Callable / block signature.
    Callable(CallableType),
    /// Type variable parameter in generic declaration.
    Parameter(TypeParameterId),
}

/// Central store for canonical type interning, hash-consing, and kind assignments.
#[derive(Clone, Debug)]
pub struct TypeStore {
    types: Vec<TypeData>,
    type_to_id: HashMap<TypeData, TypeId>,
    kinds: Vec<KindData>,
    kind_to_id: HashMap<KindData, KindId>,
    type_kinds: HashMap<TypeId, KindId>,

    never_id: TypeId,
    unit_id: TypeId,
}

impl Default for TypeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeStore {
    pub fn new() -> Self {
        let mut store = Self {
            types: Vec::new(),
            type_to_id: HashMap::new(),
            kinds: Vec::new(),
            kind_to_id: HashMap::new(),
            type_kinds: HashMap::new(),
            never_id: TypeId::DUMMY,
            unit_id: TypeId::DUMMY,
        };

        // Kind::Type is KindId(0)
        let type_kind = store.intern_kind(KindData::Type);
        assert_eq!(type_kind, KindId::TYPE);

        store.never_id = store.intern(TypeData::Never);
        store.unit_id = store.intern(TypeData::Unit);

        store.set_kind(store.never_id, KindId::TYPE);
        store.set_kind(store.unit_id, KindId::TYPE);

        store
    }

    #[inline]
    pub fn never(&self) -> TypeId {
        self.never_id
    }

    #[inline]
    pub fn unit(&self) -> TypeId {
        self.unit_id
    }

    pub fn intern_kind(&mut self, data: KindData) -> KindId {
        if let Some(&id) = self.kind_to_id.get(&data) {
            return id;
        }
        let id = KindId(self.kinds.len() as u32);
        self.kinds.push(data.clone());
        self.kind_to_id.insert(data, id);
        id
    }

    pub fn get_kind(&self, id: KindId) -> &KindData {
        &self.kinds[id.index()]
    }

    pub fn set_kind(&mut self, ty: TypeId, kind: KindId) {
        self.type_kinds.insert(ty, kind);
    }

    pub fn kind_of(&self, ty: TypeId) -> KindId {
        self.type_kinds.get(&ty).copied().unwrap_or(KindId::TYPE)
    }

    pub fn intern(&mut self, data: TypeData) -> TypeId {
        if let Some(&id) = self.type_to_id.get(&data) {
            return id;
        }
        let id = TypeId(self.types.len() as u32);
        self.types.push(data.clone());
        self.type_to_id.insert(data, id);
        id
    }

    #[inline]
    pub fn get(&self, id: TypeId) -> &TypeData {
        &self.types[id.index()]
    }

    /// Interns a nominal class declaration reference.
    pub fn nominal(&mut self, declaration: DeclarationId) -> TypeId {
        let ty = self.intern(TypeData::Nominal { declaration });
        self.set_kind(ty, KindId::TYPE);
        ty
    }

    /// Interns a generic applied type.
    pub fn applied(&mut self, origin: TypeId, arguments: Box<[TypeId]>) -> TypeId {
        let ty = self.intern(TypeData::Applied { origin, arguments });
        self.set_kind(ty, KindId::TYPE);
        ty
    }

    /// Normalizes and interns a union type.
    ///
    /// Rules:
    /// 1. Flatten nested unions.
    /// 2. Remove duplicates.
    /// 3. Remove `Never` when other members exist.
    /// 4. Sort canonically by TypeId.
    /// 5. Zero members -> `Never`.
    /// 6. One member -> member TypeId.
    pub fn union(&mut self, members: &[TypeId]) -> TypeId {
        let mut flattened = Vec::new();
        self.collect_union_members(members, &mut flattened);

        // Deduplicate and sort
        flattened.sort_unstable_by_key(|t| t.0);
        flattened.dedup();

        // Remove Never if other types present
        if flattened.len() > 1 {
            flattened.retain(|&t| t != self.never_id);
        }

        match flattened.len() {
            0 => self.never_id,
            1 => flattened[0],
            _ => {
                let ty = self.intern(TypeData::Union(flattened.into_boxed_slice()));
                self.set_kind(ty, KindId::TYPE);
                ty
            }
        }
    }

    fn collect_union_members(&self, members: &[TypeId], out: &mut Vec<TypeId>) {
        for &m in members {
            match self.get(m) {
                TypeData::Union(nested) => self.collect_union_members(nested, out),
                _ => out.push(m),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::core();
        DeclarationId::new(module, name.into())
    }

    #[test]
    fn nominal_interning_is_canonical() {
        let mut store = TypeStore::new();
        let decl = test_decl("User");
        let t1 = store.nominal(decl.clone());
        let t2 = store.nominal(decl);
        assert_eq!(t1, t2);
    }

    #[test]
    fn union_normalization_flattens_deduplicates_and_sorts() {
        let mut store = TypeStore::new();
        let d_a = test_decl("A");
        let d_b = test_decl("B");
        let d_c = test_decl("C");
        let t_a = store.nominal(d_a);
        let t_b = store.nominal(d_b);
        let t_c = store.nominal(d_c);

        let u1 = store.union(&[t_a, t_b]);
        let u2 = store.union(&[t_b, t_a]);
        assert_eq!(u1, u2, "union order invariant");

        let u3 = store.union(&[u1, t_c]);
        let u4 = store.union(&[t_c, t_a, t_b, t_a]);
        assert_eq!(u3, u4, "nested and duplicated union invariant");

        let with_never = store.union(&[t_a, store.never()]);
        assert_eq!(with_never, t_a, "never removed from nonempty union");

        let empty_union = store.union(&[]);
        assert_eq!(empty_union, store.never(), "empty union is never");
    }
}
