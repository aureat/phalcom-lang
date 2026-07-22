//! Snapshot-backed completer and inline hinter for the Phalcom REPL.
//!
//! Implements [`reedline::Completer`] and [`reedline::Hinter`] using `ReplOracle`
//! for live runtime symbol table and inheritance chain lookups.

use crate::oracle::ReplOracle;
use crate::snapshot::MemberKind;
use reedline::{Completer, Hinter, History, Span, Suggestion};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

/// Tab-completer and inline-hint provider backed by [`ReplOracle`].
pub struct PhalcomCompleter {
    pub cwd: PathBuf,
    oracle: Arc<ReplOracle>,
}

impl PhalcomCompleter {
    /// Creates a new [`PhalcomCompleter`].
    pub fn new(cwd: PathBuf, oracle: Arc<ReplOracle>) -> Self {
        Self { cwd, oracle }
    }

    fn hint(&mut self, line: &str, pos: usize) -> Option<String> {
        let (kind, _start, prefix) = classify_position(line, pos);

        let items = match kind {
            Cx::ImportPath => suggest_import_paths(&self.cwd, prefix),
            Cx::MemberAccess { receiver } => self.collect_members_for_receiver(receiver, prefix),
            Cx::Identifier => self.collect_identifiers(prefix),
        };

        let next = items.into_iter().find(|s| s.starts_with(prefix))?;
        Some(next[prefix.len()..].to_string())
    }

    fn collect_members_for_receiver(&self, receiver: &str, prefix: &str) -> Vec<String> {
        let class_id = self.oracle.global_name_class(receiver).or_else(|| self.oracle.find_class_by_name(receiver));

        let Some(cid) = class_id else {
            return Vec::new();
        };

        let members = self.oracle.members_for_class(cid);
        let mut candidates = Vec::new();

        for m in members {
            if !m.name.starts_with(prefix) {
                continue;
            }
            let formatted = if m.kind == MemberKind::Getter || m.arity == 0 {
                m.name.clone()
            } else {
                format!("{}(", m.name)
            };
            candidates.push(formatted);
        }

        candidates
    }

    fn collect_identifiers(&self, prefix: &str) -> Vec<String> {
        let mut set = BTreeSet::new();

        for kw in BUILTIN_KEYWORDS {
            if kw.starts_with(prefix) {
                set.insert((*kw).to_string());
            }
        }

        let snap = self.oracle.snapshot();
        for name in snap.global_names.keys().chain(snap.class_names.keys()) {
            if name.starts_with(prefix) {
                set.insert(name.clone());
            }
        }

        for cls in BUILTIN_CLASSES {
            if cls.starts_with(prefix) {
                set.insert((*cls).to_string());
            }
        }

        set.into_iter().collect()
    }
}

impl Completer for PhalcomCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let (kind, start, prefix) = classify_position(line, pos);

        let items = match kind {
            Cx::ImportPath => suggest_import_paths(&self.cwd, prefix),
            Cx::MemberAccess { receiver } => self.collect_members_for_receiver(receiver, prefix),
            Cx::Identifier => self.collect_identifiers(prefix),
        };

        items
            .into_iter()
            .map(|value| Suggestion {
                value: value.clone(),
                description: None,
                style: None,
                extra: None,
                span: Span::new(start, pos),
                append_whitespace: false,
            })
            .collect()
    }
}

impl Hinter for PhalcomCompleter {
    fn handle(&mut self, line: &str, pos: usize, _history: &dyn History, _use_ansi_coloring: bool, _cwd: &str) -> String {
        if let Some(hint) = self.hint(line, pos) {
            format!("{line}{hint}")
        } else {
            line.to_string()
        }
    }

    fn complete_hint(&self) -> String {
        "\t".to_string()
    }

    fn next_hint_token(&self) -> String {
        " ".to_string()
    }
}

#[derive(Debug)]
enum Cx<'a> {
    ImportPath,
    MemberAccess { receiver: &'a str },
    Identifier,
}

const BUILTIN_KEYWORDS: &[&str] = &[
    "class",
    "import",
    "for",
    "while",
    "if",
    "else",
    "return",
    "break",
    "continue",
    "true",
    "false",
    "nil",
    "let",
    "const",
    "and",
    "or",
    "not",
    "self",
    "super",
    "in",
    "is",
    "as",
    "static",
    "construct",
    "throw",
    "try",
    "fn",
];

const BUILTIN_CLASSES: &[&str] = &[
    "String", "List", "Map", "Number", "Function", "Module", "Class", "Instance", "Bool", "True", "False", "Nil", "Symbol", "Fiber", "Future",
];

fn classify_position<'a>(line: &'a str, pos: usize) -> (Cx<'a>, usize, &'a str) {
    let upto = &line[..pos];

    if upto.trim_start().starts_with("import ") {
        if let Some(qidx) = upto.rfind('"') {
            let prefix = &upto[qidx + 1..];
            return (Cx::ImportPath, qidx + 1, prefix);
        }
    }

    if let Some(dot) = upto.rfind('.') {
        let left = upto[..dot].trim_end();
        let receiver = left.split_whitespace().last().unwrap_or("");
        let prefix = &upto[dot + 1..];
        return (Cx::MemberAccess { receiver }, dot + 1, prefix);
    }

    let start = upto
        .grapheme_indices(true)
        .rev()
        .find_map(|(i, ch)| if is_ident_break(ch) { Some(i + ch.len()) } else { None })
        .unwrap_or(0);
    let prefix = &upto[start..];
    (Cx::Identifier, start, prefix)
}

fn is_ident_break(ch: &str) -> bool {
    ch.chars().any(|c| !(c.is_alphanumeric() || c == '_' || c == '/'))
}

pub fn suggest_import_paths(base_dir: &Path, typed: &str) -> Vec<String> {
    let mut base = base_dir.to_path_buf();
    let rel = Path::new(typed);
    base = base.join(rel);

    let search_dir = if typed.ends_with('/') {
        base.clone()
    } else {
        base.parent().unwrap_or(&base).to_path_buf()
    };

    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(&search_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Ok(relp) = p.strip_prefix(base_dir) {
                    let s = relp.to_string_lossy().replace('\\', "/");
                    out.push(s + "/");
                }
            } else if p.extension().and_then(|x| x.to_str()) == Some("ph") {
                if let Ok(relp) = p.strip_prefix(base_dir) {
                    let mut s = relp.to_string_lossy().replace('\\', "/");
                    if let Some(stripped) = s.strip_suffix(".ph") {
                        s = stripped.into();
                    }
                    out.push(s.to_string());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
