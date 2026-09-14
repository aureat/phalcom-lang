# Phalcom Bytecode Specification

**Status:** Draft 0.1 — normative VM instruction-set specification  
**Specification layer:** Phalcom VM and Internals  
**Implementation baseline:** `aureat/phalcom-lang`, `main`, repository state observed at commit `3d4d1a855337738eb86a056721e8c52f1cf3c18b`  
**Primary implementation references:** `phalcom-core/src/bytecode.rs`, `phalcom-core/src/chunk.rs`, `phalcom-core/src/vm/dispatch.rs`

---

## 1. Scope

This document specifies the canonical logical bytecode instruction set executed by the Phalcom virtual machine.

It defines:

- the executable bytecode unit;
- the abstract execution state required to interpret bytecode;
- instruction-pointer and control-transfer semantics;
- the value-stack discipline;
- constant, metadata, and executable-semantic references;
- the meaning of every instruction in the current Phalcom instruction set;
- ordinary completion, abrupt completion, and non-local control transfer;
- bytecode validity requirements;
- source-location requirements;
- implementation limits that are visible in the current instruction representation;
- the boundary between normative VM semantics and non-normative implementation optimizations.

This document does **not** define a portable serialized bytecode file format. The reference VM currently represents bytecode as a typed instruction vector, not as a byte stream. A future serialized representation may be standardized separately without changing the logical instruction semantics defined here.

The source-language specification remains authoritative for source-level observable semantics. This document is authoritative for the canonical Phalcom VM instruction language and for the behavior of conforming implementations that claim compatibility with that instruction language.

---

## 2. Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **MAY**, and **OPTIONAL** are normative requirements when written in uppercase.

A statement marked **Implementation note** describes the reference implementation and is not, by itself, a portability requirement.

A statement marked **Compiler note** describes lowering performed by the reference compiler and does not require another conforming bytecode producer to emit the same instruction sequence.

---

## 3. Specification layers

Phalcom distinguishes three semantic layers.

| Layer | Authority | Responsibility |
|---|---|---|
| Source-language semantics | Normative | Defines observable meaning of Phalcom programs |
| VM and bytecode semantics | Normative for the canonical VM | Defines executable units, machine transitions, instruction behavior, and VM invariants |
| Reference implementation internals | Descriptive unless explicitly promoted | Defines Rust data structures, caches, allocation strategies, optimizer choices, and engineering details |

An implementation optimization MUST NOT alter the logical effect of an instruction.

A compiler optimization MAY replace one valid instruction sequence with another sequence having equivalent VM-observable behavior.

A reference-implementation data structure MUST NOT be interpreted as a stable ABI unless this specification explicitly says that it is.

---

# Part I — Bytecode program model

## 4. Bytecode unit

The executable instruction container is called a **chunk**.

Conceptually, a chunk contains at least:

```text
Chunk
├── code
├── constants
├── source spans
├── executable-semantic metadata
└── implementation-side cache storage
```

The current reference implementation has the corresponding logical shape:

```rust
pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub spans: Vec<SourceRange>,
    // inline-cache side tables
    // executable semantic pool
}
```

Only `code`, `constants`, source association, and executable-semantic metadata are semantically relevant to this specification.

Inline-cache side tables are execution accelerators. Their contents MUST NOT change the meaning of bytecode and MUST be observationally equivalent to performing the corresponding uncached operation.

### 4.1 `code`

`code` is an ordered finite sequence of complete logical instructions.

An instruction pointer addresses an **instruction position**, not a byte offset.

If `code` contains `N` instructions, valid ordinary instruction positions are the integers in `[0, N)`.

### 4.2 `constants`

The constant pool is an ordered finite sequence of runtime values used by instructions through constant-pool indexes.

A bytecode instruction that refers to constant index `k` is valid only if `k < constants.length`.

Where an instruction requires a constant of a particular semantic kind, such as an interned selector symbol, class-name symbol, callable template, or selector pattern, the referenced value MUST have that required kind.

### 4.3 executable-semantic pool

Some operations refer to compiler-resolved semantic descriptions that are richer than ordinary runtime constants. Examples include resolved ADT variants, enum/data declaration descriptions, and statically resolved associated-callable targets.

Such metadata resides in the chunk's **executable-semantic pool**.

An executable-semantic reference is valid only when:

1. its index is in range; and
2. the entry kind is the kind required by the instruction.

The pool is part of the executable meaning of the chunk. It is not an optimization cache.

### 4.4 source spans

Every executable instruction position SHOULD have an associated source range.

The current VM maintains source metadata parallel to the instruction vector. A conforming implementation MAY compress or encode source maps differently, but it MUST be able to recover the source association required for runtime diagnostics, stack traces, and instruction-attributed runtime failures.

### 4.5 caches

Dispatch caches and global caches are not part of the logical bytecode state.

A VM MAY cache:

- method lookup;
- global lookup;
- associated-callable resolution;
- class- or shape-dependent access;
- any other operation whose cached and uncached results are semantically identical.

Cache invalidation is an implementation obligation. Stale cache state MUST NOT alter language-visible behavior.

---

## 5. No canonical byte-stream encoding

The canonical instruction sequence is a sequence of typed logical instructions.

The current reference implementation uses a Rust enum:

```text
Vec<Bytecode>
```

rather than:

```text
Vec<u8>
```

Accordingly:

- an instruction pointer counts instructions;
- branch offsets count instructions;
- operands are logical fields of an instruction;
- there is no normative byte endianness;
- there is no normative byte alignment;
- there is no normative serialized opcode number in this version of the specification.

The reference implementation assigns each variant a stable internal histogram/index position. Those indexes are documented in Appendix A for implementation correspondence, but they are **not** a portable serialized bytecode ABI.

A future serialized format MUST define its own version, encoding, operand widths, endianness, validation, and compatibility rules.

---

# Part II — Abstract machine

## 6. Machine state

For purposes of bytecode execution, the relevant abstract machine state is:

```text
M = ⟨Frames, Stack, Heap, Globals, Linked, Runtime, FiberState⟩
```

where the active frame determines:

```text
F = ⟨closure, ip, stack_base, receiver, upvalues, home_frame_token, ...⟩
```

The exact representation of heap objects, class descriptors, fibers, modules, and callable objects is specified in the corresponding VM documents. This document specifies only their interaction with instructions.

### 6.1 value stack

The VM is a stack machine.

Intermediate values, receivers, arguments, builders, and instruction results are communicated primarily through a single value stack.

Stack diagrams in this document use:

```text
[..., a, b]
```

where `b` is the top value.

A transition:

```text
[..., a, b] → [..., r]
```

means that `a` and `b` are consumed and `r` is produced.

### 6.2 frames

A callable activation has a frame.

A frame identifies at least:

- the executing closure/callable;
- its current instruction pointer;
- its stack base;
- its receiver where applicable;
- closure capture state;
- the return continuation implied by the caller's frame;
- for block activations, the lexical home-frame token required by non-local return.

A method or callable invocation may push a frame rather than recursively interpreting an independent instruction stream.

### 6.3 locals

A local slot index is interpreted relative to the active frame's stack base.

The implementation MUST reject or prevent bytecode that references a local slot outside the callable's valid slot domain.

