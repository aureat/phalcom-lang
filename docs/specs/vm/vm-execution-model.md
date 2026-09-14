# Phalcom VM Execution Model

**Status:** Draft 0.1 — authoritative implementation-internals specification  
**Specification domain:** `spec/vm/`  
**Implementation:** Phalcom reference runtime (`phalcom-core`)  
**Companion specification:** `bytecode.md`

---

## 1. Scope

This document specifies the execution architecture of the Phalcom reference virtual machine.

It defines the implementation model by which compiled Phalcom code is entered, executed, suspended, resumed, returned from, unwound, and integrated with native Rust behavior. It specifies the ownership and lifetime relationships among the VM's call frames, value stack, open upvalues, control continuations, fibers, native execution contexts, and garbage-collection safepoints.

This is an **implementation-internals specification**. It intentionally names concrete runtime structures and mechanisms where those structures are architectural contracts of the Phalcom implementation.

In particular, this document specifies:

- the execution-relevant ownership topology of `VM`;
- the current-fiber live execution mirror;
- parked fiber execution state;
- the flat `CallFrame` stack;
- frame identity and generation tokens;
- the shared per-fiber value stack and frame windows;
- selector-shaped invocation windows;
- bytecode and native callable activation;
- `CallOutcome`;
- VM-owned `ControlActivation`s and the `ControlStack`;
- return, raise, and non-local-return transfer routing;
- frame-floor execution through `run`, `run_until`, and `run_until_inner`;
- bounded native re-entry;
- fiber parking and restoration;
- the execution-loop safepoint;
- execution-state GC roots;
- stack walking and source-state relationships;
- execution invariants and resource ceilings.

This document does **not** redefine the meaning of individual bytecodes. Those semantics belong to [`bytecode.md`](bytecode.md).

It also does not fully specify:

- selector/member lookup;
- closure capture semantics;
- exception matching;
- module linkage;
- scheduler/Future policy;
- the mark/sweep algorithm;
- traceback rendering.

Those concerns have or require their own VM specifications. This document defines the execution machinery on which they depend.

---

## 2. Normative Status

Within the Phalcom implementation, the architectural invariants in this document are normative.

The terms **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** express implementation requirements.

The document distinguishes three classes of fact.

### 2.1 Architectural invariant

An architectural invariant is a property on which other runtime components may rely.

Examples include:

- the caller relationship of physical frames is represented by frame-vector order;
- open upvalues must be closed before referenced stack storage is destroyed;
- the current fiber's stack-indexed execution structures move together on a fiber switch;
- a nested `run_until` frame floor may not be invalidated by switching fibers beneath it.

Changing an architectural invariant requires an explicit VM specification amendment and an audit of dependent subsystems.

### 2.2 Current implementation mechanism

A current implementation mechanism is the concrete implementation of an architectural rule.

Examples include:

- `Vec<CallFrame>` for the call stack;
- `BTreeMap<usize, ObjRef>` for open upvalues;
- `std::mem::take` to transfer live execution vectors between `VM` and `FiberObject`.

Such mechanisms are authoritative descriptions of the current implementation but may be replaced if the replacement preserves or deliberately amends the architectural contract.

### 2.3 Optimization

An optimization improves execution cost without changing the machine's semantic state transitions.

Examples include:

- hoisting an `Rc<Callable>` in the dispatch loop;
- inline-cache side tables;
- fused bytecodes;
- optional fiber buffer pooling.

Optimizations MUST remain observationally equivalent to the unoptimized execution architecture.

---

## 3. Implementation Vocabulary

The following terms are used throughout this document.

| Term | Meaning |
|---|---|
| **VM** | One `VM` instance and its shared runtime state |
| **guest code** | Phalcom code executed by the bytecode VM |
| **native code** | Rust implementation code executing a primitive/runtime method |
| **current fiber** | The `FiberObject` identified by `VM::current` |
| **live mirror** | The current fiber's active execution structures stored directly on `VM` |
| **parked fiber** | A non-current fiber whose execution structures are stored on its `FiberObject` |
| **call frame** | One bytecode activation represented by `CallFrame` |
| **frame stack** | `VM::frames`, innermost frame last |
| **value stack** | `VM::stack`, containing invocation windows, locals, and temporaries |
| **frame floor** | Frame count at which one `run_until` drive is considered complete |
| **control activation** | One VM-owned continuation record represented by `ControlActivation` |
| **control stack** | Per-fiber stack of VM-owned control activations |
| **transfer** | A returned value, raised error, or non-local return being routed through control machinery |
| **native re-entry** | A native Rust activation recursively driving guest execution through `run_until` |
| **safepoint** | A coherent execution-loop boundary at which the VM may perform GC and ingress polling |

---

# Part I — VM ownership topology

## 4. One VM, one heap

A `VM` owns one `Heap`.

Runtime objects such as classes, instances, methods, modules, closures, strings, blocks, fibers, upvalues, and other heap objects are reached through copyable object handles rather than owning Rust references embedded through the execution stack.

`Value` and `CallFrame` are copyable runtime representations. The interpreter therefore does not require a graph of `Rc<RefCell<CallFrame>>` objects and does not encode caller links as owning pointers.

**Implementation anchor:** `phalcom-core/src/vm/mod.rs`, `phalcom-core/src/frame.rs`.

---

## 5. Execution state is partitioned by ownership

The VM's execution state is not one undifferentiated structure.

It is partitioned into:

```text
VM-global state
current-fiber live state
parked-fiber state
activation-local state
native host state
```

The distinction is load-bearing.

A field MUST be interpreted according to its ownership scope.

---

## 6. VM-global state

VM-global state is shared across fiber switches.

Representative execution-relevant VM-global state includes:

```text
heap
interner
module registry
runtime typing registry
Universe/runtime roots
class and layout registries
world version
next frame generation
next fiber sequence
ready queue
scheduler drivers
reactor
temporary GC roots
native re-entry depth
compiler-internal dispatch depth
native method contexts
```

This list is not intended to enumerate every `VM` field. It identifies categories with execution consequences.

VM-global state MUST NOT be moved into or out of a `FiberObject` merely because execution switches fibers unless the architecture is explicitly amended to make that state fiber-local.

---

## 7. The current-fiber live mirror

The current fiber's stack-indexed execution state lives directly on the `VM`.

The current live mirror consists of:

```text
VM::frames
VM::stack
VM::open_upvalues
VM::control_stack
VM::checking
```

These structures collectively belong to `VM::current`.

They are called a **live mirror** because the corresponding fields on the current `FiberObject` are empty while that fiber is running.

Conceptually:

```text
                      VM
                       │
                current: Fiber A
                       │
        ┌──────────────┴──────────────┐
        │       Fiber A live state    │
        │                             │
        │ frames                      │
        │ stack                       │
        │ open_upvalues               │
        │ control_stack               │
        │ checking                    │
        └─────────────────────────────┘

Fiber A object while Running:
    frames         = empty
    stack          = empty
    open_upvalues  = empty
    control_stack  = empty/default
    checking       = empty
```

The live mirror is authoritative for the running fiber.

---

## 8. Parked fiber state

When a fiber is not current, its execution state is stored on its `FiberObject`.

The stack-indexed parked state consists of:

```text
FiberObject::frames
FiberObject::stack
FiberObject::open_upvalues
FiberObject::control_stack
FiberObject::checking
```

A parked fiber additionally retains resident lifecycle and resumption metadata including:

```text
status
park_generation
completion_observer
is_root
consumer
result
entry
started
resume_slot
resume_destination
floor_depth
seq
spawn_file
spawn_line
```

Resident metadata is not part of the live-mirror swap merely because a fiber runs. It remains on the `FiberObject` and may be read or updated by the scheduler, consumer, or runtime.

**Implementation anchor:** `phalcom-core/src/heap/fiber.rs`.

---

## 9. Execution-state coherence family

The following fields form one coherence family:

