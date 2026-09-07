# Task Set 3 — Unified Invocation, Rest Binding, and Function Runtime

**Project:** Phalcom
**Repository:** `aureat/phalcom-lang`
**Status:** Implementation-ready coding task-set
**Execution:** Coding task 2 of 3; requires Task Set 2 complete
**Primary areas:** primitive ABI, argument transport, Function gateway, Closure rest execution, BoundMethod/Method activation, Family routing, native rest, access authority, flat VM activation
**Must leave repository buildable and testable at completion**

---

## 1. Mission

Replace the old Block/native-call special cases with one shape-aware activation architecture.

This task-set must implement:

- `Function#call(***arguments)` as the common Function gateway;
- shape-aware zero-allocation argument transport;
- native rest Methods;
- a native result that may return immediately or enter another VM activation;
- flat language-level Method/Function forwarding without recursive `run_until`;
- Closure positional-rest binding;
- exact BoundMethod execution;
- receiver validation for `bind`/`invokeOn`;
- Family routing through Function rather than intentional dNU;
- explicit callee Method lexical authority for primitive execution;
- shape-preserving `callWith`, `Method#invokeOn`, and `Object#perform` execution paths.

This task-set must preserve the language rule:

```text
rest/spread syntax is only *, **, ***
```

Never introduce `args...`.

---

## 2. Core invocation abstractions

Implement and converge on four distinct concepts:

```text
ArgumentShape
ParameterShape
ParameterLayout
BindingPlan
```

### 2.1 ArgumentShape

Describes the shape of one actual call:

```text
ordered positional lane
ordered label Symbols
labeled values corresponding 1:1 with labels
```

It contains no parameter semantics.

### 2.2 ParameterShape

Describes accepted call shapes:

```text
fixed positional count
fixed ordered labels
rest mode
```

Method can support all F.3 modes.

Closure supports only:

```text
fixed positionals
optional positional rest
no labels
```

### 2.3 ParameterLayout

Describes bytecode-frame placement:

```text
fixed parameter local slots
rest-capture local slot(s)
```

Do not use it for dispatch acceptance.

### 2.4 BindingPlan

Pure, allocation-free result of:

```text
ArgumentShape × ParameterShape
```

The plan maps actual lanes to fixed slots/rest residuals.

Matching must not dispatch user code.

Matching must not allocate.

---

## 3. ArgumentView: shape-aware, allocation-free transport

Ratified primitive ABI requires an `ArgumentView`-like object.

### 3.1 Rust borrowing constraint

Do **not** implement `ArgumentView<'a>` as slices borrowed directly from `VM.stack` while also passing `&mut VM` to primitives. That creates an avoidable self-borrow/aliasing problem in Rust.

Prefer a compact descriptor that owns no argument values and stores indexes/handles, conceptually:

```rust
#[derive(Clone, Copy)]
struct ArgumentView {
    source: ArgumentSource,
    shape: ArgumentShapeRef,
}
```

Possible `ArgumentSource` forms:

```text
StackWindow { base, ... }
DynamicPack { builder/object handle, ... }
```

Accessors take the VM explicitly:

```rust
args.positional(vm, i)
args.labeled_value(vm, i)
args.label(vm, i)
args.positional_count(...)
args.labeled_count(...)
```

The exact layout may differ, but the invariants are:

- no per-call heap allocation on the static call fast path;
- no long-lived Rust borrow into `VM` during `&mut VM` primitive execution;
- dynamic-pack calls may reuse the already-rooted F.2 pack/builder storage rather than copying into a new container;
- values remain GC-rooted for the full activation.

### 3.2 Canonical stack ordering

Choose and document one canonical VM window, preferably:

```text
receiver
fixed/actual positional values
labeled values in label order
```

Shape metadata carries label identity; do not interleave label Symbols into the value stack unless there is a measured reason.

---

## 4. Primitive ABI migration

Current conceptual ABI:

```rust
fn(&mut VM, &Value, &[Value]) -> PhResult<Value>
```

