use crate::bytecode::Bytecode;
use crate::compiler::inliner;
use crate::method::{encode_selector, make_signature, SignatureKind};
use crate::value::Value;
use phalcom_ast::ast::{BinaryOp, BlockExpr, Expr, MethodRefKind, Statement, SymbolLiteralKind, UnaryOp};
use phalcom_common::range::SourceRange;

use super::error::CompilerError;
use super::Compiler;

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
                    let selector = encode_selector(&mc.method, &labels, SignatureKind::Method(argc as u8));
                    let selector_sym = self.vm.interner.intern(&selector);
                    return self.compile_super_send(selector_sym, mc.args, argc as u8, mc.range);
                }

                // U5 Layer 1 (control-flow.md §3, ADR-0018): a sacred
                // selector sent with literal-block arguments compiles to a
                // guarded inline fast path instead of a plain send. Every
                // other call — including a sacred selector with a
                // *non*-literal block argument — falls through unchanged.
                let range = method_call.range;
                match inliner::recognize(*method_call) {
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
                        // A literal `ClassName.method(...)` receiver may name
                        // a `construct` (ADR-0011): redirect the call-site
                        // selector to the `Initializer` selector it was
                        // actually installed under, so `Counter.new()` reaches
                        // the constructor instead of the inherited
                        // `Object::new` bare-allocation primitive.
                        let receiver_class_sym = match &method_call.object {
                            Expr::Var { value, .. } => Some(self.vm.interner.intern(value)),
                            _ => None,
                        };

                        let arity = method_call.args.len();
                        let labels: Vec<Option<String>> = method_call.args.iter().map(|a| a.label.clone()).collect();
                        let selector = encode_selector(&method_call.method, &labels, SignatureKind::Method(arity as u8));
                        let selector_sym = self.vm.interner.intern(&selector);
                        let alias = receiver_class_sym.and_then(|class_sym| self.lookup_constructor_alias(class_sym, selector_sym));

                        // U7-plan §6 negative: a class with a `new`
                        // constructor has no user-visible bare allocator —
                        // a `new(...)` call whose arity/labels match no
                        // declared `construct` must not silently fall
                        // through to the inherited `Object::new` primitive.
                        if alias.is_none()
                            && method_call.method == "new"
                            && let Some(class_sym) = receiver_class_sym
                            && self.inherits_new_construct(class_sym)
                        {
                            return Err(CompilerError::Message(format!(
                                "No constructor `{}.new(...)` matches this call: arity/labels don't match any declared `construct`",
                                self.vm.resolve_symbol(class_sym)
                            )));
                        }

                        self.compile_expr(method_call.object)?;
                        for arg in &method_call.args {
                            self.compile_expr(arg.expr.clone())?;
                        }
                        let selector_sym = alias.unwrap_or(selector_sym);
                        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                        self.emit(Bytecode::Invoke(method_call.args.len() as u8, selector_idx), method_call.range);
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
                self.compile_expr(method_ref.receiver)?;
                let sym = match method_ref.kind {
                    MethodRefKind::Open { name } => self.vm.interner.intern(&name),
                    MethodRefKind::Pinned { name, labels } => {
                        let arity = labels.len() as u8;
                        let selector = encode_selector(&name, &labels, SignatureKind::Method(arity));
                        self.vm.interner.intern(&selector)
                    }
                };
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
                self.compile_expr(ix.object.clone())?;
                self.compile_expr(ix.index.clone())?;
                let selector = encode_selector("at", &[None], SignatureKind::Method(1));
                let sym = self.vm.interner.intern(&selector);
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Invoke(1, idx), ix.range);
            }
            Expr::SetIndex(six) => {
                self.compile_expr(six.object.clone())?;
                self.compile_expr(six.index.clone())?;
                self.compile_expr(six.value.clone())?;
                let labels: Vec<Option<String>> = vec![None, Some("put".to_string())];
                let selector = encode_selector("at", &labels, SignatureKind::Method(2));
                let sym = self.vm.interner.intern(&selector);
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Invoke(2, idx), six.range);
            }
            Expr::Number { value, range } => {
                let idx = self.add_constant(Value::Number(value));
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
                        let arity = labels.len() as u8;
                        encode_selector(&name, &labels, SignatureKind::Method(arity))
                    }
                };
                let sym = self.vm.interner.intern(&canonical);
                let idx = self.add_constant(Value::Symbol(sym));
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::Var { value, range } => {
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
                let class_sym = self.current_class.ok_or_else(|| {
                    CompilerError::Message(format!("Fields can only be accessed within a class: {}", value))
                })?;
                let layout = self.vm.field_layouts.get(&class_sym).cloned().ok_or_else(|| {
                    CompilerError::Message(format!("No layout registered for class: {}", self.vm.resolve_symbol(class_sym)))
                })?;

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
                        // Enforce `let` immutability (ADR-0014) before
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
                            // NOTE: reassignment of a *captured* `let` (an outer
                            // binding reached through an upvalue) is not yet
                            // rejected here — U6's stated scope is the
                            // current-function local and the module global.
                            // Tracked in DEFERRED.md.
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetUpvalue(upvalue as u16), range);
                        } else {
                            if self.immutable_globals.contains(&name_sym) {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            let name_idx = self.add_constant(Value::Symbol(name_sym));
                            self.emit(Bytecode::SetGlobal(name_idx), range);
                        }
                    }
                    Expr::Field { value, range } => {
                        let name_sym = self.vm.interner.intern(&value);
                        let class_sym = self.current_class.ok_or_else(|| {
                            CompilerError::Message(format!("Fields can only be accessed within a class: {}", value))
                        })?;
                        let layout = self.vm.field_layouts.get(&class_sym).cloned().ok_or_else(|| {
                            CompilerError::Message(format!("No layout registered for class: {}", self.vm.resolve_symbol(class_sym)))
                        })?;

                        if let Some(&slot) = layout.static_field_slots.get(&name_sym) {
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
                        return self.compile_sacred_call(inliner::SacredCall::And { receiver: binary_expr.left, rhs_block }, range);
                    }
                    BinaryOp::Or => {
                        if is_option_literal(&binary_expr.left) {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        let rhs_block = wrap_expr_as_lazy_block(binary_expr.right, range);
                        return self.compile_sacred_call(inliner::SacredCall::Or { receiver: binary_expr.left, rhs_block }, range);
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
                let closure = self.compile_block(block_expr.body, name_sym, block_expr.params, false, false)?;
                let idx = self.add_constant(Value::Obj(closure));
                self.emit(Bytecode::Closure(idx), block_expr.range);
            }
            // Expr::Call(call_expr) => {
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
        Expr::MethodCall(call) => {
            call.method == "new" && matches!(&call.object, Expr::Var { value, .. } if value == "Some")
        }
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
        BinaryOp::Modulo => "%",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And | BinaryOp::Or => unreachable!("and/or are lazy and compiled separately"),
    }
}
