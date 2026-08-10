//! The AST-to-bytecode compiler.
//!
//! Lowers a parsed [`Program`] into a [`ClosureObject`] whose [`Chunk`](crate::chunk::Chunk) the VM
//! executes. String, method and superclass constants are materialized onto the
//! [`Heap`](crate::heap::Heap) as the compiler emits them — the compiler already
//! holds `&mut VM` and therefore the heap — and are referenced from the constant
//! pool by `Copy` [`Value::Obj`] handles
//! ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md),
//! [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md)).

mod class_decl;
mod error;
mod expr;
mod jumps;
mod loops;
mod patterns;
mod scope;
mod state;

pub use error::CompilerError;
pub(crate) use error::{checked_product_count, checked_send_arity};

/// Whether a compilation unit is a whole file or a single REPL cell.
///
/// Orthogonal to [`CompileMode`](crate::compiler::attributes::CompileMode), which
/// governs contract weaving and is global; this is per-unit. A `Repl` unit keeps
/// its final expression's value instead of popping it (U-REPL §D3) and relaxes
/// prior-unit binding checks (§D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnitKind {
    /// A whole source file. Every statement's value is discarded.
    #[default]
    File,
    /// One REPL cell.
    Repl,
}

use super::boundedness;
use crate::bytecode::Bytecode;
use crate::callable::Callable;
use crate::error::PhResult;
use crate::heap::ClosureObject;
use crate::heap::{ObjRef, Object};
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::{ClassKey, VM};
use phalcom_ast::ast::{BindingKind, ClosureParameters, Expr, MethodCallExpr, Pattern, Program, Statement};
use phalcom_common::range::{EmptySourceRange, SourceRange};
use state::FunctionState;
use state::LoopContext;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct Compiler<'vm> {
    pub(crate) vm: &'vm mut VM,
    module: ObjRef,
    /// The stack of function-compilation states, innermost body last. A block
    /// literal pushes a new state, compiles into it, and pops it; upvalue
    /// resolution indexes into this stack rather than following raw pointers.
    pub(crate) functions: Vec<FunctionState>,
    /// Every module-level `let`/`const` binding declared so far, keyed by
    /// name, recording its mutability.
    ///
    /// Module-level bindings become globals rather than stack locals
    /// ([`Bytecode::DefineGlobal`]), so [`Self::functions`] never sees them and
    /// cannot record their mutability. This map lets the assignment path
    /// reject a store to a `const` global at compile time, and lets
    /// [`Self::declare_global`] reject a same-scope redeclaration of either
    /// kind (`binding.redeclared`) — a `const` may never be released by a
    /// later `let` of the same name
    /// ([ADR-0064](../../../docs/adr/accepted/0064-let-const-bindings-and-field-mutability.md),
    /// rulings L-3/L-5). There is deliberately no `remove`: a name, once
    /// declared, keeps its declared kind for the rest of compilation.
    ///
    /// **Class names register here too, as of U-CLASSCLOSE (PDR-0002
    /// ruling 8) — but only by insertion, never through [`Self::declare_global`]
    /// itself.** `Statement::Class` still emits its own `DefineGlobal`
    /// directly (`class_decl.rs`) and keeps its own diagnostic
    /// (`class.already_defined`, not `binding.redeclared` — a class can be
    /// neither assigned nor nested, so `BindingRedeclared`'s guidance would
    /// misinstruct twice). Routing a class through `declare_global` was
    /// previously unnecessary because reopening was legal and this map's
    /// only consumer was the redeclaration check; now that classes are
    /// closed (PDR-0001), an `import … as Name` and a `class Name` in
    /// the same unit must collide, so `class_decl.rs` inserts directly into
    /// this map once its own `class.already_defined` check has passed —
    /// which is also what lets `declare_global` (unmodified) catch the
    /// reverse ordering, `class Name` then `import … as Name`, as an
    /// ordinary `binding.redeclared`. U-BINDINGS §12C's stub-completion
    /// carve-out stands: a same-module class (re)declaration bypasses this
    /// map's *redeclaration check* entirely (`class_decl.rs` never calls
    /// `declare_global`), it only ever gains an entry.
    global_bindings: std::collections::HashMap<Symbol, bool>,
    /// All top-level names declared anywhere in this compilation unit.
    /// Pre-scanned so implicit-self fallback cannot steal a forward global
    /// reference; kept separate from `global_bindings` redeclaration state.
    known_globals: std::collections::HashSet<Symbol>,
    /// `import … as Name` bindings declared so far in this compilation
    /// unit, keyed by the bound name, carrying the `import` statement's own
    /// span (U-CLASSCLOSE §8). Consulted by `class_decl.rs`'s redefinition
    /// check so `import "m" as Point` then `class Point` reports
    /// `class.already_defined` — pointing at the import — rather than
    /// silently succeeding (the reverse ordering needs no such map: it is
    /// caught by [`Self::declare_global`] via `global_bindings` once the
    /// class has registered there).
    ///
    /// Scoped to this compilation unit only, like [`Self::global_bindings`]
    /// itself — an import from an *earlier* REPL cell colliding with a class
    /// in a later one is not checked here (U-CLASSCLOSE's own scope is one
    /// compile unit; cross-cell import/class collision is not one of its
    /// required fixtures).
    import_bindings: std::collections::HashMap<Symbol, SourceRange>,
    /// Immutable source facts by lexical scope. Kept separate from binding
    /// resolution: it can never change generated bytecode or runtime state.
    const_fact_scopes: Vec<HashMap<String, boundedness::SourceFacts>>,
    /// The class identity [`ClassKey`] currently being compiled, if any (ADR-0011, U-CLASSNS).
    current_class: Option<ClassKey>,
    /// Whether the current method/scope context is static (metaclass-side) (ADR-0017).
    is_static_context: bool,
    /// The stack of enclosing `for`-loop contexts, innermost last (ADR-0035 §3,
    /// U-ITER specification §4). A `for` body pushes a [`LoopContext`] while it
    /// compiles; `break`/`continue` resolve to the top frame, and a
    /// `break`/`continue` with this stack empty is a compile error (C-ITER-7).
    loop_contexts: Vec<LoopContext>,
    /// Non-zero while compiling a sacred call's **deopt-fallback** copy of its
    /// arms (`inliner.rs`), which suppresses inlining inside that copy.
    ///
    /// Every sacred call emits its arms twice: inlined for the `GuardBool` fast
    /// path, and again as block literals for the fallback that runs when the
    /// receiver is not a `Bool`. Without this flag the fallback copy inlines its
    /// own nested conditionals — each of which emits *its* arms twice — so code
    /// size doubles per nesting level: **2^depth from depth-linear source**. A
    /// 14-deep conditional (`String.codePointAt`) cost ~200 ms to compile alone
    /// and put a fixed 175 ms on every process start
    /// ([perf-log F13](../../../docs/forge/perf-log/findings.md)).
    ///
    /// Suppressing the inliner here is behavior-preserving: the fallback is the
    /// cold path, reached only via a full message send to a non-`Bool` receiver,
    /// and a non-inlined conditional is the same program — the inliner is a
    /// guarded optimization over the `bool_if_true`/`bool_and`/… primitives, not
    /// a semantic. It costs the deopt path its inner fast paths, which is a poor
    /// trade only if that path is hot; it is a message send into a user-defined
    /// `ifTrue`, so it is not.
    deopt_fallback_depth: usize,
    /// Whether the member currently being compiled is a constructor initializer.
    ///
    /// Gates `const`-field writes (ADR-0064 §3, L-3) — a write to a `const`
    /// field is legal only while this is `true`. Set/cleared around all
    /// initializer compilation in `class_decl.rs`; no
    /// flow analysis is performed, so a `const` field write anywhere else in
    /// a constructor's own nested blocks still counts (no attempt is made to
    /// track "inside a block passed to something else").
    in_constructor: bool,
    /// True only while lowering a compiler-generated method that must send an
    /// internal runtime selector. This marker is never sourced from user AST.
    compiler_internal: bool,
    /// Monotonic counter minting unique names for compiler-synthesized
    /// destructuring scratch locals (`$destructure…`, ruling L-9).
    ///
    /// A nested pattern (`let ((a, b), c) = …`) claims more than one scratch
    /// local within the *same* lexical scope — the `Pattern::Tuple`/
    /// `Pattern::List` arm in `patterns.rs` re-enters itself while the outer
    /// scratch is still live — so a fixed name per statement would collide
    /// under [`Self::add_local`]'s same-scope redeclaration check. Each call
    /// to a scratch-naming helper draws the next value and never reuses one.
    scratch_counter: u32,
    /// Index into [`ModuleObject::sources`](crate::heap::ModuleObject) of the
    /// text being compiled, stamped into every [`Chunk`](crate::chunk::Chunk)
    /// this compiler finalizes so diagnostics resolve a span against the source
    /// it came from (U-REPL §D2).
    source_id: u32,
    /// Whether this compilation unit is a whole file or a single REPL cell.
    pub(crate) unit_kind: UnitKind,
}

