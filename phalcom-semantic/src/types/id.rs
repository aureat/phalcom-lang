//! Type, Kind, and Variable IDs.

/// Store/snapshot-local canonical identifier for a type-level form.
///
/// A `TypeId` may identify a proper type (`kind == Type`) or an unsaturated
/// type constructor/higher-kinded form. The associated `KindId` determines
/// which. The integer is meaningful only with the `TypeStore` that allocated it.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const DUMMY: Self = Self(u32::MAX);

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Store/snapshot-local interned kind identifier.
///
/// The integer is meaningful only with the `TypeStore` that allocated it.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KindId(pub u32);

impl KindId {
    pub const TYPE: Self = Self(0);

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Identifier for generic type parameters within a `TypeStore`.
///
/// The integer is meaningful only with the `TypeStore` that allocated it.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeParameterId(pub u32);

impl TypeParameterId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Inference variable identifier (distinct from canonical TypeId).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InferVarId(pub u32);

impl InferVarId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}
