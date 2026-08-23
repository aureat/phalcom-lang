//! Generates `tools/vsphalcom/src/generated/core-table.json`: the harvested
//! keyword + core-class selector table the vsphalcom extension's
//! autocomplete/hover legs read (U-VSPHALCOM-2, DEC-VSP-B).
//!
//! Two sources, merged:
//! 1. Canonical universe `.ph` modules, parsed with `phalcom_ast::parse_source`
//!    (an actual parse, not a regex over Phalcom syntax).
//! 2. `src/universe/primitives.rs` — native primitive registrations, read as
//!    text and manually scanned for `primitive!`/`primitive_static!` macro
//!    calls (no `regex` dependency; the macro-call shape is uniform enough
//!    for a small hand-written scanner).
//!
//! Every selector is emitted in the **comma-form symbol-literal** spelling
//! (`lexical-structure.md` §10 / ADR-0012), e.g. `move(_,to,duration)` — a
//! positional parameter renders as `_`, a keyword parameter renders as its
//! label. This is deliberately **not** the VM-internal colon-encoded string
//! `encode_selector` produces (`move(to:duration:)`, no commas, no `_`) —
//! that is a different, internal representation of the same selector; the
//! comma-form is what a user writes in a `#move(_,to,duration)` symbol
//! literal and what Phaldoc/hover should show.
//!
//! Run manually (not part of the build): from the repo root,
//! `cargo run -p phalcom-core --bin gen-core-table -- tools/vsphalcom/src/generated/core-table.json`.
//! Output is deterministic (sorted class/selector order) so a re-run on an
//! unchanged tree produces a byte-identical file.

use phalcom_ast::ast::{AttrKind, BuiltinAttr, ClassMember, IndexAccessor, Statement};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// One harvested selector entry for a class.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SelectorEntry {
    /// Comma-form selector spelling, e.g. `move(_,to,duration)`.
    selector: String,
    /// `"method"`, `"getter"`, `"setter"`, or `"construct"`.
    kind: &'static str,
    /// Whether dispatch targets the class object rather than an instance.
    class_side: bool,
    /// `"public"`, `"private"`, `"protected"`, or `"internal"`.
    visibility: &'static str,
    /// `"universe"` or `"native"`.
    source: &'static str,
}

/// The current keyword set (matches the U-VSPHALCOM-1 grammar rewrite;
/// keep in sync with `docs/spec/v0.2/lexical-structure.md`).
const KEYWORDS: &[&str] = &[
    "let",
    "const",
    "fn",
    "class",
    "is",
    "super",
    "self",
    "static",
    "try",
    "catch",
    "on",
    "ensure",
    "throw",
    "break",
    "continue",
    "match",
    "return",
    "while",
    "for",
    "in",
    "import",
    "as",
    "if",
    "else",
    "true",
    "false",
    "and",
    "or",
    "not",
    "construct",
];

fn main() {
    let out_path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: gen-core-table <output-path.json>");
        std::process::exit(2);
    });

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let primitives_rs_path = manifest_dir.join("src/universe/primitives.rs");

    let mut classes: BTreeMap<String, Vec<SelectorEntry>> = BTreeMap::new();

    harvest_universe_sources(&mut classes);
    harvest_primitives_rs(&primitives_rs_path, &mut classes);

    for entries in classes.values_mut() {
        entries.sort();
        entries.dedup();
    }

    assert_no_legacy_names(&classes);

    let json = render_json(&classes);
    fs::write(&out_path, json).unwrap_or_else(|e| panic!("failed to write {out_path}: {e}"));
    eprintln!("wrote {} classes to {out_path}", classes.len());
}

