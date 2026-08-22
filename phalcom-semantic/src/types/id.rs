//! Type, Kind, and Variable IDs.

/// Interned canonical type identifier within a [`TypeStore`].
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

/// Interned kind identifier.
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

/// Identifier for generic type parameters.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeParameterId(pub u32);

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
