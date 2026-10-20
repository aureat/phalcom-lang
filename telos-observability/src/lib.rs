//! Run-scoped performance observability primitives for Phalcom.
//!
//! This crate owns stable report types and collection mechanics only. Semantic,
//! compiler, and runtime meaning stays with the crates that emit observations.

mod context;
mod metric;
mod phase;
mod query;
mod recorder;
mod report;
mod serialize;

pub use context::{PerfContext, PerfContextInstallGuard, PerfSpanGuard};
pub use metric::PerfCounter;
pub use phase::{ObservabilityLevel, PerfAttribution, PerfOutcome, PerfPhase, PerfScope};
pub use query::{PerfQueryEvent, PerfQueryMetrics};
pub use recorder::PerfRecorder;
pub use report::{PerfDistribution, PerfLifecycle, PerfReport, PerfRunMetadata, PerfSpanRecord, SCHEMA_VERSION};
pub use serialize::PerfReportDecodeError;
