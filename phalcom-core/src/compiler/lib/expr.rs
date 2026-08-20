use crate::bytecode::{Bytecode, FamilySpecKind, PackAccess, PackSendKind};
use crate::compiler::inliner;
use crate::method::{SignatureKind, encode_selector, make_signature};
use crate::value::Value;
use phalcom_ast::ast::{
    BinaryOp, BlockExpr, ClosureParameters, Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodCallExpr, NormalizedSelectorSpec, PackItem,
    PackLabel, ProductLabel, RecordLiteralEntry, SelectorSpecSyntax, SetLiteralEntry, Statement, SymbolLiteralKind, TupleLiteralEntry, UnaryOp,
};
use phalcom_common::range::SourceRange;

use super::error::CompilerError;
use super::scope::BareNameResolution;
use super::{Compiler, UnitKind, checked_product_count, checked_send_arity};

#[derive(Clone, Copy)]
enum PositionalExpansionTarget {
    ArgumentPack { builder_slot: u16 },
    ListLiteral { list_slot: u16 },
}

impl<'vm> Compiler<'vm> {
    /// Returns the exact selector for an immediately-called MethodRef when
    /// the call shape is statically identical to that selector. The caller
    /// can then emit an ordinary dynamic send while still evaluating the
    /// MethodRef receiver exactly once.
    fn immediate_exact_method_ref_selector(
        &mut self,
        method_call: &phalcom_ast::ast::MethodCallExpr,
    ) -> Result<Option<crate::interner::Symbol>, CompilerError> {
        if method_call.method != "call" || Self::needs_dynamic_pack(&method_call.args) {
            return Ok(None);
        }
        let Expr::MethodRef(method_ref) = &method_call.object else {
            return Ok(None);
        };
        let NormalizedSelectorSpec::Exact(selector) = method_ref
            .spec
            .normalize()
            .map_err(|error| CompilerError::Message(format!("invalid selector specification: {error}")))?
        else {
            return Ok(None);
        };
        if !matches!(selector.kind, phalcom_common::selector::SelectorKind::Method) {
            return Ok(None);
        }
        let arity = checked_send_arity("family call", method_call.args.len(), method_call.range)?;
        let labels = self.pack_labels(&method_call.args)?;
        let phalcom_common::selector::SelectorBase::Named(base) = &selector.base else {
            return Ok(None);
        };
        let call_selector = encode_selector(base, &labels, SignatureKind::Method(arity));
        if call_selector != selector.encode() {
            return Ok(None);
        }
        Ok(Some(self.vm.interner.intern(&call_selector)))
    }

    pub(super) fn needs_dynamic_pack(items: &[PackItem]) -> bool {
        items.iter().any(|item| {
            matches!(
                item,
                PackItem::Expand { .. }
                    | PackItem::Labeled {
                        label: PackLabel::Computed { .. },
                        ..
                    }
            )
        })
    }

    fn reserve_pack_scratch(&mut self, base: &str, range: SourceRange) -> Result<u16, CompilerError> {
        let name = self.fresh_scratch_symbol(base);
        self.add_local(name, true)?;
        let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
        self.emit(Bytecode::ReserveScratchLocal(slot), range);
        Ok(slot)
    }

    fn release_pack_scratch_from(&mut self, first_slot: u16, count: usize, range: SourceRange) {
        let function = self.functions.last().unwrap();
        debug_assert_eq!(function.locals.len(), function.num_locals);
        debug_assert_eq!(first_slot as usize + count, function.num_locals);
        for slot in (first_slot as usize..first_slot as usize + count).rev() {
            self.emit(Bytecode::ReleaseScratchLocal(slot as u16), range);
        }
        let function = self.functions.last_mut().unwrap();
        function.locals.truncate(function.locals.len() - count);
        function.num_locals -= count;
        debug_assert_eq!(function.locals.len(), function.num_locals);
    }

    fn emit_positional_target_before_value(&mut self, target: PositionalExpansionTarget, range: SourceRange) {
        if let PositionalExpansionTarget::ListLiteral { list_slot } = target {
            self.emit(Bytecode::GetLocal(list_slot), range);
        }
    }

    fn emit_positional_target_append(&mut self, target: PositionalExpansionTarget, range: SourceRange) {
        match target {
            PositionalExpansionTarget::ArgumentPack { builder_slot } => {
                self.emit(Bytecode::GetLocal(builder_slot), range);
                self.emit(Bytecode::PackPushPositional, range);
            }
            PositionalExpansionTarget::ListLiteral { .. } => {
                self.emit(Bytecode::ListLiteralAppend, range);
                self.emit(Bytecode::Pop, range);
            }
        }
    }

    fn emit_positional_target_for_probe(&mut self, target: PositionalExpansionTarget, range: SourceRange) {
        match target {
            PositionalExpansionTarget::ArgumentPack { builder_slot } => self.emit(Bytecode::GetLocal(builder_slot), range),
            PositionalExpansionTarget::ListLiteral { list_slot } => self.emit(Bytecode::GetLocal(list_slot), range),
        }
    }

    /// Adds one positional spread through the shared Tuple/Unit probe, then
    /// the ordinary cursor protocol. Hidden locals root source and cursor
    /// across arbitrary sends (including fiber suspension); only append
    /// destination differs between packs and List literals.
    fn compile_positional_expansion(&mut self, target: PositionalExpansionTarget, expr: Expr, range: SourceRange) -> Result<(), CompilerError> {
        self.check_bounded_expansion(&expr, range)?;
        let source_slot = self.reserve_pack_scratch("$pack_source", range)?;
        self.compile_expr(expr)?;
        self.emit(Bytecode::SetLocal(source_slot), range);
        self.emit(Bytecode::Pop, range);
        let cursor_slot = self.reserve_pack_scratch("$pack_cursor", range)?;

        self.emit(Bytecode::GetLocal(source_slot), range);
        self.emit_positional_target_for_probe(target, range);
        self.emit(
            match target {
                PositionalExpansionTarget::ArgumentPack { .. } => Bytecode::PackTryExpandTuplePositionals,
                PositionalExpansionTarget::ListLiteral { .. } => Bytecode::ListTryExpandTuplePositionals,
            },
            range,
        );
        let generic = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        let done = self.emit_forward_jump(Bytecode::Jump, range);

        self.patch_forward_jump(generic);
        self.emit(Bytecode::GetLocal(source_slot), range);
        self.emit(Bytecode::Nil, range);
        self.emit_operator_send("iterate", 1, range);
        self.emit(Bytecode::SetLocal(cursor_slot), range);
        self.emit(Bytecode::Pop, range);

        let loop_start = self.chunk_len();
        self.emit(Bytecode::GetLocal(cursor_slot), range);
        let exit = self.emit_forward_jump(Bytecode::JumpIfNone, range);
        self.emit_positional_target_before_value(target, range);
        self.emit(Bytecode::GetLocal(source_slot), range);
        self.emit(Bytecode::GetLocal(cursor_slot), range);
        self.emit_operator_send("iteratorValue", 1, range);
        self.emit_positional_target_append(target, range);
        self.emit(Bytecode::GetLocal(source_slot), range);
        self.emit(Bytecode::GetLocal(cursor_slot), range);
        self.emit_operator_send("iterate", 1, range);
        self.emit(Bytecode::SetLocal(cursor_slot), range);
        self.emit(Bytecode::Pop, range);
        self.emit_backward_loop(loop_start, range);
        self.patch_forward_jump(exit);
        self.patch_forward_jump(done);

        self.release_pack_scratch_from(source_slot, 2, range);
        Ok(())
    }

