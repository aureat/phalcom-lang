use crate::{
    ObservabilityLevel, PerfAttribution, PerfContext, PerfCounter, PerfDistribution, PerfQueryEvent, PerfQueryMetrics, PerfReport, PerfRunMetadata,
    PerfSpanRecord, SCHEMA_VERSION,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

/// A run-isolated, thread-safe collector. `Off` retains no collection state.
pub struct PerfRecorder {
    metadata: PerfRunMetadata,
    inner: Option<Arc<RecorderInner>>,
}

impl PerfRecorder {
    pub fn new(metadata: PerfRunMetadata) -> Self {
        let inner = metadata.observability_level.collects_summary().then(|| {
            Arc::new(RecorderInner {
                level: metadata.observability_level,
                started: Instant::now(),
                next_span_id: AtomicU64::new(1),
                state: Mutex::new(RecorderState::default()),
            })
        });
        Self { metadata, inner }
    }

    pub fn level(&self) -> ObservabilityLevel {
        self.metadata.observability_level
    }

    pub fn context(&self) -> PerfContext {
        PerfContext::from_recorder(self.inner.clone())
    }

    pub fn report(&self) -> PerfReport {
        let Some(inner) = &self.inner else {
            return PerfReport {
                schema_version: SCHEMA_VERSION,
                metadata: self.metadata.clone(),
                elapsed_ns: 0,
                spans: Vec::new(),
                counters: BTreeMap::new(),
                query_metrics: Vec::new(),
                distributions: BTreeMap::new(),
            };
        };

        let state = lock_unpoisoned(&inner.state);
        let mut spans = state.spans.clone();
        spans.sort_by_key(|span| span.id);
        PerfReport {
            schema_version: SCHEMA_VERSION,
            metadata: self.metadata.clone(),
            elapsed_ns: duration_ns(inner.started.elapsed()),
            spans,
            counters: state.counters.clone(),
            query_metrics: state.query_metrics.values().cloned().collect(),
            distributions: state.distributions.clone(),
        }
    }
}

pub(crate) struct RecorderInner {
    pub(crate) level: ObservabilityLevel,
    started: Instant,
    next_span_id: AtomicU64,
    state: Mutex<RecorderState>,
}

impl RecorderInner {
    pub(crate) fn next_span_id(&self) -> u64 {
        self.next_span_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn start_offset_ns(&self) -> u64 {
        duration_ns(self.started.elapsed())
    }

    pub(crate) fn increment(&self, counter: PerfCounter, amount: u64) {
        if amount == 0 {
            return;
        }
        let mut state = lock_unpoisoned(&self.state);
        let value = state.counters.entry(counter).or_default();
        *value = value.saturating_add(amount);
    }

    pub(crate) fn finish_span(&self, span: PerfSpanRecord) {
        lock_unpoisoned(&self.state).spans.push(span);
    }

    pub(crate) fn record_query_event(&self, query_kind: &str, attribution: Option<PerfAttribution>, event: PerfQueryEvent, duration_ns: Option<u64>) {
        let mut state = lock_unpoisoned(&self.state);
        let key = PerfQueryMetricKey {
            query_kind: query_kind.to_owned(),
            attribution: attribution.clone(),
        };
        let metric = state.query_metrics.entry(key).or_insert_with(|| PerfQueryMetrics {
            query_kind: query_kind.to_owned(),
            attribution,
            ..PerfQueryMetrics::default()
        });
        metric.record(event, duration_ns);
    }

    pub(crate) fn record_distribution_sample(&self, name: &str, sample: u64) {
        let mut state = lock_unpoisoned(&self.state);
        state.distributions.entry(name.to_owned()).or_default().record(sample);
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PerfQueryMetricKey {
    query_kind: String,
    attribution: Option<PerfAttribution>,
}

#[derive(Default)]
struct RecorderState {
    spans: Vec<PerfSpanRecord>,
    counters: BTreeMap<PerfCounter, u64>,
    query_metrics: BTreeMap<PerfQueryMetricKey, PerfQueryMetrics>,
    distributions: BTreeMap<String, PerfDistribution>,
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn duration_ns(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
