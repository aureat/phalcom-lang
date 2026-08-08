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

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position, Url};

use phalcom_ast::ast::{ClassMember, Expr, ListLiteralExpr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, Pattern, ProductLabel, Program, SetLiteralEntry, Statement, TupleLiteralEntry};

use crate::core_table::{CoreTable, CoreVisibility, MemberKind};
use crate::documents::Document;
use crate::index::{ClassMemberInfo, MemberVisibility, WorkspaceIndex};

/// Whether a resolved receiver is an **instance** of its class or the
/// **class object** itself (a bare class-name reference used as a receiver,
/// e.g. `Cls.<cursor>`).
///
/// [`MemberKind::StaticMethod`]/[`MemberKind::Construct`] are class-side
/// members — dispatched on the class object, never on one of its instances —
/// while [`MemberKind::Method`]/[`MemberKind::Getter`]/[`MemberKind::Setter`]
/// are instance-side. Threading this alongside the resolved class name lets
/// [`completions`] offer each side only the members that side actually
/// understands, closing the "static/construct members over-offered on an
/// instance receiver" gap (`docs/forge/DEFERRED.md`, Stage 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverKind {
    /// The receiver is an instance of the resolved class (`x.<cursor>` where
    /// `x` holds an instance) — offer instance-side members only.
    Instance,
    /// The receiver is the class object itself (`Cls.<cursor>`, a class-side
    /// message send) — offer class-side (`static`/`construct`) members only.
    ClassObject,
}

/// Names the class of the receiver expression under a completion position,
/// and whether that receiver is an instance or the class object itself.
///
/// The pluggable seam of Stage 3: the completion handler depends only on this
/// trait, so a future type-inference pass can replace the local-dataflow
/// [`ConstructResolver`] without any change to
/// [`completions`] or the backend handler (plan P4).
pub trait ReceiverResolver {
    /// Returns the class name and [`ReceiverKind`] of the receiver at
    /// `position` in `doc`, or `None` if there is no receiver, or its type
    /// cannot be determined.
    ///
    /// A `None` result degrades completion to the full builtin surface — it
    /// never suppresses completions entirely.
    fn resolve(&self, doc: &Document, position: Position) -> Option<(String, ReceiverKind)>;
}

/// A local-dataflow [`ReceiverResolver`]: resolves a receiver's class from a
/// capitalized bare class-name reference, a `self` receiver, or a
/// `let x = Cls.new(...)` / `Cls.construct(...)` binding lexically visible at
/// the cursor.
///
/// Three receiver shapes, tried in order:
/// 1. **Capitalized bare name** (`Cls.<cursor>`) — the class-name convention
///    Phalcom uses everywhere else in this module; resolves directly to that
///    class as a [`ReceiverKind::ClassObject`], no binding lookup needed.
/// 2. **`self`** — resolves to the name of the class whose member body
///    encloses the cursor, as a [`ReceiverKind::Instance`]; `None` if the
///    cursor is not inside any class member.
/// 3. **An ordinary (lowercase) identifier** — resolves through
///    `resolve_var_class`'s lexical-scope walk to a [`ReceiverKind::
///    Instance`].
///
/// Scope/shadowing-aware (U-LSP Stage 3 DEFERRED "dataflow scope" follow-up):
/// the binding walk is bounded to the lexical scope chain enclosing the
/// cursor — the nearest enclosing method/getter/setter/@constructor body (or the
/// top level), plus every nested block/`for` it is itself nested in — never a
/// sibling scope, another method, or the rest of the document. Within that
/// chain, both a fresh `let`/`var` binding and a plain reassignment
/// (`x = OtherCls.new()`) update the tracked class, and only the binding
/// lexically **before** the cursor in a given scope is considered, so a
/// later-in-source binding never leaks backwards.
///
/// Still out of scope, left to a future resolver behind the same trait:
/// method/block **parameters** are not tracked (a parameter's class is never
/// known without call-site type information), and cross-file bindings are not
/// followed (this resolver only ever sees the one open [`Document`]).
pub struct ConstructResolver;

