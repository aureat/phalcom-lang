/// Every [`Bytecode`] variant's name, indexed by [`Bytecode::index`].
///
/// Kept adjacent to the enum so the two move together; [`Bytecode::index`]'s
/// exhaustive `match` is what forces this array to stay in sync when a variant is
/// added (a new variant fails to compile there, not here — see its doc).
pub const BYTECODE_NAMES: [&str; Bytecode::VARIANTS] = [
    "Constant",
    "Nil",
    "True",
    "False",
    "Pop",
    "GetLocal",
    "SetLocal",
    "DefineGlobal",
    "GetGlobal",
    "SetGlobal",
    "GetField",
    "SetField",
    "GetSelf",
    "Invoke",
    "SuperSend",
    "Class",
    "Method",
    "Return",
    "ReturnNonLocal",
    "Closure",
    "GetUpvalue",
    "SetUpvalue",
    "CloseUpvalue",
    "Jump",
    "JumpIfFalse",
    "JumpIfNone",
    "Loop",
    "GuardBool",
    "GuardBlock",
    "NewInstance",
    "Dup",
    "WrapSome",
    "GetLinked",
    "MakeFamily",
    "FinalizeClass",
    "InvokeLocal",
    "InvokeConst",
    "GuardSymbol",
    "BuildTuple",
    "BuildRecord",
    "BeginMapLiteral",
    "MapLiteralInsertUnique",
    "FinishMapLiteral",
    "BeginSetLiteral",
    "SetLiteralAdd",
    "FinishSetLiteral",
    "BuildRange",
    "InvokeCompilerInternal",
    "BuildList",
    "NewArgumentPack",
    "PackPushPositional",
    "PackReserveStaticLabel",
    "PackReserveComputedLabel",
    "PackFillReservedLabel",
    "PackExpandLabels",
    "PackExpandComplete",
    "PackTryExpandTuplePositionals",
    "InvokePack",
    "SuperSendPack",
    "FinishTuplePack",
    "ReserveScratchLocal",
    "ReleaseScratchLocal",
    "BeginListLiteral",
    "ListLiteralAppend",
    "FinishListLiteral",
    "ListTryExpandTuplePositionals",
    "NewRecordLiteralBuilder",
    "RecordLiteralAppend",
    "RecordLiteralExpandLabels",
    "FinishRecordLiteral",
    "MapLiteralExpandLabels",
];

