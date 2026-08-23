//! Progressive workspace source scanner.
//!
//! Implements chunked, yieldable discovery of `.ph` files under workspace roots.
//! The scanner processes bounded budgets of directory starts, directory
//! entries, and files per step, returning control to the worker loop between
//! steps so that interactive (open-document) work can preempt background
//! scanning.
//!
//! Exclusion rules:
//! - Built-in: hidden directories (`.`-prefixed), `target`, `node_modules`.
//! - User-configured: path fragment strings from `phalcom.analysis.exclude`.
//!
//! Analysis modes:
//! - [`AnalysisMode::Local`]: discovers files for shallow indexing/navigation but
//!   restricts deep flow analysis to open documents and their transitive imports.
//! - [`AnalysisMode::Workspace`]: after interactive/local closure converges, deep-
//!   analyzes remaining discovered workspace modules in the background.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use tower_lsp::lsp_types::Url;

use crate::perf::PerfCounters;

// ---------------------------------------------------------------------------
// Analysis mode
// ---------------------------------------------------------------------------

/// Controls the scope of deep interprocedural flow analysis.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisMode {
    /// Restrict deep analysis to open documents and their transitive imports.
    /// Background workspace indexing still progresses for navigation.
    #[default]
    Local,
    /// After interactive/local closure converges, deep-analyze remaining
    /// discovered workspace modules in the background.
    Workspace,
}

/// Error returned when parsing an invalid analysis mode string.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParseAnalysisModeError(pub String);

impl std::fmt::Display for ParseAnalysisModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid analysis mode `{}`; expected `local` or `workspace`", self.0)
    }
}

impl std::error::Error for ParseAnalysisModeError {}

impl std::str::FromStr for AnalysisMode {
    type Err = ParseAnalysisModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "local" => Ok(Self::Local),
            "workspace" => Ok(Self::Workspace),
            other => Err(ParseAnalysisModeError(other.to_owned())),
        }
    }
}

// ---------------------------------------------------------------------------
// Exclusion matcher
// ---------------------------------------------------------------------------

/// Evaluates path exclusion rules.
///
/// Built-in rules exclude hidden (dot-prefixed) names, `target`, and
/// `node_modules`. User rules are matched as exact path component names or
/// as substring matches within the full path string.
#[derive(Clone, Debug, Default)]
pub struct ExcludeMatcher {
    /// User-supplied path fragment exclusions.
    user_rules: Vec<String>,
}

impl ExcludeMatcher {
    /// Construct a matcher from user-supplied rules.
    pub fn new(user_rules: &[String]) -> Self {
        Self {
            user_rules: user_rules.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        }
    }

    /// Returns `true` if `path` should be excluded from scanning.
    pub fn is_excluded(&self, path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Always exclude hidden (dot-prefixed) names.
        if name.starts_with('.') {
            return true;
        }
        // Built-in directory exclusions.
        if matches!(name, "target" | "node_modules") {
            return true;
        }
        // User-supplied exclusions (path component match or substring).
        if !self.user_rules.is_empty() {
            let path_str = path.to_string_lossy().replace('\\', "/");
            for rule in &self.user_rules {
                let normalized_rule = rule.replace('\\', "/");
                // Exact path component match
                if path.components().any(|c| c.as_os_str().to_str() == Some(rule.as_str())) {
                    return true;
                }
                // Treat common `**/name/**` forms as path-fragment rules.
                let fragment = normalized_rule.trim_matches('/').trim_matches('*').trim_matches('/');
                if !fragment.is_empty() && path_str.contains(fragment) {
                    return true;
                }
                // A terminal `*.ph` rule applies to matching file names.
                if let Some(file_pattern) = normalized_rule.rsplit('/').next()
                    && file_pattern.contains('*')
                    && wildcard_match(name, file_pattern)
                {
                    return true;
                }
            }
        }
        false
    }
}