### 6.4 upvalues

An upvalue represents captured lexical storage.

An upvalue may be **open**, meaning that it still refers to a live stack slot, or **closed**, meaning that the captured value has been transferred to closure-owned storage.

Closing an upvalue MUST preserve the value observable by all closures sharing that capture.

### 6.5 globals

A global reference is resolved in the module/runtime namespace associated with the active closure according to the VM module and global-binding rules.

### 6.6 linked values

A linked value is a value established by module/linking semantics rather than ordinary dynamic global lookup. `GetLinked` accesses this linkage state.

---

## 7. Instruction cycle

The canonical execution cycle is:

```text
1. determine whether the requested frame floor has drained;
2. service a coherent VM safepoint if required;
3. read the active frame;
4. let ip0 = frame.ip;
5. fetch code[ip0];
6. set frame.ip = ip0 + 1;
7. execute the fetched instruction;
8. repeat.
```

The pre-dispatch increment in step 6 is normative for interpreting relative offsets in this specification.

If an instruction has ordinary fallthrough behavior, execution continues at the already-advanced instruction pointer.

If an instruction applies a relative offset `d`, the target is:

```text
target = (ip0 + 1) + d
```

where `ip0` was the fetched instruction's position.

---

## 8. Completion modes

Instruction execution may have one of the following outcomes.

### 8.1 ordinary completion

The instruction performs its state transition and execution continues in the active frame.

### 8.2 call transfer

The instruction invokes a callable and establishes execution state for the callee, normally by pushing or otherwise activating a frame. The caller's already-advanced `ip` is the continuation point.

### 8.3 local return

`Return` terminates the current activation and transfers its result to its caller.

### 8.4 non-local return

`ReturnNonLocal` unwinds multiple block activations to a lexically designated live home frame.

### 8.5 abrupt completion

An instruction may raise a runtime error or language-level exception through an operation it performs.

The ordinary successor is not entered when abrupt completion transfers control elsewhere.

### 8.6 suspension

An instruction or invoked runtime operation may cause cooperative suspension when specified by the concurrency runtime.

The suspended execution MUST preserve the continuation necessary to resume with behavior equivalent to uninterrupted execution.

---

## 9. Stack safety

A conforming bytecode producer MUST NOT emit an instruction at a point where the instruction's required stack inputs are absent.

A conforming VM MAY treat stack underflow in externally supplied malformed bytecode as invalid bytecode or as an internal bytecode execution error.

A stack effect written as:

```text
[..., x] → [..., x, x]
```

is exact.

A stack effect written symbolically, for example:

```text
[..., receiver, args…] → [..., result]
```

uses an instruction operand or semantic descriptor to determine the number and organization of argument values.

---

## 10. Surface absence and the private nil sentinel

Phalcom source-level absence is `None`.

The VM may internally use a private nil/sentinel value for unread or uninitialized storage, but that private sentinel MUST NOT become observable as a Phalcom value.

The instruction named `Nil` is historical VM nomenclature. Its normative effect is:

```text
[...] → [..., None]
```

It MUST NOT push the private raw nil sentinel.

Any read boundary that encounters the private sentinel MUST surface it according to the runtime absence rules rather than exposing it directly.

---

# Part III — Operand vocabulary

## 11. Logical operand kinds

The current instruction representation commonly uses the following operand classes.

| Kind | Meaning |
|---|---|
| `const` | Constant-pool index |
| `symbol` | Constant-pool index whose value is an interned symbol/selector |
| `slot` | Local or field slot index |
| `upvalue` | Index into the active closure's captured-upvalue list |
| `arity` | Number of statically supplied arguments |
| `offset` | Signed relative instruction offset from the post-fetch `ip` |
| `sem` | Index into the executable-semantic pool |
| `flag` | Boolean or small enum controlling an instruction-defined mode |
| `count` | Number of stack values, tuple lanes, fields, or components |
| `kind` | Instruction-specific enumeration |

The reference implementation currently uses compact integer types, predominantly `u16` indexes, `u8` static arities, and `i32` branch offsets. Those bounds are part of the current executable representation and MUST be checked by a conforming producer targeting this VM revision.

---

## 12. Selector and argument conventions

Phalcom selectors encode callable identity, including labeled argument structure.

An `arity` operand counts values supplied by the static call sequence, but selector identity is not reducible to arity.

Static calls may use direct invocation instructions. Calls involving computed labels, expansion, or dynamically assembled argument lanes use the argument-pack instruction family.

The tuple-shaped argument model distinguishes positional and labeled lanes. Bytecode operations that build, expand, or invoke argument packs MUST preserve this distinction.

---

# Part IV — Instruction reference

## 13. Constants and primitive stack values

### `Constant(const)`

**Stack**

```text
[...] → [..., constants[const]]
```

Pushes the referenced constant-pool value.

The constant index MUST be valid.

---

### `Nil`

**Stack**

```text
[...] → [..., None]
```

Pushes canonical surface absence.

It never exposes the VM's private raw nil sentinel.

---

### `True`

```text
[...] → [..., true]
```

Pushes canonical boolean `true`.

---

### `False`

```text
[...] → [..., false]
```

Pushes canonical boolean `false`.

---

### `Pop`

```text
[..., value] → [...]
```

Discards the top value.

---

### `Dup`

```text
[..., value] → [..., value, value]
```

Duplicates the top value without changing its identity.

---

### `WrapSome`

```text
[..., value] → [..., Some(value)]
```

Constructs the canonical non-empty `Option` value around the top stack value.

---

### `GetEllipsis`

Pushes the canonical runtime value representing the ellipsis/spread marker used by the compiler/runtime protocol that requires it.

This instruction does not perform expansion by itself.

---

# 14. Local, global, linked, and field storage

### `GetLocal(slot)`

```text
[...] → [..., local[slot]]
```

Reads the active frame's local slot.

A private unread sentinel, if present internally, MUST be surfaced according to the VM absence invariant.

---

### `SetLocal(slot)`

```text
[..., value] → [..., value]
```

Stores the top value into the active frame's local slot and leaves the assigned value available as the expression result.

Where a compiler sequence does not require the value afterward, it may emit `Pop`.

---

### `DefineGlobal(symbol)`

Defines a new global/module binding whose name is the referenced symbol.

The initializer value is taken from the stack according to the global-definition convention of the runtime.

A duplicate or otherwise illegal definition MUST follow the language's binding-definition error semantics.

---

### `GetGlobal(symbol)`

Pushes the current value of the named global/module binding.

A missing binding follows the language's undefined-name/runtime lookup semantics.

A VM MAY satisfy this operation from a valid global inline cache.

---

### `SetGlobal(symbol)`

Assigns the current value of an existing writable global/module binding.

The assigned value remains available according to assignment-expression semantics.

---

### `GetLinked(index)`

Pushes a value from the active module/closure's link-resolved binding table.

The operand denotes a linker-established entry, not an ordinary source-level global name lookup.

A conforming linker MUST ensure that the referenced linked entry exists before execution.

---

### `GetField(slot)`

Pushes the receiver instance field stored at the statically assigned field-slot offset.

The slot operand is a field-layout offset, not a source field name.

