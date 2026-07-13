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

use crate::core_table::MemberKind;
use crate::selectors::{class_member_selector, comma_form_from_labels, setter_selector_from_name};

/// One occurrence of a selector at a source location within a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    /// The file the occurrence was found in.
    pub uri: Url,
    /// The occurrence's byte-offset span within that file.
    pub range: SourceRange,
}

/// One member of a class, as the completion path needs it: its ADR-0012
/// comma-form selector and its dispatch [`MemberKind`].
///
/// The class-side counterpart of [`crate::core_table::CoreMember`] — user
/// classes produce these from their AST (Stage 3), builtin classes from
/// `core-table.json`, and both render through one shared path in
/// [`crate::completion`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassMemberInfo {
    /// The member's ADR-0012 comma-form selector (via [`crate::selectors`]).
    pub selector: String,
    /// Whether the member is a getter, setter, or method.
    pub kind: MemberKind,
}

/// One definition site of a selector, enriched with the class it is declared
/// on and its dispatch [`MemberKind`] — the extra information Stage 4
/// (`textDocument/hover`, [`crate::hover`]) needs beyond the plain
/// [`Occurrence`] [`WorkspaceIndex::definitions`] returns, so it can render
/// "`method(_,to)` — method on `Point`" without re-walking the AST a third
/// time.
///
/// Only `ClassMember` declarations produce a [`DefinitionInfo`] today (the
/// only kind of definition `Collector` records); a future top-level `let`
/// doc-hover would need its own, class-less variant of this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionInfo {
    /// The file the definition was found in.
    pub uri: Url,
    /// The definition's byte-offset span within that file.
    pub range: SourceRange,
    /// The name of the class the member is declared on.
    pub class: String,
    /// The member's dispatch kind (method/getter/setter/construct).
    pub kind: MemberKind,
}

/// A `selector -> Vec<Occurrence>` map, plus enough per-file bookkeeping to
/// replace one file's contribution wholesale.
#[derive(Default)]
struct SelectorMap {
    by_selector: DashMap<String, Vec<Occurrence>>,
}

/// A `selector -> Vec<DefinitionInfo>` map, the [`DefinitionInfo`] counterpart
/// of [`SelectorMap`]. Kept as its own small map (rather than folding the
/// class/kind fields into [`Occurrence`] itself) so [`WorkspaceIndex::
/// definitions`]/`goto_definition`'s existing `Occurrence`-shaped callers are
/// untouched by Stage 4.
#[derive(Default)]
struct DefinitionMetaMap {
    by_selector: DashMap<String, Vec<DefinitionInfo>>,
}

impl DefinitionMetaMap {
    fn insert(&self, selector: String, info: DefinitionInfo) {
        self.by_selector.entry(selector).or_default().push(info);
    }

    fn remove_uri(&self, uri: &Url, selectors: &[String]) {
        for selector in selectors {
            if let Some(mut infos) = self.by_selector.get_mut(selector) {
                infos.retain(|info| &info.uri != uri);
            }
        }
    }

    fn get(&self, selector: &str) -> Vec<DefinitionInfo> {
        self.by_selector
            .get(selector)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }
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

/// One class declaration's contribution from a single file: its declared
/// members and its `extends` parent (if any).
///
/// Multiple files (or repeated declarations) can contribute to the same class
/// name; [`ClassMap`] keeps every contribution and merges them on read.
#[derive(Debug, Clone)]
struct ClassEntry {
    /// The file this class declaration was found in.
    uri: Url,
    /// The `extends` superclass name, or `None` for an implicit `Object`
    /// parent (see [`phalcom_ast::ast::ClassDef::superclass`]).
    parent: Option<String>,
    /// The class's own declared members, in declaration order.
    members: Vec<ClassMemberInfo>,
}

/// A `class name -> declarations` map, keyed by the bare class name (not a
/// selector), backing receiver-aware completion (Stage 3).
#[derive(Default)]
struct ClassMap {
    by_class: DashMap<String, Vec<ClassEntry>>,
}

impl ClassMap {
    fn insert(&self, class: String, entry: ClassEntry) {
        self.by_class.entry(class).or_default().push(entry);
    }

    fn remove_uri(&self, uri: &Url, classes: &[String]) {
        for class in classes {
            if let Some(mut entries) = self.by_class.get_mut(class) {
                entries.retain(|entry| &entry.uri != uri);
            }
        }
    }

    /// Every own member of `class`, merged across all contributing files and
    /// de-duplicated by selector (first-seen wins).
    fn members(&self, class: &str) -> Vec<ClassMemberInfo> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        if let Some(entries) = self.by_class.get(class) {
            for entry in entries.iter() {
                for member in &entry.members {
                    if seen.insert(member.selector.clone()) {
                        out.push(member.clone());
                    }
                }
            }
        }
        out
    }

