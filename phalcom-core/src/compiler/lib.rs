//! The AST-to-bytecode compiler.
//!
//! Lowers a parsed [`Program`] into a [`ClosureObject`] whose [`Chunk`] the VM
//! executes. String, method and superclass constants are materialized onto the
//! [`Heap`](crate::heap::Heap) as the compiler emits them — the compiler already
//! holds `&mut VM` and therefore the heap — and are referenced from the constant
//! pool by `Copy` [`Value::Obj`] handles
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md),
//! [ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)).

use crate::bytecode::Bytecode;
use crate::callable::{Callable, UpvalueDescriptor};
use crate::chunk::Chunk;
use crate::compiler::inliner;
use crate::closure::ClosureObject;
use crate::error::PhResult;
use crate::heap::{ObjRef, Object};
use crate::interner::Symbol;
use crate::method::{encode_selector, make_signature, MethodKind, MethodObject, SignatureKind};
use crate::value::Value;
use crate::vm::VM;
use phalcom_ast::ast::{BinaryOp, BlockExpr, ClassMember, Expr, Program, Statement, UnaryOp};
use phalcom_ast::error::SyntaxError;
use phalcom_common::range::{EmptySourceRange, SourceRange};
use thiserror::Error;
use tracing::debug;

/// An error raised while lowering the AST to bytecode.
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    /// A catch-all for otherwise-unclassified compilation failures.
    #[error("Unknown error during compilation.")]
    Unknown,

    /// A reference to a variable the compiler cannot resolve.
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),

    /// An assignment whose left-hand side is not an assignable target.
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,

    /// A syntax error surfaced from the front-end parser.
    #[error(transparent)]
    Parse(#[from] SyntaxError),

    /// A free-form compiler diagnostic.
    #[error("{0}")]
    Message(String),
}

// impl From<CompilerError> for PhError {
//     fn from(err: CompilerError) -> Self {
//         PhError::Compile(err)
//     }
// }
//
// impl From<SyntaxError> for PhError {
//     fn from(err: SyntaxError) -> Self {
//         PhError::Compile(err.into())
//     }
// }

/// A lexically-scoped local variable inside a single function/block body.
struct Local {
    /// The interned name the local was declared with.
    name: Symbol,
    /// The nesting depth of the scope that declared it.
    depth: usize,
    /// Whether a nested block captured this local as an upvalue, which forces
    /// the enclosing frame to close (heap-promote) it on scope exit
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    is_captured: bool,
}

/// The mutable per-function compilation state for one closure body.
///
/// Every method, getter, setter, block literal and the top-level module gets
/// its own `FunctionState`. The [`Compiler`] keeps them on a **stack**
/// ([`Compiler::functions`]) so upvalue resolution can walk from the innermost
/// block outward through its enclosing functions using plain indices — with no
/// aliasing `&mut` references and no raw parent pointers
/// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
pub(crate) struct FunctionState {
    /// The bytecode chunk being emitted for this body.
    pub(crate) chunk: Chunk,
    /// The live local variables, innermost last.
    locals: Vec<Local>,
    /// The current lexical scope nesting depth.
    scope_depth: usize,
    /// The number of live locals (mirrors `locals.len()`).
    num_locals: usize,
    /// The peak number of locals seen, used as the callable's slot count.
    max_slots: usize,
    /// The upvalue capture descriptors resolved for this body.
    upvalues: Vec<UpvalueDescriptor>,
}

impl FunctionState {
    /// Creates an empty function-compilation state.
    fn new() -> Self {
        FunctionState {
            chunk: Chunk::default(),
            locals: Vec::new(),
            scope_depth: 0,
            num_locals: 0,
            max_slots: 0,
            upvalues: Vec::new(),
        }
    }
}

pub(crate) struct Compiler<'vm> {
    pub(crate) vm: &'vm mut VM,
    module: ObjRef,
    /// The stack of function-compilation states, innermost body last. A block
    /// literal pushes a new state, compiles into it, and pops it; upvalue
    /// resolution indexes into this stack rather than following raw pointers.
    pub(crate) functions: Vec<FunctionState>,
}

impl<'vm> Compiler<'vm> {
    pub(crate) fn new(vm: &'vm mut VM, module: ObjRef) -> Self {
        Compiler {
            vm,
            module,
            functions: vec![FunctionState::new()],
        }
    }

