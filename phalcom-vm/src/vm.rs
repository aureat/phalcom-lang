use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::chunk::Chunk;
use crate::class::ClassObject;
use crate::error::PhResult;
use crate::frame::CallFrame;
use crate::interner::{Interner, Symbol};
use crate::method::{MethodKind, MethodObject};
use crate::module::{ModuleObject, CORE_MODULE_NAME};
use crate::nil::NIL;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::MaybeWeak::Strong;
use phalcom_common::{phref_new, PhRef};
use std::collections::HashMap;
use std::time::Instant;

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<Symbol, Value>,
    pub modules: HashMap<Symbol, PhRef<ModuleObject>>,
    pub(crate) interner: Interner,
    pub start_time: Instant,
    pub universe: Universe,
}

impl VM {
    /// Creates a new VM, ready to execute the top-level script method.
    pub fn new(top_level_method: PhRef<MethodObject>) -> Self {
        let mut interner = Interner::with_capacity(100);

        let universe = Universe::new();

        let mut vm = Self {
            frames: Vec::with_capacity(256),
            stack: Vec::with_capacity(1024),
            globals: HashMap::new(),
            interner,
            start_time: Instant::now(),
            modules: HashMap::new(),
            universe,
        };

        // The top-level script runs in its own call frame.
        let frame = CallFrame {
            method: top_level_method,
            ip: 0,
            stack_offset: 0,
        };
        vm.frames.push(frame);

        vm
    }

    // Helper methods to get the current context
    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    pub fn create_class(
        &mut self,
        name: &str,
        superclass: Option<PhRef<ClassObject>>,
    ) -> PhRef<ClassObject> {
        let metaclass_class_ptr = self.classes.metaclass_class.clone();
        let class = ClassObject::new(name, Strong(metaclass_class_ptr), superclass);

        phref_new(class)
    }

    fn module_from_str(&mut self, name: &str) -> PhRef<ModuleObject> {
        let sym = self.interner.intern(name);
        self.module(sym)
    }

    fn module(&mut self, module_sym: Symbol) -> PhRef<ModuleObject> {
        if let Some(m) = self.modules.get(&module_sym) {
            return m.clone();
        }

        let m = phref_new(ModuleObject::new(module_sym));
        self.modules.insert(module_sym, m.clone());
        m
    }

    fn get_global(&self, module_sym: Symbol, name_sym: Symbol) -> Option<Value> {
        self.modules
            .get(&module_sym)
            .and_then(|m| m.borrow().get(name_sym))
    }

