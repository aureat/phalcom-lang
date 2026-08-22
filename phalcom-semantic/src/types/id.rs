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

/// Store-relative unique identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeStoreId(pub u64);

impl TypeStoreId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A `TypeId` guaranteed to have kind `KindId::TYPE` in the store that validated it.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProperTypeId(pub TypeId);

impl ProperTypeId {
    #[inline]
    pub const fn raw(self) -> TypeId {
        self.0
    }

    #[inline]
    pub const fn type_id(self) -> TypeId {
        self.0
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0.index()
    }
}

impl From<ProperTypeId> for TypeId {
    #[inline]
    fn from(proper: ProperTypeId) -> Self {
        proper.0
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
    pub const RECORD_ROW: Self = Self(1);

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

    #[inline]
    pub fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

/// Store/snapshot-local identifier for an alpha-normalized type lambda.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeLambdaId(pub u32);

impl TypeLambdaId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

/// Scoped type identifier within a type lambda body arena.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedTypeId(pub u32);

impl ScopedTypeId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn from_index(index: usize) -> Self {
        Self(index as u32)
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