```text
frames
stack
open_upvalues
control_stack
checking
```

They MUST move together when a running fiber is parked or a parked fiber becomes current.

The reason is structural:

- `CallFrame::stack_offset` indexes the fiber's stack;
- open upvalues are keyed by indexes into that stack;
- `ControlActivation::stack_base` and callback floors describe the same execution;
- control activation phases may retain values and destinations associated with that execution;
- `checking` represents dynamic re-entrancy state that may remain active across a yield.

Moving only a subset would permit one fiber's indexes or control state to be interpreted against another fiber's stack.

That is invalid.

---

# Part II — Fibers as execution owners

## 10. Every execution occurs inside a fiber

The VM has a distinguished root fiber.

`VM::current` always identifies the fiber whose guest execution currently owns the live mirror.

The root fiber is created as:

```text
is_root = true
status  = Running
started = true
seq     = 1
entry   = None
```

Non-root fibers are created with their own entry callable and begin in `New`.

The execution model therefore does not have a meaningful state in which ordinary guest bytecode is running "outside" a fiber.

---

## 11. Fiber lifecycle states

The current implementation defines the following lifecycle states:

```text
New
Running
BlockedOnChild
Yielded
Parked(generation)
Queued
Done
Failed
```

Their execution-model meanings are:

### `New`

The fiber has an entry callable but no entry frame has yet been pushed.

### `Running`

The fiber is `VM::current`.

Its execution structures are in the VM live mirror.

### `BlockedOnChild`

The fiber invoked a child through manual coroutine `call`/`try` and is parked until that child yields, returns, or fails.

### `Yielded`

The fiber explicitly yielded to its coroutine consumer.

Its execution state is parked and can later be resumed manually.

### `Parked(generation)`

The fiber is waiting on Future/reactor-owned progress.

Only a matching wake generation has authority to make the parked wait runnable.

### `Queued`

The scheduler has reserved the fiber for one future scheduler resume.

### `Done`

The fiber's entry activation completed normally.

`FiberObject::result` stores the terminal result.

### `Failed`

The fiber's entry escaped with an uncaught error.

`FiberObject::result` stores the captured surface `Error` value.

The concurrency specification owns admission and scheduling policy. This document owns how these states correspond to execution ownership.

---

## 12. Parking a current fiber

`store_live_into(vm, fiber)` moves the live mirror into the specified fiber.

The operation is structurally:

```text
VM.frames          ──move──► Fiber.frames
VM.stack           ──move──► Fiber.stack
VM.open_upvalues   ──move──► Fiber.open_upvalues
VM.control_stack   ──move──► Fiber.control_stack
VM.checking        ──move──► Fiber.checking
```

The implementation currently performs these moves using `std::mem::take`.

After the operation, the corresponding VM containers are empty until another fiber is loaded.

The caller MUST set lifecycle metadata consistently with the reason the fiber is being parked.

**Implementation anchor:** `phalcom-core/src/primitive/fiber.rs::store_live_into`.

---

## 13. Loading a parked fiber

`load_live_from(vm, fiber)` is the inverse operation.

Conceptually:

```text
Fiber.frames          ──move──► VM.frames
Fiber.stack           ──move──► VM.stack
Fiber.open_upvalues   ──move──► VM.open_upvalues
Fiber.control_stack   ──move──► VM.control_stack
Fiber.checking        ──move──► VM.checking
```

The parked containers become empty.

The implementation MUST NOT duplicate these structures into both locations as independently mutable copies.

There is one active owner.

---

## 14. Fiber-relative stack indexes

A fiber's stack buffer begins at index zero regardless of whether it is resident in `VM::stack` or parked in `FiberObject::stack`.

Therefore:

- `CallFrame::stack_offset`;
- open-upvalue slot indexes;
- control stack bases;
- resume slots;

do not require rebasing when the buffer is moved between the VM and the fiber.

This invariant is a central reason the entire stack-indexed coherence family moves together.

---

# Part III — Frames and activation identity

## 15. The physical call stack

`VM::frames` is a `Vec<CallFrame>`.

The innermost currently executing bytecode frame is last.

The physical caller relationship is represented by vector order:

```text
frames[0]      outermost activation
frames[1]      called by frames[0]
frames[2]      called by frames[1]
...
frames[n - 1]  innermost activation
```

There is no general caller pointer embedded in a `CallFrame`.

For a frame at index `i > 0`, its physical caller is `frames[i - 1]`.

---

## 16. `CallFrame`

The current call-frame representation is conceptually:

```rust
CallFrame {
    closure,
    context,
    ip,
    stack_offset,
    caller_source,
    generation,
    home_frame_token,
    foreign_receiver_guard,
    type_environment,
}
```

The fields have the following execution meanings.

### `closure`

Handle to the `ClosureObject` whose callable/chunk is executing.

### `context`

Receiver context used for `self` and receiver-dependent operations.

### `ip`

Instruction index of the next instruction to fetch from the closure's chunk.

### `stack_offset`

Absolute index, within the owning fiber's stack, at which this activation's physical window begins.

The window begins at its receiver.

### `caller_source`

Source range of the call site that created this activation.

It exists for stack traces and diagnostic attribution.

### `generation`

Monotonically assigned activation generation.

It disambiguates reuse of a frame-vector index.

### `home_frame_token`

For block invocations, identifies the lexical home activation through which a non-local `return` returns.

Ordinary method and closure activations leave this unset.

### `foreign_receiver_guard`

Optional representation/layout guard required when bytecode behavior is executed on a receiver outside its nominal holder hierarchy.

### `type_environment`

Runtime type-substitution environment active for this activation.

The runtime typing specification owns the contents of that environment. This document owns its placement and activation lifetime.

**Implementation anchor:** `phalcom-core/src/frame.rs`.

---

## 17. Receiver context

`CallContext` has four execution forms:

```text
Instance { instance }
Class    { class }
Module   { module }
Immediate{ value }
```

### Instance context

Represents a bytecode method executing against a user-defined instance.

### Class context

Represents class-side bytecode behavior.

### Module context

Represents top-level module execution.

### Immediate context

Represents bytecode-backed behavior executing against an immediate value such as a boolean, number, or symbol.

This variant exists because an immediate receiver has no heap `ObjRef` that can represent `self`.

A frame therefore MUST NOT assume that every receiver context is a heap-object handle.

---

## 18. Creating frames

Every live activation MUST receive a fresh generation.

The normal helper is `VM::new_call_frame`, which:

```text
generation = VM.next_frame_generation
VM.next_frame_generation += 1
construct CallFrame
stamp generation
```

Any alternate frame-construction path, including top-level/module entry, MUST obey the same generation rule.

The architectural invariant is fresh activation identity, not exclusive use of one helper function.

---

## 19. Frame depth limit

Before pushing a frame, the VM enforces:

```text
MAX_CALL_DEPTH = 10_000
```

A push at or beyond that depth fails with `RuntimeError::DepthExceeded`.

The exact configured constant is an implementation resource policy, but the separation between guest frame depth and native re-entry depth is architectural.

Guest frames reside in a VM-managed vector and do not recursively consume one Rust stack frame per Phalcom call.

---

## 20. Frame position is not activation identity

A frame-vector index may later be reused.

Therefore a stable activation reference is:

```text
FrameToken {
    frame_index,
    generation,
}
```

A token is live only if:

```text
frames[token.frame_index].generation == token.generation
```

after verifying that the index remains in range.

The implementation MUST perform the liveness/generation check before mutating execution state for an operation that depends on the referenced activation.

This rule prevents a stale token from accidentally targeting an unrelated activation that later occupied the same vector position.

---

# Part IV — Value stack and invocation windows

## 21. One value stack per fiber

The running fiber's value storage is `VM::stack`.

A parked fiber's corresponding storage is `FiberObject::stack`.

`Value`s are copyable runtime values.

The stack carries:

