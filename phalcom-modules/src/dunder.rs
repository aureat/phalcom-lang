//! Compiler-facing policy for Phalcom's language-reserved `__name__` lane.
//!
//! The runtime symbol/selector machinery intentionally remains able to represent
//! dunder spellings. This module controls *source declaration roles*, not strings.

use std::collections::BTreeMap;

/// Source role in which a dunder spelling is being introduced by user syntax.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DunderRole {
    Binding,
    ImportAlias,
    Export,
    Field,
    Method,
    Parameter,
}

/// Stability/override category for a standardized dunder name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DunderCategory {
    /// Compiler/runtime intrinsic; user declarations are never legal.
    Intrinsic,
    /// Guaranteed reflection protocol; user replacement is never legal.
    GuaranteedReflection,
    /// Standardized policy hook legal only in the listed declaration roles.
    Hook { roles: &'static [DunderRole] },
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DunderPolicyError {
    #[error("dunder name '{name}' is language-reserved and cannot be declared as {role:?}")]
    Reserved { name: String, role: DunderRole },
    #[error("unknown dunder name '{name}' is language-reserved")]
    Unknown { name: String },
}

#[derive(Clone, Debug)]
pub struct DunderPolicy {
    standardized: BTreeMap<&'static str, DunderCategory>,
}

impl Default for DunderPolicy {
    fn default() -> Self {
        let mut standardized = BTreeMap::new();
        for name in ["__module__", "__package__", "__project__"] {
            standardized.insert(name, DunderCategory::Intrinsic);
        }
        for name in [
            "__name__",
            "__id__",
            "__path__",
            "__exports__",
            "__export__",
            "__metadata__",
            "__understands__",
            "__parent__",
            "__children__",
            "__namespace__",
            "__dependencies__",
            "__version__",
            "__selectors__",
            "__definedSelectors__",
            "__classSelectors__",
            "__definedClassSelectors__",
        ] {
            standardized.insert(name, DunderCategory::GuaranteedReflection);
        }
        Self { standardized }
    }
}

impl DunderPolicy {
    pub fn is_dunder(name: &str) -> bool {
        name.len() >= 4 && name.starts_with("__") && name.ends_with("__")
    }

    pub fn category(&self, name: &str) -> Option<DunderCategory> {
        self.standardized.get(name).copied()
    }

    /// Validates a user-authored declaration role. Ordinary names are ignored.
    pub fn validate_user_declaration(&self, name: &str, role: DunderRole) -> Result<(), DunderPolicyError> {
        if !Self::is_dunder(name) {
            return Ok(());
        }
        match self.category(name) {
            Some(DunderCategory::Hook { roles }) if roles.contains(&role) => Ok(()),
            Some(_) => Err(DunderPolicyError::Reserved {
                name: name.to_string(),
                role,
            }),
            None => Err(DunderPolicyError::Unknown { name: name.to_string() }),
        }
    }

    /// Test/tooling extension point proving that future hooks can be admitted by
    /// role without weakening the reservation of the whole dunder namespace.
    pub fn with_hook(mut self, name: &'static str, roles: &'static [DunderRole]) -> Self {
        assert!(Self::is_dunder(name), "standardized hooks must use dunder spelling");
        self.standardized.insert(name, DunderCategory::Hook { roles });
        self
    }
}