    fn compile_dynamic_pack_items(&mut self, builder_slot: u16, items: Vec<PackItem>) -> Result<(), CompilerError> {
        for item in items {
            let range = match &item {
                PackItem::Positional { range, .. } | PackItem::Labeled { range, .. } | PackItem::Expand { range, .. } => *range,
            };
            match item {
                PackItem::Positional { expr, .. } => {
                    self.compile_expr(expr)?;
                    self.emit(Bytecode::GetLocal(builder_slot), range);
                    self.emit(Bytecode::PackPushPositional, range);
                }
                PackItem::Labeled {
                    label: PackLabel::Static { text, .. },
                    value,
                    ..
                } => {
                    let label = self.vm.interner.intern(&text);
                    let index = self.add_constant(Value::symbol(label));
                    self.emit(Bytecode::GetLocal(builder_slot), range);
                    self.emit(Bytecode::PackReserveStaticLabel(index), range);
                    self.compile_expr(value)?;
                    self.emit(Bytecode::GetLocal(builder_slot), range);
                    self.emit(Bytecode::PackFillReservedLabel, range);
                }
                PackItem::Labeled {
                    label: PackLabel::Computed { expr, .. },
                    value,
                    ..
                } => {
                    self.compile_expr(*expr)?;
                    self.emit(Bytecode::GetLocal(builder_slot), range);
                    self.emit(Bytecode::PackReserveComputedLabel, range);
                    self.compile_expr(value)?;
                    self.emit(Bytecode::GetLocal(builder_slot), range);
                    self.emit(Bytecode::PackFillReservedLabel, range);
                }
                PackItem::Expand { mode, expr, .. } => {
                    if matches!(mode, phalcom_ast::ast::ExpansionMode::Positional) {
                        self.compile_positional_expansion(PositionalExpansionTarget::ArgumentPack { builder_slot }, expr, range)?;
                        continue;
                    }
                    self.compile_expr(expr)?;
                    self.emit(Bytecode::GetLocal(builder_slot), range);
                    self.emit(
                        if matches!(mode, phalcom_ast::ast::ExpansionMode::Labeled) {
                            Bytecode::PackExpandLabels
                        } else {
                            Bytecode::PackExpandComplete
                        },
                        range,
                    );
                }
            }
        }
        Ok(())
    }

    fn compile_dynamic_method_send(
        &mut self,
        receiver: Expr,
        base: String,
        items: Vec<PackItem>,
        kind: PackSendKind,
        access: PackAccess,
        range: SourceRange,
    ) -> Result<(), CompilerError> {
        let receiver_slot = self.reserve_pack_scratch("$pack_receiver", range)?;
        self.compile_expr(receiver)?;
        self.emit(Bytecode::SetLocal(receiver_slot), range);
        self.emit(Bytecode::Pop, range);
        let builder_slot = self.reserve_pack_scratch("$pack_builder", range)?;
        self.emit(Bytecode::NewArgumentPack, range);
        self.emit(Bytecode::SetLocal(builder_slot), range);
        self.emit(Bytecode::Pop, range);
        self.compile_dynamic_pack_items(builder_slot, items)?;
        let base = self.vm.interner.intern(&base);
        let base_idx = self.add_constant(Value::symbol(base));
        self.emit(Bytecode::GetLocal(receiver_slot), range);
        self.emit(Bytecode::GetLocal(builder_slot), range);
        self.emit(
            Bytecode::InvokePack {
                base_name: base_idx,
                kind,
                access,
            },
            range,
        );
        self.release_pack_scratch_from(receiver_slot, 2, range);
        Ok(())
    }
    /// Builds static selector slots and rejects duplicate labels before any
    /// value bytecode is emitted for a send.
    pub(super) fn pack_labels(&self, items: &[PackItem]) -> Result<Vec<Option<String>>, CompilerError> {
        let mut labels = Vec::with_capacity(items.len());
        let mut seen = std::collections::HashMap::<String, SourceRange>::new();
        for item in items {
            match item {
                PackItem::Positional { .. } => labels.push(None),
                PackItem::Labeled {
                    label: PackLabel::Static { text, range },
                    ..
                } => {
                    if let Some(first_span) = seen.get(text) {
                        return Err(CompilerError::DuplicateArgumentLabel {
                            label: text.clone(),
                            span: *range,
                            first_span: *first_span,
                        });
                    }
                    seen.insert(text.clone(), *range);
                    labels.push(Some(text.clone()));
                }
                PackItem::Labeled {
                    label: PackLabel::Computed { range, .. },
                    ..
                } => return Err(CompilerError::ComputedLabelNotYetSupported(*range)),
                PackItem::Expand { range, .. } => return Err(CompilerError::PackExpansionNotYetSupported(*range)),
            }
        }
        Ok(labels)
    }

