use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::class::ClassObject;
use crate::closure::ClosureObject;
// use std::fmt::Debug;
use crate::diagnostics::{print_rt, SourceLoc, SOURCE_MAP};
use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::{CallContext, CallFrame};
use crate::interner::{Interner, Symbol};
use crate::interpret::ModuleInfo;
use crate::method::{MethodKind, MethodObject};
use crate::module::{ModuleObject, CORE_MODULE_NAME, MAIN_MODULE_NAME};
use crate::nil::NIL;
use crate::string::phstring_new;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::range::SourceRange;
use phalcom_common::MaybeWeak::Weak;
use phalcom_common::{phref_new, PhRef, PhWeakRef};
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, span, Level};

pub struct VM {
    pub(crate) frames: Vec<PhRef<CallFrame>>,
    pub(crate) stack: Vec<Value>,

    pub modules: HashMap<Symbol, PhRef<ModuleObject>>,
    pub main_module: Option<PhRef<ModuleObject>>,
    pub last_imported_module: Option<PhRef<ModuleObject>>,

    pub classes: HashMap<Symbol, PhRef<ClassObject>>,
    pub interner: Interner,
    pub start_time: Instant,
    pub universe: Universe,
}

impl Default for VM {
    fn default() -> Self {
        todo!()
    }
}

