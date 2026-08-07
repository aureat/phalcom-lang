# Spec F.2 — Outgoing Pack Assembly and Dynamic Send

Status: implementation specification. Requires F.1, A.3, B.1/B.2, C.1, E.1 and E.3.

## 1. Mission

Implement runtime assembly of dynamic argument packs for:

```text
method sends
super sends
subscript sends
Tuple construction
```

while preserving:

- lexical evaluation order;
- early dynamic label validation;
- early duplicate detection;
- stable labeled order;
- normal VM-frame execution;
- the existing static `Invoke` fast path.

Expected public primitive-floor delta: **0**.

## 2. Do not allocate packs for ordinary calls

Compile these exactly as today:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
obj.foo(a, timeout: t)
obj[a, b]
```

when all labels are static and no expansion occurs.

Compiler path:

```text
derive selector at compile time
compile receiver
compile args left-to-right
Invoke / SuperSend
```

This preserves current inline caches, fused superinstructions, and sacred-call recognition.

Use the dynamic pack path only if at least one pack item is:

```text
Expand(* / ** / ***)
Computed label
```

A labeled trailing closure with a static label does not by itself require a dynamic pack.

## 3. Why a native spread helper is prohibited

The baseline has `VM::send_dynamic(receiver, selector, args)` for reflective/native callers. It re-enters `run_until` from Rust and deliberately activates `native_reentry_depth`.

Do **not** implement:

```text
native expand primitive
    -> exhaust Iterable from Rust
    -> call VM::send_dynamic
```

That would recreate the yield-across-native-frame restriction for:

```phalcom
target(*lazyPipeline)
```

when pipeline callbacks use `Fiber.yield`.

F must remain in the ordinary dispatch loop.

## 4. Internal transient builder

Add a private heap object, conceptually:

```rust
pub struct ArgumentPackBuilderObject {
    positionals: Vec<Value>,
    labels: Vec<Symbol>,
    labeled_values: Vec<Value>,
    pending_label: Option<usize>,
}
```

An equivalent contiguous layout is acceptable.

Requirements:

- insertion order preserved;
- duplicate lookup by interned Symbol identity;
- builder never escapes to user code;
- not hashable/user-inspectable;
- safe across nested calls and fiber switching because the builder is referenced by the owning frame's hidden local;
- finishing a call/Tuple does not expose it.

A small linear duplicate scan is acceptable for the first implementation because call label counts are normally tiny. A private `HashSet<Symbol>` acceleration is also acceptable.

Do not invoke user `hash` or `==` for label duplicate checks.

## 5. Builder placement

Use compiler-owned hidden locals, following C.1's hidden-temporary technique.

For a dynamic method call:

```text
evaluate receiver
store receiver in hidden local

create builder
store builder in hidden local

compile each pack item in lexical order

load receiver
load builder
InvokePack
```

The hidden names must be unnameable from source.

This is preferable to keeping the builder at a fragile fixed operand-stack depth while arbitrary argument expressions create nested calls/locals.

## 6. Builder bytecode family

Exact opcode names are implementation details. Recommended semantic operations:

```text
NewArgumentPack

PackPushPositional

PackReserveStaticLabel(labelConstant)
PackReserveComputedLabel
PackFillReservedLabel

PackExpandLabels
PackExpandComplete

PackTryExpandTuplePositionals

