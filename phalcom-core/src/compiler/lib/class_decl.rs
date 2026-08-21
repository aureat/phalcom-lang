use crate::bytecode::Bytecode;
use crate::compiler::attributes::{AttributeRegistry, CompileMode, ExpandCtx, expand_class_attributes};
use crate::heap::Object;
use crate::interner::Symbol;
use crate::method::{
    MemberVisibility, MethodKind, MethodObject, RestLayout, RestMode as RuntimeRestMode, SignatureKind, encode_label_component, encode_selector, make_signature,
};
use crate::value::Value;
use crate::vm::ClassKey;
use indexmap::IndexMap;
use phalcom_ast::ast::{
    AttrKind, Attribute, BuiltinAttr, ClassDef, ClassMember, ClosureParameters, Expr, IndexAccessor, ListLiteralElement, MapLiteralEntry, MapLiteralKey,
    MethodCallExpr, PackItem, PackLabel, RestMode, SetLiteralEntry, Statement,
};
use phalcom_common::range::SourceRange;

use super::checked_send_arity;
use super::error::{CompilerError, RestDeclarationErrorKind};
use super::{Compiler, UnitKind};

/// Attribute names handled entirely by the compiler itself — the guard weave
/// (`@requires`/`@ensures`/`@invariant`), the field-derive (`@construct`,
/// `@data`, U-ANNOT-LAYOUT), or the closed-hierarchy bookkeeping (`@sealed`)
/// — that carry no matching `Attribute`-subclass runtime instance and
/// therefore emit no `Name.new(args)`/`__attach` codegen (M-ATTR-ROOT,
/// `attribute-classes.md` §"What the compiler lowers"). `@On` is *not* in
/// this set: it names an ordinary `Attribute` subclass instance (retained via
/// `__attach` like any other) even though its own expansion is also a
/// compiler-side no-op (see `compiler::attributes::OnExpander`). `@variant`
/// never reaches this set at all — it is stripped from `class.members`
/// entirely by `compiler::attributes::expand_variants` before this point, so
/// it can never appear in `class_level_attrs`.
const COMPILER_ONLY_ATTRS: &[&str] = &[
    "requires",
    "ensures",
    "invariant",
    "construct",
    "constructor",
    "class",
    "data",
    "sealed",
    "private",
    "protected",
    "__synthetic",
];

fn member_visibility(name: Option<&str>, attributes: &[Attribute]) -> MemberVisibility {
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if attributes.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Private))) {
        MemberVisibility::Private
    } else if attributes.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Protected))) {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}

/// Validates F.3 method rest declarations after attribute expansion.
fn validate_rest_usage(member: &ClassMember) -> Result<(), CompilerError> {
    match member {
        ClassMember::Method(method) => {
            let is_constructor = method.is_constructor
                || method
                    .attributes
                    .iter()
                    .any(|attribute| matches!(attribute.kind, AttrKind::Builtin(BuiltinAttr::Constructor)) || attribute.name == "constructor");
            if is_constructor {
                if let Some(rest) = method.params.iter().find(|param| param.is_rest()) {
                    // F.3 leaves constructor/factory rest capture outside its
                    // method-body ABI. Reject before selector creation and
                    // method installation until that scope is specified.
                    return Err(CompilerError::RestModeUnsupportedForMember(rest.range));
                }
                return Ok(());
            }
            let positional: Vec<_> = method.params.iter().filter(|p| p.rest_mode == RestMode::Positional).collect();
            let labeled: Vec<_> = method.params.iter().filter(|p| p.rest_mode == RestMode::Labeled).collect();
            let complete: Vec<_> = method.params.iter().filter(|p| p.rest_mode == RestMode::Complete).collect();
            if positional.len() > 1 {
                return Err(CompilerError::InvalidRestDeclaration {
                    kind: RestDeclarationErrorKind::DuplicatePositional,
                    span: positional[1].range,
                });
            }
            if labeled.len() > 1 {
                return Err(CompilerError::InvalidRestDeclaration {
                    kind: RestDeclarationErrorKind::DuplicateLabeled,
                    span: labeled[1].range,
                });
            }
            if complete.len() > 1 {
                return Err(CompilerError::InvalidRestDeclaration {
                    kind: RestDeclarationErrorKind::DuplicateComplete,
                    span: complete[1].range,
                });
            }
            if !complete.is_empty() && (!positional.is_empty() || !labeled.is_empty()) {
                return Err(CompilerError::InvalidRestDeclaration {
                    kind: RestDeclarationErrorKind::CompleteConflict,
                    span: complete[0].range,
                });
            }
            if let Some(rest) = labeled.first().or_else(|| complete.first()) {
                if method.params.last().is_none_or(|p| p.range != rest.range) {
                    return Err(CompilerError::InvalidRestDeclaration {
                        kind: RestDeclarationErrorKind::TerminalRestNotLast,
                        span: rest.range,
                    });
                }
            }
        }
        ClassMember::Index(index) => {
            if let Some(rest) = index.params.iter().find(|param| param.is_rest()) {
                return Err(CompilerError::RestModeUnsupportedForMember(rest.range));
            }
        }
        ClassMember::Getter(_) | ClassMember::Setter(_) | ClassMember::Field(_) | ClassMember::Variant(_) => {}
    }
    Ok(())
}

fn rest_layout(params: &[phalcom_ast::ast::ParameterDef], interner: &mut crate::interner::Interner) -> Option<RestLayout> {
    let positional = params.iter().position(|p| p.rest_mode == RestMode::Positional);
    let labeled = params.iter().position(|p| p.rest_mode == RestMode::Labeled);
    let complete = params.iter().position(|p| p.rest_mode == RestMode::Complete);
    let mode = match (positional, labeled, complete) {
        (Some(p), Some(l), None) => RuntimeRestMode::Split {
            positional_param_index: p as u16,
            labeled_param_index: l as u16,
        },
        (Some(p), None, None) => RuntimeRestMode::Positional { param_index: p as u16 },
        (None, Some(l), None) => RuntimeRestMode::Labeled { param_index: l as u16 },
        (None, None, Some(c)) => RuntimeRestMode::Complete { param_index: c as u16 },
        (None, None, None) => return None,
        _ => unreachable!("validated rest declaration"),
    };
    let fixed_positionals = params.iter().filter(|p| p.label.is_none() && !p.is_rest()).count() as u8;
    let fixed_labels = params.iter().filter_map(|p| p.label.as_ref()).map(|label| interner.intern(label)).collect();
    Some(RestLayout::new(fixed_positionals, fixed_labels, mode))
}