must become shape-aware.

Recommended conceptual ABI:

```rust
fn(
    vm: &mut VM,
    receiver: Value,
    args: ArgumentView,
) -> PhResult<CallOutcome>
```

`receiver` is `Value` by copy.

### 4.1 CallOutcome

Ratified result:

```rust
enum CallOutcome {
    Returned(Value),
    EnteredFrame,
}
```

Equivalent names are acceptable.

Ordinary primitive:

```text
Number#+
    → Returned(result)
```

Forwarding primitive:

```text
Function#call on Closure
    → push Closure frame
    → EnteredFrame
```

This replaces hard-coded function-pointer identity tests.

### 4.2 Migration discipline

Provide helpers/macros so exact fixed-arity primitives can be migrated mechanically without rewriting every implementation body by hand.

For example, a helper may validate exact shape and expose positional arguments in a small local buffer/view.

Do not reintroduce a per-send heap `Vec` allocation for common fixed-arity primitives.

---

## 5. Amend F.3 — native rest Methods

Supersede the old restriction that primitive/native Methods cannot carry rest layouts.

Primitive Methods may use:

```text
*rest
**rest
***rest
```

and participate in exactly the same lookup rule:

```text
Pass 1: exact selector across inheritance
Pass 2: rest-family resolution
```

Rest acceptance uses structured shape metadata, not wildcard selector-string parsing.

Native rest implementations consume `ArgumentView`; they need not materialize user-visible rest products unless the primitive explicitly requests a captured product.

---

## 6. Function gateway

Install one canonical Function base-family gateway:

```phalcom
call(***arguments)
```

The base-selector family `call` is final throughout the sealed Function hierarchy.

Do not generate finite call Methods.

Do not synthesize per-signature Methods on Closure/BoundMethod/Family.

### 6.1 Application syntax

Preserve:

```phalcom
f(a, b)
```

as ordinary message syntax equivalent to:

```phalcom
f.call(a, b)
```

The compiler may optimize the send but may not create a separate language-level call semantic that bypasses message resolution.

### 6.2 Gateway dispatch

The common gateway routes by concrete Function representation:

```text
Closure
BoundMethod
Family
```

No user-extensible Function subclass exists.

---

## 7. Closure activation and positional rest

Given:

```phalcom
const f = |head, *tail| {
    ...
}
```

incoming shape validation:

```text
labels must be empty
positional_count >= fixed positional count
```

### 7.1 Rest capture

Residual positional values are captured canonically:

```text
0 residual values → ()
1+ residual values → Tuple
```

Use the existing canonical Tuple/Unit finalization path (`finish_tuple` or its current successor).

Do not capture Closure rest into List.

Do not materialize a rest value if no rest parameter exists.

Do not materialize a complete pack merely because `Function#call` is a `***` gateway.

### 7.2 Outgoing spread

These are all legal syntax:

```phalcom
f(*values)
f(**labels)
f(***arguments)
```

Acceptance is based on the resulting shape.

A non-empty labeled lane must be rejected for the current positional-only Closure model.

`***arguments` may succeed if its resulting labeled lane is empty.

### 7.3 Frame entry

Use `BindingPlan` to populate the Closure frame directly.

No native recursive `run_until` should be involved in an ordinary Closure application.

---

## 8. BoundMethod

Keep payload:

```text
exact Method
stored receiver
```

### 8.1 Binding validation

`Method#bind(receiver)` validates before allocation.

Compatibility:

```text
receiver.class == holder
or receiver.class is a subclass of holder
```

Class-side Method uses analogous metaclass ancestry.

Immediate receiver values use their actual runtime classes.

Holderless public Method:

```text
bind / invokeOn → reject
```

unless an already-existing internal-only path is explicitly exempted and never exposed as permissive public semantics.

### 8.2 Invocation

BoundMethod call:

1. does not redispatch the underlying Method selector;
2. validates actual argument shape against the underlying Method's `ParameterShape`;
3. uses stored receiver as `self`;
4. executes the exact Method;
5. preserves dynamic dispatch for sends *inside* that Method;
6. preserves original Method lexical `super` anchor/access authority.

Underlying bytecode Method:

```text
push exact Method frame
→ EnteredFrame
```

Underlying primitive Method:

```text
execute primitive with explicit callee Method context
→ Returned or EnteredFrame
```

---

## 9. `Method#invokeOn`

Replace positional-List transport with complete rest:

```phalcom
method.invokeOn(receiver, ***arguments)
```

Semantics:

- validate receiver compatibility first;
- authorize access before mutating the value stack;
- validate argument shape;
- execute the exact Method;
- do not selector-redispatch;
- do not use a separate argument binder;
- do not force Tuple allocation for the complete rest gateway.

The language-level implementation path must use flat activation.

A separately named synchronous Rust helper may remain for host code that truly requires immediate completion.

---

## 10. `callWith`

Canonical semantic definition:

```phalcom
function.callWith(pack)
```

is exactly:

```phalcom
function(***pack)
```

Implement by feeding the complete pack into the same Function gateway.

Do not preserve List-only semantics.

Do not implement a second binder.

---

## 11. Family migration

Current Family callability through intentional `doesNotUnderstand` misses is retired.

### 11.1 Open Family

For:

```phalcom
const f = obj::name
f(a, label: b)
```

the Function gateway already receives the actual `ArgumentShape`.

Build the target selector directly from:

```text
stored base family name
+
actual call slots/labels
```

then perform an ordinary message dispatch to the stored receiver.

No fake `call(...)` miss.

No reified Message allocation solely to recover call labels.

No selector decode round-trip of the missed Function call.

### 11.2 Pinned Family

Preserve the currently ratified pinned-selector semantics unless separately changed by reflection work.

Validate required argument count/shape according to pinned-reference rules, then dispatch the pinned selector to the stored receiver.

### 11.3 dNU

`Family#doesNotUnderstand` must cease being the call router.

Family still inherits ordinary Object dNU behavior for actual unrecognized messages.

---

## 12. Flat activation APIs

Split “prepare/enter activation” from “run interpreter until completion”.

Recommended conceptual APIs:

```text
activate_exact_method(...)
dispatch_and_activate(...)
activate_function(...)
```

returning `CallOutcome`.

Language-level paths must not recursively invoke `run_until`:

```text
normal send
Function#call
Closure application
BoundMethod application
Family application
Method#invokeOn
callWith
Object#perform
```

### 12.1 Synchronous helper

A clearly named helper such as:

```text
send_dynamic_sync
invoke_method_sync
```

may wrap flat activation in `run_until` for Rust/native consumers that require a synchronous value.

Its use must be audited and limited.

Do not use synchronous re-entry as the implementation of ordinary language forwarding.

### 12.2 Fiber invariant

After Method selection:

```text
ordinary send of exact Method
```

and:

```text
BoundMethod call of that exact Method
```

must have equivalent:

- frame entry;
- yield behavior;
- return behavior;
- exception behavior;
- rest binding;
- upvalue closing.

---

## 13. Native Method lexical authority

Access checking and callee authority are separate phases.

### 13.1 Caller authorization

Before entering a Method:

```text
current caller lexical authority
    → authorize target Method visibility
```

### 13.2 Callee execution authority

While Method implementation executes:

```text
callee defining/access owner
    → active lexical authority
```

For bytecode Methods, associate the frame with the executing Method or equivalent explicit authority metadata.

For primitive Methods, maintain a stackable native Method execution context, conceptually:

```rust
NativeMethodContext {
    method,
    access_owner,
    defining_module/internal authority,
}
```

Nested primitive calls must restore contexts correctly.

Do not use the caller's lexical class as the primitive callee's authority.

Do not overload diagnostic-only `native_selector`/`native_class` state as the sole semantic authority model.

---

## 14. Rest resolution and caches