/// Parses every canonical universe module and harvests every class's declared selectors in
/// comma-form.
fn harvest_universe_sources(classes: &mut BTreeMap<String, Vec<SelectorEntry>>) {
    let provider = phalcom_modules::builtin::BuiltinProjectSourceProvider::new(phalcom_modules::identity::BuiltinProject::Universe);
    for node in provider.nodes() {
        let path = phalcom_modules::identity::ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("valid builtin component"))
                .collect::<Vec<_>>(),
        );
        let module = phalcom_modules::identity::ModuleId::builtin(phalcom_modules::identity::BuiltinProject::Universe, path);
        let source = provider.source_text(&module).unwrap_or_else(|e| panic!("failed to read {module}: {e}"));
        let program = phalcom_ast::parse_source(&source, 0).unwrap_or_else(|e| panic!("failed to parse {module}: {e:?}"));

        for stmt in &program.statements {
            let Statement::Class(class_def) = stmt else { continue };
            let entries = classes.entry(class_def.name.clone()).or_default();

            for member in &class_def.members {
                let class_side = member_is_class_side(member);
                let visibility = member_visibility(member);
                match member {
                    ClassMember::Method(m) => {
                        entries.push(SelectorEntry {
                            selector: comma_form(&m.name, &m.params.iter().map(|p| p.label.clone()).collect::<Vec<_>>()),
                            kind: if m.is_constructor || m.attributes.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Constructor))) {
                                "construct"
                            } else {
                                "method"
                            },
                            class_side,
                            visibility,
                            source: "universe",
                        });
                    }
                    ClassMember::Getter(g) => {
                        entries.push(SelectorEntry {
                            selector: g.name.clone(),
                            kind: "getter",
                            class_side,
                            visibility,
                            source: "universe",
                        });
                    }
                    ClassMember::Setter(s) => {
                        entries.push(SelectorEntry {
                            selector: format!("{}=(put)", s.name),
                            kind: "setter",
                            class_side,
                            visibility,
                            source: "universe",
                        });
                    }
                    // A declared field (U-ANNOT-LAYOUT §3.1) has no selector of
                    // its own to harvest — it is not a dispatchable member.
                    ClassMember::Field(_) => {}
                    // A `@variant` arm (U-ANNOT-LAYOUT §3.4) is expanded away at
                    // compile time into a sibling top-level class this raw-AST
                    // harvest never sees — nothing to harvest here either.
                    ClassMember::Variant(_) => {}
                    // A bracket subscript method (U-INDEX, ADR-0060: `[idx] {
                    // ... }` / `[idx, put:] { ... }`) — harvested in the same
                    // bracket-delimited, no-name comma-form spelling
                    // `phalcom-lsp`'s `selectors::index_selector` uses (`[_]`,
                    // `[_,put]`, `[]`, `[put]`).
                    ClassMember::Index(ix) => {
                        entries.push(SelectorEntry {
                            selector: bracket_form(
                                &ix.params.iter().map(|p| p.label.clone()).collect::<Vec<_>>(),
                                matches!(&ix.accessor, IndexAccessor::Set { .. }),
                            ),
                            kind: "method",
                            class_side,
                            visibility,
                            source: "universe",
                        });
                    }
                }
            }
        }
    }
}

fn member_is_class_side(member: &ClassMember) -> bool {
    let (intrinsic, attrs) = match member {
        ClassMember::Method(m) => (m.is_static || m.is_constructor, m.attributes.as_slice()),
        ClassMember::Getter(g) => (g.is_static, g.attributes.as_slice()),
        ClassMember::Setter(s) => (s.is_static, s.attributes.as_slice()),
        ClassMember::Index(ix) => (false, ix.attributes.as_slice()),
        ClassMember::Field(f) => (f.is_static, f.attributes.as_slice()),
        ClassMember::Variant(v) => (false, v.attributes.as_slice()),
    };
    intrinsic
        || attrs
            .iter()
            .any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Class | BuiltinAttr::Constructor)))
}

fn member_visibility(member: &ClassMember) -> &'static str {
    let (name, attrs) = match member {
        ClassMember::Method(m) => (Some(m.name.as_str()), m.attributes.as_slice()),
        ClassMember::Getter(g) => (Some(g.name.as_str()), g.attributes.as_slice()),
        ClassMember::Setter(s) => (Some(s.name.as_str()), s.attributes.as_slice()),
        ClassMember::Index(ix) => (None, ix.attributes.as_slice()),
        ClassMember::Field(f) => (Some(f.name.as_str()), f.attributes.as_slice()),
        ClassMember::Variant(v) => (Some(v.name.as_str()), v.attributes.as_slice()),
    };
    if name.is_some_and(|name| name.starts_with("_$")) {
        "internal"
    } else if attrs.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Private))) {
        "private"
    } else if attrs.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Protected))) {
        "protected"
    } else {
        "public"
    }
}

/// Builds the bracket-form selector string for a subscript method (U-INDEX,
/// ADR-0060): `[_]`, `[_,put]`, `[]`, `[put]`, ... . Mirrors [`comma_form`]'s
/// label-joining, just bracket- rather than paren-delimited and with no
/// leading name (a bracket method carries no name token at all).
fn bracket_form(labels: &[Option<String>], setter: bool) -> String {
    let inner = labels.iter().map(|l| l.as_deref().unwrap_or("_")).collect::<Vec<_>>().join(",");
    let base = format!("[{inner}]");
    if setter { format!("{base}=(put)") } else { base }
}