fn rest_selector(name: &str, params: &[phalcom_ast::ast::ParameterDef]) -> String {
    let slots = params
        .iter()
        .map(|param| match param.rest_mode {
            RestMode::Positional => "*".to_string(),
            RestMode::Labeled => "**".to_string(),
            RestMode::Complete => "***".to_string(),
            RestMode::None => param.label.as_deref().map(encode_label_component).unwrap_or_else(|| "_".to_string()),
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{name}({slots})")
}

/// Source parsing catches this first; compiler passes can synthesize members,
/// so preserve the declaration-side selector invariant after expansion too.
fn validate_declaration_labels(params: &[phalcom_ast::ast::ParameterDef]) -> Result<(), CompilerError> {
    let mut seen = std::collections::HashMap::<String, SourceRange>::new();
    for param in params {
        let Some(label) = &param.label else {
            continue;
        };
        if let Some(first_span) = seen.get(label) {
            return Err(CompilerError::DuplicateArgumentLabel {
                label: label.clone(),
                span: param.range,
                first_span: *first_span,
            });
        }
        seen.insert(label.clone(), param.range);
    }
    Ok(())
}

impl<'vm> Compiler<'vm> {
    /// Lowers a `class Name [extends Super] { members }` declaration
    /// (ADR-0011, ADR-0017, U-INH) — a whole-class field collection pass over
    /// every member body, a compile-time [`crate::vm::ClassLayout`] build
    /// (own fields stacked on top of the superclass's, U-INH §3.5), the
    /// [`Bytecode::Class`] allocation/reopen opcode, then one method/getter/
    /// setter/constructor member at a time attached via [`Bytecode::Method`].
    ///
    /// Before any of that, runs the U-ANNOT-CONTRACTS/U-ANNOT-LAYOUT
    /// attribute-expansion pass ([`expand_class_attributes`]): derives
    /// `@construct`'s constructor, weaves `@requires`/`@ensures`/`@invariant`
    /// guards (mode-gated per [`CompileMode`]), and resolves
    /// `attr.unknown`/`attr.illegal_target` legality — all *before* field
    /// collection and layout build see the class's final member list.
    ///
    /// # Errors
    ///
    /// Propagates any attribute-expansion error (`attr.unknown`,
    /// `attr.illegal_target`, `attr.accessor_collision`,
    /// `contract.impure_predicate`), any error compiling a member body, an
    /// invalid reopen (adding fields or changing the superclass —
    /// ADR-0018/U13), an unknown or self-referential superclass, or a
    /// malformed rest parameter.
    pub(super) fn compile_class(&mut self, class_def: ClassDef) -> Result<(), CompilerError> {
        self.compile_class_impl(class_def, false)
    }

    fn compile_class_impl(&mut self, class_def: ClassDef, allow_synthetic_internal: bool) -> Result<(), CompilerError> {
        let strip_metadata = match self.vm.compile_mode {
            CompileMode::Debug => false,
            CompileMode::Release => self.vm.strip_contract_metadata,
            CompileMode::Unchecked => true,
        };
        // Validate the source AST before attribute expansion. Expansion may
        // synthesize compiler-owned `_$...` hooks; source must never be able
        // to forge the same namespace.
        if !allow_synthetic_internal && !self.compiling_privileged_core() {
            for member in &class_def.members {
                let reserved = match member {
                    ClassMember::Method(m) if m.name.starts_with("_$") => Some((&m.name, m.name_range)),
                    ClassMember::Getter(g) if g.name.starts_with("_$") => Some((&g.name, g.name_range)),
                    ClassMember::Setter(s) if s.name.starts_with("_$") => Some((&s.name, s.name_range)),
                    ClassMember::Field(f) if f.name.starts_with("__") => Some((&f.name, f.range)),
                    _ => None,
                };
                if let Some((name, range)) = reserved {
                    return Err(CompilerError::InternalNamespaceReserved(name.clone(), range));
                }
            }
        }
        // Reject source constructor rest before attribute expansion. The
        // constructor expander lowers the marker into paired factory/init
        // methods, where erasing it would otherwise occur before the
        // post-expansion defensive pass below can observe it.
        for member in &class_def.members {
            validate_rest_usage(member)?;
        }
        // M-ATTR-ROOT: whether this class itself is a would-be `Attribute`
        // subclass — a direct `extends Attribute` (transitively-inherited
        // `On`/tier declarations are v0.3, `attribute-classes.md`'s A-1 "at
        // most one tier per class" scope). Read before `expand_class_attributes`
        // consumes `class_def` and before `ctx` borrows `self.vm.interner`
        // mutably (borrow-order matters, see the handoff's "still unknown"
        // list — `class_parents` is read-only here, `interner` mutable).
        let is_attribute_class = class_def.superclass.as_ref().is_some_and(|sc| sc.leaf_name() == "Attribute");
        let core_module = self.vm.core_module();
        let mut ctx = ExpandCtx {
            interner: &mut self.vm.interner,
            compile_mode: self.vm.compile_mode,
            strip_metadata,
            class_parents: &self.vm.class_parents,
            sealed_classes: &self.vm.sealed_classes,
            module: self.module,
            core_module,
        };
        let registry = AttributeRegistry::new();
        // DEC-ANNOT-G (U-ANNOT-LAYOUT §3.4): `expand_class_attributes`
        // returns `(ClassDef, Vec<Statement>)`, not a bare `ClassDef` — the
        // second element is every sibling top-level `Statement::Class` a
        // `@variant` arm inside `class_def`'s body expanded into (empty for
        // every class with no `@variant` arms). Compiled recursively at the
        // very end of this function, once `class_def` itself is fully
        // defined (`sibling_classes`'s own doc there explains why: each
        // sibling `extends` this class by name, so its own `Bytecode::Class`
        // resolves that name via `GetGlobal`, which requires this class's
        // `DefineGlobal` to have already run).
        let (mut class_def, sibling_classes) = expand_class_attributes(class_def, &mut ctx, &registry, is_attribute_class)?;
        // Rest-scope validation must precede duplicate-selector
        // canonicalization: otherwise constructor/index rest could be erased
        // into an ordinary key.
        for member in &class_def.members {
            validate_rest_usage(member)?;
            match member {
                ClassMember::Method(method) => validate_declaration_labels(&method.params)?,
                ClassMember::Index(index) => validate_declaration_labels(&index.params)?,
                ClassMember::Getter(_) | ClassMember::Setter(_) | ClassMember::Field(_) | ClassMember::Variant(_) => {}
            }
        }
        // Retained class-level attributes (M-ATTR-ROOT): needed after
        // `DefineGlobal` below, once `class_def.members` has been moved out
        // by the member loop — captured here rather than re-read later.
        let class_level_attrs = std::mem::take(&mut class_def.attributes);

        let range = class_def.range;
        let name_sym = self.vm.interner.intern(&class_def.name);
        let name_idx = self.add_constant(Value::symbol(name_sym));

        // Pass -1: duplicate-member scan (U-CTOR §3.2).
        // - Duplicate field declarations -> `class.duplicate_field`
        // - Same-side canonical selector collisions -> `class.duplicate_selector`
        //
        // Runs over `class_def.members` post-expansion before any member compiles,
        // using the member's encoded selector identity (`encode_selector`/`SignatureKind`).
        // `Variant` never appears here (`expand_variants` already stripped it).
        {
            #[derive(PartialEq, Eq, Hash)]
            enum MemberKey {
                Field(String),
                Selector(bool, String),
            }
            let mut seen: std::collections::HashMap<MemberKey, (String, SourceRange)> = std::collections::HashMap::new();
            let mut rest_families: std::collections::HashMap<(bool, String), SourceRange> = std::collections::HashMap::new();
            for member in &class_def.members {
                let (key, display, member_name_range) = match member {
                    ClassMember::Field(f) => (MemberKey::Field(f.name.clone()), f.name.clone(), f.range),
                    ClassMember::Method(m) => {
                        let subject = if m.is_constructor { "constructor declaration" } else { "method declaration" };
                        let kind = SignatureKind::Method(checked_send_arity(subject, m.params.len(), m.range)?);
                        let sel = if m.params.iter().any(phalcom_ast::ast::ParameterDef::is_rest) {
                            rest_selector(&m.name, &m.params)
                        } else {
                            let labels: Vec<Option<String>> = m.params.iter().map(|p| p.label.clone()).collect();
                            encode_selector(&m.name, &labels, kind)
                        };
                        (MemberKey::Selector(m.is_static, sel.clone()), sel, m.name_range)
                    }
                    ClassMember::Getter(g) => {
                        let sel = encode_selector(&g.name, &[], SignatureKind::Getter);
                        (MemberKey::Selector(g.is_static, sel.clone()), sel, g.name_range)
                    }
                    ClassMember::Setter(s) => {
                        let sel = encode_selector(&s.name, &[], SignatureKind::Setter);
                        (MemberKey::Selector(s.is_static, sel.clone()), sel, s.name_range)
                    }
                    ClassMember::Index(idx) => {
                        let labels: Vec<Option<String>> = idx.params.iter().map(|p| p.label.clone()).collect();
                        let arity = checked_send_arity("subscript declaration", idx.params.len(), idx.range)?;
                        let kind = match &idx.accessor {
                            IndexAccessor::Get => SignatureKind::SubscriptGet(arity),
                            IndexAccessor::Set { .. } => SignatureKind::SubscriptSet(arity),
                        };
                        let sel = encode_selector("", &labels, kind);
                        (MemberKey::Selector(false, sel.clone()), sel, idx.name_range)
                    }
                    ClassMember::Variant(_) => continue,
                };
                if self.vm.core_module() != Some(self.module)
                    && let MemberKey::Selector(_, selector) = &key
                    && matches!(selector.as_str(), "===(_)" | "===")
                {
                    return Err(CompilerError::Message(format!(
                        "reserved semantic selector `{selector}` cannot be declared",
                    )));
                }
                if let Some((_, first_range)) = seen.get(&key) {
                    let first_range = *first_range;
                    let source = self.source_text();
                    let (line, col) = crate::diagnostics::line_col(&source, first_range.start);
                    let err = match key {
                        MemberKey::Field(_) => CompilerError::DuplicateField(class_def.name.clone(), display, member_name_range, first_range, line, col),
                        MemberKey::Selector(_, _) => {
                            CompilerError::DuplicateSelector(class_def.name.clone(), display, member_name_range, first_range, line, col)
                        }
                    };
                    return Err(err);
                }
                if let ClassMember::Method(method) = member
                    && method.params.iter().any(phalcom_ast::ast::ParameterDef::is_rest)
                    && let Some(first_span) = rest_families.insert((method.is_static, method.name.clone()), member_name_range)
                {
                    let source = self.source_text();
                    let (first_line, first_col) = crate::diagnostics::line_col(&source, first_span.start);
                    return Err(CompilerError::DuplicateRestMethodFamily {
                        class: class_def.name.clone(),
                        base: method.name.clone(),
                        span: member_name_range,
                        first_span,
                        first_line,
                        first_col,
                    });
                }
                seen.insert(key, (display, member_name_range));
            }
        }

        // 1. Whole-class field collection pass
        let mut own_instance_fields = Vec::new();
        let mut own_static_fields = Vec::new();

        // Pass 1: Collect static fields
        for member in &class_def.members {
            match member {
                ClassMember::Method(m) if m.is_static => {
                    let mut fields = Vec::new();
                    for stmt in &m.body {
                        collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                    }
                    for f in fields {
                        if !own_static_fields.contains(&f) {
                            own_static_fields.push(f);
                        }
                    }
                }
                ClassMember::Getter(g) if g.is_static => {
                    let mut fields = Vec::new();
                    for stmt in &g.body {
                        collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                    }
                    for f in fields {
                        if !own_static_fields.contains(&f) {
                            own_static_fields.push(f);
                        }
                    }
                }
                ClassMember::Setter(s) if s.is_static => {
                    let mut fields = Vec::new();
                    for stmt in &s.body {
                        collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                    }
                    for f in fields {
                        if !own_static_fields.contains(&f) {
                            own_static_fields.push(f);
                        }
                    }
                }
                _ => {}
            }
        }

        // Pass 2: Collect instance fields (only if not static).
        //
        // U-ANNOT-LAYOUT §3.1/DEC-ANNOT-H: a class that declares at least one
        // `ClassMember::Field` uses its declared `FieldDef`s, in source
        // order, as the *complete* instance-field list — the legacy
        // implicit-by-assignment inference below is skipped entirely for
        // that class. A class with zero `FieldDef`s is completely unaffected
        // (byte-for-byte the same inference path as before this unit).
        // Mixing declared and inferred fields within one class is
        // unsupported (the "Rubric" hazard) — not detected/rejected here,
        // just not attempted: any assignment-inferred field in a
        // `FieldDef`-bearing class's hand-written members is simply not
        // added to the layout, which will surface as a
        // `ReadBeforeWrite`/missing-slot error at the assignment site rather
        // than a dedicated diagnostic (deferred to a follow-on unit per
        // DEC-ANNOT-H's "not a hard error, just inference off" resolution).
        let declared_fields: Vec<Symbol> = class_def
            .members
            .iter()
            .filter_map(|m| match m {
                ClassMember::Field(f) if !f.is_static => Some(self.vm.interner.intern(&f.name)),
                _ => None,
            })
            .collect();
        for member in &class_def.members {
            if let ClassMember::Field(f) = member
                && f.is_static
            {
                own_static_fields.push(self.vm.interner.intern(&f.name));
            }
        }
        // Own `const`-declared field names (ADR-0064 §3, U-BINDINGS §5) —
        // only an explicit `FieldDef` can be `const`; an implicitly-inferred
        // field (bare assignment, no declaration) is always mutable.
        let const_field_names: std::collections::HashSet<Symbol> = class_def
            .members
            .iter()
            .filter_map(|m| match m {
                ClassMember::Field(f) if !f.mutable => Some(self.vm.interner.intern(&f.name)),
                _ => None,
            })
            .collect();
        let has_declared_fields = !declared_fields.is_empty();
        if has_declared_fields {
            own_instance_fields = declared_fields;
        } else {
            for member in &class_def.members {
                match member {
                    ClassMember::Method(m) if !m.is_static => {
                        let mut fields = Vec::new();
                        for stmt in &m.body {
                            collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                        }
                        for f in fields {
                            if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                own_instance_fields.push(f);
                            }
                        }
                    }
                    ClassMember::Getter(g) if !g.is_static => {
                        let mut fields = Vec::new();
                        for stmt in &g.body {
                            collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                        }
                        for f in fields {
                            if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                own_instance_fields.push(f);
                            }
                        }
                    }
                    ClassMember::Setter(s) if !s.is_static => {
                        let mut fields = Vec::new();
                        for stmt in &s.body {
                            collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                        }
                        for f in fields {
                            if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                own_instance_fields.push(f);
                            }
                        }
                    }
                    ClassMember::Method(c) if c.is_constructor => {
                        let mut fields = Vec::new();
                        for stmt in &c.body {
                            collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                        }
                        for f in fields {
                            if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                own_instance_fields.push(f);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // 1b. Classes are closed (PDR-0001, U-CLASSCLOSE §2.1/§4). A
        // class is defined exactly once, by exactly one module; there is no
        // reopening. "The class being declared" is looked up by an
        // own-module-only key — **no core-module fallback** — per
        // U-CLASSNS §4.1's ruling for this site: falling back here would
        // let a non-core module's `class List {}` silently resolve onto the
        // kernel `List`'s `ClassId` (the exact hazard that ruling names).
        let name_key = self.class_key(name_sym);
        let is_core_module = self.vm.core_module() == Some(self.module);

        // Reserved kernel names (ruling 3): exactly `VM::kernel_class_names`
        // — the primitives `add_class!` binds (`Object`, `List`, `Number`,
        // `Error`, …), not every class `core.ph` itself goes on to declare.
        // A core-library class like `ArgumentError` (`extends Error {}` in
        // `.ph`) is module-scoped like any other (ruling 1) and carries no
        // literal-bound `ClassId`, so redeclaring or subclassing *that* name
        // from a non-core module is not the trap this ruling guards
        // against — only the primitives are.
        if !is_core_module && self.vm.kernel_class_names.contains(&name_sym) {
            return Err(CompilerError::ClassReservedName(class_def.name.clone(), class_def.name_range));
        }

        let classes_hit = self.vm.classes.contains_key(&name_key);
        let field_layouts_hit = self.vm.field_layouts.contains_key(&name_key);
        // `field_layouts` alone is the redefinition signal — NOT
        // `classes_hit && field_layouts_hit`. `field_layouts` is written at
        // COMPILE time (synchronously, right below); `classes` is written at
        // RUNTIME, when `Bytecode::Class`/`Bytecode::Constant` actually
        // executes (`create_class`). A same-unit duplicate (`class Point {}`
        // twice in one file) compiles both blocks into *one* closure before
        // either ever runs, so at the second block's compile time
        // `field_layouts_hit` is already `true` but `classes_hit` is still
        // `false` — requiring both would silently let same-unit duplicates
        // through, which is precisely the bug this check exists to catch.
        if self.unit_kind != UnitKind::Repl && field_layouts_hit {
            // Exempt for a REPL cell (PDR-0001 ruling 6): a later
            // cell's `class Foo {}` shadows rather than reopens — it binds
            // a brand-new class and the global rebinds to it; an instance
            // made under the old definition keeps pointing at the old
            // `ClassId` (nothing migrates), and the old class simply
            // becomes unreachable by name. That is the allocate-fresh path
            // below, unconditionally, for a REPL unit — `classes_hit`
            // itself already special-cases REPL out of the stub-completion
            // fork just past this block.
            let first_range = self.vm.field_layouts[&name_key].declared_at;
            let source = self.source_text();
            let (line, col) = crate::diagnostics::line_col(&source, first_range.start);
            return Err(CompilerError::ClassAlreadyDefined(
                class_def.name.clone(),
                class_def.name_range,
                first_range,
                line,
                col,
            ));
        }
        // `classes_hit && !field_layouts_hit` is stub completion — reachable
        // only in the core module (`add_class!`/the `None` row both key to
        // the core module handle; any `.ph` declaration writes both tables
        // together) outside of a REPL cell. Assert it explicitly: an
        // assertion that can never fire is the cheapest documentation of the
        // invariant ruling 4 asks for.
        debug_assert!(
            !classes_hit || field_layouts_hit || is_core_module,
            "stub completion reachable only in the core module"
        );

        // A class name colliding with an `import … as Name` already bound
        // in this compilation unit is also `class.already_defined`
        // (PDR-0002 ruling 8) — the reverse ordering (`class` then
        // `import`) is caught by `declare_global`'s own `binding.redeclared`
        // once this class registers below, so no work is needed there.
        if let Some(&import_range) = self.import_bindings.get(&name_sym) {
            let source = self.source_text();
            let (line, col) = crate::diagnostics::line_col(&source, import_range.start);
            return Err(CompilerError::ClassAlreadyDefined(
                class_def.name.clone(),
                class_def.name_range,
                import_range,
                line,
                col,
            ));
        }

        // Register in `global_bindings` (PDR-0002 ruling 8) — insertion
        // only, never through `declare_global` (see that field's own doc):
        // this class's own checks above are already this unit's sole source
        // of `class.already_defined`, so a later `import … as` Name` of the
        // same name is left to `declare_global`'s ordinary
        // `binding.redeclared` path, unmodified.
        self.global_bindings.insert(name_sym, false);

        // 2. Build the ClassLayout and store it in VM.
        //
        // A subclass's own fields stack on top of the superclass's fields
        // (ADR-0011, U-INH §3.5): own instance/static slots begin at the
        // superclass's field count, so inherited slots keep their offsets
        // and are never aliased. `field_layouts_hit` is `false` here
        // unconditionally — §1b above already rejected the one case where it
        // could be `true` (a real redefinition) — so the superclass's counts
        // always come fresh: either from a Rust-installed stub already in
        // `vm.classes` under this exact own-module key (`classes_hit`, the
        // core-module stub-completion case, §1b's core gate), from the
        // `extends` clause (looked up in the accumulating
        // `field_layouts`/`classes` metadata, since a *user* superclass has
        // not been created at runtime yet), or the implicit `Object` root.
        let layout = {
            let (sc_field_count, sc_meta_field_count) = if self.unit_kind != UnitKind::Repl && classes_hit {
                let existing_class = self.vm.classes[&name_key];
                // Stub completion: keep the Rust stub's established superclass.
                match self.vm.heap.class(existing_class).superclass {
                    Some(sc_id) => {
                        let meta = self.vm.heap.class(sc_id).class;
                        (self.vm.heap.class(sc_id).field_count, self.vm.heap.class(meta).field_count)
                    }
                    None => (0, 0),
                }
            } else if let Some(sc_ref) = &class_def.superclass {
                let sc_sym = self.vm.interner.intern(sc_ref.leaf_name());
                let sc_key = self.resolve_superclass_ref(sc_ref);
                // Self-inheritance and unknown/forward superclasses are
                // rejected here (U-INH §3.2): a class cannot appear in its own
                // superclass chain (that would make method lookup
                // non-terminating), and the single top-down compile pass
                // requires the superclass to be defined earlier. A longer
                // cycle is rejected transitively — the earlier class in the
                // cycle refers forward to a not-yet-defined name.
                if sc_sym == name_sym {
                    return Err(CompilerError::Message(format!(
                        "A class cannot extend itself: `{}` names itself as its superclass.",
                        class_def.name
                    )));
                }
                // U-ANNOT-LAYOUT §3.4 (`attr.sealed_violation`): a subclass
                // of an `@sealed` class is only legal within the same
                // compilation unit (`VM::sealed_classes`'s own doc) — checked
                // here, at the *subclass's* definition site, not the sealed
                // class's, matching `annotations-data.md`'s own placement.
                // Fires immediately rather than deferring to an end-of-unit
                // pass: the single-pass top-down discipline already
                // guarantees a same-unit sealed superclass is recorded
                // before any of its subclasses reach this point.
                let sc_key_for_sealed = sc_key.unwrap_or(ClassKey {
                    module: self.module,
                    name: sc_sym,
                });
                if let Some(&sealed_in_module) = self.vm.sealed_classes.get(&sc_key_for_sealed) {
                    if sealed_in_module != self.module {
                        return Err(CompilerError::Message(format!(
                            "attr.sealed_violation: `{}` is `@sealed` class `{}`, but was not declared in the same compilation unit",
                            class_def.name,
                            sc_ref.leaf_name()
                        )));
                    }
                }
                let counts = if let Some(key) = sc_key {
                    if let Some(layout) = self.vm.field_layouts.get(&key) {
                        (layout.field_count, layout.static_field_count)
                    } else if let Some(&sc_id) = self.vm.classes.get(&key) {
                        let meta = self.vm.heap.class(sc_id).class;
                        (self.vm.heap.class(sc_id).field_count, self.vm.heap.class(meta).field_count)
                    } else {
                        let mut candidates = Vec::new();
                        for key in self.vm.field_layouts.keys().chain(self.vm.classes.keys()) {
                            candidates.push(self.vm.resolve_symbol(key.name).to_string());
                        }
                        for &sym in self.import_bindings.keys() {
                            candidates.push(self.vm.resolve_symbol(sym).to_string());
                        }
                        candidates.sort();
                        candidates.dedup();
                        let cand_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
                        let suggestion = crate::diagnostics::suggest::best_match(sc_ref.leaf_name(), cand_refs.into_iter());
                        let mut msg = format!(
                            "unknown superclass '{}': it must be a class defined before '{}'",
                            sc_ref.leaf_name(),
                            class_def.name
                        );
                        if let Some(sug) = suggestion {
                            msg.push_str(&format!("; did you mean '{}'?", sug));
                        }
                        return Err(CompilerError::Message(msg));
                    }
                } else {
                    let mut candidates = Vec::new();
                    for key in self.vm.field_layouts.keys().chain(self.vm.classes.keys()) {
                        candidates.push(self.vm.resolve_symbol(key.name).to_string());
                    }
                    for &sym in self.import_bindings.keys() {
                        candidates.push(self.vm.resolve_symbol(sym).to_string());
                    }
                    candidates.sort();
                    candidates.dedup();
                    let cand_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
                    let suggestion = crate::diagnostics::suggest::best_match(sc_ref.leaf_name(), cand_refs.into_iter());
                    let mut msg = format!(
                        "unknown superclass '{}': it must be a class defined before '{}'",
                        sc_ref.leaf_name(),
                        class_def.name
                    );
                    if let Some(sug) = suggestion {
                        msg.push_str(&format!("; did you mean '{}'?", sug));
                    }
                    return Err(CompilerError::Message(msg));
                };
                // Record the compile-time superclass edge (U-INH follow-on)
                // ONLY here — past the self-check, on a known/validated
                // superclass. The reopen branch above and the self/unknown
                // error paths deliberately do not populate `class_parents`, so
                // no self- or dangling edge can enter the map (the VM persists
                // across REPL lines, so a stale edge would otherwise make the
                // guard/alias chain-walks spin). Edges normally point only to a
                // strictly-earlier-defined class; the one residual way to form a
                // back-edge is a reopen-redefinition within a unit (`class A {}`,
                // `class B is A`, then `class A is B`), which the
                // `visited` guard in both chain-walks handles without spinning.
                let sc_key_val = sc_key.unwrap_or(ClassKey {
                    module: self.module,
                    name: sc_sym,
                });
                self.vm.class_parents.insert(name_key, sc_key_val);
                counts
            } else {
                // Implicit `Object` root.
                let object_class = self.vm.universe.classes.object_class;
                let meta = self.vm.heap.class(object_class).class;
                (self.vm.heap.class(object_class).field_count, self.vm.heap.class(meta).field_count)
            };

            let mut field_slots = IndexMap::new();
            if let Some(sc_ref) = &class_def.superclass {
                if let Some(key) = self.resolve_superclass_ref(sc_ref) {
                    if let Some(sc_layout) = self.vm.field_layouts.get(&key) {
                        for (k, v) in &sc_layout.field_slots {
                            field_slots.insert(*k, *v);
                        }
                    } else if let Some(&sc_id) = self.vm.classes.get(&key) {
                        let class_obj = self.vm.heap.class(sc_id);
                        for (k, v) in &class_obj.field_slots {
                            field_slots.insert(*k, *v);
                        }
                    }
                }
            } else if is_core_module || name_sym != self.vm.interner.intern("Object") {
                let object_class = self.vm.universe.classes.object_class;
                let class_obj = self.vm.heap.class(object_class);
                for (k, v) in &class_obj.field_slots {
                    field_slots.insert(*k, *v);
                }
            }
            for (i, f) in own_instance_fields.iter().enumerate() {
                field_slots.insert(*f, (sc_field_count as usize + i) as u16);
            }
            let field_count = sc_field_count + own_instance_fields.len() as u16;

            let mut static_field_slots = IndexMap::new();
            for (i, f) in own_static_fields.iter().enumerate() {
                static_field_slots.insert(*f, (sc_meta_field_count as usize + i) as u16);
            }
            let static_field_count = sc_meta_field_count + own_static_fields.len() as u16;

            crate::vm::ClassLayout {
                name: name_sym,
                field_slots,
                field_count,
                static_field_slots,
                static_field_count,
                const_fields: const_field_names,
                declared_at: class_def.range,
            }
        };
        self.vm.field_layouts.insert(name_key, layout);

        self.current_class = Some(name_key);

        if self.unit_kind != UnitKind::Repl && classes_hit {
            // Stub completion (§5.1's core gate — `classes_hit` without a
            // `field_layouts` hit is unreachable outside the core module,
            // asserted above): the compiler already knows this is not an
            // allocate-fresh, so it emits `Constant` rather than `Class`,
            // and the runtime's `Bytecode::Class` arm never needs to probe
            // `vm.classes` by name at all (§5.2 deletes that probe).
            let existing_class = self.vm.classes[&name_key];
            let class_idx = self.add_constant(Value::obj(existing_class));
            self.emit(Bytecode::Constant(class_idx), range);
        } else {
            // Push the superclass onto the stack for the `Class` handler
            // to consume (vm.rs `Bytecode::Class` pops it and wires both
            // `superclass` and the parallel metaclass via `create_class`,
            // ADR-0002 rule 4). An explicit `extends S` resolves `S` as an
            // ordinary global at runtime; with no `extends` the class
            // implicitly inherits from `Object`.
            if let Some(sc_ref) = &class_def.superclass {
                let sc_sym = self.vm.interner.intern(sc_ref.leaf_name());
                let sc_name_idx = self.add_constant(Value::symbol(sc_sym));
                self.emit(Bytecode::GetGlobal(sc_name_idx), sc_ref.range);
            } else {
                let object_class = self.vm.universe.classes.object_class;
                let superclass_idx = self.add_constant(Value::obj(object_class));
                self.emit(Bytecode::Constant(superclass_idx), range);
            }
            self.emit(Bytecode::Class(name_idx), range);
        }

        // The class object is now on top of the stack. Iterate through members.
        for member in class_def.members {
            match member {
                ClassMember::Method(method_def) if !method_def.is_constructor => {
                    let range = method_def.range;

                    let arity = method_def.params.len();
                    let encoded_arity = checked_send_arity("method declaration", arity, method_def.range)?;
                    let sig_kind = SignatureKind::Method(encoded_arity);
                    let rest = rest_layout(&method_def.params, &mut self.vm.interner);
                    let selector = if rest.is_some() {
                        rest_selector(&method_def.name, &method_def.params)
                    } else {
                        let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                        encode_selector(&method_def.name, &labels, sig_kind)
                    };
                    let selector_sym = self.vm.interner.intern(&selector);

                    let param_names: Vec<String> = method_def.params.iter().map(|p| p.name.clone()).collect();
                    self.is_static_context = method_def.is_static;
                    let prior_compiler_internal = self.compiler_internal;
                    self.compiler_internal = method_def
                        .attributes
                        .iter()
                        .any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Constructor)) || attr.name == "__synthetic");
                    let closure_result = self.compile_block(method_def.body, selector_sym, ClosureParameters::fixed(param_names), true, false, None);
                    self.compiler_internal = prior_compiler_internal;
                    let closure = closure_result?;

                    tracing::debug!("[Compiler] Compiling method: {} (static: {})", selector, method_def.is_static);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        sig_kind,
                        MethodKind::Closure(closure),
                    ))));
                    {
                        let method = self.vm.heap.method_mut(method_obj);
                        method.visibility = member_visibility(Some(&method_def.name), &method_def.attributes);
                        if let Some(rest) = rest {
                            method.signature = crate::method::Signature::new_with_arity(selector_sym, sig_kind, rest.fixed_positionals(), Some(rest));
                        }
                    }

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&method_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, method_def.is_static), range);

                    self.emit_member_attribute_attaches(&method_def.attributes, method_obj_idx, range)?;
                }
                ClassMember::Getter(getter_def) => {
                    let range = getter_def.range;

                    let selector = make_signature(&getter_def.name, SignatureKind::Getter);
                    let selector_sym = self.vm.interner.intern(&selector);

                    self.is_static_context = getter_def.is_static;
                    let closure = self.compile_block(getter_def.body, selector_sym, ClosureParameters::default(), true, false, None)?;

                    tracing::debug!("[Compiler] Compiling getter: {} (static: {})", selector, getter_def.is_static);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        SignatureKind::Getter,
                        MethodKind::Closure(closure),
                    ))));
                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&getter_def.name), &getter_def.attributes);

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&getter_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, getter_def.is_static), range);

                    self.emit_member_attribute_attaches(&getter_def.attributes, method_obj_idx, range)?;
                }
                ClassMember::Setter(setter_def) => {
                    let range = setter_def.range;

                    let selector = make_signature(&setter_def.name, SignatureKind::Setter);
                    let selector_sym = self.vm.interner.intern(&selector);

                    self.is_static_context = setter_def.is_static;
                    let closure = self.compile_block(
                        setter_def.body,
                        selector_sym,
                        ClosureParameters::fixed(vec![setter_def.param.name.clone()]),
                        true,
                        false,
                        None,
                    )?;

                    tracing::debug!("[Compiler] Compiling setter: {} (static: {})", selector, setter_def.is_static);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        SignatureKind::Setter,
                        MethodKind::Closure(closure),
                    ))));
                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&setter_def.name), &setter_def.attributes);

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&setter_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, setter_def.is_static), range);

                    self.emit_member_attribute_attaches(&setter_def.attributes, method_obj_idx, range)?;
                }
                ClassMember::Method(construct_def) => {
                    let range = construct_def.range;

                    let arity = checked_send_arity("constructor declaration", construct_def.params.len(), construct_def.range)?;
                    let labels: Vec<Option<String>> = construct_def.params.iter().map(|p| p.label.clone()).collect();
                    // Initializers install as ordinary instance-side methods
                    // under their generated, source-unspellable `init <name>`
                    // selector (ADR-0063). The paired factory is class-side
                    // and dispatches under the original constructor selector.
                    let selector = encode_selector(&construct_def.name, &labels, SignatureKind::Method(arity));
                    let selector_sym = self.vm.interner.intern(&selector);

                    let param_names: Vec<String> = construct_def.params.iter().map(|p| p.name.clone()).collect();

                    self.is_static_context = false;
                    // `const` fields are only assignable within a constructor
                    // body (ADR-0064 §3, U-BINDINGS §5) — gate the
                    // `field.const_write` check in `expr.rs` on this flag,
                    // restored unconditionally after the body compiles (a
                    // constructor never nests another constructor).
                    self.in_constructor = true;
                    let closure = self.compile_block(
                        construct_def.body,
                        selector_sym,
                        ClosureParameters::fixed(param_names),
                        true,
                        true,
                        Some(construct_def.name.strip_prefix("init ").unwrap_or(&construct_def.name).to_string()),
                    )?;
                    self.in_constructor = false;

                    tracing::debug!("[Compiler] Compiling constructor: {}", selector);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        SignatureKind::Method(arity),
                        MethodKind::Closure(closure),
                    ))));
                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&construct_def.name), &construct_def.attributes);

                    let method_obj_idx = self.add_constant(Value::obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, false), range);
                }
                // A declared field is layout-only (U-ANNOT-LAYOUT §3.1): it
                // already fed `own_instance_fields`/`field_slots` above and
                // emits no bytecode of its own. Its `default` expression, if
                // any, is data for a layout-derive attribute
                // (`@construct`/`@data`) to read — not compiled here.
                ClassMember::Field(field) if field.is_static => {
                    if let Some(default) = field.default {
                        let layout = self.vm.field_layouts.get(&name_key).unwrap().clone();
                        let field_sym = self.vm.interner.intern(&field.name);
                        let slot = *layout.static_field_slots.get(&field_sym).unwrap();
                        self.emit(Bytecode::Dup, field.range);
                        self.compile_expr(default)?;
                        self.emit(Bytecode::SetField(slot), field.range);
                        self.emit(Bytecode::Pop, field.range);
                    }
                }
                ClassMember::Field(_) => {}
                // Unreachable in practice — `expand_class_attributes`
                // (`compiler::attributes::expand_variants`) always strips
                // every `Variant` member before returning; kept for match
                // exhaustiveness over `ClassMember`'s full variant set.
                ClassMember::Variant(_) => {}
                // A bracket subscript method (U-INDEX, ADR-0060: `[idx] {
                // ... }` / `[idx, put:] { ... }`) — same codegen shape as an
                // ordinary `Method`, just with no name token and a
                // `SignatureKind::Subscript` selector (`[_]`, `[_,put]`,
                // `[]`, `[put]`, ...) instead of `name(...)`. Always an
                // instance method — the grammar has no `static [idx] {}`
                // form.
                ClassMember::Index(index_def) => {
                    let range = index_def.range;
                    let arity = checked_send_arity("subscript declaration", index_def.params.len(), index_def.range)?;
                    let labels: Vec<Option<String>> = index_def.params.iter().map(|p| p.label.clone()).collect();

                    let mut param_names: Vec<String> = index_def.params.iter().map(|p| p.name.clone()).collect();
                    let sig_kind = match &index_def.accessor {
                        IndexAccessor::Get => SignatureKind::SubscriptGet(arity),
                        IndexAccessor::Set { put } => {
                            checked_send_arity("subscript declaration", index_def.params.len() + 1, index_def.range)?;
                            param_names.push(put.name.clone());
                            SignatureKind::SubscriptSet(arity)
                        }
                    };

                    let selector = encode_selector("", &labels, sig_kind);
                    let selector_sym = self.vm.interner.intern(&selector);

                    self.is_static_context = false;
                    let closure = self.compile_block(index_def.body, selector_sym, ClosureParameters::fixed(param_names), true, false, None)?;

                    tracing::debug!("[Compiler] Compiling subscript method: {}", selector);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        sig_kind,
                        MethodKind::Closure(closure),
                    ))));
                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(None, &index_def.attributes);

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&index_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, false), range);

                    self.emit_member_attribute_attaches(&index_def.attributes, method_obj_idx, range)?;
                }
            }
        }

        self.current_class = None;

        // Rebuild the class's (and its metaclass's) base-name index
        // (selectors.md §3.1, U16-Open) now that every member of this
        // body has been attached — peeks, does not pop, the class
        // value still on the stack for `DefineGlobal` below.
        self.emit(Bytecode::FinalizeClass, range);

        // After defining all methods, the class is still on the stack.
        // Define it as a global variable — except `None` (`DEFERRED` #17,
        // U-CLASSCLOSE §7): `None`'s global is bound to the shared
        // *immediate value*, not the class object (`vm/bootstrap.rs`), so
        // rebinding it here would silently break every `x == None`. `None`
        // is reserved (ruling 3) so no non-core module ever reaches this —
        // the guard exists for when `core.ph` itself gains a `class None {}`
        // skeleton (not yet true today; `#17` stays open, this is only its
        // prerequisite). `Pop` keeps the stack balanced in its place.
        let is_none_in_core = is_core_module && self.vm.interner.find("None") == Some(name_sym);
        if is_none_in_core {
            self.emit(Bytecode::Pop, range);
        } else {
            self.emit(Bytecode::DefineGlobal(name_idx), range);
        }

        // M-ATTR-ROOT: class-level `@Name(args)` runtime instantiate+attach
        // codegen (`attribute-classes.md` §"What the compiler lowers").
        // `DefineGlobal` above *consumed* the class value off the stack — no
        // prior "run this after the class opcode" mechanism exists in this
        // compiler, so the class is re-fetched here via `GetGlobal` (the
        // closure that follows compiles the attach sends, then *runs* them —
        // `DefineGlobal` already executed by the time these run, so the
        // re-fetch always resolves). Compiler-only names
        // (`COMPILER_ONLY_ATTRS`) are woven directly into method bodies
        // above and carry no runtime `Attribute` instance, so they are
        // skipped here.
        // `None` global holds the immediate value, not the class (see the
        // `DefineGlobal` guard above) — a `GetGlobal` re-fetch here would
        // attach/freeze attributes on the wrong object, so skip this whole
        // block for the same reason.
        if !is_none_in_core {
            let has_runtime_attrs = class_level_attrs.iter().any(|a| !COMPILER_ONLY_ATTRS.contains(&a.name.as_str()));
            if has_runtime_attrs {
                for attr in &class_level_attrs {
                    if COMPILER_ONLY_ATTRS.contains(&attr.name.as_str()) {
                        continue;
                    }
                    self.emit(Bytecode::GetGlobal(name_idx), range);
                    self.emit_attribute_attach(attr)?;
                }
            }
            // The retention store is frozen once, at the end of class
            // definition (A-5, `attribute-classes.md`'s hazards table) —
            // further `__attach` calls raise `attr.frozen`. Always emitted,
            // even for a class with no attributes at all, so the invariant
            // holds uniformly.
            self.emit(Bytecode::GetGlobal(name_idx), range);
            self.emit_freeze_attributes(range);
        }

        // U-ANNOT-LAYOUT §3.4: record `@sealed`'s compile-unit-scoped
        // bookkeeping now that `class_def.name`'s global is fully defined —
        // any subclass compiled *after* this point (this unit or another)
        // checks `VM::sealed_classes` at the point in `Self::compile_class`
        // above where its own superclass is resolved.
        if class_level_attrs.iter().any(|a| a.name == "sealed") {
            self.vm.sealed_classes.insert(name_key, self.module);
        }

        // DEC-ANNOT-G: compile every `@variant`-generated sibling class now,
        // immediately after this class's own `DefineGlobal` — each sibling
        // `extends` this class by name, so its `Bytecode::Class` opcode's
        // `GetGlobal` superclass lookup requires this class to already be a
        // defined global, which it now is. A recursive `compile_class` call
        // reuses this exact function for the sibling (including its own
        // `@data` expansion, `@sealed` bookkeeping, and any further sibling
        // splicing, though `@variant` classes never themselves carry nested
        // `@variant` arms in Draft 0.1).
        for sibling in sibling_classes {
            if let Statement::Class(sibling_def) = sibling {
                self.compile_class_impl(sibling_def, true)?;
            }
        }

        Ok(())
    }

    /// Emits the member-level counterpart of the class-level attach codegen
    /// above: for each of `attrs` not in [`COMPILER_ONLY_ATTRS`], pushes
    /// `method_obj_idx` (the same constant-pool entry already used for this
    /// member's [`Bytecode::Method`] — the identical [`crate::heap::ObjRef`]
    /// now shared with the class's method dictionary, so attaching here
    /// mutates the installed method object in place) as the `__attach(_)`
    /// receiver. Freezes the method's own store afterward, same rationale as
    /// the class-level freeze.
    ///
    /// # Errors
    ///
    /// Propagates any error compiling a retained attribute's constructor
    /// arguments.
    fn emit_member_attribute_attaches(&mut self, attrs: &[Attribute], method_obj_idx: u16, range: SourceRange) -> Result<(), CompilerError> {
        let mut any = false;
        for attr in attrs {
            if COMPILER_ONLY_ATTRS.contains(&attr.name.as_str()) {
                continue;
            }
            any = true;
            self.emit(Bytecode::Constant(method_obj_idx), range);
            self.emit_attribute_attach(attr)?;
        }
        if any {
            self.emit(Bytecode::Constant(method_obj_idx), range);
            self.emit_freeze_attributes(range);
        }
        Ok(())
    }

    /// Compiles `AttrName.new(args…)` (`@Name(args)` desugars to a normal,
    /// fully positional constructor send — attribute arg lists are
    /// positional-only, `parser.rs`'s `parse_attribute_arg_list`, filed to
    /// `docs/forge/DEFERRED.md`) and immediately sends `__attach(_)` with it
    /// as the sole argument, against whatever receiver is already on top of
    /// the stack (a class or method object the caller pushed). Leaves the
    /// stack exactly as found: the `__attach` call's `None` result is
    /// popped.
    ///
    /// # Errors
    ///
    /// Propagates any error compiling the constructor call (e.g. an unknown
    /// global, a malformed argument expression).
    fn emit_attribute_attach(&mut self, attr: &Attribute) -> Result<(), CompilerError> {
        let range = attr.range;
        let ctor_call = Expr::MethodCall(Box::new(MethodCallExpr {
            object: Expr::Var {
                value: attr.name.clone(),
                range,
            },
            method: "new".to_string(),
            method_range: None,
            args: attr.args.iter().map(|expr| PackItem::Positional { expr: expr.clone(), range }).collect(),
            range,
        }));
        self.compile_expr(ctor_call)?;

        let attach_selector = make_signature("_$attach", SignatureKind::Method(1));
        let attach_sym = self.vm.interner.intern(&attach_selector);
        let attach_idx = self.add_constant(Value::symbol(attach_sym));
        self.emit(Bytecode::InvokeCompilerInternal(1, attach_idx), range);
        self.emit(Bytecode::Pop, range);
        Ok(())
    }

    /// Sends `__freezeAttributes()` against whatever receiver is already on
    /// top of the stack, then pops its `None` result — see
    /// [`Self::emit_attribute_attach`] for the receiver-already-pushed
    /// convention.
    fn emit_freeze_attributes(&mut self, range: SourceRange) {
        let freeze_selector = make_signature("_$freezeAttributes", SignatureKind::Method(0));
        let freeze_sym = self.vm.interner.intern(&freeze_selector);
        let freeze_idx = self.add_constant(Value::symbol(freeze_sym));
        self.emit(Bytecode::InvokeCompilerInternal(0, freeze_idx), range);
        self.emit(Bytecode::Pop, range);
    }

    /// Builds `MethodObject::contracts` (U-ANNOT-CONTRACTS plan §3.5,
    /// D-contract-1) from a member's post-expansion `@requires`/`@ensures`
    /// attributes: each predicate is compiled standalone, **un-woven**, as a
    /// zero-argument receiver-shaped closure (same shape as a getter body —
    /// `self` in slot 0), tagged `#requires_<n>`/`#ensures_<n>` with one
    /// counter per attribute kind in source declaration order.
    ///
    /// Returns an empty `Vec` if `attrs` carries neither kind — the caller
    /// stores `None` rather than `Some(vec![])` (`ExpandCtx::strip_metadata`
    /// callers gate on `Option`, not emptiness).
    ///
    /// # Errors
    ///
    /// Propagates any error compiling a predicate expression.
    fn build_contracts_metadata(&mut self, attrs: &[Attribute]) -> Result<Vec<(Symbol, Value)>, CompilerError> {
        let mut contracts = Vec::new();
        let mut requires_n = 0usize;
        let mut ensures_n = 0usize;
        for attr in attrs {
            let kind = match attr.name.as_str() {
                "requires" => "requires",
                "ensures" => "ensures",
                _ => continue,
            };
            for arg in &attr.args {
                let counter = if kind == "requires" { &mut requires_n } else { &mut ensures_n };
                let name = format!("#{}_{}", kind, *counter);
                *counter += 1;
                let name_sym = self.vm.interner.intern(&name);
                let range = arg.range();
                let body = vec![Statement::Expr { expr: arg.clone(), range }];
                let closure_ref = self.compile_block(body, name_sym, ClosureParameters::default(), true, false, None)?;
                contracts.push((name_sym, Value::obj(closure_ref)));
            }
        }
        Ok(contracts)
    }
}

