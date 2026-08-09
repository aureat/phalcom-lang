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
    AttrKind, BuiltinAttr, ClassDef, ClassMember, Expr, ForStatement, ListLiteralElement, ListLiteralExpr, MapLiteralEntry, MapLiteralKey, PackItem, PackLabel,
    Pattern, ProductLabel, Program, SetLiteralEntry, Statement, TupleLiteralEntry,
};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use crate::core_table::MemberKind;
use crate::selectors::{class_member_selector, comma_form_from_labels, setter_selector_from_name};

/// Dynamic labels and expansions build their concrete selector at runtime;
/// avoid indexing a fabricated static reference for those pack forms.
fn static_pack_labels(items: &[PackItem]) -> Option<Vec<Option<String>>> {
    items
        .iter()
        .map(|item| match item {
            PackItem::Positional { .. } => Some(None),
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(Some(text.clone())),
            PackItem::Labeled {
                label: PackLabel::Computed { .. },
                ..
            }
            | PackItem::Expand { .. } => None,
        })
        .collect()
}

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
    /// Class that declares this member. Used for lexical visibility checks.
    pub owner: String,
    /// Source-level access category.
    pub visibility: MemberVisibility,
    /// True for `@class` placement and constructors.
    pub is_class_side: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Source-level member access category used by completion filtering.
pub enum MemberVisibility {
    /// Accessible from ordinary source.
    Public,
    /// Accessible only in defining lexical class.
    Private,
    /// Accessible in defining class and subclasses.
    Protected,
    /// Accessible only to privileged core/runtime source.
    Internal,
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
        self.by_selector.get(selector).map(|entry| entry.clone()).unwrap_or_default()
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
        self.by_selector.get(selector).map(|entry| entry.clone()).unwrap_or_default()
    }
}

/// One class declaration: its declared members and its `extends` parent (if
/// any).
///
/// There is exactly one entry per `(file, class name)` pair. The declaring
/// file is not stored here — it is half of [`ClassMap`]'s key.
#[derive(Debug, Clone)]
struct ClassEntry {
    /// The `is` superclass name, or `None` for an implicit `Object`
    /// parent (see [`phalcom_ast::ast::ClassDef::superclass`]).
    parent: Option<String>,
    /// The class's own declared members, in declaration order.
    members: Vec<ClassMemberInfo>,
}

/// A `(file, class name) -> declaration` map backing receiver-aware
/// completion (Stage 3).
///
/// # Why the key carries the file
///
/// Class identity in Phalcom is `(module, name)`, not `name`
/// ([PDR-0001](../../../docs/decisions/0001-classes-are-closed.md); the VM
/// side is `ClassKey` in `phalcom-core/src/vm/mod.rs`). A file *is* a module
/// (ADR-0045), and this crate never resolves `import` — `Statement::Import` is
/// a no-op in every walker — so the document [`Url`] is the correct and only
/// module proxy available here.
///
/// This map previously held `DashMap<String, Vec<ClassEntry>>`, a shape whose
/// `Vec` existed solely to model *one* class reopened across several files.
/// Class reopening was removed by U-CLASSCLOSE, so that `Vec` no longer models
/// anything real — it only merged genuinely-distinct classes that happened to
/// share a name, producing two live wrong answers (see
/// `docs/logs/2026-07-20-u-classns-lsp-classmap-collapse.md`). Collapsing to
/// one entry per key is what makes those answers correct, not merely faster.
#[derive(Default)]
struct ClassMap {
    by_class: DashMap<(Url, String), ClassEntry>,
}

impl ClassMap {
    /// Records `class` as declared in `uri`.
    ///
    /// A second declaration of the same name in the same file overwrites the
    /// first. That is not a merge: intra-module redefinition is a *compile
    /// error* under PDR-0001 (`class.already_defined`), so the only file that
    /// can reach this path is already invalid, and last-wins keeps the index
    /// shaped like the code the user is editing toward.
    fn insert(&self, uri: Url, class: String, entry: ClassEntry) {
        self.by_class.insert((uri, class), entry);
    }

    fn remove_uri(&self, uri: &Url, classes: &[String]) {
        for class in classes {
            self.by_class.remove(&(uri.clone(), class.clone()));
        }
    }

