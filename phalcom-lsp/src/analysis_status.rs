//! LSP analysis status notifications and state machine tracking.

use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::notification::Notification;

use crate::workspace_scan::AnalysisMode;

/// Coarse-grained operational phase of the LSP analysis pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisPhase {
    /// Analysis worker starting or initializing.
    Starting,
    /// Resolving and loading the physical or bundled core runtime source.
    SelectingCore,
    /// Discovering and shallow-indexing workspace files.
    Indexing,
    /// Performing flow analysis and type solving on source files.
    Analyzing,
    /// Publishing updated semantic database snapshots and diagnostics.
    Publishing,
    /// Analysis pipeline is idle and safe snapshot is published.
    Ready,
    /// Analysis worker encountered an unrecoverable error.
    Error,
}

/// Fine-grained sub-step within a phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisStep {
    /// Scanning directories for `.ph` files.
    Discovering,
    /// Parsing ASTs for discovered source files.
    Parsing,
    /// Populating shallow symbol index for quick navigation.
    ShallowIndexing,
    /// Running control-flow and interprocedural analysis.
    FlowAnalysis,
    /// Solving type constraints and trait bounds.
    Solving,
}

/// Status payload sent via custom `phalcom/analysisStatus` notification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisStatus {
    /// Monotonic session identifier incremented on workspace reconfigurations/re-scans.
    pub session: u64,
    /// Monotonic update sequence within the active session.
    pub sequence: u64,
    /// Primary analysis phase.
    pub phase: AnalysisPhase,
    /// Optional fine-grained sub-step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<AnalysisStep>,
    /// Active analysis scope mode (`local` or `workspace`).
    pub mode: AnalysisMode,
    /// File URI currently being analyzed, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_uri: Option<Url>,
    /// Total workspace files discovered by scanner.
    pub discovered_files: u64,
    /// Workspace files with shallow symbol index available.
    pub indexed_files: u64,
    /// Files with full semantic analysis completed.
    pub analyzed_files: u64,
    /// Published semantic database generation, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    /// `true` when pipeline is fully idle and ready.
    pub complete: bool,
    /// Optional status or error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Custom notification definition for `phalcom/analysisStatus`.
pub struct AnalysisStatusNotification;

impl Notification for AnalysisStatusNotification {
    type Params = AnalysisStatus;
    const METHOD: &'static str = "phalcom/analysisStatus";
}

/// State machine tracking monotonic status counters and phase transitions.
#[derive(Clone, Debug)]
pub struct StatusTracker {
    session: u64,
    sequence: u64,
    discovered_files: u64,
    indexed_files: u64,
    analyzed_files: u64,
    current_phase: AnalysisPhase,
    current_step: Option<AnalysisStep>,
    current_mode: AnalysisMode,
    current_uri: Option<Url>,
    generation: Option<u64>,
    complete: bool,
    message: Option<String>,
}

impl Default for StatusTracker {
    fn default() -> Self {
        Self::new(AnalysisMode::Local)
    }
}

impl StatusTracker {
    /// Creates a new `StatusTracker` initialized to `Starting`.
    pub fn new(mode: AnalysisMode) -> Self {
        Self {
            session: 1,
            sequence: 1,
            discovered_files: 0,
            indexed_files: 0,
            analyzed_files: 0,
            current_phase: AnalysisPhase::Starting,
            current_step: None,
            current_mode: mode,
            current_uri: None,
            generation: None,
            complete: false,
            message: None,
        }
    }

    /// Captures an immutable snapshot of current analysis status.
    pub fn snapshot(&self) -> AnalysisStatus {
        AnalysisStatus {
            session: self.session,
            sequence: self.sequence,
            phase: self.current_phase,
            step: self.current_step,
            mode: self.current_mode,
            current_uri: self.current_uri.clone(),
            discovered_files: self.discovered_files,
            indexed_files: self.indexed_files,
            analyzed_files: self.analyzed_files,
            generation: self.generation,
            complete: self.complete,
            message: self.message.clone(),
        }
    }

    fn bump_sequence(&mut self) {
        self.sequence = self.sequence.saturating_add(1);
    }