The receiver is the current frame receiver.

---

### `SetField(slot)`

Stores the top value into the receiver instance's statically assigned field slot.

The assigned value remains available according to assignment-expression semantics.

---

### `GetSelf`

```text
[...] → [..., self]
```

Pushes the current activation's receiver.

---

### `ReserveScratchLocal`

Reserves compiler-managed temporary local storage in the current activation.

Scratch locals are not source-visible bindings. Their lifetime MUST obey the compiler's stack-layout discipline.

---

### `ReleaseScratchLocal`

Releases the most recently reserved scratch-local region according to the scratch-local discipline.

A bytecode producer MUST balance scratch reservations and releases across all reachable ordinary control-flow paths.

---

# 15. Closures, captures, and returns

### `Closure(const)`

Creates a runtime closure from the callable/closure template stored in the referenced constant-pool entry.

The new closure captures the upvalues described by the template's capture descriptors.

```text
[...] → [..., closure]
```

The closure's module/environment identity MUST be the identity defined by its lexical compilation context.

---

### `GetUpvalue(upvalue)`

```text
[...] → [..., captured]
```

Pushes the current value of the indexed capture.

---

### `SetUpvalue(upvalue)`

Stores the top value through the indexed capture.

If the upvalue is open, the corresponding live stack slot is updated. If closed, the closure-owned storage is updated.

---

### `CloseUpvalue(slot)`

Closes every open upvalue that points to the designated stack slot or to a stack slot above it within the leaving lexical region.

Closing preserves shared capture identity.

This instruction is used before storage holding captured locals becomes invalid.

---

### `Return`

Returns from the current activation.

If an explicit result value is present according to callable lowering, that value becomes the caller-visible result.

If the implementation reaches a return boundary with its private absence sentinel, the result MUST be surfaced as `None`.

The current frame is removed, escaping upvalues are closed as required, and the result is placed at the caller's result position.

---

### `ReturnNonLocal`

Performs a block non-local return.

The target is **not encoded in the instruction**. It is the live lexical home frame identified by the executing block frame's `home_frame_token`.

Execution MUST:

1. obtain the return value;
2. find the live frame whose token is the block's home token;
3. close any upvalues that would otherwise reference removed stack storage;
4. remove every frame from the current block activation through the target home activation;
5. restore/truncate the value stack to the home activation's return boundary;
6. place the non-local return value as the home method's result;
7. allow nested interpreter/run boundaries to observe the already-completed unwind.

If the lexical home frame no longer exists, execution raises `DeadFrameError` or the corresponding specified runtime error.

---

# 16. Control transfer and guards

### `Jump(offset)`

Unconditionally applies `offset` to the already-advanced instruction pointer.

```text
ip' = ip_after_fetch + offset
```

The target MUST identify a valid instruction position.

---

### `Loop(offset)`

Semantically identical to `Jump(offset)`.

The separate mnemonic denotes an intended loop back-edge and exists for compiler/disassembler clarity.

A conforming VM MAY execute it through the same implementation path as `Jump`.

---

### `JumpIfFalse(offset)`

```text
[..., condition] → [...]
```

Consumes the top value.

If the value is `false`, branches by `offset`.

If the value is `true`, falls through.

No truthiness conversion is performed. A non-boolean operand raises the runtime type error specified for a required boolean condition.

---

### `JumpIfNone(offset)`

```text
[..., value] → [...]
```

Consumes the top value.

If it is exactly canonical `None`, branches by `offset`; otherwise falls through.

This is a VM-owned identity test for iteration and related compiler-owned control flow. It does not send an overridable equality message.

---

### `GuardBool(offset)`

Peeks at the top stack value and guards a sacred-selector boolean fast path.

It branches to the fallback sequence when either:

- the value is not a canonical boolean; or
- the runtime epoch/state indicates that the sacred `Bool` methods relevant to the fast path are no longer pristine.

It does not consume the guarded value.

A VM that implements sacred-selector invalidation differently MUST preserve the same observable behavior: an override that changes the relevant method semantics must cause the general send behavior to be used.

---

### `GuardBlock(offset)`

Guards the compiler's sacred `Block` control-flow fast path.

The block receiver is compiler-known; this instruction therefore guards the validity/pristineness of the sacred `Block` method behavior rather than performing a general receiver-type test.

If the fast path is no longer semantically valid, the instruction branches to the compiler-emitted fallback send sequence.

---

### `GuardSymbol`

Peeks at the top stack value and requires it to be a runtime symbol acceptable to the operation being compiled.

On failure, raises the specified runtime type error.

The guarded value remains on the stack.

---

### `JumpIfUnsupported(offset)`

Consumes the top value.

If the value is the VM's canonical internal `Unsupported` result marker used by bilateral operator dispatch, branches by `offset`.

Otherwise the ordinary value remains or is routed according to the bilateral lowering contract.

`Unsupported` is an internal dispatch protocol value and is not a general source-level absence value.

---

### `MatchInvariantFailure`

Signals that execution reached a fallthrough point the compiler proved statically unreachable for an exhaustive match.

Execution MUST fail as an internal invariant violation rather than silently produce a value.

A conforming compiler MUST emit this only for a path that its static exhaustiveness analysis has established as impossible.

---

# 17. Ordinary message invocation

## 17.1 `Invoke(arity, selector)`

Performs an ordinary message send.

Conceptual input:

```text
[..., receiver, arg0, ..., argN-1]
```

where `N = arity`.

The selector operand identifies the exact selector symbol, including labels.

Execution:

1. identifies the receiver;
2. resolves the exact selector according to ordinary method lookup;
3. if a method is found, establishes a callee activation with the same receiver and supplied arguments;
4. if no method is found, follows the language's `doesNotUnderstand` / message-not-understood protocol;
5. when the callee returns, replaces the call input region with the result.

The caller's already-advanced `ip` is the return continuation.

A VM MAY use a valid per-call-site inline cache.

---

## 17.2 `SuperSend(arity, selector, definingClass)`

Performs a statically anchored `super` send.

The original receiver remains `self`.

Method lookup begins **above the defining class**, not above the receiver's dynamic class and not from a statically baked superclass object.

The defining class operand names the class whose body contains the send. At runtime the VM resolves that class and begins lookup at its current superclass.

This preserves correct behavior if superclass relationships are mutable under the runtime model.

A miss follows the same message-not-understood surface path as ordinary invocation.

---

## 17.3 `InvokeLocal(slot, arity, selector)`

A fused superinstruction equivalent in observable behavior to:

```text
GetLocal(slot)
Invoke(arity, selector)
```

with the stack arrangement required by the call sequence.

It may avoid materializing the intermediate receiver load as a distinct dispatch.

The instruction is semantically an optimization. Bytecode consumers MUST preserve the same result, lookup behavior, errors, and call continuation as the unfused sequence.

---

## 17.4 `InvokeConst(const, arity, selector)`

A fused superinstruction equivalent in observable behavior to loading the referenced constant as receiver and performing `Invoke`.

---

## 17.5 `InvokeCompilerInternal(arity, selector)`

Performs a send through the compiler-internal access path.

This instruction exists for semantics that the compiler is authorized to invoke but ordinary source code is not authorized to address through the same lookup/access route.

