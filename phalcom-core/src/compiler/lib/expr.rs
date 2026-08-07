use crate::bytecode::Bytecode;
use crate::compiler::inliner;
use crate::method::{SignatureKind, encode_selector, make_signature};
use crate::value::Value;
use phalcom_ast::ast::{BinaryOp, BlockExpr, Expr, MethodRefKind, ProductLabel, RecordLiteralField, Statement, SymbolLiteralKind, TupleLiteralEntry, UnaryOp};
use phalcom_common::range::SourceRange;

use super::checked_send_arity;
use super::error::CompilerError;
use super::{Compiler, UnitKind};

impl<'vm> Compiler<'vm> {
    /// Compiles `expr`, always leaving exactly one value on the stack.
    /// Equivalent to `compile_expr_want(expr, true)` — see that method for
    /// `want_value`.
    pub(crate) fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        self.compile_expr_want(expr, true)
    }

    /// Compiles `expr`, always leaving exactly one value on the stack.
    /// `want_value` is `false` only when the immediate caller is about to
    /// discard that value with a `Pop` right after
    /// (`compile_statement_with_pop_control`'s bare-statement case) — it lets
    /// a recognized one-armed sacred conditional (`ifTrue`/`ifFalse`) skip
    /// its `Some`-wrap allocation, since the wrap is unobservable when the
    /// value is popped unread (U-CORE-2; see
    /// [`Self::compile_sacred_call_want`]). Every other expression shape
    /// ignores `want_value` — it still pushes its one value as normal.
    pub(crate) fn compile_expr_want(&mut self, expr: Expr, want_value: bool) -> Result<(), CompilerError> {
        match expr {
            Expr::MethodCall(method_call) => {
                // A `super.sel(args)` send lowers to `SuperSend`, never an
                // ordinary `Invoke` — and must be intercepted *before* the
                // sacred inliner, so a `super.ifTrue { … }` is a real dispatch
                // starting above the defining class, not an inlined fast path
                // keyed on the receiver's static type (U-INH §3.4).
                if matches!(&method_call.object, Expr::SuperVar { .. }) {
                    let mc = *method_call;
                    let argc = mc.args.len();
                    let labels: Vec<Option<String>> = mc.args.iter().map(|a| a.label.clone()).collect();
                    // ADR-0063 splits a source constructor into a class-side
                    // factory and an instance-side `init <name>` method.
                    // Rewrite only the matching super-constructor send:
                    // ordinary `super` sends inside an initializer retain
                    // their source selector.
                    let method = match self.functions.last().unwrap().constructor_name.as_deref() {
                        Some(constructor_name) if mc.method == constructor_name => {
                            format!("init {constructor_name}")
                        }
                        _ => mc.method.clone(),
                    };
                    let argc = checked_send_arity("super send", argc, mc.range)?;
                    let selector = encode_selector(&method, &labels, SignatureKind::Method(argc));
                    let selector_sym = self.vm.interner.intern(&selector);
                    return self.compile_super_send(selector_sym, mc.args, argc, mc.range);
                }

                // U5 Layer 1 (control-flow.md §3, ADR-0018): a sacred
                // selector sent with literal-block arguments compiles to a
                // guarded inline fast path instead of a plain send. Every
                // other call — including a sacred selector with a
                // *non*-literal block argument — falls through unchanged.
                let range = method_call.range;
                // Inside a sacred call's deopt-fallback copy, skip recognition
                // and take the ordinary-send arm below: the copy is cold, and
                // inlining within it is what makes nested conditionals cost
                // 2^depth to compile (perf-log F13).
                let recognized = if self.in_deopt_fallback() {
                    Err(*method_call)
                } else {
                    inliner::recognize(*method_call)
                };
                match recognized {
                    Ok(sacred) => {
                        // BD-U6-1 (ADR-0007, values-and-absence §3.5): a
                        // conditional's condition that is a syntactically
                        // detectable `Option` literal (`if (None) { … }`,
                        // `if (Some.new(x)) { … }`, `None and …`) is a compile
                        // error — `Option` has no truth value. General
                        // non-`Bool` conditions are a hard runtime type error
                        // via the branch opcode's `Bool` requirement.
                        if let Some(condition) = branch_condition_of(&sacred)
                            && is_option_literal(condition)
                        {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        return self.compile_sacred_call_want(sacred, range, want_value);
                    }
                    Err(method_call) => {
                        let receiver_class_sym = match &method_call.object {
                            Expr::Var { value, .. } => Some(self.vm.interner.intern(value)),
                            _ => None,
                        };

                        let arity = checked_send_arity("message send", method_call.args.len(), method_call.range)?;
                        let labels: Vec<Option<String>> = method_call.args.iter().map(|a| a.label.clone()).collect();
                        let selector = encode_selector(&method_call.method, &labels, SignatureKind::Method(arity));
                        let selector_sym = self.vm.interner.intern(&selector);

                        self.compile_expr(method_call.object)?;
                        for arg in &method_call.args {
                            self.compile_expr(arg.expr.clone())?;
                        }
                        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                        self.emit(Bytecode::Invoke(arity, selector_idx), method_call.range);
                    }
                }
            }
            Expr::MethodRef(method_ref) => {
                // `receiver::name` / `receiver::#sel(...)` (selectors.md §3,
                // U16-Open, U16-Pinned): compile the receiver, intern the
                // reference's symbol as a constant, and let
                // `Bytecode::MakeFamily`'s runtime handler do the
                // reference-time empty-family check + `Family` allocation.
                // The two shapes share one opcode: Open interns a bare base
                // name, Pinned interns the full selector through the same
                // `encode_selector` a matching method definition uses
                // (ADR-0012) — the runtime handler tells them apart by
                // whether the interned string contains `(` (`VM`'s
                // `Bytecode::MakeFamily` arm).
                let method_ref = *method_ref;
                let sym = match method_ref.kind {
                    MethodRefKind::Open { name } => self.vm.interner.intern(&name),
                    MethodRefKind::Pinned { name, labels } => {
                        let arity = checked_send_arity("pinned selector", labels.len(), method_ref.range)?;
                        let selector = encode_selector(&name, &labels, SignatureKind::Method(arity));
                        self.vm.interner.intern(&selector)
                    }
                };
                self.compile_expr(method_ref.receiver)?;
                let name_idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::MakeFamily(name_idx), method_ref.range);
            }
            Expr::GetProperty(get_prop) => {
                // `super.prop` is a zero-arg super send (U-INH §3.4); the
                // getter/no-arg selector is the bare property name, matching the
                // ordinary getter dispatch below.
                if matches!(&get_prop.object, Expr::SuperVar { .. }) {
                    let selector_sym = self.vm.interner.intern(&get_prop.property);
                    return self.compile_super_send(selector_sym, Vec::new(), 0, get_prop.range);
                }
                self.compile_expr(get_prop.object)?;
                let selector_sym = self.vm.interner.intern(&get_prop.property);
                let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                self.emit(Bytecode::Invoke(0, selector_idx), get_prop.range);
            }
            Expr::SetProperty(set_prop) => {
                self.compile_expr(set_prop.object)?;
                self.compile_expr(set_prop.value)?;
                let selector = make_signature(&set_prop.property, SignatureKind::Setter);
                let selector_sym = self.vm.interner.intern(&selector);
                let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                self.emit(Bytecode::Invoke(1, selector_idx), set_prop.range);
            }
            Expr::Index(ix) => {
                // U-INDEX (ADR-0060): `xs[a, b, ...]` sends directly to the
                // bracket selector the args' arity/labels encode (`[_]`,
                // `[_,_]`, `[default]`, ...) — no `at`/`at(_,put:)` lowering.
                // `args` is a full call-shaped argument list (reusing
                // `parse_arg_list`), so this rides the ordinary generic-send
                // path for any arity/label combination without further
                // compiler changes.
                let ix = *ix;
                let argc = checked_send_arity("subscript read", ix.args.len(), ix.range)?;
                let labels: Vec<Option<String>> = ix.args.iter().map(|a| a.label.clone()).collect();
                self.compile_expr(ix.object)?;
                for arg in ix.args {
                    self.compile_expr(arg.expr)?;
                }
                let selector = encode_selector("", &labels, SignatureKind::Subscript(argc));
                let sym = self.vm.interner.intern(&selector);
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Invoke(argc, idx), ix.range);
            }
            Expr::SetIndex(six) => {
                // U-INDEX (ADR-0060): `xs[a, b, ...] = value` appends `value`
                // as the selector's trailing `put:` argument — `xs[i] = v`
                // sends `[_,put]`, `xs[] = v` sends `[put]` — never `at`.
                let six = *six;
                let mut labels: Vec<Option<String>> = six.args.iter().map(|a| a.label.clone()).collect();
                labels.push(Some("put".to_string()));
                let argc = checked_send_arity("subscript write", labels.len(), six.range)?;
                self.compile_expr(six.object)?;
                for arg in six.args {
                    self.compile_expr(arg.expr)?;
                }
                self.compile_expr(six.value)?;
                let selector = encode_selector("", &labels, SignatureKind::Subscript(argc));
                let sym = self.vm.interner.intern(&selector);
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Invoke(argc, idx), six.range);
            }
            Expr::Int { digits, radix, range } => {
                let val = if radix == 10 {
                    if let Ok(i) = digits.parse::<i64>() {
                        Value::Int(i)
                    } else if let Some(big) = num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10) {
                        let obj = self.vm.heap.alloc(crate::heap::Object::LargeInt(big));
                        Value::Obj(obj)
                    } else {
                        return Err(CompilerError::Message(format!("Invalid integer literal: {digits}")));
                    }
                } else {
                    if let Ok(i) = i64::from_str_radix(&digits, radix) {
                        Value::Int(i)
                    } else if let Some(big) = num_bigint::BigInt::parse_bytes(digits.as_bytes(), radix) {
                        let obj = self.vm.heap.alloc(crate::heap::Object::LargeInt(big));
                        Value::Obj(obj)
                    } else {
                        return Err(CompilerError::Message(format!("Invalid radix integer literal: {digits}")));
                    }
                };
                let idx = self.add_constant(val);
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::Float { value, range } => {
                let idx = self.add_constant(Value::Float(value));
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::String { value, range } => {
                let string_obj = self.vm.alloc_string_value(value);
                let idx = self.add_constant(string_obj);
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::Boolean { value, range } => {
                if value {
                    self.emit(Bytecode::True, range);
                } else {
                    self.emit(Bytecode::False, range);
                }
            }
            Expr::Symbol(symbol_expr) => {
                let range = symbol_expr.range;
                // Both symbol shapes intern to a `Value::Symbol` constant
                // (selectors.md §2). A selector symbol is canonicalized
                // through `encode_selector` — the same routine method
                // definitions use — so `#move(_,to)` interns to the *same*
                // `Symbol` as the selector a `move(_,to:)` method definition
                // registers (ADR-0012).
                let canonical = match symbol_expr.kind {
                    SymbolLiteralKind::Name(name) => name,
                    SymbolLiteralKind::Selector { name, labels } => {
                        let arity = checked_send_arity("pinned selector", labels.len(), range)?;
                        encode_selector(&name, &labels, SignatureKind::Method(arity))
                    }
                };
                let sym = self.vm.interner.intern(&canonical);
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::TupleLiteral(tuple_expr) => {
                let tuple_expr = *tuple_expr;
                if tuple_expr.entries.is_empty() {
                    let idx = self.add_constant(Value::Unit);
                    self.emit(Bytecode::Constant(idx), tuple_expr.range);
                    return Ok(());
                }
                let positional = tuple_expr
                    .entries
                    .iter()
                    .filter(|entry| matches!(entry, TupleLiteralEntry::Positional { .. }))
                    .count();
                let labeled = tuple_expr.entries.len() - positional;
                let mut seen = std::collections::HashSet::new();
                for entry in tuple_expr.entries {
                    match entry {
                        TupleLiteralEntry::Positional { expr, .. } => self.compile_expr(expr)?,
                        TupleLiteralEntry::Labeled { label, value, range } => {
                            self.compile_product_label(label, &mut seen, range)?;
                            self.compile_expr(value)?;
                        }
                    }
                }
                self.emit(
                    Bytecode::BuildTuple {
                        positional: positional as u16,
                        labeled: labeled as u16,
                    },
                    tuple_expr.range,
                );
            }
            Expr::RecordLiteral(record_expr) => {
                let record_expr = *record_expr;
                if record_expr.fields.is_empty() {
                    let idx = self.add_constant(Value::Unit);
                    self.emit(Bytecode::Constant(idx), record_expr.range);
                    return Ok(());
                }
                let mut seen = std::collections::HashSet::new();
                let fields = record_expr.fields.len();
                for RecordLiteralField { label, value, range } in record_expr.fields {
                    self.compile_product_label(label, &mut seen, range)?;
                    self.compile_expr(value)?;
                }
                self.emit(Bytecode::BuildRecord { fields: fields as u16 }, record_expr.range);
            }
            Expr::Var { value, range } => {
                if value == "nil" {
                    return Err(CompilerError::UndefinedVariable(value.clone()));
                }
                let name_sym = self.vm.interner.intern(&value);
                if let Some(slot) = self.resolve_local(name_sym) {
                    self.emit(Bytecode::GetLocal(slot as u16), range);
                } else if let Some(upvalue) = self.resolve_upvalue(name_sym) {
                    self.emit(Bytecode::GetUpvalue(upvalue as u16), range);
                } else {
                    let name_idx = self.add_constant(Value::Symbol(name_sym));
                    self.emit(Bytecode::GetGlobal(name_idx), range);
                }
            }
            Expr::Field { value, range } => {
                let name_sym = self.vm.interner.intern(&value);
                let class_key = self
                    .current_class
                    .ok_or_else(|| CompilerError::Message(format!("Fields can only be accessed within a class: {}", value)))?;
                let layout = self
                    .vm
                    .field_layouts
                    .get(&class_key)
                    .cloned()
                    .ok_or_else(|| CompilerError::Message(format!("No layout registered for class: {}", self.vm.resolve_symbol(class_key.name))))?;

                if let Some(&slot) = layout.static_field_slots.get(&name_sym) {
                    if self.is_static_context {
                        self.emit_self(range);
                    } else {
                        self.emit_self(range);
                        let class_sym = self.vm.interner.intern("class");
                        let class_idx = self.add_constant(Value::Symbol(class_sym));
                        self.emit(Bytecode::Invoke(0, class_idx), range);
                    }
                    self.emit(Bytecode::GetField(slot), range);
                } else if let Some(&slot) = layout.field_slots.get(&name_sym) {
                    self.emit_self(range);
                    self.emit(Bytecode::GetField(slot), range);
                } else {
                    return Err(CompilerError::ReadBeforeWrite(value.clone()));
                }
            }
            Expr::Assignment(assign_expr) => {
                match *assign_expr.name {
                    Expr::Var { value, range } => {
                        let name_sym = self.vm.interner.intern(&value);
                        // Enforce `const` immutability (ADR-0064, L-3) before
                        // evaluating the RHS. Resolution order mirrors the
                        // emit order below: current-function local, then an
                        // enclosing captured local (upvalue), then a global.
                        if let Some(slot) = self.resolve_local(name_sym) {
                            if !self.functions.last().unwrap().locals[slot].is_mutable {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetLocal(slot as u16), range);
                        } else if let Some(upvalue) = self.resolve_upvalue(name_sym) {
                            // L-3 closes DEFERRED #13: a captured write to an
                            // outer `const` (reached through an upvalue) is
                            // rejected exactly like a direct write.
                            if self.resolve_captured_mutable(name_sym) == Some(false) {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetUpvalue(upvalue as u16), range);
                        } else {
                            let is_const_this_unit = self.global_bindings.get(&name_sym) == Some(&false);
                            let is_const_prior_unit =
                                self.unit_kind != UnitKind::Repl && self.vm.heap.module(self.module).global_bindings.get(&name_sym) == Some(&false);
                            if is_const_this_unit || is_const_prior_unit {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            let name_idx = self.add_constant(Value::Symbol(name_sym));
                            self.emit(Bytecode::SetGlobal(name_idx), range);
                        }
                    }
                    Expr::Field { value, range } => {
                        let name_sym = self.vm.interner.intern(&value);
                        let class_key = self
                            .current_class
                            .ok_or_else(|| CompilerError::Message(format!("Fields can only be accessed within a class: {}", value)))?;
                        let layout = self
                            .vm
                            .field_layouts
                            .get(&class_key)
                            .cloned()
                            .ok_or_else(|| CompilerError::Message(format!("No layout registered for class: {}", self.vm.resolve_symbol(class_key.name))))?;

                        if let Some(&slot) = layout.static_field_slots.get(&name_sym) {
                            if !self.in_constructor && layout.const_fields.contains(&name_sym) {
                                return Err(CompilerError::ConstFieldWrite(value));
                            }
                            if self.is_static_context {
                                self.emit_self(range);
                            } else {
                                self.emit_self(range);
                                let class_sym = self.vm.interner.intern("class");
                                let class_idx = self.add_constant(Value::Symbol(class_sym));
                                self.emit(Bytecode::Invoke(0, class_idx), range);
                            }
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetField(slot), range);
                        } else if let Some(&slot) = layout.field_slots.get(&name_sym) {
                            if !self.in_constructor && layout.const_fields.contains(&name_sym) {
                                return Err(CompilerError::ConstFieldWrite(value));
                            }
                            self.emit_self(range);
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetField(slot), range);
                        } else {
                            return Err(CompilerError::Message(format!("Field not collected in layout: {}", value)));
                        }
                    }
                    _ => return Err(CompilerError::InvalidAssignmentTarget),
                }
            }
            Expr::Binary(binary_expr) => {
                // U5 (control-flow.md §1): every binary operator is an
                // ordinary `Invoke` send — none of these are opcodes anymore.
                // `and`/`or` are the two *lazy* exceptions (control-flow.md
                // §2): their right operand compiles as a 0-arity block
                // literal, not a plain expression, so `Bool::and(_)`/`or(_)`
                // can choose whether to evaluate it at all. That literal
                // block is compiler-synthesized here but is exactly as
                // "literal" as a user-written `{ ... }` — U5-plan.md §4.2's
                // "literal block at the call site" inlining condition is
                // about the block's *shape*, not its origin — so `a and b`
                // is built directly as a recognized `SacredCall::And` and
                // handed to the same guarded-jump emitter `a.and { b }`
                // uses (`inliner.rs`), not a plain send.
                let range = binary_expr.range;
                match binary_expr.op {
                    BinaryOp::And => {
                        // The left operand is a branch condition (control-flow
                        // .md §2); reject a literal `Option` there (BD-U6-1).
                        if is_option_literal(&binary_expr.left) {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        let rhs_block = wrap_expr_as_lazy_block(binary_expr.right, range);
                        return self.compile_sacred_call(
                            inliner::SacredCall::And {
                                receiver: binary_expr.left,
                                rhs_block,
                            },
                            range,
                        );
                    }
                    BinaryOp::Or => {
                        if is_option_literal(&binary_expr.left) {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        let rhs_block = wrap_expr_as_lazy_block(binary_expr.right, range);
                        return self.compile_sacred_call(
                            inliner::SacredCall::Or {
                                receiver: binary_expr.left,
                                rhs_block,
                            },
                            range,
                        );
                    }
                    op => {
                        self.compile_expr(binary_expr.left)?;
                        self.compile_expr(binary_expr.right)?;
                        self.emit_operator_send(binary_op_selector_name(&op), 1, range);
                    }
                }
            }
            Expr::Unary(unary_expr) => {
                // U5: `-x`/`!x` lower to 0-arg sends (`negated()`/`not()`)
                // via the single `encode_selector` helper, replacing the
                // hand-rolled `"-"`/`"not"` lookup strings the old opcode
                // handlers used (ADR-0012 — "do not hand-roll a divergent
                // encoder", the F8 lesson).
                self.compile_expr(unary_expr.expr)?;
                let range = unary_expr.range;
                let name = match unary_expr.op {
                    UnaryOp::Negate => "negated",
                    UnaryOp::Not => "not",
                    UnaryOp::BitNot => "~",
                };
                self.emit_operator_send(name, 0, range);
            }
            Expr::SelfVar { range } => {
                self.emit_self(range);
            }
            Expr::SuperVar { range: _ } => {
                // A bare `super` that reaches here is not the receiver of a
                // message send (the `super.sel(…)` forms are intercepted in the
                // `MethodCall`/`GetProperty` arms). `super` has no value on its
                // own — it only redirects a send's lookup start (U-INH §3.4).
                return Err(CompilerError::BareSuper);
            }
            Expr::Block(block_expr) => {
                let name_sym = self.vm.interner.intern("<block>");
                let constructor_name = self.functions.last().unwrap().constructor_name.clone();
                let closure = self.compile_block(block_expr.body, name_sym, block_expr.params, false, false, constructor_name)?;
                let idx = self.add_constant(Value::Obj(closure));
                self.emit(Bytecode::Closure(idx), block_expr.range);
            } // Expr::Call(call_expr) => {
              //     // TODO: Implement function call compilation
              //     self.compile_expr(call_expr.callee)?;
              //     for arg in call_expr.args {
              //         self.compile_expr(arg)?;
              //     }
              //     // For now, push Nil as a placeholder for the return value
              //     self.emit(Bytecode::Nil);
              // }
        }
        Ok(())
    }
}

impl<'vm> Compiler<'vm> {
    fn compile_product_label(
        &mut self,
        label: ProductLabel,
        seen: &mut std::collections::HashSet<crate::interner::Symbol>,
        range: SourceRange,
    ) -> Result<(), CompilerError> {
        match label {
            ProductLabel::Static { symbol, .. } => {
                let sym = self.canonical_symbol(symbol, range)?;
                if !seen.insert(sym) {
                    return Err(CompilerError::Message(format!("duplicate product label `{}`", self.vm.resolve_symbol(sym))));
                }
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Constant(idx), range);
            }
            ProductLabel::Computed { expr, range } => {
                self.compile_expr(*expr)?;
                self.emit(Bytecode::GuardSymbol, range);
            }
        }
        Ok(())
    }

    fn canonical_symbol(&mut self, kind: SymbolLiteralKind, range: SourceRange) -> Result<crate::interner::Symbol, CompilerError> {
        let canonical = match kind {
            SymbolLiteralKind::Name(name) => name,
            SymbolLiteralKind::Selector { name, labels } => {
                let arity = checked_send_arity("symbol selector", labels.len(), range)?;
                encode_selector(&name, &labels, SignatureKind::Method(arity))
            }
        };
        Ok(self.vm.interner.intern(&canonical))
    }
}

