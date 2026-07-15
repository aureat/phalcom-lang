use crate::bytecode::Bytecode;
use crate::compiler::attributes::{expand_class_attributes, AttributeRegistry, CompileMode, ExpandCtx};
use crate::heap::Object;
use crate::interner::Symbol;
use crate::method::{encode_selector, make_signature, MethodKind, MethodObject, SignatureKind};
use crate::value::Value;
use phalcom_ast::ast::{Argument, Attribute, ClassDef, ClassMember, Expr, MethodCallExpr, Statement};
use indexmap::IndexMap;
use phalcom_common::range::SourceRange;

use super::error::CompilerError;
use super::Compiler;

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
const COMPILER_ONLY_ATTRS: &[&str] = &["requires", "ensures", "invariant", "construct", "data", "sealed"];

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
        let strip_metadata = match self.vm.compile_mode {
            CompileMode::Debug => false,
            CompileMode::Release => self.vm.strip_contract_metadata,
            CompileMode::Unchecked => true,
        };
        // M-ATTR-ROOT: whether this class itself is a would-be `Attribute`
        // subclass — a direct `extends Attribute` (transitively-inherited
        // `On`/tier declarations are v0.3, `attribute-classes.md`'s A-1 "at
        // most one tier per class" scope). Read before `expand_class_attributes`
        // consumes `class_def` and before `ctx` borrows `self.vm.interner`
        // mutably (borrow-order matters, see the handoff's "still unknown"
        // list — `class_parents` is read-only here, `interner` mutable).
        let is_attribute_class = class_def.superclass.as_ref().is_some_and(|sc| sc.name == "Attribute");
        let mut ctx = ExpandCtx {
            interner: &mut self.vm.interner,
            compile_mode: self.vm.compile_mode,
            strip_metadata,
            class_parents: &self.vm.class_parents,
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
        // Retained class-level attributes (M-ATTR-ROOT): needed after
        // `DefineGlobal` below, once `class_def.members` has been moved out
        // by the member loop — captured here rather than re-read later.
        let class_level_attrs = std::mem::take(&mut class_def.attributes);

        let range = class_def.range;
        let name_sym = self.vm.interner.intern(&class_def.name);
        let name_idx = self.add_constant(Value::Symbol(name_sym));

        // Pass 0: class-side selector collision check. A `construct` and a
        // `static` method of the same name/arity encode to one selector and
        // both install on the metaclass, so the later would silently clobber
        // the earlier. Reject the pair rather than let declaration order pick a
        // winner (ruled: compile error; cf. `derive_data`'s sibling
        // `attr.accessor_collision` check for a derived-vs-hand-written
        // `@construct`).
        let construct_selectors: Vec<String> = class_def
            .members
            .iter()
            .filter_map(|m| match m {
                ClassMember::Construct(c) => {
                    let labels: Vec<Option<String>> = c.params.iter().map(|p| p.label.clone()).collect();
                    Some(encode_selector(&c.name, &labels, SignatureKind::Method(c.params.len() as u8)))
                }
                _ => None,
            })
            .collect();
        for member in &class_def.members {
            if let ClassMember::Method(m) = member
                && m.is_static
            {
                let labels: Vec<Option<String>> = m.params.iter().map(|p| p.label.clone()).collect();
                let selector = encode_selector(&m.name, &labels, SignatureKind::Method(m.params.len() as u8));
                if construct_selectors.contains(&selector) {
                    return Err(CompilerError::ConstructStaticCollision(class_def.name.clone(), selector, m.name_range));
                }
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
                    if g.name.starts_with('_') {
                        let f = self.vm.interner.intern(&g.name);
                        if !own_static_fields.contains(&f) {
                            own_static_fields.push(f);
                        }
                    }
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
                ClassMember::Field(f) => Some(self.vm.interner.intern(&f.name)),
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
                        if g.name.starts_with('_') {
                            let f = self.vm.interner.intern(&g.name);
                            if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                own_instance_fields.push(f);
                            }
                        }
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
                    ClassMember::Construct(c) => {
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

        // 1b. Reopen-conflict guard (U-REOPEN-FIX, ADR-0018 "attaches
        // methods, not shadows"). `self.vm.field_layouts` is keyed by
        // class name and persists for the VM's whole lifetime — a
        // second `class A { ... }` block for a name already present
        // there is reopening `A`, whether that earlier block was
        // compiled earlier in *this* unit or in a prior REPL/compile
        // call. (A bootstrap Rust stub does NOT insert into
        // `field_layouts` itself, so a `.ph` class's *first* body is
        // never mistaken for a reopen.)
        //
        // Two reopen shapes are out of scope and rejected here rather
        // than silently mishandled:
        //   - adding fields: the runtime reopen path (`vm.rs`
        //     `Bytecode::Class`) reuses the already-allocated
        //     `ClassId` and does not relayout its instances, so a
        //     later block's new fields would never get real storage.
        //   - changing the superclass: U13 sealed inheritance forbids
        //     it, and reusing the original `ClassId` with a different
        //     superclass would silently keep the old one.
        if self.vm.field_layouts.contains_key(&name_sym) {
            if !own_instance_fields.is_empty() || !own_static_fields.is_empty() {
                return Err(CompilerError::Message(format!(
                    "Cannot reopen class `{}`: adding fields to an already-defined class is not supported. Declare all fields in the class's first `class {}` block.",
                    class_def.name, class_def.name
                )));
            }
            let prev_superclass_sym = self.vm.class_parents.get(&name_sym).copied();
            let new_superclass_sym = class_def.superclass.as_ref().map(|sc_ref| self.vm.interner.intern(&sc_ref.name));
            if prev_superclass_sym != new_superclass_sym {
                return Err(CompilerError::Message(format!(
                    "Cannot reopen class `{}` with a different superclass: it was already defined with a different (or no) `extends` clause.",
                    class_def.name
                )));
            }
        }

        // 2. Build the ClassLayout and store it in VM.
        //
        // A subclass's own fields stack on top of the superclass's fields
        // (ADR-0011, U-INH §3.5): own instance/static slots begin at the
        // superclass's field count, so inherited slots keep their offsets
        // and are never aliased. The superclass's counts are resolved at
        // COMPILE time — from a reopened class already in `vm.classes`
        // (the bootstrap case: a Rust stub pre-registers the class before
        // its `.ph` body compiles, so `field_layouts` has no entry for it
        // yet and the reopen branch below does not fire), from the
        // `extends` clause (looked up in the accumulating
        // `field_layouts`/`classes` metadata, since a *user* superclass
        // has not been created at runtime yet), or the implicit `Object`
        // root.
        let layout = if let Some(existing_layout) = self.vm.field_layouts.get(&name_sym).cloned() {
            // U-REOPEN-FIX (field-layout preservation): a genuine
            // same-unit reopen is detected here by
            // `field_layouts.contains_key` — the same
            // compile-time-synchronous signal the field-adding guard
            // above already used, NOT the RUNTIME-populated
            // `self.vm.classes` (still empty for a same-unit reopen at
            // this point, since the whole unit compiles to one closure
            // before any `Bytecode::Class` executes). That guard already
            // rejected any reopen that declares new fields, so
            // `own_instance_fields`/`own_static_fields` are guaranteed
            // empty here — the existing layout (including its
            // `field_slots`/`static_field_slots` name maps, not just its
            // counts) is reused completely unchanged rather than
            // recomputed from a superclass or rebuilt with an empty
            // `field_slots`. Recomputing here was the root cause of a
            // reviewer-found regression: a method-only reopen of a
            // *field-bearing* class fell through to the "implicit Object
            // root" branch below (`sc_field_count = 0`), silently
            // overwriting block 1's real layout with an empty one before
            // any bytecode ran (`create_class`, vm.rs, then read the
            // zeroed layout and produced an out-of-bounds field slot).
            existing_layout
        } else {
            let (sc_field_count, sc_meta_field_count) = if let Some(&existing_class) = self.vm.classes.get(&name_sym) {
                // Bootstrap reopen: keep the Rust stub's established superclass.
                match self.vm.heap.class(existing_class).superclass {
                    Some(sc_id) => {
                        let meta = self.vm.heap.class(sc_id).class;
                        (self.vm.heap.class(sc_id).field_count, self.vm.heap.class(meta).field_count)
                    }
                    None => (0, 0),
                }
            } else if let Some(sc_ref) = &class_def.superclass {
                let sc_sym = self.vm.interner.intern(&sc_ref.name);
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
                if let Some(&sealed_in_module) = self.vm.sealed_classes.get(&sc_sym) {
                    if sealed_in_module != self.module {
                        return Err(CompilerError::Message(format!(
                            "attr.sealed_violation: `{}` extends `@sealed` class `{}`, but was not declared in the same compilation unit",
                            class_def.name, sc_ref.name
                        )));
                    }
                }
                let counts = if let Some(layout) = self.vm.field_layouts.get(&sc_sym) {
                    (layout.field_count, layout.static_field_count)
                } else if let Some(&sc_id) = self.vm.classes.get(&sc_sym) {
                    let meta = self.vm.heap.class(sc_id).class;
                    (self.vm.heap.class(sc_id).field_count, self.vm.heap.class(meta).field_count)
                } else {
                    return Err(CompilerError::Message(format!(
                        "Unknown superclass `{}`: it must be a class defined before `{}`.",
                        sc_ref.name, class_def.name
                    )));
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
                // `class B extends A`, then `class A extends B`), which the
                // `visited` guard in both chain-walks handles without spinning.
                self.vm.class_parents.insert(name_sym, sc_sym);
                counts
            } else {
                // Implicit `Object` root.
                let object_class = self.vm.universe.classes.object_class;
                let meta = self.vm.heap.class(object_class).class;
                (self.vm.heap.class(object_class).field_count, self.vm.heap.class(meta).field_count)
            };

            let mut field_slots = IndexMap::new();
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
            }
        };
        self.vm.field_layouts.insert(name_sym, layout);

        self.current_class = Some(name_sym);

        if let Some(&existing_class) = self.vm.classes.get(&name_sym) {
            let class_idx = self.add_constant(Value::Obj(existing_class));
            self.emit(Bytecode::Constant(class_idx), range);
        } else {
            // Push the superclass onto the stack for the `Class` handler
            // to consume (vm.rs `Bytecode::Class` pops it and wires both
            // `superclass` and the parallel metaclass via `create_class`,
            // ADR-0002 rule 4). An explicit `extends S` resolves `S` as an
            // ordinary global at runtime; with no `extends` the class
            // implicitly inherits from `Object`.
            if let Some(sc_ref) = &class_def.superclass {
                let sc_sym = self.vm.interner.intern(&sc_ref.name);
                let sc_name_idx = self.add_constant(Value::Symbol(sc_sym));
                self.emit(Bytecode::GetGlobal(sc_name_idx), sc_ref.range);
            } else {
                let object_class = self.vm.universe.classes.object_class;
                let superclass_idx = self.add_constant(Value::Obj(object_class));
                self.emit(Bytecode::Constant(superclass_idx), range);
            }
            self.emit(Bytecode::Class(name_idx), range);
        }

        // The class object is now on top of the stack. Iterate through members.
        for member in class_def.members {
            match member {
                ClassMember::Method(method_def) => {
                    let range = method_def.range;

                    let arity = method_def.params.len();
                    // At most one parameter may be the rest parameter, and only
                    // as the list's last entry — enforced by
                    // `parse_param_list` for every OTHER position, but a rest
                    // parameter that isn't last can still reach here if it is
                    // the only param (`self.is_rest` false paths aside, this
                    // guards the compiler's own invariant defensively, per U9's
                    // write-set: "reject any other param in the list with
                    // is_rest set").
                    if let Some(bad) = method_def.params.iter().take(arity.saturating_sub(1)).find(|p| p.is_rest) {
                        return Err(CompilerError::Message(format!(
                            "rest parameter \"*{}\" must be the last parameter of \"{}\"",
                            bad.name, method_def.name
                        )));
                    }
                    let is_variadic = method_def.params.last().is_some_and(|p| p.is_rest);

                    let sig_kind = if is_variadic {
                        // The rest parameter itself occupies one local slot
                        // beyond the `F` fixed parameters; `F` is the payload
                        // U9 corrections §0 point 3 requires for
                        // `SignatureKind::Variadic`, never spelled into the
                        // selector text.
                        let fixed_arity = (arity - 1) as u8;
                        SignatureKind::Variadic(fixed_arity)
                    } else {
                        SignatureKind::Method(arity as u8)
                    };

                    let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                    let selector = encode_selector(&method_def.name, &labels, sig_kind);
                    let selector_sym = self.vm.interner.intern(&selector);

                    let param_names: Vec<String> = method_def.params.iter().map(|p| p.name.clone()).collect();
                    self.is_static_context = method_def.is_static;
                    let closure = self.compile_block(method_def.body, selector_sym, param_names, true, false)?;

                    tracing::debug!("[Compiler] Compiling method: {} (static: {})", selector, method_def.is_static);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        sig_kind,
                        MethodKind::Closure(closure),
                    ))));

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&method_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, method_def.is_static), range);

                    self.emit_member_attribute_attaches(&method_def.attributes, method_obj_idx, range)?;
                }
                ClassMember::Getter(getter_def) => {
                    let range = getter_def.range;

                    if getter_def.name.starts_with('_') {
                        let layout = self.vm.field_layouts.get(&name_sym).unwrap().clone();
                        let field_name_sym = self.vm.interner.intern(&getter_def.name);
                        let slot = if getter_def.is_static {
                            *layout.static_field_slots.get(&field_name_sym).ok_or_else(|| {
                                CompilerError::Message(format!("Static field slot not found: {}", getter_def.name))
                            })?
                        } else {
                            *layout.field_slots.get(&field_name_sym).ok_or_else(|| {
                                CompilerError::Message(format!("Instance field slot not found: {}", getter_def.name))
                            })?
                        };

                        self.emit(Bytecode::Dup, range);
                        if let Statement::Expr { expr, .. } = &getter_def.body[0] {
                            self.compile_expr(expr.clone())?;
                        } else {
                            return Err(CompilerError::Message("Invalid field initializer body".to_string()));
                        }
                        self.emit(Bytecode::SetField(slot), range);
                        self.emit(Bytecode::Pop, range);
                    } else {
                        let selector = make_signature(&getter_def.name, SignatureKind::Getter);
                        let selector_sym = self.vm.interner.intern(&selector);

                        self.is_static_context = getter_def.is_static;
                        let closure = self.compile_block(getter_def.body, selector_sym, Vec::new(), true, false)?;

                        tracing::debug!("[Compiler] Compiling getter: {} (static: {})", selector, getter_def.is_static);

                        let method_obj =
                            self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(selector_sym, SignatureKind::Getter, MethodKind::Closure(closure)))));

                        if !strip_metadata {
                            let contracts = self.build_contracts_metadata(&getter_def.attributes)?;
                            if !contracts.is_empty() {
                                self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                            }
                        }

                        let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                        self.emit(Bytecode::Constant(method_obj_idx), range);

                        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                        self.emit(Bytecode::Method(selector_idx, getter_def.is_static), range);

                        self.emit_member_attribute_attaches(&getter_def.attributes, method_obj_idx, range)?;
                    }
                }
                ClassMember::Setter(setter_def) => {
                    let range = setter_def.range;

                    let selector = make_signature(&setter_def.name, SignatureKind::Setter);
                    let selector_sym = self.vm.interner.intern(&selector);

                    self.is_static_context = setter_def.is_static;
                    let closure = self.compile_block(setter_def.body, selector_sym, vec![setter_def.param.clone()], true, false)?;

                    tracing::debug!("[Compiler] Compiling setter: {} (static: {})", selector, setter_def.is_static);

                    let method_obj =
                        self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(selector_sym, SignatureKind::Setter, MethodKind::Closure(closure)))));

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&setter_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, setter_def.is_static), range);

                    self.emit_member_attribute_attaches(&setter_def.attributes, method_obj_idx, range)?;
                }
                ClassMember::Construct(construct_def) => {
                    let range = construct_def.range;

                    let arity = construct_def.params.len();
                    let labels: Vec<Option<String>> = construct_def.params.iter().map(|p| p.label.clone()).collect();
                    // A constructor installs under the *ordinary* selector its
                    // call sites already encode (`Counter.new()` -> `new()`),
                    // on the metaclass (`Bytecode::Method(_, is_static: true)`
                    // below). The parallel metaclass tower (ADR-0002,
                    // object-model §5) then resolves it exactly as it already
                    // resolves the kernel's own class-side `new` primitives
                    // (`List.new()` et al, `universe/primitives.rs`): this
                    // class's metaclass entry shadows the bare allocator
                    // `Class >> new()` inherited from the tower root, by
                    // ordinary lookup, for **any** receiver expression.
                    //
                    // The `Signature`'s *kind* stays `Initializer` — it no
                    // longer picks the selector, and survives purely as the
                    // "this is a constructor" marker that `Bytecode::SuperSend`
                    // gates its super-construct metaclass hop on (U-INH §3.5).
                    let selector = encode_selector(&construct_def.name, &labels, SignatureKind::Method(arity as u8));
                    let selector_sym = self.vm.interner.intern(&selector);

                    // Record the declared constructor selector. Dispatch no
                    // longer consults this — it exists only so the *arity*
                    // guard in `expr.rs` stays a compile-time diagnostic:
                    // `has_new_construct` answers "does a `new` constructor
                    // exist in this chain at all", `constructor_aliases`
                    // answers "does one match this exact selector", and a
                    // yes/no pair is what proves a `Foo.new(...)` call would
                    // otherwise fall through to the bare allocator.
                    let class_name_sym = self.current_class.expect("construct is only compiled within a class body");
                    self.vm.constructor_aliases.insert((class_name_sym, selector_sym), selector_sym);
                    if construct_def.name == "new" {
                        self.vm.has_new_construct.insert(class_name_sym);
                    }

                    let param_names: Vec<String> = construct_def.params.iter().map(|p| p.name.clone()).collect();

                    self.is_static_context = false;
                    let closure = self.compile_block(construct_def.body, selector_sym, param_names, true, true)?;

                    tracing::debug!("[Compiler] Compiling constructor: {}", selector);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        SignatureKind::Initializer(arity as u8),
                        MethodKind::Closure(closure),
                    ))));

                    let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                    self.emit(Bytecode::Method(selector_idx, true), range);
                }
                // A declared field is layout-only (U-ANNOT-LAYOUT §3.1): it
                // already fed `own_instance_fields`/`field_slots` above and
                // emits no bytecode of its own. Its `default` expression, if
                // any, is data for a layout-derive attribute
                // (`@construct`/`@data`) to read — not compiled here.
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
                    let arity = index_def.params.len();
                    let labels: Vec<Option<String>> = index_def.params.iter().map(|p| p.label.clone()).collect();
                    let sig_kind = SignatureKind::Subscript(arity as u8);
                    let selector = encode_selector("", &labels, sig_kind);
                    let selector_sym = self.vm.interner.intern(&selector);

                    let param_names: Vec<String> = index_def.params.iter().map(|p| p.name.clone()).collect();
                    self.is_static_context = false;
                    let closure = self.compile_block(index_def.body, selector_sym, param_names, true, false)?;

                    tracing::debug!("[Compiler] Compiling subscript method: {}", selector);

                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                        selector_sym,
                        sig_kind,
                        MethodKind::Closure(closure),
                    ))));

                    if !strip_metadata {
                        let contracts = self.build_contracts_metadata(&index_def.attributes)?;
                        if !contracts.is_empty() {
                            self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                        }
                    }

                    let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                    self.emit(Bytecode::Constant(method_obj_idx), range);

                    let selector_idx = self.add_constant(Value::Symbol(selector_sym));
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
        // Define it as a global variable.
        self.emit(Bytecode::DefineGlobal(name_idx), range);

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
        // definition (A-5, `attribute-classes.md`'s hazards table) — further
        // `__attach` calls raise `attr.frozen`. Always emitted, even for a
        // class with no attributes at all, so the invariant holds uniformly.
        self.emit(Bytecode::GetGlobal(name_idx), range);
        self.emit_freeze_attributes(range);

        // U-ANNOT-LAYOUT §3.4: record `@sealed`'s compile-unit-scoped
        // bookkeeping now that `class_def.name`'s global is fully defined —
        // any subclass compiled *after* this point (this unit or another)
        // checks `VM::sealed_classes` at the point in `Self::compile_class`
        // above where its own superclass is resolved.
        if class_level_attrs.iter().any(|a| a.name == "sealed") {
            self.vm.sealed_classes.insert(name_sym, self.module);
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
                self.compile_class(sibling_def)?;
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
            object: Expr::Var { value: attr.name.clone(), range },
            method: "new".to_string(),
            args: attr.args.iter().map(|expr| Argument { label: None, expr: expr.clone(), range }).collect(),
            range,
        }));
        self.compile_expr(ctor_call)?;

        let attach_selector = make_signature("__attach", SignatureKind::Method(1));
        let attach_sym = self.vm.interner.intern(&attach_selector);
        let attach_idx = self.add_constant(Value::Symbol(attach_sym));
        self.emit(Bytecode::Invoke(1, attach_idx), range);
        self.emit(Bytecode::Pop, range);
        Ok(())
    }

    /// Sends `__freezeAttributes()` against whatever receiver is already on
    /// top of the stack, then pops its `None` result — see
    /// [`Self::emit_attribute_attach`] for the receiver-already-pushed
    /// convention.
    fn emit_freeze_attributes(&mut self, range: SourceRange) {
        let freeze_selector = make_signature("__freezeAttributes", SignatureKind::Method(0));
        let freeze_sym = self.vm.interner.intern(&freeze_selector);
        let freeze_idx = self.add_constant(Value::Symbol(freeze_sym));
        self.emit(Bytecode::Invoke(0, freeze_idx), range);
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
                let closure_ref = self.compile_block(body, name_sym, Vec::new(), true, false)?;
                contracts.push((name_sym, Value::Obj(closure_ref)));
            }
        }
        Ok(contracts)
    }

    /// Reports whether `class_sym` or any of its compile-time ancestors
    /// (via [`crate::vm::VM::class_parents`]) declares a
    /// `new`-named `construct`.
    ///
    /// Inheritance-aware form of a bare `has_new_construct` membership test.
    /// A subclass that *inherits* a `new` constructor but declares none still
    /// has no user-visible bare allocator, so a mis-arity `Sub.new(...)` must
    /// error rather than silently fall through to the `Object.class::new`
    /// bare allocator and yield an uninitialized instance (U-INH follow-on;
    /// `docs/forge/DEFERRED.md` correctness entry). The walk terminates at the
    /// implicit `Object` root, which never has an edge in `class_parents`.
    pub(super) fn inherits_new_construct(&self, mut class_sym: Symbol) -> bool {
        // `class_parents` is normally a strict-ancestry DAG, but a reopen-
        // redefinition within a unit can form a back-edge (see the populate site);
        // the `visited` guard makes the walk terminate regardless, so the compiler
        // never spins.
        let mut visited = std::collections::HashSet::new();
        while visited.insert(class_sym) {
            if self.vm.has_new_construct.contains(&class_sym) {
                return true;
            }
            match self.vm.class_parents.get(&class_sym) {
                Some(&parent) => class_sym = parent,
                None => return false,
            }
        }
        false
    }

    /// Reports whether `class_sym` — or any ancestor, walking the compile-time
    /// superclass chain ([`crate::vm::VM::class_parents`]) — declares a
    /// `construct` whose selector is exactly `selector_sym`, returning that
    /// selector if so and `None` otherwise.
    ///
    /// Dispatch does **not** consult this: a constructor installs under the
    /// very selector its call sites encode, so the metaclass tower resolves it
    /// by ordinary lookup with no call-site rewrite. This survives only to keep
    /// the *arity* guard in [`Compiler::compile_expr`]'s `new(...)` arm a
    /// compile-time diagnostic wherever the receiver names a statically known
    /// class: paired with [`Self::inherits_new_construct`] ("does a `new`
    /// constructor exist in this chain at all"), a yes/no answer here proves a
    /// `Foo.new(...)` call matches no declared `construct` and would otherwise
    /// fall through to the bare allocator `Class >> new()`.
    ///
    /// [`Compiler::compile_expr`]: crate::compiler::Compiler
    pub(super) fn lookup_constructor_alias(&self, mut class_sym: Symbol, selector_sym: Symbol) -> Option<Symbol> {
        // Nearest-declared wins: the walk checks each class before its parent, so
        // a subclass `construct` shadows an ancestor's of the same selector. The
        // `visited` guard terminates the walk even on a reopen-redefinition back-
        // edge (see [`Self::inherits_new_construct`]).
        let mut visited = std::collections::HashSet::new();
        while visited.insert(class_sym) {
            if let Some(&alias) = self.vm.constructor_aliases.get(&(class_sym, selector_sym)) {
                return Some(alias);
            }
            match self.vm.class_parents.get(&class_sym) {
                Some(&parent) => class_sym = parent,
                None => return None,
            }
        }
        None
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
                collect_assigned_fields(&arg.expr, fields, interner);
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
                collect_assigned_fields(&arg.expr, fields, interner);
            }
        }
        Expr::SetIndex(six) => {
            collect_assigned_fields(&six.object, fields, interner);
            for arg in &six.args {
                collect_assigned_fields(&arg.expr, fields, interner);
            }
            collect_assigned_fields(&six.value, fields, interner);
        }
        Expr::Block(block) => {
            for stmt in &block.body {
                collect_assigned_fields_stmt(stmt, fields, interner);
            }
        }
        _ => {}
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
            collect_assigned_fields(&for_stmt.iter, fields, interner);
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