    /// The own members of `class` **as declared in `uri`**, in declaration
    /// order. Empty if that file declares no such class.
    ///
    /// No cross-file merge and no de-duplication: there is one declaration, so
    /// there is nothing to merge and no selector can collide with itself.
    fn members(&self, uri: &Url, class: &str) -> Vec<ClassMemberInfo> {
        self.by_class
            .get(&(uri.clone(), class.to_string()))
            .map(|entry| entry.members.clone())
            .unwrap_or_default()
    }

    /// The `is` parent of `class` **as declared in `uri`**, if it named
    /// one.
    fn parent(&self, uri: &Url, class: &str) -> Option<String> {
        self.by_class.get(&(uri.clone(), class.to_string())).and_then(|entry| entry.parent.clone())
    }

    /// Whether `uri` declares a class named `class`.
    fn contains(&self, uri: &Url, class: &str) -> bool {
        self.by_class.contains_key(&(uri.clone(), class.to_string()))
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
                uri.clone(),
                class.clone(),
                ClassEntry {
                    parent: collected.parent,
                    members: collected.members,
                },
            );
            file_classes.push(class);
        }

        let mut file_defs = Vec::with_capacity(collector.definitions.len());
        for (selector, range, class, kind) in collector.definitions {
            let occ = Occurrence { uri: uri.clone(), range };
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
            let occ = Occurrence { uri: uri.clone(), range };
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
            self.definition_meta.remove_uri(uri, &contribution.definitions);
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

    /// The own declared members of the user class named `class` **as declared
    /// in `uri`** (its selectors and kinds), in declaration order.
    ///
    /// Does **not** include inherited members — the completion path
    /// ([`crate::completion`]) walks [`Self::class_parent`] itself, so it can
    /// stop the walk when a parent is a builtin (whose members live in
    /// [`crate::core_table`], not here). Empty if `uri` declares no such class.
    ///
    /// `uri` is not an optimization. A class named `Point` in one file and a
    /// class named `Point` in another are *different classes* (PDR-0001), so a
    /// name alone does not identify one — see [`ClassMap`].
    pub fn class_members(&self, uri: &Url, class: &str) -> Vec<ClassMemberInfo> {
        self.classes.members(uri, class)
    }

    /// The `is` superclass name of the user class named `class` **as
    /// declared in `uri`**, or `None` if it named no explicit superclass
    /// (implicit `Object`) or `uri` declares no such class.
    pub fn class_parent(&self, uri: &Url, class: &str) -> Option<String> {
        self.classes.parent(uri, class)
    }

    /// Whether `class` is `ancestor` or inherits from it in this document.
    pub fn is_same_or_subclass(&self, uri: &Url, class: &str, ancestor: &str) -> bool {
        let mut current = Some(class.to_string());
        let mut seen = std::collections::HashSet::new();
        while let Some(name) = current {
            if name == ancestor {
                return true;
            }
            if !seen.insert(name.clone()) {
                return false;
            }
            current = self.class_parent(uri, &name);
        }
        false
    }

    /// Whether `uri` declares a user class named `class`.
    ///
    /// Used by the completion path to decide whether a resolved receiver type
    /// is a user class (walk [`Self::class_members`]) or should fall through
    /// to the builtin [`crate::core_table`].
    pub fn has_class(&self, uri: &Url, class: &str) -> bool {
        self.classes.contains(uri, class)
    }
}

/// One class declaration harvested by [`Collector`]: its name, `is`
/// parent, and own member surface.
struct CollectedClass {
    name: String,
    parent: Option<String>,
    members: Vec<ClassMemberInfo>,
}