It MUST NOT become a general privilege-escalation path for arbitrary user-produced bytecode.

A conforming verifier or trusted-bytecode boundary SHOULD restrict this instruction to compiler-produced code.

---

# 18. Callable references, families, and associated invocation

Phalcom distinguishes exact callable identity from callable families.

A family may represent all selector members matching a family specification rather than one already-resolved selector.

The precise family, selector-pattern, bound-method, and associated-callable object models are defined in the VM dispatch/reflection specification. This section defines bytecode interaction with them.

### `MakeFamily { spec, kind }`

Consumes a receiver and constructs a bound callable-family object described by `spec`.

`kind` distinguishes an exact selector specification from a structural selector-pattern specification.

```text
[..., receiver] → [..., family]
```

Construction MUST preserve receiver identity and family matching semantics.

---

### `MakeResolvedBoundMethod`

Constructs a bound method reference from a statically resolved behavioral target while retaining the concrete receiver to which the method is bound.

The resulting callable reference MUST invoke the resolved method semantics without reinterpreting source reference syntax.

---

### `InvokeResolvedAssociated`

Invokes a compiler-resolved associated callable target using statically supplied arguments.

The target is referenced through executable-semantic metadata.

This instruction is valid only for associated-callable forms supported by Phalcom's associated lookup model.

---

### `MakeAssociatedFamily`

Constructs a callable family representing associated callables for a resolved associated owner/base.

The family itself is a first-class runtime callable-family value.

---

### `InvokeAssociatedFamilyStatic`

Performs associated-family member selection when the call's selector structure is statically known.

It MUST select the same family member that the general family invocation semantics would select for the same argument lanes.

---

### `InvokeAssociatedFamilyPack`

Performs associated-family member selection and invocation using a dynamically assembled argument pack.

---

### `InvokeConditional`

Invokes a behavioral member whose availability depends on runtime satisfaction of compiler-retained conditional-member constraints.

The VM MUST check the retained executable condition before treating the member as callable.

Failure to satisfy the condition follows the conditional-behavior lookup semantics rather than invoking an invalid implementation.

---

### `MakeConditionalFamily`

Constructs a bound family whose visible/selectable members are conditioned by retained runtime constraints.

Family reflection and invocation MUST not expose a conditional member as callable when its runtime condition is unsatisfied.

---

# 19. Classes and ordinary instances

### `Class(name)`

Allocates a fresh class object whose declared name is the referenced symbol.

The runtime execution of this instruction always denotes fresh class construction.

Compiler-resolved completion of pre-installed kernel classes, when permitted, is not a second runtime meaning of `Class`; such cases are lowered differently.

The new class is pushed for subsequent method installation/finalization.

---

### `Method(selector, isStatic)`

Attaches a compiled method to the class-under-construction at the top of the class-construction stack protocol.

The selector is exact.

If `isStatic` is false, the method is installed on the instance side.

If `isStatic` is true, the method is installed on the class/associated side defined by the class model.

Installation MUST respect method-family/selector identity rules.

---

### `FinalizeClass`

Completes runtime class construction after all compiler-emitted class body installation operations.

Finalization establishes the invariants required before ordinary instances or lookup may treat the class as complete.

Finalization MUST NOT silently change already-established source-level declaration semantics.

---

### `NewInstance`

Consumes a class object and allocates a new ordinary instance whose runtime class is that class.

Conceptually:

```text
[..., class] → [..., instance]
```

Instance storage is initialized according to the object-layout rules. Private unread field storage MUST not expose the raw VM nil sentinel.

This opcode performs allocation; constructor method semantics remain governed by constructor lowering and invocation.

---

# 20. Product values: tuples and records

### `BuildTuple { positional, labeled }`

Consumes the compiler-known values comprising a tuple's positional and labeled lanes and constructs one tuple value.

The resulting tuple MUST preserve:

- positional order;
- label identity;
- labeled-value association;
- the tuple/product invariants defined by the language.

Duplicate or otherwise invalid labels MUST follow the product-construction error semantics.

---

### `BuildRecord { fields }`

Consumes the compiler-known labeled field entries and constructs one record value.

Record field identity and duplicate detection MUST follow the record product rules.

---

### `BuildStaticTuple`

Constructs a tuple through the representation-aware static product path retained by compiler semantic metadata.

Its observable value MUST be equivalent to construction of the corresponding source tuple.

The instruction may exploit known representation/layout information but MUST NOT change tuple equality, reflection, field access, or lane semantics.

---

### `BuildStaticRecord`

Constructs a record through the representation-aware static product path.

Its observable result MUST satisfy the ordinary record semantics for the represented fields.

---

# 21. List, map, set, record-literal, and range construction

Literal builders are VM-supported construction protocols. Builder objects or partially constructed containers are compiler/runtime intermediates and MUST NOT escape to source code except where the language explicitly defines them as ordinary values.

## 21.1 maps

### `BeginMapLiteral`

```text
[...] → [..., map]
```

Allocates a new map literal target.

### `MapLiteralInsertUnique`

Conceptually:

```text
[..., map, key, value] → [..., map]
```

Inserts one key/value pair.

A duplicate key within literal construction MUST obey the map literal's uniqueness/error semantics rather than silently changing meaning through implementation-dependent insertion order.

### `MapLiteralExpandLabels`

Expands a labeled product/record-like source into the map under the language's labeled-spread rules.

The expansion order and duplicate handling MUST follow source semantics.

### `FinishMapLiteral`

Marks completion of the map-literal construction protocol.

The current implementation may require no additional mutation at this point. Even where operationally a no-op, the instruction remains a logical construction boundary.

---

## 21.2 sets

### `BeginSetLiteral`

Allocates a new set literal target.

### `SetLiteralAdd`

Adds the top element to the set-under-construction.

The set's equality/hash semantics determine element identity.

### `FinishSetLiteral`

Completes the set-literal construction protocol.

---

## 21.3 lists

### `BuildList(count)`

Consumes `count` statically known element values and constructs a list in source order.

This is appropriate when no dynamic expansion requires incremental building.

### `BeginListLiteral`

Allocates an incremental list builder/target.

### `ListLiteralAppend`

Appends one source element, preserving left-to-right source evaluation order.

### `ListTryExpandTuplePositionals`

Attempts positional expansion of a tuple-shaped source into the list under spread semantics.

If the value is not expandable in the required manner, the operation follows the language's spread failure semantics.

### `FinishListLiteral`

Produces/completes the final list value.

---

## 21.4 record literals

### `NewRecordLiteralBuilder`

Allocates a record-literal construction builder.

### `RecordLiteralAppend`

Appends one statically or dynamically named field entry.

### `RecordLiteralExpandLabels`

Expands the labeled lane of a record/tuple-shaped source into the record builder.

Duplicate handling MUST follow record literal semantics.

### `FinishRecordLiteral`

Finalizes the record builder into the canonical runtime record value.

---

## 21.5 ranges

### `BuildRange { hasLower, hasUpper, ... }`

Constructs the canonical range value represented by the source range expression.

The instruction's flags describe which endpoints and boundary modes are present.

Endpoint values are consumed in source evaluation order.