impl VM {
    /// Creates a new VM.
    pub fn new() -> Self {
        let interner = Interner::with_capacity(100);
        let universe = Universe::new();

        let mut vm = Self {
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

        vm
    }

    pub fn get_or_intern(&mut self, name: &str) -> Symbol {
        self.interner.intern(name)
    }

    pub fn resolve_symbol(&self, symbol: Symbol) -> &str {
        self.interner.lookup(symbol)
    }

    // Helper methods to get the current context
    fn current_frame(&self) -> &PhRef<CallFrame> {
        self.frames.last().unwrap()
    }

    pub fn create_single_class(&mut self, name: &str, superclass: Option<PhRef<ClassObject>>) -> PhRef<ClassObject> {
        let class = ClassObject::new(name, Weak(PhWeakRef::default()), superclass);
        phref_new(class)
    }

    pub fn create_class(&mut self, name: &str, superclass: Option<PhRef<ClassObject>>) -> PhRef<ClassObject> {
        let class_class = self.universe.classes.class_class.clone();
        let object_class_class = self.universe.classes.object_class.borrow().class().clone();

        let metaclass_name = name.to_owned() + ".class";
        let metaclass = self.create_single_class(metaclass_name.as_str(), Some(object_class_class));
        metaclass.borrow_mut().set_class_owned(&class_class);

        let class = self.create_single_class(name, superclass);
        class.borrow_mut().set_class_owned(&metaclass);

        self.classes.insert(self.interner.intern(name), class.clone());
        self.classes.insert(self.interner.intern(metaclass_name.as_str()), metaclass.clone());

        class
    }

    pub fn create_module(&mut self, logical_name: &str, abs_path: &str) -> PhRef<ModuleObject> {
        let module_sym = self.interner.intern(logical_name);
        let module_name = phstring_new(logical_name.to_string());
        let m = phref_new(ModuleObject::new(module_name, module_sym, abs_path.to_string(), None));
        self.modules.insert(module_sym, m.clone());
        m
    }

    pub fn register_path(&mut self, module_sym: Symbol, abs_path: &str) {
        if let Some(module) = self.modules.get(&module_sym) {
            module.borrow_mut().path = abs_path.to_string();
        } else {
            debug!("Module with symbol {:?} not found for path registration", module_sym);
        }
    }

    pub fn register_source(&mut self, logical_name: &str, source: &str) {
        let source_ref = Arc::new(String::from(source));
        let module_sym = self.interner.intern(logical_name);
        SOURCE_MAP.write().unwrap().insert(module_sym, source_ref.clone());

        let module_sym = self.interner.intern(logical_name);
        let src_ref = Arc::new(String::from(source));
        SOURCE_MAP.write().unwrap().insert(module_sym, src_ref.clone());
    }

    /// Returns the module for a given symbol
    pub fn get_module(&mut self, module_sym: Symbol) -> Option<PhRef<ModuleObject>> {
        self.modules.get(&module_sym).cloned()
    }

    /// Returns the module for a given name.
    pub fn get_module_from_str(&mut self, name: &str) -> Option<PhRef<ModuleObject>> {
        let sym = self.interner.intern(name);
        self.modules.get(&sym).cloned()
    }

    pub fn define_global(&mut self, module_sym: Symbol, name_sym: Symbol, val: Value) -> PhResult<usize> {
        let module = self.get_module(module_sym);
        module.expect("correct module").borrow().define(name_sym, val)
    }

    pub fn install_core(&mut self) {
        let m = self.create_module(CORE_MODULE_NAME, "<internal core module>");
        self.register_source(CORE_MODULE_NAME, include_str!("../core/core.ph"));
        let core_sym = m.borrow().symbol();
        self.modules.insert(core_sym, m.clone());

        macro_rules! add_class {
            ($field:ident) => {
                let class_obj = self.universe.classes.$field.clone();
                let name_sym = self.interner.intern(class_obj.borrow().name().borrow().as_str());
                self.define_global(core_sym, name_sym, Value::Class(class_obj.clone())).ok();
                self.classes.insert(name_sym, class_obj);
            };
        }

        add_class!(object_class);
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

    fn call_method(&mut self, callee: &Value, method: PhRef<MethodObject>, arity: usize, source_range: SourceRange) -> PhResult<()> {
        match &method.borrow().kind {
            MethodKind::Primitive(native_fn) => {
                let receiver_idx = self.stack.len() - 1 - arity;
                let receiver = self.stack[receiver_idx].clone();
                let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                let result = native_fn(self, &receiver, &args);
                result.map(|result| {
                    self.stack.truncate(receiver_idx);
                    self.stack.push(result);
                })

                // match result {
                //     Ok(result) => {
                //         self.stack.truncate(receiver_idx);
                //         self.stack.push(result);
                //         Ok(())
                //     }
                //     Err(err) => Err(err),
                // }
            }
            MethodKind::Closure(closure) => {
                let new_frame = phref_new(CallFrame::new(
                    closure.clone(),
                    callee.to_context(),
                    0,
                    self.stack.len() - arity - 1,
                    Some(source_range),
                ));
                self.frames.push(new_frame);
                Ok(())
            }
        }
    }

    pub fn runtime_error(&mut self, err: PhError) -> PhResult<()> {
        let mut frames = Vec::new();
        for frame_ref in self.frames.iter().rev() {
            let frame = frame_ref.borrow();
            let closure = frame.closure.borrow();
            let module_name = closure.module.borrow().name.borrow().value();
            let module_source = closure.module.borrow().source.clone();
            let method_name = self.resolve_symbol(closure.callable.name_sym);
            let trace_name = match frame.context() {
                CallContext::Class { class } => {
                    let class_name = class.borrow().class().borrow().name_copy();
                    format!("{}::{} in {}", class_name, method_name, module_name)
                }
                CallContext::Instance { instance } => {
                    let class_name = instance.borrow().class().borrow().name_copy();
                    format!("{}::{} in {}", class_name, method_name, module_name)
                }
                CallContext::Module { module } => {
                    format!("{module_name}")
                }
            };

            let span = closure.callable.chunk.spans[frame.ip - 1];

            frames.push(SourceLoc {
                source: module_source.unwrap(),
                module_name,
                method_name: String::from(method_name),
                span,
            });
        }

        print_rt(&err.to_string(), &frames);
        Err(err)
    }

    pub fn compiler_error(&mut self, err: PhError) {}

    pub fn pop(&mut self) -> Result<Value, PhError> {
        self.stack.pop().ok_or_else(|| RuntimeError::Internal("Stack underflow".to_string()).into())
    }

    // pub fn run_loop(&mut self) -> PhResult<()> {
    //     loop {
    //
    //     }
    // }

    pub fn handle_primitive_op(&self, a: &Value, b: &Value, op: &str) -> PhResult<Option<Value>> {
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
            (Value::String(a_str), Value::String(b_str)) if op == "+" => {
                let mut result = a_str.borrow().value();
                result.push_str(b_str.borrow().as_str());
                Ok(Some(Value::String(phstring_new(result))))
            }
            (Value::String(a_str), Value::Number(n)) if op == "*" => {
                let a_str = a_str.borrow();
                let s = a_str.as_str();
                let result = s.repeat(*n as usize);
                Ok(Some(Value::String(phstring_new(result))))
            }
            (Value::Number(n), Value::String(b_str)) if op == "*" => {
                let b_str = b_str.borrow();
                let s = b_str.as_str();
                let result = s.repeat(*n as usize);
                Ok(Some(Value::String(phstring_new(result))))
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
                    self.stack.push(a.clone());
                    self.stack.push(b.clone());
                    self.call_method(&a, method, 1, $span)?;
                } else if let Some(method) = b.lookup_method(self, selector) {
                    self.stack.push(a.clone());
                    self.stack.push(b.clone());
                    self.call_method(&b, method, 1, $span)?;
                } else {
                    let a = a.to_debug(self);
                    let b = b.to_debug(self);
                    return Err(RuntimeError::BinaryNotSupported {
                        op,
                        left: a.borrow().as_str().to_owned(),
                        right: b.borrow().as_str().to_owned(),
                    }
                    .into());
                }
            }};
        }