- invocation receivers;
- positional arguments;
- labeled argument values;
- distinguished setter values;
- local slots;
- compiler temporaries;
- intermediate bytecode results;
- builder/intermediate runtime values.

The exact interpretation of a stack location depends on its owning frame and instruction sequence.

---

## 22. Frame windows

A frame's `stack_offset` identifies the start of its window.

The first value in an ordinary callable window is the receiver.

Conceptually:

```text
lower stack
────────────────────────────────
caller state
────────────────────────────────
receiver                  ← frame.stack_offset / receiver_index
structural positional 0
...
structural positional N-1
structural labeled value 0
...
structural labeled value M-1
setter value?             ← setter-shaped invocations only
callee locals
callee temporaries
...
────────────────────────────────
stack top
```

The precise number of values after the receiver is described by the invocation layout.

---

## 23. Invocation layout

`InvocationLayout` describes the physical argument lanes for one selector-shaped invocation.

It records:

```text
structural_positionals
labels
setter_value
```

`physical_arity` is:

```text
structural_positionals
+ labels.len()
+ (1 if setter_value else 0)
```

The distinguished setter lane is not counted as an ordinary positional argument.

Accessor layouts are constrained by signature kind.

The VM MUST validate that an invocation layout is compatible with the selected method's signature kind.

**Implementation anchor:** `phalcom-core/src/method/object.rs`.

---

## 24. Argument views

`ArgumentView` is a compact, non-borrowing description of an active invocation window.

It stores:

```text
receiver_index
InvocationLayout
caller access authority
caller internal-authority flag
```

The view does not copy the argument values.

It indexes the values already present in `VM::stack`.

This permits native shape-aware gateways to inspect:

- positional values;
- labeled values;
- labels;
- the distinguished setter value;

without allocating an argument collection for ordinary calls.

An `ArgumentView` MUST NOT outlive the validity of the stack window it describes.

---

## 25. Local addressing

Bytecode local slot addressing is relative to the active frame's `stack_offset`.

Conceptually:

```text
physical_stack_index = frame.stack_offset + local_slot
```

The compiler and VM MUST agree on which physical positions correspond to receiver, parameters, locals, and temporaries.

A local access beyond the valid active stack window is an internal bytecode/runtime invariant failure.

---

# Part V — Execution entry and frame floors

## 26. Direct module execution

The direct `run_in_module(module, closure)` path:

1. clears the current live frame stack;
2. clears the current live value stack;
3. creates a `CallFrame` with:
   - `CallContext::Module`;
   - `ip = 0`;
   - `stack_offset = 0`;
   - no caller source;
   - a fresh generation;
4. pushes the frame;
5. calls `run()`.

This path is the direct execution entry used by `interpret_source`.

Compiled-program initialization and module-loading subsystems may establish entry frames through additional orchestration, but guest execution ultimately obeys the same frame/stack rules.

**Implementation anchor:** `phalcom-core/src/interpret.rs::run_in_module`.

---

## 27. `run`

`VM::run()` is:

```text
run_until(0)
```

A zero frame floor means root/top-level execution.

`run` is therefore not a separate interpreter.

---

## 28. Frame floors

A **frame floor** is the number of call frames owned by the caller of one interpreter drive.

For a drive with:

```text
base_frames = N
```

`run_until_inner` executes guest frames while:

```text
frames.len() > N
```

When execution drains back to `N`, the nested drive is complete.

A frame floor permits native code to synchronously drive guest execution without draining frames that existed before the native callback began.

---

## 29. `run_until(base_frames)`

`run_until` has two execution modes.

### 29.1 Nonzero frame floor

When:

```text
base_frames != 0
```

the implementation directly delegates to:

```text
run_until_inner(base_frames)
```

The fiber-floor scheduler/failure wrapper is bypassed.

This is the ordinary mode for bounded native re-entry.

### 29.2 Zero frame floor

When:

```text
base_frames == 0
```

`run_until` is the fiber-aware root execution driver.

It repeatedly drives:

```text
run_until_inner(0)
```

and handles the terminal result or failure of the currently running fiber.

This distinction is architectural.

---

## 30. `run_until_inner`

`run_until_inner` is the central typed-bytecode dispatch loop.

Its frame-floor termination rule is checked before each instruction cycle.

If:

```text
frames.len() <= base_frames
```

the drive is complete.

If no explicit result remains, the implementation surfaces absence as `None` rather than exposing the private raw nil sentinel.

The control structure of `run_until_inner` does not perform coroutine scheduling. Fiber completion/switch orchestration lives around it in `run_until(0)` and fiber primitives.

Individual bytecode handlers may nevertheless be fiber-aware when required by their data, notably open-upvalue access.

---

# Part VI — Root execution driver

## 31. Root execution is more than frame drainage

`run_until(0)` is not equivalent to:

```text
while frames not empty:
    execute
return
```

It also owns the top-level fiber floor and scheduler/reactor drive.

When `run_until_inner(0)` returns successfully, the wrapper determines which fiber finished.

---

## 32. Root fiber completion

If the finished fiber is the root fiber, the root result is retained independently of scheduler-pump results.

The driver then attempts to make further progress through runtime work, including current mechanisms such as:

- queued fibers;
- reactor completion delivery;
- reactor progress/idle waiting;
- scheduler failure reporting.

The root drive returns only after the implementation reaches its quiescent completion condition.

A scheduled child result MUST NOT replace the root program's own result merely because the scheduler executed after the host/root activation drained.

---

## 33. Non-root fiber completion

If a non-root fiber's entry drains normally:

1. its terminal status becomes `Done`;
2. its terminal result is stored;
3. its completion observer, if any, is detached and scheduled through the scheduler path;
4. a coroutine consumer, if any, receives the value;
5. otherwise scheduler/reactor progression determines the next current fiber or eventual root completion.

The concurrency specification owns the policy details.

This document establishes that non-root frame drainage is a **fiber-floor event**, not immediate termination of the whole VM.

---

## 34. Fiber failure at the zero floor

An error escaping `run_until_inner(0)` first interacts with VM control continuations if a control stack is active.

If still unhandled at the fiber floor, the execution driver:

- captures a surface error value;
- closes live upvalues before destroying affected execution storage;
- marks the failing fiber `Failed`;
- records the terminal error;
- handles any completion observer;
- delivers according to coroutine `Call` versus `Try` mode or scheduler ownership;
- may propagate failure through a consumer chain;
- may continue scheduler/reactor work;
- reports an unhandled root failure to the host.

The exact scheduler/failure-observation policy belongs to the concurrency specification.

---

# Part VII — Fetch–advance–execute

## 35. Instruction cycle

The central instruction cycle is:

```text
1. test frame floor
2. service the execution safepoint
3. copy/read the active CallFrame
4. obtain closure id, ip, stack offset
5. resolve or reuse the executing Callable
6. fetch code[ip]
7. record optional instrumentation
8. increment the active frame ip
9. dispatch the instruction
10. return to loop head
```

The instruction sequence is a typed `Vec<Bytecode>`.

The IP is an instruction index, not a byte offset.

---

## 36. Pre-dispatch IP advancement

The active frame's `ip` is incremented before the instruction handler executes.

If the fetched instruction was at:

```text
ip0
```

the active caller frame holds:

```text
ip0 + 1
```

while the handler runs, unless the instruction modifies it further.

This rule has several consequences.

### 36.1 Fallthrough

Ordinary instructions automatically continue at the next instruction.

### 36.2 Relative branches

Branch offsets are applied to the already advanced IP.

### 36.3 Call continuation

When an invocation pushes a callee frame, the caller's current IP is already its return continuation.

### 36.4 Traceback source position

For a suspended or active frame whose current `ip` points after the most recently executed/failing call site, stack walking commonly maps:

```text
ip.saturating_sub(1)
```

to source.

The debugging specification owns full rendering semantics.

---

## 37. Callable hoist