The resulting range MUST implement the language-defined range semantics independently of the reference implementation's object layout.

---

# 22. Dynamic argument-pack construction

An **argument pack** is a VM construction object used when a call cannot be fully represented by a simple static `arity` plus exact selector at compile time.

It contains logically separate positional and labeled lanes and enough state to enforce Phalcom's argument-expansion rules.

The construction protocol is compiler-owned.

### `NewArgumentPack`

Allocates an empty argument-pack builder.

```text
[...] → [..., pack]
```

---

### `PackPushPositional`

Adds one value to the positional lane in source order.

---

### `PackReserveStaticLabel`

Reserves a labeled lane entry whose label is statically known.

Reservation permits source evaluation order to be preserved when label/value computation requires later filling.

---

### `PackReserveComputedLabel`

Consumes/records a runtime-computed label and reserves its associated value position.

The computed label MUST satisfy symbol/label requirements.

---

### `PackFillReservedLabel`

Fills the value of a previously reserved label entry.

A producer MUST obey reservation/fill discipline exactly once per reservation.

---

### `PackExpandLabels`

Expands the labeled lane of a tuple/record-shaped value into the argument pack.

Duplicate labels MUST be detected according to call argument semantics.

---

### `PackExpandComplete`

Completes a general argument expansion step whose source contributes the complete supported lane structure.

The operation MUST preserve source evaluation order and duplicate-label rules.

---

### `PackTryExpandTuplePositionals`

Expands tuple positional values into the positional lane.

Failure to satisfy tuple-position expansion requirements follows the spread/argument runtime semantics.

---

### `FinishTuplePack`

Finalizes an argument-pack construction into the canonical tuple-shaped argument value used by call/reference machinery when the pack itself is the resulting value rather than immediately invoked.

---

# 23. Packed invocation

### `InvokePack`

Performs ordinary message/family dispatch using a dynamically assembled argument pack.

The pack determines positional values, labels, and therefore final selector/member selection under the dynamic invocation rules.

The resulting behavior MUST equal a direct static `Invoke` whenever the dynamic pack resolves to the same exact selector and arguments.

---

### `SuperSendPack`

Performs a `super` send using a dynamically assembled argument pack.

Lookup begins above the statically known defining class while the receiver remains the original `self`.

---

### `InvokeSubscriptSetPack`

Invokes the subscript-assignment callable form using a dynamically assembled pack.

The instruction exists because subscript-set dispatch has a distinct callable/selector shape that must be preserved when arguments are dynamically expanded.

---

# 24. Bilateral and reflected operator protocol

Some binary operators use a bilateral dispatch protocol rather than a single unconditional ordinary send.

The protocol may:

1. compare left and right operand dispatch priorities;
2. prefer a reflected implementation;
3. try an exact implementation without automatically falling into generic message-not-understood behavior;
4. propagate an internal `Unsupported` result;
5. validate ordering results;
6. raise the operator-specific unsupported-operation error if neither side supports the operation.

The following instructions are compiler/runtime primitives for this protocol.

### `BilateralPreferReflected(selector)`

Examines the bilateral operands and determines whether the reflected/right-hand implementation should be preferred for the specified operator family.

It MUST use the language-defined bilateral precedence rule rather than arbitrary class-address or allocation ordering.

---

### `TryInvokeExact`

Attempts an exact selector invocation in bilateral-dispatch mode.

Unlike ordinary `Invoke`, failure to find/support the exact operation yields the bilateral protocol's internal `Unsupported` outcome rather than immediately entering ordinary message-not-understood behavior.

---

### `JumpIfUnsupported(offset)`

Branches based on the internal unsupported marker as specified in §16.

---

### `ValidateOrdering`

Validates that the result produced by an ordering/comparison protocol is one of the values permitted by the ordering contract.

Invalid comparison protocol results raise the specified runtime error.

---

### `Same`

Performs VM identity/sameness comparison required by the bilateral/comparison lowering.

This is not an overridable user message.

Its exact identity relation is defined by the runtime value/object model.

---

### `RaiseUnsupported`

Raises the language-defined unsupported-operator error after bilateral dispatch has determined that no permitted implementation supports the requested operation.

The instruction's operator metadata MUST identify the operation for diagnostic construction.

---

# 25. Enumeration and ADT instructions

Enum bytecodes interact with semantic descriptors retained by the compiler.

The runtime representation may use enum descriptors, variant descriptors, singleton case values, constructor case objects, and hidden behavior classes. Those structures are specified by the runtime ADT model; this document defines instruction effects.

### `Enum(sem)`

Begins or materializes the runtime enum root described by the referenced executable enum semantic descriptor.

If the runtime registry already contains the canonical declaration identity in a context where reuse is specified, execution MUST respect that canonical identity rather than create an observably distinct duplicate descriptor.

---

### `VariantMethod`

Installs behavior associated with an enum variant/case according to the compiler-provided variant method metadata and class/variant construction protocol.

---

### `FinalizeEnum(sem)`

Finalizes the enum root and its hidden/runtime case behavior classes.

After finalization, variant construction, singleton loading, lookup, matching, and reflection MUST observe a coherent completed enum definition.

---

### `LoadVariantSingleton(sem)`

Pushes the canonical singleton value of a singleton-shaped enum variant.

Repeated execution for the same declaration/variant identity MUST preserve singleton identity.

---

### `ConstructVariant { variant, arity }`

Consumes the payload values for the statically resolved constructor-shaped variant and allocates the corresponding ADT case value.

Payload ordering and labels MUST match the variant declaration's product shape.

The referenced semantic target MUST be a constructor-shaped variant whose arity matches the instruction.

---

### `IsVariant`

Tests whether a runtime value is an instance of the statically identified variant.

The test is based on canonical variant identity, not merely equal names.

It produces a canonical boolean.

---

### `GetVariantPayload`

Extracts the payload or one specified payload component from a value whose variant identity has already been established by the match/lowering contract.

Applying it to an incompatible value is invalid execution and MUST fail rather than reinterpret memory/layout.

---

# 26. Data declaration instructions

`data` declarations use a product-like runtime construction model parallel to, but distinct from, enum variants.

### `Data(sem)`

Begins or materializes the runtime data descriptor described by executable-semantic metadata.

---

### `FinalizeData(sem)`

Completes the data declaration's runtime descriptor, behavior, construction metadata, and any canonical singleton state.

---

### `LoadDataSingleton(sem)`

Pushes the canonical singleton value for a zero-component/singleton data declaration.

Repeated loads preserve canonical singleton identity.

---

### `ConstructData`

Consumes the data component values required by the statically resolved data constructor and produces a runtime data value.

Representation-aware optimization MAY scalarize or specialize construction only when all observable data semantics remain identical.

---

### `GetDataComponent`

Extracts a statically selected data component from a compatible data value.

The component identity is determined by executable semantic metadata rather than by ad-hoc runtime string lookup.

---

# 27. Miscellaneous compiler/runtime construction operations

### `BuildStaticTuple`

See §20.

### `BuildStaticRecord`

See §20.

### `GetEllipsis`

See §13.

These instructions are grouped separately in the physical enum for historical/evolution reasons but remain governed by their semantic categories above.

---

# Part V — Calls, frames, and dispatch invariants