/// Maps an AST [`ClassMember`] to the [`MemberKind`] the completion path
/// renders it under.
///
/// Declaration shape determines `MemberKind`. Class-side placement is stored
/// separately in [`ClassMemberInfo::is_class_side`], so an `@class` getter
/// remains a getter for snippet rendering.
///
/// A constructor is a class-side factory — [`MemberKind::Construct`], never
/// dispatched on an instance.
///
/// A declared field (U-ANNOT-LAYOUT §3.1) renders like a getter — bare name,
/// no parens, no synthesized accessor yet.
///
/// A `@variant` arm ([`ClassMember::Variant`]) is not a message selector at
/// all (see [`crate::selectors::class_member_selector`]'s doc); it renders as
/// [`MemberKind::Getter`] only so it has *some* harmless completion-item
/// shape rather than being silently dropped.
fn member_kind(member: &ClassMember) -> MemberKind {
    match member {
        ClassMember::Method(m) if m.is_constructor || m.attributes.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Constructor))) => {
            MemberKind::Construct
        }
        ClassMember::Method(_) => MemberKind::Method,
        ClassMember::Getter(_) => MemberKind::Getter,
        ClassMember::Setter(_) => MemberKind::Setter,
        ClassMember::Field(_) | ClassMember::Variant(_) => MemberKind::Getter,
        // A bracket subscript method (U-INDEX, ADR-0060: `[idx] { ... }`) is
        // an ordinary dispatchable instance method, just with no name token
        // — closest existing completion shape is `MemberKind::Method`.
        ClassMember::Index(_) => MemberKind::Method,
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

