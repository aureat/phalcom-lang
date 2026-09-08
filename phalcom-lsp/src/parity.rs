//! Shadow Parity Harness for LSP queries (Spec 04.5 / Wave 6 Workstream L).
//!
//! Under DEC-IMPL-LSP-PARITY-COMPATIBILITY, queries check formal compiler
//! products against legacy advisory facts and record divergences without
//! disrupting user-visible LSP responses.

use std::sync::{Arc, Mutex};

/// The LSP surface on which a formal/advisory divergence was observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParitySurface {
    /// A hover type comparison.
    Hover,
    /// A receiver/completion comparison.
    Receiver,
    /// An inlay-hint type comparison.
    InlayHint,
}

/// A retained formal/advisory divergence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityMismatch {
    /// Surface that produced the mismatch.
    pub surface: ParitySurface,
    /// User-facing target or binding name associated with the comparison.
    pub target_name: String,
    /// Formal compiler representation, if one was available.
    pub formal: Option<String>,
    /// Advisory representation, if one was available.
    pub advisory: Option<String>,
}

/// One formal/advisory comparison observed by a production LSP query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityObservation {
    /// Surface that produced the comparison.
    pub surface: ParitySurface,
    /// User-facing target or binding name associated with the comparison.
    pub target_name: String,
    /// Formal compiler representation, if one was available.
    pub formal: Option<String>,
    /// Advisory representation, if one was available.
    pub advisory: Option<String>,
}

/// Records shadow comparisons between formal compiler facts and advisory LSP facts.
///
/// The harness is intentionally observational: recording a mismatch never
/// changes an LSP response. Tests and diagnostics can inspect the retained
/// mismatches and assert that a parity-sensitive query stayed aligned.
#[derive(Clone, Debug, Default)]
pub struct ShadowParityHarness {
    mismatches: Arc<Mutex<Vec<ParityMismatch>>>,
    observations: Arc<Mutex<Vec<ParityObservation>>>,
}

impl ShadowParityHarness {
    /// Creates a new shadow parity harness instance.
    pub fn new() -> Self {
        Self {
            mismatches: Arc::new(Mutex::new(Vec::new())),
            observations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Records hover parity between formal type representation and advisory inferred value.
    pub fn record_hover_parity(&self, target_name: &str, formal_type: Option<&str>, advisory_type: Option<&str>) {
        self.record(ParitySurface::Hover, target_name, formal_type, advisory_type);
    }

    /// Records receiver/completion parity between formal resolved receiver and advisory receiver.
    pub fn record_receiver_parity(&self, receiver_name: &str, formal_classes: &[String], advisory_classes: &[String]) {
        let formal = (!formal_classes.is_empty()).then(|| formal_classes.join(", "));
        let advisory = (!advisory_classes.is_empty()).then(|| advisory_classes.join(", "));
        self.record(ParitySurface::Receiver, receiver_name, formal.as_deref(), advisory.as_deref());
    }

    /// Records inlay hint parity between formal binding type and advisory runtime shape.
    pub fn record_inlay_hint_parity(&self, binding_name: &str, formal_type: Option<&str>, advisory_shape: Option<&str>) {
        self.record(ParitySurface::InlayHint, binding_name, formal_type, advisory_shape);
    }

    /// Returns a snapshot of all retained mismatches.
    pub fn mismatches(&self) -> Vec<ParityMismatch> {
        self.mismatches.lock().expect("parity mismatch lock poisoned").clone()
    }

    /// Returns a snapshot of every comparison observed by production queries.
    pub fn observations(&self) -> Vec<ParityObservation> {
        self.observations.lock().expect("parity observation lock poisoned").clone()
    }

    /// Returns the number of retained mismatches.
    pub fn mismatch_count(&self) -> usize {
        self.mismatches.lock().expect("parity mismatch lock poisoned").len()
    }

    /// Clears retained mismatches so a harness can be reused for another run.
    pub fn clear(&self) {
        self.mismatches.lock().expect("parity mismatch lock poisoned").clear();
        self.observations.lock().expect("parity observation lock poisoned").clear();
    }

    /// Panics with the retained evidence if any parity mismatch was observed.
    pub fn assert_no_mismatches(&self) {
        let mismatches = self.mismatches();
        assert!(mismatches.is_empty(), "formal/advisory parity mismatches: {mismatches:#?}");
    }

    fn record(&self, surface: ParitySurface, target_name: &str, formal: Option<&str>, advisory: Option<&str>) {
        self.observations.lock().expect("parity observation lock poisoned").push(ParityObservation {
            surface,
            target_name: target_name.to_string(),
            formal: formal.map(str::to_string),
            advisory: advisory.map(str::to_string),
        });
        if formal == advisory {
            return;
        }
        self.mismatches.lock().expect("parity mismatch lock poisoned").push(ParityMismatch {
            surface,
            target_name: target_name.to_string(),
            formal: formal.map(str::to_string),
            advisory: advisory.map(str::to_string),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parity_harness_records_without_panic() {
        let harness = ShadowParityHarness::new();
        harness.record_hover_parity("x", Some("Int"), Some("Int"));
        harness.record_hover_parity("x", Some("Int"), Some("String"));
        harness.record_hover_parity("x", Some("Int"), None);
        harness.record_hover_parity("x", None, Some("Int"));
        harness.record_hover_parity("x", None, None);

        harness.record_receiver_parity("u", &["User".into()], &["User".into()]);
        harness.record_receiver_parity("u", &["User".into()], &[]);

        harness.record_inlay_hint_parity("x", Some("Int"), Some("Int"));
        harness.record_inlay_hint_parity("x", Some("Int"), None);

        assert_eq!(harness.mismatch_count(), 5);
        assert_eq!(
            harness.mismatches()[0],
            ParityMismatch {
                surface: ParitySurface::Hover,
                target_name: "x".into(),
                formal: Some("Int".into()),
                advisory: Some("String".into()),
            }
        );
    }

    #[test]
    fn matching_formal_and_advisory_facts_are_assertion_clean() {
        let harness = ShadowParityHarness::new();
        harness.record_hover_parity("x", Some("Int"), Some("Int"));
        harness.record_receiver_parity("receiver", &["User".into()], &["User".into()]);
        harness.record_inlay_hint_parity("x", None, None);

        harness.assert_no_mismatches();
        assert_eq!(harness.mismatches(), Vec::new());
    }

    #[test]
    fn cloned_harnesses_share_observations_and_clear_together() {
        let harness = ShadowParityHarness::new();
        let clone = harness.clone();
        clone.record_hover_parity("x", Some("Int"), Some("String"));

        assert_eq!(harness.mismatch_count(), 1);
        assert_eq!(clone.observations().len(), 1);
        harness.clear();
        assert_eq!(clone.mismatch_count(), 0);
        assert_eq!(clone.observations().len(), 0);
    }
}