    /// Lowers a statically shaped F.1 pack contribution.
    pub(super) fn compile_pack_item(&mut self, item: PackItem) -> Result<(), CompilerError> {
        match item {
            PackItem::Positional { expr, .. } => self.compile_expr(expr),
            PackItem::Labeled {
                label: PackLabel::Static { .. },
                value,
                ..
            } => self.compile_expr(value),
            PackItem::Labeled {
                label: PackLabel::Computed { range, .. },
                ..
            } => Err(CompilerError::ComputedLabelNotYetSupported(range)),
            PackItem::Expand { range, .. } => Err(CompilerError::PackExpansionNotYetSupported(range)),
        }
    }
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
    /// its immediate `Some` wrap, since the wrap is unobservable when the
    /// value is popped unread (U-CORE-2; see
    /// [`Self::compile_sacred_call_want`]). Every other expression shape
    /// ignores `want_value` — it still pushes its one value as normal.
    pub(crate) fn compile_expr_want(&mut self, expr: Expr, want_value: bool) -> Result<(), CompilerError> {
        match expr {
            Expr::UnqualifiedCall(call) => {
                let call = *call;
                let name_sym = self.vm.interner.intern(&call.name);
                let resolution = self.resolve_bare_name(name_sym);
                if Self::needs_dynamic_pack(&call.args) {
                    if matches!(resolution, BareNameResolution::ImplicitSelf) {
                        let receiver = Expr::SelfVar { range: call.range };
                        return self.compile_dynamic_method_send(receiver, call.name, call.args, PackSendKind::Method, PackAccess::Ordinary, call.range);
                    } else {
                        // A callable receiver can appear above partial-expression
                        // values on the operand stack. Root it before assembling
                        // the pack; treating the already-pushed value as a local
                        // would instead capture that earlier operand.
                        let receiver_slot = self.reserve_pack_scratch("$pack_receiver", call.range)?;
                        match resolution {
                            BareNameResolution::Local(slot) => self.emit(Bytecode::GetLocal(slot as u16), call.range),
                            BareNameResolution::Upvalue(upvalue) => self.emit(Bytecode::GetUpvalue(upvalue as u16), call.range),
                            BareNameResolution::Linked(binding) => self.emit(Bytecode::GetLinked(binding.0 as u16), call.range),
                            BareNameResolution::Global | BareNameResolution::Unresolved => {
                                let name_idx = self.add_constant(Value::symbol(name_sym));
                                self.emit(Bytecode::GetGlobal(name_idx), call.range);
                            }
                            BareNameResolution::ImplicitSelf => unreachable!("handled above"),
                        }
                        self.emit(Bytecode::SetLocal(receiver_slot), call.range);
                        self.emit(Bytecode::Pop, call.range);

                        let builder_slot = self.reserve_pack_scratch("$pack_builder", call.range)?;
                        self.emit(Bytecode::NewArgumentPack, call.range);
                        self.emit(Bytecode::SetLocal(builder_slot), call.range);
                        self.emit(Bytecode::Pop, call.range);
                        self.compile_dynamic_pack_items(builder_slot, call.args)?;
                        let base = self.vm.interner.intern("call");
                        let base_idx = self.add_constant(Value::symbol(base));
                        self.emit(Bytecode::GetLocal(receiver_slot), call.range);
                        self.emit(Bytecode::GetLocal(builder_slot), call.range);
                        self.emit(
                            Bytecode::InvokePack {
                                base_name: base_idx,
                                kind: PackSendKind::Method,
                                access: PackAccess::Ordinary,
                            },
                            call.range,
                        );
                        self.release_pack_scratch_from(receiver_slot, 2, call.range);
                        return Ok(());
                    }
                }
                match resolution {
                    BareNameResolution::Local(slot) => self.emit(Bytecode::GetLocal(slot as u16), call.range),
                    BareNameResolution::Upvalue(upvalue) => self.emit(Bytecode::GetUpvalue(upvalue as u16), call.range),
                    BareNameResolution::Linked(binding) => self.emit(Bytecode::GetLinked(binding.0 as u16), call.range),
                    BareNameResolution::Global | BareNameResolution::Unresolved => {
                        let name_idx = self.add_constant(Value::symbol(name_sym));
                        self.emit(Bytecode::GetGlobal(name_idx), call.range);
                    }
                    BareNameResolution::ImplicitSelf => {
                        let arity = checked_send_arity("implicit message send", call.args.len(), call.range)?;
                        let labels = self.pack_labels(&call.args)?;
                        let selector = encode_selector(&call.name, &labels, SignatureKind::Method(arity));
                        let selector_sym = self.vm.interner.intern(&selector);
                        self.emit_self(call.range);
                        for arg in call.args {
                            self.compile_pack_item(arg)?;
                        }
                        let selector_idx = self.add_constant(Value::symbol(selector_sym));
                        self.emit(Bytecode::Invoke(arity, selector_idx), call.range);
                        return Ok(());
                    }
                }
                let arity = checked_send_arity("callable call", call.args.len(), call.range)?;
                let labels = self.pack_labels(&call.args)?;
                for arg in call.args {
                    self.compile_pack_item(arg)?;
                }
                let selector = encode_selector("call", &labels, SignatureKind::Method(arity));
                let selector_sym = self.vm.interner.intern(&selector);
                let selector_idx = self.add_constant(Value::symbol(selector_sym));
                self.emit(Bytecode::Invoke(arity, selector_idx), call.range);
            }
            Expr::MethodCall(method_call) => {
                self.check_bounded_method_call(&method_call)?;
                if let Some(selector_sym) = self.immediate_exact_method_ref_selector(&method_call)? {
                    let method_call = *method_call;
                    let Expr::MethodRef(method_ref) = method_call.object else {
                        unreachable!("immediate exact MethodRef selector was validated above");
                    };
                    let arity = method_call.args.len() as u8;
                    self.compile_expr(method_ref.receiver)?;
                    for arg in method_call.args {
                        self.compile_pack_item(arg)?;
                    }
                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Invoke(arity, selector_idx), method_call.range);
                    return Ok(());
                }
                let internal_call = method_call.method.starts_with("_$");
                let is_invariant_guard = method_call.method == "_$invariantEnter" || method_call.method == "_$invariantExit";
                if internal_call && !is_invariant_guard && !self.compiling_privileged_core() && !self.compiler_internal {
                    return Err(CompilerError::InternalNamespaceReserved(method_call.method.clone(), method_call.range));
                }
                if Self::needs_dynamic_pack(&method_call.args) && !matches!(&method_call.object, Expr::SuperVar { .. }) {
                    let method_call = *method_call;
                    return self.compile_dynamic_method_send(
                        method_call.object,
                        method_call.method,
                        method_call.args,
                        PackSendKind::Method,
                        if internal_call { PackAccess::CompilerInternal } else { PackAccess::Ordinary },
                        method_call.range,
                    );
                }
                // A `super.sel(args)` send lowers to `SuperSend`, never an
                // ordinary `Invoke` — and must be intercepted *before* the
                // sacred inliner, so a `super.ifTrue { … }` is a real dispatch
                // starting above the defining class, not an inlined fast path
                // keyed on the receiver's static type (U-INH §3.4).
                if matches!(&method_call.object, Expr::SuperVar { .. }) {
                    let mc = *method_call;
                    if Self::needs_dynamic_pack(&mc.args) {
                        let class_key = self.current_class.ok_or(CompilerError::SuperOutsideMethod)?;
                        let defining = if self.is_static_context {
                            let name = self.vm.resolve_symbol(class_key.name).to_string();
                            self.vm.interner.intern(&format!("{name}.class"))
                        } else {
                            class_key.name
                        };
                        let base = match self.functions.last().unwrap().constructor_name.as_deref() {
                            Some(constructor) if mc.method == constructor => format!("init {constructor}"),
                            _ => mc.method.clone(),
                        };
                        let receiver_slot = self.reserve_pack_scratch("$pack_receiver", mc.range)?;
                        self.emit_self(mc.range);
                        self.emit(Bytecode::SetLocal(receiver_slot), mc.range);
                        self.emit(Bytecode::Pop, mc.range);
                        let builder_slot = self.reserve_pack_scratch("$pack_builder", mc.range)?;
                        self.emit(Bytecode::NewArgumentPack, mc.range);
                        self.emit(Bytecode::SetLocal(builder_slot), mc.range);
                        self.emit(Bytecode::Pop, mc.range);
                        self.compile_dynamic_pack_items(builder_slot, mc.args)?;
                        let base_sym = self.vm.interner.intern(&base);
                        let base_idx = self.add_constant(Value::symbol(base_sym));
                        let defining_idx = self.add_constant(Value::symbol(defining));
                        self.emit(Bytecode::GetLocal(receiver_slot), mc.range);
                        self.emit(Bytecode::GetLocal(builder_slot), mc.range);
                        self.emit(
                            Bytecode::SuperSendPack {
                                base_name: base_idx,
                                defining_class: defining_idx,
                            },
                            mc.range,
                        );
                        self.release_pack_scratch_from(receiver_slot, 2, mc.range);
                        return Ok(());
                    }
                    let argc = mc.args.len();
                    let labels = self.pack_labels(&mc.args)?;
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
                    inliner::Recognition::Ordinary(method_call)
                } else {
                    inliner::recognize(method_call)
                };
                match recognized {
                    inliner::Recognition::Sacred(sacred) => {
                        // BD-U6-1 (ADR-0007, values-and-absence §3.5): a
                        // conditional's condition that is a syntactically
                        // detectable `Option` literal (`if (None) { … }`,
                        // `if (Some(x)) { … }`, `None and …`) is a compile
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
                    inliner::Recognition::Ordinary(method_call) => {
                        let receiver_class_sym = match &method_call.object {
                            Expr::Var { value, .. } => Some(self.vm.interner.intern(value)),
                            _ => None,
                        };

                        let arity = checked_send_arity("message send", method_call.args.len(), method_call.range)?;
                        let labels = self.pack_labels(&method_call.args)?;
                        let selector = encode_selector(&method_call.method, &labels, SignatureKind::Method(arity));
                        let selector_sym = self.vm.interner.intern(&selector);

                        self.compile_expr(method_call.object)?;
                        for arg in method_call.args {
                            self.compile_pack_item(arg)?;
                        }
                        let selector_idx = self.add_constant(Value::symbol(selector_sym));
                        let opcode = if internal_call {
                            Bytecode::InvokeCompilerInternal(arity, selector_idx)
                        } else {
                            Bytecode::Invoke(arity, selector_idx)
                        };
                        self.emit(opcode, method_call.range);
                    }
                }
            }
            Expr::MethodRef(method_ref) => {
                let method_ref = *method_ref;
                let (spec_idx, kind) = self.compile_selector_spec_constant(&method_ref.spec)?;
                self.compile_expr(method_ref.receiver)?;
                self.emit(Bytecode::MakeFamily { spec: spec_idx, kind }, method_ref.range);
            }
            Expr::GetProperty(get_prop) => {
                self.check_bounded_property(&get_prop.property, &get_prop.object, get_prop.range)?;
                // `super.prop` is a zero-arg super send (U-INH §3.4); the
                // getter/no-arg selector is the bare property name, matching the
                // ordinary getter dispatch below.
                if matches!(&get_prop.object, Expr::SuperVar { .. }) {
                    let selector_sym = self.vm.interner.intern(&get_prop.property);
                    return self.compile_super_send(selector_sym, Vec::new(), 0, get_prop.range);
                }
                self.compile_expr(get_prop.object)?;
                let selector_sym = self.vm.interner.intern(&get_prop.property);
                let selector_idx = self.add_constant(Value::symbol(selector_sym));
                self.emit(Bytecode::Invoke(0, selector_idx), get_prop.range);
            }
            Expr::SetProperty(set_prop) => {
                self.compile_expr(set_prop.object)?;
                self.compile_expr(set_prop.value)?;
                let selector = make_signature(&set_prop.property, SignatureKind::Setter);
                let selector_sym = self.vm.interner.intern(&selector);
                let selector_idx = self.add_constant(Value::symbol(selector_sym));
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
                if Self::needs_dynamic_pack(&ix.args) {
                    return self.compile_dynamic_method_send(ix.object, String::new(), ix.args, PackSendKind::SubscriptGet, PackAccess::Ordinary, ix.range);
                }
                let argc = checked_send_arity("subscript read", ix.args.len(), ix.range)?;
                let labels = self.pack_labels(&ix.args)?;
                self.compile_expr(ix.object)?;
                for arg in ix.args {
                    self.compile_pack_item(arg)?;
                }
                let selector = encode_selector("", &labels, SignatureKind::SubscriptGet(argc));
                let sym = self.vm.interner.intern(&selector);
                let idx = self.add_constant(Value::symbol(sym));
                self.emit(Bytecode::Invoke(argc, idx), ix.range);
            }
            Expr::SetIndex(six) => {
                let six = *six;
                if Self::needs_dynamic_pack(&six.args) {
                    let receiver_slot = self.reserve_pack_scratch("$pack_receiver", six.range)?;
                    self.compile_expr(six.object)?;
                    self.emit(Bytecode::SetLocal(receiver_slot), six.range);
                    self.emit(Bytecode::Pop, six.range);
                    let builder_slot = self.reserve_pack_scratch("$pack_builder", six.range)?;
                    self.emit(Bytecode::NewArgumentPack, six.range);
                    self.emit(Bytecode::SetLocal(builder_slot), six.range);
                    self.emit(Bytecode::Pop, six.range);
                    self.compile_dynamic_pack_items(builder_slot, six.args)?;
                    let put = self.vm.interner.intern("put");
                    let put_idx = self.add_constant(Value::symbol(put));
                    self.emit(Bytecode::GetLocal(builder_slot), six.range);
                    self.emit(Bytecode::PackReserveStaticLabel(put_idx), six.range);
                    let rhs_slot = self.reserve_pack_scratch("$setindex_rhs", six.range)?;
                    self.compile_expr(six.value)?;
                    self.emit(Bytecode::SetLocal(rhs_slot), six.range);
                    self.emit(Bytecode::GetLocal(builder_slot), six.range);
                    self.emit(Bytecode::PackFillReservedLabel, six.range);
                    let base = self.vm.interner.intern("");
                    let base_idx = self.add_constant(Value::symbol(base));
                    self.emit(Bytecode::GetLocal(receiver_slot), six.range);
                    self.emit(Bytecode::GetLocal(builder_slot), six.range);
                    self.emit(
                        Bytecode::InvokePack {
                            base_name: base_idx,
                            kind: PackSendKind::SubscriptSet,
                            access: PackAccess::Ordinary,
                        },
                        six.range,
                    );
                    self.emit(Bytecode::Pop, six.range);
                    self.emit(Bytecode::GetLocal(rhs_slot), six.range);
                    self.release_pack_scratch_from(receiver_slot, 3, six.range);
                    return Ok(());
                }
                // Compiler-owned `put` occupies the final setter label.
                if let Some(PackItem::Labeled {
                    label: PackLabel::Static { text, range },
                    ..
                }) = six
                    .args
                    .iter()
                    .find(|item| matches!(item, PackItem::Labeled { label: PackLabel::Static { text, .. }, .. } if text == "put"))
                {
                    return Err(CompilerError::DuplicateArgumentLabel {
                        label: text.clone(),
                        span: *range,
                        first_span: six.range,
                    });
                }

                let labels = self.pack_labels(&six.args)?;
                let index_argc = checked_send_arity("subscript write index", six.args.len(), six.range)?;
                let invoke_argc = checked_send_arity("subscript write", six.args.len() + 1, six.range)?;

                // 1. Reserve hidden slot (push placeholder)
                let scratch_sym = self.fresh_scratch_symbol("$setindex_rhs");
                self.add_local(scratch_sym, true)?;
                let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
                self.emit(Bytecode::Nil, six.range);

                // 2. Compile receiver
                self.compile_expr(six.object)?;

                // 3. Compile subscript arguments in lexical order
                for arg in six.args {
                    self.compile_pack_item(arg)?;
                }

                // 4. Compile RHS
                self.compile_expr(six.value)?;

                // 5. Copy RHS into hidden slot using SetLocal
                self.emit(Bytecode::SetLocal(slot), six.range);

                // 6. Invoke setter
                let selector = encode_selector("", &labels, SignatureKind::SubscriptSet(index_argc));
                let sym = self.vm.interner.intern(&selector);
                let idx = self.add_constant(Value::symbol(sym));
                self.emit(Bytecode::Invoke(invoke_argc, idx), six.range);

                // 7. Pop setter result
                self.emit(Bytecode::Pop, six.range);

                // 8. Remove compiler-local metadata without emitting Pop
                let func = self.functions.last_mut().unwrap();
                func.locals.pop();
                func.num_locals -= 1;
            }
            Expr::Range(range) => {
                let range = *range;
                let has_lower = range.lower.is_some();
                let has_upper = range.upper.is_some();
                if let Some(lower) = range.lower {
                    self.compile_expr(lower)?;
                }
                if let Some(upper) = range.upper {
                    self.compile_expr(upper)?;
                }
                self.emit(
                    Bytecode::BuildRange {
                        has_lower,
                        has_upper,
                        upper_inclusive: range.upper_inclusive,
                    },
                    range.range,
                );
            }
            Expr::Int { digits, radix, range } => {
                let val = if radix == 10 {
                    if let Ok(i) = digits.parse::<i64>() {
                        Value::int(i)
                    } else if let Some(big) = num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10) {
                        let obj = self.vm.heap.alloc(crate::heap::Object::LargeInt(big));
                        Value::obj(obj)
                    } else {
                        return Err(CompilerError::Message(format!("Invalid integer literal: {digits}")));
                    }
                } else {
                    if let Ok(i) = i64::from_str_radix(&digits, radix) {
                        Value::int(i)
                    } else if let Some(big) = num_bigint::BigInt::parse_bytes(digits.as_bytes(), radix) {
                        let obj = self.vm.heap.alloc(crate::heap::Object::LargeInt(big));
                        Value::obj(obj)
                    } else {
                        return Err(CompilerError::Message(format!("Invalid radix integer literal: {digits}")));
                    }
                };
                let idx = self.add_constant(val);
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::Float { value, range } => {
                let idx = self.add_constant(Value::float(value));
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
                    SymbolLiteralKind::Pattern(pattern) => {
                        let normalized = SelectorSpecSyntax::Pattern(pattern)
                            .normalize()
                            .map_err(|error| CompilerError::Message(format!("invalid selector pattern: {error}")))?;
                        let NormalizedSelectorSpec::Pattern(pattern) = normalized else {
                            unreachable!("pattern syntax normalized to exact selector")
                        };
                        let pattern_object = crate::heap::SelectorPatternObject::compile(pattern, &mut self.vm.interner);
                        let pattern = self.vm.heap.alloc(crate::heap::Object::SelectorPattern(Box::new(pattern_object)));
                        let idx = self.add_constant(Value::obj(pattern));
                        self.emit(Bytecode::Constant(idx), range);
                        return Ok(());
                    }
                };
                let sym = self.vm.interner.intern(&canonical);
                let idx = self.add_constant(Value::symbol(sym));
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::TupleLiteral(tuple_expr) => {
                let tuple_expr = *tuple_expr;
                if tuple_expr.entries.is_empty() {
                    let idx = self.add_constant(Value::unit());
                    self.emit(Bytecode::Constant(idx), tuple_expr.range);
                    return Ok(());
                }
                let dynamic = tuple_expr.entries.iter().any(|entry| {
                    matches!(
                        entry,
                        TupleLiteralEntry::Expand { .. }
                            | TupleLiteralEntry::Labeled {
                                label: ProductLabel::Computed { .. },
                                ..
                            }
                    )
                });
                if dynamic {
                    let builder_slot = self.reserve_pack_scratch("$tuple_pack_builder", tuple_expr.range)?;
                    self.emit(Bytecode::NewArgumentPack, tuple_expr.range);
                    self.emit(Bytecode::SetLocal(builder_slot), tuple_expr.range);
                    self.emit(Bytecode::Pop, tuple_expr.range);
                    for entry in tuple_expr.entries {
                        match entry {
                            TupleLiteralEntry::Positional { expr, range } => {
                                self.compile_expr(expr)?;
                                self.emit(Bytecode::GetLocal(builder_slot), range);
                                self.emit(Bytecode::PackPushPositional, range);
                            }
                            TupleLiteralEntry::Labeled { label, value, range } => match label {
                                ProductLabel::Static { symbol, .. } => {
                                    let sym = self.canonical_symbol(symbol, range)?;
                                    let idx = self.add_constant(Value::symbol(sym));
                                    self.emit(Bytecode::GetLocal(builder_slot), range);
                                    self.emit(Bytecode::PackReserveStaticLabel(idx), range);
                                    self.compile_expr(value)?;
                                    self.emit(Bytecode::GetLocal(builder_slot), range);
                                    self.emit(Bytecode::PackFillReservedLabel, range);
                                }
                                ProductLabel::Computed { expr, .. } => {
                                    self.compile_expr(*expr)?;
                                    self.emit(Bytecode::GetLocal(builder_slot), range);
                                    self.emit(Bytecode::PackReserveComputedLabel, range);
                                    self.compile_expr(value)?;
                                    self.emit(Bytecode::GetLocal(builder_slot), range);
                                    self.emit(Bytecode::PackFillReservedLabel, range);
                                }
                            },
                            TupleLiteralEntry::Expand { mode, expr, range } => match mode {
                                phalcom_ast::ast::ExpansionMode::Positional => {
                                    self.compile_positional_expansion(PositionalExpansionTarget::ArgumentPack { builder_slot }, expr, range)?;
                                }
                                phalcom_ast::ast::ExpansionMode::Labeled => {
                                    self.compile_expr(expr)?;
                                    self.emit(Bytecode::GetLocal(builder_slot), range);
                                    self.emit(Bytecode::PackExpandLabels, range);
                                }
                                phalcom_ast::ast::ExpansionMode::Complete => {
                                    self.compile_expr(expr)?;
                                    self.emit(Bytecode::GetLocal(builder_slot), range);
                                    self.emit(Bytecode::PackExpandComplete, range);
                                }
                            },
                        }
                    }
                    self.emit(Bytecode::GetLocal(builder_slot), tuple_expr.range);
                    self.emit(Bytecode::FinishTuplePack, tuple_expr.range);
                    self.release_pack_scratch_from(builder_slot, 1, tuple_expr.range);
                    return Ok(());
                }
                let positional = tuple_expr
                    .entries
                    .iter()
                    .filter(|entry| matches!(entry, TupleLiteralEntry::Positional { .. }))
                    .count();
                let labeled = tuple_expr.entries.len() - positional;
                let positional = checked_product_count("Tuple positional lane", positional, tuple_expr.range)?;
                let labeled = checked_product_count("Tuple labeled lane", labeled, tuple_expr.range)?;
                let mut seen = std::collections::HashSet::new();
                for entry in tuple_expr.entries {
                    match entry {
                        TupleLiteralEntry::Positional { expr, .. } => self.compile_expr(expr)?,
                        TupleLiteralEntry::Labeled { label, value, range } => {
                            self.compile_product_label(label, &mut seen, range)?;
                            self.compile_expr(value)?;
                        }
                        TupleLiteralEntry::Expand { range, .. } => return Err(CompilerError::PackExpansionNotYetSupported(range)),
                    }
                }
                self.emit(Bytecode::BuildTuple { positional, labeled }, tuple_expr.range);
            }
            Expr::RecordLiteral(record_expr) => {
                let record_expr = *record_expr;
                if record_expr.entries.is_empty() {
                    let idx = self.add_constant(Value::unit());
                    self.emit(Bytecode::Constant(idx), record_expr.range);
                    return Ok(());
                }

                let dynamic = record_expr.entries.iter().any(|entry| matches!(entry, RecordLiteralEntry::Expansion { .. }));
                if !dynamic {
                    let mut seen = std::collections::HashSet::new();
                    let fields = checked_product_count("Record", record_expr.entries.len(), record_expr.range)?;
                    for entry in record_expr.entries {
                        let RecordLiteralEntry::Field(field) = entry else {
                            unreachable!("static Record literal cannot contain expansion");
                        };
                        let phalcom_ast::ast::RecordLiteralField { label, value, range } = field;
                        self.compile_product_label(label, &mut seen, range)?;
                        self.compile_expr(value)?;
                    }
                    self.emit(Bytecode::BuildRecord { fields }, record_expr.range);
                    return Ok(());
                }

                let builder_slot = self.reserve_pack_scratch("$record_literal_builder", record_expr.range)?;
                self.emit(Bytecode::NewRecordLiteralBuilder, record_expr.range);
                self.emit(Bytecode::SetLocal(builder_slot), record_expr.range);
                self.emit(Bytecode::Pop, record_expr.range);
                let mut seen = std::collections::HashSet::new();
                for entry in record_expr.entries {
                    match entry {
                        RecordLiteralEntry::Field(field) => {
                            let phalcom_ast::ast::RecordLiteralField { label, value, range } = field;
                            // Keep builder below label/value so the append opcode
                            // can consume `builder | label | value` after the
                            // label's required-before-value evaluation.
                            self.emit(Bytecode::GetLocal(builder_slot), range);
                            self.compile_product_label(label, &mut seen, range)?;
                            self.compile_expr(value)?;
                            self.emit(Bytecode::RecordLiteralAppend, range);
                        }
                        RecordLiteralEntry::Expansion { expr, range } => {
                            self.emit(Bytecode::GetLocal(builder_slot), range);
                            self.compile_expr(expr)?;
                            self.emit(Bytecode::RecordLiteralExpandLabels, range);
                        }
                    }
                }
                self.emit(Bytecode::GetLocal(builder_slot), record_expr.range);
                self.emit(Bytecode::FinishRecordLiteral, record_expr.range);
                self.release_pack_scratch_from(builder_slot, 1, record_expr.range);
            }
            Expr::MapLiteral(map_expr) => {
                let map_expr = *map_expr;
                let mut static_symbols = std::collections::HashSet::new();
                self.emit(Bytecode::BeginMapLiteral, map_expr.range);
                for entry in map_expr.entries {
                    match entry {
                        MapLiteralEntry::Association { key, value, range } => {
                            match key {
                                MapLiteralKey::BareSymbol { name, range } => {
                                    if !static_symbols.insert(name.clone()) {
                                        return Err(CompilerError::Message(format!("duplicate Map literal key `#{name}`")));
                                    }
                                    let symbol = self.vm.interner.intern(&name);
                                    let idx = self.add_constant(Value::symbol(symbol));
                                    self.emit(Bytecode::Constant(idx), range);
                                }
                                MapLiteralKey::Computed { expr, .. } => {
                                    if let Expr::Symbol(symbol_expr) = &expr
                                        && let SymbolLiteralKind::Name(name) = &symbol_expr.kind
                                        && !static_symbols.insert(name.clone())
                                    {
                                        return Err(CompilerError::Message(format!("duplicate Map literal key `#{name}`")));
                                    }
                                    self.compile_expr(expr)?;
                                }
                            }
                            self.compile_expr(value)?;
                            self.emit(Bytecode::MapLiteralInsertUnique, range);
                        }
                        MapLiteralEntry::Expansion { expr, range } => {
                            self.compile_expr(expr)?;
                            self.emit(Bytecode::MapLiteralExpandLabels, range);
                        }
                    }
                }
                self.emit(Bytecode::FinishMapLiteral, map_expr.range);
            }
            Expr::SetLiteral(set_expr) => {
                let set_expr = *set_expr;
                self.emit(Bytecode::BeginSetLiteral, set_expr.range);
                for entry in set_expr.entries {
                    match entry {
                        SetLiteralEntry::Element { expr, range } => {
                            self.compile_expr(expr)?;
                            self.emit(Bytecode::SetLiteralAdd, range);
                        }
                        SetLiteralEntry::Expansion { .. } => {
                            return Err(CompilerError::Message("Set literal `*` expansion is pending Spec F".to_string()));
                        }
                    }
                }
                self.emit(Bytecode::FinishSetLiteral, set_expr.range);
            }
            Expr::ListLiteral(list_expr) => {
                let list_expr = *list_expr;
                let dynamic = list_expr.elements.iter().any(|element| matches!(element, ListLiteralElement::Expansion { .. }));
                if !dynamic {
                    let count = list_expr.elements.len();
                    if count > u16::MAX as usize {
                        return Err(CompilerError::Message("List literal elements exceed maximum count".to_string()));
                    }
                    for element in list_expr.elements {
                        let ListLiteralElement::Element { expr, .. } = element else {
                            unreachable!("static List literal cannot contain expansion");
                        };
                        self.compile_expr(expr)?;
                    }
                    self.emit(Bytecode::BuildList(count as u16), list_expr.range);
                    return Ok(());
                }

                let list_slot = self.reserve_pack_scratch("$list_literal_builder", list_expr.range)?;
                self.emit(Bytecode::BeginListLiteral, list_expr.range);
                self.emit(Bytecode::SetLocal(list_slot), list_expr.range);
                self.emit(Bytecode::Pop, list_expr.range);
                let target = PositionalExpansionTarget::ListLiteral { list_slot };
                for element in list_expr.elements {
                    let range = match &element {
                        ListLiteralElement::Element { range, .. } | ListLiteralElement::Expansion { range, .. } => *range,
                    };
                    match element {
                        ListLiteralElement::Element { expr, .. } => {
                            self.emit_positional_target_before_value(target, range);
                            self.compile_expr(expr)?;
                            self.emit_positional_target_append(target, range);
                        }
                        ListLiteralElement::Expansion { expr, range } => {
                            self.compile_positional_expansion(target, expr, range)?;
                        }
                    }
                }
                self.emit(Bytecode::GetLocal(list_slot), list_expr.range);
                self.emit(Bytecode::FinishListLiteral, list_expr.range);
                self.release_pack_scratch_from(list_slot, 1, list_expr.range);
            }
            Expr::Var { value, range } => {
                if value == "nil" {
                    return Err(CompilerError::UndefinedVariable(value.clone()));
                }
                let name_sym = self.vm.interner.intern(&value);
                match self.resolve_bare_name(name_sym) {
                    BareNameResolution::Local(slot) => self.emit(Bytecode::GetLocal(slot as u16), range),
                    BareNameResolution::Upvalue(upvalue) => self.emit(Bytecode::GetUpvalue(upvalue as u16), range),
                    BareNameResolution::Linked(binding) => self.emit(Bytecode::GetLinked(binding.0 as u16), range),
                    BareNameResolution::Global | BareNameResolution::Unresolved => {
                        let name_idx = self.add_constant(Value::symbol(name_sym));
                        self.emit(Bytecode::GetGlobal(name_idx), range);
                    }
                    BareNameResolution::ImplicitSelf => {
                        self.emit_self(range);
                        let selector_idx = self.add_constant(Value::symbol(name_sym));
                        self.emit(Bytecode::Invoke(0, selector_idx), range);
                    }
                }
            }
            Expr::ImplementationSelector { value, range } => {
                if !self.compiling_privileged_core() {
                    return Err(CompilerError::InternalNamespaceReserved(value, range));
                }
                self.emit_self(range);
                let selector_sym = self.vm.interner.intern(&value);
                let selector_idx = self.add_constant(Value::symbol(selector_sym));
                self.emit(Bytecode::Invoke(0, selector_idx), range);
            }
            Expr::Field { value, kind, range } => {
                if matches!(kind, phalcom_ast::ast::FieldKind::Implementation) && !self.compiling_privileged_core() {
                    return Err(CompilerError::InternalNamespaceReserved(value, range));
                }
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
                        let class_idx = self.add_constant(Value::symbol(class_sym));
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
                        } else if self.linked_binding(name_sym).is_some() {
                            return Err(CompilerError::AssignToImmutable(value));
                        } else if self.resolves_known_global(name_sym) {
                            let is_const_this_unit = self.global_bindings.get(&name_sym) == Some(&false);
                            let is_const_prior_unit =
                                self.unit_kind != UnitKind::Repl && self.vm.heap.module(self.module).global_bindings.get(&name_sym) == Some(&false);
                            if is_const_this_unit || is_const_prior_unit {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            let name_idx = self.add_constant(Value::symbol(name_sym));
                            self.emit(Bytecode::SetGlobal(name_idx), range);
                        } else if self.functions.last().is_some_and(|function| function.has_self) {
                            self.emit_self(range);
                            self.compile_expr(assign_expr.value)?;
                            let selector = make_signature(&value, SignatureKind::Setter);
                            let selector_sym = self.vm.interner.intern(&selector);
                            let selector_idx = self.add_constant(Value::symbol(selector_sym));
                            self.emit(Bytecode::Invoke(1, selector_idx), range);
                        } else {
                            self.compile_expr(assign_expr.value)?;
                            let name_idx = self.add_constant(Value::symbol(name_sym));
                            self.emit(Bytecode::SetGlobal(name_idx), range);
                        }
                    }
                    Expr::Field { value, kind, range } => {
                        if matches!(kind, phalcom_ast::ast::FieldKind::Implementation) && !self.compiling_privileged_core() {
                            return Err(CompilerError::InternalNamespaceReserved(value, range));
                        }
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
                                let class_idx = self.add_constant(Value::symbol(class_sym));
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
                // All four unary operators lower to bare getter sends so user-defined
                // classes can implement them as zero-arg getters with bare selector
                // names: `+`, `-`, `not`, `~`.
                self.compile_expr(unary_expr.expr)?;
                let range = unary_expr.range;
                match unary_expr.op {
                    UnaryOp::Plus => self.emit_getter_send("+", range),
                    UnaryOp::Minus => self.emit_getter_send("-", range),
                    UnaryOp::Not => self.emit_getter_send("not", range),
                    UnaryOp::BitNot => self.emit_getter_send("~", range),
                }
            }
            Expr::Membership(m) => {
                let range = m.range;
                let left_slot = self.reserve_pack_scratch("$mem_left", range)?;
                self.compile_expr(m.left)?;
                self.emit(Bytecode::SetLocal(left_slot), range);
                self.emit(Bytecode::Pop, range);

                let right_slot = self.reserve_pack_scratch("$mem_right", range)?;
                self.compile_expr(m.right)?;
                self.emit(Bytecode::SetLocal(right_slot), range);
                self.emit(Bytecode::Pop, range);

                self.emit(Bytecode::GetLocal(right_slot), range);
                self.emit(Bytecode::GetLocal(left_slot), range);

                self.emit_operator_send("contains", 1, range);
                if m.negated {
                    self.emit_getter_send("not", range);
                }

                self.release_pack_scratch_from(left_slot, 2, range);
            }
            Expr::IsMembership(m) => {
                let range = m.range;
                let left_sym = self.fresh_scratch_symbol("$is_mem_left");
                self.add_local(left_sym, true)?;
                let left_slot = (self.functions.last().unwrap().num_locals - 1) as u16;
                self.emit(Bytecode::ReserveScratchLocal(left_slot), range);
                self.compile_expr(m.left)?;
                self.emit(Bytecode::SetLocal(left_slot), range);
                self.emit(Bytecode::Pop, range);

                let cand_slot = self.reserve_pack_scratch("$is_mem_cand", range)?;
                self.compile_expr(m.candidates)?;
                self.emit(Bytecode::SetLocal(cand_slot), range);
                self.emit(Bytecode::Pop, range);

                self.emit(Bytecode::GetLocal(cand_slot), range);

                let candidate_param_name = "$is_mem_c".to_string();
                let method_name = if m.strict { "is!" } else { "is" }.to_string();
                let left_var_name = self.vm.resolve_symbol(left_sym).to_string();

                let body_expr = Expr::MethodCall(Box::new(MethodCallExpr {
                    object: Expr::Var { value: left_var_name, range },
                    method: method_name,
                    method_range: None,
                    args: vec![PackItem::Positional {
                        expr: Expr::Var {
                            value: candidate_param_name.clone(),
                            range,
                        },
                        range,
                    }],
                    range,
                }));

                let block_expr = Expr::Block(Box::new(BlockExpr {
                    params: ClosureParameters::fixed(vec![candidate_param_name]),
                    body: vec![Statement::Expr { expr: body_expr, range }],
                    expr_body: true,
                    range,
                }));

                self.compile_expr(block_expr)?;

                let labels = [Some("where".to_string())];
                let selector = encode_selector("any", &labels, SignatureKind::Method(1));
                let selector_sym = self.vm.interner.intern(&selector);
                let selector_idx = self.add_constant(Value::symbol(selector_sym));
                self.emit(Bytecode::Invoke(1, selector_idx), range);

                if m.negated {
                    self.emit_getter_send("not", range);
                }

                self.release_pack_scratch_from(left_slot, 2, range);
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
                let name_sym = self.vm.interner.intern("<closure>");
                let constructor_name = self.functions.last().unwrap().constructor_name.clone();
                let closure = self.compile_block(block_expr.body, name_sym, block_expr.params, false, false, constructor_name)?;
                let idx = self.add_constant(Value::obj(closure));
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
                let idx = self.add_constant(Value::symbol(sym));
                self.emit(Bytecode::Constant(idx), range);
            }
            ProductLabel::Computed { expr, range } => {
                self.compile_expr(*expr)?;
                self.emit(Bytecode::GuardSymbol, range);
            }
        }
        Ok(())
    }