fn member_visibility(member: &ClassMember) -> MemberVisibility {
    let (name, attrs, is_field) = match member {
        ClassMember::Method(m) => (Some(m.name.as_str()), m.attributes.as_slice(), false),
        ClassMember::Getter(g) => (Some(g.name.as_str()), g.attributes.as_slice(), false),
        ClassMember::Setter(s) => (Some(s.name.as_str()), s.attributes.as_slice(), false),
        ClassMember::Index(ix) => (None, ix.attributes.as_slice(), false),
        ClassMember::Field(f) => (Some(f.name.as_str()), f.attributes.as_slice(), true),
        ClassMember::Variant(v) => (Some(v.name.as_str()), v.attributes.as_slice(), false),
    };
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if attrs.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Private))) || is_field {
        MemberVisibility::Private
    } else if attrs.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Protected))) {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
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
///
/// Also returns the matched node's own [`SourceRange`] alongside the
/// selector, so a caller (`hover_at`) can underline the exact span the
/// selector was resolved from — the same span [`WorkspaceIndex::
/// definitions`]/`update_file` would have indexed this occurrence under.
pub fn selector_at_offset(program: &Program, offset: usize) -> Option<(String, SourceRange)> {
    let mut collector = Collector {
        definitions: Vec::new(),
        references: Vec::new(),
        classes: Vec::new(),
    };
    collector.walk_program(program);

    let def_iter = collector.definitions.iter().map(|(selector, range, _class, _kind)| (selector, range));
    let ref_iter = collector.references.iter().map(|(selector, range)| (selector, range));

    def_iter
        .chain(ref_iter)
        .filter(|(_, range)| range.contains(offset))
        .min_by_key(|(_, range)| range.len())
        .map(|(selector, range)| (selector.clone(), *range))
}

/// Finds the bound name of the top-level `let`/`var` binding whose
/// non-destructuring [`Pattern::Name`] identifier occurrence contains
/// `offset` — either the binding's own declaration site or a later bare
/// [`Expr::Var`] read/write of that name within a top-level statement.
///
/// The doc-hover counterpart of [`selector_at_offset`] for a top-level
/// binding (`hover.rs`'s `harvest_doc_for_selector` keys a `///` block above
/// a top-level `let`/`var` by the bound name, not a selector — see
/// `hover::top_level_binding_name_at_line`'s doc). Deliberately narrow: only
/// `program.statements` themselves and the expressions they directly embed
/// are walked, never a class member's body — a same-named local variable
/// inside a method is a *different* binding and must not resolve here.
///
/// Returns `None` if `offset` sits on no identifier occurrence naming a
/// top-level binding.
pub fn top_level_binding_at_offset(program: &Program, offset: usize) -> Option<String> {
    let mut names = std::collections::HashSet::new();
    for statement in &program.statements {
        if let Statement::Let(binding) = statement {
            if let Pattern::Name { name, range } = &binding.pattern {
                names.insert(name.clone());
                if range.contains(offset) {
                    return Some(name.clone());
                }
            }
        }
    }
    if names.is_empty() {
        return None;
    }

    let mut hits: Vec<(String, SourceRange)> = Vec::new();
    for statement in &program.statements {
        collect_var_occurrences(statement, &names, &mut hits);
    }
    hits.into_iter()
        .filter(|(_, range)| range.contains(offset))
        .min_by_key(|(_, range)| range.len())
        .map(|(name, _)| name)
}

/// Collects every bare [`Expr::Var`] occurrence in `statement` naming one of
/// `names`, paired with its own [`SourceRange`] — [`top_level_binding_at_offset`]'s
/// worker. Recurses into every expression position a top-level statement can
/// embed one at (including a nested block's own statements), but never into
/// a [`Statement::Class`]'s member bodies (out of scope, see that function's
/// doc).
fn collect_var_occurrences(statement: &Statement, names: &std::collections::HashSet<String>, out: &mut Vec<(String, SourceRange)>) {
    match statement {
        Statement::Let(binding) => {
            if let Some(value) = &binding.value {
                collect_var_occurrences_in_expr(value, names, out);
            }
        }
        Statement::Return(r) => {
            if let Some(value) = &r.value {
                collect_var_occurrences_in_expr(value, names, out);
            }
        }
        Statement::Expr { expr, .. } => collect_var_occurrences_in_expr(expr, names, out),
        Statement::For(f) => {
            collect_var_occurrences_in_expr(&f.iter, names, out);
            for s in &f.body {
                collect_var_occurrences(s, names, out);
            }
        }
        Statement::Throw { expr, .. } => collect_var_occurrences_in_expr(expr, names, out),
        Statement::Break { .. } | Statement::Continue { .. } | Statement::Import(_) | Statement::Class(_) => {}
    }
}

/// The expression-level worker behind [`collect_var_occurrences`], recursing
/// through every sub-expression position (including nested block bodies) to
/// find bare [`Expr::Var`] reads/writes naming one of `names`.
fn collect_var_occurrences_in_expr(expr: &Expr, names: &std::collections::HashSet<String>, out: &mut Vec<(String, SourceRange)>) {
    match expr {
        Expr::Var { value, range } => {
            if names.contains(value) {
                out.push((value.clone(), *range));
            }
        }
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Field { .. }
        | Expr::ImplementationSelector { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::Symbol(_) => {}
        Expr::Assignment(a) => {
            collect_var_occurrences_in_expr(&a.name, names, out);
            collect_var_occurrences_in_expr(&a.value, names, out);
        }
        Expr::Range(range) => {
            if let Some(lower) = &range.lower {
                collect_var_occurrences_in_expr(lower, names, out);
            }
            if let Some(upper) = &range.upper {
                collect_var_occurrences_in_expr(upper, names, out);
            }
        }
        Expr::Unary(u) => collect_var_occurrences_in_expr(&u.expr, names, out),
        Expr::Binary(b) => {
            collect_var_occurrences_in_expr(&b.left, names, out);
            collect_var_occurrences_in_expr(&b.right, names, out);
        }
        Expr::MethodCall(m) => {
            collect_var_occurrences_in_expr(&m.object, names, out);
            for arg in &m.args {
                collect_var_occurrences_in_pack_item(arg, names, out);
            }
        }
        Expr::UnqualifiedCall(m) => {
            for arg in &m.args {
                collect_var_occurrences_in_pack_item(arg, names, out);
            }
        }
        Expr::GetProperty(g) => collect_var_occurrences_in_expr(&g.object, names, out),
        Expr::SetProperty(s) => {
            collect_var_occurrences_in_expr(&s.object, names, out);
            collect_var_occurrences_in_expr(&s.value, names, out);
        }
        Expr::Index(i) => {
            collect_var_occurrences_in_expr(&i.object, names, out);
            for arg in &i.args {
                collect_var_occurrences_in_pack_item(arg, names, out);
            }
        }
        Expr::SetIndex(si) => {
            collect_var_occurrences_in_expr(&si.object, names, out);
            for arg in &si.args {
                collect_var_occurrences_in_pack_item(arg, names, out);
            }
            collect_var_occurrences_in_expr(&si.value, names, out);
        }
        Expr::Block(b) => {
            for s in &b.body {
                collect_var_occurrences(s, names, out);
            }
        }
        Expr::MethodRef(mr) => collect_var_occurrences_in_expr(&mr.receiver, names, out),
        Expr::TupleLiteral(tuple) => {
            for entry in &tuple.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, .. } => collect_var_occurrences_in_expr(expr, names, out),
                    TupleLiteralEntry::Labeled { label, value, .. } => {
                        collect_product_label_var_occurrences(label, names, out);
                        collect_var_occurrences_in_expr(value, names, out);
                    }
                    TupleLiteralEntry::Expand { expr, .. } => collect_var_occurrences_in_expr(expr, names, out),
                }
            }
        }
        Expr::RecordLiteral(record) => {
            for field in &record.fields {
                collect_product_label_var_occurrences(&field.label, names, out);
                collect_var_occurrences_in_expr(&field.value, names, out);
            }
        }
        Expr::MapLiteral(map) => {
            for entry in &map.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_var_occurrences_in_expr(expr, names, out);
                        }
                        collect_var_occurrences_in_expr(value, names, out);
                    }
                    MapLiteralEntry::Expansion { expr, .. } => collect_var_occurrences_in_expr(expr, names, out),
                }
            }
        }
        Expr::SetLiteral(set) => {
            for entry in &set.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                        collect_var_occurrences_in_expr(expr, names, out);
                    }
                }
            }
        }
        Expr::ListLiteral(list) => {
            for element in &list.elements {
                match element {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => {
                        collect_var_occurrences_in_expr(expr, names, out);
                    }
                }
            }
        }
    }
}

