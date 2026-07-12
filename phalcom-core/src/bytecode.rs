// The set of instructions for our VM. This is the language the compiler "speaks".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    /// Pushes a constant from the constant pool onto the stack.
    /// 0: index in the constant pool.
    Constant(u16),

    /// Pushes the `None` singleton — the surface absence value
    /// ([ADR-0007](../../../docs/adr/0007-option-some-none.md)) — onto the
    /// stack. It **never** pushes the private [`crate::value::Value::Nil`]
    /// sentinel ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)).
    ///
    /// Emitted both to seed an uninitialized slot (e.g. a `var x` with no
    /// initializer) and as the result of sacred-inlined one-armed control flow
    /// (`ifTrue`/`ifFalse`/`whileTrue`). Because such a result can flow straight
    /// into an argument or `print` without crossing a read boundary, this opcode
    /// yields `None` directly rather than a raw sentinel that a later read would
    /// surface. There is **no surface syntax** for it: U6 removed the `nil`
    /// literal. The raw `Value::Nil` sentinel is never pushed by any opcode — it
    /// only backs unread allocator storage and is surfaced to `None` at reads
    /// (the `Get*` handlers and the `Return` default in `vm.rs`), so it can
    /// never reach user code (Invariant 4).
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

    /// Sets the value of a global variable.
    /// 0: The index of the variable's name in the constant pool.
    SetGlobal(u16),

    /// Pushes the value of an instance field onto the stack.
    /// 0: The slot offset of the field in the receiver's slots array (ADR-0011).
    GetField(u16),

    /// Sets the value of an instance field of the receiver.
    /// 0: The slot offset of the field in the receiver's slots array (ADR-0011).
    SetField(u16),

    /// Pushes the receiver (`self`) of the current frame onto the stack.
    GetSelf,

    /// Calls a method directly on a receiver, bypassing property lookup.
    /// 0: number of arguments
    /// 1: index of selector constant
    Invoke(u8, u16),

    /// Sends `selector` starting the method walk **above** a statically-known
    /// class, with the original receiver (`self`) — the lowering of a
    /// `super.sel(…)` send (method-lookup.md §1.14, U-INH, ADR-0040).
    ///
    /// 0: number of arguments.
    /// 1: constant-pool index of the selector [`Symbol`](crate::interner::Symbol).
    /// 2: constant-pool index of the **defining class name**
    ///    [`Symbol`](crate::interner::Symbol) — the class whose body is being
    ///    compiled. The class object does not exist at compile time, so its
    ///    *name* is baked and resolved to a class handle at dispatch (the same
    ///    global lookup as [`Bytecode::GetGlobal`]).
    ///
    /// The **defining class** is baked in (not its superclass, DEC-INH-B) so the
    /// VM computes `defining.superclass` at dispatch time; this stays correct
    /// under a future `superclass=` mutation (U13). The walk begins at
    /// `defining.superclass` on the instance side; for a super-*construct*
    /// (the selector encodes a `SignatureKind::Initializer`) it also considers
    /// the superclass's metaclass chain, where constructors are installed
    /// (U-INH §3.5). The receiver stays the original `self`, so an overridden
    /// method runs its *super*'s definition against the same instance. A walk
    /// that exhausts the chain routes to the same `doesNotUnderstand(_:)` →
    /// surface `MessageNotUnderstood` path as an ordinary [`Bytecode::Invoke`]
    /// miss — never a panic. Unlike `Invoke`, the start class is a static
    /// per-call-site constant, so a `SuperSend` cache key differs from a
    /// receiver-polymorphic one; this first cut is uncached (DEC-INH-F).
    SuperSend(u8, u16, u16),

    /// Creates a new class.
    /// 0: index of class name in constant pool.
    Class(u16),

    /// Attaches a method to the class on top of the stack.
    /// 0: index of method selector in constant pool.
    /// 1: is_static flag
    Method(u16, bool),

    /// Returns a value from the current method.
    Return,

    /// Performs a **non-local return** from a block body: unwinds to the block's
    /// lexically-enclosing method activation (its *home frame*) rather than just
    /// the block's own activation ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md),
    /// blocks.md §5).
    ///
    /// Emitted (in place of [`Bytecode::Return`]) for a `return` written inside a
    /// braced block literal; a `return` in a method body still compiles to
    /// [`Bytecode::Return`]. It carries **no operand** — the unwind target is the
    /// executing frame's [`home_frame_token`](crate::frame::CallFrame::home_frame_token),
    /// stamped by [`block_call`](crate::primitive::block::block_call) when the
    /// block was invoked, not encoded in the instruction stream.
    ///
    /// Unlike [`Bytecode::Return`], its VM handler does **not** yield a value out
    /// of the current `run_until` invocation (there may be several nested
    /// `run_until`/`block_call` Rust frames between where this executes and the
    /// home frame — see the module docs of [`crate::primitive::block`]). Instead
    /// it eagerly, in one shot, closes escaping upvalues, truncates the value
    /// stack and frame stack down to and including the home frame, and pushes the
    /// return value at the home frame's stack offset; the unmodified top-of-loop
    /// drain check in each nested `run_until` then picks the value up naturally.
    /// If no live frame matches the home token (the home method already
    /// returned), it raises [`RuntimeError::DeadFrameError`](crate::error::RuntimeError::DeadFrameError).
    ReturnNonLocal,

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

    /// Allocates a new instance of a class popped from the stack.
    ///
    /// Used by constructor initializers to allocate the instance on which
    /// the initializer body executes (ADR-0011).
    NewInstance,

    /// Duplicates the top value on the stack.
    Dup,

    /// Pops the top value and pushes it back wrapped in a fresh `Some`
    /// instance ([ADR-0007](../../../docs/adr/0007-option-some-none.md)).
    ///
    /// Emitted by the sacred-selector inliner's one-armed `ifTrue`/`ifFalse`
    /// fast path ([`crate::compiler::inliner`]) to `Some`-lift the taken
    /// arm's value, keeping the inlined fast path observationally identical
    /// to the `bool_if_true`/`bool_if_false` primitive fallback, which
    /// `Some`-wraps the same way
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)
    /// amendment, U-CORE-2). The untaken arm still pushes [`Bytecode::Nil`]
    /// (surfaced to `None`) directly — only the taken arm needs the wrap.
    WrapSome,
}
