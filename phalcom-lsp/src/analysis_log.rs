//! Structured analysis logging for LSP notifications and observability.

use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::notification::Notification;

use crate::analysis_status::AnalysisPhase;
use crate::perf::CounterSnapshot;

/// Log level for structured analysis events.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisLogLevel {
    /// Severe error during analysis.
    Error,
    /// Informational lifecycle or progress event.
    Info,
    /// Fine-grained internal event.
    Verbose,
}

/// Structured analysis log payload sent via `phalcom/analysisLog`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisLogEvent {
    /// Monotonic session identifier.
    pub session: u64,
    /// Monotonic sequence within active session.
    pub sequence: u64,
    /// Log severity level.
    pub level: AnalysisLogLevel,
    /// Analysis phase at time of event.
    pub phase: AnalysisPhase,
    /// Stable event name (e.g. `formal.update.published`).
    pub event: String,
    /// Active work epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u64>,
    /// Semantic generation number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    /// File URI involved in event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,
    /// Source file revision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    /// Size of batch in files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<u32>,
    /// Duration of operation in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Optional human-readable message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Performance counter snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counters: Option<CounterSnapshot>,
}

/// Custom notification definition for `phalcom/analysisLog`.
pub struct AnalysisLogNotification;

impl Notification for AnalysisLogNotification {
    type Params = AnalysisLogEvent;
    const METHOD: &'static str = "phalcom/analysisLog";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_log_event_serde_roundtrip() {
        let event = AnalysisLogEvent {
            session: 2,
            sequence: 14,
            level: AnalysisLogLevel::Info,
            phase: AnalysisPhase::Analyzing,
            event: "formal.update.published".to_string(),
            epoch: Some(3),
            generation: Some(10),
            uri: Some(Url::parse("file:///workspace/main.ph").unwrap()),
            revision: Some(1),
            batch_size: Some(1),
            duration_ms: Some(25),
            message: Some("analysis succeeded".to_string()),
            counters: None,
        };

        let json = serde_json::to_string(&event).expect("serialize log event");
        assert!(json.contains(r#""event":"formal.update.published""#));
        assert!(json.contains(r#""level":"info""#));

        let deserialized: AnalysisLogEvent = serde_json::from_str(&json).expect("deserialize log event");
        assert_eq!(deserialized, event);
    }
}
