//! Receiver-aware completion: resolve a receiver's class, then offer its
//! selectors as snippet [`CompletionItem`]s (Stage 3).
//!
//! `docs/forge/units/U-LSP/plan.md` "Stage 3" (ADR-0056 §4). Two pieces:
//!
//! 1. A **pluggable** [`ReceiverResolver`] — given the document and the
//!    completion position, name the class of the receiver under the cursor
//!    (the `x` in `x.<cursor>`). The first and only concrete resolver here,
//!    [`ConstructResolver`], does light local dataflow: it finds the
//!    `let x = Cls.new(...)` / `Cls.construct(...)` binding and returns `Cls`.
//!    A smarter future inference pass drops in behind the same trait without
//!    reshaping the completion handler (plan P4's "must not preclude").
//! 2. A renderer, [`completions`], that turns a resolved class (or `None`,
//!    the "receiver type unknown" fallback) into snippet completion items —
//!    from the [`WorkspaceIndex`] for user classes, from
//!    [`CoreTable`](crate::core_table) for builtins, and the full builtin
//!    surface when nothing resolves (so Stage 3 is never worse than the
//!    pre-LSP `completions.ts`).
//!
//! Every selector rendered here is ADR-0012 comma-form (from
//! [`crate::selectors`] / `core-table.json`), so a method's argument slots and
//! labels are read straight off the selector spelling — no re-derivation.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position};

use phalcom_ast::ast::{Expr, Pattern, Program, Statement};

use crate::core_table::{CoreTable, MemberKind};
use crate::documents::Document;
use crate::index::{ClassMemberInfo, WorkspaceIndex};

/// Names the class of the receiver expression under a completion position.
///
/// The pluggable seam of Stage 3: the completion handler depends only on this
/// trait, so a future type-inference pass can replace the local-dataflow
/// [`ConstructResolver`] without any change to
/// [`completions`] or the backend handler (plan P4).
pub trait ReceiverResolver {
    /// Returns the class name of the receiver at `position` in `doc`, or
    /// `None` if there is no receiver, or its type cannot be determined.
    ///
    /// A `None` result degrades completion to the full builtin surface — it
    /// never suppresses completions entirely.
    fn resolve(&self, doc: &Document, position: Position) -> Option<String>;
}

/// A local-dataflow [`ReceiverResolver`]: resolves `x`'s class from its
/// `let x = Cls.new(...)` / `Cls.construct(...)` binding within the document.
///
/// Deliberately minimal (the plan's "first impl"): it reads the receiver
/// identifier textually from just left of the cursor, then walks the parsed
/// program for a `let`/`var` binding of that name whose initializer is a
/// message send to a capitalized `Var` receiver (the class-name convention),
/// and returns that class name. The *last* such binding in document order
/// wins (a later rebind shadows an earlier one). It does not follow
/// assignments, method parameters, or cross-file bindings — those are left to
/// a future resolver behind the same trait.
pub struct ConstructResolver;

impl ReceiverResolver for ConstructResolver {
    fn resolve(&self, doc: &Document, position: Position) -> Option<String> {
        let offset = doc.line_index.offset(position);
        let (receiver, _partial) = receiver_prefix(&doc.text, offset)?;
        resolve_var_class(&doc.parse.program, &receiver)
    }
}

/// Extracts the receiver identifier and the partially-typed member name from
/// the text immediately left of `offset`.
///
/// For `m.mov|` (cursor `|`) this returns `("m", "mov")`; for `m.|` it returns
/// `("m", "")`. Returns `None` when the cursor is not positioned after a
/// `receiver.` member access (no `.`, or no identifier before it) — the case
/// where completion falls back to the full builtin surface.
///
/// Identifiers are matched as ASCII alphanumerics and `_`, so every byte
/// scanned is a single-byte UTF-8 code unit and the returned slices are always
/// on char boundaries.
fn receiver_prefix(text: &str, offset: usize) -> Option<(String, String)> {
    let bytes = text.as_bytes();
    let end = offset.min(text.len());

    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    // Consume the partial member name being typed after the `.`.
    let mut partial_start = end;
    while partial_start > 0 && is_ident(bytes[partial_start - 1]) {
        partial_start -= 1;
    }
    let partial = &text[partial_start..end];

    // The byte before the partial must be the member-access `.`.
    if partial_start == 0 || bytes[partial_start - 1] != b'.' {
        return None;
    }
    let dot = partial_start - 1;

    // Consume the receiver identifier immediately left of the `.`.
    let mut recv_start = dot;
    while recv_start > 0 && is_ident(bytes[recv_start - 1]) {
        recv_start -= 1;
    }
    if recv_start == dot {
        return None;
    }
    Some((text[recv_start..dot].to_string(), partial.to_string()))
}