/// Returns the branch-condition sub-expression of a recognized [`inliner::SacredCall`],
/// or `None` for the forms whose condition is a block rather than a plain
/// expression (`whileTrue`).
///
/// The condition is the receiver of the inlined conditional/short-circuit
/// selectors (`ifTrue:`/`ifFalse:`/`ifTrue:ifFalse:`/`and:`/`or:`); it is the
/// value the branch opcode tests, so it is exactly where the "no `Option`
/// truthiness" rule applies (BD-U6-1, values-and-absence §3.5).
fn branch_condition_of(sacred: &inliner::SacredCall) -> Option<&Expr> {
    match sacred {
        inliner::SacredCall::IfTrue { receiver, .. }
        | inliner::SacredCall::IfFalse { receiver, .. }
        | inliner::SacredCall::IfTrueIfFalse { receiver, .. }
        | inliner::SacredCall::And { receiver, .. }
        | inliner::SacredCall::Or { receiver, .. } => Some(receiver),
        inliner::SacredCall::WhileTrue { .. } => None,
    }
}

/// Reports whether `expr` is a syntactically detectable `Option` literal.
///
/// Matches the surface forms of the two `Option` cases that carry no truth
/// value ([ADR-0007](../../../docs/adr/accepted/0007-option-type.md)):
///
/// - the `None` singleton, which lexes to `Var { value: "None" }`; and
/// - a `Some.new(…)` construction — an [`Expr::MethodCall`] of `new` on the
///   `Some` class (Phalcom has no bare `Some(x)` call syntax, so construction
///   is always the explicit static `Some.new(x)` send).
///
/// This is the literal-only half of BD-U6-1's `if (opt)` compile check; every
/// non-literal, non-`Bool` condition is caught at runtime by the branch
/// opcode's `Bool` requirement.
fn is_option_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Var { value, .. } => value == "None",
        Expr::MethodCall(call) => call.method == "new" && matches!(&call.object, Expr::Var { value, .. } if value == "Some"),
        _ => false,
    }
}

/// Wraps `expr` in a synthetic 0-parameter, expression-bodied block literal
/// spanning `range`, for `and`/`or`'s lazily-evaluated right-hand side
/// (control-flow.md §2: `a and b` ≡ `a.and { b }`).
fn wrap_expr_as_lazy_block(expr: Expr, range: SourceRange) -> Expr {
    Expr::Block(Box::new(BlockExpr {
        params: Vec::new(),
        body: vec![Statement::Expr { expr, range }],
        expr_body: true,
        range,
    }))
}

/// Maps a non-lazy [`BinaryOp`] to the base selector name `emit_operator_send`
/// encodes it under. `And`/`Or` are handled separately (lazy — see
/// [`Compiler::compile_lazy_block_operand`]) and never reach this function.
///
/// # Panics
///
/// Panics if called with `BinaryOp::And`/`BinaryOp::Or` — a compiler-internal
/// invariant violation, not a user-reachable error.
fn binary_op_selector_name(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::IntegerDivide => "~/",
        BinaryOp::Power => "**",
        BinaryOp::Modulo => "%",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitXor => "^",
        BinaryOp::BitOr => "|",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And | BinaryOp::Or => unreachable!("and/or are lazy and compiled separately"),
    }
}