fn wildcard_match(value: &str, pattern: &str) -> bool {
    let (mut value_index, mut pattern_index) = (0, 0);
    let (mut star, mut retry) = (None, 0);
    let value = value.as_bytes();
    let pattern = pattern.as_bytes();
    while value_index < value.len() {
        if pattern_index < pattern.len() && (pattern[pattern_index] == value[value_index] || pattern[pattern_index] == b'?') {
            value_index += 1;
            pattern_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star = Some(pattern_index);
            retry = value_index;
            pattern_index += 1;
        } else if let Some(star_index) = star {
            pattern_index = star_index + 1;
            retry += 1;
            value_index = retry;
        } else {
            return false;
        }
    }
    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

// ---------------------------------------------------------------------------
// Scan budget
// ---------------------------------------------------------------------------

/// Maximum work units for one scanner step before yielding back to the
/// worker loop. After each step the worker loop re-checks for interactive
/// pending work so that open-document updates preempt background scanning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanBudget {
    /// Maximum number of directories to start reading in one step.
    pub max_dirs_started: usize,
    /// Maximum number of directory entries to consume in one step.
    pub max_entries: usize,
    /// Maximum number of `.ph` files to collect in one step.
    pub max_files: usize,
}

impl Default for ScanBudget {
    fn default() -> Self {
        SCAN_BUDGET
    }
}

/// Conservative default: keeps interactive edit latency low while making
/// steady scanning progress in quiet periods.
pub const SCAN_BUDGET: ScanBudget = ScanBudget {
    max_dirs_started: 16,
    max_entries: 256,
    max_files: 32,
};

// ---------------------------------------------------------------------------
// Discovered file
// ---------------------------------------------------------------------------

/// A `.ph` file found during a scan step.
#[derive(Clone, Debug)]
pub struct DiscoveredFile {
    /// Absolute filesystem path.
    pub path: PathBuf,
    /// Derived LSP URL.
    pub uri: Url,
}

// ---------------------------------------------------------------------------
// Scanner state
// ---------------------------------------------------------------------------

/// Progressive workspace scanner state.
///
/// The scanner maintains its own cursor (pending directories and files) so
/// that scanning can be paused and resumed between worker-loop iterations.
/// The worker loop calls [`Self::step`], processes the batch, and loops so
/// that newly queued interactive work can preempt the next step.
pub struct WorkspaceScanState {
    /// Directories yet to be traversed.
    pending_dirs: Vec<PathBuf>,
    /// Directory currently being consumed, if any.
    active_dir: Option<std::fs::ReadDir>,
    /// `.ph` files found but not yet emitted.
    pending_files: VecDeque<PathBuf>,
    /// Workspace roots provided by the client.
    roots: Vec<PathBuf>,
    /// Path exclusion policy.
    excluded: ExcludeMatcher,
    /// Analysis mode: controls whether background deep analysis is scheduled.
    pub mode: AnalysisMode,
    /// Physical path of the selected core source file, if any.
    /// Files at this path are skipped from ordinary workspace registration.
    core_physical_path: Option<PathBuf>,
}

impl WorkspaceScanState {
    /// Construct an idle scanner.
    ///
    /// Call [`Self::set_roots`] to prime discovery.
    pub fn new(mode: AnalysisMode, excluded: ExcludeMatcher) -> Self {
        Self {
            pending_dirs: Vec::new(),
            active_dir: None,
            pending_files: VecDeque::new(),
            roots: Vec::new(),
            excluded,
            mode,
            core_physical_path: None,
        }
    }

    /// Returns `true` if there is pending scanning work remaining.
    pub fn has_work(&self) -> bool {
        self.active_dir.is_some() || !self.pending_dirs.is_empty() || !self.pending_files.is_empty()
    }

    /// Set workspace roots and reset scanner state.
    ///
    /// This replaces all pending work so that discovery restarts from the new
    /// roots (e.g. after `workspaceFolders/didChange`).
    pub fn set_roots(&mut self, roots: Vec<PathBuf>, core_physical_path: Option<PathBuf>) {
        self.roots = roots;
        self.pending_dirs.clear();
        self.active_dir = None;
        self.pending_files.clear();
        self.core_physical_path = core_physical_path.map(|path| path.canonicalize().unwrap_or(path));
        // Seed dirs from roots.
        for root in &self.roots {
            if root.is_dir() && !self.excluded.is_excluded(root) {
                self.pending_dirs.push(root.clone());
            }
        }
    }

    /// Update exclusion rules and analysis mode without resetting discovery.
    pub fn update_config(&mut self, mode: AnalysisMode, excluded: ExcludeMatcher) {
        self.mode = mode;
        self.excluded = excluded;
    }