/// Walks `program` for the class bound to the variable `name` by a `construct`
/// call site, returning the class name of the last such binding.
///
/// Recognizes `let name = Cls.<ctor>(...)` (and the `var` form) anywhere in
/// the program — top level, method/getter/setter/construct bodies, block
/// bodies, `for` bodies — where `Cls` is a capitalized `Var` receiver.
fn resolve_var_class(program: &Program, name: &str) -> Option<String> {
    let mut result = None;
    scan_statements(&program.statements, name, &mut result);
    result
}

/// Recurses `statements`, updating `out` with the class of the most recent
/// `construct`-binding of `target`.
fn scan_statements(statements: &[Statement], target: &str, out: &mut Option<String>) {
    for statement in statements {
        match statement {
            Statement::Let(binding) => {
                if let Pattern::Name { name, .. } = &binding.pattern {
                    if name == target {
                        if let Some(value) = &binding.value {
                            if let Some(class) = class_of_construct(value) {
                                *out = Some(class);
                            }
                        }
                    }
                }
                if let Some(value) = &binding.value {
                    scan_expr(value, target, out);
                }
            }
            Statement::Return(r) => {
                if let Some(value) = &r.value {
                    scan_expr(value, target, out);
                }
            }
            Statement::Expr { expr, .. } => scan_expr(expr, target, out),
            Statement::For(f) => {
                scan_expr(&f.iter, target, out);
                scan_statements(&f.body, target, out);
            }
            Statement::Throw { expr, .. } => scan_expr(expr, target, out),
            Statement::Class(class_def) => {
                for member in &class_def.members {
                    match member {
                        phalcom_ast::ast::ClassMember::Method(m) => {
                            scan_statements(&m.body, target, out)
                        }
                        phalcom_ast::ast::ClassMember::Getter(g) => {
                            scan_statements(&g.body, target, out)
                        }
                        phalcom_ast::ast::ClassMember::Setter(s) => {
                            scan_statements(&s.body, target, out)
                        }
                        phalcom_ast::ast::ClassMember::Construct(c) => {
                            scan_statements(&c.body, target, out)
                        }
                        phalcom_ast::ast::ClassMember::Field(f) => {
                            if let Some(default) = &f.default {
                                scan_expr(default, target, out);
                            }
                        }
                    }
                }
            }
            Statement::Break { .. } | Statement::Continue { .. } | Statement::Import(_) => {}
        }
    }
}

