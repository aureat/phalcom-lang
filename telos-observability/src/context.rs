use crate::{PerfAttribution, PerfCounter, PerfOutcome, PerfPhase, PerfQueryEvent, PerfSpanRecord};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use crate::recorder::RecorderInner;

thread_local! {
    static CURRENT_CONTEXTS: std::cell::RefCell<Vec<PerfContext>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

/// A cheap handle that carries run and parent-span identity across layers.
#[derive(Clone, Default)]
pub struct PerfContext {
    inner: Option<Arc<RecorderInner>>,
    parent_id: Option<u64>,
    attribution: Option<PerfAttribution>,
}

impl PerfContext {
    pub(crate) fn from_recorder(inner: Option<Arc<RecorderInner>>) -> Self {
        Self {
            inner,
            parent_id: None,
            attribution: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.is_some()
    }

    /// Returns the context installed by the current run on this thread.
    pub fn current_thread() -> Self {
        CURRENT_CONTEXTS.with(|contexts| contexts.borrow().last().cloned().unwrap_or_default())
    }

    /// Installs this context for synchronous work on the current thread.
    ///
    /// This is used at boundaries that must observe process-cold singleton
    /// initialization without adding instrumentation parameters to every API.
    pub fn install_current_thread(&self) -> PerfContextInstallGuard {
        let previous_depth = CURRENT_CONTEXTS.with(|contexts| {
            let mut contexts = contexts.borrow_mut();
            let previous_depth = contexts.len();
            contexts.push(self.clone());
            previous_depth
        });
        PerfContextInstallGuard {
            previous_depth,
            _not_send: std::marker::PhantomData,
        }
    }

    pub fn record_counter(&self, counter: PerfCounter, amount: u64) {
        if let Some(inner) = &self.inner {
            inner.increment(counter, amount);
        }
    }

    /// Records one sample into a bounded count/sum/max distribution. Samples
    /// are retained only in Detailed and Trace modes.
    pub fn record_distribution_sample(&self, name: &str, sample: u64) {
        if let Some(inner) = &self.inner
            && inner.level.collects_details()
        {
            inner.record_distribution_sample(name, sample);
        }
    }

    /// Records one semantic query lifecycle event, aggregated by stable query
    /// kind and by attribution only when detailed collection is enabled.
    pub fn record_query_event(&self, query_kind: &str, event: PerfQueryEvent, duration_ns: Option<u64>) {
        if let Some(inner) = &self.inner {
            let attribution = inner.level.collects_details().then(|| self.attribution.clone()).flatten();
            inner.record_query_event(query_kind, attribution, event, duration_ns);
        }
    }

    /// Add optional module/entity attribution to child spans.
    ///
    /// Attribution is retained only at `Detailed` and `Trace` levels.
    pub fn with_attribution(mut self, attribution: PerfAttribution) -> Self {
        if self.inner.as_ref().is_some_and(|inner| inner.level.collects_details()) {
            self.attribution = Some(attribution);
        }
        self
    }

    /// Runs a synchronous operation as a successful phase unless it panics.
    pub fn with_span<T>(&self, phase: PerfPhase, operation: impl FnOnce(PerfContext) -> T) -> T {
        let span = self.span(phase);
        let child = span.child_context();
        let installed = child.install_current_thread();
        let result = operation(child);
        drop(installed);
        span.finish(PerfOutcome::Success);
        result
    }

    /// Runs a fallible operation and records its caller-defined failure class.
    pub fn with_span_result<T, E>(
        &self,
        phase: PerfPhase,
        operation: impl FnOnce(PerfContext) -> Result<T, E>,
        classify_error: impl FnOnce(&E) -> PerfOutcome,
    ) -> Result<T, E> {
        let span = self.span(phase);
        let child = span.child_context();
        let installed = child.install_current_thread();
        let result = operation(child);
        drop(installed);
        let outcome = match &result {
            Ok(_) => PerfOutcome::Success,
            Err(error) => classify_error(error),
        };
        span.finish(outcome);
        result
    }

    pub fn span(&self, phase: PerfPhase) -> PerfSpanGuard {
        let Some(inner) = &self.inner else {
            return PerfSpanGuard::disabled();
        };
        let id = inner.next_span_id();
        let attribution = inner.level.collects_details().then(|| self.attribution.clone()).flatten();
        let started = Instant::now();
        let start_offset_ns = inner.start_offset_ns();
        PerfSpanGuard {
            active: Some(ActiveSpan {
                inner: inner.clone(),
                id,
                parent_id: self.parent_id,
                phase,
                start_offset_ns,
                started,
                attribution,
                tags: BTreeMap::new(),
            }),
        }
    }
}

/// Restores the prior thread-local context stack when the installed scope ends.
/// This guard is deliberately !Send because it must be dropped on its creating
/// thread.
pub struct PerfContextInstallGuard {
    previous_depth: usize,
    _not_send: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl Drop for PerfContextInstallGuard {
    fn drop(&mut self) {
        CURRENT_CONTEXTS.with(|contexts| contexts.borrow_mut().truncate(self.previous_depth));
    }
}

struct ActiveSpan {
    inner: Arc<RecorderInner>,
    id: u64,
    parent_id: Option<u64>,
    phase: PerfPhase,
    start_offset_ns: u64,
    started: Instant,
    attribution: Option<PerfAttribution>,
    tags: BTreeMap<String, String>,
}

/// RAII span guard. A guard dropped without an explicit outcome is recorded as
/// failed (or as a panic boundary while unwinding), never as successful.
pub struct PerfSpanGuard {
    active: Option<ActiveSpan>,
}

impl PerfSpanGuard {
    fn disabled() -> Self {
        Self { active: None }
    }

    pub fn child_context(&self) -> PerfContext {
        let Some(active) = &self.active else {
            return PerfContext::default();
        };
        PerfContext {
            inner: Some(active.inner.clone()),
            parent_id: Some(active.id),
            attribution: active.attribution.clone(),
        }
    }

    pub fn tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let Some(active) = &mut self.active else {
            return;
        };
        if active.inner.level.collects_details() {
            active.tags.insert(key.into(), value.into());
        }
    }

    pub fn finish(mut self, outcome: PerfOutcome) {
        self.close(outcome);
    }

    fn close(&mut self, fallback: PerfOutcome) {
        let Some(active) = self.active.take() else {
            return;
        };
        active.inner.finish_span(PerfSpanRecord {
            id: active.id,
            parent_id: active.parent_id,
            phase: active.phase,
            start_offset_ns: active.start_offset_ns,
            duration_ns: u64::try_from(active.started.elapsed().as_nanos()).unwrap_or(u64::MAX),
            outcome: fallback,
            attribution: active.attribution,
            tags: active.tags,
        });
    }
}

impl Drop for PerfSpanGuard {
    fn drop(&mut self) {
        let fallback = if std::thread::panicking() {
            PerfOutcome::PanicBoundary
        } else {
            PerfOutcome::OtherFailure
        };
        self.close(fallback);
    }
}
