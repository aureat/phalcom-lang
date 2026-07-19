//! Keyword, member, and import-path completion for the Phalcom REPL.
//!
//! [`PhalcomCompleter`] implements the [`reedline::Completer`] and
//! [`reedline::Hinter`] traits.  It classifies the cursor position as an
//! import-path context, a member-access context, or a plain identifier
//! context, then produces appropriate [`Suggestion`]s.

use crate::common::KEYWORDS;
use reedline::{Completer, Hinter, History, Span, Suggestion};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use unicode_segmentation::UnicodeSegmentation;

/// Tab-completer and inline-hint provider for the Phalcom REPL.
///
/// Completion candidates are grouped into three context kinds:
/// - **ImportPath** — triggered by `import "<prefix>`, suggests `.ph` file and
///   module directory names relative to the working directory.
/// - **MemberAccess** — triggered by `receiver.`, suggests known members of
///   the inferred receiver type.
/// - **Identifier** — fallback context, suggests keywords, built-in classes,
///   and visible variable names.
pub struct PhalcomCompleter {
    /// The working directory used as the base for import-path suggestions.
    pub cwd: PathBuf,
}

impl PhalcomCompleter {
    /// Creates a new [`PhalcomCompleter`] rooted at the given working directory.
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    fn hint(&mut self, line: &str, pos: usize) -> Option<String> {
        let (kind, _start, prefix) = classify_position(line, pos);

        let mut items: Vec<String> = match kind {
            Cx::ImportPath => suggest_import_paths(&self.cwd, prefix),
            Cx::MemberAccess { receiver_type } => collect_member_candidates_for_type(receiver_type).into_iter().collect(),
            Cx::Identifier => collect_visible_symbols().into_iter().collect(),
        };

        items.sort();
        let next = items.into_iter().find(|s| s.starts_with(prefix))?;
        // show only the completion tail
        Some(next[prefix.len()..].to_string())
    }
}

impl Completer for PhalcomCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let (kind, start, prefix) = classify_position(line, pos);

        let mut items: Vec<String> = match kind {
            Cx::ImportPath => suggest_import_paths(&self.cwd, prefix).into_iter().collect(),
            Cx::MemberAccess { receiver_type } => collect_member_candidates_for_type(receiver_type).into_iter().collect(),
            Cx::Identifier => collect_visible_symbols().into_iter().collect(),
        };

        items.retain(|s| s.starts_with(prefix));
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
            .collect::<Vec<_>>()
    }
}

impl Hinter for PhalcomCompleter {
    fn handle(&mut self, line: &str, pos: usize, _history: &dyn History, _use_ansi_coloring: bool, _cwd: &str) -> String {
        // The current text with any active hint applied
        if let Some(hint) = self.hint(line, pos) {
            format!("{}{}", line, hint)
        } else {
            line.to_string()
        }
    }

    fn complete_hint(&self) -> String {
        // Apply the entire hint when user accepts it
        "\t".to_string()
    }

    fn next_hint_token(&self) -> String {
        // Move to next token in multi-token hints
        " ".to_string()
    }
}

// Simple context classifier
#[derive(Debug)]
enum Cx<'a> {
    ImportPath,
    MemberAccess { receiver_type: &'a str },
    Identifier,
}

/// Returns the built-in language keywords accepted by the Phalcom lexer.
///
/// Duplicates [`super::common::KEYWORDS`] as a convenience for callers that
/// import only `completer`.  Will be wired up to a live symbol table once
/// the VM exposes one.
#[allow(dead_code)] // public API; not yet called from the active binary path
pub fn builtin_keywords() -> &'static [&'static str] {
    &[
        "class", "import", "for", "while", "if", "else", "return", "break", "continue", "true", "false", "nil", "let", "const", "and", "or", "not", "self",
        "super", "in", "is", "as",
    ]
}

/// Returns a static set of placeholder variable names (demo only).
///
/// Replace with symbol-table lookups once the VM exposes them.
pub fn variables() -> &'static [&'static str] {
    &["hello", "world", "foo", "bar", "baz", "qux", "x", "y", "z"]
}

