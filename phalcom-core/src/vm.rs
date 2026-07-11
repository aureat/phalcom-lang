//! The bytecode virtual machine: dispatch loop, call stack, and heap ownership.
//!
//! The [`VM`] owns exactly one [`Heap`] ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)):
//! every class, instance, method, module, closure and string lives there and is
//! reached through a `Copy` [`ObjRef`] handle rather than an `Rc<RefCell<T>>`.
//! Values on the operand stack are `Copy` [`Value`]s
//! ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)); call frames
//! ([`CallFrame`]) are `Copy` too, so the interpreter carries no borrow-panic
//! surface. Method lookup keys on signature symbols (`object-model.md` §3).

use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::class::ClassObject;
use crate::diagnostics::{print_rt, SourceLoc, SOURCE_MAP};
use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::CallFrame;
use crate::heap::{ClassId, Heap, ObjRef, Object};
use crate::interner::{Interner, Symbol};
use crate::method::MethodKind;
use crate::module::{ModuleObject, CORE_MODULE_NAME};
use crate::nil::NIL;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::range::SourceRange;
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, span, Level};

/// The bytecode virtual machine: owns the [`Heap`], the operand stack, and the
/// call stack, and drives dispatch.
///
/// See the module docs for the ownership model
/// ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
pub struct VM {
    /// The single object heap; all class/instance/method/module/closure/string
    /// storage lives here, keyed by [`ObjRef`].
    pub heap: Heap,
    /// The active call stack, innermost frame last. [`CallFrame`] is `Copy`.
    pub(crate) frames: Vec<CallFrame>,
    /// The operand stack of `Copy` [`Value`]s.
    pub(crate) stack: Vec<Value>,

    /// Loaded modules by name [`Symbol`], each a [`ModuleObject`] handle.
    pub modules: HashMap<Symbol, ObjRef>,
    /// Handle to the program entry module, once known.
    pub main_module: Option<ObjRef>,
    /// Handle to the most recently imported module, for `import` resolution.
    pub last_imported_module: Option<ObjRef>,

    /// Named classes by name [`Symbol`], each a [`ClassId`] handle.
    pub classes: HashMap<Symbol, ClassId>,
    /// The symbol interner backing selectors, names and string identity.
    pub interner: Interner,
    /// VM start time, used for `System` timing primitives.
    pub start_time: Instant,
    /// The kernel: handles to the bootstrapped core classes.
    pub universe: Universe,
}

impl Default for VM {
    fn default() -> Self {
        todo!()
    }
}

impl VM {
    /// Creates a new VM: builds the heap, bootstraps the kernel tower, and
    /// installs the core module and native primitives.
    pub fn new() -> Self {
        let interner = Interner::with_capacity(100);
        let mut heap = Heap::new();
        let universe = Universe::new(&mut heap);

        let mut vm = Self {
            heap,
            frames: Vec::with_capacity(256),
            stack: Vec::with_capacity(1024),
            interner,
            start_time: Instant::now(),
            modules: HashMap::new(),
            main_module: None,
            last_imported_module: None,
            classes: HashMap::new(),
            universe,
        };

        // Bootstrap core module and primitive methods
        vm.install_core();
        Universe::install_primitives(&mut vm);
        vm.universe
            .verify_invariants(&vm.heap)
            .expect("kernel invariants (object-model.md §5-6)");

        vm
    }

    /// Interns `name`, returning its [`Symbol`].
    pub fn get_or_intern(&mut self, name: &str) -> Symbol {
        self.interner.intern(name)
    }

    /// Resolves `symbol` back to its interned string.
    pub fn resolve_symbol(&self, symbol: Symbol) -> &str {
        self.interner.lookup(symbol)
    }

    /// Allocates an immutable string on the heap and returns it as a [`Value`].
    ///
    /// Native primitives that produce strings (e.g. `toString`, string `+`) use
    /// this to move an owned [`String`] into an
    /// [`Object::Str`] and hand back a
    /// [`Value::Obj`] handle ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
    pub fn alloc_string_value(&mut self, text: String) -> Value {
        Value::Obj(self.heap.alloc_string(text))
    }