impl<'vm> Compiler<'vm> {
    /// Creates a compiler targeting `module`, whose chunks carry `source_id`
    /// as their [`Chunk::source_id`](crate::chunk::Chunk::source_id).
    pub(crate) fn new(vm: &'vm mut VM, module: ObjRef, source_id: u32, unit_kind: UnitKind) -> Self {
        Compiler {
            vm,
            module,
            functions: vec![FunctionState::new(false, false, false, None)],
            global_bindings: std::collections::HashMap::new(),
            known_globals: std::collections::HashSet::new(),
            import_bindings: std::collections::HashMap::new(),
            const_fact_scopes: vec![HashMap::new()],
            current_class: None,
            is_static_context: false,
            loop_contexts: Vec::new(),
            deopt_fallback_depth: 0,
            in_constructor: false,
            compiler_internal: false,
            scratch_counter: 0,
            source_id,
            unit_kind,
        }
    }

    /// The bootstrap core module is the sole source module permitted to spell
    /// implementation selectors and fields. Compare handles so a user module
    /// named `core` cannot acquire this authority.
    pub(crate) fn compiling_privileged_core(&self) -> bool {
        self.vm
            .interner
            .find(crate::heap::CORE_MODULE_NAME)
            .and_then(|name| self.vm.modules.get(&name).copied())
            == Some(self.module)
    }

