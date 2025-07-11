use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::error::{PhResult, PhError};
use crate::frame::CallFrame;
use crate::interner::{Interner, Symbol};
use crate::method::MethodKind;
use crate::module::{ModuleObject, CORE_MODULE_NAME};
use crate::nil::NIL;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::MaybeWeak::Weak;
use phalcom_common::{phref_new, PhRef, PhWeakRef};
use std::collections::HashMap;
use std::time::Instant;

pub struct VM {
    frames: Vec<PhRef<CallFrame>>,
    stack: Vec<Value>,
    pub modules: HashMap<Symbol, PhRef<ModuleObject>>,
    pub interner: Interner,
    pub start_time: Instant,
    pub universe: Universe,
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

    fn current_frame_mut(&mut self) -> &PhRef<CallFrame> {
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

    fn get_global(&self, module_sym: Symbol, name_sym: Symbol) -> Option<Value> {
        self.modules.get(&module_sym).and_then(|m| m.borrow().get(name_sym))
    }

    fn define_global(&mut self, module_sym: Symbol, name_sym: Symbol, val: Value) -> PhResult<usize> {
        let module = self.module(module_sym);
        module.borrow().define(name_sym, val)
    }

    pub fn install_core(&mut self) {
        let core_sym = self.interner.intern(CORE_MODULE_NAME);
        let core_mod = self.module_from_str(CORE_MODULE_NAME);

        let mut add = |name: &str, val: Value| {
            let name_sym = self.interner.intern(name);
            // ignore re‑definition errors during hot reload
            let _ = core_mod.borrow().define(name_sym, val);
        };

        macro_rules! add_class {
            ($field:ident) => {
                add(
                    self.universe.classes.$field.borrow().name().borrow().as_str(),
                    Value::Class(self.universe.classes.$field.clone()),
                );
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

        self.define_global(core_sym, core_sym, Value::Module(core_mod)).ok();
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
                        let receiver = a;
                        let args = vec![b];
                        let send_result = self.do_send(&receiver, selector, &args);
                        match send_result {
                            Ok(value) => self.stack.push(value),
                            Err(err) => return Err(PhError::VMError {
                                message: format!("Native method error: {}", err),
                                stack_trace: self.format_stack_trace(format!("Native method error: {}", err)),
                            }),
                        }
                    }
                }
            };
            ($op:tt, $type:ty, $selector:expr) => {
                {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if a.is_number() && b.is_number() {
                        self.stack.push(Value::Bool((a.as_number().map_err(PhError::StringError)? $op b.as_number().map_err(PhError::StringError)?) as $type));
                    } else {
                        let selector = self.interner.intern($selector);
                        let receiver = a;
                        let args = vec![b];
                        let send_result = self.do_send(&receiver, selector, &args);
                        match send_result {
                            Ok(value) => self.stack.push(value),
                            Err(err) => return Err(PhError::VMError {
                                message: format!("Native method error: {}", err),
                                stack_trace: self.format_stack_trace(format!("Native method error: {}", err)),
                            }),
                        }
                    }
                }
            }
        }

