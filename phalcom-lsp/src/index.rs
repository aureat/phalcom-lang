//! Workspace symbol index: `selector -> definitions`/`selector -> references`.
//!
//! Stage 2 (ADR-0056 §4, `docs/forge/units/U-LSP/plan.md`). Built by scanning
//! every `.ph` file under the workspace root(s) at `initialize`, then kept
//! current by [`WorkspaceIndex::update_file`] on each `did_change` — always a
//! **wholesale replace** of the one changed file's slice, never a partial
//! patch (plan "P4": no `phalcom-ast` incremental/sub-file reparse is assumed
//! anywhere here, so a future one only touches this module's update step).
//!
//! Keyed strictly on the ADR-0012 **comma-form** selector — never a bare
//! name — via `crate::selectors`.

use dashmap::DashMap;
use phalcom_ast::ast::{
    Argument, ClassDef, ClassMember, Expr, ForStatement, Pattern, Program, Statement,
};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use crate::selectors::{class_member_selector, comma_form_from_labels, setter_selector_from_name};

/// One occurrence of a selector at a source location within a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    /// The file the occurrence was found in.
    pub uri: Url,
    /// The occurrence's byte-offset span within that file.
    pub range: SourceRange,
}

/// A `selector -> Vec<Occurrence>` map, plus enough per-file bookkeeping to
/// replace one file's contribution wholesale.
#[derive(Default)]
struct SelectorMap {
    by_selector: DashMap<String, Vec<Occurrence>>,
}

impl SelectorMap {
    fn insert(&self, selector: String, occurrence: Occurrence) {
        self.by_selector.entry(selector).or_default().push(occurrence);
    }

    fn remove_uri(&self, uri: &Url, selectors: &[String]) {
        for selector in selectors {
            if let Some(mut occurrences) = self.by_selector.get_mut(selector) {
                occurrences.retain(|occ| &occ.uri != uri);
            }
        }
    }

    fn get(&self, selector: &str) -> Vec<Occurrence> {
        self.by_selector
            .get(selector)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }
}

/// Which selectors a single file last contributed as definitions/references,
/// so [`WorkspaceIndex::update_file`] knows exactly what to remove before
/// reinserting.
#[derive(Default, Clone)]
struct FileContribution {
    definitions: Vec<String>,
    references: Vec<String>,
}

/// The workspace symbol index: definitions and references, both keyed by
/// ADR-0012 comma-form selector.
#[derive(Default)]
pub struct WorkspaceIndex {
    definitions: SelectorMap,
    references: SelectorMap,
    files: DashMap<Url, FileContribution>,
}

impl WorkspaceIndex {
    /// Creates an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes `uri`'s previous contribution (if it was indexed before) and
    /// inserts its current one, walked fresh from `program`.
    ///
    /// Reparsing the same unchanged `program` twice yields byte-identical
    /// definition/reference sets (determinism; plan "Tests / verification").
    pub fn update_file(&self, uri: Url, program: &Program) {
        self.remove_file(&uri);

        let mut collector = Collector {
            definitions: Vec::new(),
            references: Vec::new(),
        };
        collector.walk_program(program);

        let mut file_defs = Vec::with_capacity(collector.definitions.len());
        for (selector, range) in collector.definitions {
            let occ = Occurrence {
                uri: uri.clone(),
                range,
            };
            self.definitions.insert(selector.clone(), occ);
            file_defs.push(selector);
        }

        let mut file_refs = Vec::with_capacity(collector.references.len());
        for (selector, range) in collector.references {
            let occ = Occurrence {
                uri: uri.clone(),
                range,
            };
            self.references.insert(selector.clone(), occ);
            file_refs.push(selector);
        }

        self.files.insert(
            uri,
            FileContribution {
                definitions: file_defs,
                references: file_refs,
            },
        );
    }

    /// Removes every definition/reference previously contributed by `uri`.
    ///
    /// A no-op if `uri` was never indexed. Called by [`Self::update_file`]
    /// before it reinserts, and directly when a file is deleted from the
    /// workspace.
    pub fn remove_file(&self, uri: &Url) {
        if let Some((_, contribution)) = self.files.remove(uri) {
            self.definitions.remove_uri(uri, &contribution.definitions);
            self.references.remove_uri(uri, &contribution.references);
        }
    }