    /// Whether compilation is currently inside a sacred call's deopt-fallback
    /// copy, where the inliner is suppressed (see
    /// [`Self::deopt_fallback_depth`]).
    pub(crate) fn in_deopt_fallback(&self) -> bool {
        self.deopt_fallback_depth > 0
    }

    /// The [`ClassKey`] for `name` in the module currently being compiled.
    pub(crate) fn class_key(&self, name: Symbol) -> ClassKey {
        ClassKey { module: self.module, name }
    }

    /// The source text of the unit currently being compiled.
    ///
    /// Resolved through [`ModuleObject::sources`](crate::heap::ModuleObject::sources)
    /// at [`Self::source_id`] — the same text [`compile_closure_as`](crate::VM::compile_closure_as)
    /// stamped there before constructing this compiler — so a diagnostic
    /// raised mid-compile can resolve a byte offset to a `line:col` pair
    /// (`diagnostics::line_col`, U-CLASSCLOSE §3 option A) without threading
    /// a second copy of the source string through every call site.
    pub(crate) fn source_text(&self) -> std::sync::Arc<String> {
        self.vm.heap.module(self.module).sources[self.source_id as usize].clone()
    }

    /// Resolve a superclass name to the [`ClassKey`] that actually owns it.
    ///
    /// Mirrors the runtime global-resolution order (`vm/dispatch.rs`,
    /// `Bytecode::GetGlobal`): the compiling module first, then the core
    /// module. A superclass is always a bare identifier
    /// (`phalcom_ast::ast::SuperclassRef`), so those two are the complete
    /// resolution space.
    ///
    /// **Not** for the class being declared — see
    /// `docs/forge/units/U-CLASSNS/implementation-spec.md` §4.1.
    pub(crate) fn resolve_superclass_key(&self, name: Symbol) -> Option<ClassKey> {
        let own_key = self.class_key(name);
        if self.vm.field_layouts.contains_key(&own_key) || self.vm.classes.contains_key(&own_key) {
            return Some(own_key);
        }
        if let Some(core_module_sym) = self.vm.interner.find(crate::heap::CORE_MODULE_NAME) {
            if let Some(core_module) = self.vm.modules.get(&core_module_sym).copied() {
                let core_key = ClassKey { module: core_module, name };
                if self.vm.field_layouts.contains_key(&core_key) || self.vm.classes.contains_key(&core_key) {
                    return Some(core_key);
                }
            }
        }
        None
    }