        loop {
            // If there are no frames left, execution is complete.
            if self.frames.is_empty() {
                // The final result is the last value on the stack, or Nil.
                return Ok(self.stack.pop().unwrap_or(Value::Nil));
            }

            // Prepare current frame for execution
            let frame_ref = self.current_frame().clone();
            let mut frame = frame_ref.borrow_mut();
            // Extract closure and its chunk
            let closure = frame.method.clone();
            let chunk = &closure.borrow().callable.chunk;
            // Fetch and advance instruction pointer
            let opcode = chunk.code[frame.ip];
            frame.ip += 1;
            drop(frame);

            // --- Main Dispatch Loop ---
            match opcode {

                Bytecode::GetProperty(idx) => {
                    let property_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(property_sym) = property_val {
                        let receiver = self.stack.pop().ok_or("Stack underflow on property access")?;
                        println!("[VM] GetProperty: receiver = {:?}, property_sym = {:?}", receiver, property_sym);
                        let value = match self.do_send(&receiver, *property_sym, &[]) {
                            Ok(val) => {
                                println!("[VM] GetProperty: returned value = {:?}", val);
                                val
                            },
                            Err(err) => return Err(PhError::VMError {
                                message: format!("Property access error: {}", err),
                                stack_trace: self.format_stack_trace(format!("Property access error: {}", err)),
                            }),
                        };
                        // self.stack.push(value);
                    }
                }
                Bytecode::Number(idx) | Bytecode::String(idx) => {
                    let constant = chunk.constants[idx as usize].clone();
                    self.stack.push(constant);
                }

                Bytecode::Nil => {
                    self.stack.push(NIL);
                }

                Bytecode::True => {
                    self.stack.push(TRUE);
                }

                Bytecode::False => {
                    self.stack.push(FALSE);
                }

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
                            let name = self.resolve_symbol(*name_sym);
                            return Err(PhError::VMError {
                                message: format!("Undefined variable '{}'.", name),
                                stack_trace: self.format_stack_trace(format!("Undefined variable '{}'.", name)),
                            });
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
                    if let Value::String(name) = name_val {
                        let superclass = self.stack.pop().unwrap();
                        if let Value::Class(superclass_obj) = superclass {
                            let new_class = self.create_class(name.borrow().as_str(), Some(superclass_obj));
                            self.stack.push(Value::Class(new_class));
                        } else {
                            return Err(PhError::VMError {
                            message: "Superclass must be a class.".to_string(),
                            stack_trace: self.format_stack_trace("Superclass must be a class.".to_string()),
                        });
                        }
                    }
                }

                Bytecode::Method(idx) => {
                    let selector_val = &chunk.constants[idx as usize];
                    if let Value::Symbol(selector) = selector_val {
                        let method_val = self.stack.pop().unwrap();
                        let class_val = self.stack.last().unwrap(); // Class is still on the stack
                        if let (Value::Method(method_obj), Value::Class(class_obj)) = (method_val, class_val) {
                            class_obj.borrow_mut().add_method(*selector, method_obj);
                        } else {
                            return Err(PhError::VMError {
                            message: "VM Error: Invalid types for method definition.".to_string(),
                            stack_trace: self.format_stack_trace("VM Error: Invalid types for method definition.".to_string()),
                        });
                        }
                    }
                }

                Bytecode::Call(arity, selector_idx) => {
                    let selector_val = &chunk.constants[selector_idx as usize];
                    let arity = arity as usize;

                    // The receiver is under the arguments on the stack.
                    let receiver_idx = self.stack.len() - 1 - arity;
                    let receiver = self.stack[receiver_idx].clone();

                    let args = &self.stack[receiver_idx + 1..];
                    let args_copied: Vec<Value> = args.to_vec();

                    // Perform dynamic dispatch: lookup the method on the receiver's class.
                    let _ = self.do_send(&receiver, selector_val.as_symbol().map_err(PhError::StringError)?, &args_copied)?;
                }

                Bytecode::Return => {
                    // The return value is on top of the stack.
                    let return_value = self.stack.pop().unwrap_or(Value::Nil);

                    // Pop the current frame and restore caller context.
                    let popped = self.frames.pop().unwrap();

                    // If we just popped the very last frame, we're done.
                    if self.frames.is_empty() {
                        return Ok(return_value);
                    }

                    // Discard the stack window used by the completed function.
                    let popped_ref = popped.borrow();
                    self.stack.truncate(popped_ref.stack_offset);

                    // Push the return value onto the caller's stack.
                    self.stack.push(return_value);
                }

                Bytecode::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow during addition")?;
                    let a = self.stack.pop().ok_or("Stack underflow during addition")?;

                    if a.is_number() && b.is_number() {
                        let result = a.as_number().map_err(PhError::StringError)? + b.as_number().map_err(PhError::StringError)?;
                        self.stack.push(Value::Number(result));
                    } else {
                        let selector = self.interner.intern("+:");
                        let receiver = a;
                        let args = vec![b];

                        let send_result = self.do_send(&receiver, selector, &args);
                        match send_result {
                            Ok(value) => self.stack.push(value),
                            Err(err) => return Err(PhError::VMError {
                                message: format!("Native method error: {}", err),
                                stack_trace: self.format_stack_trace(format!("Native method error: {}", err)),
                            }),
                        }
                    }
                }
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
                Bytecode::Greater => binary_op!(>, bool, ">(_)"),
                Bytecode::GreaterEqual => binary_op!(>=, bool, ">=(_)"),
                Bytecode::Less => binary_op!(<, bool, "<(_)"),
                Bytecode::LessEqual => binary_op!(<=, bool, "<=(_)"),

