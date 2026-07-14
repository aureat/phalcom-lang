use crate::bytecode::Bytecode;
use crate::value::Value;
use phalcom_ast::ast::Pattern;
use phalcom_common::range::SourceRange;

use super::error::CompilerError;
use super::Compiler;

// ── Destructuring `let`/`var` bindings (U14, open-questions.md Q7) ───────
//
// A destructuring pattern desugars to a **single** evaluation of the
// initializer, then positional reads through the ordinary `at(_)` selector
// `List`/`Tuple` already expose (ADR-0020) — no parallel `_0`/`_1`
// accessor protocol. See [`docs/adr/accepted/0046-destructuring-bindings.md`] for
// the full design record.

impl<'vm> Compiler<'vm> {
    /// Binds `pattern` against the value currently sitting on top of the
    /// operand stack — the single, shared entry point for every `let`/`var`
    /// binding (U14): a bare [`Pattern::Name`] claims the value directly
    /// (identical to the pre-U14 binding path); a [`Pattern::Tuple`]/
    /// [`Pattern::List`] first claims it into a synthetic scratch local (so
    /// its sub-patterns can read it positionally more than once) and then
    /// recurses through [`Self::compile_pattern_bind_from_slot`].
    ///
    /// `as_global` mirrors ADR-0014's existing local-vs-global split (module
    /// top level binds a global; anywhere else binds a local) and is threaded
    /// unchanged through every leaf of the pattern, so `let (a, b) = point`
    /// at module scope defines two globals exactly as two sequential
    /// `let a = …`/`let b = …` statements would.
    ///
    /// # Errors
    ///
    /// Propagates any error from a nested pattern (there are currently none —
    /// the pattern was already validated by the parser — but the signature
    /// stays fallible so a future refutable pattern can plug in here without
    /// reshaping the call sites, ADR-0046 §4).
    pub(super) fn compile_pattern_bind_top_of_stack(&mut self, pattern: &Pattern, mutable: bool, as_global: bool) -> Result<(), CompilerError> {
        match pattern {
            Pattern::Name { name, range } => {
                let name_sym = self.vm.interner.intern(name);
                if as_global {
                    // Module-level (global) variable. Globals never appear in
                    // `functions`, so an immutable `let` is tracked in a side
                    // set the assignment path consults (ADR-0014).
                    if !mutable {
                        self.immutable_globals.insert(name_sym);
                    }
                    let name_idx = self.add_constant(Value::Symbol(name_sym));
                    self.emit(Bytecode::DefineGlobal(name_idx), *range);
                } else {
                    // Local variable — record its mutability for the
                    // assignment path (ADR-0014).
                    self.add_local(name_sym, mutable);
                    let slot = self.functions.last().unwrap().num_locals - 1;
                    self.emit(Bytecode::SetLocal(slot as u16), *range);
                }
                Ok(())
            }
            Pattern::Tuple { range, .. } | Pattern::List { range, .. } => {
                let scratch_sym = self.vm.interner.intern("$destructure");
                self.add_local(scratch_sym, true);
                let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
                self.emit(Bytecode::SetLocal(slot), *range);
                self.compile_pattern_bind_from_slot(pattern, slot, mutable, as_global)
            }
        }
    }

