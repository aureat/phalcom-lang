//! Immediate representation for `Option` values.
//!
//! Nested `Some` values are encoded via the 32-bit depth metadata field in [`Value`].

use crate::error::RuntimeError;
use crate::value::repr::ValueTag;

use super::Value;

/// Maximum number of generic `Some` wrappers representable by the VM.
pub const MAX_OPTION_NESTING: u32 = u32::MAX;

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
        self.tag() == ValueTag::None || self.some_depth_raw() > 0
    }

    /// Returns whether this value is exactly the immediate `None` variant.
    #[inline]
    pub fn is_none(self) -> bool {
        self.tag() == ValueTag::None && self.some_depth_raw() == 0
    }

    /// Returns whether this value is an immediate `Some` wrapper.
    #[inline]
    pub fn is_some(self) -> bool {
        self.some_depth_raw() > 0
    }

    /// Returns the number of `Some` wrappers, or zero for `None` and non-Option
    /// values.
    #[inline]
    pub fn option_depth(self) -> u32 {
        self.some_depth_raw()
    }

    /// Adds exactly one `Some` layer without allocating an Option wrapper.
    pub fn wrap_some(self) -> Result<Self, RuntimeError> {
        if self.is_nil() {
            return Err(RuntimeError::Internal("private Nil cannot be wrapped in Some".into()));
        }
        let current = self.some_depth_raw();
        let new_depth = current.checked_add(1).ok_or(RuntimeError::OptionNestingLimit { limit: MAX_OPTION_NESTING })?;
        Ok(self.with_some_depth(new_depth))
    }

    /// Peels exactly one `Some` layer for the native `Option.match` primitive.
    pub(crate) fn option_case(self) -> OptionCase {
        if self.is_none() {
            OptionCase::None
        } else if self.is_some() {
            OptionCase::Some(self.with_some_depth(self.some_depth_raw() - 1))
        } else {
            OptionCase::NotOption
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OptionCase, Value};
    use crate::error::RuntimeError;

    #[test]
    fn wraps_immediate_without_object_handle() {
        let value = Value::int(42).wrap_some().unwrap();
        assert_eq!(value.option_depth(), 1);
        assert!(value.is_option());
        assert!(!value.is_none());
        assert_eq!(value.option_case(), OptionCase::Some(Value::int(42)));
        assert!(value.as_obj().is_none());
    }

    #[test]
    fn some_none_is_distinct_from_none() {
        let none = Value::none();
        let some_none = none.wrap_some().unwrap();

        assert_ne!(none, some_none);
        assert!(none.is_none());
        assert_eq!(some_none.option_depth(), 1);
        assert_eq!(some_none.option_case(), OptionCase::Some(Value::none()));
    }

    #[test]
    fn peeling_preserves_exact_nested_depth() {
        let value = Value::int(7).wrap_some().unwrap().wrap_some().unwrap().wrap_some().unwrap();
        assert_eq!(value.option_depth(), 3);
        let OptionCase::Some(level_two) = value.option_case() else {
            panic!("expected Some")
        };
        assert_eq!(level_two.option_depth(), 2);
        let OptionCase::Some(level_one) = level_two.option_case() else {
            panic!("expected Some")
        };
        assert_eq!(level_one.option_depth(), 1);
        assert_eq!(level_one.option_case(), OptionCase::Some(Value::int(7)));
    }

    #[test]
    fn wraps_beyond_seven_layers() {
        let mut value = Value::none();
        for depth in 1..=8 {
            value = value.wrap_some().unwrap();
            assert_eq!(value.option_depth(), depth);
        }
        assert_eq!(value.option_depth(), 8);
    }

    #[test]
    fn private_nil_cannot_be_wrapped() {
        assert!(matches!(Value::nil().wrap_some(), Err(RuntimeError::Internal(message)) if message.contains("private Nil")));
    }

    #[test]
    fn wrapped_object_is_visible_only_to_gc_seam() {
        let object = crate::heap::ObjRef::default();
        let value = Value::obj(object).wrap_some().unwrap();
        assert!(value.as_obj().is_none());
        assert_eq!(value.gc_obj_ref(), Some(object));
    }
}
