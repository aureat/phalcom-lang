//! Bounded canonical parity evidence for LSP adapters.
//!
//! Compiler/LSP parity is agreement on the canonical semantic products an LSP
//! adapter consumes. Advisory runtime shapes are a separate semantic domain
//! and are intentionally not compared with formal presentations here.

use phalcom_modules::ModuleId;
use phalcom_semantic::{FormalPresentation, SemanticTargetId};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Maximum number of canonical samples retained for test diagnostics.
pub const MAX_SAMPLES: usize = 64;

/// The LSP surface that consumed a canonical semantic product.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParitySurface {
    /// A hover type/signature comparison.
    Hover,
    /// A receiver/completion comparison.
    Receiver,
    /// An inlay-hint type comparison.
    InlayHint,
}

/// One bounded sample of canonical data consumed by an LSP adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalParitySample {
    /// Surface that consumed the canonical product.
    pub surface: ParitySurface,
    /// Canonical source/module owner for the query.
    pub module: ModuleId,
    /// Canonical semantic target, when the adapter resolved one.
    pub target: Option<SemanticTargetId>,
    /// Formal product consumed by the adapter, preserving its epistemic state.
    pub formal: Option<FormalPresentation>,
}

/// Aggregate counts and bounded-buffer state for canonical parity evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CanonicalParityStats {
    /// Number of canonical observations recorded since the last clear.
    pub observations_total: u64,
    /// Number of hover observations recorded.
    pub hover_observations: u64,
    /// Number of receiver observations recorded.
    pub receiver_observations: u64,
    /// Number of inlay-hint observations recorded.
    pub inlay_hint_observations: u64,
    /// Number of retained diagnostic samples.
    pub retained_samples: usize,
}

#[derive(Debug, Default)]
struct ParityCounts {
    observations_total: AtomicU64,
    hover_observations: AtomicU64,
    receiver_observations: AtomicU64,
    inlay_hint_observations: AtomicU64,
    retained_samples: AtomicU64,
}

/// Records bounded, canonical compiler facts consumed by LSP requests.
///
/// Production backends construct this harness in the disabled state (`None`);
/// the test client explicitly enables it. Once the bounded sample buffer is
/// full, normal queries update only atomics and do not contend on the sample
/// mutex or allocate retained evidence.
#[derive(Clone, Debug, Default)]
pub struct CanonicalParityHarness {
    counts: Arc<ParityCounts>,
    samples: Arc<Mutex<VecDeque<CanonicalParitySample>>>,
}

impl CanonicalParityHarness {
    /// Creates an empty bounded parity harness.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a hover adapter consuming canonical compiler data.
    pub fn observe_hover(&self, module: &ModuleId, target: &SemanticTargetId, formal: Option<&FormalPresentation>) {
        self.observe(ParitySurface::Hover, module, Some(target), formal);
    }

    /// Records a receiver/completion adapter consuming canonical compiler data.
    pub fn observe_receiver(&self, module: &ModuleId, target: Option<&SemanticTargetId>, formal: Option<&FormalPresentation>) {
        self.observe(ParitySurface::Receiver, module, target, formal);
    }

    /// Records an inlay-hint adapter consuming canonical compiler data.
    pub fn observe_inlay_hint(&self, module: &ModuleId, target: Option<&SemanticTargetId>, formal: Option<&FormalPresentation>) {
        self.observe(ParitySurface::InlayHint, module, target, formal);
    }

    /// Returns aggregate counts and the number of retained samples.
    pub fn stats(&self) -> CanonicalParityStats {
        CanonicalParityStats {
            observations_total: self.counts.observations_total.load(Ordering::Relaxed),
            hover_observations: self.counts.hover_observations.load(Ordering::Relaxed),
            receiver_observations: self.counts.receiver_observations.load(Ordering::Relaxed),
            inlay_hint_observations: self.counts.inlay_hint_observations.load(Ordering::Relaxed),
            retained_samples: self.counts.retained_samples.load(Ordering::Relaxed) as usize,
        }
    }

    /// Returns the bounded canonical samples retained for diagnostics.
    pub fn samples(&self) -> Vec<CanonicalParitySample> {
        self.samples.lock().expect("parity sample lock poisoned").iter().cloned().collect()
    }