                Bytecode::And => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let a_clone = a.clone();
                    let b_clone = b.clone();
                    if let (Value::Bool(a_bool), Value::Bool(b_bool)) = (&a, &b) {
                        self.stack.push(Value::Bool(*a_bool && *b_bool));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand types for logical AND: {:?} and {:?}", a_clone, b_clone),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand types for logical AND: {:?} and {:?}", a_clone, b_clone)),
                        });
                    }
                }
                Bytecode::Or => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let a_clone = a.clone();
                    let b_clone = b.clone();
                    if let (Value::Bool(a_bool), Value::Bool(b_bool)) = (&a, &b) {
                        self.stack.push(Value::Bool(*a_bool || *b_bool));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand types for logical OR: {:?} and {:?}", a_clone, b_clone),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand types for logical OR: {:?} and {:?}", a_clone, b_clone)),
                        });
                    }
                }

                Bytecode::Negate => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Number(num) = val {
                        self.stack.push(Value::Number(-num));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand type for negation: {:?}", val),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand type for negation: {:?}", val)),
                        });
                    }
                }
                Bytecode::Not => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Bool(b) = val {
                        self.stack.push(Value::Bool(!b));
                    } else {
                        return Err(PhError::VMError {
                            message: format!("Unsupported operand type for logical NOT: {:?}", val),
                            stack_trace: self.format_stack_trace(format!("Unsupported operand type for logical NOT: {:?}", val)),
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
            trace.push_str(&format!("  at {}.{}\n", module_name, method_name));
        }
        trace
    }

    pub fn do_send(&mut self, receiver: &Value, selector: Symbol, args: &[Value]) -> PhResult<Value> {
        // Perform dynamic dispatch: lookup the method on the receiver's class.
        if let Some(method) = receiver.lookup_method(self, selector) {
            match &method.borrow().kind {
                MethodKind::Primitive(native_fn) => {
                    // For native methods, call the Rust function directly.
                    println!("Calling native method: {}", self.resolve_symbol(selector));
                    let result = native_fn(self, receiver, args);
                    println!("Native method returned: {:?}", result);
                    result.map(|v| { self.stack.push(v); Value::Nil })
                }
                MethodKind::Closure(closure) => {
                    // For Phalcom methods, push a new CallFrame with the closure entry.
                    let new_frame = phref_new(CallFrame {
                        method: closure.clone(),
                        ip: 0,
                        stack_offset: self.stack.len() - args.len() - 1,
                    });
                    self.frames.push(new_frame);
                    Ok(Value::Nil)
                }
            }
        } else {
            let selector_name = self.resolve_symbol(selector);
            Err(PhError::VMError {
                message: format!("Method '{selector_name}' not found for value {receiver:?}."),
                stack_trace: self.format_stack_trace(format!("Method '{selector_name}' not found for value {receiver:?}.")),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;
    use crate::chunk::Chunk;

    #[test]
    fn test_vm_addition() {
        // --- 2. Manual Chunk Assembly Phase ---
        // We will manually create the bytecode for `10 + 20`, which is treated
        // as the message send `10.__add__(20)`.

        let mut vm = VM::new();
        let module = vm.module_from_str("test");

        let mut chunk = Chunk::default();

        // The constants needed for this operation:
        // Index 0: The receiver, 10.0
        let string1_idx = chunk.add_constant(Value::string_from_str("hello, "));
        // Index 1: The argument, 20.0
        let string2_idx = chunk.add_constant(Value::string_from_str("world!"));
        // Index 2: The selector for the method call, "__add__:"
        let selector_sym = vm.interner.intern("+(_)");
        let const_selector_idx = chunk.add_constant(Value::Symbol(selector_sym));

        // The sequence of bytecode instructions:
        chunk.code.extend_from_slice(&[
            Bytecode::String(string1_idx),
            Bytecode::String(string2_idx),
            Bytecode::Call(1, const_selector_idx),
            Bytecode::Return,
        ]);

        // --- 3. Top-Level Closure Creation ---
        // Wrap our handcrafted chunk in a ClosureObject.
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

        // --- 4. VM Initialization and Execution ---
        let result = vm.run_module(module, entry).expect("VM execution failed with an error");

        // --- 6. Assertion ---
        // The final value left on the stack should be the result of the addition.
        let expected = Value::string_from_str("hello, world!");

        assert_eq!(result, expected, "String addition did not produce the expected result");
    }

    #[test]
    fn test_global_variable_definition_and_assignment() {
        let mut vm = VM::new();
        let module = vm.module_from_str("test_globals");

        let mut chunk = Chunk::default();

        // Define a global variable 'x' with initial value 10
        let x_sym = vm.interner.intern("x");
        let x_idx = chunk.add_constant(Value::Symbol(x_sym));
        let ten_idx = chunk.add_constant(Value::Number(10.0));
        chunk.add_instruction(Bytecode::Number(ten_idx));
        chunk.add_instruction(Bytecode::DefineGlobal(x_idx));

        // Get the value of 'x' and push it to stack
        chunk.add_instruction(Bytecode::GetGlobal(x_idx));

        // Assign a new value 20 to 'x'
        let twenty_idx = chunk.add_constant(Value::Number(20.0));
        chunk.add_instruction(Bytecode::Number(twenty_idx));
        chunk.add_instruction(Bytecode::SetGlobal(x_idx));

        // Get the new value of 'x' and push it to stack
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

        let result = vm.run_module(module, entry).expect("VM execution failed with an error");

        assert_eq!(result, Value::Number(20.0), "Global variable assignment did not produce the expected result");
    }
}