The current dispatch loop hoists the active closure's immutable `Rc<Callable>` in a loop-local cache keyed by closure handle.

The optimization avoids re-resolving the callable through the heap on every opcode.

The guard intentionally does **not** hoist `ip`.

Two fibers may execute the same closure at different instruction positions. A fiber switch may therefore leave the closure identity unchanged while changing the live frame and IP.

Any optimization that hoists frame-specific state across loop iterations MUST guard by activation identity, not merely closure identity.

This is an optimization constraint, not a source-language semantic rule.

---

# Part VIII — Garbage-collection execution boundary

## 38. The dispatch safepoint

`run_until_inner` services:

```text
service_gc_safepoint()
```

at the loop boundary before the active frame is copied and before the next instruction begins.

This is the coherent collection point for ordinary execution.

The placement is deliberate.

At that boundary:

- stack and frame ownership are coherent;
- no opcode is part-way through holding a popped live value only in an unrooted Rust local;
- active control state is visible through VM-owned structures;
- the current fiber's roots are in the live mirror.

The read/decode/execute region is deliberately collection-free under the ordinary allocation latch policy.

---

## 39. Allocation latching

Heap allocation MUST NOT trigger arbitrary collection in the middle of an opcode if doing so would make Rust locals invisible to root enumeration.

Instead, ordinary allocation may latch that collection is due, and the dispatch safepoint services the pending collection.

The memory-management specification owns the complete collector policy.

---

## 40. Reactor ingress at the safepoint

The current safepoint also services bounded reactor ingress polling before checking whether GC is due.

This is an implementation integration point: external completion ingress is admitted at a coherent VM boundary rather than racing guest execution.

The concurrency/reactor specification owns the external event model.

---

## 41. Temporary native roots

The interpreter stack and frames describe values held by guest execution.

They do not describe an object handle held only in a Rust local inside a native primitive.

This matters when native code re-enters guest execution, because the nested dispatch loop reaches a GC safepoint.

`VM::temp_roots` therefore provides an explicit root escape hatch.

Native code that holds an otherwise unreachable heap object across re-entrant execution MUST root that handle before re-entry and release the temporary root afterward on every path.

A native local held only across allocations, without re-entering a safepoint-capable guest drive, does not require the same mechanism under the allocation-latch invariant.

---

# Part IX — Callable activation

## 42. Selected method representations

A `MethodObject` is implemented by one of two major strategies:

```text
Closure
Primitive
```

Closure-backed methods enter guest bytecode.

Primitive methods execute Rust code.

Method lookup is specified elsewhere; this section begins after a concrete callable target has been selected.

---

## 43. Closure-backed method activation

For an ordinary closure-backed method:

1. the receiver's `CallContext` is derived;
2. the physical receiver index is determined from the current invocation window;
3. `stack_offset` is set to that receiver index;
4. a new `CallFrame` is created at `ip = 0`;
5. the call-site source range is attached;
6. the frame is pushed;
7. the existing dispatch loop continues.

The interpreter is not recursively invoked merely because a Phalcom method called another Phalcom method.

That is the normal flat-loop call path.

---

## 44. First-class callable activation

The sealed Function hierarchy may activate runtime representations including:

- `Block`;
- `Closure`;
- `BoundMethod`;
- `Family`;
- `AssociatedFamily`;
- `BoundMethodFamily`.

The concrete representation determines how activation proceeds.

Closure/block activation ultimately enters a bytecode frame.

Families and bound callables may select another behavior and then enter frame, control, native, or switch paths.

Detailed callable-family selection belongs to the dispatch/callables VM specification.

---

## 45. Block activation state

A block-backed activation may additionally install:

```text
home_frame_token
foreign_receiver_guard
type_environment
```

onto its `CallFrame`.

The block's home-frame token is the basis for non-local return.

The block's type environment preserves the runtime generic environment captured at block creation.

---

# Part X — Native execution

## 46. Native method contexts

Native primitives do not push a bytecode frame solely to represent their Rust body.

Therefore the caller's bytecode frame is not sufficient to represent the primitive's own lexical access authority.

For the duration of a native method body, the VM may push a `NativeMethodContext` containing:

```text
access_owner
internal
```

Lookup/access checks inside the primitive use that context.

The context is removed before forwarded guest execution resumes as ordinary ambient native authority.

This prevents private/protected/internal sends performed inside a primitive from being misclassified as originating from the primitive's caller.

---

## 47. Compiler-internal dispatch authority

Compiler-generated helper sends may temporarily increment:

```text
compiler_internal_dispatch_depth
```

for the dispatch operation itself.

The capability authorizes the dispatch boundary.

It MUST NOT become ambient privilege inherited indefinitely by called user code.

---

## 48. Native ABI generations

The implementation currently contains two primitive ABI shapes.

### 48.1 Legacy primitive ABI

Legacy primitives return:

```text
PhResult<Value>
```

The runtime reconciles execution after the call using:

- `switch_pending`;
- frame-depth comparison;
- stack-window collapse.

This path is compatibility machinery.

### 48.2 Shape-aware primitive ABI

Shape-aware primitives receive:

```text
Value receiver
ArgumentView
```

and return:

```text
PhResult<CallOutcome>
```

This ABI makes control transitions explicit.

The implementation specification MUST describe both while both ship.

It MUST NOT pretend that the migration is already complete.

---

# Part XI — `CallOutcome`

## 49. Activation result vocabulary

`CallOutcome` has four forms:

```text
Returned(Value)
EnteredFrame
EnteredControl
SwitchedFiber
```

It describes what activating a callable did to VM execution state.

---

## 50. `Returned(Value)`

The native operation completed synchronously.

The runtime collapses/reconciles the invocation window and delivers the returned value unless another execution transfer already invalidated the old window.

---

## 51. `EnteredFrame`

The operation established a bytecode `CallFrame`.

The current dispatch loop MUST continue with that frame.

A caller MUST NOT recursively start another interpreter merely because the outcome is `EnteredFrame` unless it is specifically a native synchronous boundary that requires the result before returning to its own Rust caller.

---

## 52. `EnteredControl`

The operation established a VM-owned control continuation.

The runtime MUST continue execution through the control machinery rather than treating the native call as synchronously complete.

---

## 53. `SwitchedFiber`

Execution ownership moved to another fiber.

Any stack indexes captured against the pre-switch live mirror are invalid for the now-current fiber.

The caller MUST NOT perform ordinary post-call window collapse using those old indexes.

---

## 54. Legacy switch signalling

Legacy primitive reconciliation still uses:

```text
VM::switch_pending
```

as an explicit VM-side signal that a primitive switched fibers.

The legacy path also compares frame depth to distinguish an ordinary native return from a non-local return that removed frames during nested guest execution.

Shape-aware code has the explicit `CallOutcome::SwitchedFiber` outcome, but current call machinery still retains `switch_pending` for compatibility and reconciliation.

Until the legacy ABI is removed, both mechanisms are part of the implementation.

A future cleanup that removes `switch_pending` MUST audit every legacy primitive and every reconciliation path before amending this specification.

---

# Part XII — VM-owned control continuations

## 55. Why a control stack exists

Some language/runtime operations must:

1. retain semantic continuation state;
2. invoke arbitrary guest code;
3. resume afterward;
4. possibly react to return, raise, or non-local return.

Keeping a Rust stack frame suspended across arbitrary guest execution would couple language control flow to native recursion and conflict with fiber suspension.

Phalcom therefore represents such continuations as VM-owned data.

`ControlActivation`s live in a per-fiber `ControlStack`.

Examples of operations currently represented through control phases include:

- `Block.on`;
- `Block.ensure`;
- `Block.whileTrue`;
- Boolean branch callbacks;
- Option branch callbacks;
- ordering reversal callbacks.

---

## 56. `ControlStack`

Conceptually:

```rust
ControlStack {
    records: Vec<ControlActivation>,
    pending: Option<RoutedTransfer>,
    next_id: u64,
}
```

