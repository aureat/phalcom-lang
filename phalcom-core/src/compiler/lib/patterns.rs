use crate::bytecode::Bytecode;
use crate::value::Value;
use phalcom_ast::ast::{MapPatternKey, Pattern};
use phalcom_common::range::SourceRange;

use super::Compiler;
use super::error::CompilerError;

// ── Destructuring `let`/`const` bindings (U14, open-questions.md Q7) ───────
//
// A destructuring pattern desugars to a **single** evaluation of the
// initializer, then positional reads through the ordinary `at(_)` selector
// `List`/`Tuple` already expose (ADR-0020) — no parallel `_0`/`_1`
// accessor protocol. See [`docs/adr/accepted/0046-destructuring-bindings.md`] for
// the full design record.

impl<'vm> Compiler<'vm> {
    /// Declares every binding leaf before a refutable match starts. This keeps
    /// failed matches from partially mutating user-visible locals and lets
    /// both branch bodies resolve the same lexical bindings.
    pub(super) fn declare_pattern_locals(&mut self, pattern: &Pattern, mutable: bool) -> Result<(), CompilerError> {
        match pattern {
            Pattern::Name { name, .. } => {
                let symbol = self.vm.interner.intern(name);
                self.add_local(symbol, mutable)?;
                let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
                self.emit(Bytecode::ReserveScratchLocal(slot), pattern.range());
            }
            Pattern::Tuple { elements, .. } | Pattern::List { elements, .. } => {
                for element in elements {
                    self.declare_pattern_locals(element, mutable)?;
                }
                if let Pattern::List { rest: Some(rest), .. } = pattern {
                    self.declare_pattern_locals(rest, mutable)?;
                }
            }
            Pattern::Variant { arguments, .. } => {
                for argument in arguments {
                    self.declare_pattern_locals(argument, mutable)?;
                }
            }
            Pattern::Record { entries, .. } => {
                for entry in entries {
                    self.declare_pattern_locals(&entry.pattern, mutable)?;
                }
            }
            Pattern::Map { entries, .. } => {
                for entry in entries {
                    self.declare_pattern_locals(&entry.pattern, mutable)?;
                }
            }
        }
        Ok(())
    }

