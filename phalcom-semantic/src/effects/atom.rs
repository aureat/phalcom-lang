//! Compact bitset representation for effect atoms.

use phalcom_native_meta::primitive::NativeEffect;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EffectAtom {
    Mutation = 0,
    Io = 1,
    Scheduling = 2,
    Reflection = 3,
    Nondeterminism = 4,
    Blocking = 5,
}

impl EffectAtom {
    pub const ALL: [Self; 6] = [
        Self::Mutation,
        Self::Io,
        Self::Scheduling,
        Self::Reflection,
        Self::Nondeterminism,
        Self::Blocking,
    ];

    pub fn from_native(native: NativeEffect) -> Self {
        match native {
            NativeEffect::Mutation => Self::Mutation,
            NativeEffect::Io => Self::Io,
            NativeEffect::Scheduling => Self::Scheduling,
            NativeEffect::Reflection => Self::Reflection,
            NativeEffect::Nondeterminism => Self::Nondeterminism,
            NativeEffect::Blocking => Self::Blocking,
        }
    }
}

/// Compact bitset for 6 effect atoms — no heap allocation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EffectSet(pub u8); // 6 bits used

impl EffectSet {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub fn contains(self, a: EffectAtom) -> bool {
        (self.0 & (1 << (a as u8))) != 0
    }

    #[inline]
    pub fn insert(self, a: EffectAtom) -> Self {
        Self(self.0 | (1 << (a as u8)))
    }

    #[inline]
    pub fn remove(self, a: EffectAtom) -> Self {
        Self(self.0 & !(1 << (a as u8)))
    }

    #[inline]
    pub fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn is_subset_of(self, other: Self) -> bool {
        (self.0 & !other.0) == 0
    }

    pub fn atoms(self) -> impl Iterator<Item = EffectAtom> {
        EffectAtom::ALL.into_iter().filter(move |&a| self.contains(a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effects_join_and_subset() {
        let empty = EffectSet::EMPTY;
        assert!(empty.is_empty());

        let io = empty.insert(EffectAtom::Io);
        let mut_ = empty.insert(EffectAtom::Mutation);

        assert!(io.contains(EffectAtom::Io));
        assert!(!io.contains(EffectAtom::Mutation));

        let joined = io.join(mut_);
        assert!(joined.contains(EffectAtom::Io));
        assert!(joined.contains(EffectAtom::Mutation));
        assert!(io.is_subset_of(joined));
        assert!(!joined.is_subset_of(io));

        // commutativity and idempotency
        assert_eq!(io.join(mut_), mut_.join(io));
        assert_eq!(joined.join(joined), joined);
        assert_eq!(empty.join(joined), joined);
    }
}