The stack is fiber-owned execution state.

It moves with the fiber during parking/resumption.

`next_id` supplies monotonically increasing control IDs within the fiber's control stack.

---

## 57. `ControlActivation`

A control activation contains:

```text
id
owner
stack_base
callback_floor
caller_authority
source_range
destination
phase
```

### `id`

Unique `ControlId` within the control stack's identity domain.

### `owner`

Optional `FrameToken` tying the continuation to a guest activation.

### `stack_base`

Stack length/index to which protected/callback execution can be unwound.

### `callback_floor`

Frame depth corresponding to the callback activation owned by this control operation.

### `caller_authority`

Captured access/internal authority for calls initiated as part of the control protocol.

### `source_range`

Source association for runtime diagnostics and callback invocation.

### `destination`

Explicit location to which a completed control result is delivered.

### `phase`

State-machine phase describing what the continuation is currently waiting for.

---

## 58. Control destinations

A control result is not always equivalent to "push a value at stack top".

The current destination vocabulary is:

```text
StackOperand { target_index }
ControlActivation { id }
```

A control continuation may therefore deliver to:

- a precise stack operand position; or
- another VM control activation.

Destination identity MUST remain valid for the lifetime of the control operation.

---

## 59. Control phases

The current `ControlPhase` set includes:

```text
OnBody
OnMatch
OnHandler
EnsureBody
EnsureCleanup
WhileCondition
WhileBody
BoolBranch
OptionBranch
OrderingReverse
```

A phase may retain values required after guest execution returns.

Those retained heap values are part of the GC root set through `ControlStack::trace_roots`.

The detailed language semantics of each phase belong to the relevant exception/control-flow specifications.

---

# Part XIII — Transfer routing

## 60. Unified transfer vocabulary

Control continuations reason about three major guest execution transfers:

```rust
Transfer::Returned(Value)
Transfer::Raise(PhError)
Transfer::NonLocalReturn {
    target: FrameToken,
    value: Value,
}
```

This vocabulary separates **what happened to guest execution** from **what an enclosing control construct does about it**.

---

## 61. `Transfer::Returned`

A guest callback returned normally.

A control phase may:

- accept the value and complete;
- transform the operation into another phase;
- invoke another callback;
- deliver to an enclosing destination.

---

## 62. `Transfer::Raise`

Guest execution raised an error.

A control phase may:

- inspect/match it;
- run an `ensure` cleanup;
- replace it with a later failure;
- propagate it outward.

Detailed catch/match rules belong to the exception specification.

---

## 63. `Transfer::NonLocalReturn`

A block initiated a non-local return to a `FrameToken`.

Control continuations such as `ensure` must have the opportunity to process the transfer before the final multi-frame unwind executes.

Therefore a non-local return is not always immediately applied to the frame stack.

---

## 64. Control reducer

`VM::step_control_transfer` drives the control stack.

Conceptually:

```text
incoming Transfer
      │
      ▼
pop top ControlActivation
      │
      ▼
interpret current phase
      │
      ├── deliver value and finish
      ├── update phase and push activation back
      ├── enter guest callback
      ├── transform transfer
      └── propagate transfer outward
```

The result is one of:

```text
Continued
Completed(Value)
Propagate(Transfer)
```

### `Continued`

The reducer entered more guest work, pushed/re-established control, or switched execution such that the dispatch engine should continue.

### `Completed(Value)`

The control operation completed and delivered a value.

### `Propagate(Transfer)`

No active control record consumed the transfer; it must be handled by the outer execution machinery.

---

# Part XIV — Return paths

## 65. Ordinary bytecode return

The ordinary `Return` execution path is structurally:

```text
return_value = pop stack or private nil sentinel
return_value = surface absence

popped_frame = frames.pop()

close_upvalues_from(popped_frame.stack_offset)
stack.truncate(popped_frame.stack_offset)

if a ControlActivation callback floor now matches:
    route Transfer::Returned(return_value)
else if frame floor has drained:
    return return_value from run_until_inner
else:
    push return_value for caller
```

The exact opcode definition belongs to `bytecode.md`.

This document specifies the activation destruction sequence.

---

## 66. Close before truncate

The ordering:

```text
close upvalues
then truncate stack
```

is mandatory.

An open upvalue may still read the outgoing frame's stack slot.

Destroying that slot before copying its value into the closed cell would invalidate an escaping closure.

This invariant applies to:

- ordinary return;
- non-local return;
- caught-error unwind;
- parked-fiber failure cleanup;
- any future execution mechanism that destroys stack storage.

---

## 67. Result placement

After an ordinary return that remains inside the same interpreter drive, the callee's entire stack window is removed and one result value is placed for the caller.

If the returned activation was the last frame above the active frame floor, the value is returned from `run_until_inner` rather than pushed for a guest caller that is outside that drive.

---

# Part XV — Non-local return

## 68. Home-frame identity

A block activation carries:

```text
home_frame_token: Some(FrameToken)
```

A `ReturnNonLocal` uses that token as its lexical destination.

The token is not a caller pointer.

It identifies the activation through:

```text
frame index + generation
```

---

## 69. Liveness check

Before mutating stack or frame state, the VM verifies:

```text
token.frame_index exists
and
frames[token.frame_index].generation == token.generation
```

If not, the lexical home activation is dead and `DeadFrameError` is raised.

The check-before-mutation order is required so that a catchable dead-frame failure leaves the existing execution stack coherent.

---

## 70. Interaction with control activations

If the current fiber has active control continuations, a non-local return is first represented as:

```text
Transfer::NonLocalReturn
```

and routed through `step_control_transfer`.

This allows constructs such as `ensure` to run before the lexical return is finalized.

Only a propagated non-local-return transfer executes the final frame unwind.

---

## 71. Applying the non-local return

`execute_non_local_return`:

1. revalidates target liveness;
2. obtains the home frame's `stack_offset`;
3. surfaces the result value;
4. closes open upvalues from that offset;
5. truncates the stack to the home offset;
6. places the return value;
7. truncates the frame vector to remove the target home activation and all activations above it.

The result is therefore delivered as the result of the home activation, not merely the immediate block activation.

---

# Part XVI — Generic unwind

## 72. `unwind_to`

The common caught-error/unwind primitive takes relative snapshots:

```text
stack_len
frames_len
```

and executes:

```text
close_upvalues_from(stack_len)
frames.truncate(frames_len)
stack.truncate(stack_len)
```

Snapshots are relative to the currently owning fiber's stack/frame buffers.

They MUST NOT be interpreted as global indexes across fibers.

---

## 73. Error paths and retained frames

An error escaping a nested guest operation may temporarily leave throwing frames live so traceback capture can observe them.

Once a control handler decides to catch and resume, abandoned frames must be explicitly unwound to the protected operation's saved snapshots.

This distinction permits complete uncaught tracebacks without allowing caught abandoned frames to remain live.

---

# Part XVII — Upvalues

## 74. Open upvalue map

The running fiber's open upvalues are stored as:

```text
BTreeMap<stack_index, upvalue_cell>
```

The map enforces one shared open cell for repeated captures of the same live stack slot.

An open upvalue cell identifies:

```text
owning fiber
stack slot
```

not merely a slot number.

That owner identity is required because the closure using the upvalue may run while the defining fiber is parked.

---

## 75. Capturing a local

When a local at absolute stack index `i` is first captured:

1. the VM checks the current fiber's `open_upvalues`;
2. if a cell already exists for `i`, it is reused;
3. otherwise a new `Upvalue::Open { fiber: current, slot: i }` is allocated;
4. the cell is registered under `i`.

Closures capturing the same live local therefore share mutation through one cell.

---

## 76. Reading an open upvalue

If:

```text
upvalue.fiber == VM::current
```

the slot is read from:

```text
VM::stack
```

Otherwise it is read from:

```text
heap.fiber(upvalue.fiber).stack
```

