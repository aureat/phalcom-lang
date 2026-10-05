use std::sync::atomic::{AtomicUsize, Ordering};

pub static SEMANTIC_BASELINE_BUILDS: AtomicUsize = AtomicUsize::new(0);
pub static SEMANTIC_SESSION_INSTANTIATIONS: AtomicUsize = AtomicUsize::new(0);

pub fn semantic_baseline_build_count() -> usize {
    SEMANTIC_BASELINE_BUILDS.load(Ordering::Relaxed)
}

pub fn semantic_session_instantiation_count() -> usize {
    SEMANTIC_SESSION_INSTANTIATIONS.load(Ordering::Relaxed)
}

pub fn reset_semantic_counters() {
    SEMANTIC_BASELINE_BUILDS.store(0, Ordering::Relaxed);
    SEMANTIC_SESSION_INSTANTIATIONS.store(0, Ordering::Relaxed);
}

#[inline]
pub fn record_baseline_build() {
    SEMANTIC_BASELINE_BUILDS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_session_instantiation() {
    SEMANTIC_SESSION_INSTANTIATIONS.fetch_add(1, Ordering::Relaxed);
}
