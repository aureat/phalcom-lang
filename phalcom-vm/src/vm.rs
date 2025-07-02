use crate::bootstrap;
use crate::bootstrap::bootstrap;
use crate::bytecode::OpCode;
use crate::chunk::Chunk;
use crate::frame::CallFrame;
use crate::method::{MethodKind, MethodObject};
use crate::object::ClassObject;
use crate::universe::Universe;
use crate::value::Value;
use phalcom_common::MaybeWeak::Strong;
use phalcom_common::{PhRef, phref_new};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    universe: Universe,
}

impl VM {
    /// Creates a new VM, ready to execute the top-level script method.
    pub fn new(top_level_method: PhRef<MethodObject>) -> Self {
        let universe = bootstrap(); // Bootstrap the object model
        let mut vm = Self {
            frames: Vec::with_capacity(256),
            stack: Vec::with_capacity(1024),
            globals: HashMap::new(),
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

    // fn current_chunk(&self) -> &Chunk {
    //     if let MethodKind::Bytecode(chunk) = &self.current_frame().method.borrow().kind {
    //         chunk
    //     } else {
    //         panic!("Attempted to get chunk from a native method frame.");
    //     }
    // }

    pub fn create_class(
        &mut self,
        name: &str,
        superclass: Option<PhRef<ClassObject>>,
    ) -> PhRef<ClassObject> {
        let metaclass_class_ptr = self.universe.metaclass_class.clone();
        let class = ClassObject::new(name, Strong(metaclass_class_ptr), superclass);

        phref_new(class)
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
                MethodKind::Bytecode(chunk) => chunk,
                MethodKind::Native(_) => {
                    return Err(
                        "VM Error: Attempted to execute bytecode from a native method.".to_string(),
                    );
                }
            };

            // Fetch the next instruction.
            let (opcode, operand) = chunk.code[self.current_frame().ip];
            self.current_frame_mut().ip += 1;

            // --- Main Dispatch Loop ---
            match opcode {
                OpCode::OpConstant => {
                    let constant = chunk.constants[operand as usize].clone();
                    self.stack.push(constant);
                }

                OpCode::OpPop => {
                    self.stack.pop();
                }

                OpCode::OpDefineGlobal => {
                    let name_val = &chunk.constants[operand as usize];
                    if let Value::String(name) = name_val {
                        // The value to be assigned is on top of the stack.
                        // We clone it because `globals` takes ownership.
                        self.globals
                            .insert(name.to_string(), self.stack.last().unwrap().clone());
                        // Important: Defining a global should not leave the value on the stack.
                        self.stack.pop();
                    }
                }

                OpCode::OpGetGlobal => {
                    let name_val = &chunk.constants[operand as usize];
                    if let Value::String(name) = name_val {
                        if let Some(value) = self.globals.get(&**name) {
                            self.stack.push(value.clone());
                        } else {
                            return Err(format!("Undefined variable '{}'.", name));
                        }
                    }
                }

                OpCode::OpClass => {
                    let name_val = &chunk.constants[operand as usize];
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

                OpCode::OpMethod => {
                    let selector_val = &chunk.constants[operand as usize];
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

                OpCode::OpCall => {
                    // The arity is the operand of the OpCall instruction itself.
                    let arity = operand as usize;

                    // The selector index is in the operand of the *next* instruction in the stream.
                    let selector_idx = chunk.code[self.current_frame().ip].1 as usize;
                    self.current_frame_mut().ip += 1; // Consume the instruction containing the selector index.

                    let selector_val = &chunk.constants[selector_idx];
                    if let Value::String(selector) = selector_val {
                        // The receiver is under the arguments on the stack.
                        let receiver_idx = self.stack.len() - 1 - arity;
                        let receiver = self.stack[receiver_idx].clone();

                        // Perform dynamic dispatch: lookup the method on the receiver's class.
                        if let Some(method) = receiver.lookup_method(&self.universe, &selector) {
                            match &method.borrow().kind {
                                MethodKind::Native(native_fn) => {
                                    // For native methods, call the Rust function directly.
                                    let args = &self.stack[receiver_idx + 1..];
                                    let args_copied: Vec<Value> = args.to_vec();
                                    let result = native_fn(self, &receiver, &*args_copied)?;

                                    // Pop receiver and args, then push the result.
                                    self.stack.truncate(receiver_idx);
                                    self.stack.push(result);
                                }
                                MethodKind::Bytecode(_) => {
                                    // For Phalcom methods, push a new CallFrame.
                                    let frame = CallFrame {
                                        method: method.clone(),
                                        ip: 0,
                                        stack_offset: receiver_idx,
                                    };
                                    self.frames.push(frame);
                                }
                            }
                        } else {
                            return Err(format!(
                                "Method '{selector}' not found for value {selector:?}. {receiver:?}",
                            ));
                        }
                    }
                }

                OpCode::OpReturn => {
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::bootstrap;
    use crate::bytecode::OpCode;
    use crate::chunk::Chunk;
    use crate::method::MethodObject;
    use std::arch::aarch64::vaba_s8;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_vm_addition() {
        // --- 2. Manual Chunk Assembly Phase ---
        // We will manually create the bytecode for `10 + 20`, which is treated
        // as the message send `10.__add__(20)`.

        let mut chunk = Chunk::default();

        // The constants needed for this operation:
        // Index 0: The receiver, 10.0
        let const_10_idx = chunk.add_constant(Value::Number(10.0));
        // Index 1: The argument, 20.0
        let const_20_idx = chunk.add_constant(Value::Number(20.0));
        // Index 2: The selector for the method call, "__add__:"
        let const_selector_idx = chunk.add_constant(Value::String(Rc::new("__add__:".to_string())));

        // The sequence of bytecode instructions:
        chunk.code.extend_from_slice(&[
            // Push the receiver (10.0) onto the stack.
            (OpCode::OpConstant, const_10_idx),
            // Push the argument (20.0) onto the stack.
            (OpCode::OpConstant, const_20_idx),
            // Call the method. OpCall is followed by two bytes:
            // 1. Arity (number of arguments)
            // 2. Index of the selector string in the constant pool
            (OpCode::OpCall, 1),                      // Arity is 1
            (OpCode::OpConstant, const_selector_idx), // Using OpConstant's operand byte for selector index
            // Return the result from the top of the stack.
            (OpCode::OpReturn, 0), // Operand is unused for return
        ]);

        // --- 3. Top-Level Method Creation ---
        // The VM starts by executing a top-level script, which is itself a method.
        // We wrap our handcrafted chunk in this method.
        let top_level_method = PhRef::new(RefCell::new(MethodObject {
            kind: MethodKind::Bytecode(chunk),
            arity: 0,
            parameters: Vec::new(),
        }));

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
        let expected = Value::Number(30.0);
        assert_eq!(result, expected, "The result of 10 + 20 should be 30.0");
    }
}
