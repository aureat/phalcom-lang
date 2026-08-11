//! Bounded immediate representation for `Option` values.
//!
//! The physical `Value` layout is intentionally still an ordinary Rust enum.
//! These arms are a correctness-first landing substrate: nested `Some` values
//! are flattened into a depth plus one non-recursive payload, so wrapping never
//! needs `Box<Value>` or a heap allocation. A later layout pass can replace this
//! module behind the same helpers.

use crate::error::RuntimeError;
use crate::heap::ObjRef;
use crate::interner::Symbol;

use super::Value;
use std::hash::{Hash, Hasher};

/// Maximum number of generic `Some` wrappers representable by the VM.
pub const MAX_OPTION_NESTING: u8 = 7;

/// The non-recursive payload carried by an immediate `Some` value.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionPayload {
    None,
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}

impl OptionPayload {
    fn from_base(value: Value) -> Result<Self, RuntimeError> {
        match value {
            Value::None => Ok(Self::None),
            Value::Unit => Ok(Self::Unit),
            Value::Bool(value) => Ok(Self::Bool(value)),
            Value::Int(value) => Ok(Self::Int(value)),
            Value::Float(value) => Ok(Self::Float(value)),
            Value::Symbol(value) => Ok(Self::Symbol(value)),
            Value::Obj(value) => Ok(Self::Obj(value)),
            Value::Nil => Err(RuntimeError::Internal("private Nil cannot be wrapped in Some".into())),
            Value::Some1(..) | Value::Some2(..) | Value::Some3(..) | Value::Some4(..) | Value::Some5(..) | Value::Some6(..) | Value::Some7(..) => {
                Err(RuntimeError::Internal("nested Option payload was not normalized".into()))
            }
        }
    }

    pub(crate) fn into_value(self) -> Value {
        match self {
            Self::None => Value::None,
            Self::Unit => Value::Unit,
            Self::Bool(value) => Value::Bool(value),
            Self::Int(value) => Value::Int(value),
            Self::Float(value) => Value::Float(value),
            Self::Symbol(value) => Value::Symbol(value),
            Self::Obj(value) => Value::Obj(value),
        }
    }

    pub(crate) fn gc_obj_ref(self) -> Option<ObjRef> {
        match self {
            Self::Obj(value) => Some(value),
            Self::None | Self::Unit | Self::Bool(_) | Self::Int(_) | Self::Float(_) | Self::Symbol(_) => None,
        }
    }

    pub(crate) fn hash<H: Hasher>(self, state: &mut H) {
        match self {
            Self::None => 0u8.hash(state),
            Self::Unit => 1u8.hash(state),
            Self::Bool(value) => value.hash(state),
            Self::Int(value) => value.hash(state),
            Self::Float(value) => value.to_bits().hash(state),
            Self::Symbol(value) => value.0.hash(state),
            Self::Obj(value) => value.hash(state),
        }
    }
}

/// The result of peeling exactly one `Option` layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum OptionCase {
    None,
    Some(Value),
    NotOption,
}

impl Value {
    /// Returns whether this value is a surface `Option` value.
    #[inline]
    pub fn is_option(self) -> bool {
        matches!(
            self,
            Self::None | Self::Some1(..) | Self::Some2(..) | Self::Some3(..) | Self::Some4(..) | Self::Some5(..) | Self::Some6(..) | Self::Some7(..)
        )
    }

    /// Returns whether this value is exactly the immediate `None` variant.
    #[inline]
    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns the number of `Some` wrappers, or zero for `None` and non-Option
    /// values.
    #[inline]
    pub fn option_depth(self) -> u8 {
        match self {
            Self::Some1(..) => 1,
            Self::Some2(..) => 2,
            Self::Some3(..) => 3,
            Self::Some4(..) => 4,
            Self::Some5(..) => 5,
            Self::Some6(..) => 6,
            Self::Some7(..) => 7,
            Self::Nil | Self::Unit | Self::Bool(_) | Self::Int(_) | Self::Float(_) | Self::Symbol(_) | Self::Obj(_) | Self::None => 0,
        }
    }

