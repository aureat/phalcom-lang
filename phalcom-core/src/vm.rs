use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::class::ClassObject;
use crate::closure::ClosureObject;
// use std::fmt::Debug;
use crate::diagnostics::SOURCE_MAP;
use crate::error::{PhError, PhResult};
use crate::frame::{CallContext, CallFrame};
use crate::interner::{Interner, Symbol};
use crate::method::{MethodKind, MethodObject};
use crate::module::{ModuleObject, CORE_MODULE_NAME};
use crate::nil::NIL;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::MaybeWeak::Weak;
use phalcom_common::{phref_new, PhRef, PhWeakRef};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, span, Level};

pub struct VM {
    frames: Vec<PhRef<CallFrame>>,
    stack: Vec<Value>,
    pub modules: HashMap<Symbol, PhRef<ModuleObject>>,
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
            classes: HashMap::new(),
            universe,
        };

        // Bootstrap core module and primitive methods
        vm.install_core();
        Universe::install_primitives(&mut vm);

        vm
    }

    /// Runs a module with given closure as entry point.
    pub fn run_module(&mut self, module: PhRef<ModuleObject>, entry: PhRef<ClosureObject>) -> PhResult<Value> {
        let module_sym = module.borrow().symbol();
        self.modules.insert(module_sym, module.clone());
        entry.borrow_mut().module = module.clone();
        self.frames.clear();
        self.stack.clear();
        let frame = phref_new(CallFrame::new(entry, CallContext::Module { module }, 0, 0));
        self.frames.push(frame);
        self.run()
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

    pub fn create_module(&mut self, module_sym: Symbol, source: &str) -> PhRef<ModuleObject> {
        let source_ref = Arc::new(String::from(source));
        SOURCE_MAP.write().unwrap().insert(module_sym, source_ref.clone());

        let m = phref_new(ModuleObject::new(self, module_sym, Some(source_ref)));
        self.modules.insert(module_sym, m.clone());
        m
    }

    pub fn create_module_from_str(&mut self, name: &str, source: &str) -> PhRef<ModuleObject> {
        let module_sym = self.interner.intern(name);

        let source_ref = Arc::new(String::from(source));
        SOURCE_MAP.write().unwrap().insert(module_sym, source_ref.clone());

        let m = phref_new(ModuleObject::new(self, module_sym, Some(source_ref)));
        self.modules.insert(module_sym, m.clone());
        m
    }

    pub fn create_module_from_stdin(&mut self) -> PhRef<ModuleObject> {
        let module_sym = self.interner.intern("<main>");

        let m = phref_new(ModuleObject::new(self, module_sym, None));
        self.modules.insert(module_sym, m.clone());
        m
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
        let core_sym = self.interner.intern(CORE_MODULE_NAME);
        let core_mod = self.create_module(core_sym, "");

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

        self.define_global(core_sym, core_sym, Value::Module(core_mod)).ok();
    }

    fn call_method(&mut self, callee: &Value, method: PhRef<MethodObject>, arity: usize) -> PhResult<()> {
        match &method.borrow().kind {
            MethodKind::Primitive(native_fn) => {
                let receiver_idx = self.stack.len() - 1 - arity;
                let receiver = self.stack[receiver_idx].clone();
                let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                let result = native_fn(self, &receiver, &args);
                match result {
                    Ok(result) => {
                        self.stack.truncate(receiver_idx);
                        self.stack.push(result);
                        Ok(())
                    }
                    Err(err) => Err(PhError::VMError {
                        message: format!("{}", err),
                        stack_trace: self.format_stack_trace(format!("Error calling primitive function: {}", err)),
                    }),
                }
            }
            MethodKind::Closure(closure) => {
                let new_frame = phref_new(CallFrame::new(closure.clone(), callee.to_context(), 0, self.stack.len() - arity - 1));
                self.frames.push(new_frame);
                Ok(())
            }
        }
    }

    pub fn run(&mut self) -> PhResult<Value> {
        macro_rules! binary_op {
            ($op:tt, $selector:expr) => {
                {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if a.is_number() && b.is_number() {
                        let result = a.as_number().map_err(PhError::StringError)? $op b.as_number().map_err(PhError::StringError)?;
                        self.stack.push(Value::Number(result));
                    } else {
                        let selector = self.interner.intern($selector);
                        self.stack.push(a);
                        self.stack.push(b);
                        let callee = self.stack[self.stack.len() - 2].clone();
                        if let Some(method) = callee.lookup_method(self, selector) {
                            self.call_method(&callee, method, 1)?;
                        } else {
                            let selector_name = self.resolve_symbol(selector);
                            let receiver = &self.stack[self.stack.len() - 2];
                            return Err(PhError::VMError {
                                message: format!("Method '{selector_name}' not found for value {receiver}."),
                                stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for value {receiver}.")),
                            });
                        }
                    }
                }
            };
            ($op:tt, $type:ty, $selector:expr) => {
                {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if a.is_number() && b.is_number() {
                        self.stack.push(Value::Bool((a.as_number().map_err(PhError::StringError)? $op b.as_number().map_err(PhError::StringError)?)));
                    } else {
                        let selector = self.interner.intern($selector);
                        self.stack.push(a);
                        self.stack.push(b);
                        let callee = self.stack[self.stack.len() - 2].clone();
                        if let Some(method) = callee.lookup_method(self, selector) {
                            self.call_method(&callee, method, 1)?;
                        } else {
                            let selector_name = self.resolve_symbol(selector);
                            let receiver = &self.stack[self.stack.len() - 2];
                            return Err(PhError::VMError {
                                message: format!("Method '{selector_name}' not found for value {receiver}."),
                                stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for value {receiver}.")),
                            });
                        }
                    }
                }
            }
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
                                return Err(PhError::VMError {
                                    message: format!("Undefined variable '{}'.", name),
                                    stack_trace: self.format_stack_trace(format!("Undefined variable '{}'.", name)),
                                });
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
                            return Err(PhError::VMError {
                                message: format!("Undefined variable '{}'.", name),
                                stack_trace: self.format_stack_trace(format!("Undefined variable '{}'.", name)),
                            });
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
                        return Err(PhError::VMError {
                            message: format!("Local variable slot {} out of bounds", slot),
                            stack_trace: self.format_stack_trace(format!("Local variable slot {} out of bounds", slot)),
                        });
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
                        return Err(PhError::VMError {
                            message: format!("Local variable slot {} out of bounds", slot),
                            stack_trace: self.format_stack_trace(format!("Local variable slot {} out of bounds", slot)),
                        });
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
                            return Err(PhError::VMError {
                                message: "Superclass must be a class.".to_string(),
                                stack_trace: self.format_stack_trace("Superclass must be a class.".to_string()),
                            });
                        }
                    }
                }
                Bytecode::Method(selector_idx, is_static) => {
                    let selector_val = &chunk.constants[selector_idx as usize];
                    if let Value::Symbol(selector) = selector_val {
                        let method_val = self.stack.pop().unwrap();
                        let class_val = self.stack.last().unwrap();
                        let selector_name = self.resolve_symbol(*selector);
                        if let (Value::Method(method_obj), Value::Class(class_obj)) = (method_val, class_val) {
                            debug!("Adding method {} to class {}", selector_name, class_obj.borrow().name_copy());
                            method_obj.borrow_mut().set_holder(PhRef::downgrade(class_obj));
                            if is_static {
                                class_obj.borrow().class().borrow_mut().add_method(*selector, method_obj);
                            } else {
                                class_obj.borrow_mut().add_method(*selector, method_obj);
                            }
                        } else {
                            return Err(PhError::VMError {
                                message: "VM Error: Invalid types for method definition.".to_string(),
                                stack_trace: self.format_stack_trace("VM Error: Invalid types for method definition.".to_string()),
                            });
                        }
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
                            return Err(PhError::VMError {
                                message: format!("Only instances can have fields."),
                                stack_trace: self.format_stack_trace(format!("Only instances can have fields.")),
                            });
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
                            return Err(PhError::VMError {
                                message: format!("Only instances can have fields."),
                                stack_trace: self.format_stack_trace(format!("Only instances can have fields.")),
                            });
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
                        self.call_method(&receiver, method, arity)?;
                    } else {
                        let selector_name = self.resolve_symbol(selector_sym);
                        return Err(PhError::VMError {
                            message: format!("Method '{selector_name}' not found for value {receiver}."),
                            stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for value {receiver}.")),
                        });
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
                Bytecode::Add => binary_op!(+, "+(_)"),
                Bytecode::Subtract => binary_op!(-, "-(_)"),
                Bytecode::Multiply => binary_op!(*, "*(_)"),
                Bytecode::Divide => binary_op!(/, "/(_)"),
                Bytecode::Modulo => binary_op!(%, "%(_)"),
                Bytecode::Equal => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Bool(a == b));
                }
                Bytecode::NotEqual => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Bool(a != b));
                }
                Bytecode::Greater => binary_op!(>, bool, ">( _)"),
                Bytecode::GreaterEqual => binary_op!(>=, bool, ">=(_)"),
                Bytecode::Less => binary_op!(<, bool, "<(_)"),
                Bytecode::LessEqual => binary_op!(<=, bool, "<=(_)"),
                Bytecode::And => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let a_clone = a.clone();
                    let b_clone = b.clone();
                    if let (Value::Bool(a_bool), Value::Bool(b_bool)) = (a, b) {
                        self.stack.push(Value::Bool(a_bool && b_bool));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand types for logical AND: {a_clone:?} and {b_clone:?}"),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand types for logical AND: {a_clone:?} and {b_clone:?}")),
                        });
                    }
                }
                Bytecode::Or => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let a_clone = a.clone();
                    let b_clone = b.clone();
                    if let (Value::Bool(a_bool), Value::Bool(b_bool)) = (a, b) {
                        self.stack.push(Value::Bool(a_bool || b_bool));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand types for logical OR: {a_clone:?} and {b_clone:?}"),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand types for logical OR: {a_clone:?} and {b_clone:?}")),
                        });
                    }
                }
                Bytecode::Negate => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Number(num) = val {
                        self.stack.push(Value::Number(-num));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand type for negation: {val:?}"),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand type for negation: {val:?}")),
                        });
                    }
                }
                Bytecode::Not => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Bool(b) = val {
                        self.stack.push(Value::Bool(!b));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand type for logical NOT: {val:?}"),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand type for logical NOT: {val:?}")),
                        });
                    }
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