impl ReceiverResolver for ConstructResolver {
    fn resolve(&self, doc: &Document, position: Position) -> Option<(String, ReceiverKind)> {
        let offset = doc.line_index.offset(position);
        let (receiver, _partial) = receiver_prefix(&doc.text, offset)?;

        if receiver == "self" {
            let (_, enclosing_class) = enclosing_scope_chain(&doc.parse.program, offset);
            return enclosing_class.map(|class| (class, ReceiverKind::Instance));
        }
        if receiver.chars().next().is_some_and(char::is_uppercase) {
            return Some((receiver, ReceiverKind::ClassObject));
        }
        resolve_var_class(&doc.parse.program, &receiver, offset).map(|class| (class, ReceiverKind::Instance))
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

/// Finds the lexical scope chain enclosing `offset`, from outermost to
/// innermost, together with the name of the class the cursor's member body
/// (if any) is declared on.
///
/// The chain is bounded to the nearest enclosing method/getter/setter/
/// construct body — or the top level, if `offset` is not inside any class
/// member's body — and never crosses into a sibling member, another class, or
/// the rest of the document. Within that boundary it descends through every
/// nested block/`for` body that lexically contains `offset` (innermost last),
/// so [`resolve_var_class`]'s per-scope walk can implement proper shadowing:
/// the nearest enclosing scope's binding wins over an outer one.
///
/// The returned class name is `Some` whenever `offset` sits anywhere inside a
/// `class` declaration's range (not only inside one of its member bodies),
/// since that is also what a `self` receiver needs (`self` is meaningful
/// throughout a class's own member bodies).
fn enclosing_scope_chain(program: &Program, offset: usize) -> (Vec<&[Statement]>, Option<String>) {
    for statement in &program.statements {
        if let Statement::Class(class_def) = statement {
            if class_def.range.contains(offset) {
                for member in &class_def.members {
                    let body: Option<&[Statement]> = match member {
                        ClassMember::Method(m) if m.range.contains(offset) => Some(&m.body),
                        ClassMember::Getter(g) if g.range.contains(offset) => Some(&g.body),
                        ClassMember::Setter(s) if s.range.contains(offset) => Some(&s.body),
                        _ => None,
                    };
                    if let Some(body) = body {
                        let mut chain = vec![body];
                        descend_scope_chain(body, offset, &mut chain);
                        return (chain, Some(class_def.name.clone()));
                    }
                }
                // Inside the class but not inside any member's own body (e.g.
                // a field default, or between members) — no local scope, but
                // `self` is still meaningful for the class name.
                return (Vec::new(), Some(class_def.name.clone()));
            }
        }
    }
    let mut chain = vec![program.statements.as_slice()];
    descend_scope_chain(&program.statements, offset, &mut chain);
    (chain, None)
}

/// Defining lexical class at a document position, if any.
pub fn lexical_class_at(program: &Program, offset: usize) -> Option<String> {
    enclosing_scope_chain(program, offset).1
}

/// Extends `chain` with every nested block/`for` body that lexically contains
/// `offset`, recursing as deep as the AST goes (innermost pushed last).
fn descend_scope_chain<'a>(statements: &'a [Statement], offset: usize, chain: &mut Vec<&'a [Statement]>) {
    for statement in statements {
        if let Statement::For(f) = statement {
            if f.range.contains(offset) {
                chain.push(&f.body);
                descend_scope_chain(&f.body, offset, chain);
                return;
            }
        }
        if let Some(inner) = nested_block_in_statement(statement, offset) {
            chain.push(inner);
            descend_scope_chain(inner, offset, chain);
            return;
        }
    }
}

/// Finds a `{ … }` block body one expression-level deep inside `statement`
/// whose range contains `offset` (a block used as a `let`/`return` value, a
/// bare expression statement, or a `throw` operand).
fn nested_block_in_statement(statement: &Statement, offset: usize) -> Option<&[Statement]> {
    let expr = match statement {
        Statement::Let(binding) => binding.value.as_ref(),
        Statement::Return(r) => r.value.as_ref(),
        Statement::Expr { expr, .. } => Some(expr),
        Statement::Throw { expr, .. } => Some(expr),
        _ => None,
    }?;
    nested_block_in_expr(expr, offset)
}

/// Recurses into `expr` for a nested block body whose range contains
/// `offset`. Stops at the first (outermost) block found along the path — any
/// deeper nesting inside that block is picked up by [`descend_scope_chain`]'s
/// own recursion into the returned body.
fn nested_block_in_expr(expr: &Expr, offset: usize) -> Option<&[Statement]> {
    if !expr.range().contains(offset) {
        return None;
    }
    match expr {
        Expr::Block(b) => Some(&b.body),
        Expr::Assignment(a) => nested_block_in_expr(&a.name, offset).or_else(|| nested_block_in_expr(&a.value, offset)),
        Expr::Range(range) => range
            .lower
            .as_ref()
            .and_then(|lower| nested_block_in_expr(lower, offset))
            .or_else(|| range.upper.as_ref().and_then(|upper| nested_block_in_expr(upper, offset))),
        Expr::Unary(u) => nested_block_in_expr(&u.expr, offset),
        Expr::Binary(b) => nested_block_in_expr(&b.left, offset).or_else(|| nested_block_in_expr(&b.right, offset)),
        Expr::UnqualifiedCall(m) => m.args.iter().find_map(|a| nested_block_in_expr(&a.expr, offset)),
        Expr::MethodCall(m) => nested_block_in_expr(&m.object, offset).or_else(|| m.args.iter().find_map(|a| nested_block_in_expr(&a.expr, offset))),
        Expr::GetProperty(g) => nested_block_in_expr(&g.object, offset),
        Expr::SetProperty(s) => nested_block_in_expr(&s.object, offset).or_else(|| nested_block_in_expr(&s.value, offset)),
        Expr::Index(i) => nested_block_in_expr(&i.object, offset).or_else(|| i.args.iter().find_map(|a| nested_block_in_expr(&a.expr, offset))),
        Expr::SetIndex(si) => nested_block_in_expr(&si.object, offset)
            .or_else(|| si.args.iter().find_map(|a| nested_block_in_expr(&a.expr, offset)))
            .or_else(|| nested_block_in_expr(&si.value, offset)),
        Expr::MethodRef(mr) => nested_block_in_expr(&mr.receiver, offset),
        Expr::TupleLiteral(tuple) => tuple.entries.iter().find_map(|entry| match entry {
            TupleLiteralEntry::Positional { expr, .. } => nested_block_in_expr(expr, offset),
            TupleLiteralEntry::Labeled { label, value, .. } => nested_block_in_product_label(label, offset).or_else(|| nested_block_in_expr(value, offset)),
        }),
        Expr::RecordLiteral(record) => record
            .fields
            .iter()
            .find_map(|field| nested_block_in_product_label(&field.label, offset).or_else(|| nested_block_in_expr(&field.value, offset))),
        Expr::MapLiteral(map) => map.entries.iter().find_map(|entry| match entry {
            MapLiteralEntry::Association { key, value, .. } => {
                let key_block = match key {
                    MapLiteralKey::BareSymbol { .. } => None,
                    MapLiteralKey::Computed { expr, .. } => nested_block_in_expr(expr, offset),
                };
                key_block.or_else(|| nested_block_in_expr(value, offset))
            }
            MapLiteralEntry::Expansion { expr, .. } => nested_block_in_expr(expr, offset),
        }),
        Expr::SetLiteral(set) => set.entries.iter().find_map(|entry| match entry {
            SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => nested_block_in_expr(expr, offset),
        }),
        Expr::ListLiteral(list) => list.elements.iter().find_map(|element| match element {
            ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => nested_block_in_expr(expr, offset),
        }),
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Var { .. }
        | Expr::Field { .. }
        | Expr::ImplementationSelector { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::Symbol(_) => None,
    }
}

fn nested_block_in_product_label(label: &ProductLabel, offset: usize) -> Option<&[Statement]> {
    match label {
        ProductLabel::Static { .. } => None,
        ProductLabel::Computed { expr, .. } => nested_block_in_expr(expr, offset),
    }
}

/// Resolves the class bound to `target` at `offset` by walking
/// [`enclosing_scope_chain`]'s scopes from innermost to outermost — the
/// nearest enclosing scope's binding shadows an outer one, and neither a
/// sibling scope nor another class member is ever consulted.
fn resolve_var_class(program: &Program, target: &str, offset: usize) -> Option<String> {
    let (chain, _enclosing_class) = enclosing_scope_chain(program, offset);
    chain.iter().rev().find_map(|scope| resolve_in_scope(scope, target, offset))
}

/// Scans one scope's own top-level statements — never descending into a
/// nested block/`for` body, since those are separate entries in
/// [`enclosing_scope_chain`]'s chain — for the class of the *last*
/// `let`/`var` construct-binding or plain reassignment (`target =
/// OtherCls.new()`) of `target` whose own range starts strictly before
/// `offset`. A later-in-source binding (at or after `offset`) is never
/// considered, so a rebind written after the cursor cannot leak backwards.
fn resolve_in_scope(statements: &[Statement], target: &str, offset: usize) -> Option<String> {
    let mut result = None;
    for statement in statements {
        let start = match statement {
            Statement::Let(b) => b.range.start,
            Statement::Return(r) => r.range.start,
            Statement::Expr { range, .. } => range.start,
            Statement::For(f) => f.range.start,
            Statement::Throw { range, .. } => range.start,
            Statement::Break { range } | Statement::Continue { range } => range.start,
            Statement::Import(i) => i.range.start,
            Statement::Class(c) => c.range.start,
        };
        // Statements are in monotonically increasing source order; once one
        // starts at or after `offset`, nothing later in this scope is
        // lexically before the cursor either.
        if start >= offset {
            break;
        }
        match statement {
            Statement::Let(binding) => {
                if let Pattern::Name { name, .. } = &binding.pattern {
                    if name == target {
                        result = binding.value.as_ref().and_then(class_of_construct);
                    }
                }
            }
            Statement::Expr {
                expr: Expr::Assignment(assignment),
                ..
            } => {
                if let Expr::Var { value: name, .. } = assignment.name.as_ref() {
                    if name == target {
                        result = class_of_construct(&assignment.value);
                    }
                }
            }
            _ => {}
        }
    }
    result
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

/// Builds the completion item list for a (possibly unresolved) receiver class
/// and [`ReceiverKind`].
///
/// - `Some((class, kind))` and the class has known members: that class's own
///   members plus, walking [`WorkspaceIndex::class_parent`], its user-class
///   ancestors' members up to (and including) the first builtin ancestor
///   whose members come from [`CoreTable`] — de-duplicated by selector,
///   most-derived winning — then filtered by `kind` (see
///   `filter_by_receiver_kind`) so instance and class-object receivers see
///   only their matching placement side.
/// - `Some((class, _))` but no members resolve at all, or `None`: the full
///   builtin surface ([`CoreTable::all_members`]), unfiltered — the graceful
///   "receiver type unknown" fallback (never worse than pre-Stage-3
///   completion).
///
/// Items are returned sorted by label for a deterministic order.
pub fn completions(
    resolved: Option<(&str, ReceiverKind)>,
    lexical_class: Option<&str>,
    privileged: bool,
    uri: &Url,
    index: &WorkspaceIndex,
    table: &CoreTable,
) -> Vec<CompletionItem> {
    let (members, kind_opt) = match resolved {
        Some((class, kind)) => {
            let collected = collect_class_members(class, uri, index, table);
            if collected.is_empty() {
                (table_members(table), Some(kind))
            } else {
                (filter_by_receiver_kind(collected, kind), Some(kind))
            }
        }
        None => (table_members(table), None),
    };

    let mut items: Vec<CompletionItem> = members
        .iter()
        .filter(|member| match member.visibility {
            MemberVisibility::Public => true,
            MemberVisibility::Private => lexical_class == Some(member.owner.as_str()),
            MemberVisibility::Protected => lexical_class.is_some_and(|caller| index.is_same_or_subclass(uri, caller, &member.owner)),
            MemberVisibility::Internal => privileged,
        })
        .map(to_completion_item)
        .collect();

    // Every class object implicitly has `new()` (Class#new) — guarantee it is
    // offered even when the builtin ancestor chain does not walk up to `Class`.
    if kind_opt == Some(ReceiverKind::ClassObject) {
        if !items.iter().any(|item| item.label == "new()") {
            items.push(CompletionItem {
                label: "new()".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                insert_text: Some("new()".to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..CompletionItem::default()
            });
        }
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}


/// Completion for an editor position, including implicit-self members and
/// visible bare bindings when no explicit `receiver.` prefix is present.
pub fn contextual_completions(
    resolved: Option<(&str, ReceiverKind)>,
    program: &Program,
    text: &str,
    offset: usize,
    privileged: bool,
    uri: &Url,
    index: &WorkspaceIndex,
    table: &CoreTable,
) -> Vec<CompletionItem> {
    let lexical_class = lexical_class_at(program, offset);
    let mut items = completions(resolved, lexical_class.as_deref(), privileged, uri, index, table);
    if receiver_prefix(text, offset).is_none() {
        if let Some(class) = lexical_class.as_deref() {
            let self_members = filter_by_receiver_kind(collect_class_members(class, uri, index, table), ReceiverKind::Instance);
            items.extend(
                self_members
                    .iter()
                    .filter(|member| match member.visibility {
                        MemberVisibility::Public => true,
                        MemberVisibility::Private => member.owner == class,
                        MemberVisibility::Protected => index.is_same_or_subclass(uri, class, &member.owner),
                        MemberVisibility::Internal => privileged,
                    })
                    .map(to_completion_item),
            );
        }
        items.extend(visible_names_at(program, offset).into_iter().map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            insert_text: Some(name),
            ..CompletionItem::default()
        }));
    }
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);
    items
}

fn visible_names_at(program: &Program, offset: usize) -> Vec<String> {
    let mut names = Vec::new();
    for statement in &program.statements {
        match statement {
            Statement::Class(class) => names.push(class.name.clone()),
            Statement::Import(import) => names.push(import.binding.clone()),
            Statement::Let(binding) if binding.range.start < offset => collect_pattern_names(&binding.pattern, &mut names),
            _ => {}
        }
    }

    for statement in &program.statements {
        let Statement::Class(class) = statement else { continue };
        if !class.range.contains(offset) {
            continue;
        }
        for member in &class.members {
            match member {
                ClassMember::Method(method) if method.range.contains(offset) => {
                    names.extend(method.params.iter().map(|param| param.name.clone()));
                }
                ClassMember::Setter(setter) if setter.range.contains(offset) => names.push(setter.param.name.clone()),
                ClassMember::Index(index) if index.range.contains(offset) => {
                    names.extend(index.params.iter().map(|param| param.name.clone()));
                    if let phalcom_ast::ast::IndexAccessor::Set { put } = &index.accessor {
                        names.push(put.name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    let (chain, _) = enclosing_scope_chain(program, offset);
    for scope in chain {
        for statement in scope {
            match statement {
                Statement::Let(binding) if binding.range.start < offset => collect_pattern_names(&binding.pattern, &mut names),
                Statement::For(for_stmt) if for_stmt.range.contains(offset) => names.push(for_stmt.binding.clone()),
                _ => {}
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn collect_pattern_names(pattern: &Pattern, out: &mut Vec<String>) {
    match pattern {
        Pattern::Name { name, .. } => out.push(name.clone()),
        Pattern::Tuple { elements, .. } => {
            for element in elements {
                collect_pattern_names(element, out);
            }
        }
        Pattern::List { elements, rest, .. } => {
            for element in elements {
                collect_pattern_names(element, out);
            }
            if let Some(rest) = rest {
                collect_pattern_names(rest, out);
            }
        }
    }
}

/// Filters `members` down to the ones dispatchable on a receiver of `kind`:
/// an [`ReceiverKind::Instance`] receiver keeps only instance-side members;
/// a [`ReceiverKind::ClassObject`] receiver keeps only class-side members.
fn filter_by_receiver_kind(members: Vec<ClassMemberInfo>, kind: ReceiverKind) -> Vec<ClassMemberInfo> {
    members
        .into_iter()
        .filter(|member| match kind {
            ReceiverKind::Instance => !member.is_class_side,
            ReceiverKind::ClassObject => member.is_class_side,
        })
        .collect()
}

/// The full builtin surface as [`ClassMemberInfo`]s.
fn table_members(table: &CoreTable) -> Vec<ClassMemberInfo> {
    table
        .all_members()
        .into_iter()
        .map(|m| {
            let visibility = if m.selector.starts_with("_$") {
                MemberVisibility::Internal
            } else {
                core_visibility(m.visibility)
            };
            ClassMemberInfo {
                selector: m.selector,
                kind: m.kind,
                owner: String::new(),
                visibility,
                is_class_side: m.class_side || matches!(m.kind, MemberKind::StaticMethod | MemberKind::Construct),
            }
        })
        .collect()
}

/// The class every user class implicitly is when it writes no superclass
/// clause (object-model.md §5.1).
const IMPLICIT_ROOT_CLASS: &str = "Object";

/// Collects the members visible on `class`: its own, plus inherited members
/// up the `is` superclass chain, stopping at (and including) the first builtin
/// ancestor.
///
/// A user class's parent defaults to [`IMPLICIT_ROOT_CLASS`] whenever
/// [`WorkspaceIndex::class_parent`] reports no explicit `is` superclass — both for
/// a class that never wrote one, and for a chain that ends at a user class
/// with no further superclass — so `Object`'s selectors (`==`, `hash`,
/// `isNil`, …) are always eventually walked, closing the "implicit `Object`
/// never offered" gap (`docs/forge/DEFERRED.md`, Stage 3).
///
/// Builtin superclass chains beyond that one implicit step are not encoded in
/// `core-table.json`, so a builtin ancestor (`Object` included) contributes
/// only its own listed members and terminates the walk. A cycle guard bounds
/// the walk defensively against a malformed (self-inheriting) chain, and also
/// prevents `Object` from being visited twice if a chain already reaches it
/// explicitly.
///
/// # The whole walk stays inside `uri`
///
/// Every user-class hop resolves against `uri` — the file the completion was
/// requested in. That is not a simplification, it is the semantics: a class is
/// identified by `(module, name)` (PDR-0001), a file is a module (ADR-0045),
/// and this crate never resolves `import`. A superclass declared in *another*
/// file is therefore not nameable from here, so the walk falls through to
/// [`CoreTable`] and terminates rather than guessing.
///
/// This is a deliberate behavior change. The previous name-keyed index would
/// happily answer with some *other* file's class of the same name — which is
/// how `class Dog is Animal` in one file used to pick up an unrelated
/// `Animal` from another. Losing that is the fix, not a regression.
fn collect_class_members(class: &str, uri: &Url, index: &WorkspaceIndex, table: &CoreTable) -> Vec<ClassMemberInfo> {
    let mut out: Vec<ClassMemberInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut guard = std::collections::HashSet::new();
    let mut current = Some(class.to_string());
    // Whether the walk ever resolved a user class in `uri`. Distinguishes
    // "this receiver's class is unknown entirely" (return empty, so
    // `completions` falls back to the *full* builtin surface) from "we are
    // mid-chain and hit an unresolvable parent" (fall back to `Object`).
    // Collapsing the two costs the unknown-receiver case its full surface.
    let mut walked_user_class = false;

    while let Some(name) = current {
        if !guard.insert(name.clone()) {
            break;
        }
        if index.has_class(uri, &name) {
            walked_user_class = true;
            for member in index.class_members(uri, &name) {
                if seen.insert(member.selector.clone()) {
                    out.push(member);
                }
            }
            current = Some(index.class_parent(uri, &name).unwrap_or_else(|| IMPLICIT_ROOT_CLASS.to_string()));
        } else if let Some(members) = table.class_members(&name) {
            for member in members {
                if seen.insert(member.selector.clone()) {
                    out.push(ClassMemberInfo {
                        selector: member.selector.clone(),
                        kind: member.kind,
                        owner: name.clone(),
                        visibility: if member.selector.starts_with("_$") {
                            MemberVisibility::Internal
                        } else {
                            core_visibility(member.visibility)
                        },
                        is_class_side: member.class_side || matches!(member.kind, MemberKind::StaticMethod | MemberKind::Construct),
                    });
                }
            }
            current = None;
        } else if walked_user_class && name != IMPLICIT_ROOT_CLASS {
            // A *parent* that resolves to neither a class declared in this
            // file nor a builtin. Two ways to get here: a superclass declared
            // in another file (not nameable from here — this crate never
            // resolves `import`), or a plain typo.
            //
            // `walked_user_class` is what keeps this from swallowing the
            // unknown-receiver case: if the *starting* class resolved to
            // nothing we must return empty, so `completions` serves the full
            // builtin surface rather than just `Object`'s handful.
            //
            // Fall back to the implicit root rather than terminating, so this
            // function's stated contract — `Object`'s selectors are *always*
            // eventually walked — survives an unresolvable parent. Without
            // this, file-scoping the index (PDR-0001) would silently drop the
            // whole builtin surface for every class whose `extends` names
            // something outside its own file, which is a strictly worse
            // completion than before the fix.
            //
            // `guard` makes this terminate: if the chain already visited
            // `Object`, the next iteration breaks immediately.
            current = Some(IMPLICIT_ROOT_CLASS.to_string());
        } else {
            current = None;
        }
    }
    out
}

fn core_visibility(visibility: CoreVisibility) -> MemberVisibility {
    match visibility {
        CoreVisibility::Public => MemberVisibility::Public,
        CoreVisibility::Private => MemberVisibility::Private,
        CoreVisibility::Protected => MemberVisibility::Protected,
        CoreVisibility::Internal => MemberVisibility::Internal,
    }
}

/// Renders one class member as a snippet [`CompletionItem`].
///
/// The label is the ADR-0012 comma-form selector; the insert text is a
/// snippet reflecting it (see [`method_snippet`] / [`setter_snippet`]).
fn to_completion_item(member: &ClassMemberInfo) -> CompletionItem {
    let (kind, insert_text, format) = match member.kind {
        MemberKind::Getter => (CompletionItemKind::PROPERTY, member.selector.clone(), InsertTextFormat::PLAIN_TEXT),
        MemberKind::Setter => (CompletionItemKind::PROPERTY, setter_snippet(&member.selector), InsertTextFormat::SNIPPET),
        // Methods, static methods, and constructs all render as a
        // parenthesized argument-slot snippet.
        _ => (CompletionItemKind::METHOD, method_snippet(&member.selector), InsertTextFormat::SNIPPET),
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
    let inner = selector[open + 1..].strip_suffix(')').unwrap_or(&selector[open + 1..]);
    if inner.is_empty() {
        return format!("{name}()");
    }
    let slots: Vec<String> = inner
        .split(',')
        .enumerate()
        .map(|(i, slot)| {
            let n = i + 1;
            if slot == "_" { format!("${{{n}:_}}") } else { format!("{slot}: ${{{n}:_}}") }
        })
        .collect();
    format!("{name}({})", slots.join(", "))
}

/// Renders a setter selector's snippet insert text.
///
/// `x=(put)` becomes `x = ${1:value}` — a bare-name write with a single
/// tab-stop, no parens. Falls back to the raw selector if it is not in the
/// expected `name=(put)` shape.
fn setter_snippet(selector: &str) -> String {
    match selector.strip_suffix("=(put)") {
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

    /// The file every single-file completion test indexes into and queries
    /// from. Completion is now file-scoped, so the query URI must match the
    /// one `index_with` was given or the class resolves to nothing.
    fn a_uri() -> Url {
        Url::parse("file:///a.ph").unwrap()
    }

    #[test]
    fn receiver_prefix_reads_receiver_and_partial() {
        assert_eq!(receiver_prefix("m.mov", 5), Some(("m".to_string(), "mov".to_string())));
        assert_eq!(receiver_prefix("foo.", 4), Some(("foo".to_string(), String::new())));
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
        assert_eq!(ConstructResolver.resolve(&doc, pos), Some(("Mover".to_string(), ReceiverKind::Instance)));
    }

    #[test]
    fn construct_resolver_none_for_lowercase_receiver_call() {
        let src = "let c = counter.next();\nc.\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 1, character: 2 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), None);
    }

    #[test]
    fn construct_resolver_bare_capitalized_receiver_is_class_object() {
        let src = "Mover.\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 0, character: 6 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), Some(("Mover".to_string(), ReceiverKind::ClassObject)));
    }

    #[test]
    fn construct_resolver_self_resolves_to_enclosing_class_as_instance() {
        // Cursor sits right after the `.`, before `size` — the shape
        // `receiver_prefix` needs to read a complete, parseable `self.size`
        // access rather than a dangling `self.` (which the parser cannot
        // recover a full method body from).
        let src = "class Mover {\n  move() {\n    self.size\n  }\n}\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 2, character: 9 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), Some(("Mover".to_string(), ReceiverKind::Instance)));
    }

    #[test]
    fn construct_resolver_self_outside_any_class_is_none() {
        let src = "self.\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 0, character: 5 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), None);
    }

    #[test]
    fn construct_resolver_reassignment_wins_over_earlier_binding() {
        // `m` is first bound to a `Car`, then reassigned to a `Mover` before
        // the completion point — the reassignment must win.
        let src = "let m = Car.new();\nm = Mover.new();\nm.\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 2, character: 2 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), Some(("Mover".to_string(), ReceiverKind::Instance)));
    }

    #[test]
    fn construct_resolver_ignores_binding_lexically_after_the_cursor() {
        // The `Mover` rebind happens *after* the completion point; only the
        // original `Car` binding is visible there.
        let src = "let m = Car.new();\nm.\nm = Mover.new();\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 1, character: 2 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), Some(("Car".to_string(), ReceiverKind::Instance)));
    }

    #[test]
    fn construct_resolver_sibling_method_binding_does_not_leak() {
        // `m` inside `other()` is a completely separate scope from the `m.`
        // completion inside `move()` — the sibling's binding must not apply.
        // Cursor sits right after the `.`, before `next`, so the source stays
        // a fully parseable `m.next` access (a dangling `m.` at buffer end
        // does not recover a usable method-body range, see the `self`
        // resolver test's note).
        let src = "class Mover {\n  other() {\n    let m = Truck.new();\n  }\n  move() {\n    m.next\n  }\n}\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 5, character: 6 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), None);
    }

    #[test]
    fn construct_resolver_nested_block_binding_does_not_leak_to_outer_scope() {
        // The `Truck` binding lives only inside the nested block; the outer
        // `m.` after the block still sees the outer `Car` binding.
        let src = "let m = Car.new();\nlet nested = || {\n  let m = Truck.new();\n};\nm.\n";
        let doc = Document::new(src.to_string());
        let pos = Position { line: 4, character: 2 };
        assert_eq!(ConstructResolver.resolve(&doc, pos), Some(("Car".to_string(), ReceiverKind::Instance)));
    }

    #[test]
    fn method_snippet_positional_and_labeled_slots() {
        assert_eq!(method_snippet("move(_,to,duration)"), "move(${1:_}, to: ${2:_}, duration: ${3:_})");
        assert_eq!(method_snippet("reset()"), "reset()");
        assert_eq!(method_snippet("at(_,put)"), "at(${1:_}, put: ${2:_})");
    }

    #[test]
    fn setter_snippet_renders_bare_name_write() {
        assert_eq!(setter_snippet("x=(put)"), "x = ${1:value}");
    }

    #[test]
    fn completions_for_resolved_user_class_only_that_class() {
        let index = index_with("file:///a.ph", "class Point {\n  move(_ x, to) { }\n  size { }\n}\n");
        let items = completions(Some(("Point", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"move(_,to)"));
        assert!(labels.contains(&"size"));
        // A builtin-only selector must NOT leak in for a resolved user class.
        assert!(!labels.contains(&"ifTrue(_)"));
    }

    #[test]
    fn completions_walk_user_superclass_chain() {
        let index = index_with("file:///a.ph", "class Animal {\n  eat() { }\n}\nclass Dog is Animal {\n  bark() { }\n}\n");
        let items = completions(Some(("Dog", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"bark()"));
        assert!(labels.contains(&"eat()"));
    }

    #[test]
    fn completions_walk_implicit_object_parent_when_no_extends() {
        // `Point` writes no `extends` clause; its implicit parent is `Object`,
        // whose own members (`hash`, `toString`, …) must be offered too.
        let index = index_with("file:///a.ph", "class Point {\n  size { }\n}\n");
        let items = completions(Some(("Point", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"size"));
        assert!(labels.contains(&"hash"), "{labels:#?}");
        assert!(labels.contains(&"toString"), "{labels:#?}");
        // `Object.new()` is class-side (`static-method`) — must not appear on
        // an instance receiver.
        assert!(!labels.contains(&"new()"), "{labels:#?}");
    }

    #[test]
    fn completions_walk_implicit_object_parent_past_a_user_superclass_chain() {
        // `Animal` also writes no `is` superclass, so `Dog`'s walk must reach
        // `Object` past `Animal`, not stop at `Animal`.
        let index = index_with("file:///a.ph", "class Animal {\n  eat() { }\n}\nclass Dog is Animal {\n  bark() { }\n}\n");
        let items = completions(Some(("Dog", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"bark()"));
        assert!(labels.contains(&"eat()"));
        assert!(labels.contains(&"hash"), "{labels:#?}");
    }

    #[test]
    fn completions_for_builtin_class() {
        let index = WorkspaceIndex::new();
        let items = completions(Some(("Bool", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        assert!(!items.is_empty());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"ifTrue(_)"));
    }

    #[test]
    fn completions_instance_receiver_excludes_static_and_construct_members() {
        // `Object` carries both an instance getter (`name`) and class-side
        // members (`new()` static, per `core-table.json`); an instance
        // receiver must drop the class-side ones.
        let index = WorkspaceIndex::new();
        let items = completions(Some(("Object", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"name"), "{labels:#?}");
        assert!(!labels.contains(&"new()"), "{labels:#?}");
    }

    #[test]
    fn completions_class_object_receiver_offers_only_static_and_construct_members() {
        let index = WorkspaceIndex::new();
        let items = completions(Some(("Object", ReceiverKind::ClassObject)), None, false, &a_uri(), &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"new()"), "{labels:#?}");
        // Instance-side members must not leak onto the class-object side.
        assert!(!labels.contains(&"name"), "{labels:#?}");
        assert!(!labels.contains(&"hash"), "{labels:#?}");
    }

    #[test]
    fn completions_class_object_receiver_offers_user_class_method_only() {
        // A user class with an `@class` method and an instance method: the
        // class-object receiver must see only the class-side one.
        let index = index_with("file:///a.ph", "class Counter {\n  @class\n  make() { }\n  next() { }\n}\n");
        let items = completions(
            Some(("Counter", ReceiverKind::ClassObject)),
            None,
            false,
            &a_uri(),
            &index,
            CoreTable::bundled(),
        );
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"make()"), "{labels:#?}");
        assert!(!labels.contains(&"next()"), "{labels:#?}");
    }

    #[test]
    fn class_side_getter_keeps_getter_snippet_shape() {
        let index = index_with("file:///a.ph", "class Counter {\n  @class\n  count => 1\n}\n");
        let items = completions(
            Some(("Counter", ReceiverKind::ClassObject)),
            None,
            false,
            &a_uri(),
            &index,
            CoreTable::bundled(),
        );
        let count = items.iter().find(|item| item.label == "count").expect("class getter completion");
        assert_eq!(count.insert_text.as_deref(), Some("count"));
    }

    #[test]
    fn visibility_filters_private_protected_and_internal_members() {
        let index = index_with(
            "file:///a.ph",
            "class Base {\n  @private\n  secret() {}\n  @protected\n  family() {}\n}\nclass Child is Base {\n  own() {}\n}\n",
        );
        let outside = completions(Some(("Base", ReceiverKind::Instance)), None, false, &a_uri(), &index, CoreTable::bundled());
        assert!(!outside.iter().any(|item| item.label == "secret()" || item.label == "family()"));

        let base = completions(Some(("Base", ReceiverKind::Instance)), Some("Base"), false, &a_uri(), &index, CoreTable::bundled());
        assert!(base.iter().any(|item| item.label == "secret()"));
        assert!(base.iter().any(|item| item.label == "family()"));

        let child = completions(Some(("Child", ReceiverKind::Instance)), Some("Child"), false, &a_uri(), &index, CoreTable::bundled());
        assert!(!child.iter().any(|item| item.label == "secret()"));
        assert!(child.iter().any(|item| item.label == "family()"));
        assert!(!child.iter().any(|item| item.label.starts_with("_$")));
    }

    /// Completion for `Point` in `a.ph` must not offer `b.ph`'s same-named
    /// class's members.
    ///
    /// **Negative control:** fails on the pre-collapse index, which merged both
    /// files' entries under the bare name `"Point"`.
    #[test]
    fn completions_do_not_leak_a_same_named_class_from_another_file() {
        let index = WorkspaceIndex::new();
        let a = Url::parse("file:///a.ph").unwrap();
        let b = Url::parse("file:///b.ph").unwrap();
        index.update_file(a.clone(), &phalcom_ast::parser::parse("class Point {\n  aye() { }\n}\n", 0).program);
        index.update_file(b.clone(), &phalcom_ast::parser::parse("class Point {\n  bee() { }\n}\n", 0).program);

        let items = completions(Some(("Point", ReceiverKind::Instance)), None, false, &a, &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"aye()"), "{labels:#?}");
        assert!(!labels.contains(&"bee()"), "b.ph's member leaked: {labels:#?}");
    }

    /// A superclass declared in *another* file is not nameable from here, so
    /// the walk stops rather than guessing. Deliberate behavior change: the
    /// old name-keyed index would resolve it and offer the wrong members.
    #[test]
    fn inheritance_walk_does_not_cross_files() {
        let index = WorkspaceIndex::new();
        let a = Url::parse("file:///a.ph").unwrap();
        let b = Url::parse("file:///b.ph").unwrap();
        index.update_file(a.clone(), &phalcom_ast::parser::parse("class Dog is Animal {\n  bark() { }\n}\n", 0).program);
        index.update_file(b.clone(), &phalcom_ast::parser::parse("class Animal {\n  eat() { }\n}\n", 0).program);

        let items = completions(Some(("Dog", ReceiverKind::Instance)), None, false, &a, &index, CoreTable::bundled());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"bark()"), "{labels:#?}");
        assert!(
            !labels.contains(&"eat()"),
            "b.ph's Animal is a different class and must not be walked: {labels:#?}"
        );
        // The walk still terminates through the implicit Object root, so the
        // builtin surface is present — this is a scoping fix, not a regression
        // to "no completions".
        assert!(labels.contains(&"hash"), "implicit Object walk still runs: {labels:#?}");
    }

    #[test]
    fn completions_unknown_receiver_falls_back_to_full_builtin_surface() {
        let index = WorkspaceIndex::new();
        let none = completions(None, None, false, &a_uri(), &index, CoreTable::bundled());
        assert!(!none.is_empty());
        // Unresolvable class name also falls back rather than returning empty.
        let unknown = completions(
            Some(("Nonexistent", ReceiverKind::Instance)),
            None,
            false,
            &a_uri(),
            &index,
            CoreTable::bundled(),
        );
        assert_eq!(none.len(), unknown.len());
    }

    #[test]
    fn completions_are_sorted_by_label() {
        let index = WorkspaceIndex::new();
        let items = completions(None, None, false, &a_uri(), &index, CoreTable::bundled());
        for pair in items.windows(2) {
            assert!(pair[0].label <= pair[1].label);
        }
    }
}
