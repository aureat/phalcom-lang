//! 16-byte explicit tagged representation for [`Value`].
//!
//! Realizes the 16-byte Value representation specification.
//! Value is exactly two 64-bit words:
//! - payload: u64
//! - meta: u64 (bits 0..=7: tag, bits 8..=39: Some depth, bits 40..=63: reserved)

use crate::heap::ObjRef;
use crate::interner::Symbol;
use std::hash::{Hash, Hasher};

const TAG_MASK: u64 = 0xff;
const DEPTH_SHIFT: u32 = 8;
const DEPTH_MASK: u64 = 0xffff_ffffu64 << DEPTH_SHIFT;
const RESERVED_MASK: u64 = !(TAG_MASK | DEPTH_MASK);

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ValueTag {
    Nil = 0,
    Unit = 1,
    Bool = 2,
    Int = 3,
    Float = 4,
    Symbol = 5,
    Obj = 6,
    None = 7,
}

impl ValueTag {
    #[inline]
    const fn from_u8(tag: u8) -> Self {
        match tag {
            0 => Self::Nil,
            1 => Self::Unit,
            2 => Self::Bool,
            3 => Self::Int,
            4 => Self::Float,
            5 => Self::Symbol,
            6 => Self::Obj,
            7 => Self::None,
            _ => panic!("invalid ValueTag"),
        }
    }
}

/// A uniform 16-byte Phalcom value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value {
    payload: u64,
    meta: u64,
}

impl Value {
    #[inline]
    pub(crate) const fn nil() -> Self {
        Self {
            payload: 0,
            meta: ValueTag::Nil as u64,
        }
    }

    #[inline]
    pub const fn unit() -> Self {
        Self {
            payload: 0,
            meta: ValueTag::Unit as u64,
        }
    }

    #[inline]
    pub const fn bool(value: bool) -> Self {
        Self {
            payload: if value { 1 } else { 0 },
            meta: ValueTag::Bool as u64,
        }
    }

    #[inline]
    pub const fn int(value: i64) -> Self {
        Self {
            payload: value as u64,
            meta: ValueTag::Int as u64,
        }
    }

    #[inline]
    pub const fn float(value: f64) -> Self {
        Self {
            payload: value.to_bits(),
            meta: ValueTag::Float as u64,
        }
    }

    #[inline]
    pub const fn symbol(value: Symbol) -> Self {
        Self {
            payload: value.0 as u64,
            meta: ValueTag::Symbol as u64,
        }
    }

    #[inline]
    pub fn obj(value: ObjRef) -> Self {
        Self {
            payload: value.to_opaque_u64(),
            meta: ValueTag::Obj as u64,
        }
    }

    #[inline]
    pub const fn none() -> Self {
        Self {
            payload: 0,
            meta: ValueTag::None as u64,
        }
    }

    #[inline]
    pub(crate) fn tag(self) -> ValueTag {
        ValueTag::from_u8((self.meta & TAG_MASK) as u8)
    }

    #[inline]
    pub(crate) fn some_depth_raw(self) -> u32 {
        ((self.meta & DEPTH_MASK) >> DEPTH_SHIFT) as u32
    }

    #[inline]
    pub(crate) fn with_some_depth(self, depth: u32) -> Self {
        let meta = (self.meta & !DEPTH_MASK) | ((depth as u64) << DEPTH_SHIFT);
        debug_assert!(meta & RESERVED_MASK == 0, "with_some_depth: reserved bits must be zero after depth write");
        Self { payload: self.payload, meta }
    }

    #[inline]
    pub(crate) fn without_some_wrappers(self) -> Self {
        let meta = self.meta & !DEPTH_MASK;
        Self { payload: self.payload, meta }
    }