## 28. Call-site continuation

Every call instruction executes after the caller frame's `ip` has already advanced.

Therefore the current caller `ip` is the return address.

A callee MUST NOT manually decrement the caller `ip` on ordinary return.

---

## 29. Receiver preservation

For an ordinary method send, the runtime receiver is the selected value.

For `super` sends, lookup origin changes but receiver identity does not.

For bound-method and bound-family references, the receiver retained by the reference is the receiver used when invoked unless the specific callable form defines otherwise.

---

## 30. Selector exactness

Exact selectors are interned runtime identities.

Named argument labels participate in selector identity.

An implementation MUST NOT collapse selectors merely because they have equal arity.

Family/pattern matching is a separate operation and MUST be represented as such.

---

## 31. Message-not-understood behavior

Ordinary invocation lookup failure follows Phalcom's message-not-understood protocol.

Instructions whose contract explicitly performs a **probe**—for example bilateral `TryInvokeExact`—MAY return an internal unsupported state rather than immediately enter message-not-understood handling.

That distinction is semantic.

---

# Part VI — Control-flow validity

## 32. Branch targets

A branch target MUST:

- fall within the chunk's code domain unless the instruction semantics explicitly define completion instead;
- identify the start of a logical instruction, which is automatic in the typed-vector representation;
- be compatible with the compiler's stack-shape assumptions at that control-flow merge.

Because the canonical current representation contains one complete instruction per vector position, there is no concept of branching into the middle of an encoded instruction.

---

## 33. Stack-shape agreement

At every reachable control-flow merge, all predecessor paths MUST agree on the stack layout required by the successor.

A compiler MAY retain additional temporary values only where the successor instruction sequence is defined to consume them.

Malformed bytecode with incompatible merge shapes is invalid.

---

## 34. Protected/non-local regions

A bytecode producer MUST NOT use ordinary jumps to emulate frame unwinding across regions where closure capture, ensure/finalization behavior, exception handlers, or non-local return semantics require dedicated runtime processing.

Non-local return MUST use the non-local-return mechanism.

Exception transfer MUST use the runtime unwinding mechanism defined by the exception specification.

---

# Part VII — Source mapping and diagnostics

## 35. Instruction source ownership

Every instruction emitted from source SHOULD be associated with the narrowest source range that accurately explains the operation.

Compiler-synthesized instructions MUST still be attributable to a meaningful enclosing source construct when they can raise a user-visible runtime error.

Examples:

- an `Invoke` should map to the send/call expression;
- a `JumpIfFalse` synthesized for sacred control flow should map to the controlling expression;
- a `ConstructVariant` should map to the associated constructor call;
- a duplicate-label expansion error should map to the spread/argument expression responsible for expansion.

---

## 36. Instruction pointer to source range

If an error is attributed to the instruction just fetched at `ip0`, diagnostic lookup MUST use that instruction's source association, not the post-fetch `ip0 + 1`.

The implementation may retain the pre-increment instruction index for this purpose.

---

## 37. Stack traces

Call-site stack trace entries SHOULD resolve to the source range of the invocation instruction that established the callee continuation.

A conforming implementation MAY store richer frame metadata, but it MUST NOT systematically attribute runtime failures to the instruction following the call merely because the caller's `ip` was pre-incremented.

---

# Part VIII — Bytecode validity and trusted execution

## 38. Structural validity

A bytecode unit is structurally valid only if:

1. every instruction variant is recognized;
2. every constant-pool reference is in range;
3. every constant referenced as a symbol/selector has the required kind;
4. every executable-semantic reference is in range and of the required kind;
5. every local slot reference is valid for the owning callable;
6. every upvalue reference is valid for the owning closure;
7. every branch target is valid;
8. static arities and statically described payload counts agree with the corresponding semantic descriptors;
9. compiler-owned construction protocols are balanced on every reachable path;
10. protected internal instructions occur only where permitted by the trust model.

---

## 39. Semantic protocol validity

The following are examples of bytecode that may be structurally decodable but semantically invalid:

- `GetVariantPayload` applied without the variant-match precondition required by its lowering;
- a `ConstructVariant` whose arity disagrees with the target descriptor;
- a `PackFillReservedLabel` with no outstanding reservation;
- a `ReleaseScratchLocal` without a corresponding reservation;
- a `FinalizeClass` when the stack does not contain the class-under-construction required by the protocol;
- a branch into a successor whose expected stack shape is incompatible with the predecessor;
- `InvokeCompilerInternal` in untrusted externally supplied bytecode.

A conforming producer MUST NOT generate such sequences.

---

## 40. Malformed bytecode behavior

The canonical Phalcom compiler produces trusted bytecode.

If Phalcom later accepts bytecode from disk, plugins, network sources, or third-party compilers, the consumer MUST validate untrusted bytecode before execution or enforce equivalent safety dynamically.

Malformed bytecode MUST NOT be able to violate host memory safety.

The VM MAY reject malformed bytecode with a bytecode-validation error rather than attempting to assign source-language semantics to an impossible state.

---

# Part IX — Current representation limits

## 41. Index widths

The reference enum currently uses compact operand fields, including many `u16` pool/slot indexes and `u8` static arities.

A compiler targeting this VM revision MUST diagnose overflow before truncation.

The following classes of limits therefore require explicit checked construction:

- constant-pool indexes;
- executable-semantic indexes;
- local/upvalue indexes where represented as `u16`;
- selector/name indexes;
- any static count stored in a bounded operand;
- static arity stored in `u8`.

A compiler MUST NOT silently wrap, saturate, or truncate an operand to fit the instruction representation.

---

## 42. Branch range

Relative branches use a signed 32-bit logical offset in the current representation.

The offset counts instructions.

A compiler MUST diagnose a branch displacement that cannot be represented.

---

## 43. Instruction count

The typed-vector representation is indexed by the host container's index domain, but every operation that converts an instruction count or index to a narrower instruction operand MUST be checked.

No language-level limit should be inferred merely from an implementation-local vector index type unless the limit is explicitly standardized.

---

# Part X — Optimization and canonical semantics

## 44. Superinstructions

The current ISA contains instructions such as `InvokeLocal` and `InvokeConst` that fuse multiple canonical stack operations.

These instructions are normative executable instructions in this VM revision, but their semantics are defined by equivalence to the underlying operations.

A future bytecode architecture MAY distinguish:

```text
canonical semantic bytecode
        ↓
bytecode optimization / fusion
        ↓
executable bytecode
```

without changing source-language behavior.

Until such a distinction is standardized, tools that consume executable chunks MUST recognize the fused instructions listed in this document.

---

## 45. Inline caches

Inline caches MAY accelerate:

- ordinary invoke;
- global lookup;
- associated lookup;
- other stable lookup operations.

A cache hit MUST be observationally equivalent to performing current lookup from first principles.

Any mutation that can change lookup results MUST invalidate or version-check affected caches.

---

## 46. Sacred-selector guards

`GuardBool` and `GuardBlock` exist because certain source control-flow forms are lowered to compiler-owned fast paths while the corresponding methods remain semantically overridable.

These guards are therefore **semantic deoptimization guards**, not optional micro-optimizations.