/// Builds the comma-form selector string: `name(_,label,...)`, or `name()`
/// for a zero-arity form. A positional parameter (no label) renders as `_`;
/// a keyword parameter renders as its label.
fn comma_form(name: &str, labels: &[Option<String>]) -> String {
    if labels.is_empty() {
        return format!("{name}()");
    }
    let inner = labels.iter().map(|l| l.as_deref().unwrap_or("_")).collect::<Vec<_>>().join(",");
    format!("{name}({inner})")
}

/// Scans `primitives.rs` as text for `primitive!`/`primitive_static!` macro
/// calls and the `let <var> = vm.universe.classes.<name>_class;` bindings
/// that name their receiver class, without a `regex` dependency — the macro
/// call shape is uniform enough for a small hand-written scanner.
fn harvest_primitives_rs(path: &PathBuf, classes: &mut BTreeMap<String, Vec<SelectorEntry>>) {
    let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    // Pass 1: `let object_cls = vm.universe.classes.object_class;` -> var -> "Object"
    let mut var_to_class: BTreeMap<String, String> = BTreeMap::new();
    for line in source.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("let ") else { continue };
        let Some((var, rhs)) = rest.split_once('=') else { continue };
        let rhs = rhs.trim().trim_end_matches(';').trim();
        let Some(class_field) = rhs.strip_prefix("vm.universe.classes.") else {
            continue;
        };
        let Some(snake_name) = class_field.strip_suffix("_class") else { continue };
        var_to_class.insert(var.trim().to_string(), capitalize(snake_name));
    }

    // Pass 2: every `primitive!(...)` / `primitive_static!(...)` call, which
    // may span multiple lines, so scan the whole source for each macro
    // invocation by locating `macro_name!(` and matching the closing `)`.
    for macro_name in ["primitive!", "primitive_internal!", "primitive_static!", "primitive_static_internal!"] {
        let mut search_from = 0;
        while let Some(start) = source[search_from..].find(macro_name) {
            let abs_start = search_from + start + macro_name.len();
            let Some(open) = source[abs_start..].find('(') else { break };
            let open = abs_start + open;
            let Some(close) = find_matching_paren(&source, open) else { break };
            let call_body = &source[open + 1..close];
            search_from = close + 1;

            let args = split_top_level_commas(call_body);
            // Expected: vm, <class_var>, "<selector-name>", SignatureKind::Kind[(N)], <fn>
            if args.len() < 4 {
                continue;
            }
            let class_var = args[1].trim();
            let Some(class_name) = var_to_class.get(class_var) else {
                eprintln!("gen-core-table: warning: no class binding found for var `{class_var}` in {macro_name} call, skipping");
                continue;
            };
            let selector_name = args[2].trim().trim_matches('"');
            let kind_expr = args[3].trim();

            let (selector, base_kind) = comma_form_from_signature_kind(selector_name, kind_expr);
            let class_side = macro_name.contains("static");
            let visibility = if macro_name.contains("internal") { "internal" } else { "public" };
            let entries = classes.entry(class_name.clone()).or_default();
            entries.push(SelectorEntry {
                selector,
                kind: base_kind,
                class_side,
                visibility,
                source: "native",
            });
        }
    }
}

/// Given a selector's bare name and its `SignatureKind::Method[(n)]` source
/// expression, builds the comma-form spelling. Native primitives carry no
/// parameter labels in `primitives.rs` — only arity — so a `Method(n)`
/// entry renders as `name(_,_,...)` (n placeholders), matching the
/// unlabelled positional convention; a real labelled native primitive is
/// rare enough on the current floor that this is an acceptable
/// approximation (see DEC-VSP-C's sibling note on grammar approximation).
fn comma_form_from_signature_kind(name: &str, kind_expr: &str) -> (String, &'static str) {
    if kind_expr.starts_with("SignatureKind::Getter") {
        return (name.to_string(), "getter");
    }
    if kind_expr.starts_with("SignatureKind::Setter") {
        return (format!("{name}=(put)"), "setter");
    }
    if let Some(n) = extract_arity(kind_expr, "SignatureKind::Method") {
        return (positional_comma_form(name, n), "method");
    }
    if kind_expr.starts_with("SignatureKind::SubscriptGet") {
        return ("[]".to_string(), "method");
    }
    if kind_expr.starts_with("SignatureKind::SubscriptSet") {
        return ("[]=(put)".to_string(), "method");
    }
    // Unrecognized kind expression (future SignatureKind variant): fall back
    // to the bare name rather than panic — a codegen tool must not crash on
    // a floor it doesn't yet know about.
    (name.to_string(), "method")
}

