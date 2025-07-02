// The set of instructions for our VM. This is the language the compiler "speaks".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    /// Pushes a constant from the chunk's pool onto the stack.
    /// Operand: index of the constant.
    OpConstant,
    /// Pops a value off the stack.
    OpPop,

    // --- Variable Opcodes ---
    /// Defines a new global variable.
    /// Operand: index of the variable name (a string constant).
    OpDefineGlobal,
    /// Pushes the value of a global variable onto the stack.
    /// Operand: index of the variable name.
    OpGetGlobal,

    // --- Object Model Opcodes ---
    /// Creates a new class.
    /// Operand: index of the class name (a string constant).
    OpClass,
    /// Attaches a method to the class on top of the stack.
    /// Operand: index of the method selector (a string constant).
    OpMethod,

    // --- Function/Method Opcodes ---
    /// Calls a method.
    /// Operands: index of selector constant, number of arguments.
    OpCall,
    /// Returns a value from the current method.
    OpReturn,
}