fn collect_var_occurrences_in_pack_item(
    item: &PackItem,
    names: &std::collections::HashSet<String>,
    out: &mut Vec<(String, SourceRange)>,
) {
    match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => collect_var_occurrences_in_expr(expr, names, out),
        PackItem::Labeled { label, value, .. } => {
            if let PackLabel::Computed { expr, .. } = label {
                collect_var_occurrences_in_expr(expr, names, out);
            }
            collect_var_occurrences_in_expr(value, names, out);
        }
    }
}

fn collect_product_label_var_occurrences(label: &ProductLabel, names: &std::collections::HashSet<String>, out: &mut Vec<(String, SourceRange)>) {
    if let ProductLabel::Computed { expr, .. } = label {
        collect_var_occurrences_in_expr(expr, names, out);
    }
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
        // and its `is` superclass parent for receiver-aware completion.
        let members = class_def
            .members
            .iter()
            .map(|member| ClassMemberInfo {
                selector: class_member_selector(member),
                kind: member_kind(member),
                owner: class_def.name.clone(),
                visibility: member_visibility(member),
                is_class_side: member_is_class_side(member),
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
                ClassMember::Field(f) => {
                    self.definitions.push((selector, f.range, class, kind));
                    if let Some(default) = &f.default {
                        self.walk_expr(default);
                    }
                }
                // No body and no default-valued sub-expression to walk (see
                // `member_kind`'s doc for why this isn't a real selector).
                ClassMember::Variant(v) => {
                    self.definitions.push((selector, v.range, class, kind));
                }
                ClassMember::Index(ix) => {
                    self.definitions.push((selector, ix.range, class, kind));
                    for statement in &ix.body {
                        self.walk_statement(statement);
                    }
                }
            }
        }
        for (expr, _range) in &class_def.invariants {
            self.walk_expr(expr);
        }
    }

    fn walk_args(&mut self, args: &[PackItem]) {
        for arg in args {
            match arg {
                PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => self.walk_expr(expr),
                PackItem::Labeled { label, value, .. } => {
                    if let PackLabel::Computed { expr, .. } = label {
                        self.walk_expr(expr);
                    }
                    self.walk_expr(value);
                }
            }
        }
    }

    fn walk_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::Var { .. }
            | Expr::Field { .. }
            | Expr::ImplementationSelector { .. }
            | Expr::SelfVar { .. }
            | Expr::SuperVar { .. }
            | Expr::Symbol(_) => {}
            Expr::Assignment(a) => {
                self.walk_expr(&a.name);
                self.walk_expr(&a.value);
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    self.walk_expr(lower);
                }
                if let Some(upper) = &range.upper {
                    self.walk_expr(upper);
                }
            }
            Expr::Unary(u) => self.walk_expr(&u.expr),
            Expr::Binary(b) => {
                self.walk_expr(&b.left);
                self.walk_expr(&b.right);
            }
            Expr::MethodCall(m) => {
                self.walk_expr(&m.object);
                if let Some(labels) = static_pack_labels(&m.args) {
                    self.references.push((comma_form_from_labels(&m.method, &labels), m.range));
                }
                self.walk_args(&m.args);
            }
            Expr::UnqualifiedCall(m) => {
                if let Some(labels) = static_pack_labels(&m.args) {
                    self.references.push((comma_form_from_labels(&m.name, &labels), m.range));
                }
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
                self.references.push((setter_selector_from_name(&s.property), s.range));
            }
            Expr::Index(i) => {
                self.walk_expr(&i.object);
                self.walk_args(&i.args);
            }
            Expr::SetIndex(si) => {
                self.walk_expr(&si.object);
                self.walk_args(&si.args);
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
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    match entry {
                        TupleLiteralEntry::Positional { expr, .. } => self.walk_expr(expr),
                        TupleLiteralEntry::Labeled { label, value, .. } => {
                            self.walk_product_label(label);
                            self.walk_expr(value);
                        }
                        TupleLiteralEntry::Expand { expr, .. } => self.walk_expr(expr),
                    }
                }
            }
            Expr::RecordLiteral(record) => {
                for field in &record.fields {
                    self.walk_product_label(&field.label);
                    self.walk_expr(&field.value);
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        MapLiteralEntry::Association { key, value, .. } => {
                            if let MapLiteralKey::Computed { expr, .. } = key {
                                self.walk_expr(expr);
                            }
                            self.walk_expr(value);
                        }
                        MapLiteralEntry::Expansion { expr, .. } => self.walk_expr(expr),
                    }
                }
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    match entry {
                        SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => self.walk_expr(expr),
                    }
                }
            }
            Expr::ListLiteral(list) => {
                for element in &list.elements {
                    match element {
                        ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => self.walk_expr(expr),
                    }
                }
            }
        }
    }

    fn walk_product_label(&mut self, label: &ProductLabel) {
        if let ProductLabel::Computed { expr, .. } = label {
            self.walk_expr(expr);
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
        let src = "class Point {\n  move(_ x, to, duration) { }\n}\n\nlet p = Point.new();\np.move(1, to: 2, duration: 3);\n";
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
        let src = "class A {\n  greet(_ x, to) { }\n}\nlet a = A.new();\na.greet(1, to: 2);\n";
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
        let src = "class Point {\n  move(_ x, to, duration) { }\n}\n\nlet p = Point.new();\np.move(1, to: 2, duration: 3);\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let call_site_move = src.rfind("move").unwrap();
        let (selector, range) = selector_at_offset(&parsed.program, call_site_move).unwrap();
        assert_eq!(selector, "move(_,to,duration)");
        assert!(range.contains(call_site_move));
    }

    #[test]
    fn selector_at_offset_prefers_the_innermost_nested_call() {
        let src = "class A {\n  outer(_ x) { }\n  inner() { }\n}\n\nlet a = A.new();\na.outer(a.inner());\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let inner_offset = src.rfind("inner()").unwrap();
        let (selector, _range) = selector_at_offset(&parsed.program, inner_offset).unwrap();
        assert_eq!(selector, "inner()");
    }

    #[test]
    fn selector_at_offset_on_declaration_resolves_to_its_own_selector() {
        let src = "class Point {\n  greet() { }\n}\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let decl_offset = src.find("greet").unwrap();
        let (selector, _range) = selector_at_offset(&parsed.program, decl_offset).unwrap();
        assert_eq!(selector, "greet()");
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
        let src = "class A {\n  moveTo(_ x) { }\n}\n";
        index.update_file(a, &parse(src, 0).program);

        let matches = index.symbols_matching("moveto");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, "moveTo(_)");
    }

    #[test]
    fn class_members_records_selectors_and_kinds() {
        let index = WorkspaceIndex::new();
        let src = "class Point {\n  move(_ x, to) { }\n  size { }\n  size=(put v) { }\n}\n";
        index.update_file(uri("file:///a.ph"), &parse(src, 0).program);

        let a = uri("file:///a.ph");
        let members = index.class_members(&a, "Point");
        assert!(members.iter().any(|m| m.selector == "move(_,to)" && m.kind == MemberKind::Method));
        assert!(members.iter().any(|m| m.selector == "size" && m.kind == MemberKind::Getter));
        assert!(members.iter().any(|m| m.selector == "size=(put)" && m.kind == MemberKind::Setter));
        assert!(index.has_class(&a, "Point"));
        assert!(!index.has_class(&a, "Nope"));
    }

    #[test]
    fn class_parent_reflects_extends() {
        let index = WorkspaceIndex::new();
        let src = "class Dog is Animal {\n  bark() { }\n}\n";
        index.update_file(uri("file:///a.ph"), &parse(src, 0).program);
        assert_eq!(index.class_parent(&uri("file:///a.ph"), "Dog").as_deref(), Some("Animal"));

        let src2 = "class Animal {\n  eat() { }\n}\n";
        index.update_file(uri("file:///b.ph"), &parse(src2, 0).program);
        assert_eq!(index.class_parent(&uri("file:///b.ph"), "Animal"), None);
    }

    /// Two files, same class name, different members. Each file's `Point` must
    /// answer with *only its own* members.
    ///
    /// **Negative control:** this fails on the pre-collapse index, whose
    /// `members()` unioned every contributing file's entry — `a.ph`'s `Point`
    /// would offer `bee()`. Class identity is `(module, name)` (PDR-0001) and a
    /// file is a module (ADR-0045), so these are two unrelated classes.
    #[test]
    fn same_class_name_in_two_files_does_not_merge_members() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let b = uri("file:///b.ph");
        index.update_file(a.clone(), &parse("class Point {\n  aye() { }\n}\n", 0).program);
        index.update_file(b.clone(), &parse("class Point {\n  bee() { }\n}\n", 0).program);

        let from_a: Vec<String> = index.class_members(&a, "Point").into_iter().map(|m| m.selector).collect();
        let from_b: Vec<String> = index.class_members(&b, "Point").into_iter().map(|m| m.selector).collect();

        assert_eq!(from_a, vec!["aye()".to_string()], "a.ph's Point must not see b.ph's members");
        assert_eq!(from_b, vec!["bee()".to_string()], "b.ph's Point must not see a.ph's members");
    }

    /// Same name, but only *one* file's declaration has an `extends`. The file
    /// without one must report no parent.
    ///
    /// **Negative control:** this fails on the pre-collapse index, whose
    /// `parent()` was `entries.iter().find_map(|e| e.parent.clone())` — the
    /// first entry that named *any* superclass answered for every file, so
    /// `a.ph`'s parentless `Point` inherited `b.ph`'s `Shape`.
    #[test]
    fn same_class_name_in_two_files_does_not_share_a_parent() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let b = uri("file:///b.ph");
        index.update_file(a.clone(), &parse("class Point {\n  aye() { }\n}\n", 0).program);
        index.update_file(b.clone(), &parse("class Point is Shape {\n  bee() { }\n}\n", 0).program);

        assert_eq!(index.class_parent(&a, "Point"), None, "a.ph's Point declared no extends");
        assert_eq!(index.class_parent(&b, "Point").as_deref(), Some("Shape"));
    }

    /// A class declared in one file is not "present" when asked about another.
    ///
    /// **Negative control:** `contains()` took only a name before, so this
    /// returned `true` for every open file in the workspace.
    #[test]
    fn has_class_is_scoped_to_the_declaring_file() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let b = uri("file:///b.ph");
        index.update_file(a.clone(), &parse("class Point {\n  aye() { }\n}\n", 0).program);
        index.update_file(b.clone(), &parse("class Other {\n  bee() { }\n}\n", 0).program);

        assert!(index.has_class(&a, "Point"));
        assert!(!index.has_class(&b, "Point"), "b.ph does not declare Point");
        assert!(index.has_class(&b, "Other"));
        assert!(!index.has_class(&a, "Other"), "a.ph does not declare Other");
    }

    /// Removing one file's declaration leaves the other file's same-named class
    /// intact — the invalidation path is keyed too, not just the reads.
    #[test]
    fn removing_one_files_class_leaves_the_same_name_in_another_file() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        let b = uri("file:///b.ph");
        index.update_file(a.clone(), &parse("class Point {\n  aye() { }\n}\n", 0).program);
        index.update_file(b.clone(), &parse("class Point {\n  bee() { }\n}\n", 0).program);

        // a.ph is edited to drop the class entirely.
        index.update_file(a.clone(), &parse("let x = 1\n", 0).program);

        assert!(!index.has_class(&a, "Point"));
        assert!(index.has_class(&b, "Point"), "b.ph's Point must survive a.ph's edit");
        let from_b: Vec<String> = index.class_members(&b, "Point").into_iter().map(|m| m.selector).collect();
        assert_eq!(from_b, vec!["bee()".to_string()]);
    }

    #[test]
    fn reparse_replaces_only_the_changed_file_class_members() {
        let index = WorkspaceIndex::new();
        let a = uri("file:///a.ph");
        index.update_file(a.clone(), &parse("class A {\n  one() { }\n}\n", 0).program);
        assert!(index.class_members(&a, "A").iter().any(|m| m.selector == "one()"));

        index.update_file(a.clone(), &parse("class A {\n  two() { }\n}\n", 0).program);
        let members = index.class_members(&a, "A");
        assert!(members.iter().any(|m| m.selector == "two()"));
        assert!(!members.iter().any(|m| m.selector == "one()"));
    }

    #[test]
    fn pinned_method_ref_is_recorded_as_a_reference() {
        let src = "class Point {\n  move(_ x, to) { }\n}\n\nlet p = Point.new();\nlet f = p::#move(_,to);\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let index = WorkspaceIndex::new();
        index.update_file(uri("file:///a.ph"), &parsed.program);

        let refs = index.references("move(_,to)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].uri, uri("file:///a.ph"));
    }

    #[test]
    fn top_level_binding_at_offset_resolves_the_declaration_site() {
        let src = "let counter = Counter.new();\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let offset = src.find("counter").unwrap();
        assert_eq!(top_level_binding_at_offset(&parsed.program, offset), Some("counter".to_string()));
    }

    #[test]
    fn top_level_binding_at_offset_resolves_a_later_usage() {
        let src = "let counter = Counter.new();\ncounter.increment();\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let usage_offset = src.rfind("counter").unwrap();
        assert_eq!(top_level_binding_at_offset(&parsed.program, usage_offset), Some("counter".to_string()));
    }

    #[test]
    fn top_level_binding_at_offset_none_for_a_local_binding_inside_a_method() {
        // `counter` here is a local variable inside `Widget::run`, not a
        // top-level binding — must not resolve.
        let src = "class Widget {\n  run() {\n    let counter = 0;\n    counter;\n  }\n}\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let offset = src.rfind("counter").unwrap();
        assert_eq!(top_level_binding_at_offset(&parsed.program, offset), None);
    }

    #[test]
    fn top_level_binding_at_offset_none_for_unrelated_identifier() {
        let src = "let counter = Counter.new();\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let offset = src.find("Counter").unwrap();
        assert_eq!(top_level_binding_at_offset(&parsed.program, offset), None);
    }

    #[test]
    fn top_level_binding_occurrences_traverse_every_pack_item_position() {
        let src = "let xs = value\nlet label = key\ntarget(*xs)\ntarget([label]: xs)\nreceiver[***xs]\n";
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

        let star_xs = src.find("*xs").unwrap() + 1;
        let computed_label = src.find("[label]").unwrap() + 1;
        let labeled_value = src.find("[label]: xs").unwrap() + "[label]: ".len();
        let index_xs = src.rfind("***xs").unwrap() + 3;
        assert_eq!(top_level_binding_at_offset(&parsed.program, star_xs), Some("xs".to_string()));
        assert_eq!(top_level_binding_at_offset(&parsed.program, computed_label), Some("label".to_string()));
        assert_eq!(top_level_binding_at_offset(&parsed.program, labeled_value), Some("xs".to_string()));
        assert_eq!(top_level_binding_at_offset(&parsed.program, index_xs), Some("xs".to_string()));
    }
}