/// Extracts `n` from a `Prefix(n)` expression, or `None` if `kind_expr`
/// doesn't start with `prefix`.
fn extract_arity(kind_expr: &str, prefix: &str) -> Option<u32> {
    let rest = kind_expr.strip_prefix(prefix)?;
    let rest = rest.trim().strip_prefix('(')?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Builds `name(_,_,...)` with `n` positional placeholders (0 -> `name()`).
fn positional_comma_form(name: &str, n: u32) -> String {
    if n == 0 {
        return format!("{name}()");
    }
    let inner = vec!["_"; n as usize].join(",");
    format!("{name}({inner})")
}

/// Finds the index of the `)` matching the `(` at `open_idx`, accounting for
/// nested parens and string literals (so a `")"` inside a selector-name
/// string literal, e.g. `"[]"`, doesn't confuse the scan).
fn find_matching_paren(source: &str, open_idx: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut i = open_idx;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_string {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => in_string = true,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Splits `s` on top-level commas (depth-0, outside string literals) —
/// `SignatureKind::Method(1)`'s internal comma-free arity paren nests safely
/// since it never contains a comma; this still guards against any future
/// kind that does.
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut current = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            current.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                current.push(c);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

/// Capitalizes a snake_case class-var fragment's first letter, e.g.
/// `object` -> `Object`. Every current core class name is a single word, so
/// this is not a general snake_case->PascalCase converter.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Asserts none of the harvested class names are stale 2023-era names the
/// spec has since deleted (`Null`, `Void`, `ObjectType`) — the determinism
/// gate `docs/forge/units/U-VSPHALCOM/plan.md`'s "Tests / verification"
/// section names for the codegen leg.
fn assert_no_legacy_names(classes: &BTreeMap<String, Vec<SelectorEntry>>) {
    for legacy in ["Null", "Void", "ObjectType", "NullType", "VoidType"] {
        assert!(!classes.contains_key(legacy), "harvested table contains a legacy 2023-era class name: {legacy}");
    }
    for entries in classes.values() {
        for entry in entries {
            if let Some(eq) = entry.selector.find("=(_)") {
                let preceding = entry.selector[..eq].chars().last();
                assert!(
                    !preceding.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ']'),
                    "legacy setter selector: {}",
                    entry.selector
                );
            }
            assert!(!entry.selector.starts_with("__"), "legacy implementation selector: {}", entry.selector);
            if let Some(close) = entry.selector.find(']') {
                assert!(!entry.selector[..close].contains("put"), "legacy subscript put slot: {}", entry.selector);
            }
            let base = entry.selector.split(['(', '[', '=']).next().unwrap_or("");
            assert!(!base.ends_with('_'), "legacy trailing-underscore selector: {}", entry.selector);
        }
    }
}

/// Hand-rolled JSON serialization (no `serde_json` dependency) — the output
/// shape is small and fixed, so a manual writer with string escaping is
/// simpler than adding a new crate dependency for one codegen tool.
fn render_json(classes: &BTreeMap<String, Vec<SelectorEntry>>) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"keywords\": [\n");
    for (i, kw) in KEYWORDS.iter().enumerate() {
        out.push_str("    ");
        out.push_str(&json_escape(kw));
        if i + 1 < KEYWORDS.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    out.push_str("  \"classes\": {\n");

    let class_count = classes.len();
    for (ci, (class_name, entries)) in classes.iter().enumerate() {
        out.push_str("    ");
        out.push_str(&json_escape(class_name));
        out.push_str(": [\n");
        for (ei, entry) in entries.iter().enumerate() {
            out.push_str("      {\"selector\": ");
            out.push_str(&json_escape(&entry.selector));
            out.push_str(", \"kind\": ");
            out.push_str(&json_escape(entry.kind));
            out.push_str(", \"source\": ");
            out.push_str(&json_escape(entry.source));
            out.push_str(", \"classSide\": ");
            out.push_str(if entry.class_side { "true" } else { "false" });
            out.push_str(", \"visibility\": ");
            out.push_str(&json_escape(entry.visibility));
            out.push('}');
            if ei + 1 < entries.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("    ]");
        if ci + 1 < class_count {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}

/// Escapes a string as a JSON string literal (quotes included).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