    /// Allocates a bare class named `name`, wired only to `superclass`.
    ///
    /// The metaclass link is left unset; callers such as [`Self::create_class`]
    /// patch it. Realizes the allocate-then-patch bootstrap
    /// ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
    pub fn create_single_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let id = self.heap.alloc_class(ClassObject::bare(name));
        self.heap.class_mut(id).set_superclass(superclass);
        id
    }

    /// Creates a user class `name` with its own metaclass and wires the tower.
    ///
    /// Follows the metaclass parallel rule
    /// ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)):
    /// the metaclass `"{name}.class"` is an instance of `Metaclass` whose
    /// superclass is `superclass`'s own metaclass (`Class` if `superclass` is
    /// `None`), and the class itself is an instance of that metaclass with the
    /// requested `superclass`.
    pub fn create_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let metaclass_class = self.universe.classes.metaclass_class;
        let metaclass_superclass = match superclass {
            Some(sc) => self.heap.class(sc).class,
            None => self.universe.classes.class_class,
        };

        let metaclass_name = name.to_owned() + ".class";
        let metaclass = self.create_single_class(&metaclass_name, Some(metaclass_superclass));
        self.heap.class_mut(metaclass).set_class(metaclass_class);

        let class = self.create_single_class(name, superclass);
        self.heap.class_mut(class).set_class(metaclass);

        let name_sym = self.interner.intern(name);
        let meta_sym = self.interner.intern(&metaclass_name);
        self.classes.insert(name_sym, class);
        self.classes.insert(meta_sym, metaclass);

        class
    }

    /// Allocates a module with `logical_name`/`abs_path` and registers it.
    pub fn create_module(&mut self, logical_name: &str, abs_path: &str) -> ObjRef {
        let module_sym = self.interner.intern(logical_name);
        let module = ModuleObject::new(logical_name.to_string(), module_sym, abs_path.to_string(), None);
        let id = self.heap.alloc(Object::Module(module));
        self.modules.insert(module_sym, id);
        id
    }

    /// Updates the absolute filesystem path of the module named `module_sym`.
    pub fn register_path(&mut self, module_sym: Symbol, abs_path: &str) {
        if let Some(&module_id) = self.modules.get(&module_sym) {
            self.heap.module_mut(module_id).path = abs_path.to_string();
        } else {
            debug!("Module with symbol {:?} not found for path registration", module_sym);
        }
    }

    /// Registers `source` text for the module `logical_name` in the source map.
    pub fn register_source(&mut self, logical_name: &str, source: &str) {
        let source_ref = Arc::new(String::from(source));
        let module_sym = self.interner.intern(logical_name);
        SOURCE_MAP.write().unwrap().insert(module_sym, source_ref.clone());

        let module_sym = self.interner.intern(logical_name);
        let src_ref = Arc::new(String::from(source));
        SOURCE_MAP.write().unwrap().insert(module_sym, src_ref.clone());
    }

    /// Returns the module handle for `module_sym`, if loaded.
    pub fn get_module(&mut self, module_sym: Symbol) -> Option<ObjRef> {
        self.modules.get(&module_sym).copied()
    }

    /// Returns the module handle for the module named `name`, if loaded.
    pub fn get_module_from_str(&mut self, name: &str) -> Option<ObjRef> {
        let sym = self.interner.intern(name);
        self.modules.get(&sym).copied()
    }

    /// Defines global `name_sym = val` in the module `module_sym`.
    ///
    /// # Errors
    ///
    /// Propagates [`ModuleObject::define`](crate::module::ModuleObject::define)
    /// errors (e.g. too many globals).
    pub fn define_global(&mut self, module_sym: Symbol, name_sym: Symbol, val: Value) -> PhResult<usize> {
        let module = self.get_module(module_sym).expect("correct module");
        self.heap.module_mut(module).define(name_sym, val)
    }

    /// Bootstraps the core module and exposes each kernel class as a global.
    pub fn install_core(&mut self) {
        let m = self.create_module(CORE_MODULE_NAME, "<internal core module>");
        self.register_source(CORE_MODULE_NAME, include_str!("../core/core.ph"));
        let core_sym = self.heap.module(m).symbol();
        self.modules.insert(core_sym, m);

        macro_rules! add_class {
            ($field:ident) => {
                let class_id = self.universe.classes.$field;
                let name = self.heap.class(class_id).name.clone();
                let name_sym = self.interner.intern(&name);
                self.define_global(core_sym, name_sym, Value::Obj(class_id)).ok();
                self.classes.insert(name_sym, class_id);
            };
        }

        add_class!(object_class);
        add_class!(behavior_class);
        add_class!(class_class);
        add_class!(metaclass_class);
        add_class!(number_class);
        add_class!(string_class);
        add_class!(bool_class);
        add_class!(nil_class);
        add_class!(method_class);
        add_class!(symbol_class);
        add_class!(system_class);
    }

    /// Dispatches a call to `method` on `callee` with `arity` arguments.
    ///
    /// A primitive runs its native function in place; a closure pushes a new
    /// [`CallFrame`] to be executed by [`Self::run`].
    ///
    /// # Errors
    ///
    /// Propagates errors returned by a primitive implementation.
    fn call_method(&mut self, callee: &Value, method: ObjRef, arity: usize, source_range: SourceRange) -> PhResult<()> {
        let kind = self.heap.method(method).kind;
        match kind {
            MethodKind::Primitive(native_fn) => {
                let receiver_idx = self.stack.len() - 1 - arity;
                let receiver = self.stack[receiver_idx];
                let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                let result = native_fn(self, &receiver, &args);
                result.map(|result| {
                    self.stack.truncate(receiver_idx);
                    self.stack.push(result);
                })
            }
            MethodKind::Closure(closure_id) => {
                let context = callee.to_context(&self.heap);
                let new_frame = CallFrame::new(closure_id, context, 0, self.stack.len() - arity - 1, Some(source_range));
                self.frames.push(new_frame);
                Ok(())
            }
        }
    }

    /// Prints a runtime error with a source-mapped stack trace and returns it.
    ///
    /// # Errors
    ///
    /// Always returns `err` (the trace is a side effect).
    pub fn runtime_error(&mut self, err: PhError) -> PhResult<()> {
        let mut frames = Vec::new();
        for frame in self.frames.clone().iter().rev() {
            let closure = self.heap.closure(frame.closure);
            let module_id = closure.module;
            let name_sym = closure.callable.name_sym;
            let span = closure.callable.chunk.spans[frame.ip - 1];

            let module = self.heap.module(module_id);
            let module_name = module.name.clone();
            let module_source = module.source.clone();
            let method_name = self.resolve_symbol(name_sym).to_string();

            frames.push(SourceLoc {
                source: module_source.unwrap(),
                module_name,
                method_name,
                span,
            });
        }

        print_rt(&err.to_string(), &frames);
        Err(err)
    }

    /// Placeholder for compiler-error reporting (currently a no-op).
    pub fn compiler_error(&mut self, err: PhError) {}

    /// Pops a value from the operand stack.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Internal`] on stack underflow.
    pub fn pop(&mut self) -> Result<Value, PhError> {
        self.stack.pop().ok_or_else(|| RuntimeError::Internal("Stack underflow".to_string()).into())
    }

    /// Applies a fast-path binary `op` to immediate operands, if both qualify.
    ///
    /// Returns `Some(result)` for the arithmetic/logic/string fast paths and
    /// `None` when the operands need full method dispatch. String results are
    /// allocated on the heap.
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but keeps the fallible signature for the
    /// dispatch fast path.
    pub fn handle_primitive_op(&mut self, a: &Value, b: &Value, op: &str) -> PhResult<Option<Value>> {
        match (a, b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                let result = match op {
                    "+" => Value::Number(a_num + b_num),
                    "-" => Value::Number(a_num - b_num),
                    "*" => Value::Number(a_num * b_num),
                    "/" => Value::Number(a_num / b_num),
                    "%" => Value::Number(a_num % b_num),
                    _ => return Ok(None),
                };
                Ok(Some(result))
            }
            (Value::Obj(a_id), Value::Obj(b_id)) if op == "+" => {
                let concat = match (self.heap.as_string(*a_id), self.heap.as_string(*b_id)) {
                    (Some(a_str), Some(b_str)) => Some(a_str.value() + b_str.as_str()),
                    _ => None,
                };
                match concat {
                    Some(result) => Ok(Some(self.alloc_string_value(result))),
                    None => Ok(None),
                }
            }
            (Value::Obj(a_id), Value::Number(n)) if op == "*" => {
                let repeated = self.heap.as_string(*a_id).map(|s| s.as_str().repeat(*n as usize));
                match repeated {
                    Some(result) => Ok(Some(self.alloc_string_value(result))),
                    None => Ok(None),
                }
            }
            (Value::Number(n), Value::Obj(b_id)) if op == "*" => {
                let repeated = self.heap.as_string(*b_id).map(|s| s.as_str().repeat(*n as usize));
                match repeated {
                    Some(result) => Ok(Some(self.alloc_string_value(result))),
                    None => Ok(None),
                }
            }
            (Value::Bool(a_bool), Value::Bool(b_bool)) => {
                let result = match op {
                    "&&" => Value::Bool(*a_bool && *b_bool),
                    "||" => Value::Bool(*a_bool || *b_bool),
                    _ => return Ok(None),
                };
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    /// Runs the dispatch loop until the call stack empties, returning the result.
    ///
    /// # Errors
    ///
    /// Returns any [`RuntimeError`] raised during execution (undefined variable,
    /// method-not-found, unsupported operator, and so on).
    pub fn run(&mut self) -> PhResult<Value> {
        macro_rules! binary_op {
            ($op:tt, $selector:expr, $span:expr) => {{
                let b = self.pop()?;
                let a = self.pop()?;

                let op = stringify!($op);

                if let Some(result) = self.handle_primitive_op(&a, &b, op)? {
                    self.stack.push(result);
                    continue;
                }

                let selector = self.interner.intern($selector);

                if let Some(method) = a.lookup_method(self, selector) {
                    self.stack.push(a);
                    self.stack.push(b);
                    self.call_method(&a, method, 1, $span)?;
                } else if let Some(method) = b.lookup_method(self, selector) {
                    self.stack.push(a);
                    self.stack.push(b);
                    self.call_method(&b, method, 1, $span)?;
                } else {
                    let left = a.to_debug(self);
                    let right = b.to_debug(self);
                    return Err(RuntimeError::BinaryNotSupported { op, left, right }.into());
                }
            }};
        }

        loop {
            if self.frames.is_empty() {
                return Ok(self.stack.pop().unwrap_or(Value::Nil));
            }

            let frame = *self.frames.last().unwrap();
            let closure_id = frame.closure;
            let ip = frame.ip;
            let stack_offset = frame.stack_offset;

            let (opcode, source_range) = {
                let chunk = &self.heap.closure(closure_id).callable.chunk;
                (chunk.code[ip], chunk.spans[ip])
            };

            let span = span!(Level::DEBUG, "vm_opcode", opcode = ?opcode);
            let _enter = span.enter();
            debug!("Stack before: {:?}", self.stack);

            self.frames.last_mut().unwrap().ip += 1;

            match opcode {
                Bytecode::Constant(idx) => {
                    let constant = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    debug!("Pushing constant: {:?}", constant);
                    self.stack.push(constant);
                }
                Bytecode::Nil => self.stack.push(NIL),
                Bytecode::True => self.stack.push(TRUE),
                Bytecode::False => self.stack.push(FALSE),
                Bytecode::Pop => {
                    self.stack.pop();
                }
                Bytecode::DefineGlobal(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module_id = self.heap.closure(closure_id).module;
                        let value = *self.stack.last().unwrap();
                        self.heap.module_mut(module_id).define(name_sym, value).unwrap();
                        self.stack.pop();
                    }
                }
                Bytecode::GetGlobal(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module_id = self.heap.closure(closure_id).module;
                        if let Some(value) = self.heap.module(module_id).get(name_sym) {
                            self.stack.push(value);
                        } else {
                            // If not in the current module, try the core module.
                            let core_module_sym = self.interner.intern(CORE_MODULE_NAME);
                            let core_module = self.get_module(core_module_sym).expect("core module");
                            if let Some(value) = self.heap.module(core_module).get(name_sym) {
                                self.stack.push(value);
                            } else {
                                let name = self.resolve_symbol(name_sym).to_string();
                                return Err(RuntimeError::Message(format!("Undefined variable '{}'.", name)).into());
                            }
                        }
                    }
                }
                Bytecode::SetGlobal(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module_id = self.heap.closure(closure_id).module;
                        let slot = self.heap.module(module_id).name_to_slot.get(&name_sym).copied();
                        if let Some(slot) = slot {
                            let value = *self.stack.last().unwrap();
                            self.heap.module_mut(module_id).set_global(slot, value).unwrap();
                        } else {
                            let name = self.resolve_symbol(name_sym).to_string();
                            return Err(RuntimeError::Message(format!("Undefined variable '{}'.", name)).into());
                        }
                    }
                }
                Bytecode::GetLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx < self.stack.len() {
                        let value = self.stack[local_idx];
                        self.stack.push(value);
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::SetLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx < self.stack.len() {
                        let value = *self.stack.last().unwrap();
                        self.stack[local_idx] = value;
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::Class(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let name = self.resolve_symbol(name_sym).to_string();
                        let superclass = self.stack.pop().unwrap();
                        match superclass {
                            Value::Obj(sc_id) if self.heap.as_class(sc_id).is_some() => {
                                let new_class = self.create_class(&name, Some(sc_id));
                                self.stack.push(Value::Obj(new_class));
                            }
                            _ => return Err(RuntimeError::InvalidSuperClass(format!("{superclass}")).into()),
                        }
                    }
                }
                Bytecode::Method(selector_idx, is_static) => {
                    let selector_val = self.heap.closure(closure_id).callable.chunk.constants[selector_idx as usize];
                    let selector = selector_val.as_symbol().unwrap();
                    let method_val = self.stack.pop().unwrap();
                    let class_val = *self.stack.last().unwrap();
                    if let (Value::Obj(method_id), Value::Obj(class_id)) = (method_val, class_val) {
                        self.heap.method_mut(method_id).set_holder(class_id);
                        if is_static {
                            let meta = self.heap.class(class_id).class;
                            self.heap.class_mut(meta).add_method(selector, method_id);
                        } else {
                            self.heap.class_mut(class_id).add_method(selector, method_id);
                        }
                    } else {
                        return Err(RuntimeError::Internal("Invalid types for method definition.".to_string()).into());
                    }
                }
                Bytecode::GetSelf => {
                    let receiver = self.stack[stack_offset];
                    self.stack.push(receiver);
                }
                Bytecode::GetField(idx) => {
                    let field_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(field_sym) = field_val {
                        let receiver = self.stack.pop().ok_or("Stack underflow for GetField receiver")?;
                        match receiver {
                            Value::Obj(id) if self.heap.as_instance(id).is_some() => {
                                let field_value = self.heap.instance(id).fields.get(&field_sym).copied();
                                self.stack.push(field_value.unwrap_or(Value::Nil));
                            }
                            _ => return Err(RuntimeError::Internal("Only instances can have fields.".to_string()).into()),
                        }
                    }
                }
                Bytecode::SetField(idx) => {
                    let field_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(field_sym) = field_val {
                        let value_to_assign = self.stack.pop().ok_or("Stack underflow on field assignment")?;
                        let receiver = self.stack.pop().ok_or("Stack underflow for SetField receiver")?;
                        match receiver {
                            Value::Obj(id) if self.heap.as_instance(id).is_some() => {
                                self.heap.instance_mut(id).fields.insert(field_sym, value_to_assign);
                                self.stack.push(value_to_assign);
                            }
                            _ => return Err(RuntimeError::Internal("Only instances can have fields.".to_string()).into()),
                        }
                    }
                }
                Bytecode::Invoke(arity, selector_idx) => {
                    let selector_val = self.heap.closure(closure_id).callable.chunk.constants[selector_idx as usize];
                    let arity = arity as usize;
                    let receiver_idx = self.stack.len() - 1 - arity;
                    let receiver = self.stack[receiver_idx];

                    let selector_sym = selector_val.as_symbol().unwrap();

                    if let Some(method) = receiver.lookup_method(self, selector_sym) {
                        self.call_method(&receiver, method, arity, source_range)?;
                    } else {
                        let selector_name = self.resolve_symbol(selector_sym).to_string();
                        let receiver_name = receiver.to_string(self);
                        return Err(RuntimeError::MethodNotFound {
                            selector: selector_name,
                            value: receiver_name,
                        }
                        .into());
                    }
                }
                Bytecode::Return => {
                    let return_value = self.stack.pop().unwrap_or(Value::Nil);
                    let popped = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        return Ok(return_value);
                    }
                    self.stack.truncate(popped.stack_offset);
                    self.stack.push(return_value);
                }
                Bytecode::Add => binary_op!(+, "+(_:)", source_range),
                Bytecode::Subtract => binary_op!(-, "-(_:)", source_range),
                Bytecode::Multiply => binary_op!(*, "*(_:)", source_range),
                Bytecode::Divide => binary_op!(/, "/(_:)", source_range),
                Bytecode::Modulo => binary_op!(%, "%(_:)", source_range),
                Bytecode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a.value_eq(&b, &self.heap)));
                }
                Bytecode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(!a.value_eq(&b, &self.heap)));
                }
                Bytecode::Greater => binary_op!(>, ">(_:)", source_range),
                Bytecode::GreaterEqual => binary_op!(>=, ">=(_:)", source_range),
                Bytecode::Less => binary_op!(<, "<(_:)", source_range),
                Bytecode::LessEqual => binary_op!(<=, "<=(_:)", source_range),
                Bytecode::And => binary_op!(&&, "and(_:)", source_range),
                Bytecode::Or => binary_op!(||, "or(_:)", source_range),
                Bytecode::Negate => {
                    let val = self.pop()?;
                    if let Value::Number(num) = val {
                        self.stack.push(Value::Number(-num));
                        continue;
                    }

                    let selector = self.interner.intern("-");
                    if let Some(method) = val.lookup_method(self, selector) {
                        self.stack.push(val);
                        self.call_method(&val, method, 0, source_range)?;
                    }

                    let value = val.to_debug(self);
                    return Err(RuntimeError::UnaryNotSupported { op: "-", value }.into());
                }
                Bytecode::Not => {
                    let val = self.pop()?;
                    if let Value::Bool(b) = val {
                        self.stack.push(Value::Bool(!b));
                        continue;
                    }

                    let selector = self.interner.intern("not");
                    if let Some(method) = val.lookup_method(self, selector) {
                        self.stack.push(val);
                        self.call_method(&val, method, 0, source_range)?;
                        continue;
                    }

                    let value = val.to_debug(self);
                    return Err(RuntimeError::UnaryNotSupported { op: "not", value }.into());
                }
            }
            debug!("Stack after opcode {:?}: {:?}", opcode, self.stack);
        }
    }
}
