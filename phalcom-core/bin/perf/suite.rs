use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteManifest {
    pub schema_version: u32,
    pub name: String,
    pub cases: Vec<CaseManifestSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseManifestSpec {
    pub id: String,
    pub path: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub heavy: bool,
    #[serde(default = "default_samples")]
    pub default_samples: usize,
    #[serde(default = "default_warmup")]
    pub default_warmup: usize,
    pub verification: ManifestVerification,
}

fn default_samples() -> usize {
    5
}
fn default_warmup() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ManifestVerification {
    StdoutExact { expected: String },
    ExitZeroOnly,
}

#[derive(Debug, Clone)]
pub enum CaseVerification {
    StdoutExact { expected: String },
    SidecarExpected,
    NegativeDiagnostic { substring: String },
    ExitZeroOnly,
}

#[derive(Debug, Clone)]
pub struct CaseSpec {
    pub id: String,
    pub tag: String,
    pub path: PathBuf,
    pub heavy: bool,
    pub samples: usize,
    pub warmup: usize,
    pub verification: CaseVerification,
}

pub fn load_suite(workspace_root: &Path, suite_name: &str) -> Result<(SuiteManifest, Vec<CaseSpec>), String> {
    let manifest_path = workspace_root.join("benchmarks/suites").join(format!("{suite_name}.json"));
    if !manifest_path.exists() {
        return Err(format!("suite manifest not found at {}", manifest_path.display()));
    }
    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    parse_suite_manifest(workspace_root, &content)
}

pub fn parse_suite_manifest(workspace_root: &Path, json_str: &str) -> Result<(SuiteManifest, Vec<CaseSpec>), String> {
    let manifest: SuiteManifest = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
    if manifest.schema_version != 1 {
        return Err(format!("unsupported suite schema version {}", manifest.schema_version));
    }

    let mut seen_ids = HashSet::new();
    let mut specs = Vec::new();

    for case in &manifest.cases {
        if !seen_ids.insert(&case.id) {
            return Err(format!("duplicate case id '{}' in suite manifest", case.id));
        }
        let full_path = workspace_root.join(&case.path);
        let tag = case.tags.first().cloned().unwrap_or_else(|| "default".into());
        let verification = match &case.verification {
            ManifestVerification::StdoutExact { expected } => CaseVerification::StdoutExact { expected: expected.clone() },
            ManifestVerification::ExitZeroOnly => CaseVerification::ExitZeroOnly,
        };

        specs.push(CaseSpec {
            id: case.id.clone(),
            tag,
            path: full_path,
            heavy: case.heavy,
            samples: case.default_samples,
            warmup: case.default_warmup,
            verification,
        });
    }

    Ok((manifest, specs))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum Lane {
    Pass,
    Negative,
    Pending,
    Bench,
}

pub fn collect_corpus_cases(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_ph_files(dir, &mut out);
    out.retain(|p| p.with_extension("expected").exists());
    out.sort();
    out
}

pub fn collect_bench_cases(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_ph_files(dir, &mut out);
    out.sort();
    out
}

fn walk_ph_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_ph_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ph") {
            out.push(path);
        }
    }
}

pub fn classify_lane(path: &Path, corpus_root: &Path) -> Lane {
    let rel = path.strip_prefix(corpus_root).unwrap_or(path);
    let components: Vec<&str> = rel.components().filter_map(|c| c.as_os_str().to_str()).collect();

    if components.contains(&"pending") {
        return Lane::Pending;
    }
    if components.contains(&"negative") {
        return Lane::Negative;
    }
    match components.first() {
        Some(&("runtime-errors" | "syntax-errors" | "compile-errors")) => Lane::Negative,
        _ => Lane::Pass,
    }
}

pub fn label_of(path: &Path, corpus_root: &Path) -> String {
    path.strip_prefix(corpus_root)
        .ok()
        .and_then(|rel| rel.components().next())
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("?")
        .to_string()
}

pub fn case_name(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_manifest() {
        let json = r#"{
            "schema_version": 1,
            "name": "test_suite",
            "cases": [
                {
                    "id": "c1",
                    "path": "benchmarks/c1.ph",
                    "tags": ["quick"],
                    "heavy": false,
                    "default_samples": 3,
                    "default_warmup": 1,
                    "verification": { "kind": "exit_zero_only" }
                }
            ]
        }"#;
        let (manifest, specs) = parse_suite_manifest(Path::new("/workspace"), json).unwrap();
        assert_eq!(manifest.name, "test_suite");
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "c1");
    }

    #[test]
    fn reject_duplicate_ids() {
        let json = r#"{
            "schema_version": 1,
            "name": "test_suite",
            "cases": [
                {
                    "id": "c1",
                    "path": "benchmarks/c1.ph",
                    "tags": [],
                    "verification": { "kind": "exit_zero_only" }
                },
                {
                    "id": "c1",
                    "path": "benchmarks/c2.ph",
                    "tags": [],
                    "verification": { "kind": "exit_zero_only" }
                }
            ]
        }"#;
        let res = parse_suite_manifest(Path::new("/workspace"), json);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("duplicate case id 'c1'"));
    }
}