because the owning fiber is parked.

This is why the inner bytecode loop's control structure can be fiber-neutral while some opcode handlers remain necessarily fiber-aware.

---

## 77. Closing upvalues

`close_upvalues_from(from)` closes every live open upvalue whose stack index is at least `from`.

For each:

1. copy the stack value;
2. replace the heap cell with `Upvalue::Closed(value)`;
3. remove the cell from the live open-upvalue map.

A corresponding parked-fiber helper performs the same operation against a parked fiber's own stack and open-upvalue map.

---

# Part XVIII — Native re-entry

## 78. Why native re-entry exists

The normal guest call model is flat:

```text
push guest frame
continue same interpreter loop
```

However, some native runtime operations require a guest result synchronously before the Rust function can complete.

Such a native boundary may:

1. snapshot the current frame depth;
2. activate guest work;
3. recursively call `run_until(base_frames)`;
4. wait until the added guest activation region drains;
5. resume native processing.

This is **native re-entry**.

---

## 79. Native re-entry depth

`VM::native_reentry_depth` counts nested re-entrant interpreter drives currently present on the real Rust stack.

Before beginning such a drive, native code MUST enforce the configured ceiling.

The normal shape is:

```text
check_native_reentry()
native_reentry_depth += 1
result = run_until(base_frames)
native_reentry_depth -= 1
```

All exit paths MUST restore the counter.

---

## 80. Native re-entry ceiling

The current ceiling is:

```text
MAX_NATIVE_REENTRY = 32
```

This is intentionally much lower than `MAX_CALL_DEPTH`.

The reason is resource ownership:

```text
guest CallFrame
    lives in VM-managed Vec
    does not consume a recursive Rust interpreter frame

native run_until re-entry
    lives on the Rust stack
    consumes host stack space
```

The two limits MUST remain conceptually separate even if their configured values later change.

---

## 81. Fiber switching restriction

A nested `run_until(base_frames)` computed its frame floor against the currently running fiber's frame vector.

A fiber switch replaces that vector.

Therefore a fiber switch underneath a nested native drive would make:

```text
base_frames
```

refer to a depth in the wrong fiber.

That is invalid.

The implementation therefore prohibits fiber switching across active native re-entry.

This rule applies to both:

- yielding the current fiber;
- resuming/switching to another fiber.

The restriction is an execution-integrity constraint, not merely concurrency policy.

---

## 82. Fiber floor depth

A running fiber records the native re-entry depth at which it began/resumed as its `floor_depth`.

A yield/park operation verifies that the current `native_reentry_depth` still equals that recorded floor.

This detects a new native re-entrant boundary introduced since the fiber began executing.

Manual resume additionally rejects switching while nonzero native re-entry is active.

---

# Part XIX — Manual coroutine switching

## 83. Resuming a fiber

Manual `Fiber#call` / `Fiber#try` uses the shared `fiber_resume` engine.

Before mutating execution state, the runtime validates:

- no forbidden native re-entry is active;
- the receiver is a fiber;
- terminal/running/blocked/parked/queued states are not resumed illegally;
- first-entry argument shape is acceptable.

Validation before mutation prevents partial state transitions.

---

## 84. Parking the caller

For a valid manual resume:

1. determine the caller's invocation receiver index;
2. store it as the caller's `resume_slot`;
3. mark the caller `BlockedOnChild`;
4. move the caller's live state into its `FiberObject`;
5. record the caller as the callee's `consumer` together with `Call` or `Try` mode.

After this point, the previous live stack belongs to the parked caller.

Captured indexes from it MUST NOT be used against the new current fiber.

---

## 85. First resume of a new fiber

If the callee is `New`:

1. resolve its entry Block/Closure;
2. prepare its bound entry arguments;
3. establish its initial stack window;
4. push the entry callable/receiver and arguments;
5. create the entry `CallFrame`;
6. attach any block home-frame token;
7. mark the fiber started.

The new fiber's execution state is built directly in the now-empty VM live mirror.

---

## 86. Resume of a yielded fiber

If the callee has already started:

1. load its parked state into the VM live mirror;
2. obtain its stored `resume_slot`;
3. truncate the stack to that slot;
4. push the value delivered by the new resume.

This replaces the suspended call/yield window with its resume result.

---

## 87. Completing the switch

After establishing the target execution state:

```text
target.status = Running
target.floor_depth = current native_reentry_depth
VM.current = target
```

The legacy switch signal is set so legacy/native reconciliation does not touch the new fiber using old invocation indexes.

---

## 88. Yield

A manual yield requires:

- the current fiber is not root;
- it has a coroutine consumer;
- that consumer is actually blocked on this child;
- no forbidden native re-entry has appeared since the fiber's floor was established.

The current fiber then:

1. clears/consumes its active consumer link as required by the protocol;
2. records the yield call's receiver index as `resume_slot`;
3. becomes `Yielded`;
4. parks its live execution state;
5. restores the resumer fiber;
6. replaces the resumer's blocked call window with the yielded value;
7. marks the resumer `Running`.

---

# Part XX — Future-owned parking

## 89. Future park generation

Future-owned suspension is distinguished from manual `yield`.

A fiber records a monotonically changing `park_generation`.

A Future wake must carry authority corresponding to the exact generation that was parked.

This prevents stale wake events from reviving a newer park episode.

The concurrency specification owns Future/wake semantics.

---

## 90. Park prepare and commit

The implementation separates validation/generation preparation from park commit.

Before recording a Future waiter, the runtime validates that a switch can legally occur.

On commit:

- the fiber records its resume slot;
- status becomes `Parked(generation)`;
- the live mirror is parked;
- scheduler/driver/quiescence logic determines which execution becomes current next.

This sequencing prevents an external wait owner from being recorded if suspension would later be rejected for an already-known local reason.

---

# Part XXI — Native post-call reconciliation

## 91. Why reconciliation exists

A native primitive may mutate execution in ways not represented by an ordinary returned `Value`.

Examples include:

- entering a bytecode frame;
- establishing a control continuation;
- switching fibers;
- triggering a non-local return through nested guest execution.

The caller therefore cannot blindly execute:

```text
truncate original receiver window
push native return value
```

after every primitive.

---

## 92. Shape-aware reconciliation

Shape-aware primitives communicate the dominant transition through `CallOutcome`.

### Returned

Perform ordinary result reconciliation unless legacy switch/non-local state already invalidated the old window.

### EnteredFrame

Leave the guest activation live and continue dispatch.

### EnteredControl

Leave VM control continuation state live and continue dispatch/control processing.

### SwitchedFiber

Do not touch the original invocation window; it belongs to a parked fiber.

---

## 93. Legacy reconciliation

Legacy primitives do not return `CallOutcome`.

The runtime records:

```text
frames_before
receiver_idx
switch_pending = false
```

before the call.

After successful return:

1. if `switch_pending`, clear it and do not reconcile the old window;
2. else if current frame depth is at least `frames_before`, perform ordinary receiver/args collapse;
3. else a non-local return removed frames beneath the native call, so do not truncate by the now-stale old receiver index; route/push the returned value according to the surviving execution state.

This logic is compatibility-specific and should shrink as legacy primitives migrate.

---

# Part XXII — Execution and control-stack integration

## 94. Callback floor

A `ControlActivation` records a `callback_floor`.

When an ordinary bytecode `Return` pops a frame and the current frame count equals the top control activation's callback floor, the returned value belongs to that control continuation.

The VM therefore routes:

```text
Transfer::Returned(value)
```

into the reducer instead of immediately treating it as an ordinary caller result.

This is the bridge between physical call frames and VM-owned continuations.

---

## 95. Raised errors

An error escaping the inner dispatch loop at the root drive is similarly offered to the current control stack as:

```text
Transfer::Raise(error)
```

before the error is treated as a terminal fiber failure.

This permits catch/ensure/control machinery to resume execution without requiring the host Rust call stack to remember the continuation.