fn collect_assigned_fields(expr: &Expr, fields: &mut Vec<Symbol>, interner: &mut crate::interner::Interner) {
    match expr {
        Expr::Assignment(assign) => {
            if let Expr::Field { value, .. } = &*assign.name {
                let sym = interner.intern(value);
                if !fields.contains(&sym) {
                    fields.push(sym);
                }
            }
            collect_assigned_fields(&assign.value, fields, interner);
        }
        Expr::Unary(unary) => {
            collect_assigned_fields(&unary.expr, fields, interner);
        }
        Expr::Binary(binary) => {
            collect_assigned_fields(&binary.left, fields, interner);
            collect_assigned_fields(&binary.right, fields, interner);
        }
        Expr::MethodCall(call) => {
            collect_assigned_fields(&call.object, fields, interner);
            for arg in &call.args {
                collect_assigned_fields_pack(arg, fields, interner);
            }
        }
        Expr::UnqualifiedCall(call) => {
            for arg in &call.args {
                collect_assigned_fields_pack(arg, fields, interner);
            }
        }
        Expr::GetProperty(get_prop) => {
            collect_assigned_fields(&get_prop.object, fields, interner);
        }
        Expr::SetProperty(set_prop) => {
            collect_assigned_fields(&set_prop.object, fields, interner);
            collect_assigned_fields(&set_prop.value, fields, interner);
        }
        Expr::Index(ix) => {
            collect_assigned_fields(&ix.object, fields, interner);
            for arg in &ix.args {
                collect_assigned_fields_pack(arg, fields, interner);
            }
        }
        Expr::SetIndex(six) => {
            collect_assigned_fields(&six.object, fields, interner);
            for arg in &six.args {
                collect_assigned_fields_pack(arg, fields, interner);
            }
            collect_assigned_fields(&six.value, fields, interner);
        }
        Expr::Block(block) => {
            for stmt in &block.body {
                collect_assigned_fields_stmt(stmt, fields, interner);
            }
        }
        Expr::MapLiteral(map) => {
            for entry in &map.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_assigned_fields(expr, fields, interner);
                        }
                        collect_assigned_fields(value, fields, interner);
                    }
                    MapLiteralEntry::Expansion { expr, .. } => collect_assigned_fields(expr, fields, interner),
                }
            }
        }
        Expr::SetLiteral(set) => {
            for entry in &set.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => collect_assigned_fields(expr, fields, interner),
                }
            }
        }
        Expr::ListLiteral(list) => {
            for element in &list.elements {
                match element {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => collect_assigned_fields(expr, fields, interner),
                }
            }
        }
        _ => {}
    }
}