        loop {
            if self.frames.is_empty() {
                return Ok(self.stack.pop().unwrap_or(Value::Nil));
            }

            let frame_ref = self.current_frame().clone();
            let mut frame = frame_ref.borrow_mut();
            let closure = frame.closure.clone();
            let chunk = &closure.borrow().callable.chunk;
            let opcode = chunk.code[frame.ip];
            let source_range = chunk.spans[frame.ip];
            let stack_offset = frame.stack_offset; // Get stack_offset before dropping frame

            let span = span!(Level::DEBUG, "vm_opcode", opcode = ?opcode);
            let _enter = span.enter();
            debug!("Stack before: {:?}", self.stack);

            frame.ip += 1;
            drop(frame); // Drop frame here after extracting necessary info

            match opcode {
                Bytecode::Constant(idx) => {
                    let constant = chunk.constants[idx as usize].clone();
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
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module = closure.borrow().module.clone();
                        module.borrow().define(*name_sym, self.stack.last().unwrap().clone()).unwrap();
                        self.stack.pop();
                    }
                }
                Bytecode::GetGlobal(idx) => {
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module = closure.borrow().module.clone();
                        if let Some(value) = module.borrow().get(*name_sym) {
                            self.stack.push(value.clone());
                        } else {
                            // If not in the current module, try the core module.
                            let core_module_sym = self.interner.intern(CORE_MODULE_NAME);
                            let core_module = self.get_module(core_module_sym).expect("core module");
                            if let Some(value) = core_module.borrow().get(*name_sym) {
                                self.stack.push(value.clone());
                            } else {
                                let name = self.resolve_symbol(*name_sym);
                                return Err(RuntimeError::Message(format!("Undefined variable '{}'.", name)).into());
                            }
                        }
                    }
                }
                Bytecode::SetGlobal(idx) => {
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module = closure.borrow().module.clone();
                        if let Some(slot) = module.borrow().name_to_slot.borrow().get(name_sym) {
                            module.borrow().set_global(*slot, self.stack.last().unwrap().clone()).unwrap();
                        } else {
                            let name = self.resolve_symbol(*name_sym);
                            return Err(RuntimeError::Message(format!("Undefined variable '{}'.", name)).into());
                        }
                    }
                }
                Bytecode::GetLocal(slot) => {
                    let frame_borrow = frame_ref.borrow();
                    let local_idx = frame_borrow.stack_offset + slot as usize;
                    drop(frame_borrow);

                    if local_idx < self.stack.len() {
                        let value = self.stack[local_idx].clone();
                        self.stack.push(value);
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::SetLocal(slot) => {
                    let frame_borrow = frame_ref.borrow();
                    let local_idx = frame_borrow.stack_offset + slot as usize;
                    drop(frame_borrow);

                    if local_idx < self.stack.len() {
                        let value = self.stack.last().unwrap().clone();
                        self.stack[local_idx] = value;
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::Class(idx) => {
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let name = self.resolve_symbol(*name_sym).to_string();
                        let superclass = self.stack.pop().unwrap();
                        if let Value::Class(superclass_obj) = superclass {
                            let new_class = self.create_class(&name, Some(superclass_obj));
                            self.stack.push(Value::Class(new_class));
                        } else {
                            return Err(RuntimeError::InvalidSuperClass(format!("{superclass}")).into());
                        }
                    }
                }
                Bytecode::Method(selector_idx, is_static) => {
                    let selector_val = &chunk.constants[selector_idx as usize];
                    let selector = selector_val.as_symbol().unwrap();
                    let method_val = self.stack.pop().unwrap();
                    let class_val = self.stack.last().unwrap();
                    let selector_name = self.resolve_symbol(selector);
                    if let (Value::Method(method_obj), Value::Class(class_obj)) = (method_val, class_val) {
                        debug!("Adding method {} to class {}", selector_name, class_obj.borrow().name_copy());
                        method_obj.borrow_mut().set_holder(PhRef::downgrade(class_obj));
                        if is_static {
                            class_obj.borrow().class().borrow_mut().add_method(selector, method_obj);
                        } else {
                            class_obj.borrow_mut().add_method(selector, method_obj);
                        }
                    } else {
                        return Err(RuntimeError::Internal("Invalid types for method definition.".to_string()).into());
                    }
                }
                Bytecode::GetSelf => {
                    let frame_borrow = frame_ref.borrow();
                    let receiver = self.stack[frame_borrow.stack_offset].clone();
                    drop(frame_borrow);
                    self.stack.push(receiver);
                }
                Bytecode::GetField(idx) => {
                    let field_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(field_sym) = field_val {
                        let receiver = self.stack.pop().ok_or("Stack underflow for GetField receiver")?; // Pop the receiver pushed by GetSelf
                        let field_str = self.resolve_symbol(*field_sym);
                        debug!("Getting field {} from value {}", field_str, receiver);
                        if let Value::Instance(instance_obj) = &receiver {
                            if let Some(field_value) = instance_obj.borrow().fields.get(field_sym) {
                                self.stack.push(field_value.clone()); // Push the field value
                            } else {
                                self.stack.push(Value::Nil); // Push nil if field not found
                            }
                        } else {
                            return Err(RuntimeError::Internal("Only instances can have fields.".to_string()).into());
                        }
                    }
                }
                Bytecode::SetField(idx) => {
                    let field_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(field_sym) = field_val {
                        let value_to_assign = self.stack.pop().ok_or("Stack underflow on field assignment")?;
                        let receiver = self.stack.pop().ok_or("Stack underflow for SetField receiver")?;

                        if let Value::Instance(instance_obj) = &receiver {
                            instance_obj.borrow_mut().fields.insert(*field_sym, value_to_assign.clone());
                            self.stack.push(value_to_assign); // Push the assigned value back
                        } else {
                            return Err(RuntimeError::Internal("Only instances can have fields.".to_string()).into());
                        }
                    }
                }
                Bytecode::Invoke(arity, selector_idx) => {
                    let selector_val = &chunk.constants[selector_idx as usize];
                    let arity = arity as usize;
                    let receiver_idx = self.stack.len() - 1 - arity;
                    let receiver = self.stack[receiver_idx].clone();

                    let selector_sym = selector_val.as_symbol().unwrap();

                    if let Some(method) = receiver.lookup_method(self, selector_sym) {
                        self.call_method(&receiver, method, arity, source_range)?;
                    } else {
                        let selector_name = self.resolve_symbol(selector_sym);
                        let receiver_name = receiver.to_string(self);
                        return Err(RuntimeError::MethodNotFound {
                            selector: selector_name.to_owned(),
                            value: receiver_name.borrow().as_str().to_owned(),
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
                    let popped_ref = popped.borrow();
                    self.stack.truncate(popped_ref.stack_offset);
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
                    self.stack.push(Value::Bool(a == b));
                }
                Bytecode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a != b));
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
                        self.stack.push(val.clone());
                        self.call_method(&val, method, 0, source_range)?;
                    }

                    let val = val.to_debug(self);
                    return Err(RuntimeError::UnaryNotSupported {
                        op: "-",
                        value: val.borrow().as_str().to_owned(),
                    }
                    .into());
                }
                Bytecode::Not => {
                    let val = self.pop()?;
                    if let Value::Bool(b) = val {
                        self.stack.push(Value::Bool(!b));
                        continue;
                    }

                    let selector = self.interner.intern("not");
                    if let Some(method) = val.lookup_method(self, selector) {
                        self.stack.push(val.clone());
                        self.call_method(&val, method, 0, source_range)?;
                        continue;
                    }

                    let val = val.to_debug(self);
                    return Err(RuntimeError::UnaryNotSupported {
                        op: "not",
                        value: val.borrow().as_str().to_owned(),
                    }
                    .into());
                }
            }
            debug!("Stack after opcode {:?}: {:?}", opcode, self.stack);
        }
    }

    fn format_stack_trace(&self, error_message: String) -> String {
        let mut trace = String::new();
        trace.push_str(&format!("Error: {}\n", error_message));
        for frame_ref in self.frames.iter().rev() {
            let frame = frame_ref.borrow();
            let closure = frame.closure.borrow();
            let module_name = closure.module.borrow().name.borrow().value();
            let method_name = self.resolve_symbol(closure.callable.name_sym);
            let trace_name = match frame.context() {
                CallContext::Class { class } => {
                    let class_name = class.borrow().class().borrow().name_copy();
                    format!("{}::{} in {}", class_name, method_name, module_name)
                }
                CallContext::Instance { instance } => {
                    let class_name = instance.borrow().class().borrow().name_copy();
                    format!("{}::{} in {}", class_name, method_name, module_name)
                }
                CallContext::Module { module } => {
                    // let module_name = module.borrow().name().borrow();
                    format!("{module_name}")
                }
            };
            trace.push_str(&format!("  at {}\n", trace_name));
        }
        trace
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::Bytecode;
//     use crate::chunk::Chunk;
//
//     #[test]
//     fn test_vm_addition() {
//         let mut vm = VM::new();
//         let module = vm.module_from_str("test");
//         let mut chunk = Chunk::default();
//         let string1_idx = chunk.add_constant(Value::string_from_str("hello, "));
//         let string2_idx = chunk.add_constant(Value::string_from_str("world!"));
//         let selector_sym = vm.interner.intern("+(_)");
//         let const_selector_idx = chunk.add_constant(Value::Symbol(selector_sym));
//         chunk.add_instruction(Bytecode::Constant(string1_idx));
//         chunk.add_instruction(Bytecode::Constant(string2_idx));
//         chunk.add_instruction(Bytecode::Invoke(1, const_selector_idx));
//         chunk.add_instruction(Bytecode::Return);
//         use crate::callable::Callable;
//         let entry = phref_new(ClosureObject {
//             callable: Callable {
//                 chunk,
//                 max_slots: 0,
//                 num_upvalues: 0,
//                 arity: 0,
//                 name_sym: vm.interner.intern("test_vm_addition"),
//             },
//             module: module.clone(),
//             upvalues: Vec::new(),
//         });
//         let result = vm.run_module(module, entry).expect("VM execution failed");
//         let expected = Value::string_from_str("hello, world!");
//         assert_eq!(result, expected, "String addition failed");
//     }
//
//     #[test]
//     fn test_global_variable_definition_and_assignment() {
//         let mut vm = VM::new();
//         let module = vm.module_from_str("test_globals");
//         let mut chunk = Chunk::default();
//         let x_sym = vm.interner.intern("x");
//         let x_idx = chunk.add_constant(Value::Symbol(x_sym));
//         let ten_idx = chunk.add_constant(Value::Number(10.0));
//         chunk.add_instruction(Bytecode::Constant(ten_idx));
//         chunk.add_instruction(Bytecode::DefineGlobal(x_idx));
//         chunk.add_instruction(Bytecode::GetGlobal(x_idx));
//         let twenty_idx = chunk.add_constant(Value::Number(20.0));
//         chunk.add_instruction(Bytecode::Constant(twenty_idx));
//         chunk.add_instruction(Bytecode::SetGlobal(x_idx));
//         chunk.add_instruction(Bytecode::GetGlobal(x_idx));
//         chunk.add_instruction(Bytecode::Return);
//         use crate::callable::Callable;
//         let entry = phref_new(ClosureObject {
//             callable: Callable {
//                 chunk,
//                 max_slots: 0,
//                 num_upvalues: 0,
//                 arity: 0,
//                 name_sym: vm.interner.intern("test_global_variable_definition_and_assignment"),
//             },
//             module: module.clone(),
//             upvalues: Vec::new(),
//         });
//         let result = vm.run_module(module, entry).expect("VM execution failed");
//         assert_eq!(result, Value::Number(20.0), "Global var assignment failed");
//     }
// }