InvokePack(baseNameConstant, kind)
SuperSendPack(baseNameConstant, definingClassConstant)
FinishTuplePack
```

Do not expose these as public native methods.

The disassembler and bytecode name table must be updated for every added opcode.

## 7. Positional append

`PackPushPositional`:

input:

```text
builder, value
```

effect:

```text
builder.positionals.push(value)
```

No callback, coercion, or cloning beyond `Value` copy.

## 8. Explicit static labeled contribution

For:

```phalcom
target(label: valueExpr)
```

dynamic pack lowering is:

```text
load builder
PackReserveStaticLabel(#label)

compile valueExpr

load builder
[value already available, exact stack arrangement is implementation choice]
PackFillReservedLabel
```

Reservation happens before `valueExpr`.

Therefore a duplicate against an earlier `***`/`**` contribution fails before `valueExpr` runs.

## 9. Computed label contribution

For:

```phalcom
target([labelExpr]: valueExpr)
```

compile:

```text
compile labelExpr
require Value::Symbol
reserve/check duplicate
compile valueExpr
fill reservation
```

If the computed value is a String, Selector object, Number, etc., fail before evaluating `valueExpr`.

No String→Symbol coercion.

## 10. Pending label representation

A convenient implementation reserves a label by appending:

```text
label
Value::Nil
```

privately, then `PackFillReservedLabel` replaces the last placeholder.

This is safe only because:

- the builder is unobservable;
- one source contribution is compiled to completion before the next item;
- a nested call allocates its own builder;
- an error abandons the outer builder.

Assert in `InvokePack` / `FinishTuplePack` that no pending placeholder remains.

Never surface private `Value::Nil`.

## 11. `**` expansion

`PackExpandLabels` accepts only:

```text
Tuple / Unit
Record
Map
```

Behavior:

### Tuple

Append its labeled lane only.

Ignore positionals.

### Unit

Append nothing.

### Record

Append fields in Record encounter order.

### Map

Append associations in Map insertion/encounter order.

For every association:

1. require key to be Symbol;
2. reject duplicate against earlier builder labels;
3. append `(Symbol, value)`.

A Map with a String key is a valid Map but is invalid as `**` call/Tuple expansion.

Failure occurs at the first offending association.

Do not sort Map/Record labels.

Do not convert Map keys from String.

Do not accept arbitrary Iterable-of-pairs as `**` in F.

## 12. `***` expansion

`PackExpandComplete` accepts:

```text
Tuple
Unit
```

only.

For Tuple:

1. append all positional-lane values to builder positionals;
2. append all labeled-lane entries in Tuple label order, checking duplicates.

For Unit: no contribution.

Reject Record/Map even though they have only a labeled lane. Older `***record` drafts are obsolete.

Multiple complete expansions are allowed.

## 13. `***` and later source positionals

Example:

```phalcom
target(***pack, later, ***second)
```

If `pack` contains labels, append those labels immediately to the builder's labeled lane, but continue accepting later **source** positional contributions because parser phase remains positional.

Final storage still has canonical lanes:

```text
all positionals
then all labels
```

Evaluation order and lane order are different concepts.

## 14. `*` Tuple versus `*` Iterable

Tuple normal iteration is total product order and would include labeled values. That is **not** correct for `*tuple`.

Therefore `*` lowering needs a Tuple lane-special case.

Recommended opcode:

```text
PackTryExpandTuplePositionals
```

Inputs:

```text
builder, operand
```

Behavior:

- Tuple: append only positional lane; return `true`;
- Unit: append none; return `true`;
- anything else: do not mutate; return `false`.

The compiler branches on the result.

If handled, expansion ends.

If not handled, compile the generic Iterable exhaustion loop below.

This avoids adding a user-overridable hidden expansion protocol solely to distinguish Tuple lane projection.

## 15. Generic Iterable `*` lowering

For a non-Tuple operand, lower using the same cursor protocol as `for`:

```text
source = operand
cursor = source.iterate(None)

while cursor != None:
    value = source.iteratorValue(cursor)
    builder.pushPositional(value)
    cursor = source.iterate(cursor)
```

Generate ordinary bytecode sends for:

```text
iterate(_)
iteratorValue(_)
```

and use the existing `JumpIfNone` loop mechanics.

Do not call `each`.

Do not call an expansion primitive that loops in Rust.

This automatically supports:

```text
List
Bytes
Range
Progression once implemented
Set
Iterator pipelines
user Iterable subclasses
```

through their normal iteration semantics.

If a non-Iterable reaches the generic path, ordinary method lookup failure becomes the runtime expansion error. It is acceptable to sharpen this to the existing surface TypeError/ArgumentError convention, but do not panic.

## 16. E.3 boundedness hook

Before emitting generic iterable exhaustion for:

```text
*operand
```

call E.3's shared exhaustibility checker.

If:

```text
Unbounded
```

reject at compile time.

If:

```text
Bounded
Unknown
```

compile normally.

Required:

```phalcom
target(*(0..))                       // compile error
target(*((0..).iter.take(3)))        // legal
target(*unknownIterator)             // legal
```

The Tuple direct lane case is always finite and need not be rejected even if general source inference is Unknown.

Do not duplicate E.3's Range/pipeline analyzer.

## 17. Evaluation timing for `*`

The operand expression is evaluated exactly once before its iteration starts.

The source is exhausted completely before the next pack item expression is evaluated.

Therefore:

```phalcom
target(*sourceA(), later())
```

runs all effects required to exhaust `sourceA()` before calling `later()`.

This follows from selector arity depending on the completed positional contribution.

## 18. Fiber/yield invariant

Because generic `*` is a compiler-generated cursor loop of ordinary sends:

```phalcom
target(*pipeline)
```

must permit the same fiber behavior as:

```phalcom
for (x in pipeline) { ... }
```

subject to the same underlying iterator methods.

Add a regression where a lazy mapping/filter stage yields during expansion. It must not raise the old `CannotYieldAcrossNativeFrame` merely because expansion is occurring.

## 19. Dynamic concrete selector derivation

At `InvokePack`, derive:

```text
P = builder.positionals.len()
L = builder.labels in order
```

then use F.1's shared concrete selector encoder.

Method example:

```text
base = "send"
P = 1
L = [#timeout, #onError]
```

derives:

```text
send(_,timeout,onError)
```

using escaped label components where necessary.

Subscript derives bracket form:

```text
[_,label]
```

with the same lane/escaping rules.

Do not stringify labels through user `toString`; use interned Symbol text.

## 20. Dynamic send stack rewrite

Before dispatch, `InvokePack` converts:

```text
receiver, builder
```

into:

```text
receiver,
positional values...,
labeled values...
```

and records/derives the concrete selector.

Then enter the ordinary VM lookup/call machinery.

Do not call the public/re-entrant `VM::send_dynamic` helper.

Factor a non-reentrant dispatch helper from `invoke_at` if useful:

```rust
dispatch_selector_on_current_stack(
    receiver_idx,
    selector,
    actual_arity,
    source_range,
    cache_policy,
)
```

Both static and dynamic opcodes may share lookup/dNU/rest-fallback logic.

## 21. Dynamic-site cache policy

The existing `InlineCache` is keyed only by receiver class/version because the static selector is fixed per site.

Do not reuse it unsafely for `InvokePack`, whose selector may change between executions.

First implementation MAY run dynamic pack sites uncached.

Preferred follow-up-compatible design:

```text
DynamicInlineCache {
    class,
    selector,
    method,
    worldVersion
}
```

kept separate from the static hot-path cache.

Do not add an extra selector compare to every ordinary static `Invoke` merely to reuse one struct unless measurement justifies it.

## 22. Concrete send arity limit

The current VM encodes fixed call arity in `u8`.

F does not widen that architecture.

Before a dynamic send dispatches, require:

```text
positionals.len + labels.len <= 255
```

If exceeded, raise the existing language-level argument/arity error before lookup.

Tuple construction is not subject to the send-arity limit.

Static call parsing/compilation should also stop silently casting `usize` to `u8`; add a checked diagnostic for >255 source arguments if not already present.

## 23. `doesNotUnderstand`

On a dynamic miss, dNU must see:

- the actual derived concrete selector;
- args flattened in canonical pack lane order;
- decoded raw label texts, not F.1 escape transport.

Preserve the existing single-forward dNU rule.

## 24. Super sends with dynamic packs

Support:

```phalcom
super.foo(*args)
super.foo(**labels)
```

through a sibling opcode/path:

```text
SuperSendPack
```

It derives the concrete selector from the pack, then begins lookup above the defining class exactly like current `SuperSend`.

Do not lower a dynamic super send to ordinary receiver lookup.

Constructor `super` rewriting to hidden `init <name>` selector must happen before dynamic selector assembly exactly as the static path does.

## 25. Subscript reads

Dynamic subscript:

```phalcom
obj[*indices, label: x]
```

uses the same builder and derives a `SignatureKind::Subscript` selector.

No `at` desugaring.

## 26. Subscript setters and implicit `put`

C.1 requires evaluation:

```text
receiver
subscript pack items in lexical order
RHS
setter send
```

and result:

```text
original RHS
```

The implicit `put` label must also detect duplicates **before RHS evaluation**.

For a dynamic setter:

1. evaluate receiver;
2. assemble all source subscript items;
3. reserve compiler-owned static label `#put` in builder;
   - failure here catches `**map` / computed-label `#put` duplicates before RHS;
4. evaluate RHS;
5. fill reserved `put` value;
6. preserve RHS through C.1's hidden-local mechanism;
7. dynamic subscript send;
8. discard setter return;
9. yield original RHS.

Do not append `put` only after RHS has executed.

## 27. Tuple finish

`FinishTuplePack` consumes the transient builder and calls A's product finalizer.

Result:

```text
0 entries -> Unit
otherwise -> Tuple
```

Labels remain in builder encounter order.

Do not stage through List.

Do not allocate a separate empty Tuple object.

## 28. Tuple expansion examples

```phalcom
const p = (1, 2)
const l = (x: 3)

const t = (0, *p, ***l, y: 4)
```

result:

```phalcom
(0, 1, 2, x: 3, y: 4)
```

`***l` contributes no positionals and label `x`.

This:

```phalcom
(*record,)
```

is invalid unless Record independently becomes Iterable later; Record is not a positional pack source.

This:

```phalcom
(**record,)
```

is labeled expansion and legal.

## 29. Error taxonomy

Dynamic pack failures must be catchable language failures using existing error conventions.

Required categories/messages:

```text
computed label is not Symbol
duplicate argument/product label
invalid `**` expansion operand
non-Symbol Map key during `**`
invalid `***` operand (expected Tuple)
send arity exceeds VM limit
non-Iterable `*` operand
```

Do not introduce Rust panics/assertions as user-visible control flow.

Internal assertions about impossible pending-builder state are acceptable only after parser/compiler invariants guarantee it.

## 30. Tests

### 30.1 Order

Counters/logs proving:

- receiver first;
- each operand once;
- `*` fully exhausted before following source item;
- `**` operand evaluated once;
- duplicate explicit/dynamic label prevents later value/argument evaluation;
- computed label guard precedes value.

### 30.2 Expansion sources

Positive:

- `*Tuple` ignores labeled lane;
- `*Unit` empty;
- `*List`;
- `*Bytes`;
- finite `*Range`;
- `*Iterator`;
- `**Tuple`;
- `**Unit`;
- `**Record`;
- `**Map` stable order;
- `***Tuple`;
- `***Unit`;
- multiple `***`.

Negative:

- `*Map`;
- `**List`;
- `***Record`;
- Map `**` with non-Symbol key;
- duplicate across `***` then explicit label;
- duplicate across two `**`.

### 30.3 Boundedness

- unbounded Range spread compile error;
- bounded `take` spread legal;
- Unknown source spread compiles.

### 30.4 Dispatch

- dynamic arity chooses correct exact overload;
- dynamic label order chooses correct selector;
- escaped `#*` label calls concrete `foo(#*)`, not rest `foo(*)`;
- dNU receives correct dynamic selector/args;
- dynamic super send starts above defining class.

### 30.5 Tuple

- dynamic expanded Tuple equality/lane access;
- empty dynamic expansion normalizes Unit;
- labels preserve encounter order.

### 30.6 Fiber

- iterator callback can yield during `*` expansion.

## 31. Completion checklist

F.2 is complete when:

- static calls retain static Invoke path;
- private pack builder exists;
- contribution assembly is incremental;
- computed labels validate before value;
- duplicate labels have correct timing;
- `**` preserves Map/Record/Tuple order;
- `***` is Tuple/Unit only;
- multiple `***` works;
- generic `*` is cursor-bytecode-driven;
- E.3 unbounded rejection is reused;
- dynamic selector uses shared encoder;
- dynamic send does not use re-entrant `send_dynamic`;
- dynamic super/subscript paths work;
- setter implicit `put` timing is correct;
- Tuple finish uses A finalizer;
- static hot-path cache is not degraded;
- no public primitive binding was added.