fn collect_assigned_fields_pack(item: &PackItem, fields: &mut Vec<Symbol>, interner: &mut crate::interner::Interner) {
    match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => collect_assigned_fields(expr, fields, interner),
        PackItem::Labeled { label, value, .. } => {
            if let PackLabel::Computed { expr, .. } = label {
                collect_assigned_fields(expr, fields, interner);
            }
            collect_assigned_fields(value, fields, interner);
        }
    }
}

fn collect_assigned_fields_stmt(stmt: &Statement, fields: &mut Vec<Symbol>, interner: &mut crate::interner::Interner) {
    match stmt {
        Statement::Expr { expr, .. } => {
            collect_assigned_fields(expr, fields, interner);
        }
        Statement::Let(binding) => {
            if let Some(ref val) = binding.value {
                collect_assigned_fields(val, fields, interner);
            }
        }
        Statement::Return(ret) => {
            if let Some(ref val) = ret.value {
                collect_assigned_fields(val, fields, interner);
            }
        }
        Statement::For(for_stmt) => {
            // A field assigned only inside a `for` body must still be
            // collected into the class layout (ADR-0035; U-ITER), or its
            // first read would trip `ReadBeforeWrite`.
            for lane in &for_stmt.lanes {
                collect_assigned_fields(&lane.iter, fields, interner);
            }
            for body_stmt in &for_stmt.body {
                collect_assigned_fields_stmt(body_stmt, fields, interner);
            }
        }
        Statement::Throw { expr, .. } => {
            collect_assigned_fields(expr, fields, interner);
        }
        _ => {}
    }
}