/// Distinguishes exact selector identity from a structural selector pattern
/// at a `MakeFamily` site. The opcode keeps one stable histogram index while
/// removing the old punctuation heuristic from runtime dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilySpecKind {
    Exact,
    Pattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSendKind {
    Method,
    SubscriptGet,
    SubscriptSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackAccess {
    Ordinary,
    CompilerInternal,
}

// The set of instructions for our VM. This is the language the compiler "speaks".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    /// Pushes a constant from the constant pool onto the stack.
    /// 0: index in the constant pool.
    Constant(u16),

    /// Pushes immediate `None` — the surface absence value
    /// ([ADR-0007](../../../docs/adr/accepted/0007-option-some-none.md)) — onto the
    /// stack. It **never** pushes the private [`crate::value::Value::Nil`]
    /// sentinel ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md)).
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
    /// `defining.superclass` on the instance side. Constructor super-sends are
    /// lowered to their hidden initializer selector before this opcode is
    /// emitted, so they follow this same ordinary instance-side path. The
    /// receiver stays the original `self`, so an overridden method runs its
    /// super definition against the same instance. A walk
    /// that exhausts the chain routes to the same `doesNotUnderstand(_:)` →
    /// surface `MessageNotUnderstood` path as an ordinary [`Bytecode::Invoke`]
    /// miss — never a panic. Unlike `Invoke`, the start class is a static
    /// per-call-site constant, so a `SuperSend` cache key differs from a
    /// receiver-polymorphic one; this first cut is uncached (DEC-INH-F).
    SuperSend(u8, u16, u16),

    /// Creates a new class. This is the *whole* truth (U-CLASSCLOSE §5.3,
    /// PDR-0001 ruling 4) — classes are closed, so there is no second
    /// meaning to discriminate at runtime. Stub completion (installing
    /// `.ph` methods onto a Rust-installed kernel class, `core.ph` only) is
    /// a compile-time-resolved case the compiler emits [`Bytecode::Constant`]
    /// for instead; every `Class` the runtime ever executes allocates fresh.
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
    /// the block's own activation ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md),
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
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    Jump(i32),

    /// Pops the top of the stack (expected [`crate::value::Value::Bool`]);
    /// if it is `false`, adds `offset` to `ip` (see [`Bytecode::Jump`] for the
    /// offset convention); if it is `true`, falls through. If the popped
    /// value is not a `Bool` at all, the VM raises a runtime type error —
    /// this is what gives the sacred-selector inliner's per-iteration
    /// `whileTrue` condition check "no truthiness" for free, without a
    /// separate guard opcode ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    JumpIfFalse(i32),

    /// Pops the top of stack; if it is exactly `Value::None`, adds `offset` to
    /// `ip` (see
    /// `Bytecode::Jump` for the offset convention). Otherwise falls through, the popped
    /// value already consumed. Realizes the cursor-protocol end-of-iteration test
    /// (ADR-0048 §2, iteration.md §2) as one direct, non-overridable branch — `for`'s
    /// loop condition is compiler-owned, not a `!=` send.
    JumpIfNone(i32),

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
    /// [ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
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
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
    GuardBlock(i32),

    /// Allocates a new instance of a class popped from the stack.
    ///
    /// Used by constructor initializers to allocate the instance on which
    /// the initializer body executes (ADR-0011).
    NewInstance,

    /// Duplicates the top value on the stack.
    Dup,

    /// Pops the top value and pushes it back with one immediate `Some` layer
    /// ([PDR-0033](../../../pdr/0033-immediate-bounded-option.md)).
    ///
    /// Emitted by the sacred-selector inliner's one-armed `ifTrue`/`ifFalse`
    /// fast path ([`crate::compiler::inliner`]) to `Some`-lift the taken
    /// arm's value, keeping the inlined fast path observationally identical
    /// to the `bool_if_true`/`bool_if_false` primitive fallback, which
    /// `Some`-wraps the same way
    /// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)
    /// amendment, U-CORE-2). The untaken arm still pushes [`Bytecode::Nil`]
    /// (surfaced to `None`) directly — only the taken arm needs the wrap.
    WrapSome,

    /// Pushes one value from the current module's already-materialized linked
    /// read table. Static imports never execute or resolve filesystem paths.
    ///
    /// The operand is an index into the module's runtime linked-read table;
    /// materialization is performed by the program runtime, not this opcode.
    GetLinked(u16),

    /// Builds a bound `::` method-reference **Family** value from a receiver
    /// on the stack. `kind` explicitly identifies whether `spec` is an exact
    /// interned selector or a first-class structural pattern. Construction is
    /// pure capture: it does not resolve or authorize a target method. The
    /// existing dispatcher owns all future-send lookup, fallback, and dNU
    /// behavior.
    ///
    /// [ADR-0047]: ../../../docs/adr/accepted/0047-amend-floor-admit-family-call-router.md
    MakeFamily {
        /// Constant-pool index containing a `Value::Symbol` or selector
        /// pattern object.
        spec: u16,
        /// Representation of the selector specification in that constant.
        kind: FamilySpecKind,
    },

    /// Rebuilds a class's (and its metaclass's) flattened
    /// [`base_names`](crate::heap::ClassObject::base_names) index from its
    /// own directly-defined methods merged with its superclass's
    /// already-finalized index (selectors.md §3.1, U16-Open).
    ///
    /// No operand: it **peeks** (does not pop) the class object left on top
    /// of the stack by the class-body compile — the same value the following
    /// [`Bytecode::DefineGlobal`] binds. Emitted once at the tail of every
    /// class-body compile (`compiler/lib/class_decl.rs`'s `Statement::Class`
    /// lowering, after the member loop, before `DefineGlobal`). Classes are
    /// closed (U-CLASSCLOSE, PDR-0001): this always rebuilds a
    /// brand-new class's index from its own directly-defined methods merged
    /// with its superclass's — there is no reopen case to distinguish it
    /// from. The kernel's native-only rows (no `.ph` class body) are
    /// finalized once directly in `VM::install_core`, mirroring this same
    /// rebuild; `core.ph`'s own stub-completion classes (`Constant`, not
    /// `Class`) still emit this too, once, when their `.ph` body compiles.
    FinalizeClass,

    /// Fused [`Bytecode::GetLocal`] + [`Bytecode::Invoke`] — pushes local `0` and
    /// performs the send in one dispatch (perf-log cut 008).
    ///
    /// 0: local slot index. 1: number of arguments. 2: selector constant index.
    ///
    /// Emitted only by [`Chunk::fuse_superinstructions`](crate::chunk::Chunk::fuse_superinstructions), which rewrites the
    /// `GetLocal` **in place** and leaves the original `Invoke` at `ip + 1` as
    /// dead code, so every `ip`-indexed parallel array (`spans`, `caches`,
    /// `gcaches`) and every jump offset in the chunk stays valid without a
    /// re-layout pass. This opcode therefore advances `ip` by **2**, and reads
    /// its inline cache and span at `ip + 1` — the dead `Invoke`'s slots — so an
    /// error raised here is indistinguishable from the unfused form's.
    InvokeLocal(u16, u8, u16),

    /// Fused [`Bytecode::Constant`] + [`Bytecode::Invoke`] — pushes a constant and
    /// performs the send in one dispatch (perf-log cut 008).
    ///
    /// 0: constant-pool index of the pushed value. 1: number of arguments.
    /// 2: selector constant index.
    ///
    /// Same in-place rewrite, `ip += 2` advance and `ip + 1` cache/span
    /// convention as [`Bytecode::InvokeLocal`].
    InvokeConst(u16, u8, u16),

    /// Requires the top stack value to be a Symbol, without consuming it.
    GuardSymbol,

    /// Finalizes positional values followed by `(Symbol, value)` pairs.
    BuildTuple {
        positional: u16,
        labeled: u16,
    },

    /// Finalizes `(Symbol, value)` pairs into a Record.
    BuildRecord {
        fields: u16,
    },
    BeginMapLiteral,
    MapLiteralInsertUnique,
    FinishMapLiteral,
    BeginSetLiteral,
    SetLiteralAdd,
    FinishSetLiteral,
    /// Builds a native Range directly from present lower-then-upper endpoints.
    BuildRange {
        has_lower: bool,
        has_upper: bool,
        upper_inclusive: bool,
    },

    /// Compiler-emitted send with authority to invoke one internal runtime
    /// helper. Source code cannot emit this opcode; it exists for compiler
    /// lowering such as attribute-retention finalization.
    ///
    /// 0: number of arguments. 1: index of selector constant.
    InvokeCompilerInternal(u8, u16),
    /// Builds a List from the top `n` stack items.
    BuildList(u16),
    NewArgumentPack,
    PackPushPositional,
    PackReserveStaticLabel(u16),
    PackReserveComputedLabel,
    PackFillReservedLabel,
    PackExpandLabels,
    PackExpandComplete,
    PackTryExpandTuplePositionals,
    InvokePack {
        base_name: u16,
        kind: PackSendKind,
        access: PackAccess,
    },
    SuperSendPack {
        base_name: u16,
        defining_class: u16,
    },
    FinishTuplePack,
    /// Inserts one compiler-only local slot into the current frame's local window.
    ///
    /// `slot` is frame-relative. The VM inserts `Value::Nil` at
    /// `frame.stack_offset + slot`, shifting every transient operand at or above
    /// that index upward while preserving order. The compiler emits this only for
    /// scratch locals declared while an enclosing expression has live operands.
    ReserveScratchLocal(u16),
    /// Removes one compiler-only local slot from the current frame's local window.
    ///
    /// `slot` is frame-relative. The VM removes exactly the value at
    /// `frame.stack_offset + slot`, shifting the operand suffix downward while
    /// preserving order. The compiler releases multi-slot scratch regions from
    /// highest slot to lowest slot.
    ReleaseScratchLocal(u16),
    /// Allocates the compiler-owned native List used by a spread-containing
    /// List literal.
    BeginListLiteral,
    /// Appends one value to a compiler-owned List literal builder.
    ListLiteralAppend,
    /// Validates and completes a compiler-owned List literal builder.
    FinishListLiteral,
    /// Tries Unit/Tuple positional projection for a List spread.
    ListTryExpandTuplePositionals,
    /// Allocates the compiler-owned builder used by a spread-containing Record.
    NewRecordLiteralBuilder,
    /// Appends one `(Symbol, value)` pair to a compiler-owned Record builder.
    RecordLiteralAppend,
    /// Snapshots one `**source` and appends its labeled lane to a Record builder.
    RecordLiteralExpandLabels,
    /// Finalizes a compiler-owned Record builder through `finish_record`.
    FinishRecordLiteral,
    /// Snapshots one `**source` and inserts its labeled lane into a Map.
    MapLiteralExpandLabels,
}