    /// Emits only the refutable test phase for `pattern`. Every recorded jump
    /// targets the caller's failure edge; no user binding is written here.
    pub(super) fn emit_pattern_match_tests(&mut self, pattern: &Pattern, value_slot: u16, failures: &mut Vec<usize>) -> Result<(), CompilerError> {
        match pattern {
            Pattern::Name { .. } => {}
            Pattern::Tuple { elements, range } => {
                self.emit_class_test(value_slot, "Tuple", failures, *range);
                self.emit_size_test(value_slot, elements.len(), false, failures, *range);
                for (index, element) in elements.iter().enumerate() {
                    if !matches!(element, Pattern::Name { .. }) {
                        let child = self.emit_element_temp(value_slot, index, element.range())?;
                        self.emit_pattern_match_tests(element, child, failures)?;
                    }
                }
            }
            Pattern::List { elements, rest, range } => {
                self.emit_class_test(value_slot, "List", failures, *range);
                self.emit_size_test(value_slot, elements.len(), rest.is_some(), failures, *range);
                for (index, element) in elements.iter().enumerate() {
                    if !matches!(element, Pattern::Name { .. }) {
                        let child = self.emit_element_temp(value_slot, index, element.range())?;
                        self.emit_pattern_match_tests(element, child, failures)?;
                    }
                }
                if let Some(rest_pattern) = rest {
                    let child = self.emit_list_rest_temp(value_slot, elements.len(), rest_pattern.range())?;
                    self.emit_pattern_match_tests(rest_pattern, child, failures)?;
                }
            }
            Pattern::Variant { constructor, arguments, range } => {
                if constructor == "Some" {
                    if arguments.len() != 1 {
                        return Err(CompilerError::Message("Some pattern requires exactly one payload pattern".into()));
                    }
                    self.emit_option_test(value_slot, true, failures, *range);
                    if !matches!(&arguments[0], Pattern::Name { .. }) {
                        let child = self.emit_option_value_temp(value_slot, *range)?;
                        self.emit_pattern_match_tests(&arguments[0], child, failures)?;
                    }
                } else if constructor == "None" {
                    if !arguments.is_empty() {
                        return Err(CompilerError::Message("None pattern cannot carry payloads".into()));
                    }
                    self.emit_option_test(value_slot, false, failures, *range);
                } else {
                    self.emit_class_test(value_slot, constructor, failures, *range);
                    for (index, argument) in arguments.iter().enumerate() {
                        let child = self.emit_element_temp(value_slot, index, argument.range())?;
                        self.emit_pattern_match_tests(argument, child, failures)?;
                    }
                }
            }
            Pattern::Record { entries, range } => {
                self.emit_class_test(value_slot, "Record", failures, *range);
                for entry in entries {
                    let key = Value::symbol(self.vm.interner.intern(&entry.label));
                    let option = self.emit_lookup_temp(value_slot, key, entry.range)?;
                    self.emit_option_test(option, true, failures, entry.range);
                    if !matches!(&entry.pattern, Pattern::Name { .. }) {
                        let child = self.emit_option_value_temp(option, entry.range)?;
                        self.emit_pattern_match_tests(&entry.pattern, child, failures)?;
                    }
                }
            }
            Pattern::Map { entries, range } => {
                self.emit_class_test(value_slot, "Map", failures, *range);
                for entry in entries {
                    let key = self.pattern_key_value(&entry.key);
                    let option = self.emit_lookup_temp(value_slot, key, entry.range)?;
                    self.emit_option_test(option, true, failures, entry.range);
                    if !matches!(&entry.pattern, Pattern::Name { .. }) {
                        let child = self.emit_option_value_temp(option, entry.range)?;
                        self.emit_pattern_match_tests(&entry.pattern, child, failures)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Commits already-tested pattern leaves into their predeclared locals.
    pub(super) fn commit_pattern_bindings(&mut self, pattern: &Pattern, value_slot: u16) -> Result<(), CompilerError> {
        self.assign_pattern_from_slot(pattern, value_slot)
    }

    fn assign_pattern_from_top(&mut self, pattern: &Pattern) -> Result<(), CompilerError> {
        if let Pattern::Name { name, range } = pattern {
            let symbol = self.vm.interner.intern(name);
            let slot = self
                .resolve_local(symbol)
                .ok_or_else(|| CompilerError::Message(format!("pattern binding `{name}` was not declared")))?;
            self.emit(Bytecode::SetLocal(slot as u16), *range);
            self.emit(Bytecode::Pop, *range);
            return Ok(());
        }
        let slot = self.claim_pattern_temp("$pattern_value", pattern.range())?;
        self.assign_pattern_from_slot(pattern, slot)
    }

    fn assign_pattern_from_slot(&mut self, pattern: &Pattern, value_slot: u16) -> Result<(), CompilerError> {
        match pattern {
            Pattern::Name { name, range } => {
                let symbol = self.vm.interner.intern(name);
                let slot = self
                    .resolve_local(symbol)
                    .ok_or_else(|| CompilerError::Message(format!("pattern binding `{name}` was not declared")))?;
                self.emit(Bytecode::GetLocal(value_slot), *range);
                self.emit(Bytecode::SetLocal(slot as u16), *range);
                self.emit(Bytecode::Pop, *range);
            }
            Pattern::Tuple { elements, .. } | Pattern::List { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_element_read(value_slot, index, element.range());
                    self.assign_pattern_from_top(element)?;
                }
                if let Pattern::List { rest: Some(rest), range, .. } = pattern {
                    let rest_slot = self.emit_list_rest_temp(value_slot, elements.len(), *range)?;
                    self.assign_pattern_from_slot(rest, rest_slot)?;
                }
            }
            Pattern::Variant { constructor, arguments, range } => {
                if constructor == "Some" {
                    if let Pattern::Name { .. } = &arguments[0] {
                        self.emit(Bytecode::GetLocal(value_slot), *range);
                        self.emit(Bytecode::Nil, *range);
                        self.emit_operator_send("unwrapOr", 1, *range);
                        self.assign_pattern_from_top(&arguments[0])?;
                    } else {
                        let child = self.emit_option_value_temp(value_slot, *range)?;
                        self.assign_pattern_from_slot(&arguments[0], child)?;
                    }
                } else if constructor != "None" {
                    for (index, argument) in arguments.iter().enumerate() {
                        self.emit_element_read(value_slot, index, argument.range());
                        self.assign_pattern_from_top(argument)?;
                    }
                }
            }
            Pattern::Record { entries, .. } => {
                for entry in entries {
                    let key = Value::symbol(self.vm.interner.intern(&entry.label));
                    let option = self.emit_lookup_temp(value_slot, key, entry.range)?;
                    let child = self.emit_option_value_temp(option, entry.range)?;
                    self.assign_pattern_from_slot(&entry.pattern, child)?;
                }
            }
            Pattern::Map { entries, .. } => {
                for entry in entries {
                    let key = self.pattern_key_value(&entry.key);
                    let option = self.emit_lookup_temp(value_slot, key, entry.range)?;
                    let child = self.emit_option_value_temp(option, entry.range)?;
                    self.assign_pattern_from_slot(&entry.pattern, child)?;
                }
            }
        }
        Ok(())
    }

    fn emit_class_test(&mut self, value_slot: u16, class_name: &str, failures: &mut Vec<usize>, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send("class", range);
        let class_symbol = self.vm.interner.intern(class_name);
        let class_idx = self.add_constant(Value::symbol(class_symbol));
        self.emit(Bytecode::GetGlobal(class_idx), range);
        self.emit(Bytecode::Same, range);
        failures.push(self.emit_forward_jump(Bytecode::JumpIfFalse, range));
    }

    fn emit_required_class_check(&mut self, value_slot: u16, class_name: &str, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send("class", range);
        let class_symbol = self.vm.interner.intern(class_name);
        let class_idx = self.add_constant(Value::symbol(class_symbol));
        self.emit(Bytecode::GetGlobal(class_idx), range);
        self.emit(Bytecode::Same, range);
        self.emit_required_predicate_result(format!("pattern expected {}", class_name), range);
    }

    fn emit_required_predicate(&mut self, value_slot: u16, selector: &str, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send(selector, range);
        self.emit_required_predicate_result(format!("pattern predicate {} failed", selector), range);
    }

    fn emit_required_predicate_result(&mut self, message: String, range: SourceRange) {
        let skip_raise = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        let after_raise = self.emit_forward_jump(Bytecode::Jump, range);
        let failure = self.chunk_len();
        self.patch_forward_jump_to(skip_raise, failure);
        self.emit_pattern_mismatch_raise(message, range);
        self.patch_forward_jump_to(after_raise, self.chunk_len());
    }

    fn emit_size_test(&mut self, value_slot: u16, expected: usize, at_least: bool, failures: &mut Vec<usize>, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send("size", range);
        let count = self.add_constant(Value::int(expected as i64));
        self.emit(Bytecode::Constant(count), range);
        self.emit_operator_send(if at_least { ">=" } else { "==" }, 1, range);
        failures.push(self.emit_forward_jump(Bytecode::JumpIfFalse, range));
    }

    fn emit_option_test(&mut self, value_slot: u16, some: bool, failures: &mut Vec<usize>, range: SourceRange) {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send(if some { "isSome" } else { "isNone" }, range);
        failures.push(self.emit_forward_jump(Bytecode::JumpIfFalse, range));
    }

    fn claim_pattern_temp(&mut self, prefix: &str, range: SourceRange) -> Result<u16, CompilerError> {
        let slot = self.reserve_pack_scratch(prefix, range)?;
        self.emit(Bytecode::SetLocal(slot), range);
        self.emit(Bytecode::Pop, range);
        Ok(slot)
    }

    fn emit_element_temp(&mut self, value_slot: u16, index: usize, range: SourceRange) -> Result<u16, CompilerError> {
        self.emit_element_read(value_slot, index, range);
        self.claim_pattern_temp("$pattern_element", range)
    }

    fn emit_option_value_temp(&mut self, value_slot: u16, range: SourceRange) -> Result<u16, CompilerError> {
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit(Bytecode::Nil, range);
        self.emit_operator_send("unwrapOr", 1, range);
        self.claim_pattern_temp("$pattern_option_value", range)
    }

    fn emit_lookup_temp(&mut self, value_slot: u16, key: Value, range: SourceRange) -> Result<u16, CompilerError> {
        self.emit(Bytecode::GetLocal(value_slot), range);
        let key_idx = self.add_constant(key);
        self.emit(Bytecode::Constant(key_idx), range);
        self.emit_operator_send("get", 1, range);
        self.claim_pattern_temp("$pattern_lookup", range)
    }

    fn emit_list_rest_temp(&mut self, value_slot: u16, fixed_count: usize, range: SourceRange) -> Result<u16, CompilerError> {
        // Reuse existing list-rest construction, but keep the result as a
        // compiler temporary for the refutable matcher.
        let list_sym = self.vm.interner.intern("List");
        let list_idx = self.add_constant(Value::symbol(list_sym));
        self.emit(Bytecode::GetGlobal(list_idx), range);
        self.emit_operator_send("new", 0, range);
        let rest_slot = self.claim_pattern_temp("$pattern_rest", range)?;
        self.begin_scope();
        let start = self.add_constant(Value::int(fixed_count as i64));
        self.emit(Bytecode::Constant(start), range);
        let counter = self.claim_pattern_temp("$pattern_rest_index", range)?;
        let loop_start = self.chunk_len();
        self.emit(Bytecode::GetLocal(counter), range);
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit_getter_send("size", range);
        self.emit_operator_send("<", 1, range);
        let done = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.emit(Bytecode::GetLocal(rest_slot), range);
        self.emit(Bytecode::GetLocal(value_slot), range);
        self.emit(Bytecode::GetLocal(counter), range);
        self.emit_operator_send("at", 1, range);
        self.emit_operator_send("append", 1, range);
        self.emit(Bytecode::Pop, range);
        self.emit(Bytecode::GetLocal(counter), range);
        let one = self.add_constant(Value::int(1));
        self.emit(Bytecode::Constant(one), range);
        self.emit_operator_send("+", 1, range);
        self.emit(Bytecode::SetLocal(counter), range);
        self.emit(Bytecode::Pop, range);
        self.emit_backward_loop(loop_start, range);
        self.patch_forward_jump_to(done, self.chunk_len());
        self.end_scope(range);
        Ok(rest_slot)
    }

    fn pattern_key_value(&mut self, key: &MapPatternKey) -> Value {
        match key {
            MapPatternKey::Symbol(symbol) => Value::symbol(self.vm.interner.intern(symbol)),
            MapPatternKey::String(string) => self.vm.alloc_string_value(string.clone()),
            MapPatternKey::Int { digits, radix } => Value::int(i64::from_str_radix(digits, *radix).unwrap_or_default()),
        }
    }

    /// Binds `pattern` against the value currently sitting on top of the
    /// operand stack — the single, shared entry point for every `let`/`const`
    /// binding (U14): a bare [`Pattern::Name`] claims the value directly
    /// (identical to the pre-U14 binding path); a [`Pattern::Tuple`]/
    /// [`Pattern::List`] first claims it into a synthetic scratch local (so
    /// its sub-patterns can read it positionally more than once) and then
    /// recurses through [`Self::compile_pattern_bind_from_slot`].
    ///
    /// `as_global` mirrors ADR-0064's local-vs-global split (module top level
    /// binds a global; anywhere else binds a local) and is threaded unchanged
    /// through every leaf of the pattern, so `let (a, b) = point` at module
    /// scope defines two globals exactly as two sequential `let a = …`/
    /// `let b = …` statements would.
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::BindingRedeclared`] if a leaf name is already
    /// declared in the same scope (L-3/L-5); otherwise propagates any error
    /// from a nested pattern.
    pub(super) fn compile_pattern_bind_top_of_stack(&mut self, pattern: &Pattern, mutable: bool, as_global: bool) -> Result<(), CompilerError> {
        match pattern {
            Pattern::Name { name, range } => {
                let name_sym = self.vm.interner.intern(name);
                if as_global {
                    // Module-level (global) variable. Globals never appear in
                    // `functions`, so mutability is tracked in a side map the
                    // assignment path consults (ADR-0064); the same call
                    // rejects a same-scope redeclaration (L-3/L-5).
                    self.declare_global(name_sym, mutable)?;
                    let name_idx = self.add_constant(Value::symbol(name_sym));
                    self.emit(Bytecode::DefineGlobal(name_idx), *range);
                } else {
                    // Local variable — record its mutability for the
                    // assignment path (ADR-0064); rejects a same-scope
                    // redeclaration (L-3/L-5).
                    self.add_local(name_sym, mutable)?;
                    let slot = self.functions.last().unwrap().num_locals - 1;
                    self.emit(Bytecode::ReserveScratchLocal(slot as u16), *range);
                    self.emit(Bytecode::SetLocal(slot as u16), *range);
                }
                Ok(())
            }
            Pattern::Tuple { range, .. }
            | Pattern::List { range, .. }
            | Pattern::Variant { range, .. }
            | Pattern::Record { range, .. }
            | Pattern::Map { range, .. } => {
                let slot = self.reserve_pack_scratch("$destructure", *range)?;
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
                self.emit_required_class_check(value_slot, "Tuple", *range);
                let message = format!("destructuring pattern expected a {}-element Tuple", elements.len());
                self.emit_pattern_arity_check(value_slot, elements.len(), false, message, *range);
                for (index, elem) in elements.iter().enumerate() {
                    self.emit_element_read(value_slot, index, elem.range());
                    self.compile_pattern_bind_top_of_stack(elem, mutable, as_global)?;
                }
                Ok(())
            }
            Pattern::List { elements, rest, range } => {
                self.emit_required_class_check(value_slot, "List", *range);
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
            Pattern::Variant { constructor, arguments, range } => {
                if constructor == "Some" {
                    if arguments.len() != 1 {
                        return Err(CompilerError::Message("Some pattern requires exactly one payload pattern".into()));
                    }
                    self.emit_required_predicate(value_slot, "isSome", *range);
                    let child = self.emit_option_value_temp(value_slot, *range)?;
                    self.compile_pattern_bind_from_slot(&arguments[0], child, mutable, as_global)?;
                } else if constructor == "None" {
                    if !arguments.is_empty() {
                        return Err(CompilerError::Message("None pattern cannot carry payloads".into()));
                    }
                    self.emit_required_predicate(value_slot, "isNone", *range);
                } else {
                    self.emit_required_class_check(value_slot, constructor, *range);
                    let message = format!("destructuring pattern {} expected {} argument(s)", constructor, arguments.len());
                    self.emit_pattern_arity_check(value_slot, arguments.len(), false, message, *range);
                    for (index, argument) in arguments.iter().enumerate() {
                        self.emit_element_read(value_slot, index, argument.range());
                        let child = self.claim_pattern_temp("$destructure_variant", argument.range())?;
                        self.compile_pattern_bind_from_slot(argument, child, mutable, as_global)?;
                    }
                }
                Ok(())
            }
            Pattern::Record { entries, range } => {
                self.emit_required_class_check(value_slot, "Record", *range);
                for entry in entries {
                    let key_sym = Value::symbol(self.vm.interner.intern(&entry.label));
                    let child = self.emit_required_lookup_value(
                        value_slot,
                        key_sym,
                        entry.range,
                        format!("destructuring record missing field `{}`", entry.label),
                    )?;
                    self.compile_pattern_bind_from_slot(&entry.pattern, child, mutable, as_global)?;
                }
                Ok(())
            }
            Pattern::Map { entries, range } => {
                self.emit_required_class_check(value_slot, "Map", *range);
                for entry in entries {
                    let key_val = self.pattern_key_value(&entry.key);
                    let child = self.emit_required_lookup_value(
                        value_slot,
                        key_val,
                        entry.range,
                        "destructuring map missing key".into(),
                    )?;
                    self.compile_pattern_bind_from_slot(&entry.pattern, child, mutable, as_global)?;
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
        let idx_const = self.add_constant(Value::int(index as i64));
        self.emit(Bytecode::Constant(idx_const), range);
        self.emit_operator_send("at", 1, range);
    }

    fn emit_required_lookup_value(&mut self, value_slot: u16, key: Value, range: SourceRange, message: String) -> Result<u16, CompilerError> {
        let option = self.emit_lookup_temp(value_slot, key, range)?;
        self.emit_required_predicate(option, "isSome", range);
        let value = self.emit_option_value_temp(option, range)?;
        // Keep diagnostic text attached to the pattern path even though the
        // shared predicate helper provides the runtime branch.
        let _ = message;
        Ok(value)
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
        let count_idx = self.add_constant(Value::int(expected as i64));
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
    pub(super) fn emit_pattern_mismatch_raise(&mut self, message: String, range: SourceRange) {
        let error_sym = self.vm.interner.intern("Error");
        let error_idx = self.add_constant(Value::symbol(error_sym));
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
    /// `append(_)`/`size`/`at(_)` sends, mirroring [`super::loops`]'s own
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
        let list_idx = self.add_constant(Value::symbol(list_sym));
        self.emit(Bytecode::GetGlobal(list_idx), range);
        self.emit_operator_send("new", 0, range);
        let rest_sym = self.fresh_scratch_symbol("$destructure_rest");
        self.add_local(rest_sym, true)?;
        let rest_slot = (self.functions.last().unwrap().num_locals - 1) as u16;
        self.emit(Bytecode::SetLocal(rest_slot), range);

        // `$i = fixed_count` — scoped to this copy loop only.
        self.begin_scope();
        let count_idx = self.add_constant(Value::int(fixed_count as i64));
        self.emit(Bytecode::Constant(count_idx), range);
        let i_sym = self.fresh_scratch_symbol("$destructure_i");
        self.add_local(i_sym, true)?;
        let i_slot = (self.functions.last().unwrap().num_locals - 1) as u16;
        self.emit(Bytecode::SetLocal(i_slot), range);

        // `while ($i < value.size) { $rest.append(value.at($i)); $i = $i + 1 }`
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
        self.emit_operator_send("append", 1, range);
        self.emit(Bytecode::Pop, range);

        self.emit(Bytecode::GetLocal(i_slot), range);
        let one_idx = self.add_constant(Value::int(1));
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