    /// Runs `f` with the inliner suppressed, for compiling a sacred call's
    /// deopt-fallback copy of its arms. Nests correctly: the counter is
    /// restored on the way out, so an outer fallback stays suppressed.
    pub(crate) fn with_deopt_fallback<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, CompilerError>) -> Result<T, CompilerError> {
        self.deopt_fallback_depth += 1;
        let result = f(self);
        self.deopt_fallback_depth -= 1;
        result
    }

    /// Compiles a block or method body into a heap-allocated closure.
    ///
    /// Slot 0 holds the receiver (`self` for a method, the block object
    /// otherwise). A body whose last statement leaves a value on the operand
    /// stack ([`Statement::Expr`], [`Statement::Let`], or [`Statement::Return`])
    /// returns that value. A **value-less** body — an empty body, or one ending
    /// in a [`Statement::Class`] or any other statement that leaves nothing
    /// behind — yields `None`: a [`Bytecode::Nil`] placeholder is pushed before
    /// the fallback [`Bytecode::Return`] so that falling off the end surfaces
    /// absence (U6-plan.md §4) rather than the receiver sitting in slot 0. This
    /// mirrors [`compile_inline_block_body`] so the inlined fast path and the
    /// non-inlined fallback agree.
    ///
    /// [`compile_inline_block_body`]: crate::compiler::inliner
    ///
    /// # Errors
    ///
    /// Propagates any error compiling the body's statements.
    fn compile_block(
        &mut self,
        statements: Vec<Statement>,
        name_sym: Symbol,
        params: ClosureParameters,
        is_method: bool,
        is_constructor: bool,
        constructor_name: Option<String>,
    ) -> Result<ObjRef, CompilerError> {
        // Intern parameter and receiver names before pushing the function state.
        let mut param_names = params.fixed.clone();
        if let Some(rest) = &params.positional_rest {
            param_names.push(rest.clone());
        }
        let mut param_symbols = Vec::with_capacity(param_names.len());
        for param_name in &param_names {
            param_symbols.push(self.vm.interner.intern(param_name));
        }
        let self_sym = self.vm.interner.intern("self");
        let dummy_sym = self.vm.interner.intern("<block-receiver>");

        // Push a fresh function-compilation state for this body.
        let has_self = is_method || self.functions.last().is_some_and(|function| function.has_self);
        self.functions.push(FunctionState::new(is_constructor, !is_method, has_self, constructor_name));
        self.begin_scope();

        if is_method {
            // Slot 0 holds the receiver `self`. Never reachable through
            // `Expr::Var`/`Expr::Assignment` (`self` is `Expr::SelfVar`, a
            // distinct AST node — see `ast.rs`), so `is_mutable` is purely
            // documentary here; the constructor's own `SetLocal(0)` below
            // writes this slot directly, bypassing the immutability check
            // entirely (L-6).
            self.add_local(self_sym, false)
                .expect("receiver slot is the first local in a fresh function state");
        } else {
            // Slot 0 holds the block object itself (blocks reach `self` via an
            // upvalue, functions.md §2), so we reserve it with a dummy local.
            self.add_local(dummy_sym, false)
                .expect("block-receiver slot is the first local in a fresh function state");
        }

        for param_sym in param_symbols {
            // L-6: every implicit binding (parameters, block parameters) is
            // immutable. To vary a parameter, declare a local from it.
            self.add_local(param_sym, false)?;
        }

        let len = statements.len();
        let mut last_is_return = false;
        // Track whether the last statement leaves a value on the operand stack.
        // `Expr`, `Let`, and `Return` do; an empty body or a trailing `Class`
        // (or any other value-less statement) does not. This mirrors
        // `compile_inline_block_body` so the fast (inlined) and fallback paths
        // agree on the fall-off-end result.
        let mut leaves_value = false;
        for (i, statement) in statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
                leaves_value = matches!(statement, Statement::Expr { .. } | Statement::Let(_) | Statement::Return(_));
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }

        let max_slots = self.functions.last().unwrap().max_slots;
        self.end_scope(EmptySourceRange);

        if !last_is_return {
            if self.functions.last().unwrap().is_constructor {
                self.emit(Bytecode::GetLocal(0), EmptySourceRange);
            } else if !leaves_value {
                self.emit(Bytecode::Nil, EmptySourceRange);
            }
            self.emit(Bytecode::Return, EmptySourceRange);
        }

        let mut func = self.functions.pop().unwrap();
        func.chunk.fuse_superinstructions();
        func.chunk.source_id = self.source_id;
        let callable = Rc::new(Callable {
            chunk: func.chunk,
            max_slots,
            num_upvalues: func.upvalues.len(),
            upvalues: func.upvalues,
            arity: params.fixed.len(),
            parameter_shape: crate::parameters::ParameterShape::closure(params.fixed.len(), params.positional_rest.is_some()),
            name_sym,
            local_names: func.local_names,
        });

        let closure = self.vm.heap.alloc(Object::Closure(Box::new(ClosureObject {
            callable,
            module: self.module,
            upvalues: Vec::new(),
            lexical_class: None,
        })));
        Ok(closure)
    }

    pub(crate) fn compile(mut self, program: Program) -> PhResult<ObjRef> {
        self.predeclare_known_globals(&program);
        let len = program.statements.len();
        let mut last_is_return = false;
        let mut leaves_value = false;
        for (i, statement) in program.statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            // Check if last statement is a return
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
                leaves_value = matches!(statement, Statement::Expr { .. });
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }

        if !last_is_return {
            if !leaves_value {
                self.emit(Bytecode::Nil, EmptySourceRange);
            }
            self.emit(Bytecode::Return, EmptySourceRange);
        }

        let name_sym = self.vm.heap.module(self.module).name_sym;
        let mut func = self.functions.pop().unwrap();
        func.chunk.fuse_superinstructions();
        func.chunk.source_id = self.source_id;
        let callable = Rc::new(Callable {
            chunk: func.chunk,
            max_slots: func.max_slots,
            num_upvalues: 0,
            upvalues: Vec::new(),
            arity: 0,
            parameter_shape: crate::parameters::ParameterShape::closure(0, false),
            name_sym,
            local_names: func.local_names,
        });

        let closure = self.vm.heap.alloc(Object::Closure(Box::new(ClosureObject {
            callable,
            module: self.module,
            upvalues: Vec::new(),
            lexical_class: None,
        })));

        let module_obj = self.vm.heap.module_mut(self.module);
        module_obj.merge_global_bindings(&self.global_bindings);

        Ok(closure)
    }

    fn predeclare_known_globals(&mut self, program: &Program) {
        fn collect_pattern(pattern: &phalcom_ast::ast::Pattern, out: &mut Vec<String>) {
            match pattern {
                phalcom_ast::ast::Pattern::Name { name, .. } => out.push(name.clone()),
                phalcom_ast::ast::Pattern::Tuple { elements, .. } => {
                    for element in elements {
                        collect_pattern(element, out);
                    }
                }
                phalcom_ast::ast::Pattern::List { elements, rest, .. } => {
                    for element in elements {
                        collect_pattern(element, out);
                    }
                    if let Some(rest) = rest {
                        collect_pattern(rest, out);
                    }
                }
            }
        }

        let mut names = Vec::new();
        for statement in &program.statements {
            match statement {
                Statement::Class(class) => names.push(class.name.clone()),
                Statement::Import(import) => names.push(import.binding.clone()),
                Statement::Let(binding) => collect_pattern(&binding.pattern, &mut names),
                _ => {}
            }
        }
        for name in names {
            let symbol = self.vm.interner.intern(&name);
            self.known_globals.insert(symbol);
        }
    }

    pub(crate) fn compile_statement_with_pop_control(&mut self, statement: Statement, emit_pop: bool) -> Result<(), CompilerError> {
        match statement {
            Statement::Expr { expr, range } => {
                // A bare-statement expression whose value is about to be
                // popped is an "unused" position (U-CORE-2): pass that
                // through so a one-armed sacred conditional can skip its
                // `Some`-wrap allocation — see `compile_expr_want`.
                self.compile_expr_want(expr, !emit_pop)?;
                if emit_pop {
                    // println!("[Compiler] Emitting Pop");
                    self.emit(Bytecode::Pop, range);
                }
            }
            Statement::Let(binding) => {
                let range = binding.range;
                let facts = binding.value.as_ref().map(|expr| self.const_facts_for(expr));
                // `let` is mutable and may be left uninitialized; `const` is
                // immutable and *requires* an initializer (ADR-0064). A
                // destructuring pattern (`Pattern::Tuple`/`Pattern::List`)
                // always requires an initializer regardless of `let`/`const`
                // (U14, open-questions.md Q7, ADR-0046 §2) — there is nothing
                // to unpack from an absent value.
                let mutable = matches!(binding.kind, BindingKind::Let);

                match binding.value {
                    Some(expr) => self.compile_expr(expr)?,
                    None => {
                        let Pattern::Name { name, .. } = &binding.pattern else {
                            return Err(CompilerError::DestructuringWithoutInitializer(binding.pattern.range()));
                        };
                        if !mutable {
                            // `const x` with no initializer is a compile error.
                            return Err(CompilerError::ConstWithoutInitializer(name.clone()));
                        }
                        // `let x` with no initializer is backed by the private
                        // `nil` sentinel; the VM surfaces every read of that
                        // slot as the surface `None` value (ADR-0007/ADR-0010).
                        self.emit(Bytecode::Nil, range);
                    }
                }

                // The initializer's single evaluated value is sitting on top
                // of the operand stack; bind it — positionally, through the
                // sub-patterns' `at(_)` reads, for a destructuring pattern —
                // as a local or a module global depending on where this
                // binding appears (ADR-0064's local-vs-global rule, threaded
                // through every leaf of the pattern). Also rejects a
                // same-scope redeclaration of either kind (L-3/L-5).
                let as_global = self.functions.last().unwrap().scope_depth == 0;
                self.compile_pattern_bind_top_of_stack(&binding.pattern, mutable, as_global)?;
                if !mutable {
                    if let (Pattern::Name { name, .. }, Some(facts)) = (&binding.pattern, facts) {
                        self.const_fact_scopes.last_mut().unwrap().insert(name.clone(), facts);
                    }
                }
            }
            Statement::Import(import_stmt) => {
                self.compile_import(import_stmt)?;
            }
            Statement::Return(return_stmt) => {
                let range = return_stmt.range;
                if self.functions.last().unwrap().is_constructor {
                    if return_stmt.value.is_some() {
                        return Err(CompilerError::ReturnValueFromInitializer);
                    }
                    self.emit(Bytecode::GetLocal(0), range);
                } else {
                    if let Some(expr) = return_stmt.value {
                        self.compile_expr(expr)?;
                    } else {
                        self.emit(Bytecode::Nil, range);
                    }
                }
                // A `return` inside a block literal is a *non-local* return: it
                // unwinds to the block's enclosing method activation, not just
                // the block's own frame (blocks.md §5, ADR-0013). A method or
                // constructor body keeps the ordinary single-frame `Return`.
                // Constructors are never block bodies (`is_constructor` always
                // accompanies `is_method`), so `is_block` is the sole gate.
                if self.functions.last().unwrap().is_block {
                    self.emit(Bytecode::ReturnNonLocal, range);
                } else {
                    self.emit(Bytecode::Return, range);
                }
            }
            Statement::Class(class_def) => {
                self.compile_class(class_def)?;
            }
            Statement::For(for_stmt) => {
                // A `for` is a statement consumed for effect (U-ITER spec
                // §1.2): it leaves no value, so `emit_pop` is irrelevant.
                self.compile_for(for_stmt)?;
            }
            Statement::Break { range } => {
                self.compile_break(range)?;
            }
            Statement::Continue { range } => {
                self.compile_continue(range)?;
            }
            Statement::Throw { expr, range } => {
                // The compile-time half of error-handling.md §1: a
                // syntactically-literal non-`Error` operand is rejected before
                // lowering (the runtime half — a genuine `doesNotUnderstand`
                // miss on `raise()` — already exists via U-CORE-6's `Error`
                // primitive, and covers everything this static check can't
                // prove, e.g. `throw someVariable`).
                if is_non_error_literal(&expr) {
                    return Err(CompilerError::ThrowNonError(range));
                }
                // `throw expr` is surface sugar for `expr.raise()`
                // (ADR-0031 §1) — reuse the ordinary `MethodCall` lowering
                // rather than hand-rolling an `Invoke`, so it gets the same
                // selector encoding / IC treatment as a written `.raise()`
                // send. `raise()` always unwinds (never returns normally), so
                // its "value" never materialises; `emit_pop` is honored purely
                // to keep the stack discipline uniform with every other
                // statement, not because a value is meaningfully produced.
                let call = Expr::MethodCall(Box::new(MethodCallExpr {
                    object: expr,
                    method: "raise".to_string(),
                    args: Vec::new(),
                    range,
                }));
                self.compile_expr_want(call, !emit_pop)?;
                if emit_pop {
                    self.emit(Bytecode::Pop, range);
                }
            }
        }
        Ok(())
    }

    fn const_fact_env(&self) -> HashMap<String, boundedness::SourceFacts> {
        let mut env = HashMap::new();
        for scope in &self.const_fact_scopes {
            env.extend(scope.iter().map(|(name, facts)| (name.clone(), *facts)));
        }
        env
    }

    fn const_facts_for(&self, expr: &Expr) -> boundedness::SourceFacts {
        boundedness::infer_source_facts(expr, &self.const_fact_env())
    }

    pub(crate) fn check_bounded_method_call(&self, call: &MethodCallExpr) -> Result<(), CompilerError> {
        boundedness::check_method_call(call, &self.const_fact_env())
    }

    pub(crate) fn check_bounded_property(&self, property: &str, receiver: &Expr, range: SourceRange) -> Result<(), CompilerError> {
        boundedness::check_property(property, receiver, range, &self.const_fact_env())
    }

    /// Applies E.3's conservative full-exhaustion rule to positional `*`.
    pub(crate) fn check_bounded_expansion(&self, source: &Expr, range: SourceRange) -> Result<(), CompilerError> {
        boundedness::require_exhaustible(source, range, &self.const_fact_env())
    }

    /// Lowers `import "path" as Name` (U15, DEC-U15 A+A) to a
    /// [`Bytecode::Import`] carrying the raw path, immediately followed by
    /// the same [`Bytecode::DefineGlobal`] a module-level `const Name = …`
    /// would emit — the `as Name` binding is an ordinary immutable global
    /// ([ADR-0064](../../../docs/adr/accepted/0064-let-const-bindings-and-field-mutability.md)),
    /// not a new binding kind.
    ///
    /// Restricted to a compilation unit's own top level (`self.functions`
    /// has exactly the module body's [`FunctionState`] and its scope depth
    /// is 0) — mirroring how `class` is a program-shape construct rather
    /// than an ordinary statement. `import` inside a method/block/class body
    /// is a [`CompilerError::ImportNotAtTopLevel`] compile error.
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::ImportNotAtTopLevel`] if `import_stmt`
    /// appears anywhere but the module's own top level.
    fn compile_import(&mut self, import_stmt: phalcom_ast::ast::ImportStatement) -> Result<(), CompilerError> {
        if self.functions.len() > 1 || self.functions.last().unwrap().scope_depth > 0 {
            return Err(CompilerError::ImportNotAtTopLevel);
        }

        let range = import_stmt.range;
        let path_sym = self.vm.interner.intern(&import_stmt.path);
        let path_idx = self.add_constant(Value::Symbol(path_sym));
        self.emit(Bytecode::Import(path_idx), range);

        // `as Name` is always an immutable module-level global — the whole
        // binding is `const`-shaped (ADR-0064), never a local (import is
        // top-level-only, see the guard above).
        let name_sym = self.vm.interner.intern(&import_stmt.binding);
        self.declare_global(name_sym, false)?;
        // Recorded for `class_decl.rs`'s redefinition check (U-CLASSCLOSE
        // §8): `import "m" as Point` then `class Point` must report
        // `class.already_defined` pointing at this import, not silently
        // succeed. `declare_global` above already rejects a second `import`
        // of the same name, so this insert never overwrites an earlier span.
        self.import_bindings.insert(name_sym, range);
        let name_idx = self.add_constant(Value::Symbol(name_sym));
        self.emit(Bytecode::DefineGlobal(name_idx), range);
        Ok(())
    }
}

/// Reports whether `expr` is a syntactically detectable non-`Error` literal —
/// a `Number`/`String`/`Boolean` literal, none of which can ever answer
/// `raise()` (only `Error` and its subclasses do). Guards `throw`'s
/// compile-time half of error-handling.md §1 ("`throw "oops"` is a compile
/// error"); a non-literal operand (a variable, a list/map-literal send, a
/// user `Error` construction) cannot be statically classified and defers to
/// the runtime `doesNotUnderstand` miss on `raise()` instead — deliberately no
/// flow typing (U-ERR plan §2.2).
fn is_non_error_literal(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Int { .. } | Expr::Float { .. } | Expr::String { .. } | Expr::Boolean { .. } | Expr::Symbol(_)
    )
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_primitive_class() {
    //     // Number
    //     let result = run_test("return 123.class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Number"),
    //         _ => panic!("Expected Value::Class for 123.class"),
    //     }
    //     // String
    //     let result = run_test("return \"abc\".class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "String"),
    //         _ => panic!("Expected Value::Class for string.class"),
    //     }
    //     // Boolean
    //     let result = run_test("return true.class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Bool"),
    //         _ => panic!("Expected Value::Class for true.class"),
    //     }
    //     // Nil
    //     let result = run_test("return nil.class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Nil"),
    //         _ => panic!("Expected Value::Class for nil.class"),
    //     }
    // }

    // #[test]
    // fn test_primitive_superclass() {
    //     // Number superclass
    //     let result = run_test("return 123.class.superclass;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
    //         _ => panic!("Expected Value::Class for 123.class.superclass"),
    //     }
    //     // String superclass
    //     let result = run_test("return \"abc\".class.superclass;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
    //         _ => panic!("Expected Value::Class for \"abc\".class.superclass"),
    //     }
    // }

    // #[test]
    // fn test_primitive_name() {
    //     // Number name
    //     let result = run_test("return 123.name;").unwrap();
    //     println!("{:?}", result);

    //     match result {
    //         Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "Number"),
    //         _ => panic!("Expected Value::String for 123.name"),
    //     }
    //     // String name
    //     let result = run_test("\"abc\".name;").unwrap();
    //     match result {
    //         Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "String"),
    //         _ => panic!("Expected Value::String for string.name"),
    //     }
    // }

    // #[test]
    // fn test_class_identity() {
    //     // .class.class should be Class
    //     let result = run_test("return 123.class.class;").unwrap();
    //     println!("{:?}", result);

    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Class"),
    //         _ => panic!("Expected Value::Class for 123.class.class"),
    //     }
    //     // .class.superclass should be Object
    //     let result = run_test("return 123.class.superclass;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
    //         _ => panic!("Expected Value::Class for 123.class.superclass"),
    //     }
    // }
    // use super::*;

    // fn run_test(source: &str) -> Result<Value, PhError> {
    //     let mut vm = VM::new();
    //     let closure = compile(&mut vm, source)?;
    //     let module = vm.create_module_from_stdin();
    //     vm.run_module(module, closure)
    // }

    // #[test]
    // fn test_compile_number() {
    //     let result = run_test("123;").unwrap();
    //     assert_eq!(result, Value::Number(123.0));
    // }

    // #[test]
    // fn test_compile_string() {
    //     let result = run_test("\"hello\";").unwrap();
    //     assert_eq!(result, Value::string_from("hello".to_string()));
    // }

    // #[test]
    // fn test_compile_boolean() {
    //     let result = run_test("return true;").unwrap();
    //     assert_eq!(result, Value::Bool(true));
    //     let result = run_test("return false;").unwrap();
    //     assert_eq!(result, Value::Bool(false));
    // }

    // #[test]
    // fn test_compile_nil() {
    //     let result = run_test("return nil;").unwrap();
    //     assert_eq!(result, Value::Nil);
    // }

    // #[test]
    // fn test_compile_binary_expr() {
    //     let result = run_test("return 1 + 2;").unwrap();
    //     assert_eq!(result, Value::Number(3.0));
    // }

    // #[test]
    // fn test_compile_binary_mult() {
    //     let result = run_test("return 4 * 3;").unwrap();
    //     assert_eq!(result, Value::Number(12.0));
    // }

    // #[test]
    // fn test_compile_unary_expr() {
    //     let result = run_test("-10;").unwrap();
    //     assert_eq!(result, Value::Number(-10.0));
    // }

    // #[test]
    // fn test_compile_global_let() {
    //     let result = run_test("let a = 10; return a;").unwrap();
    //     assert_eq!(result, Value::Number(10.0));
    // }

    // #[test]
    // fn test_compile_global_assignment() {
    //     let result = run_test("let a = 10; a += 20; return a;").unwrap();
    //     assert_eq!(result, Value::Number(30.0));
    // }

    // #[test]
    // fn test_complex_global_assignment() {
    //     let source = "
    //         let a = 5;
    //         let b = 10;
    //         a += b; // a should be 15
    //         return a;           // return 15
    //     ";
    //     let result = run_test(source).unwrap();
    //     assert_eq!(result, Value::Number(15.0));
    // }

    // #[test]
    // fn test_compile_precedence() {
    //     let result = run_test("return 1 + 2 * 3;").unwrap();
    //     assert_eq!(result, Value::Number(7.0));
    // }

    // #[test]
    // fn test_compile_return() {
    //     let result = run_test("return 15; 20;").unwrap();
    //     assert_eq!(result, Value::Number(15.0));

    //     let result = run_test("return;").unwrap();
    //     assert_eq!(result, Value::Nil);
    // }

    // #[test]
    // fn test_compile_method_expr() {
    //     let result = run_test("return 123.class.name.class;").unwrap();
    //     println!("{:?}", result);
    // }

    // #[test]
    // fn test_compile_class_add_call() {
    //     let result = run_test("return 123.class + true.class;").unwrap();
    //     println!("{:?}", result);
    // }
}