/// Recurses into an expression, descending into nested block bodies (and the
/// sub-expressions that may contain them) to reach `let` bindings of `target`.
fn scan_expr(expr: &Expr, target: &str, out: &mut Option<String>) {
    match expr {
        Expr::Block(b) => scan_statements(&b.body, target, out),
        Expr::MethodCall(m) => {
            scan_expr(&m.object, target, out);
            for arg in &m.args {
                scan_expr(&arg.expr, target, out);
            }
        }
        Expr::Assignment(a) => {
            scan_expr(&a.name, target, out);
            scan_expr(&a.value, target, out);
        }
        Expr::Unary(u) => scan_expr(&u.expr, target, out),
        Expr::Binary(b) => {
            scan_expr(&b.left, target, out);
            scan_expr(&b.right, target, out);
        }
        Expr::GetProperty(g) => scan_expr(&g.object, target, out),
        Expr::SetProperty(s) => {
            scan_expr(&s.object, target, out);
            scan_expr(&s.value, target, out);
        }
        Expr::Index(i) => {
            scan_expr(&i.object, target, out);
            scan_expr(&i.index, target, out);
        }
        Expr::SetIndex(si) => {
            scan_expr(&si.object, target, out);
            scan_expr(&si.index, target, out);
            scan_expr(&si.value, target, out);
        }
        Expr::MethodRef(mr) => scan_expr(&mr.receiver, target, out),
        Expr::Number { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Var { .. }
        | Expr::Field { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::Symbol(_) => {}
    }
}

/// If `expr` is a message send to a capitalized `Var` receiver (`Cls.new(...)`,
/// `Cls.construct(...)`, `Cls.make(...)`, …), returns `Cls` — the class the
/// binding's value constructs.
///
/// Capitalization is the class-name convention Phalcom uses; requiring it
/// avoids treating an ordinary lowercase-receiver call (`counter.next()`) as a
/// construction.
fn class_of_construct(expr: &Expr) -> Option<String> {
    if let Expr::MethodCall(m) = expr {
        if let Expr::Var { value, .. } = &m.object {
            if value.chars().next().is_some_and(char::is_uppercase) {
                return Some(value.clone());
            }
        }
    }
    None
}

/// Builds the completion item list for a (possibly unresolved) receiver class.
///
/// - `Some(class)` and the class has known members: that class's own members
///   plus, walking [`WorkspaceIndex::class_parent`], its user-class ancestors'
///   members, stopping at (and including) the first builtin ancestor whose
///   members come from [`CoreTable`]. De-duplicated by selector, most-derived
///   winning.
/// - `Some(class)` but no members resolve, or `None`: the full builtin surface
///   ([`CoreTable::all_members`]) — the graceful "receiver type unknown"
///   fallback.
///
/// Items are returned sorted by label for a deterministic order.
pub fn completions(
    resolved: Option<&str>,
    index: &WorkspaceIndex,
    table: &CoreTable,
) -> Vec<CompletionItem> {
    let members = match resolved {
        Some(class) => {
            let collected = collect_class_members(class, index, table);
            if collected.is_empty() {
                table_members(table)
            } else {
                collected
            }
        }
        None => table_members(table),
    };

    let mut items: Vec<CompletionItem> = members.iter().map(to_completion_item).collect();
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

/// The full builtin surface as [`ClassMemberInfo`]s.
fn table_members(table: &CoreTable) -> Vec<ClassMemberInfo> {
    table
        .all_members()
        .into_iter()
        .map(|m| ClassMemberInfo {
            selector: m.selector,
            kind: m.kind,
        })
        .collect()
}

/// Collects the members visible on `class`: its own, plus inherited members
/// up the `extends` chain, stopping at (and including) the first builtin
/// ancestor.
///
/// Builtin superclass chains are not encoded in `core-table.json`, so a
/// builtin ancestor contributes only its own listed members and terminates the
/// walk. A cycle guard bounds the walk defensively against a malformed
/// (self-inheriting) chain.
fn collect_class_members(
    class: &str,
    index: &WorkspaceIndex,
    table: &CoreTable,
) -> Vec<ClassMemberInfo> {
    let mut out: Vec<ClassMemberInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut guard = std::collections::HashSet::new();
    let mut current = Some(class.to_string());

    while let Some(name) = current {
        if !guard.insert(name.clone()) {
            break;
        }
        if index.has_class(&name) {
            for member in index.class_members(&name) {
                if seen.insert(member.selector.clone()) {
                    out.push(member);
                }
            }
            current = index.class_parent(&name);
        } else if let Some(members) = table.class_members(&name) {
            for member in members {
                if seen.insert(member.selector.clone()) {
                    out.push(ClassMemberInfo {
                        selector: member.selector.clone(),
                        kind: member.kind,
                    });
                }
            }
            current = None;
        } else {
            current = None;
        }
    }
    out
}

/// Renders one class member as a snippet [`CompletionItem`].
///
/// The label is the ADR-0012 comma-form selector; the insert text is a
/// snippet reflecting it (see [`method_snippet`] / [`setter_snippet`]).
fn to_completion_item(member: &ClassMemberInfo) -> CompletionItem {
    let (kind, insert_text, format) = match member.kind {
        MemberKind::Getter => (
            CompletionItemKind::PROPERTY,
            member.selector.clone(),
            InsertTextFormat::PLAIN_TEXT,
        ),
        MemberKind::Setter => (
            CompletionItemKind::PROPERTY,
            setter_snippet(&member.selector),
            InsertTextFormat::SNIPPET,
        ),
        // Methods, static methods, and constructs all render as a
        // parenthesized argument-slot snippet.
        _ => (
            CompletionItemKind::METHOD,
            method_snippet(&member.selector),
            InsertTextFormat::SNIPPET,
        ),
    };
    CompletionItem {
        label: member.selector.clone(),
        kind: Some(kind),
        insert_text: Some(insert_text),
        insert_text_format: Some(format),
        ..CompletionItem::default()
    }
}

/// Renders a method selector's snippet insert text.
///
/// `move(_,to,duration)` becomes `move(${1:_}, to: ${2:_}, duration: ${3:_})`:
/// each positional slot is a bare `${n:_}` tab-stop, each labeled slot is
/// `label: ${n:_}`. A zero-arity `reset()` renders unchanged (no tab-stops).
fn method_snippet(selector: &str) -> String {
    let Some(open) = selector.find('(') else {
        return selector.to_string();
    };
    let name = &selector[..open];
    let inner = selector[open + 1..]
        .strip_suffix(')')
        .unwrap_or(&selector[open + 1..]);
    if inner.is_empty() {
        return format!("{name}()");
    }
    let slots: Vec<String> = inner
        .split(',')
        .enumerate()
        .map(|(i, slot)| {
            let n = i + 1;
            if slot == "_" {
                format!("${{{n}:_}}")
            } else {
                format!("{slot}: ${{{n}:_}}")
            }
        })
        .collect();
    format!("{name}({})", slots.join(", "))
}

/// Renders a setter selector's snippet insert text.
///
/// `x=(_)` becomes `x = ${1:value}` — a bare-name write with a single
/// tab-stop, no parens. Falls back to the raw selector if it is not in the
/// expected `name=(_)` shape.
fn setter_snippet(selector: &str) -> String {
    match selector.strip_suffix("=(_)") {
        Some(base) => format!("{base} = ${{1:value}}"),
        None => selector.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::Document;
    use tower_lsp::lsp_types::Url;

    fn index_with(uri: &str, src: &str) -> WorkspaceIndex {
        let index = WorkspaceIndex::new();
        index.update_file(Url::parse(uri).unwrap(), &phalcom_ast::parser::parse(src, 0).program);
        index
    }

    #[test]
    fn receiver_prefix_reads_receiver_and_partial() {
        assert_eq!(
            receiver_prefix("m.mov", 5),
            Some(("m".to_string(), "mov".to_string()))
        );
        assert_eq!(
            receiver_prefix("foo.", 4),
            Some(("foo".to_string(), String::new()))
        );
    }

    #[test]
    fn receiver_prefix_none_without_dot() {
        assert_eq!(receiver_prefix("bare", 4), None);
        assert_eq!(receiver_prefix(".x", 2), None);
    }

    #[test]
    fn construct_resolver_resolves_new_binding() {
        let src = "let m = Mover.new();\nm.\n";
        let doc = Document::new(src.to_string());
        // Position at end of `m.` on line 1.
        let pos = Position { line: 1, character: 2 };
        assert_eq!(
            ConstructResolver.resolve(&doc, pos).as_deref(),
            Some("Mover")
        );
    }

    #[test]
    fn construct_resolver_none_for_lowercase_receiver_call() {
        let src = "let c = counter.next();\nc.\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 1, character: 2 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), None);
    }

    #[test]
    fn method_snippet_positional_and_labeled_slots() {
        assert_eq!(
            method_snippet("move(_,to,duration)"),
            "move(${1:_}, to: ${2:_}, duration: ${3:_})"
        );
        assert_eq!(method_snippet("reset()"), "reset()");
        assert_eq!(method_snippet("at(_,put)"), "at(${1:_}, put: ${2:_})");
    }

    #[test]
    fn setter_snippet_renders_bare_name_write() {
        assert_eq!(setter_snippet("x=(_)"), "x = ${1:value}");
    }

    #[test]
    fn completions_for_resolved_user_class_only_that_class() {
        let index = index_with(
            "file:///a.ph",
            "class Point {\n  move(x, to:) { }\n  size { }\n}\n",
        );
        let items = completions(Some("Point"), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"move(_,to)"));
        assert!(labels.contains(&"size"));
        // A builtin-only selector must NOT leak in for a resolved user class.
        assert!(!labels.contains(&"ifTrue(_)"));
    }

    #[test]
    fn completions_walk_user_superclass_chain() {
        let index = index_with(
            "file:///a.ph",
            "class Animal {\n  eat() { }\n}\nclass Dog extends Animal {\n  bark() { }\n}\n",
        );
        let items = completions(Some("Dog"), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"bark()"));
        assert!(labels.contains(&"eat()"));
    }

    #[test]
    fn completions_for_builtin_class() {
        let index = WorkspaceIndex::new();
        let items = completions(Some("Bool"), &index, CoreTable::bundled());
        assert!(!items.is_empty());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"ifTrue(_)"));
    }

    #[test]
    fn completions_unknown_receiver_falls_back_to_full_builtin_surface() {
        let index = WorkspaceIndex::new();
        let none = completions(None, &index, CoreTable::bundled());
        assert!(!none.is_empty());
        // Unresolvable class name also falls back rather than returning empty.
        let unknown = completions(Some("Nonexistent"), &index, CoreTable::bundled());
        assert_eq!(none.len(), unknown.len());
    }

    #[test]
    fn completions_are_sorted_by_label() {
        let index = WorkspaceIndex::new();
        let items = completions(None, &index, CoreTable::bundled());
        for pair in items.windows(2) {
            assert!(pair[0].label <= pair[1].label);
        }
    }
}