    /// Binds `pattern` against the value already resident in local `value_slot`
    /// — the recursive workhorse behind [`Self::compile_pattern_bind_top_of_stack`].
    ///
    /// A [`Pattern::Tuple`] requires the scrutinee's `size` to match the
    /// pattern's arity **exactly**; a [`Pattern::List`] with no `*rest`
    /// likewise; a [`Pattern::List`] with `*rest` requires `size` to be **at
    /// least** the fixed element count. Either mismatch raises a clean
    /// `Error` at runtime ([`Self::emit_pattern_arity_check`]) rather than
    /// silently truncating or panicking — the bind is irrefutable (ADR-0046
    /// §2). A non-`Tuple`/`List` scrutinee instead raises the natural
    /// `doesNotUnderstand` miss on the `size`/`at(_)` sends themselves.
    ///
    /// # Errors
    ///
    /// Propagates any error from a nested pattern (see
    /// [`Self::compile_pattern_bind_top_of_stack`]'s `# Errors`).
    fn compile_pattern_bind_from_slot(&mut self, pattern: &Pattern, value_slot: u16, mutable: bool, as_global: bool) -> Result<(), CompilerError> {
        match pattern {
            Pattern::Name { .. } => {
                let range = pattern.range();
                self.emit(Bytecode::GetLocal(value_slot), range);
                self.compile_pattern_bind_top_of_stack(pattern, mutable, as_global)
            }
            Pattern::Tuple { elements, range } => {
                let message = format!("destructuring pattern expected a {}-element Tuple", elements.len());
                self.emit_pattern_arity_check(value_slot, elements.len(), false, message, *range);
                for (index, elem) in elements.iter().enumerate() {
                    self.emit_element_read(value_slot, index, elem.range());
                    self.compile_pattern_bind_top_of_stack(elem, mutable, as_global)?;
                }
                Ok(())
            }
            Pattern::List { elements, rest, range } => {
                let message = if rest.is_some() {
                    format!("destructuring pattern expected a List of at least {} element(s)", elements.len())
                } else {
                    format!("destructuring pattern expected a {}-element List", elements.len())
                };
                self.emit_pattern_arity_check(value_slot, elements.len(), rest.is_some(), message, *range);
                for (index, elem) in elements.iter().enumerate() {
                    self.emit_element_read(value_slot, index, elem.range());
                    self.compile_pattern_bind_top_of_stack(elem, mutable, as_global)?;
                }
                if let Some(rest_pattern) = rest {
                    self.compile_list_rest_and_bind(value_slot, elements.len(), rest_pattern, mutable, as_global, *range)?;
                }
                Ok(())
            }
        }
    }

    /// Emits `GetLocal(value_slot).at(index)`, leaving the element's value on
    /// top of the operand stack — the shared positional read every pattern
    /// element compiles through (ADR-0020's `at(_)`, ADR-0046 §1).
    fn emit_element_read(&mut self, value_slot: u16, index: usize, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        let idx_const = self.add_constant(Value::Number(index as f64));
        self.emit(Bytecode::Constant(idx_const), range);
        self.emit_operator_send("at", 1, range);
    }

    /// Emits an inline arity guard for a destructuring pattern (ADR-0046 §2):
    /// if the scrutinee held in `value_slot` doesn't match the pattern's
    /// shape, raises a fresh `Error` carrying `message` instead of falling
    /// through to a silent truncation.
    ///
    /// `at_least` selects the comparison: `false` requires `size` to equal
    /// `expected` exactly (a `Tuple` pattern, or a rest-less `List` pattern);
    /// `true` requires `size >= expected` (a `List` pattern with `*rest`).
    fn emit_pattern_arity_check(&mut self, value_slot: u16, expected: usize, at_least: bool, message: String, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send("size", range);
        let count_idx = self.add_constant(Value::Number(expected as f64));
        self.emit(Bytecode::Constant(count_idx), range);
        // `size < expected` (rest form) or `size != expected` (exact form) is
        // a mismatch — the condition pushed here is true exactly when the
        // pattern does NOT match.
        self.emit_operator_send(if at_least { "<" } else { "!=" }, 1, range);
        let skip_raise = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.emit_pattern_mismatch_raise(message, range);
        let after_raise = self.chunk_len();
        self.patch_forward_jump_to(skip_raise, after_raise);
    }