    /// The definition site(s) of `selector` (usually one; more than one
    /// means a redefinition, which the index does not adjudicate — that is
    /// the compiler's job).
    pub fn definitions(&self, selector: &str) -> Vec<Occurrence> {
        self.definitions.get(selector)
    }

    /// Every recorded send-site reference to `selector`.
    pub fn references(&self, selector: &str) -> Vec<Occurrence> {
        self.references.get(selector)
    }

    /// Every defined selector containing `query` as a case-insensitive
    /// substring, each paired with its first definition occurrence
    /// (`workspace/symbol`).
    pub fn symbols_matching(&self, query: &str) -> Vec<(String, Occurrence)> {
        let needle = query.to_lowercase();
        self.definitions
            .by_selector
            .iter()
            .filter(|entry| entry.key().to_lowercase().contains(&needle))
            .filter_map(|entry| entry.value().first().cloned().map(|occ| (entry.key().clone(), occ)))
            .collect()
    }
}

/// Finds the selector of the *smallest* selector-bearing AST node in
/// `program` whose source range contains `offset`.
///
/// This is the "selector under the cursor" resolution `textDocument/
/// definition` and `textDocument/references` (`backend.rs`) need: the
/// smallest enclosing `ClassMember` declaration, `MethodCall`, bare-name
/// `GetProperty`, or `SetProperty` write. Reuses `Collector`'s single walk
/// — the exact same selector spellings and ranges [`WorkspaceIndex::
/// update_file`] indexes a file under — so a hit here is guaranteed to key
/// the same map slot a full index build over the same file would have
/// populated. "Smallest" matters for nested sends (`a.foo(b.bar())`): the
/// inner `b.bar()` call's narrower range wins over the outer `a.foo(...)`
/// call's wider one when the cursor sits inside the inner call.
///
/// Returns `None` if no definition or reference range contains `offset`
/// (e.g. the cursor sits on whitespace, a literal, or a bare variable).
pub fn selector_at_offset(program: &Program, offset: usize) -> Option<String> {
    let mut collector = Collector {
        definitions: Vec::new(),
        references: Vec::new(),
    };
    collector.walk_program(program);

    collector
        .definitions
        .iter()
        .chain(collector.references.iter())
        .filter(|(_, range)| range.contains(offset))
        .min_by_key(|(_, range)| range.len())
        .map(|(selector, _)| selector.clone())
}

/// Walks one file's AST, recording every `ClassMember` declaration as a
/// definition and every selector-bearing send expression as a reference.
struct Collector {
    definitions: Vec<(String, SourceRange)>,
    references: Vec<(String, SourceRange)>,
}

impl Collector {
    fn walk_program(&mut self, program: &Program) {
        for statement in &program.statements {
            self.walk_statement(statement);
        }
    }