    /// Compatibility accessor for the former observation terminology.
    pub fn observations(&self) -> Vec<CanonicalParitySample> {
        self.samples()
    }

    /// Clears aggregate counts and retained samples for a fresh test run.
    pub fn clear(&self) {
        self.samples.lock().expect("parity sample lock poisoned").clear();
        self.counts.observations_total.store(0, Ordering::Relaxed);
        self.counts.hover_observations.store(0, Ordering::Relaxed);
        self.counts.receiver_observations.store(0, Ordering::Relaxed);
        self.counts.inlay_hint_observations.store(0, Ordering::Relaxed);
        self.counts.retained_samples.store(0, Ordering::Relaxed);
    }

    fn observe(&self, surface: ParitySurface, module: &ModuleId, target: Option<&SemanticTargetId>, formal: Option<&FormalPresentation>) {
        self.counts.observations_total.fetch_add(1, Ordering::Relaxed);
        match surface {
            ParitySurface::Hover => self.counts.hover_observations.fetch_add(1, Ordering::Relaxed),
            ParitySurface::Receiver => self.counts.receiver_observations.fetch_add(1, Ordering::Relaxed),
            ParitySurface::InlayHint => self.counts.inlay_hint_observations.fetch_add(1, Ordering::Relaxed),
        };

        let reserved = self
            .counts
            .retained_samples
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |count| (count < MAX_SAMPLES as u64).then_some(count + 1))
            .is_ok();
        if !reserved {
            return;
        }

        self.samples.lock().expect("parity sample lock poisoned").push_back(CanonicalParitySample {
            surface,
            module: module.clone(),
            target: target.cloned(),
            formal: formal.cloned(),
        });
    }
}

/// Compatibility name for callers that used the initial shadow-harness API.
/// The retained data is canonical-only; this alias does not restore the old
/// formal/advisory string comparison behavior.
pub type ShadowParityHarness = CanonicalParityHarness;

/// Compatibility name for the old observation type.
pub type ParityObservation = CanonicalParitySample;

#[cfg(test)]
mod tests {
    use super::*;

    fn module() -> ModuleId {
        ModuleId::universe(phalcom_modules::identity::ModulePath::from_components(vec![
            phalcom_modules::identity::ModuleComponent::from_identifier("test").unwrap(),
        ]))
    }

    #[test]
    fn canonical_samples_preserve_identity_and_formal_state() {
        let harness = CanonicalParityHarness::new();
        let module = module();
        let target = SemanticTargetId::Module(module.clone());
        let formal = FormalPresentation::Known("Int | String".into());
        harness.observe_hover(&module, &target, Some(&formal));

        assert_eq!(harness.stats().observations_total, 1);
        assert_eq!(
            harness.samples(),
            vec![CanonicalParitySample {
                surface: ParitySurface::Hover,
                module,
                target: Some(target),
                formal: Some(formal),
            }]
        );
    }

    #[test]
    fn samples_are_bounded_but_aggregate_counts_continue() {
        let harness = CanonicalParityHarness::new();
        let module = module();
        let target = SemanticTargetId::Module(module.clone());
        let formal = FormalPresentation::Unknown;
        for _ in 0..(MAX_SAMPLES + 12) {
            harness.observe_inlay_hint(&module, Some(&target), Some(&formal));
        }

        let stats = harness.stats();
        assert_eq!(stats.observations_total, (MAX_SAMPLES + 12) as u64);
        assert_eq!(stats.inlay_hint_observations, (MAX_SAMPLES + 12) as u64);
        assert_eq!(stats.retained_samples, MAX_SAMPLES);
        assert_eq!(harness.samples().len(), MAX_SAMPLES);
    }

    #[test]
    fn cloned_harnesses_share_and_clear_evidence() {
        let harness = CanonicalParityHarness::new();
        let clone = harness.clone();
        let module = module();
        let target = SemanticTargetId::Module(module.clone());
        clone.observe_hover(&module, &target, None);

        assert_eq!(harness.stats().observations_total, 1);
        harness.clear();
        assert_eq!(clone.stats(), CanonicalParityStats::default());
        assert!(clone.samples().is_empty());
    }
}