    /// Emits `Error.new(message).raise()` — the shape-mismatch diagnostic a
    /// destructuring pattern's arity guard raises (ADR-0046 §2). Mirrors
    /// [`super::loops`]'s deopt-block-control-trap raise-and-balance idiom.
    fn emit_pattern_mismatch_raise(&mut self, message: String, range: SourceRange) {
        let error_sym = self.vm.interner.intern("Error");
        let error_idx = self.add_constant(Value::Symbol(error_sym));
        self.emit(Bytecode::GetGlobal(error_idx), range);
        let message_obj = self.vm.alloc_string_value(message);
        let message_idx = self.add_constant(message_obj);
        self.emit(Bytecode::Constant(message_idx), range);
        self.emit_operator_send("new", 1, range);
        self.emit_operator_send("raise", 0, range);
        // `raise` never returns normally, so this `Pop` is unreachable dead
        // code — kept only so the chunk's static stack shape stays balanced.
        self.emit(Bytecode::Pop, range);
    }

    /// Builds the `*rest` tail of a [`Pattern::List`] as a fresh `List`
    /// holding every element of `value_slot` from index `fixed_count`
    /// onward, then binds `rest_pattern` against it (ADR-0046 §1).
    ///
    /// Realized as an inlined `while` copy loop — `List`/`Tuple` expose no
    /// slice/tail selector, so this reuses only the existing `List.new()`/
    /// `add(_)`/`size`/`at(_)` sends, mirroring [`super::loops`]'s own
    /// `compile_for` hand-rolled loop skeleton. The loop counter is a synthetic local
    /// scoped away once the copy finishes (it never needs to outlive this
    /// call); the built `List` itself is claimed as a scratch local that
    /// survives to be bound below.
    ///
    /// # Errors
    ///
    /// Propagates any error from binding `rest_pattern`.
    fn compile_list_rest_and_bind(
        &mut self,
        value_slot: u16,
        fixed_count: usize,
        rest_pattern: &Pattern,
        mutable: bool,
        as_global: bool,
        range: SourceRange,
    ) -> Result<(), CompilerError> {
        // `$rest = List.new()`
        let list_sym = self.vm.interner.intern("List");
        let list_idx = self.add_constant(Value::Symbol(list_sym));
        self.emit(Bytecode::GetGlobal(list_idx), range);
        self.emit_operator_send("new", 0, range);
        let rest_sym = self.vm.interner.intern("$destructure_rest");
        self.add_local(rest_sym, true);
        let rest_slot = (self.functions.last().unwrap().num_locals - 1) as u16;
        self.emit(Bytecode::SetLocal(rest_slot), range);

        // `$i = fixed_count` — scoped to this copy loop only.
        self.begin_scope();
        let count_idx = self.add_constant(Value::Number(fixed_count as f64));
        self.emit(Bytecode::Constant(count_idx), range);
        let i_sym = self.vm.interner.intern("$destructure_i");
        self.add_local(i_sym, true);
        let i_slot = (self.functions.last().unwrap().num_locals - 1) as u16;
        self.emit(Bytecode::SetLocal(i_slot), range);

        // `while ($i < value.size) { $rest.add(value.at($i)); $i = $i + 1 }`
        let loop_start = self.chunk_len();
        self.emit(Bytecode::GetLocal(i_slot), range);
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send("size", range);
        self.emit_operator_send("<", 1, range);
        let exit_on_false = self.emit_forward_jump(Bytecode::JumpIfFalse, range);

        self.emit(Bytecode::GetLocal(rest_slot), range);
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit(Bytecode::GetLocal(i_slot), range);
        self.emit_operator_send("at", 1, range);
        self.emit_operator_send("add", 1, range);
        self.emit(Bytecode::Pop, range);

        self.emit(Bytecode::GetLocal(i_slot), range);
        let one_idx = self.add_constant(Value::Number(1.0));
        self.emit(Bytecode::Constant(one_idx), range);
        self.emit_operator_send("+", 1, range);
        self.emit(Bytecode::SetLocal(i_slot), range);
        self.emit(Bytecode::Pop, range);
        self.emit_backward_loop(loop_start, range);

        let exit_label = self.chunk_len();
        self.patch_forward_jump_to(exit_on_false, exit_label);
        self.end_scope(range);

        self.compile_pattern_bind_from_slot(rest_pattern, rest_slot, mutable, as_global)
    }
}