    /// Emits `opcode` into the current function's chunk.
    pub(crate) fn emit(&mut self, opcode: Bytecode, range: SourceRange) {
        self.functions.last_mut().unwrap().chunk.add_instruction(opcode, range);
    }

    /// Adds `value` to the current function's constant pool, returning its index.
    pub(crate) fn add_constant(&mut self, value: Value) -> u16 {
        self.functions.last_mut().unwrap().chunk.add_constant(value)
    }

    /// Emits a read of the receiver `self` for the current body.
    ///
    /// In a method/module body `self` is the frame receiver, emitted as
    /// [`Bytecode::GetSelf`]. Inside a block, `self` is not the block object —
    /// it is the enclosing method's receiver, captured as an ordinary upvalue
    /// (functions.md §2), so it resolves through [`Self::resolve_upvalue`] and is
    /// emitted as [`Bytecode::GetUpvalue`].
    fn emit_self(&mut self, range: SourceRange) {
        if self.functions.len() > 1 {
            let self_sym = self.vm.interner.intern("self");
            if let Some(upvalue) = self.resolve_upvalue(self_sym) {
                self.emit(Bytecode::GetUpvalue(upvalue as u16), range);
                return;
            }
        }
        self.emit(Bytecode::GetSelf, range);
    }

    pub(crate) fn begin_scope(&mut self) {
        self.functions.last_mut().unwrap().scope_depth += 1;
    }