The common Function gateway makes rest-family lookup hot.

Ensure exact→rest resolution participates in the normal caching strategy.

Cache conceptually by:

```text
receiver class
concrete ArgumentShape / selector shape
world/method-table version
```

Cache result categories may include:

```text
Exact(Method)
Rest(Method + reusable binding metadata)
Miss
```

Do not repeatedly exact-miss + scan rest families for every Function call if the shape is stable.

---

## 15. `Object#perform`

Keep `Object#perform` as ratified.

Migrate its execution path onto the common shape-aware dispatch/activation machinery.

Do not move it to Behavior.

Preserve the distinction:

```text
perform
    dynamic send to this concrete receiver

methodFor
    Method reflection (moved later in Task Set 4)
```

If current public `perform` has a positional-List helper, migrate it to complete argument-pack shape rather than retaining a second positional-only path.

Exact public surface spelling beyond already-ratified behavior must not invent new reflection policy.

---

## 16. Primitive combinators that genuinely need synchronous execution

Some existing native callable combinators (`on`, `ensure`, etc.) execute a nested callable and then need to continue native logic after that nested result/error.

Do not force these through an unsafe fake flat path.

For this task-set it is acceptable for such primitives to use the explicitly named synchronous helper from section 12.1.

Requirements:

- document each remaining synchronous re-entry call site;
- preserve native-reentry/Fiber safety checks;
- do not let ordinary Function application use this path;
- remove old non-local-return-specific stack repair from these paths in Task Set 4.

A future continuation-frame redesign may eliminate these synchronous helpers, but it is not required here.

---

## 17. Required tests

### Function gateway

- zero args;
- many args;
- labeled shape transport;
- `*`, `**`, `***`;
- no arbitrary call-arity ceiling;
- no finite synthetic call Method requirement.

### Closure

- fixed positional;
- positional rest;
- empty rest → Unit;
- non-empty rest → Tuple;
- outgoing spread;
- `***` positional-only success;
- labeled-lane rejection.

### BoundMethod

- compatible exact holder;
- subclass receiver;
- unrelated receiver rejected at bind;
- unrelated receiver rejected at invokeOn;
- exact parent Method on subclass receiver does not redispatch the entry Method;
- inner sends still dispatch dynamically;
- `super` anchor preserved.

### Family

- open references route without dNU;
- labels preserved;
- pinned behavior preserved;
- no Message allocation required for ordinary Family call routing.

### Native rest

Add at least one test primitive or kernel primitive exercising each supported rest mode or the shared matcher directly.

### Fiber parity

Compare ordinary Method call and equivalent BoundMethod call containing `Fiber.yield`.

### Access authority

- private/protected Method invoked normally;
- same Method invoked through BoundMethod;
- primitive Method making private/protected sends uses its own lexical authority;
- rejected access leaves stack unmodified.

---

## 18. Explicit removals by completion

By the end of Task Set 3, remove:

- Function-pointer comparison against legacy `block_call`;
- BoundMethod routing through old Block surface assumptions;
- Family dNU call router;
- List-based `Method#invokeOn`;
- List-based `callWith`;
- ordinary language-level recursive `run_until` for Function/BoundMethod/Family/invokeOn/perform forwarding;
- scalar arity checks as the Closure invocation source of truth.

Non-local-return machinery itself remains until Task Set 4.

---

## 19. Completion gate

Task Set 3 is complete when:

- common Function `call(***arguments)` is live;
- Function call family is final;
- PrimitiveFn-equivalent ABI is shape-aware;
- `CallOutcome` supports frame forwarding;
- native rest Methods work;
- Closure positional rest works;
- BoundMethod activation is flat and exact;
- receiver validation is enforced;
- Family uses Function routing rather than dNU;
- primitive callee authority is explicit;
- `callWith`, `invokeOn`, and `perform` preserve complete shape;
- normal Function calls do not recursively `run_until`;
- all new tests and existing dispatch/Fiber tests pass.
