// The set of instructions for our VM. This is the language the compiler "speaks".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    /// Pushes a number constant onto the stack.
    /// 0: index in the constant pool.
    Number(u16),

    /// Pushes a string constant onto the stack.
    /// 0: index in the constant pool.
    String(u16),

    /// Pushes the `nil` value onto the stack.
    Nil,

    /// Pushes the boolean value `true` onto the stack.
    True,

    /// Pushes the boolean value `false` onto the stack.
    False,

    /// Pops the top value from the stack.
    Pop,

    /// Defines a new global variable.
    /// 0: identifier index in the symbol table.
    DefineGlobal(u32),

    /// Pushes the value of a global variable onto the stack.
    /// 0: identifier index in the symbol table.
    GetGlobal(u32),

    /// Sets the value of a global variable.
    /// 0: identifier index in the symbol table.
    SetGlobal(u32),

    /// Creates a new class.
    /// 0:
    Class(u16),

    /// Attaches a method to the class on top of the stack.
    /// 0:
    Method(u32),

    /// Performs addition operation.
    Add,

    /// Calls a method.
    /// 0: number of arguments
    /// 1: index of selector constant
    Send(u8, u16),

    /// Returns a value from the current method.
    Return,
}