    /// Closes the innermost scope, promoting captured locals to heap cells.
    ///
    /// The function-body scope is cleaned up by [`Bytecode::Return`], which
    /// truncates the whole frame window (see the VM's `Return`/`run_until`
    /// handling). We therefore never `Pop` locals here — doing so would discard
    /// the return value sitting on top of them (the calculator-getter bug). We
    /// only emit [`Bytecode::CloseUpvalue`] for captured locals so their heap
    /// cells are promoted (`Open` -> `Closed`) before the slot is reclaimed
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)); the VM's
    /// `Return` closes them again idempotently for the explicit-return path.
    pub(crate) fn end_scope(&mut self, range: SourceRange) {
        let func = self.functions.last_mut().unwrap();
        func.scope_depth -= 1;
        let scope_depth = func.scope_depth;
        let mut to_close = Vec::new();
        while func.num_locals > 0 && func.locals[func.num_locals - 1].depth > scope_depth {
            func.num_locals -= 1;
            let local = func.locals.pop().unwrap();
            if local.is_captured {
                to_close.push(func.num_locals as u16);
            }
        }
        for slot in to_close {
            self.emit(Bytecode::CloseUpvalue(slot), range);
        }
    }

    fn add_local(&mut self, name: Symbol) {
        let func = self.functions.last_mut().unwrap();
        debug!("[Compiler] Adding local at depth {}", func.scope_depth);
        func.locals.push(Local { name, depth: func.scope_depth, is_captured: false });
        func.num_locals += 1;
        if func.num_locals > func.max_slots {
            func.max_slots = func.num_locals;
        }
    }

    /// Resolves `name` as a local in the current function, returning its slot.
    fn resolve_local(&self, name: Symbol) -> Option<usize> {
        self.resolve_local_in(self.functions.len() - 1, name)
    }

    /// Resolves `name` as a local in the function at `func_idx`, returning its
    /// slot index (which doubles as the runtime stack slot within the frame).
    fn resolve_local_in(&self, func_idx: usize, name: Symbol) -> Option<usize> {
        let func = &self.functions[func_idx];
        (0..func.num_locals).rev().find(|&i| func.locals[i].name == name)
    }

    /// Resolves `name` as an upvalue captured by the current function.
    ///
    /// Walks the enclosing function-compilation states (via [`Self::functions`]
    /// indices — no aliasing borrows) marking the captured local so the
    /// enclosing frame closes it on scope exit
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    fn resolve_upvalue(&mut self, name: Symbol) -> Option<usize> {
        self.resolve_upvalue_in(self.functions.len() - 1, name)
    }

    /// Resolves `name` as an upvalue of the function at `func_idx`, recursing
    /// into the enclosing function when the variable is itself an upvalue there.
    fn resolve_upvalue_in(&mut self, func_idx: usize, name: Symbol) -> Option<usize> {
        if func_idx == 0 {
            return None;
        }
        let enclosing = func_idx - 1;

        // 1. Resolve as a local in the enclosing function -> capture it directly.
        if let Some(slot) = self.resolve_local_in(enclosing, name) {
            self.functions[enclosing].locals[slot].is_captured = true;
            return Some(self.add_upvalue(func_idx, slot, true));
        }

        // 2. Otherwise resolve recursively as an upvalue of the enclosing
        //    function and chain through it.
        if let Some(upvalue_idx) = self.resolve_upvalue_in(enclosing, name) {
            return Some(self.add_upvalue(func_idx, upvalue_idx, false));
        }

        None
    }

    /// Records (deduplicating) an upvalue descriptor on the function at
    /// `func_idx`, returning its index in that function's upvalue list.
    fn add_upvalue(&mut self, func_idx: usize, index: usize, is_local: bool) -> usize {
        let upvalues = &mut self.functions[func_idx].upvalues;
        for (i, upval) in upvalues.iter().enumerate() {
            if upval.index == index && upval.is_local == is_local {
                return i;
            }
        }
        upvalues.push(UpvalueDescriptor { is_local, index });
        upvalues.len() - 1
    }

    /// Emits an ordinary `Invoke` send for operator selector `name` at
    /// `arity` (control-flow.md §1; U5-plan.md §4.1). Always builds the
    /// selector through [`encode_selector`] — never a hand-rolled string —
    /// so the compiler and the primitive registrations in `universe.rs`
    /// stay in lockstep (ADR-0012).
    fn emit_operator_send(&mut self, name: &str, arity: u8, range: SourceRange) {
        let labels = vec![None; arity as usize];
        let selector = encode_selector(name, &labels, SignatureKind::Method(arity));
        let selector_sym = self.vm.interner.intern(&selector);
        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
        self.emit(Bytecode::Invoke(arity, selector_idx), range);
    }

    fn compile_block(
        &mut self,
        statements: Vec<Statement>,
        name_sym: Symbol,
        params: Vec<String>,
        is_method: bool,
    ) -> Result<ObjRef, CompilerError> {
        // Intern parameter and receiver names before pushing the function state.
        let mut param_symbols = Vec::with_capacity(params.len());
        for param_name in &params {
            param_symbols.push(self.vm.interner.intern(param_name));
        }
        let self_sym = self.vm.interner.intern("self");
        let dummy_sym = self.vm.interner.intern("<block-receiver>");

        // Push a fresh function-compilation state for this body.
        self.functions.push(FunctionState::new());
        self.begin_scope();

        if is_method {
            // Slot 0 holds the receiver `self`.
            self.add_local(self_sym);
        } else {
            // Slot 0 holds the block object itself (blocks reach `self` via an
            // upvalue, functions.md §2), so we reserve it with a dummy local.
            self.add_local(dummy_sym);
        }

        for param_sym in param_symbols {
            self.add_local(param_sym);
        }

        let len = statements.len();
        let mut last_is_return = false;
        for (i, statement) in statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }

        let max_slots = self.functions.last().unwrap().max_slots;
        self.end_scope(EmptySourceRange);

        if !last_is_return {
            self.emit(Bytecode::Return, EmptySourceRange);
        }

        let func = self.functions.pop().unwrap();
        let callable = Callable {
            chunk: func.chunk,
            max_slots,
            num_upvalues: func.upvalues.len(),
            upvalues: func.upvalues,
            arity: params.len(),
            name_sym,
        };

        let closure = self.vm.heap.alloc(Object::Closure(ClosureObject {
            callable,
            module: self.module,
            upvalues: Vec::new(),
        }));
        Ok(closure)
    }

    pub(crate) fn compile(mut self, program: Program) -> PhResult<ObjRef> {
        let len = program.statements.len();
        let mut last_is_return = false;
        for (i, statement) in program.statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            // Check if last statement is a return
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }

        if !last_is_return {
            self.emit(Bytecode::Return, EmptySourceRange);
        }

        let name_sym = self.vm.heap.module(self.module).name_sym;
        let func = self.functions.pop().unwrap();
        let callable = Callable {
            chunk: func.chunk,
            max_slots: func.max_slots,
            num_upvalues: 0,
            upvalues: Vec::new(),
            arity: 0,
            name_sym,
        };

        let closure = self.vm.heap.alloc(Object::Closure(ClosureObject {
            callable,
            module: self.module,
            upvalues: Vec::new(),
        }));

        Ok(closure)
    }

    pub(crate) fn compile_statement_with_pop_control(&mut self, statement: Statement, emit_pop: bool) -> Result<(), CompilerError> {
        match statement {
            Statement::Expr { expr, range } => {
                self.compile_expr(expr)?;
                if emit_pop {
                    // println!("[Compiler] Emitting Pop");
                    self.emit(Bytecode::Pop, range);
                }
            }
            Statement::Let(binding) => {
                let range = binding.range;

                if let Some(expr) = binding.value {
                    self.compile_expr(expr)?;
                } else {
                    self.emit(Bytecode::Nil, range);
                }

                let name_sym = self.vm.interner.intern(&binding.name);
                if self.functions.last().unwrap().scope_depth > 0 {
                    // Local variable
                    self.add_local(name_sym);
                    let slot = self.functions.last().unwrap().num_locals - 1;
                    self.emit(Bytecode::SetLocal(slot as u16), range);
                } else {
                    // Global variable
                    let name_idx = self.add_constant(Value::Symbol(name_sym));
                    self.emit(Bytecode::DefineGlobal(name_idx), range);
                }
            }
            Statement::Return(return_stmt) => {
                let range = return_stmt.range;
                if let Some(expr) = return_stmt.value {
                    self.compile_expr(expr)?;
                } else {
                    self.emit(Bytecode::Nil, range);
                }
                self.emit(Bytecode::Return, range);
            }
            Statement::Class(class_def) => {
                let range = class_def.range;
                let name_sym = self.vm.interner.intern(&class_def.name);
                let name_idx = self.add_constant(Value::Symbol(name_sym));

                if let Some(&existing_class) = self.vm.classes.get(&name_sym) {
                    // Reopening: a `class Name { ... }` whose name already
                    // resolves to a global class attaches its members to
                    // *that* class row instead of shadowing it with a fresh
                    // one under the same name. This is a deliberate, narrowly
                    // scoped U5 addition (not a general object-model
                    // redesign — `class-hierarchy mutability` stays open,
                    // open-Q4) needed to make sacred-selector overriding
                    // exercisable from surface Phalcom at all, so the
                    // inliner's override-epoch deopt guard has a real,
                    // testable trigger (ADR-0018; see
                    // `Universe::note_method_installed`).
                    let class_idx = self.add_constant(Value::Obj(existing_class));
                    self.emit(Bytecode::Constant(class_idx), range);
                } else {
                    // Push superclass onto the stack (for now, default to Object)
                    let object_class = self.vm.universe.classes.object_class;
                    let superclass_idx = self.add_constant(Value::Obj(object_class));
                    self.emit(Bytecode::Constant(superclass_idx), range);

                    // TODO: Handle explicit superclass syntax later
                    self.emit(Bytecode::Class(name_idx), range);
                }

                // The class object is now on top of the stack. Iterate through members.
                for member in class_def.members {
                    match member {
                        ClassMember::Method(method_def) => {
                            let range = method_def.range;

                            let arity = method_def.params.len();
                            let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                            let selector = encode_selector(&method_def.name, &labels, SignatureKind::Method(arity as u8));
                            let selector_sym = self.vm.interner.intern(&selector);

                            let param_names: Vec<String> = method_def.params.iter().map(|p| p.name.clone()).collect();
                            let closure = self.compile_block(method_def.body, selector_sym, param_names, true)?;

                            debug!("[Compiler] Compiling method: {} (static: {})", selector, method_def.is_static);

                            let method_obj = self.vm.heap.alloc(Object::Method(MethodObject::new_single(
                                selector_sym,
                                SignatureKind::Method(arity as u8),
                                MethodKind::Closure(closure),
                            )));

                            let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                            // println!("[Compiler] Emitting Constant for method_obj_idx: {}", method_obj_idx);
                            self.emit(Bytecode::Constant(method_obj_idx), range);

                            let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                            // println!(
                            //     "[Compiler] Emitting Method for selector_idx: {}, is_static: {}",
                            //     selector_idx, method_def.is_static
                            // );
                            self.emit(Bytecode::Method(selector_idx, method_def.is_static), range);
                        }
                        ClassMember::Getter(getter_def) => {
                            let range = getter_def.range;

                            let selector = make_signature(&getter_def.name, SignatureKind::Getter);
                            let selector_sym = self.vm.interner.intern(&selector);

                            let closure = self.compile_block(getter_def.body, selector_sym, Vec::new(), true)?;

                            debug!("[Compiler] Compiling getter: {} (static: {})", selector, getter_def.is_static);

                            let method_obj =
                                self.vm.heap.alloc(Object::Method(MethodObject::new_single(selector_sym, SignatureKind::Getter, MethodKind::Closure(closure))));

                            let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                            self.emit(Bytecode::Constant(method_obj_idx), range);

                            let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                            self.emit(Bytecode::Method(selector_idx, getter_def.is_static), range);
                        }
                        ClassMember::Setter(setter_def) => {
                            let range = setter_def.range;

                            let selector = make_signature(&setter_def.name, SignatureKind::Setter);
                            let selector_sym = self.vm.interner.intern(&selector);

                            let closure = self.compile_block(setter_def.body, selector_sym, vec!["value".to_string()], true)?;

                            debug!("[Compiler] Compiling setter: {} (static: {})", selector, setter_def.is_static);

                            let method_obj =
                                self.vm.heap.alloc(Object::Method(MethodObject::new_single(selector_sym, SignatureKind::Setter, MethodKind::Closure(closure))));

                            let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                            self.emit(Bytecode::Constant(method_obj_idx), range);

                            let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                            self.emit(Bytecode::Method(selector_idx, setter_def.is_static), range);
                        }
                    }
                }

                // After defining all methods, the class is still on the stack.
                // Define it as a global variable.
                self.emit(Bytecode::DefineGlobal(name_idx), range);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        match expr {
            Expr::MethodCall(method_call) => {
                // U5 Layer 1 (control-flow.md §3, ADR-0018): a sacred
                // selector sent with literal-block arguments compiles to a
                // guarded inline fast path instead of a plain send. Every
                // other call — including a sacred selector with a
                // *non*-literal block argument — falls through unchanged.
                let range = method_call.range;
                match inliner::recognize(*method_call) {
                    Ok(sacred) => return self.compile_sacred_call(sacred, range),
                    Err(method_call) => {
                        self.compile_expr(method_call.object)?;
                        for arg in &method_call.args {
                            self.compile_expr(arg.expr.clone())?;
                        }
                        let arity = method_call.args.len();
                        let labels: Vec<Option<String>> = method_call.args.iter().map(|a| a.label.clone()).collect();
                        let selector = encode_selector(&method_call.method, &labels, SignatureKind::Method(arity as u8));
                        let selector_sym = self.vm.interner.intern(&selector);
                        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                        self.emit(Bytecode::Invoke(method_call.args.len() as u8, selector_idx), method_call.range);
                    }
                }
            }
            Expr::GetProperty(get_prop) => {
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
            Expr::Nil { range } => {
                self.emit(Bytecode::Nil, range);
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
                self.emit_self(range);
                let name_sym = self.vm.interner.intern(&value);
                let name_idx = self.add_constant(Value::Symbol(name_sym));
                self.emit(Bytecode::GetField(name_idx), range);
            }
            Expr::Assignment(assign_expr) => {
                match *assign_expr.name {
                    Expr::Var { value, range } => {
                        self.compile_expr(assign_expr.value)?;
                        let name_sym = self.vm.interner.intern(&value);
                        if let Some(slot) = self.resolve_local(name_sym) {
                            self.emit(Bytecode::SetLocal(slot as u16), range);
                        } else if let Some(upvalue) = self.resolve_upvalue(name_sym) {
                            self.emit(Bytecode::SetUpvalue(upvalue as u16), range);
                        } else {
                            let name_idx = self.add_constant(Value::Symbol(name_sym));
                            self.emit(Bytecode::SetGlobal(name_idx), range);
                        }
                    }
                    Expr::Field { value, range } => {
                        self.emit_self(range); // Push receiver first
                        self.compile_expr(assign_expr.value)?; // Then push value
                        let name_sym = self.vm.interner.intern(&value);
                        let name_idx = self.add_constant(Value::Symbol(name_sym));
                        self.emit(Bytecode::SetField(name_idx), range);
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
                        let rhs_block = wrap_expr_as_lazy_block(binary_expr.right, range);
                        return self.compile_sacred_call(inliner::SacredCall::And { receiver: binary_expr.left, rhs_block }, range);
                    }
                    BinaryOp::Or => {
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
            Expr::SuperVar { range } => {
                // TODO: Handle `super` keyword. For now, push Nil.
                self.emit(Bytecode::Nil, range);
            }
            Expr::Block(block_expr) => {
                let name_sym = self.vm.interner.intern("<block>");
                let closure = self.compile_block(block_expr.body, name_sym, block_expr.params, false)?;
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