    /// Increments session identifier on reconfiguration or full re-scan.
    pub fn increment_session(&mut self, mode: AnalysisMode) -> AnalysisStatus {
        self.session = self.session.saturating_add(1);
        self.sequence = 1;
        self.current_mode = mode;
        self.current_phase = AnalysisPhase::Starting;
        self.current_step = None;
        self.current_uri = None;
        self.complete = false;
        self.message = None;
        self.snapshot()
    }

    /// Updates phase and step, incrementing sequence counter.
    pub fn transition(&mut self, phase: AnalysisPhase, step: Option<AnalysisStep>) -> AnalysisStatus {
        self.bump_sequence();
        self.current_phase = phase;
        self.current_step = step;
        if phase == AnalysisPhase::Ready {
            self.complete = true;
            self.current_uri = None;
            self.current_step = None;
            self.message = None;
        } else {
            self.complete = false;
        }
        self.snapshot()
    }

    /// Sets current mode without changing sequence or emitting.
    pub fn set_mode(&mut self, mode: AnalysisMode) {
        self.current_mode = mode;
    }

    /// Sets file URI currently being processed.
    pub fn set_current_uri(&mut self, uri: Option<Url>) {
        self.current_uri = uri;
    }

    /// Sets latest semantic generation.
    pub fn set_generation(&mut self, generation: u64) {
        self.generation = Some(generation);
    }

    /// Sets status message.
    pub fn set_message(&mut self, message: Option<String>) {
        self.message = message;
    }

    /// Updates file counts and emits updated status snapshot.
    pub fn update_counts(&mut self, discovered: u64, indexed: u64, analyzed: u64) -> AnalysisStatus {
        self.bump_sequence();
        self.discovered_files = discovered;
        self.indexed_files = indexed;
        self.analyzed_files = analyzed;
        self.snapshot()
    }

    /// Transitions to error phase with message.
    pub fn set_error(&mut self, message: String) -> AnalysisStatus {
        self.bump_sequence();
        self.current_phase = AnalysisPhase::Error;
        self.complete = false;
        self.message = Some(message);
        self.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_json_roundtrip() {
        let status = AnalysisStatus {
            session: 1,
            sequence: 3,
            phase: AnalysisPhase::Indexing,
            step: Some(AnalysisStep::Discovering),
            mode: AnalysisMode::Local,
            current_uri: Some(Url::parse("file:///workspace/main.ph").unwrap()),
            discovered_files: 10,
            indexed_files: 5,
            analyzed_files: 1,
            generation: Some(42),
            complete: false,
            message: Some("scanning".to_string()),
        };

        let json = serde_json::to_string(&status).expect("serialize status");
        assert!(json.contains(r#""phase":"indexing""#));
        assert!(json.contains(r#""step":"discovering""#));
        assert!(json.contains(r#""mode":"local""#));

        let deserialized: AnalysisStatus = serde_json::from_str(&json).expect("deserialize status");
        assert_eq!(deserialized, status);
    }

    #[test]
    fn status_tracker_sequence_and_session_increment() {
        let mut tracker = StatusTracker::new(AnalysisMode::Local);
        let initial = tracker.snapshot();
        assert_eq!(initial.session, 1);
        assert_eq!(initial.sequence, 1);
        assert_eq!(initial.phase, AnalysisPhase::Starting);

        let second = tracker.transition(AnalysisPhase::SelectingCore, Some(AnalysisStep::Solving));
        assert_eq!(second.session, 1);
        assert_eq!(second.sequence, 2);
        assert_eq!(second.phase, AnalysisPhase::SelectingCore);

        let new_session = tracker.increment_session(AnalysisMode::Workspace);
        assert_eq!(new_session.session, 2);
        assert_eq!(new_session.sequence, 1);
        assert_eq!(new_session.mode, AnalysisMode::Workspace);

        let error_status = tracker.set_error("test error".to_string());
        assert_eq!(error_status.session, 2);
        assert_eq!(error_status.sequence, 2);
        assert_eq!(error_status.phase, AnalysisPhase::Error);
        assert_eq!(error_status.message, Some("test error".to_string()));
        assert!(!error_status.complete);
    }
}