    /// Advance scanner by up to `budget` work units.
    ///
    /// Starts up to `budget.max_dirs_started` directories, consumes up to
    /// `budget.max_entries` entries, then emits up to `budget.max_files` files.
    ///
    /// The open directory iterator is retained between calls, so a wide
    /// directory cannot monopolize one scanner step.
    ///
    /// Returns an empty `Vec` when [`Self::has_work`] is also `false`.
    pub fn step(&mut self, budget: ScanBudget) -> Vec<DiscoveredFile> {
        self.step_with_counters(budget, None)
    }

    /// Advance scanner while recording consumed directory entries.
    pub fn step_with_counters(&mut self, budget: ScanBudget, counters: Option<&PerfCounters>) -> Vec<DiscoveredFile> {
        let mut dirs_started = 0;
        let mut entries_consumed = 0;

        // Expand directories. Keep the current ReadDir alive when the entry
        // budget is exhausted so the next step resumes at the same entry.
        while entries_consumed < budget.max_entries {
            if self.active_dir.is_none() {
                if dirs_started >= budget.max_dirs_started {
                    break;
                }
                let Some(dir) = self.pending_dirs.pop() else {
                    break;
                };
                dirs_started += 1;
                if self.excluded.is_excluded(&dir) {
                    continue;
                }
                let Ok(entries) = std::fs::read_dir(&dir) else {
                    continue;
                };
                self.active_dir = Some(entries);
            }

            let next_entry = self.active_dir.as_mut().and_then(|entries| entries.next());
            let Some(next_entry) = next_entry else {
                self.active_dir = None;
                continue;
            };
            entries_consumed += 1;
            if let Some(counters) = counters {
                counters.scan_directory_entries_consumed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }

            let Ok(entry) = next_entry else {
                continue;
            };
            let path = entry.path();
            if path.is_dir() {
                if !self.excluded.is_excluded(&path) {
                    self.pending_dirs.push(path);
                }
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("ph") && !self.excluded.is_excluded(&path) {
                self.pending_files.push_back(path);
            }
        }

        // Emit files.
        let mut discovered = Vec::new();
        while discovered.len() < budget.max_files {
            let Some(path) = self.pending_files.pop_front() else {
                break;
            };
            // Skip selected physical core path — registered under CORE_MODULE_URI separately.
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
            if self.core_physical_path.as_deref().is_some_and(|cp| cp == canonical_path.as_path()) {
                continue;
            }
            if let Ok(uri) = Url::from_file_path(&path) {
                discovered.push(DiscoveredFile { path, uri });
            }
        }

        discovered
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmpdir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("phalcom_wscan_{name}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn scanner_discovers_ph_files_excluding_builtin() {
        let root = tmpdir("builtin_excl");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("src/nested")).unwrap();
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join("src/a.ph"), "class A {}").unwrap();
        fs::write(root.join("src/b.ph"), "class B {}").unwrap();
        fs::write(root.join("src/nested/c.ph"), "class C {}").unwrap();
        fs::write(root.join("target/debug/x.ph"), "class X {}").unwrap();
        fs::write(root.join("not_phalcom.rs"), "").unwrap();

        let mut scanner = WorkspaceScanState::new(AnalysisMode::Local, ExcludeMatcher::new(&[]));
        scanner.set_roots(vec![root.clone()], None);

        let mut all = Vec::new();
        loop {
            let batch = scanner.step(SCAN_BUDGET);
            if batch.is_empty() && !scanner.has_work() {
                break;
            }
            all.extend(batch);
        }

        let names: std::collections::BTreeSet<_> = all.iter().map(|f| f.path.file_name().unwrap().to_str().unwrap().to_string()).collect();

        assert!(names.contains("a.ph"), "missing a.ph: {names:?}");
        assert!(names.contains("b.ph"), "missing b.ph: {names:?}");
        assert!(names.contains("c.ph"), "missing c.ph: {names:?}");
        assert!(!names.contains("x.ph"), "target/x.ph should be excluded: {names:?}");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scanner_respects_user_exclusion_rules() {
        let root = tmpdir("user_excl");
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("lib/main.ph"), "").unwrap();
        fs::write(root.join("tests/test.ph"), "").unwrap();

        let mut scanner = WorkspaceScanState::new(AnalysisMode::Local, ExcludeMatcher::new(&["tests".to_string()]));
        scanner.set_roots(vec![root.clone()], None);

        let mut all = Vec::new();
        loop {
            let batch = scanner.step(SCAN_BUDGET);
            if batch.is_empty() && !scanner.has_work() {
                break;
            }
            all.extend(batch);
        }

        let names: std::collections::BTreeSet<_> = all.iter().map(|f| f.path.file_name().unwrap().to_str().unwrap().to_string()).collect();

        assert!(names.contains("main.ph"), "missing main.ph: {names:?}");
        assert!(!names.contains("test.ph"), "tests/ should be excluded: {names:?}");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scanner_skips_canonical_core_physical_path() {
        let root = tmpdir("core_skip");
        fs::create_dir_all(root.join("core/universe/src")).unwrap();
        let core_path = root.join("core/universe/src/package.ph");
        let other_path = root.join("other.ph");
        fs::write(&core_path, "").unwrap();
        fs::write(&other_path, "").unwrap();

        let mut scanner = WorkspaceScanState::new(AnalysisMode::Workspace, ExcludeMatcher::new(&[]));
        scanner.set_roots(vec![root.clone()], Some(core_path.clone()));

        let mut all = Vec::new();
        loop {
            let batch = scanner.step(SCAN_BUDGET);
            if batch.is_empty() && !scanner.has_work() {
                break;
            }
            all.extend(batch);
        }

        let paths: std::collections::BTreeSet<_> = all.iter().map(|f| f.path.clone()).collect();
        assert!(!paths.contains(&core_path), "canonical core source should be skipped from ordinary scan");
        assert!(paths.contains(&other_path), "other.ph should be discovered");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scanner_bounds_wide_directory_entries_and_finishes_later() {
        let root = tmpdir("wide_directory");
        let file_count = 2_048;
        for index in 0..file_count {
            fs::write(root.join(format!("file_{index:04}.ph")), "").unwrap();
        }

        let mut scanner = WorkspaceScanState::new(AnalysisMode::Local, ExcludeMatcher::new(&[]));
        scanner.set_roots(vec![root.clone()], None);
        let budget = ScanBudget {
            max_dirs_started: 1,
            max_entries: 16,
            max_files: 8,
        };

        let first = scanner.step(budget);
        assert_eq!(first.len(), budget.max_files);
        assert!(first.len() + scanner.pending_files.len() <= budget.max_entries);
        assert!(scanner.has_work(), "wide directory must remain resumable");

        let mut discovered = first;
        let mut steps = 0;
        while scanner.has_work() {
            discovered.extend(scanner.step(budget));
            steps += 1;
            assert!(steps <= file_count, "scanner failed to make progress");
        }

        let paths: std::collections::BTreeSet<_> = discovered.into_iter().map(|file| file.path).collect();
        assert_eq!(paths.len(), file_count);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn analysis_mode_from_str() {
        assert_eq!("local".parse::<AnalysisMode>(), Ok(AnalysisMode::Local));
        assert_eq!("workspace".parse::<AnalysisMode>(), Ok(AnalysisMode::Workspace));
        assert!("unknown".parse::<AnalysisMode>().is_err());
        assert_eq!("  workspace  ".parse::<AnalysisMode>(), Ok(AnalysisMode::Workspace));
    }

    #[test]
    fn exclude_matcher_builtin_rules() {
        let matcher = ExcludeMatcher::new(&[]);
        assert!(matcher.is_excluded(Path::new("target")));
        assert!(matcher.is_excluded(Path::new(".hidden")));
        assert!(matcher.is_excluded(Path::new("node_modules")));
        assert!(!matcher.is_excluded(Path::new("src")));
    }

    #[test]
    fn exclude_matcher_user_rules() {
        let matcher = ExcludeMatcher::new(&["fixtures".to_string(), "generated".to_string()]);
        assert!(matcher.is_excluded(Path::new("tests/fixtures")));
        assert!(matcher.is_excluded(Path::new("generated/output.ph")));
        assert!(!matcher.is_excluded(Path::new("src/main.ph")));
    }

    #[test]
    fn exclude_matcher_accepts_common_glob_fragments() {
        let matcher = ExcludeMatcher::new(&["**/generated/**".to_string(), "**/*.gen.ph".to_string()]);
        assert!(matcher.is_excluded(Path::new("src/generated/output.ph")));
        assert!(matcher.is_excluded(Path::new("src/output.gen.ph")));
    }
}