A conforming VM may implement their mechanism differently, but it MUST preserve the rule:

> if user-visible override state makes the inlined sacred behavior no longer equivalent to a real send, execution must take behavior equivalent to the real send.

---

# Part XI — Instruction catalog

## 47. Catalog by category

The current instruction set contains 101 logical variants.

### Constants and stack

```text
Constant
Nil
True
False
Pop
Dup
WrapSome
GetEllipsis
```

### Locals, globals, linkage, fields, temporaries

```text
GetLocal
SetLocal
DefineGlobal
GetGlobal
SetGlobal
GetLinked
GetField
SetField
GetSelf
ReserveScratchLocal
ReleaseScratchLocal
```

### Calls, dispatch, references, families

```text
Invoke
SuperSend
InvokeLocal
InvokeConst
InvokeCompilerInternal
InvokePack
SuperSendPack
InvokeSubscriptSetPack
MakeFamily
MakeResolvedBoundMethod
InvokeResolvedAssociated
MakeAssociatedFamily
InvokeAssociatedFamilyStatic
InvokeAssociatedFamilyPack
InvokeConditional
MakeConditionalFamily
```

### Closures and returns

```text
Return
ReturnNonLocal
Closure
GetUpvalue
SetUpvalue
CloseUpvalue
```

### Control flow and guards

```text
Jump
JumpIfFalse
JumpIfNone
Loop
GuardBool
GuardBlock
GuardSymbol
JumpIfUnsupported
MatchInvariantFailure
```

### Classes and instances

```text
Class
Method
FinalizeClass
NewInstance
```

### Products and literals

```text
BuildTuple
BuildRecord
BuildStaticTuple
BuildStaticRecord
BeginMapLiteral
MapLiteralInsertUnique
MapLiteralExpandLabels
FinishMapLiteral
BeginSetLiteral
SetLiteralAdd
FinishSetLiteral
BuildList
BeginListLiteral
ListLiteralAppend
ListTryExpandTuplePositionals
FinishListLiteral
NewRecordLiteralBuilder
RecordLiteralAppend
RecordLiteralExpandLabels
FinishRecordLiteral
BuildRange
```

### Argument packs

```text
NewArgumentPack
PackPushPositional
PackReserveStaticLabel
PackReserveComputedLabel
PackFillReservedLabel
PackExpandLabels
PackExpandComplete
PackTryExpandTuplePositionals
FinishTuplePack
```

### Bilateral operators

```text
BilateralPreferReflected
TryInvokeExact
JumpIfUnsupported
ValidateOrdering
Same
RaiseUnsupported
```

### Enum/ADT

```text
Enum
VariantMethod
FinalizeEnum
LoadVariantSingleton
ConstructVariant
IsVariant
GetVariantPayload
```

### Data

```text
Data
FinalizeData
LoadDataSingleton
ConstructData
GetDataComponent
```

---

# Part XII — Worked execution contracts

## 48. Constant and local binding

Conceptually:

```phalcom
let x = 42
x
```

may produce a sequence of the form:

```text
Constant(k42)
SetLocal(xslot)
Pop
GetLocal(xslot)
```

The exact lowering is non-normative. The instruction effects are normative.

---

## 49. Ordinary send

Conceptually:

```phalcom
receiver.doThing(arg)
```

may lower to:

```text
<receiver>
<arg>
Invoke(1, #doThing(_))
```

Immediately before `Invoke`, the stack contains the receiver and argument region required by the call convention.

`Invoke` establishes the callee; the caller's current post-fetch `ip` is the continuation.

After ordinary callee return, the call region is replaced by one result value.

---

## 50. Super send

Conceptually:

```phalcom
super.render()
```

lowers to a `SuperSend` carrying:

- the exact selector;
- static arity;
- the name/identity needed to resolve the defining class.

The runtime does **not** replace the receiver with the superclass.

It changes only the lookup starting point.

---

## 51. Branch

Given an instruction at index `10`:

```text
10  Jump(+4)
```

fetch first advances `ip` to `11`.

The jump target is therefore:

```text
11 + 4 = 15
```

not `14`.

---

## 52. Iteration termination

A compiler-owned loop may place a cursor result on the stack and execute:

```text
JumpIfNone(exit)
```

`None` means iteration exhaustion.

The test is exact and non-overridable.

Any non-`None` cursor value falls through and may then be passed through the iterator-value protocol.

---

## 53. Non-local return

A method invokes a block whose activation has a home-frame token referring to the method activation.

Inside the block:

```phalcom
return value
```

executes `ReturnNonLocal`.

The VM does not merely return from the block. It removes the intervening block/call activations up to and including the live home method return boundary, closes escaping captures, and delivers `value` as the method result.

If the method activation already returned before the block tries to return non-locally, the home token is dead and execution raises the dead-frame error.

---

# Part XIII — Conformance

## 54. Bytecode producer conformance

A conforming producer:

- emits only defined instructions;
- respects all operand domains;
- preserves stack-shape correctness;
- emits valid branch targets;
- respects local/upvalue limits;
- emits semantic-pool entries of the required kind;
- respects builder and argument-pack protocols;
- does not rely on stale cache state;
- does not expose the private nil sentinel;
- emits internal/compiler-only instructions only when authorized;
- reports operand-width overflow rather than truncating.

---

## 55. VM consumer conformance

A conforming VM:

- implements the state transitions defined by this document;
- treats `ip` and relative branches according to the post-fetch rule;
- preserves selector identity and labeled argument semantics;
- preserves closure capture identity;
- implements ordinary and non-local return correctly;
- preserves source-level absence rules;
- performs lookup and super lookup according to their respective contracts;
- ensures cache use is semantically transparent;
- preserves left-to-right source effects as encoded by the producer;
- rejects or safely handles malformed untrusted bytecode;
- provides source attribution sufficient for specified diagnostics and traces.

---

## 56. Tooling conformance

A disassembler, debugger, profiler, bytecode linter, or optimizer claiming conformance SHOULD understand every instruction in the current catalog.

A tool MAY classify instructions as:

- semantic core;
- construction protocol;
- superinstruction;
- guard/deoptimization;
- compiler-internal;
- metadata-driven.

Such classification does not alter instruction semantics.

---

# Appendix A — Current reference variant order

This appendix records the current `Bytecode::index()` / `BYTECODE_NAMES` ordering for correspondence with the reference implementation.

It is **not** a serialized ABI.