    fn compile_selector_spec_constant(&mut self, spec: &SelectorSpecSyntax) -> Result<(u16, FamilySpecKind), CompilerError> {
        let normalized = spec.normalize().map_err(|error| match error {
            phalcom_common::selector::SelectorError::TooManySlots => CompilerError::ArityLimit {
                subject: "pinned selector",
                found: selector_spec_slot_count(spec),
                limit: u8::MAX,
                span: spec.range(),
            },
            error => CompilerError::Message(format!("invalid selector specification: {error}")),
        })?;
        match normalized {
            NormalizedSelectorSpec::Exact(selector) => {
                let symbol = self.vm.interner.intern(&selector.encode());
                Ok((self.add_constant(Value::symbol(symbol)), FamilySpecKind::Exact))
            }
            NormalizedSelectorSpec::Pattern(pattern) => {
                let pattern_object = crate::heap::SelectorPatternObject::compile(pattern, &mut self.vm.interner);
                let object = self.vm.heap.alloc(crate::heap::Object::SelectorPattern(Box::new(pattern_object)));
                Ok((self.add_constant(Value::obj(object)), FamilySpecKind::Pattern))
            }
        }
    }

    fn canonical_symbol(&mut self, kind: SymbolLiteralKind, range: SourceRange) -> Result<crate::interner::Symbol, CompilerError> {
        let canonical = match kind {
            SymbolLiteralKind::Name(name) => name,
            SymbolLiteralKind::Selector { name, labels } => {
                let arity = checked_send_arity("symbol selector", labels.len(), range)?;
                encode_selector(&name, &labels, SignatureKind::Method(arity))
            }
            SymbolLiteralKind::Pattern(_) => return Err(CompilerError::Message("selector patterns are not product labels".into())),
        };
        Ok(self.vm.interner.intern(&canonical))
    }
}

