use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::class::{lookup_method_in_hierarchy, ClassObject};
use crate::closure::ClosureObject;
use crate::error::{PhError, PhResult};
use crate::frame::CallFrame;
use crate::interner::{Interner, Symbol};
use crate::method::{MethodKind, MethodObject};
use crate::module::{ModuleObject, CORE_MODULE_NAME};
use crate::nil::NIL;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::MaybeWeak::Weak;
use phalcom_common::{phref_new, PhRef, PhWeakRef};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Instant;

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

    /// Returns (or creates) the module for a given name.
    pub fn module_from_str(&mut self, name: &str) -> PhRef<ModuleObject> {
        let sym = self.interner.intern(name);
        self.module(sym)
    }

    /// Runs a module with given closure as entry point.
    pub fn run_module(&mut self, module: PhRef<ModuleObject>, entry: PhRef<ClosureObject>) -> PhResult<Value> {
        let module_sym = module.borrow().symbol();
        self.modules.insert(module_sym, module.clone());
        entry.borrow_mut().module = module;
        self.frames.clear();
        self.stack.clear();
        let frame = phref_new(CallFrame {
            method: entry,
            ip: 0,
            stack_offset: 0,
        });
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

        let metaclass_name = name.to_owned() + ".class";
        let metaclass = self.create_single_class(metaclass_name.as_str(), Some(class_class.clone()));
        metaclass.borrow_mut().set_class_owned(&class_class);
        metaclass.borrow_mut().set_superclass(Some(class_class.clone()));

        let class = self.create_single_class(name, superclass);
        class.borrow_mut().set_class_owned(&metaclass);

        self.classes.insert(self.interner.intern(name), class.clone());
        self.classes.insert(self.interner.intern(metaclass_name.as_str()), metaclass.clone());

        class
    }

    fn module(&mut self, module_sym: Symbol) -> PhRef<ModuleObject> {
        if let Some(m) = self.modules.get(&module_sym) {
            return m.clone();
        }

        let m = phref_new(ModuleObject::new(self, module_sym));
        self.modules.insert(module_sym, m.clone());
        m
    }

    fn define_global(&mut self, module_sym: Symbol, name_sym: Symbol, val: Value) -> PhResult<usize> {
        let module = self.module(module_sym);
        module.borrow().define(name_sym, val)
    }

    pub fn install_core(&mut self) {
        let core_sym = self.interner.intern(CORE_MODULE_NAME);
        let core_mod = self.module_from_str(CORE_MODULE_NAME);

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
        add_class!(system_class);

        self.define_global(core_sym, core_sym, Value::Module(core_mod)).ok();
    }

    fn call_method(&mut self, method: PhRef<MethodObject>, arity: usize) -> PhResult<()> {
        match &method.borrow().kind {
            MethodKind::Primitive(native_fn) => {
                let receiver_idx = self.stack.len() - 1 - arity;
                let receiver = self.stack[receiver_idx].clone();
                let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                let result = native_fn(self, &receiver, &args)?;
                self.stack.truncate(receiver_idx);
                self.stack.push(result);
                Ok(())
            }
            MethodKind::Closure(closure) => {
                let new_frame = phref_new(CallFrame {
                    method: closure.clone(),
                    ip: 0,
                    stack_offset: self.stack.len() - arity - 1,
                });
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
                        if let Some(method) = self.stack[self.stack.len() - 2].lookup_method(self, selector) {
                            self.call_method(method, 1)?;
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
                        if let Some(method) = self.stack[self.stack.len() - 2].lookup_method(self, selector) {
                            self.call_method(method, 1)?;
                        } else {
                            let selector_name = self.resolve_symbol(selector);
                            let receiver = &self.stack[self.stack.len() - 2];
                            return Err(PhError::VMError {
                                message: format!("Method '{selector_name}' not found for value {receiver:?}."),
                                stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for value {receiver:?}.")),
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
            let closure = frame.method.clone();
            let chunk = &closure.borrow().callable.chunk;
            let opcode = chunk.code[frame.ip];
            println!("[VM] Executing opcode: {:?}", opcode);
            frame.ip += 1;
            drop(frame);

            match opcode {
                Bytecode::GetProperty(idx) => {
                    let property_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(property_sym) = property_val {
                        let receiver = self.stack.last().unwrap().clone();
                        if let Value::Instance(instance_obj) = &receiver {
                            if let Some(field_value) = instance_obj.borrow().fields.get(self.resolve_symbol(*property_sym)) {
                                self.stack.pop(); // Pop receiver
                                self.stack.push(field_value.clone());
                                continue;
                            }
                        }
                        if let Some(method) = receiver.lookup_method(self, *property_sym) {
                            self.call_method(method, 0)?;
                        } else {
                            let selector_name = self.resolve_symbol(*property_sym);
                            return Err(PhError::VMError {
                                message: format!("Property or method '{selector_name}' not found for value {receiver:?}."),
                                stack_trace: self.format_stack_trace(format!("Property or method '{selector_name}' not found for value {receiver:?}.")),
                            });
                        }
                    }
                }
                Bytecode::SetProperty(idx) => {
                    let property_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(property_sym) = property_val {
                        let value_to_assign = self.stack.pop().ok_or("Stack underflow on property assignment")?;
                        let receiver = self.stack.pop().ok_or("Stack underflow on property assignment")?;

                        if let Value::Instance(instance_obj) = &receiver {
                            instance_obj
                                .borrow_mut()
                                .fields
                                .insert(self.resolve_symbol(*property_sym).to_string(), value_to_assign.clone());
                            self.stack.push(value_to_assign);
                            continue;
                        }

                        let setter_name = format!("{}=", self.resolve_symbol(*property_sym));
                        let setter_selector = self.interner.intern(&setter_name);
                        self.stack.push(receiver);
                        self.stack.push(value_to_assign);
                        if let Some(method) = self.stack[self.stack.len() - 2].lookup_method(self, setter_selector) {
                            self.call_method(method, 1)?;
                        } else {
                            return Err(PhError::VMError {
                                message: format!("Setter method '{setter_name}' not found."),
                                stack_trace: self.format_stack_trace(format!("Setter method '{setter_name}' not found.")),
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
                Bytecode::Constant(idx) => {
                    let constant = chunk.constants[idx as usize].clone();
                    println!("[VM] Pushing constant: {:?}", constant);
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
                            let core_module = self.module(core_module_sym);
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
                        if let (Value::Method(method_obj), Value::Class(class_obj)) = (method_val, class_val) {
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
                Bytecode::Invoke(arity, selector_idx) => {
                    let selector_val = &chunk.constants[selector_idx as usize];
                    let arity = arity as usize;
                    let receiver_idx = self.stack.len() - 1 - arity;
                    let receiver = self.stack[receiver_idx].clone();

                    let selector_sym = selector_val.as_symbol().unwrap();

                    if let Value::Class(class_obj) = &receiver {
                        // if let Some(method) = class_obj.borrow().get_method(selector_val.as_symbol().unwrap()) {
                        //     self.call_method(method, arity)?;
                        // } else if let Some(method) = class_obj.borrow().class().borrow().get_method(selector_val.as_symbol().unwrap()) {
                        //     self.call_method(method, arity)?;
                        // } else {
                        //     let selector_name = self.resolve_symbol(selector_val.as_symbol().unwrap());
                        //     return Err(PhError::VMError {
                        //         message: format!("Method '{selector_name}' not found for class {receiver}."),
                        //         stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for class {receiver}.")),
                        //     });
                        // }

                        // if let Some(method) = class_obj.borrow().get_method(selector_sym) {
                        //     self.call_method(method, arity)?;
                        // } else {
                        let metaclass = class_obj.borrow().class();
                        if let Some(method) = lookup_method_in_hierarchy(metaclass, selector_sym) {
                            self.call_method(method, arity)?;
                        } else {
                            let selector_name = self.resolve_symbol(selector_sym);
                            return Err(PhError::VMError {
                                message: format!("Method '{selector_name}' not found for class {receiver}."),
                                stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for class {receiver}.")),
                            });
                        }
                        // }
                    } else if let Some(method) = receiver.lookup_method(self, selector_val.as_symbol().map_err(PhError::StringError)?) {
                        self.call_method(method, arity)?;
                    } else {
                        let selector_name = self.resolve_symbol(selector_val.as_symbol().unwrap());
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
        }
    }

    fn format_stack_trace(&self, error_message: String) -> String {
        let mut trace = String::new();
        trace.push_str(&format!("Error: {}\n", error_message));
        for frame_ref in self.frames.iter().rev() {
            let frame = frame_ref.borrow();
            let closure = frame.method.borrow();
            let module_name = closure.module.borrow().name.borrow().value();
            let method_name = self.resolve_symbol(closure.callable.name_sym);
            trace.push_str(&format!("  at <{}>.{}\n", module_name, method_name));
        }
        trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;
    use crate::chunk::Chunk;

    #[test]
    fn test_vm_addition() {
        let mut vm = VM::new();
        let module = vm.module_from_str("test");
        let mut chunk = Chunk::default();
        let string1_idx = chunk.add_constant(Value::string_from_str("hello, "));
        let string2_idx = chunk.add_constant(Value::string_from_str("world!"));
        let selector_sym = vm.interner.intern("+(_)");
        let const_selector_idx = chunk.add_constant(Value::Symbol(selector_sym));
        chunk.add_instruction(Bytecode::Constant(string1_idx));
        chunk.add_instruction(Bytecode::Constant(string2_idx));
        chunk.add_instruction(Bytecode::Invoke(1, const_selector_idx));
        chunk.add_instruction(Bytecode::Return);
        use crate::callable::Callable;
        let entry = phref_new(ClosureObject {
            callable: Callable {
                chunk,
                max_slots: 0,
                num_upvalues: 0,
                arity: 0,
                name_sym: vm.interner.intern("test_vm_addition"),
            },
            module: module.clone(),
            upvalues: Vec::new(),
        });
        let result = vm.run_module(module, entry).expect("VM execution failed");
        let expected = Value::string_from_str("hello, world!");
        assert_eq!(result, expected, "String addition failed");
    }

    #[test]
    fn test_global_variable_definition_and_assignment() {
        let mut vm = VM::new();
        let module = vm.module_from_str("test_globals");
        let mut chunk = Chunk::default();
        let x_sym = vm.interner.intern("x");
        let x_idx = chunk.add_constant(Value::Symbol(x_sym));
        let ten_idx = chunk.add_constant(Value::Number(10.0));
        chunk.add_instruction(Bytecode::Constant(ten_idx));
        chunk.add_instruction(Bytecode::DefineGlobal(x_idx));
        chunk.add_instruction(Bytecode::GetGlobal(x_idx));
        let twenty_idx = chunk.add_constant(Value::Number(20.0));
        chunk.add_instruction(Bytecode::Constant(twenty_idx));
        chunk.add_instruction(Bytecode::SetGlobal(x_idx));
        chunk.add_instruction(Bytecode::GetGlobal(x_idx));
        chunk.add_instruction(Bytecode::Return);
        use crate::callable::Callable;
        let entry = phref_new(ClosureObject {
            callable: Callable {
                chunk,
                max_slots: 0,
                num_upvalues: 0,
                arity: 0,
                name_sym: vm.interner.intern("test_global_variable_definition_and_assignment"),
            },
            module: module.clone(),
            upvalues: Vec::new(),
        });
        let result = vm.run_module(module, entry).expect("VM execution failed");
        assert_eq!(result, Value::Number(20.0), "Global var assignment failed");
    }
}