| Index | Instruction |
|---:|---|
| 0 | `Constant` |
| 1 | `Nil` |
| 2 | `True` |
| 3 | `False` |
| 4 | `Pop` |
| 5 | `GetLocal` |
| 6 | `SetLocal` |
| 7 | `DefineGlobal` |
| 8 | `GetGlobal` |
| 9 | `SetGlobal` |
| 10 | `GetField` |
| 11 | `SetField` |
| 12 | `GetSelf` |
| 13 | `Invoke` |
| 14 | `SuperSend` |
| 15 | `Class` |
| 16 | `Method` |
| 17 | `Return` |
| 18 | `ReturnNonLocal` |
| 19 | `Closure` |
| 20 | `GetUpvalue` |
| 21 | `SetUpvalue` |
| 22 | `CloseUpvalue` |
| 23 | `Jump` |
| 24 | `JumpIfFalse` |
| 25 | `JumpIfNone` |
| 26 | `Loop` |
| 27 | `GuardBool` |
| 28 | `GuardBlock` |
| 29 | `NewInstance` |
| 30 | `Dup` |
| 31 | `WrapSome` |
| 32 | `GetLinked` |
| 33 | `MakeFamily` |
| 34 | `FinalizeClass` |
| 35 | `InvokeLocal` |
| 36 | `InvokeConst` |
| 37 | `GuardSymbol` |
| 38 | `BuildTuple` |
| 39 | `BuildRecord` |
| 40 | `BeginMapLiteral` |
| 41 | `MapLiteralInsertUnique` |
| 42 | `FinishMapLiteral` |
| 43 | `BeginSetLiteral` |
| 44 | `SetLiteralAdd` |
| 45 | `FinishSetLiteral` |
| 46 | `BuildRange` |
| 47 | `InvokeCompilerInternal` |
| 48 | `BuildList` |
| 49 | `NewArgumentPack` |
| 50 | `PackPushPositional` |
| 51 | `PackReserveStaticLabel` |
| 52 | `PackReserveComputedLabel` |
| 53 | `PackFillReservedLabel` |
| 54 | `PackExpandLabels` |
| 55 | `PackExpandComplete` |
| 56 | `PackTryExpandTuplePositionals` |
| 57 | `InvokePack` |
| 58 | `SuperSendPack` |
| 59 | `FinishTuplePack` |
| 60 | `ReserveScratchLocal` |
| 61 | `ReleaseScratchLocal` |
| 62 | `BeginListLiteral` |
| 63 | `ListLiteralAppend` |
| 64 | `FinishListLiteral` |
| 65 | `ListTryExpandTuplePositionals` |
| 66 | `NewRecordLiteralBuilder` |
| 67 | `RecordLiteralAppend` |
| 68 | `RecordLiteralExpandLabels` |
| 69 | `FinishRecordLiteral` |
| 70 | `MapLiteralExpandLabels` |
| 71 | `GetEllipsis` |
| 72 | `BilateralPreferReflected` |
| 73 | `TryInvokeExact` |
| 74 | `JumpIfUnsupported` |
| 75 | `ValidateOrdering` |
| 76 | `Same` |
| 77 | `RaiseUnsupported` |
| 78 | `Enum` |
| 79 | `VariantMethod` |
| 80 | `FinalizeEnum` |
| 81 | `LoadVariantSingleton` |
| 82 | `ConstructVariant` |
| 83 | `MakeResolvedBoundMethod` |
| 84 | `InvokeResolvedAssociated` |
| 85 | `MakeAssociatedFamily` |
| 86 | `InvokeAssociatedFamilyStatic` |
| 87 | `InvokeAssociatedFamilyPack` |
| 88 | `IsVariant` |
| 89 | `GetVariantPayload` |
| 90 | `MatchInvariantFailure` |
| 91 | `InvokeSubscriptSetPack` |
| 92 | `Data` |
| 93 | `FinalizeData` |
| 94 | `LoadDataSingleton` |
| 95 | `ConstructData` |
| 96 | `GetDataComponent` |
| 97 | `BuildStaticTuple` |
| 98 | `BuildStaticRecord` |
| 99 | `InvokeConditional` |
| 100 | `MakeConditionalFamily` |

---

# Appendix B — Instruction equivalence classes

The following equivalences are semantic, not necessarily identical in performance or diagnostic metadata.

```text
InvokeLocal(slot, n, sel)
≈ GetLocal(slot) + Invoke(n, sel)

InvokeConst(k, n, sel)
≈ Constant(k) + Invoke(n, sel)

Loop(d)
≈ Jump(d)
```

The equivalence symbol means: for valid execution states accepted by both forms, the language-visible behavior is the same.

A producer/optimizer replacing one form with another MUST preserve:

- source-attribution quality;
- lookup behavior;
- call continuation;
- runtime errors;
- stack shape;
- cache invalidation semantics.

---

# Appendix C — Required companion VM specifications

This bytecode specification deliberately delegates deeper runtime semantics to companion documents.

The authoritative VM corpus should include, at minimum:

1. **Virtual Machine Execution Model** — frames, stack, interpreter entry/exit, fibers, safepoints.
2. **Runtime Value and Object Model** — value tagging, canonical values, identity, object headers/layout abstraction.
3. **Callable, Selector, Family, and Dispatch Model** — exact selectors, method families, bound references, associated callables, conditional members.
4. **Frames, Calls, Closures, and Environments** — call layout, slot ownership, upvalues, block home frames.
5. **Exceptions, Raising, and Unwinding** — handlers, ensure/finalization, traceback construction, abrupt transfer.
6. **Modules, Packages, Imports, and Linkage** — globals, linked entries, runtime descriptors, module initialization.
7. **Classes, Traits, Enums, Data, and Runtime Type Identity** — descriptor construction, class finalization, ADT identity, applied types.
8. **Iteration Runtime** — cursor protocol and compiler-owned `None` termination.
9. **Fibers, Futures, and Scheduler** — suspension/resumption and scheduler-visible frame state.
10. **Memory Management** — roots, collection points, open upvalues, object lifetime.
11. **Reflection and Runtime Metadata** — descriptors and executable-semantic identity.
12. **Compiler-to-VM Lowering** — source construct lowering, optimizer/fusion passes, sacred selector inlining.
13. **Bytecode Validation and Security** — trusted vs untrusted bytecode boundary and verifier.
14. **Reference VM Optimization** — inline caches, opcode fusion, dispatch loop implementation, performance invariants.

Where this document refers to a runtime concept whose detailed rules live in one of those documents, the companion specification supplies the deeper algorithmic contract; it may not contradict the bytecode-level state transition specified here.

---

# Appendix D — Implementation correspondence notes

These notes are descriptive and exist to keep the normative specification auditable against the current reference implementation.

- `phalcom-core/src/bytecode.rs` defines the current `Bytecode` enum and variant-index/name correspondence.
- `phalcom-core/src/chunk.rs` defines `Chunk`, including `code`, `constants`, `spans`, cache side tables, and executable-semantic metadata.
- `phalcom-core/src/vm/dispatch.rs` contains the primary fetch/advance/dispatch execution loop and opcode handlers.
- The VM fetches one complete typed `Bytecode` instruction by instruction index.
- The active frame's `ip` advances before the handler executes.
- `Jump`, `JumpIfFalse`, `JumpIfNone`, `Loop`, and deoptimization guards therefore interpret relative offsets from the post-fetch instruction pointer.
- The reference VM is a stack machine.
- Dispatch/global inline caches are side state indexed by call/instruction site and are not part of bytecode semantics.
- `Nil` pushes surface `None`; the private raw nil sentinel remains internal.
- `ReturnNonLocal` derives its destination from the block activation's home-frame token, not from an instruction operand.
- `SuperSend` retains the original receiver and begins lookup above the statically identified defining class.
- ADT/data instructions use the executable-semantic pool to retain compiler-resolved declaration and constructor identity.

A change to the implementation that violates any normative rule above requires either an implementation fix or an explicit specification amendment.