    /// The `extends` parent of `class`, if any declaration named one.
    fn parent(&self, class: &str) -> Option<String> {
        self.by_class
            .get(class)
            .and_then(|entries| entries.iter().find_map(|e| e.parent.clone()))
    }

    /// Whether any file declares a class named `class`.
    fn contains(&self, class: &str) -> bool {
        self.by_class
            .get(class)
            .is_some_and(|entries| !entries.is_empty())
    }
}

/// Which selectors a single file last contributed as definitions/references,
/// so [`WorkspaceIndex::update_file`] knows exactly what to remove before
/// reinserting.
#[derive(Default, Clone)]
struct FileContribution {
    definitions: Vec<String>,
    references: Vec<String>,
    classes: Vec<String>,
}

/// The workspace symbol index: definitions and references, both keyed by
/// ADR-0012 comma-form selector, plus a class-member map (Stage 3) keyed by
/// bare class name for receiver-aware completion.
#[derive(Default)]
pub struct WorkspaceIndex {
    definitions: SelectorMap,
    references: SelectorMap,
    classes: ClassMap,
    /// The [`DefinitionInfo`] (class + kind) counterpart of `definitions`,
    /// consulted by [`Self::definition_info`] (Stage 4 hover). Always
    /// contributed/removed in lockstep with `definitions`, keyed by the same
    /// selector set (see [`FileContribution::definitions`]).
    definition_meta: DefinitionMetaMap,
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
            classes: Vec::new(),
        };
        collector.walk_program(program);

        let mut file_classes = Vec::with_capacity(collector.classes.len());
        for collected in collector.classes {
            let class = collected.name.clone();
            self.classes.insert(
                class.clone(),
                ClassEntry {
                    uri: uri.clone(),
                    parent: collected.parent,
                    members: collected.members,
                },
            );
            file_classes.push(class);
        }

        let mut file_defs = Vec::with_capacity(collector.definitions.len());
        for (selector, range, class, kind) in collector.definitions {
            let occ = Occurrence {
                uri: uri.clone(),
                range,
            };
            self.definitions.insert(selector.clone(), occ);
            self.definition_meta.insert(
                selector.clone(),
                DefinitionInfo {
                    uri: uri.clone(),
                    range,
                    class,
                    kind,
                },
            );
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
                classes: file_classes,
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
            self.definition_meta
                .remove_uri(uri, &contribution.definitions);
            self.references.remove_uri(uri, &contribution.references);
            self.classes.remove_uri(uri, &contribution.classes);
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

    /// The definition site(s) of `selector`, each enriched with the class it
    /// is declared on and its dispatch [`MemberKind`].
    ///
    /// The Stage 4 (`textDocument/hover`) counterpart of [`Self::definitions`]:
    /// hover needs the extra class/kind fields to render "`kind` on `Class`",
    /// which a plain [`Occurrence`] does not carry.
    pub fn definition_info(&self, selector: &str) -> Vec<DefinitionInfo> {
        self.definition_meta.get(selector)
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

    /// The own declared members of the user class named `class` (its selectors
    /// and kinds), merged across every file that declares it and de-duplicated
    /// by selector.
    ///
    /// Does **not** include inherited members — the completion path
    /// ([`crate::completion`]) walks [`Self::class_parent`] itself, so it can
    /// stop the walk when a parent is a builtin (whose members live in
    /// [`crate::core_table`], not here). Empty if no user class of that name
    /// is indexed.
    pub fn class_members(&self, class: &str) -> Vec<ClassMemberInfo> {
        self.classes.members(class)
    }

    /// The `extends` superclass name of the user class named `class`, or
    /// `None` if the class named no explicit superclass (implicit `Object`) or
    /// is not indexed.
    pub fn class_parent(&self, class: &str) -> Option<String> {
        self.classes.parent(class)
    }

    /// Whether a user class named `class` is indexed in the workspace.
    ///
    /// Used by the completion path to decide whether a resolved receiver type
    /// is a user class (walk [`Self::class_members`]) or should fall through
    /// to the builtin [`crate::core_table`].
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.contains(class)
    }
}

/// One class declaration harvested by [`Collector`]: its name, `extends`
/// parent, and own member surface.
struct CollectedClass {
    name: String,
    parent: Option<String>,
    members: Vec<ClassMemberInfo>,
}