    fn define_global(
        &mut self,
        module_sym: Symbol,
        name_sym: Symbol,
        val: Value,
    ) -> PhResult<usize> {
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
                    self.classes.$field.borrow().name_str(),
                    Value::Class(self.classes.$field.clone()),
                );
            };
        }

        add(
            self.classes.object_class.borrow().name_str(),
            Value::Class(self.classes.object_class.clone()),
        );

        add_class!(object_class);
        add_class!(class_class);
        add_class!(metaclass_class);
        add_class!(number_class);
        add_class!(string_class);
        add_class!(bool_class);
        add_class!(nil_class);
        add_class!(method_class);

        self.define_global(core_sym, core_sym, Value::Module(core_mod))
            .ok();
    }

    pub fn run(&mut self) -> Result<Value, String> {
        loop {
            // If there are no frames left, execution is complete.
            if self.frames.is_empty() {
                // The final result is the last value on the stack, or Nil.
                return Ok(self.stack.pop().unwrap_or(Value::Nil));
            }

            // Get a reference to the current frame's chunk.
            // We clone the Gc<Method> to satisfy the borrow checker, which is cheap.
            let method = self.current_frame().method.clone();

            let method_borrowed = method.borrow();
            let chunk = match &method_borrowed.kind {
                MethodKind::Closure(chunk) => chunk,
                MethodKind::Primitive(_) => {
                    return Err(
                        "VM Error: Attempted to execute bytecode from a native method.".to_string(),
                    );
                }
            };

            // Fetch the next instruction.
            let opcode = chunk.code[self.current_frame().ip];
            self.current_frame_mut().ip += 1;

            // --- Main Dispatch Loop ---
            match opcode {
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
                    if let Value::String(name) = name_val {
                        // The value to be assigned is on top of the stack.
                        // We clone it because `globals` takes ownership.
                        self.globals
                            .insert(name.to_string(), self.stack.last().unwrap().clone());
                        // Important: Defining a global should not leave the value on the stack.
                        self.stack.pop();
                    }
                }

                Bytecode::GetGlobal(idx) => {
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::String(name) = name_val {
                        if let Some(value) = self.globals.get(&**name) {
                            self.stack.push(value.clone());
                        } else {
                            return Err(format!("Undefined variable '{}'.", name));
                        }
                    }
                }

                Bytecode::SetGlobal(idx) => {
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::String(name) = name_val {
                        // The value to be assigned is on top of the stack.
                        if let Some(value) = self.stack.last() {
                            self.globals.insert(name.to_string(), value.clone());
                        } else {
                            return Err(format!("No value to assign to '{}'.", name));
                        }
                    }
                }

                Bytecode::Class(idx) => {
                    let name_val = &chunk.constants[idx as usize];
                    if let Value::String(name) = name_val {
                        let superclass = self.stack.pop().unwrap();
                        if let Value::Class(superclass_obj) = superclass {
                            let new_class = self.create_class(name.as_str(), Some(superclass_obj));
                            self.stack.push(Value::Class(new_class));
                        } else {
                            return Err("Superclass must be a class.".to_string());
                        }
                    }
                }

                Bytecode::Method(idx) => {
                    let selector_val = &chunk.constants[idx as usize];
                    if let Value::String(selector) = selector_val {
                        let method_val = self.stack.pop().unwrap();
                        let class_val = self.stack.last().unwrap(); // Class is still on the stack
                        if let (Value::Method(method_obj), Value::Class(class_obj)) =
                            (method_val, class_val)
                        {
                            class_obj.borrow_mut().add_method(&selector, method_obj);
                        } else {
                            return Err(
                                "VM Error: Invalid types for method definition.".to_string()
                            );
                        }
                    }
                }

                Bytecode::Send(arity, selector_idx) => {
                    let selector_val = &chunk.constants[selector_idx as usize];
                    let arity = arity as usize;

                    if let Value::String(selector) = selector_val {
                        // The receiver is under the arguments on the stack.
                        let receiver_idx = self.stack.len() - 1 - arity;
                        let receiver = self.stack[receiver_idx].clone();

                        let args = &self.stack[receiver_idx + 1..];
                        let args_copied: Vec<Value> = args.to_vec();

                        // Perform dynamic dispatch: lookup the method on the receiver's class.
                        let send_result = self.do_send(&receiver, selector, &args_copied);
                    } else {
                        return Err(format!(
                            "VM Error: Expected a string selector, got {selector_val:?}.",
                        ));
                    }
                }

                Bytecode::Return => {
                    // The return value is on top of the stack.
                    let return_value = self.stack.pop().unwrap_or(Value::Nil);

                    // Pop the current frame.
                    let frame = self.frames.pop().unwrap();

                    // If we just popped the very last frame, we're done.
                    if self.frames.is_empty() {
                        return Ok(return_value);
                    }

                    // Discard the stack window used by the completed function.
                    self.stack.truncate(frame.stack_offset);

                    // Push the return value onto the caller's stack.
                    self.stack.push(return_value);
                }

                Bytecode::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow during addition")?;
                    let a = self.stack.pop().ok_or("Stack underflow during addition")?;

                    if a.is_number() && b.is_number() {
                        let result = a.as_number()? + b.as_number()?;
                        self.stack.push(Value::Number(result));
                    } else {
                        let selector = "+:".to_string();
                        let receiver = a;
                        let args = vec![b];

                        let send_result = self.do_send(&receiver, &selector, &args);
                        match send_result {
                            Ok(value) => self.stack.push(value),
                            Err(err) => return Err(err),
                        }
                    }
                }
            }
        }
    }

    pub fn do_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        // Perform dynamic dispatch: lookup the method on the receiver's class.
        if let Some(method) = receiver.lookup_method(&self, selector) {
            match &method.borrow().kind {
                MethodKind::Primitive(native_fn) => {
                    // For native methods, call the Rust function directly.
                    let result = native_fn(self, receiver, args);
                    match result {
                        Ok(value) => {
                            self.stack.push(value);
                            Ok(Value::Nil)
                        }
                        Err(err) => Err(format!("Native method error: {err}")),
                    }
                }
                MethodKind::Closure(_) => {
                    // For Phalcom methods, push a new CallFrame.
                    let frame = CallFrame {
                        method: method.clone(),
                        ip: 0,
                        stack_offset: self.stack.len() - args.len() - 1,
                    };
                    self.frames.push(frame);
                    Ok(Value::Nil) // Placeholder until the method returns
                }
            }
        } else {
            Err(format!(
                "Method '{selector}' not found for value {receiver:?}."
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;
    use crate::chunk::Chunk;
    use crate::method::MethodObject;

    #[test]
    fn test_vm_addition() {
        // --- 2. Manual Chunk Assembly Phase ---
        // We will manually create the bytecode for `10 + 20`, which is treated
        // as the message send `10.__add__(20)`.

        let mut chunk = Chunk::default();

        // The constants needed for this operation:
        // Index 0: The receiver, 10.0
        let string1_idx = chunk.add_constant(Value::string_from_str("hello, "));
        // Index 1: The argument, 20.0
        let string2_idx = chunk.add_constant(Value::string_from_str("world!"));
        // Index 2: The selector for the method call, "__add__:"
        let const_selector_idx = chunk.add_constant(Value::string_from_str("+:"));

        // The sequence of bytecode instructions:
        chunk.code.extend_from_slice(&[
            Bytecode::String(string1_idx),
            Bytecode::Number(string2_idx),
            Bytecode::Send(1, const_selector_idx),
            Bytecode::Return,
        ]);

        // --- 3. Top-Level Method Creation ---
        // The VM starts by executing a top-level script, which is itself a method.
        // We wrap our handcrafted chunk in this method.
        let top_level_method = phref_new(MethodObject {
            kind: MethodKind::Closure(chunk),
            arity: 0,
            parameters: Vec::new(),
        });

        // --- 4. VM Initialization ---
        // Create a new VM, which will set up the initial call frame for our
        // top-level method. We pass our bootstrapped universe to it.
        let mut vm = VM::new(top_level_method);

        // --- 5. Execution ---
        // Run the VM until it finishes (i.e., the last OpReturn is executed).
        // We expect it to succeed.
        let result = vm.run().expect("VM execution failed with an error");

        // --- 6. Assertion ---
        // The final value left on the stack should be the result of the addition.
        let expected = Value::string_from_str("hello, world!");

        assert_eq!(
            result, expected,
            "String addition did not produce the expected result"
        );
    }
}