    fn walk_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Class(class_def) => self.walk_class(class_def),
            Statement::Let(binding) => {
                self.walk_pattern(&binding.pattern);
                if let Some(value) = &binding.value {
                    self.walk_expr(value);
                }
            }
            Statement::Return(r) => {
                if let Some(value) = &r.value {
                    self.walk_expr(value);
                }
            }
            Statement::Expr { expr, .. } => self.walk_expr(expr),
            Statement::For(f) => self.walk_for(f),
            Statement::Break { .. } | Statement::Continue { .. } => {}
            Statement::Throw { expr, .. } => self.walk_expr(expr),
            Statement::Import(_) => {}
        }
    }

    fn walk_for(&mut self, f: &ForStatement) {
        self.walk_expr(&f.iter);
        for statement in &f.body {
            self.walk_statement(statement);
        }
    }

    fn walk_pattern(&mut self, _pattern: &Pattern) {
        // Patterns bind names only; no selector-bearing content to record.
    }

    fn walk_class(&mut self, class_def: &ClassDef) {
        for member in &class_def.members {
            let selector = class_member_selector(member);
            match member {
                ClassMember::Method(m) => {
                    self.definitions.push((selector, m.range));
                    for statement in &m.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Getter(g) => {
                    self.definitions.push((selector, g.range));
                    for statement in &g.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Setter(s) => {
                    self.definitions.push((selector, s.range));
                    for statement in &s.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Construct(c) => {
                    self.definitions.push((selector, c.range));
                    for statement in &c.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Field(f) => {
                    // A declared field has no body to recurse into, only an
                    // optional default-value expression (which may itself
                    // contain send-site references worth indexing).
                    self.definitions.push((selector, f.range));
                    if let Some(default) = &f.default {
                        self.walk_expr(default);
                    }
                }
            }
        }
        for (expr, _range) in &class_def.invariants {
            self.walk_expr(expr);
        }
    }

    fn walk_args(&mut self, args: &[Argument]) {
        for arg in args {
            self.walk_expr(&arg.expr);
        }
    }

    fn walk_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::Var { .. }
            | Expr::Field { .. }
            | Expr::SelfVar { .. }
            | Expr::SuperVar { .. }
            | Expr::Symbol(_) => {}
            Expr::Assignment(a) => {
                self.walk_expr(&a.name);
                self.walk_expr(&a.value);
            }
            Expr::Unary(u) => self.walk_expr(&u.expr),
            Expr::Binary(b) => {
                self.walk_expr(&b.left);
                self.walk_expr(&b.right);
            }
            Expr::MethodCall(m) => {
                self.walk_expr(&m.object);
                let labels: Vec<Option<String>> =
                    m.args.iter().map(|a| a.label.clone()).collect();
                let selector = comma_form_from_labels(&m.method, &labels);
                self.references.push((selector, m.range));
                self.walk_args(&m.args);
            }
            Expr::GetProperty(g) => {
                self.walk_expr(&g.object);
                // Bare-name access resolves to the getter selector — the
                // same "no parens" spelling `selectors::getter_selector`
                // gives a `GetterDef` declaration.
                self.references.push((g.property.clone(), g.range));
            }
            Expr::SetProperty(s) => {
                self.walk_expr(&s.object);
                self.walk_expr(&s.value);
                self.references
                    .push((setter_selector_from_name(&s.property), s.range));
            }
            Expr::Index(i) => {
                self.walk_expr(&i.object);
                self.walk_expr(&i.index);
            }
            Expr::SetIndex(si) => {
                self.walk_expr(&si.object);
                self.walk_expr(&si.index);
                self.walk_expr(&si.value);
            }
            Expr::Block(b) => {
                for statement in &b.body {
                    self.walk_statement(statement);
                }
            }
            Expr::MethodRef(mr) => {
                self.walk_expr(&mr.receiver);
                // Only the Pinned form (`obj::#name(_,to,duration)`) carries
                // a full selector at the reference site — it interns to the
                // same `Symbol` as its target method (ADR-0012, see
                // `MethodRefKind::Pinned`'s doc). The Open form (`obj::name`)
                // is a bare base name whose selector is resolved at call
                // time from the caller's argument labels, so there is no
                // single selector to index it under here.
                if let phalcom_ast::ast::MethodRefKind::Pinned { name, labels } = &mr.kind {
                    let selector = comma_form_from_labels(name, labels);
                    self.references.push((selector, mr.range));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    fn uri(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn definition_and_reference_land_at_the_right_selector() {
        let src = "class Point {\n  move(x, to:, duration:) { }\n}\n\nlet p = Point.new();\np.move(1, to: 2, duration: 3);\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let index = WorkspaceIndex::new();
        index.update_file(uri("file:///a.ph"), &parsed.program);

        let defs = index.definitions("move(_,to,duration)");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].uri, uri("file:///a.ph"));

        let refs = index.references("move(_,to,duration)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].uri, uri("file:///a.ph"));
    }

    #[test]
    fn getter_and_zero_arity_method_do_not_alias() {
        let src = "class Point {\n  y { }\n  x() { }\n}\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let index = WorkspaceIndex::new();
        index.update_file(uri("file:///a.ph"), &parsed.program);

        assert_eq!(index.definitions("y").len(), 1);
        assert_eq!(index.definitions("x()").len(), 1);
        assert!(index.definitions("y()").is_empty());
    }

    #[test]
    fn did_change_reparse_replaces_only_the_changed_file() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let b = uri("file:///b.ph");

        let src_a = "class A {\n  greet() { }\n}\n";
        let src_b = "class B {\n  greet() { }\n}\n";
        index.update_file(a.clone(), &parse(src_a, 0).program);
        index.update_file(b.clone(), &parse(src_b, 0).program);

        assert_eq!(index.definitions("greet()").len(), 2);

        // Reparse `a` with a renamed method — `a`'s old entry must vanish,
        // `b`'s must survive untouched.
        let src_a2 = "class A {\n  hello() { }\n}\n";
        index.update_file(a.clone(), &parse(src_a2, 0).program);

        let greet_defs = index.definitions("greet()");
        assert_eq!(greet_defs.len(), 1);
        assert_eq!(greet_defs[0].uri, b);

        let hello_defs = index.definitions("hello()");
        assert_eq!(hello_defs.len(), 1);
        assert_eq!(hello_defs[0].uri, a);
    }

    #[test]
    fn reparsing_unchanged_file_is_deterministic() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let src = "class A {\n  greet(x, to:) { }\n}\nlet a = A.new();\na.greet(1, to: 2);\n";
        let program = parse(src, 0).program;

        index.update_file(a.clone(), &program);
        let defs1 = index.definitions("greet(_,to)");
        let refs1 = index.references("greet(_,to)");

        index.update_file(a.clone(), &program);
        let defs2 = index.definitions("greet(_,to)");
        let refs2 = index.references("greet(_,to)");

        assert_eq!(defs1, defs2);
        assert_eq!(refs1, refs2);
    }

    #[test]
    fn selector_at_offset_finds_the_innermost_call() {
        // `p.move(1, to: 2, duration: 3)` — cursor placed on `move`, inside
        // the outer call, must resolve to the call's own selector.
        let src = "class Point {\n  move(x, to:, duration:) { }\n}\n\nlet p = Point.new();\np.move(1, to: 2, duration: 3);\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let call_site_move = src.rfind("move").unwrap();
        let selector = selector_at_offset(&parsed.program, call_site_move);
        assert_eq!(selector.as_deref(), Some("move(_,to,duration)"));
    }

    #[test]
    fn selector_at_offset_prefers_the_innermost_nested_call() {
        let src = "class A {\n  outer(x) { }\n  inner() { }\n}\n\nlet a = A.new();\na.outer(a.inner());\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let inner_offset = src.rfind("inner()").unwrap();
        let selector = selector_at_offset(&parsed.program, inner_offset);
        assert_eq!(selector.as_deref(), Some("inner()"));
    }

    #[test]
    fn selector_at_offset_on_declaration_resolves_to_its_own_selector() {
        let src = "class Point {\n  greet() { }\n}\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let decl_offset = src.find("greet").unwrap();
        let selector = selector_at_offset(&parsed.program, decl_offset);
        assert_eq!(selector.as_deref(), Some("greet()"));
    }

    #[test]
    fn selector_at_offset_returns_none_on_whitespace() {
        let src = "class Point {\n  greet() { }\n}\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        // Offset 0 sits on `c` of `class` — not inside any selector-bearing
        // node's range.
        assert_eq!(selector_at_offset(&parsed.program, 0), None);
    }

    #[test]
    fn workspace_symbol_matches_substring_case_insensitively() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let src = "class A {\n  moveTo(x) { }\n}\n";
        index.update_file(a, &parse(src, 0).program);

        let matches = index.symbols_matching("moveto");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, "moveTo(_)");
    }

    #[test]
    fn pinned_method_ref_is_recorded_as_a_reference() {
        let src = "class Point {\n  move(x, to:) { }\n}\n\nlet p = Point.new();\nlet f = p::#move(_,to);\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let index = WorkspaceIndex::new();
        index.update_file(uri("file:///a.ph"), &parsed.program);

        let refs = index.references("move(_,to)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].uri, uri("file:///a.ph"));
    }
}