    /// Adds exactly one `Some` layer without allocating an Option wrapper.
    pub(crate) fn wrap_some(self) -> Result<Self, RuntimeError> {
        match self {
            Self::Some1(payload) => Ok(Self::Some2(payload)),
            Self::Some2(payload) => Ok(Self::Some3(payload)),
            Self::Some3(payload) => Ok(Self::Some4(payload)),
            Self::Some4(payload) => Ok(Self::Some5(payload)),
            Self::Some5(payload) => Ok(Self::Some6(payload)),
            Self::Some6(payload) => Ok(Self::Some7(payload)),
            Self::Some7(..) => Err(RuntimeError::OptionNestingLimit { limit: MAX_OPTION_NESTING }),
            Self::Nil => Err(RuntimeError::Internal("private Nil cannot be wrapped in Some".into())),
            base => Ok(Self::Some1(OptionPayload::from_base(base)?)),
        }
    }

    /// Peels exactly one `Some` layer for the native `Option.match` primitive.
    pub(crate) fn option_case(self) -> OptionCase {
        match self {
            Self::None => OptionCase::None,
            Self::Some1(payload) => OptionCase::Some(payload.into_value()),
            Self::Some2(payload) => OptionCase::Some(Self::Some1(payload)),
            Self::Some3(payload) => OptionCase::Some(Self::Some2(payload)),
            Self::Some4(payload) => OptionCase::Some(Self::Some3(payload)),
            Self::Some5(payload) => OptionCase::Some(Self::Some4(payload)),
            Self::Some6(payload) => OptionCase::Some(Self::Some5(payload)),
            Self::Some7(payload) => OptionCase::Some(Self::Some6(payload)),
            Self::Nil | Self::Unit | Self::Bool(_) | Self::Int(_) | Self::Float(_) | Self::Symbol(_) | Self::Obj(_) => OptionCase::NotOption,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_OPTION_NESTING, OptionCase};
    use crate::error::RuntimeError;
    use crate::value::Value;

    #[test]
    fn wraps_immediate_without_object_handle() {
        let value = Value::Int(42).wrap_some().unwrap();
        assert_eq!(value.option_depth(), 1);
        assert!(value.is_option());
        assert!(!value.is_none());
        assert_eq!(value.option_case(), OptionCase::Some(Value::Int(42)));
        assert!(value.as_obj().is_none());
    }

    #[test]
    fn some_none_is_distinct_from_none() {
        let none = Value::None;
        let some_none = none.wrap_some().unwrap();

        assert_ne!(none, some_none);
        assert!(none.is_none());
        assert_eq!(some_none.option_depth(), 1);
        assert_eq!(some_none.option_case(), OptionCase::Some(Value::None));
    }

    #[test]
    fn peeling_preserves_exact_nested_depth() {
        let value = Value::Int(7).wrap_some().unwrap().wrap_some().unwrap().wrap_some().unwrap();
        assert_eq!(value.option_depth(), 3);
        let OptionCase::Some(level_two) = value.option_case() else {
            panic!("expected Some")
        };
        assert_eq!(level_two.option_depth(), 2);
        let OptionCase::Some(level_one) = level_two.option_case() else {
            panic!("expected Some")
        };
        assert_eq!(level_one.option_depth(), 1);
        assert_eq!(level_one.option_case(), OptionCase::Some(Value::Int(7)));
    }

    #[test]
    fn wraps_seven_layers_and_rejects_eighth() {
        let mut value = Value::None;
        for depth in 1..=MAX_OPTION_NESTING {
            value = value.wrap_some().unwrap();
            assert_eq!(value.option_depth(), depth);
        }

        assert!(matches!(value.wrap_some(), Err(RuntimeError::OptionNestingLimit { limit: 7 })));
    }

    #[test]
    fn private_nil_cannot_be_wrapped() {
        assert!(matches!(Value::Nil.wrap_some(), Err(RuntimeError::Internal(message)) if message.contains("private Nil")));
    }

    #[test]
    fn wrapped_object_is_visible_only_to_gc_seam() {
        let object = crate::heap::ObjRef::default();
        let value = Value::Obj(object).wrap_some().unwrap();
        assert!(value.as_obj().is_none());
        assert_eq!(value.gc_obj_ref(), Some(object));
    }
}