/// Maps an AST [`ClassMember`] to the [`MemberKind`] the completion path
/// renders it under.
///
/// A `construct` renders like a method (parenthesized, argument slots).
///
/// A declared field (U-ANNOT-LAYOUT §3.1) renders like a getter — bare name,
/// no parens, no synthesized accessor yet.
fn member_kind(member: &ClassMember) -> MemberKind {
    match member {
        ClassMember::Method(_) | ClassMember::Construct(_) => MemberKind::Method,
        ClassMember::Getter(_) | ClassMember::Field(_) => MemberKind::Getter,
        ClassMember::Setter(_) => MemberKind::Setter,
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
        classes: Vec::new(),
    };
    collector.walk_program(program);

    let def_iter = collector
        .definitions
        .iter()
        .map(|(selector, range, _class, _kind)| (selector, range));
    let ref_iter = collector.references.iter().map(|(selector, range)| (selector, range));

    def_iter
        .chain(ref_iter)
        .filter(|(_, range)| range.contains(offset))
        .min_by_key(|(_, range)| range.len())
        .map(|(selector, _)| selector.clone())
}

/// Walks one file's AST, recording every `ClassMember` declaration as a
/// definition and every selector-bearing send expression as a reference.
struct Collector {
    /// Each definition's selector, source range, defining class name, and
    /// dispatch kind — the class/kind fields feed [`WorkspaceIndex::
    /// definition_meta`] (Stage 4 hover) alongside the plain `definitions`
    /// map.
    definitions: Vec<(String, SourceRange, String, MemberKind)>,
    references: Vec<(String, SourceRange)>,
    classes: Vec<CollectedClass>,
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
        // Stage 3: record the class's own member surface (selector + kind)
        // and its `extends` parent for receiver-aware completion.
        let members = class_def
            .members
            .iter()
            .map(|member| ClassMemberInfo {
                selector: class_member_selector(member),
                kind: member_kind(member),
            })
            .collect();
        self.classes.push(CollectedClass {
            name: class_def.name.clone(),
            parent: class_def.superclass.as_ref().map(|s| s.name.clone()),
            members,
        });

        for member in &class_def.members {
            let selector = class_member_selector(member);
            let class = class_def.name.clone();
            let kind = member_kind(member);
            match member {
                ClassMember::Method(m) => {
                    self.definitions.push((selector, m.range, class, kind));
                    for statement in &m.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Getter(g) => {
                    self.definitions.push((selector, g.range, class, kind));
                    for statement in &g.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Setter(s) => {
                    self.definitions.push((selector, s.range, class, kind));
                    for statement in &s.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Construct(c) => {
                    self.definitions.push((selector, c.range, class, kind));
                    for statement in &c.body {
                        self.walk_statement(statement);
                    }
                }
                ClassMember::Field(f) => {
                    self.definitions.push((selector, f.range, class, kind));
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
    fn class_members_records_selectors_and_kinds() {
        let index = WorkspaceIndex::new();
        let src = "class Point {\n  move(x, to:) { }\n  size { }\n  size=(v) { }\n}\n";
        index.update_file(uri("file:///a.ph"), &parse(src, 0).program);

        let members = index.class_members("Point");
        assert!(members.iter().any(|m| m.selector == "move(_,to)"
            && m.kind == MemberKind::Method));
        assert!(members
            .iter()
            .any(|m| m.selector == "size" && m.kind == MemberKind::Getter));
        assert!(members
            .iter()
            .any(|m| m.selector == "size=(_)" && m.kind == MemberKind::Setter));
        assert!(index.has_class("Point"));
        assert!(!index.has_class("Nope"));
    }

    #[test]
    fn class_parent_reflects_extends() {
        let index = WorkspaceIndex::new();
        let src = "class Dog extends Animal {\n  bark() { }\n}\n";
        index.update_file(uri("file:///a.ph"), &parse(src, 0).program);
        assert_eq!(index.class_parent("Dog").as_deref(), Some("Animal"));

        let src2 = "class Animal {\n  eat() { }\n}\n";
        index.update_file(uri("file:///b.ph"), &parse(src2, 0).program);
        assert_eq!(index.class_parent("Animal"), None);
    }

    #[test]
    fn reparse_replaces_only_the_changed_file_class_members() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        index.update_file(a.clone(), &parse("class A {\n  one() { }\n}\n", 0).program);
        assert!(index
            .class_members("A")
            .iter()
            .any(|m| m.selector == "one()"));

        index.update_file(a.clone(), &parse("class A {\n  two() { }\n}\n", 0).program);
        let members = index.class_members("A");
        assert!(members.iter().any(|m| m.selector == "two()"));
        assert!(!members.iter().any(|m| m.selector == "one()"));
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
