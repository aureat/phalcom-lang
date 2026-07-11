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

    /// Get local variable by slot index
    GetLocal(u16),

    /// Set local variable by slot index
    SetLocal(u16),

    /// Defines a new global variable.
    /// 0: The index of the variable's name in the constant pool.
    DefineGlobal(u16),

    /// Pushes the value of a global variable onto the stack.
    /// 0: The index of the variable's name in the constant pool.
    GetGlobal(u16),

    SetGlobal(u16),

    GetField(u16),

    SetField(u16),

    /// Pushes the receiver (`self`) of the current frame onto the stack.
    GetSelf,

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

    /// Creates a closure from a template.
    /// 0: constant index of the template Callable/ClosureObject.
    Closure(u16),

    /// Pushes the value of a captured upvalue onto the stack.
    /// 0: index in the closure's upvalue list.
    GetUpvalue(u16),

    /// Sets the value of a captured upvalue to the top value on the stack.
    /// 0: index in the closure's upvalue list.
    SetUpvalue(u16),

    /// Closes any open upvalues pointing to slot index or above.
    /// 0: stack slot index.
    CloseUpvalue(u16),

    /// Unconditional relative jump.
    ///
    /// `offset` is added to the instruction pointer *after* it has already
    /// been advanced past this instruction (the VM increments `ip` before
    /// dispatching), in units of **instructions**, not bytes — a [`Chunk`](crate::chunk::Chunk)
    /// is a `Vec<Bytecode>`, not a byte stream, so there is no fixed-width
    /// encoding to economize; `i32` is used to comfortably cover an inlined
    /// block body of any realistic size without the relative-offset
    /// overflow risk a `clox`-style `i16` would carry
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    Jump(i32),

    /// Pops the top of the stack (expected [`crate::value::Value::Bool`]);
    /// if it is `false`, adds `offset` to `ip` (see [`Bytecode::Jump`] for the
    /// offset convention); if it is `true`, falls through. If the popped
    /// value is not a `Bool` at all, the VM raises a runtime type error —
    /// this is what gives the sacred-selector inliner's per-iteration
    /// `whileTrue` condition check "no truthiness" for free, without a
    /// separate guard opcode ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    JumpIfFalse(i32),

    /// Backward relative jump, semantically identical to [`Bytecode::Jump`]
    /// (`offset` is typically negative). Kept as a distinct opcode purely so
    /// disassembly reads as a loop back-edge, matching `clox`'s `OP_LOOP`
    /// convention.
    Loop(i32),

    /// Deopt guard for the `Bool`-receiver sacred selectors (`ifTrue(_)`,
    /// `ifFalse(_)`, `ifTrue(_)ifFalse(_)`, `and(_)`, `or(_)`).
    ///
    /// Peeks (does **not** pop) the top of the stack: if it is not
    /// [`crate::value::Value::Bool`] **or** the kernel `Bool`'s sacred
    /// methods have been redefined since bootstrap
    /// (`!Universe::bool_sacred_pristine`), adds `offset` to `ip`, landing on
    /// the fallback real-send sequence the compiler emits alongside the
    /// inlined fast path. Otherwise falls through into the inlined code.
    /// This is the override-epoch half of the deopt guard — a type-only
    /// check would be unsound because `and`/`or`/`ifTrue` are ordinary,
    /// overridable methods (control-flow.md §2–3,
    /// [ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    GuardBool(i32),

    /// Deopt guard for the `Block`-receiver sacred selectors (`whileTrue(_)`).
    ///
    /// Unlike [`Bytecode::GuardBool`], this does **not** peek a receiver
    /// value: the receiver of an inlined `whileTrue` is always a
    /// compiler-materialized block literal (`{ cond }.whileTrue { body }`),
    /// so its *type* is already statically Block — the only thing that can
    /// go stale at runtime is whether `Block>>whileTrue(_)` itself has been
    /// redefined since bootstrap. Tests `!Universe::block_sacred_pristine`
    /// and, if dirty, adds `offset` to `ip`
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    GuardBlock(i32),
}
