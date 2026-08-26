//! Compact causal invalidity state used by formal checker products.

use crate::identity::DiagnosticCauseId;

/// Cardinality-bounded dependence on invalid upstream judgments.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CausalInvalidity {
    #[default]
    Clean,
    One(DiagnosticCauseId),
    Multiple,
}

/// Non-clean cause payload for expression-level suppression.
///
/// This deliberately has no `Clean` variant, so `Suppressed(Clean)` cannot be
/// represented by the public status type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppressionCause {
    One(DiagnosticCauseId),
    Multiple,
}

impl CausalInvalidity {
    /// Joins causal dependence without selecting an arbitrary root cause.
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Clean, other) | (other, Self::Clean) => other,
            (Self::One(left), Self::One(right)) if left == right => Self::One(left),
            (Self::One(_), Self::One(_)) | (Self::Multiple, _) | (_, Self::Multiple) => Self::Multiple,
        }
    }

    /// Returns whether this bounded causal summary can contain `cause`.
    ///
    /// `Multiple` deliberately does not retain exact cause identity, so any
    /// concrete cause is conservatively considered contained.
    pub fn contains(self, cause: DiagnosticCauseId) -> bool {
        match self {
            Self::Clean => false,
            Self::One(actual) => actual == cause,
            Self::Multiple => true,
        }
    }

    /// Converts causal dependence to the only valid suppression payloads.
    pub fn suppression_cause(self) -> Option<SuppressionCause> {
        match self {
            Self::Clean => None,
            Self::One(cause) => Some(SuppressionCause::One(cause)),
            Self::Multiple => Some(SuppressionCause::Multiple),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CausalInvalidity, SuppressionCause};
    use crate::identity::DiagnosticCauseId;

    #[test]
    fn join_is_idempotent_and_preserves_multiple_roots() {
        let root = CausalInvalidity::One(DiagnosticCauseId(4));
        assert_eq!(CausalInvalidity::Clean.join(root), root);
        assert_eq!(root.join(root), root);
        assert_eq!(root.join(CausalInvalidity::One(DiagnosticCauseId(5))), CausalInvalidity::Multiple);
        assert_eq!(CausalInvalidity::Multiple.join(root), CausalInvalidity::Multiple);
    }

    #[test]
    fn suppression_requires_non_clean_cause() {
        assert_eq!(CausalInvalidity::Clean.suppression_cause(), None);
        assert_eq!(
            CausalInvalidity::One(DiagnosticCauseId(4)).suppression_cause(),
            Some(SuppressionCause::One(DiagnosticCauseId(4)))
        );
        assert_eq!(CausalInvalidity::Multiple.suppression_cause(), Some(SuppressionCause::Multiple));
    }

    #[test]
    fn contains_reports_bounded_cause_membership() {
        let c1 = DiagnosticCauseId(4);
        let c2 = DiagnosticCauseId(5);

        assert!(!CausalInvalidity::Clean.contains(c1));
        assert!(CausalInvalidity::One(c1).contains(c1));
        assert!(!CausalInvalidity::One(c1).contains(c2));
        assert!(CausalInvalidity::Multiple.contains(c1));
        assert!(CausalInvalidity::Multiple.contains(c2));
    }
}