    #[inline]
    pub fn is_nil(self) -> bool {
        self.tag() == ValueTag::Nil && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn is_unit(self) -> bool {
        self.tag() == ValueTag::Unit && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn is_bool(self) -> bool {
        self.tag() == ValueTag::Bool && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn is_int(self) -> bool {
        self.tag() == ValueTag::Int && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn is_float(self) -> bool {
        self.tag() == ValueTag::Float && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn is_symbol(self) -> bool {
        self.tag() == ValueTag::Symbol && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn is_obj(self) -> bool {
        self.tag() == ValueTag::Obj && self.some_depth_raw() == 0
    }

    #[inline]
    pub fn as_bool(self) -> Option<bool> {
        if self.is_bool() { Some(self.payload != 0) } else { None }
    }

    #[inline]
    pub fn as_int(self) -> Option<i64> {
        if self.is_int() { Some(self.payload as i64) } else { None }
    }

    #[inline]
    pub fn as_float(self) -> Option<f64> {
        if self.is_float() { Some(f64::from_bits(self.payload)) } else { None }
    }

    #[inline]
    pub fn as_obj(&self) -> Option<ObjRef> {
        if self.is_obj() { Some(ObjRef::from_opaque_u64(self.payload)) } else { None }
    }

    #[inline]
    pub(crate) fn symbol_value(self) -> Option<Symbol> {
        if self.is_symbol() { Some(Symbol(self.payload as u32)) } else { None }
    }

    #[inline]
    pub fn as_symbol(&self) -> Result<Symbol, String> {
        self.symbol_value().ok_or_else(|| "Type Error: Expected a Symbol.".to_string())
    }

    #[inline]
    pub fn gc_obj_ref(&self) -> Option<ObjRef> {
        if self.tag() == ValueTag::Obj {
            Some(ObjRef::from_opaque_u64(self.payload))
        } else {
            None
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.tag() != other.tag() || self.some_depth_raw() != other.some_depth_raw() {
            return false;
        }

        match self.tag() {
            ValueTag::Nil | ValueTag::Unit | ValueTag::None => true,
            ValueTag::Bool => (self.payload != 0) == (other.payload != 0),
            ValueTag::Int => (self.payload as i64) == (other.payload as i64),
            ValueTag::Float => {
                let f1 = f64::from_bits(self.payload);
                let f2 = f64::from_bits(other.payload);
                f1 == f2
            }
            ValueTag::Symbol => (self.payload as u32) == (other.payload as u32),
            ValueTag::Obj => self.payload == other.payload,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag().hash(state);
        self.some_depth_raw().hash(state);

        match self.tag() {
            ValueTag::Nil | ValueTag::Unit | ValueTag::None => {}
            ValueTag::Bool => (self.payload != 0).hash(state),
            ValueTag::Int => (self.payload as i64).hash(state),
            ValueTag::Float => {
                let f = f64::from_bits(self.payload);
                if f == 0.0 {
                    0.0f64.to_bits().hash(state);
                } else if f.is_nan() {
                    f64::NAN.to_bits().hash(state);
                } else {
                    self.payload.hash(state);
                }
            }
            ValueTag::Symbol => (self.payload as u32).hash(state),
            ValueTag::Obj => self.payload.hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn value_is_exactly_sixteen_bytes() {
        assert_eq!(std::mem::size_of::<Value>(), 16);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn value_has_eight_byte_alignment_on_64_bit_targets() {
        assert_eq!(std::mem::align_of::<Value>(), 8);
    }

    #[test]
    fn scalar_round_trips() {
        // i64
        for n in [i64::MIN, -1, 0, 1, i64::MAX] {
            let v = Value::int(n);
            assert!(v.is_int());
            assert_eq!(v.as_int(), Some(n));
        }

        // f64
        for f in [0.0, -0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, std::f64::consts::PI, -123.456] {
            let v = Value::float(f);
            assert!(v.is_float());
            let decoded = v.as_float().unwrap();
            assert_eq!(decoded.to_bits(), f.to_bits());
        }

        // bool
        let t = Value::bool(true);
        let f = Value::bool(false);
        assert_eq!(t.as_bool(), Some(true));
        assert_eq!(f.as_bool(), Some(false));

        // symbol
        let s1 = Value::symbol(Symbol(0));
        let s2 = Value::symbol(Symbol(1_000_000));
        assert_eq!(s1.symbol_value(), Some(Symbol(0)));
        assert_eq!(s2.symbol_value(), Some(Symbol(1_000_000)));

        // unit
        let u = Value::unit();
        assert!(u.is_unit());

        // none
        let none = Value::none();
        assert!(none.is_none());

        // nil
        let nil = Value::nil();
        assert!(nil.is_nil());
    }

    #[test]
    fn partial_eq_and_hash_consistency() {
        let p_zero = Value::float(0.0);
        let n_zero = Value::float(-0.0);
        assert_eq!(p_zero, n_zero);
        assert_eq!(calculate_hash(&p_zero), calculate_hash(&n_zero));

        let nan1 = Value::float(f64::NAN);
        let nan2 = Value::float(f64::NAN);
        assert_ne!(nan1, nan2);

        let i1 = Value::int(1);
        let f1 = Value::float(1.0);
        assert_ne!(i1, f1);
    }

    #[test]
    fn arbitrary_nan_bit_patterns_round_trip() {
        // IEEE 754 allows many distinct NaN encodings; all must survive the
        // payload round-trip. Two distinct NaN payloads are unequal by
        // IEEE semantics and thus by Value::PartialEq.
        let nan_a = f64::from_bits(0x7ff8_0000_0000_0001u64); // quiet NaN, payload 1
        let nan_b = f64::from_bits(0x7ff4_0000_0000_0001u64); // signaling NaN-ish
        let va = Value::float(nan_a);
        let vb = Value::float(nan_b);
        assert!(va.is_float());
        assert!(vb.is_float());
        assert_eq!(va.as_float().unwrap().to_bits(), nan_a.to_bits());
        assert_eq!(vb.as_float().unwrap().to_bits(), nan_b.to_bits());
        // Two distinct NaN payloads are not PartialEq-equal
        assert_ne!(va, vb);
    }

    #[test]
    fn option_depth_boundary_255() {
        let base = Value::int(42);
        let wrapped = base.with_some_depth(255);
        assert_eq!(wrapped.some_depth_raw(), 255);
        assert!(wrapped.is_some());
        assert!(!wrapped.is_int());
    }

    #[test]
    fn option_depth_boundary_65535() {
        let base = Value::bool(true);
        let wrapped = base.with_some_depth(65_535);
        assert_eq!(wrapped.some_depth_raw(), 65_535);
        assert!(wrapped.is_some());
        assert!(!wrapped.is_bool());
    }

    #[test]
    fn option_depth_u32_max_saturates() {
        // with_some_depth at u32::MAX: only bits 8..=39 are stored (32 bits).
        // Verify that the stored value equals u32::MAX and no reserved bits leak.
        let base = Value::unit();
        let wrapped = base.with_some_depth(u32::MAX);
        assert_eq!(wrapped.some_depth_raw(), u32::MAX);
        assert_eq!(wrapped.meta & RESERVED_MASK, 0, "reserved bits must be zero");
    }

    #[test]
    fn option_equality_same_depth() {
        let a = Value::int(7).with_some_depth(3);
        let b = Value::int(7).with_some_depth(3);
        assert_eq!(a, b);
        assert_eq!(calculate_hash(&a), calculate_hash(&b));
    }

    #[test]
    fn option_equality_different_depth() {
        let a = Value::int(7).with_some_depth(2);
        let b = Value::int(7).with_some_depth(3);
        assert_ne!(a, b);
    }

    #[test]
    fn wrapped_scalar_rejects_plain_accessors() {
        let wrapped_int = Value::int(1).with_some_depth(1);
        assert!(!wrapped_int.is_int(), "Some(Int) must not report is_int()");
        assert!(wrapped_int.as_int().is_none(), "Some(Int) as_int must return None");
        assert!(wrapped_int.is_some());

        let wrapped_bool = Value::bool(true).with_some_depth(1);
        assert!(!wrapped_bool.is_bool());
        assert!(wrapped_bool.as_bool().is_none());
    }

    #[test]
    fn canonical_constructors_have_no_reserved_bits() {
        let values = [
            Value::nil(),
            Value::unit(),
            Value::bool(true),
            Value::bool(false),
            Value::int(0),
            Value::float(0.0),
            Value::none(),
            Value::symbol(Symbol(12345)),
        ];
        for v in &values {
            assert_eq!(v.meta & RESERVED_MASK, 0, "canonical constructor produced non-zero reserved bits: {v:?}");
        }
    }

    #[test]
    fn nil_unit_none_have_zero_payload() {
        assert_eq!(Value::nil().payload, 0);
        assert_eq!(Value::unit().payload, 0);
        assert_eq!(Value::none().payload, 0);
    }
}