---

## 96. Non-local returns

A `ReturnNonLocal` is offered to the control stack as:

```text
Transfer::NonLocalReturn
```

before the final lexical-home frame truncation.

This permits cleanup continuations to run while preserving the original transfer as pending control state.

---

# Part XXIII — GC roots from execution

## 97. Running-fiber roots

The running fiber's `FiberObject` contains empty parked execution buffers.

Therefore the running guest roots are enumerated from the VM live mirror.

Execution roots include:

- heap handles reachable through active `CallFrame`s;
- heap values on `VM::stack`;
- values/objects retained by `VM::control_stack`;
- open-upvalue cells;
- `VM::current`;
- the root fiber handle;
- fiber-scoped `checking` references.

---

## 98. Parked-fiber roots

Parked fibers are themselves heap objects.

Their internal tracing must retain:

- parked stack values;
- parked frames;
- parked open upvalues;
- parked control stack values/transfers;
- consumer/completion links;
- entry callable;
- other resident heap references.

The memory-management specification owns recursive object tracing.

---

## 99. Scheduler/native roots

Execution-adjacent global roots include current structures such as:

- ready queue fibers;
- scheduler driver fibers;
- unhandled scheduler failure records;
- temporary native roots;
- reactor-owned heap references where applicable.

Root enumeration is intentionally exhaustive over `VM` fields so new fields cannot silently evade classification.

---

# Part XXIV — Stack walking and diagnostics

## 100. Stack walking uses execution frames

`StackWalk` walks the actual physical call-frame storage rather than a separately maintained caller graph.

The current live walk:

```text
VM::frames
oldest → newest
```

produces logical `FrameView`s for traceback/observability consumers.

Today one physical `CallFrame` expands to one logical frame.

The implementation keeps an expansion seam so future frame inlining can expose multiple logical frames from one physical frame without changing every renderer.

---

## 101. Instruction attribution

Because the IP is pre-incremented, a live frame normally attributes the most recently executing call/instruction through:

```text
frame.ip.saturating_sub(1)
```

before resolving the chunk's source span.

A frame that has executed nothing remains safe at `ip = 0` through saturating behavior.

---

## 102. Native frames

A native primitive does not automatically have a `CallFrame`.

Native diagnostic context is therefore maintained separately through fields such as active native selector/class context and may later be synthesized into logical traceback frames.

The debugging specification owns the rendering contract.

---

# Part XXV — Root scheduler and reactor integration

## 103. Ready queue

The VM owns a FIFO of scheduler-admitted fibers.

A queued fiber is a GC root until consumed.

The zero-floor root driver may resume queued work after root or detached fiber activity reaches a boundary.

---

## 104. Scheduler drivers

The VM may retain driver fibers that wait for scheduled work to complete.

Driver fibers are runtime roots and may become execution targets after child work parks or terminates.

---

## 105. Reactor progress

The root drive may:

- drain delivered external completions;
- wait for reactor progress when progress remains possible;
- resume newly queued work;
- diagnose quiescence/deadlock-like conditions where a root-blocking wait has no remaining source of progress.

Detailed reactor policy belongs to the concurrency specification.

The execution-model requirement is that these transitions occur only at coherent fiber/driver boundaries, not in the middle of an arbitrary bytecode instruction.

---

# Part XXVI — Resource ceilings

## 106. Guest call depth

Current limit:

```text
MAX_CALL_DEPTH = 10_000
```

It protects the runtime against unbounded guest activation growth.

The high ceiling is viable because physical guest frames are stored in a heap vector rather than recursively on the Rust stack.

---

## 107. Native re-entry depth

Current limit:

```text
MAX_NATIVE_REENTRY = 32
```

It protects the actual host thread stack.

The limit is deliberately much lower and not equivalent to guest recursion depth.

---

## 108. Distinct failure domains

A long ordinary Phalcom call chain should reach the guest call-depth ceiling rather than native re-entry depth.

A chain of primitives recursively driving `run_until` consumes native re-entry depth.

The implementation MUST NOT accidentally turn ordinary bytecode method dispatch into recursive Rust interpreter entry, because that would collapse the two resource models.

---

# Part XXVII — Execution invariants

## 109. Active-frame invariant

When bytecode is fetched, `VM::frames.last()` is the active physical bytecode activation.

---

## 110. Caller-order invariant

Physical caller relationship is encoded by order in `VM::frames`.

No parent-pointer repair is required on ordinary pop or vector truncate.

---

## 111. Stack-window invariant

Every live `CallFrame::stack_offset` refers to the stack owned by the same fiber as that frame.

---

## 112. Fiber-relative-index invariant

Frame offsets, open-upvalue slots, resume slots, and control stack indexes MUST NOT be interpreted against another fiber's stack.

---

## 113. Live-mirror invariant

The current fiber's stack-indexed execution state resides in VM live fields.

The current fiber's corresponding parked fields are empty.

---

## 114. Parked-state invariant

A non-current resumable fiber stores the execution state required for exact resumption on its `FiberObject`.

---

## 115. State-coherence invariant

`frames`, `stack`, `open_upvalues`, `control_stack`, and `checking` move together across a fiber switch.

---

## 116. Upvalue destruction invariant

Open upvalues are closed before stack storage they reference is truncated or discarded.

---

## 117. Activation identity invariant

A `FrameToken` is valid only when both frame index and generation match the live frame.

---

## 118. Call continuation invariant

The caller's IP has advanced to its continuation before a callee frame becomes active.

---

## 119. Control routing invariant

A callback result/raise/non-local return owned by an active `ControlActivation` is routed through the control reducer rather than bypassing it.

---

## 120. Native re-entry invariant

A fiber switch MUST NOT invalidate the frame floor of an active nested `run_until`.

---

## 121. Safepoint invariant

GC occurs only at execution states where the complete root truth required by the collector is represented in VM/fiber-owned structures or explicit temporary roots.

---

## 122. Private absence invariant

The VM's private raw nil/uninitialized sentinel MUST be surfaced to the language's absence value before it becomes guest-visible.

---

## 123. Resource separation invariant

Guest call depth and native re-entry depth are distinct resources and use distinct limits.

---

## 124. Native authority invariant

Native method lexical/internal authority is represented explicitly and MUST NOT be inferred solely from the primitive's caller bytecode frame.

---

## 125. Current-fiber invariant

At most one fiber is `VM::current`.

The live mirror belongs to that fiber.

A switch changes both ownership identity and the state loaded into the mirror before guest execution proceeds.

---

# Part XXVIII — Implementation-performance notes

## 126. Copy frames

`CallFrame` is deliberately copyable.

The dispatch loop may copy the active frame metadata into Rust locals without borrowing the frame vector across mutable VM operations.

This avoids a runtime architecture based on `Rc<RefCell<Frame>>`.

---

## 127. Copy values

`Value` is copyable.

The operand stack therefore stores compact value representations rather than owning polymorphic Rust objects directly.

Heap-backed values use object handles.

The runtime value model is specified separately.

---

## 128. Flat frame vector

A flat vector makes:

```text
push     O(1) amortized
pop      O(1)
unwind   truncate
caller   previous index
```

and avoids frame-object allocation for every guest call.

Frame identity requiring persistence across stack changes is handled through `FrameToken`, not by pointer-stable frame objects.

---

## 129. Callable hoisting

The current dispatch loop caches the immutable `Callable` associated with the active closure and refreshes it when closure identity changes.

The optimization MUST be invalidated/guarded correctly across:

- calls;
- returns;
- fiber switches;
- any other active-frame change.

Only closure-stable data may use the closure-id guard.

---

## 130. Instrumentation gating

Per-opcode tracing/histogram instrumentation is compile-time feature gated where necessary because even disabled runtime subscribers can impose dispatch-loop cost if callsites remain compiled into the hot path.

This is an implementation performance decision, not an execution semantic.

---