fn selector_spec_slot_count(spec: &SelectorSpecSyntax) -> usize {
    match spec {
        SelectorSpecSyntax::Exact(exact) => exact.slots.len(),
        SelectorSpecSyntax::Pattern(pattern) => pattern.prefix.len() + pattern.suffix.len() + 1,
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
/// - immediate `None`, which lexes to `Var { value: "None" }`; and
/// - a canonical `Some(…)` unqualified call; and
/// - explicit `Some.call(…)` or compatibility `Some.new(…)` sends.
///
/// This is the literal-only half of BD-U6-1's `if (opt)` compile check; every
/// non-literal, non-`Bool` condition is caught at runtime by the branch
/// opcode's `Bool` requirement.
fn is_option_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Var { value, .. } => value == "None",
        Expr::UnqualifiedCall(call) => call.name == "Some",
        Expr::MethodCall(call) => (call.method == "call" || call.method == "new") && matches!(&call.object, Expr::Var { value, .. } if value == "Some"),
        _ => false,
    }
}

/// Wraps `expr` in a synthetic 0-parameter, expression-bodied block literal
/// spanning `range`, for `and`/`or`'s lazily-evaluated right-hand side
/// (control-flow.md §2: `a and b` ≡ `a.and { b }`).
fn wrap_expr_as_lazy_block(expr: Expr, range: SourceRange) -> Expr {
    Expr::Block(Box::new(BlockExpr {
        params: phalcom_ast::ast::ClosureParameters::default(),
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