impl Bytecode {
    /// Number of distinct opcodes — the length of [`BYTECODE_NAMES`] and of the
    /// histogram in [`opcode_stats`](crate::opcode_stats).
    pub const VARIANTS: usize = 71;

    /// This opcode's dense index in `0..VARIANTS`, for array-indexed bookkeeping.
    ///
    /// The `match` is exhaustive and deliberately not a catch-all: adding a
    /// [`Bytecode`] variant must fail to compile *here*, which is the prompt to
    /// extend [`BYTECODE_NAMES`] and bump [`Self::VARIANTS`] with it. A default arm
    /// would silently alias a new opcode onto an existing bucket and quietly
    /// corrupt every histogram that followed.
    pub fn index(&self) -> usize {
        match self {
            Bytecode::Constant(..) => 0,
            Bytecode::Nil => 1,
            Bytecode::True => 2,
            Bytecode::False => 3,
            Bytecode::Pop => 4,
            Bytecode::GetLocal(..) => 5,
            Bytecode::SetLocal(..) => 6,
            Bytecode::DefineGlobal(..) => 7,
            Bytecode::GetGlobal(..) => 8,
            Bytecode::SetGlobal(..) => 9,
            Bytecode::GetField(..) => 10,
            Bytecode::SetField(..) => 11,
            Bytecode::GetSelf => 12,
            Bytecode::Invoke(..) => 13,
            Bytecode::SuperSend(..) => 14,
            Bytecode::Class(..) => 15,
            Bytecode::Method(..) => 16,
            Bytecode::Return => 17,
            Bytecode::ReturnNonLocal => 18,
            Bytecode::Closure(..) => 19,
            Bytecode::GetUpvalue(..) => 20,
            Bytecode::SetUpvalue(..) => 21,
            Bytecode::CloseUpvalue(..) => 22,
            Bytecode::Jump(..) => 23,
            Bytecode::JumpIfFalse(..) => 24,
            Bytecode::JumpIfNone(..) => 25,
            Bytecode::Loop(..) => 26,
            Bytecode::GuardBool(..) => 27,
            Bytecode::GuardBlock(..) => 28,
            Bytecode::NewInstance => 29,
            Bytecode::Dup => 30,
            Bytecode::WrapSome => 31,
            Bytecode::GetLinked(..) => 32,
            Bytecode::MakeFamily { .. } => 33,
            Bytecode::FinalizeClass => 34,
            Bytecode::InvokeLocal(..) => 35,
            Bytecode::InvokeConst(..) => 36,
            Bytecode::GuardSymbol => 37,
            Bytecode::BuildTuple { .. } => 38,
            Bytecode::BuildRecord { .. } => 39,
            Bytecode::BeginMapLiteral => 40,
            Bytecode::MapLiteralInsertUnique => 41,
            Bytecode::FinishMapLiteral => 42,
            Bytecode::BeginSetLiteral => 43,
            Bytecode::SetLiteralAdd => 44,
            Bytecode::FinishSetLiteral => 45,
            Bytecode::BuildRange { .. } => 46,
            Bytecode::InvokeCompilerInternal(..) => 47,
            Bytecode::BuildList(..) => 48,
            Bytecode::NewArgumentPack => 49,
            Bytecode::PackPushPositional => 50,
            Bytecode::PackReserveStaticLabel(..) => 51,
            Bytecode::PackReserveComputedLabel => 52,
            Bytecode::PackFillReservedLabel => 53,
            Bytecode::PackExpandLabels => 54,
            Bytecode::PackExpandComplete => 55,
            Bytecode::PackTryExpandTuplePositionals => 56,
            Bytecode::InvokePack { .. } => 57,
            Bytecode::SuperSendPack { .. } => 58,
            Bytecode::FinishTuplePack => 59,
            Bytecode::ReserveScratchLocal(..) => 60,
            Bytecode::ReleaseScratchLocal(..) => 61,
            Bytecode::BeginListLiteral => 62,
            Bytecode::ListLiteralAppend => 63,
            Bytecode::FinishListLiteral => 64,
            Bytecode::ListTryExpandTuplePositionals => 65,
            Bytecode::NewRecordLiteralBuilder => 66,
            Bytecode::RecordLiteralAppend => 67,
            Bytecode::RecordLiteralExpandLabels => 68,
            Bytecode::FinishRecordLiteral => 69,
            Bytecode::MapLiteralExpandLabels => 70,
        }
    }

    /// This opcode's name, as it appears in a histogram dump.
    pub fn name(&self) -> &'static str {
        BYTECODE_NAMES[self.index()]
    }
}