/// Returns the built-in class names available at the Phalcom top level.
pub fn classes() -> &'static [&'static str] {
    &["String", "List", "Map", "Number", "Function", "Module", "Class", "Instance"]
}

/// Collects all identifiers visible at module/global scope.
///
/// Currently combines keywords, placeholder variables, and built-in class
/// names.  Extend this once the VM provides a live symbol table.
pub fn collect_visible_symbols() -> BTreeSet<String> {
    let mut set = BTreeSet::new();

    // 1) builtins
    for k in KEYWORDS {
        set.insert((*k).to_string());
    }

    for k in variables() {
        set.insert((*k).to_string());
    }

    for k in classes() {
        set.insert((*k).to_string());
    }

    set
}

/// Returns member-completion candidates for a given class name.
///
/// Starts with a small, statically-known member set for the core types
/// (`String`, `List`, `Map`, `Number`).  All types receive `name`, `class`,
/// and `toString` from `Object`.
///
/// Replace with live class-method lookups once the VM exposes class metadata.
pub fn collect_member_candidates_for_type(type_name: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    set.extend(["name", "class", "toString"].map(String::from));
    match type_name {
        "String" => {
            set.extend(["length", "slice(_,_)", "contains(_)", "toNumber()"].map(String::from));
        }
        "List" => {
            set.extend(["count", "add(_)", "removeAt(_)", "map(_)", "filter(_)"].map(String::from));
        }
        "Map" => {
            set.extend(["count", "has(_)", "get(_)", "set(_,_)", "remove(_)"].map(String::from));
        }
        "Number" => {
            set.extend(["abs()", "floor()", "ceil()", "toString()"].map(String::from));
        }
        _ => {}
    }
    set
}

fn classify_position<'a>(line: &'a str, pos: usize) -> (Cx<'a>, usize, &'a str) {
    let upto = &line[..pos];

    // import "..."
    if upto.trim_start().starts_with("import ") {
        if let Some(qidx) = upto.rfind('"') {
            let prefix = &upto[qidx + 1..];
            return (Cx::ImportPath, qidx + 1, prefix);
        }
    }

    // member access: foo.<prefix>
    if let Some(dot) = upto.rfind('.') {
        // naive: get identifier left of dot; use it to guess type
        let left = upto[..dot].trim_end();
        let receiver_name = left.split_whitespace().last().unwrap_or("");
        let ty = guess_type_from_name(receiver_name); // heuristic; see below
        let prefix = &upto[dot + 1..];
        return (Cx::MemberAccess { receiver_type: ty }, dot + 1, prefix);
    }

    // default identifier
    let start = upto
        .grapheme_indices(true)
        .rev()
        .find_map(|(i, ch)| if is_ident_break(ch) { Some(i + ch.len()) } else { None })
        .unwrap_or(0);
    let prefix = &upto[start..];
    (Cx::Identifier, start, prefix)
}

fn is_ident_break(ch: &str) -> bool {
    // break on whitespace and punctuation
    ch.chars().any(|c| !(c.is_alphanumeric() || c == '_' || c == '/'))
}

// Dumb heuristic for demo: infer type by suffix/prefix.
// Replace with your symbol table lookups later.
fn guess_type_from_name(name: &str) -> &str {
    if name.ends_with("s") {
        "List"
    } else if name.starts_with("is_") {
        "Bool"
    } else if name.contains("map") {
        "Map"
    } else if name.contains("str") {
        "String"
    } else if name.contains("num") || name.contains("count") {
        "Number"
    } else {
        "Object"
    }
}

/// Suggests `.ph` module paths and subdirectory names matching a typed prefix.
///
/// Walks the filesystem starting from `base_dir`, returning paths relative to
/// `base_dir` using forward slashes.  Directory entries end with `/`;
/// `.ph` files have their extension stripped.
pub fn suggest_import_paths(base_dir: &Path, typed: &str) -> Vec<String> {
    // typed like "co" → suggest "core", "core/models", etc.
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
                // module directories
                // build suggestion relative to base_dir with forward slashes
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
