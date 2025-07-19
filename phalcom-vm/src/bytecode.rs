// The set of instructions for our VM. This is the language the compiler "speaks".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    /// Pushes a constant from the constant pool onto the stack.
    /// 0: index in the constant pool.
    Constant(u16),

    

    /// Pushes the `nil` value onto the stack.
    Nil,

    /// Pushes the boolean value `true` onto the stack.
    True,

    /// Pushes the boolean value `false` onto the stack.
    False,

    /// Pops the top value from the stack.
    Pop,

    /// Defines a new global variable.
    /// 0: The index of the variable's name in the constant pool.
    DefineGlobal(u16),

    /// Pushes the value of a global variable onto the stack.
    /// 0: The index of the variable's name in the constant pool.
    GetGlobal(u16),

    /// Sets the value of an existing global variable.
    /// 0: The index of the variable's name in the constant pool.
    SetGlobal(u16),

    /// Gets a property from an object/value.
    /// 0: index of property name in constant pool.
    GetProperty(u16),

    /// Sets a property on an object/value.
    /// 0: index of property name in constant pool.
    SetProperty(u16),

    /// Calls a method directly on a receiver, bypassing property lookup.
    /// 0: number of arguments
    /// 1: index of selector constant
    Invoke(u8, u16),

    /// Creates a new class.
    /// 0: index of class name in constant pool.
    Class(u16),

    /// Attaches a method to the class on top of the stack.
    /// 0: index of method selector in constant pool.
    /// 1: is_static flag
    Method(u16, bool),

    /// Returns a value from the current method.
    Return,

    // --- Binary Operators ---
    /// Performs addition.
    Add,
    /// Performs subtraction.
    Subtract,
    /// Performs multiplication.
    Multiply,
    /// Performs division.
    Divide,
    /// Performs modulo.
    Modulo,
    /// Performs equality comparison.
    Equal,
    /// Performs inequality comparison.
    NotEqual,
    /// Performs less than comparison.
    Less,
    /// Performs less than or equal comparison.
    LessEqual,
    /// Performs greater than comparison.
    Greater,
    /// Performs greater than or equal comparison.
    GreaterEqual,
    /// Performs logical AND.
    And,
    /// Performs logical OR.
    Or,

    // --- Unary Operators ---
    /// Negates a number.
    Negate,
    /// Performs logical NOT.
    Not,
}