## 131. Fiber buffer pooling

The implementation may optionally recycle fiber stack/frame buffers behind a feature gate.

Pooling MUST NOT change:

- fiber identity;
- frame generation;
- stack indexes during a live activation;
- GC reachability;
- terminal lifecycle semantics.

---

# Part XXIX — Required subsystem boundaries

## 132. Bytecodes

[`bytecode.md`](bytecode.md) owns:

- instruction catalog;
- instruction operands;
- per-opcode stack effects;
- branch semantics;
- instruction-specific runtime failures.

This document owns the loop and activation state in which those instructions execute.

---

## 133. Values and objects

The runtime value/object specification owns:

- `Value` representation;
- object handles;
- identity;
- object layout;
- heap object categories.

This document owns where values and object handles participate in execution state.

---

## 134. Calls, frames, and closures

A dedicated calls/closures specification should own:

- compiler slot assignment;
- capture descriptors;
- open versus closed upvalue representation in full;
- block construction;
- callable parameter binding;
- non-local-return language/runtime details.

This document owns the common activation, stack lifetime, frame identity, and unwind substrate.

---

## 135. Dispatch

The dispatch specification should own:

- exact selector identity;
- family selection;
- method lookup;
- rest fallback;
- visibility lookup semantics;
- `doesNotUnderstand`;
- inline-cache validity.

This document owns what happens after/while a callable target is activated.

---

## 136. Exceptions and unwinding

The exception specification should own:

- `on` matching;
- `ensure` semantics;
- error object construction;
- propagation rules;
- traceback attachment.

This document owns the generic `Transfer`, `ControlActivation`, frame/stack unwind, and callback-floor mechanism.

---

## 137. Concurrency

The concurrency specification should own:

- manual coroutine language semantics;
- scheduler admission;
- Future park/wake;
- completion observers;
- reactor policy;
- fairness/quiescence;
- scheduler failure observation.

This document owns where fiber execution state physically lives and how switching changes execution ownership.

---

## 138. Memory management

The memory-management specification should own:

- heap allocator;
- GC algorithm;
- mark/sweep;
- object tracing;
- collection thresholds;
- weak/future GC features.

This document owns the dispatch safepoint and execution-state root topology.

---

# Part XXX — Implementation anchors

## 139. Primary source map

The implementation described by this document is principally realized in:

| Concern | Primary source |
|---|---|
| `VM` ownership/state | `phalcom-core/src/vm/mod.rs` |
| frame representation | `phalcom-core/src/frame.rs` |
| interpreter/root driver | `phalcom-core/src/vm/dispatch.rs` |
| VM public/internal helpers | `phalcom-core/src/vm/api.rs` |
| method/call activation | `phalcom-core/src/vm/send.rs` |
| invocation ABI | `phalcom-core/src/method/object.rs` |
| control continuations | `phalcom-core/src/vm/control.rs` |
| fiber object/lifecycle | `phalcom-core/src/heap/fiber.rs` |
| fiber park/resume/yield | `phalcom-core/src/primitive/fiber.rs` |
| block/native re-entry | `phalcom-core/src/primitive/block.rs` |
| module source execution | `phalcom-core/src/interpret.rs` |
| GC roots/safepoint | `phalcom-core/src/vm/gc.rs` |
| stack walking | `phalcom-core/src/vm/walk.rs` |

Changes to these files that affect the invariants specified here require review of this document.

---

# Appendix A — High-level execution topology

```text
                              ┌─────────────────────┐
                              │       VM            │
                              │                     │
                              │ Heap / Universe     │
                              │ registries          │
                              │ scheduler/reactor   │
                              │ global generations  │
                              └──────────┬──────────┘
                                         │
                                    current Fiber
                                         │
                              ┌──────────▼──────────┐
                              │  live execution     │
                              │                     │
                              │ frames              │
                              │ stack               │
                              │ open_upvalues       │
                              │ control_stack       │
                              │ checking            │
                              └──────────┬──────────┘
                                         │
                                  run_until_inner
                                         │
                    ┌────────────────────┼─────────────────────┐
                    │                    │                     │
                 Return               Transfer            Fiber switch
                    │                    │                     │
                    ▼                    ▼                     ▼
             frame destruction   control reducer      park/load mirror
                    │                    │                     │
                    └────────────────────┼─────────────────────┘
                                         │
                                  coherent next state
```

---

# Appendix B — Callable activation topology

```text
selected callable
      │
      ├── bytecode closure
      │      │
      │      └── push CallFrame
      │               │
      │               └── EnteredFrame
      │
      └── native primitive
             │
             └── Rust implementation
                    │
                    ├── Returned(value)
                    ├── EnteredFrame
                    ├── EnteredControl
                    └── SwitchedFiber
```

The legacy primitive ABI communicates less of this structure explicitly and remains subject to compatibility reconciliation.

---

# Appendix C — Frame-floor execution

```text
native Rust activation
        │
        ├── base_frames = VM.frames.len()
        ├── establish guest activation
        ├── check native re-entry ceiling
        ├── native_reentry_depth += 1
        │
        └── run_until(base_frames)
                 │
                 │ while frames.len() > base_frames
                 │     dispatch guest code
                 │
                 └── returns when added activation region drains
        │
        └── native_reentry_depth -= 1
```

A fiber switch is forbidden while this nested drive is active because replacing `VM::frames` would invalidate the meaning of `base_frames`.

---

# Appendix D — Fiber switch

```text
Before:

VM.current = Fiber A

VM live:
    A.frames
    A.stack
    A.open_upvalues
    A.control_stack
    A.checking

Fiber B parked:
    B.frames
    B.stack
    B.open_upvalues
    B.control_stack
    B.checking


Park A:

VM live
    ───────────────► Fiber A parked


Load B:

Fiber B parked
    ───────────────► VM live

VM.current = Fiber B
Fiber B.status = Running
```

Stack offsets do not change because the frame vector and stack buffer move as one fiber-relative state.

---

# Appendix E — Ordinary return

```text
active callee frame
        │
        ├── obtain return value
        ├── pop frame
        ├── close open upvalues
        ├── truncate callee stack window
        │
        ├── callback floor reached?
        │      └── route Returned through ControlStack
        │
        ├── run_until frame floor reached?
        │      └── return to native/root driver
        │
        └── otherwise
               └── push result for guest caller
```

---

# Appendix F — Non-local return

```text
block executes ReturnNonLocal
        │
        ├── read home FrameToken
        ├── validate index + generation
        │
        ├── active ControlStack?
        │      │
        │      └── route NonLocalReturn transfer
        │             │
        │             └── ensure/other controls may run
        │
        └── propagated final transfer
               │
               ├── close upvalues at home boundary
               ├── truncate stack to home boundary
               ├── place result
               └── truncate home frame + intervening frames
```

---

# Appendix G — Architectural checklist

Any change to the execution engine should be reviewed against the following questions.

1. Does it introduce execution state that must move with a fiber?
2. Does it introduce a stack index that becomes invalid after a fiber switch?
3. Does it hold a heap handle only in Rust across re-entrant guest execution?
4. Does it add a continuation that currently survives only through the Rust stack?
5. Should that continuation instead be a `ControlActivation`?
6. Does it destroy stack storage without first closing open upvalues?
7. Does it create a frame without assigning a fresh generation?
8. Does it change the meaning of a frame floor?
9. Can it switch fibers while a nested frame floor is active?
10. Does it add an execution-state field that must be classified by GC root enumeration?
11. Does it infer native lexical authority from the wrong bytecode frame?
12. Does an optimization cache frame-specific state using only closure identity?
13. Does it accidentally make ordinary guest calls recursively consume the Rust stack?
14. Does it mutate execution state before validating a switch/activation that may fail?
15. Does it preserve source/traceback attribution under pre-incremented IP semantics?

A change that answers one of these questions incorrectly is an execution-model violation even when ordinary unit tests appear to pass.
