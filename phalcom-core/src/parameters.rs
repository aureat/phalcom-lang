//! Shared calling-shape metadata.
//!
//! Syntax-specific parameter declarations are normalized before compilation.
//! Dispatch and binding code can then reason about argument shape without
//! reparsing selector text or encoding rest semantics in a scalar arity.

use crate::interner::Symbol;

/// Rest capture family shared by closures and method signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestKind {
    /// Capture additional positional arguments.
    Positional,
    /// Capture additional labeled arguments.
    Labeled,
    /// Capture both additional positional and labeled arguments.
    Split,
    /// Capture the complete residual argument pack.
    Complete,
}

/// Shape of an evaluated argument pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentShape {
    /// Number of positional arguments in pack.
    pub positional_count: usize,
    /// Labels in source/evaluation order.
    pub labels: Vec<Symbol>,
}

impl ArgumentShape {
    pub fn positional(positional_count: usize) -> Self {
        Self {
            positional_count,
            labels: Vec::new(),
        }
    }
}

/// Normalized parameter acceptance shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterShape {
    /// Number of fixed positional parameters.
    pub fixed_positionals: usize,
    /// Fixed labeled parameters, in selector order.
    pub fixed_labels: Vec<Symbol>,
    /// Optional residual capture family.
    pub rest: Option<RestKind>,
}

impl ParameterShape {
    pub fn closure(fixed_positionals: usize, positional_rest: bool) -> Self {
        Self {
            fixed_positionals,
            fixed_labels: Vec::new(),
            rest: positional_rest.then_some(RestKind::Positional),
        }
    }

    /// Returns whether an argument shape can bind to these parameters.
    pub fn accepts(&self, args: &ArgumentShape) -> bool {
        let fixed = self.fixed_positionals;
        let positional_ok = match self.rest {
            Some(RestKind::Positional | RestKind::Split | RestKind::Complete) => args.positional_count >= fixed,
            _ => args.positional_count == fixed,
        };
        let labels_ok = match self.rest {
            Some(RestKind::Labeled | RestKind::Split | RestKind::Complete) => args.labels.len() >= self.fixed_labels.len(),
            _ => args.labels == self.fixed_labels,
        };
        positional_ok && labels_ok
    }

    /// Produces slot mapping metadata for a successful positional bind.
    pub fn binding_plan(&self, args: &ArgumentShape) -> Option<BindingPlan> {
        if !self.accepts(args) || !self.fixed_labels.is_empty() || !args.labels.is_empty() {
            return None;
        }
        Some(BindingPlan {
            positional_slots: (1..=self.fixed_positionals as u16).collect(),
            rest_slot: self.rest.map(|_| self.fixed_positionals as u16 + 1),
        })
    }
}

/// Local-slot layout for normalized parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterLayout {
    /// Slots holding fixed positional parameters, excluding receiver slot 0.
    pub fixed_slots: Vec<u16>,
    /// Slot holding residual capture, when present.
    pub rest_slot: Option<u16>,
}

/// Result of matching an argument shape to a parameter shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingPlan {
    /// Slots receiving fixed positional values.
    pub positional_slots: Vec<u16>,
    /// Slot receiving residual capture, when present.
    pub rest_slot: Option<u16>,
}

impl From<&ParameterShape> for ParameterLayout {
    fn from(shape: &ParameterShape) -> Self {
        Self {
            fixed_slots: (1..=shape.fixed_positionals as u16).collect(),
            rest_slot: shape.rest.map(|_| shape.fixed_positionals as u16 + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ArgumentShape, ParameterLayout, ParameterShape, RestKind};

    #[test]
    fn closure_shape_accepts_only_fixed_positionals_or_positional_rest() {
        let exact = ParameterShape::closure(2, false);
        assert!(exact.accepts(&ArgumentShape::positional(2)));
        assert!(!exact.accepts(&ArgumentShape::positional(1)));
        assert!(!exact.accepts(&ArgumentShape::positional(3)));

        let rest = ParameterShape::closure(2, true);
        assert!(rest.accepts(&ArgumentShape::positional(2)));
        assert!(rest.accepts(&ArgumentShape::positional(5)));
        assert!(matches!(rest.rest, Some(RestKind::Positional)));
        assert_eq!(ParameterLayout::from(&rest).fixed_slots, vec![1, 2]);
        assert_eq!(ParameterLayout::from(&rest).rest_slot, Some(3));
        assert_eq!(rest.binding_plan(&ArgumentShape::positional(4)).unwrap().rest_slot, Some(3));
    }

    #[test]
    fn labeled_shape_does_not_accept_unlabeled_or_reordered_arguments() {
        let label = crate::interner::Symbol(1);
        let shape = ParameterShape {
            fixed_positionals: 0,
            fixed_labels: vec![label],
            rest: None,
        };
        assert!(!shape.accepts(&ArgumentShape::positional(0)));
        assert!(shape.accepts(&ArgumentShape {
            positional_count: 0,
            labels: vec![label],
        }));
    }
}
