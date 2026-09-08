# F.2 — Outgoing Pack Assembly and Dynamic Send
## Amended implementation specification and plan

Status: **implementation specification — amended and ratified**

This document supersedes the earlier F.2 implementation plan while preserving the language semantics of the attached F.2 specification. The amendments resolve the original open questions and correct implementation details discovered while reviewing the current Phalcom VM/compiler architecture.

Requires:

- F.1 AST and selector syntax (`PackItem`, `PackLabel`, `ExpansionMode`, escaping)
- A.3 product finalization (`finish_tuple`)
- B.1/B.2 product and collection representations
- C.1 hidden-local technique and subscript-assignment result preservation
- E.1 iteration cursor protocol
- E.3 boundedness/exhaustibility analysis

Expected public primitive-floor delta: **0**.

---

# 1. Mission

Implement runtime assembly of dynamic argument packs for:

```text
method sends
unqualified/implicit-self sends
callable `call(...)` sends
super sends
subscript reads
subscript writes
Tuple construction
```

while preserving all of the following:

- lexical source evaluation order;
- each operand evaluated exactly once;
- early computed-label type validation;
- early duplicate-label detection;
- stable label encounter order;
- canonical outgoing lane order;
- normal VM-frame execution;
- fiber/yield safety for generic `*`;
- ordinary method lookup, variadic fallback, visibility checks, and dNU behavior;
- the existing static `Invoke`/`SuperSend` fast paths;
- zero new public primitive bindings.

The dynamic path is an implementation mechanism, not a new reflective object visible to Phalcom programs.

---

# 2. Ratified decisions

## D1 — Builder representation: private heap object

**Decision:** use a private, boxed heap object referenced through compiler-owned hidden locals.

```rust
Object::PackBuilder(Box<ArgumentPackBuilderObject>)
```

Do **not** use:

- a Rust-side side table indexed by a hidden local;
- a raw pointer to a Rust stack object;
- a native helper frame that owns the builder.

Rationale:

- the builder stores arbitrary `Value`s;
- those values must remain GC-reachable across nested sends;
- generic `*` may run arbitrary bytecode and switch fibers;
- parked fibers already preserve their own value stacks;
- a hidden-local `ObjRef` naturally participates in the existing root model;
- a separate side table would create a second lifetime/rooting system.

The builder remains unobservable by construction: no source syntax, public primitive, method, hash, equality, or reflection API is added for it.

## D2 — Hidden locals retain builders; pack opcodes consume temporary loaded copies

**Decision:** the owning frame's hidden local is the durable root. Every `GetLocal(builder_slot)` used as an opcode operand is a temporary stack copy and is consumed by that opcode.

Do **not** leave one builder copy on the operand stack for the whole assembly.

This is the central stack-discipline correction to the previous plan.

## D3 — `InvokePack` rewrites the top operand window, not the hidden-local slot

At send time the compiler loads:

```text
GetLocal(receiver_slot)
GetLocal(builder_slot)
InvokePack(...)
```

The operand window immediately before the opcode is therefore:

```text
..., hidden receiver local, hidden builder local, receiver_copy, builder_copy
                                                      ^^^^^^^^^^^^^^^^^^^^^^
                                                      dynamic send window
```

`InvokePack` consumes/replaces the **temporary `receiver_copy, builder_copy` window**.

The frame's `stack_offset` is not changed and does not "account for the builder slot" in any special way. It remains the frame base used by `GetLocal`/`SetLocal`.

## D4 — Dedicated typed bytecodes

**Decision:** add dedicated bytecode variants.

Do not reuse `InvokeCompilerInternal` as a sub-opcode namespace.

`InvokeCompilerInternal` is an access-authority mechanism for compiler-generated sends. Dynamic pack assembly is a distinct VM operation with distinct stack effects, disassembly, instrumentation, and selector derivation.

Use typed Rust operands rather than unvalidated numeric tags where practical.

## D5 — Dynamic sites are uncached in F.2

**Decision:** do not route dynamic sites through the ordinary static inline cache and do not add a selector comparison to every static `Invoke`.

F.2 dynamic sites perform uncached lookup.

A future optimization may add a separate:

```rust
DynamicInlineCache {
    class,
    selector,
    method,
    world_version,
}
```

but that is explicitly out of scope for the first implementation.

## D6 — Do not refactor the static hot path merely to share code

`invoke_at` is already performance-sensitive.

F.2 may factor narrow cold/common helpers, but must not introduce a runtime `CachePolicy` branch into every ordinary `Invoke` merely to reuse one dispatch function.

Prefer:

- static `invoke_at` remains materially unchanged;
- dynamic pack dispatch gets an uncached sibling path;
- common miss/variadic/dNU helpers may be shared where this does not perturb the static fast path.

## D7 — Dynamic subscript setter stores `#put` in the builder but excludes it from bracket-label encoding

The implicit setter value participates in:

- duplicate detection;
- argument flattening;
- send arity;
- evaluation timing.

It does **not** become an ordinary bracket label.

For a final builder:

```text
positionals = P
labels      = [source labels..., #put]
values      = [source labeled values..., rhs]
```

`InvokePack(SubscriptSet)` must:

1. require the final builder label to be `#put`;
2. exclude that final `#put` from the bracket-slot label list;
3. derive subscript index arity as `total_args - 1`;
4. encode the selector as the existing canonical:

```text
[...source index slots...]=(put)
```

5. flatten the RHS as the final argument value.

It must never derive:

```text
[...,put]=(put)
```

## D8 — Unqualified calls are in F.2 scope

F.1 already allows `PackItem`s in unqualified call AST.

Therefore F.2 must support both compiler resolutions:

```text
foo(*args)
```

when `foo` resolves to implicit self:

```text
dynamic send with base name "foo"
```

and when `foo` resolves to a local/upvalue/global callable:

```text
dynamic send with base name "call"
```

Omitting this path would leave valid F.1 AST forms stuck on the F.1 "not yet supported" errors.

## D9 — `PackTryExpandTuplePositionals` returns Bool; generic lane reloads the operand local

There is no new `JumpIfTrue` requirement.

Contract:

```text
..., operand, builder -> ..., Bool
```

Behavior:

- Tuple: append only positional lane; return `true`;
- Unit: append nothing; return `true`;
- other: no mutation; return `false`.

The compiler has already stored the operand into a hidden source local. On `false`, it reloads that local for generic cursor iteration.

No operand is pushed back by the opcode.

## D10 — Computed-label validation lives in `PackReserveComputedLabel`

Do not emit `GuardSymbol` followed by a second validation in the pack opcode.

`PackReserveComputedLabel` atomically:

1. validates that the computed value is `Symbol`;
2. checks duplicates by Symbol identity;
3. reserves the label.

This guarantees the value expression has not run on either failure path.

## D11 — Runtime pack failures get structured runtime-error variants

Use dedicated structured `RuntimeError` variants for F.2-specific failures rather than routing every case through a generic string-only `Message`.

They remain catchable through the existing runtime error-to-surface-`Error` machinery.

## D12 — Existing static send-arity checks are already a prerequisite, not new F.2 work

The compiler already uses checked send arity on current static send paths.

F.2 adds the runtime post-expansion `<= 255` check because the final dynamic arity is unknowable until assembly finishes.

Do not duplicate or replace the existing compile-time static checks.

## D13 — Compiler-internal dynamic sends preserve compiler authority

A compiler-generated privileged send must not silently lose access semantics merely because it contains a dynamic pack.

`InvokePack` therefore carries a typed access mode:

```rust
enum PackAccess {
    Ordinary,
    CompilerInternal,
}
```

or an equivalent typed representation.

`CompilerInternal` grants authority only during lookup/authorization, matching the existing `InvokeCompilerInternal` lifetime: it must be removed before a callee frame executes.

Source code must never be able to manufacture this access mode.

## D14 — E.3 integration consumes E.3's landed API

F.2 depends on E.3 semantics:

```text
Unbounded -> compile error
Bounded   -> compile
Unknown   -> compile
```

Do not hard-code a speculative function signature into F.2 before E.3 lands.

Once E.3 is present, call its shared exhaustor/boundedness API rather than duplicating Range or pipeline analysis.

---

# 3. Static path remains unchanged

Compile ordinary statically-shaped calls exactly as today when:

```text
no Expand(* / ** / ***)
and
no computed label
```

Examples:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
obj.foo(a, timeout: t)
obj[a, b]
```

Static path:

```text
derive selector at compile time
compile receiver
compile arguments left-to-right
Invoke / SuperSend
```

A labeled trailing closure with a static label does not by itself require a dynamic pack.

The F.2 detection predicate is:

```text
needs_dynamic_pack(items) ==
    any item is Expand(* | ** | ***)
    OR
    any labeled item uses PackLabel::Computed
```

The static path must retain:

- current inline-cache layout;
- current fusion behavior;
- sacred-selector recognition;
- current static checked arity;
- current selector constants;
- current `InvokeCompilerInternal` behavior.

---

# 4. Why a native spread helper remains prohibited

Do not implement:

```text
native expansion helper
    -> loop over Iterable in Rust
    -> VM::send_dynamic(...)
```

The baseline reflective/native send helper re-enters the interpreter from Rust.

A generic `*` source may invoke user iteration methods, lazy callbacks, or fibers. Exhausting it from a native Rust loop would recreate the yield-across-native-frame restriction F.2 exists to avoid.

Required execution shape:

```text
compiler-generated ordinary bytecode
    -> ordinary iterate(_) send
    -> ordinary iteratorValue(_) send
    -> ordinary VM loop
```

No Rust exhaustion loop.

---

# 5. Internal builder design

## 5.1 Heap object

Add:

```rust
pub struct ArgumentPackBuilderObject {
    positionals: Vec<Value>,
    labels: Vec<Symbol>,
    labeled_values: Vec<Value>,
    pending_label: Option<usize>,
}
```

Use `Option<usize>`, not `Option<Symbol>`.

The pending index identifies the exact reserved labeled-value slot.

Recommended invariant:

```text
labels.len() == labeled_values.len()
pending_label is None
    OR
pending_label == Some(labels.len() - 1)
```

while an explicit/computed label reservation is waiting for its value.

## 5.2 Builder methods

Recommended API:

```rust
impl ArgumentPackBuilderObject {
    fn new() -> Self;

    fn push_positional(&mut self, value: Value);

    fn reserve_label(
        &mut self,
        label: Symbol,
    ) -> Result<(), DuplicateArgumentLabel>;

    fn fill_reserved(
        &mut self,
        value: Value,
    ) -> Result<(), InternalPackState>;

    fn append_labeled(
        &mut self,
        label: Symbol,
        value: Value,
    ) -> Result<(), DuplicateArgumentLabel>;

    fn has_pending(&self) -> bool;

    fn take_parts(
        &mut self,
    ) -> (Vec<Value>, Vec<Symbol>, Vec<Value>);
}
```

`append_labeled` is useful for `**`/`***`, where the label and value already exist simultaneously and no user expression runs between them.

## 5.3 Reservation

`reserve_label(sym)`:

1. asserts/rejects an existing pending reservation as an internal invariant;
2. scans existing `labels` for exact Symbol identity;
3. on duplicate, returns a structured duplicate failure;
4. appends `sym`;
5. appends private `Value::Nil` placeholder;
6. records the new index in `pending_label`.

Linear scan is preferred for F.2.

Typical call-site label counts are tiny, and adding a private hash set would add another allocation/state structure for no demonstrated benefit.

A future thresholded acceleration is allowed if profiling justifies it.

## 5.4 Fill

`fill_reserved(value)`:

1. obtains `pending_label`;
2. writes the value into exactly that index;
3. clears `pending_label`.

Do not infer the target merely from "last value is Nil".

`Value::Nil` is private and must never be surfaced.

## 5.5 Duplicate semantics

Duplicate checks use interned `Symbol` identity only.

They must never invoke:

```text
hash
==
toString
```

or any other user method.

## 5.6 Finishing

Before a send/Tuple finish:

```text
builder.has_pending() == false
```

must hold.

A pending reservation at this point indicates compiler/VM corruption, not a language-level user error.

An internal assertion/error is appropriate there.

## 5.7 GC integration

Add:

```rust
Object::PackBuilder(Box<ArgumentPackBuilderObject>)
```

Trace:

- every `positionals` value;
- every `labeled_values` value.

`labels: Vec<Symbol>` contains no heap handles.

Also update the normative memory-management object-edge documentation in the same change.

Because `Object` matching is intentionally exhaustive in several subsystems, compile failures after adding the new variant are expected and should be resolved deliberately. Any defensive `class`/debug/display handling for `PackBuilder` must preserve the invariant that the object is not intentionally exposed as a public Phalcom value.

---

# 6. Bytecode family

Add exactly **11 new opcode variants**.

The repository currently has 49 bytecode variants before F.2, so a direct implementation of this family makes:

```text
Bytecode::VARIANTS = 60
```

Update:

- `Bytecode` enum;
- `BYTECODE_NAMES`;
- `Bytecode::VARIANTS`;
- exhaustive `Bytecode::index()`;
- disassembler formatting where useful;
- opcode histogram/instrumentation assumptions if any other code relies on the variant count.

## 6.1 Typed send operands

Recommended internal enums:

```rust
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
```

Do not encode these as unchecked arbitrary `u8` values unless there is a real serialized bytecode-format requirement. `Chunk` stores typed `Bytecode` values, so impossible numeric states provide no benefit.

## 6.2 Opcodes

Recommended variants:

```rust
NewArgumentPack

PackPushPositional

PackReserveStaticLabel(u16)
PackReserveComputedLabel
PackFillReservedLabel

PackExpandLabels
PackExpandComplete

PackTryExpandTuplePositionals

InvokePack {
    base_name: u16,
    kind: PackSendKind,
    access: PackAccess,
}

SuperSendPack {
    base_name: u16,
    defining_class: u16,
}

FinishTuplePack
```

Exact Rust field syntax may vary.

No public native methods are added for any of these operations.

---

# 7. Exact stack contracts

Pack bytecode stack effects must be explicit and testable.

The hidden local is the durable owner. Temporary builder loads are consumed.

## 7.1 `NewArgumentPack`

```text
before: ...
after:  ..., builder
```

Allocates and pushes `Value::Obj(pack_builder_ref)`.

## 7.2 `PackPushPositional`

```text
before: ..., value, builder
after:  ...
```

Consumes both temporary operands.

Effect:

```rust
builder.positionals.push(value)
```

No callback, conversion, clone beyond `Value` copy, or user dispatch.

Compiler form:

```text
compile value
GetLocal(builder_slot)
PackPushPositional
```

## 7.3 `PackReserveStaticLabel(label_idx)`

```text
before: ..., builder
after:  ...
```

The constant must be a `Symbol`.

Effect:

```text
reserve_label(label)
```

A duplicate fails now, before the source value expression runs.

Compiler form:

```text
GetLocal(builder_slot)
PackReserveStaticLabel(label_idx)
compile value
GetLocal(builder_slot)
PackFillReservedLabel
```

## 7.4 `PackReserveComputedLabel`

```text
before: ..., computed_label_value, builder
after:  ...
```

Effect:

1. require `computed_label_value` to be `Value::Symbol`;
2. reserve/check duplicate.

Failure occurs before the labeled value expression is compiled/executed further.

Compiler form:

```text
compile label_expr
GetLocal(builder_slot)
PackReserveComputedLabel
compile value_expr
GetLocal(builder_slot)
PackFillReservedLabel
```

No preceding `GuardSymbol`.

## 7.5 `PackFillReservedLabel`

```text
before: ..., value, builder
after:  ...
```

Fills the exact pending index and clears it.

## 7.6 `PackExpandLabels`

```text
before: ..., operand, builder
after:  ...
```

Implements `**`.

Consumes both temporary operands.

## 7.7 `PackExpandComplete`

```text
before: ..., operand, builder
after:  ...
```

Implements `***`.

Consumes both temporary operands.

## 7.8 `PackTryExpandTuplePositionals`

```text
before: ..., operand, builder
after:  ..., Bool
```

- Tuple -> append positional lane, push `true`;
- Unit -> push `true`;
- otherwise -> no mutation, push `false`.

No operand is restored.

## 7.9 `InvokePack`

```text
before:
    ..., receiver, builder

internal rewrite:
    ..., receiver, positional_values..., labeled_values...

after dispatch eventually:
    ..., result
```

The `receiver,builder` values here are temporary copies loaded from hidden locals, not the hidden-local storage itself.

## 7.10 `SuperSendPack`

Same operand window and flattening as `InvokePack`, with superclass lookup semantics.

## 7.11 `FinishTuplePack`

```text
before: ..., builder
after:  ..., product_result
```

Consumes the temporary builder copy and finalizes through A.3.

The hidden builder local remains below until compiler scratch cleanup.

---

# 8. Hidden-local discipline and cleanup

Dynamic lowering needs scratch locals because arbitrary item expressions can:

- call methods;
- create nested dynamic packs;
- allocate locals;
- switch fibers;
- run loops.

Never keep a builder at a guessed fixed operand-stack depth.

## 8.1 Scratch-local creation

Use fresh compiler-owned unnameable symbols, e.g.:

```text
$pack_receiver
$pack_builder
$pack_source
$pack_cursor
$setindex_rhs
```

Actual names must go through the existing fresh scratch-symbol facility.

A typical hidden-local initialization pattern is:

```text
reserve local metadata
push placeholder
compile expression
SetLocal(slot)
Pop
```

The placeholder has no user-visible effect, so lexical expression evaluation order is unchanged.

## 8.2 Dynamic send scratch layout

Prefer adjacent scratch locals in this order:

```text
receiver_slot
builder_slot
```

Before final send:

```text
... receiver_local builder_local
GetLocal(receiver_slot)
GetLocal(builder_slot)
InvokePack
```

After the send returns:

```text
... receiver_local builder_local result
```

## 8.3 Collapse scratch locals while preserving result

Factor a compiler helper for this pattern.

For two adjacent scratch locals where the first slot should become the result:

```text
SetLocal(receiver_slot)   // copies result into earliest scratch local
Pop                       // remove temporary result copy
Pop                       // remove builder local
```

Then remove both compiler-local metadata entries without emitting another pop.

Physical stack becomes:

```text
... result
```

The old receiver scratch slot has become the expression result slot and is no longer considered a local.

This avoids adding a dedicated "drop locals under TOS" opcode.

## 8.4 Tuple finish cleanup

For one builder scratch local:

```text
... builder_local result
SetLocal(builder_slot)
Pop
```

Then drop the builder local's compiler metadata.

The physical builder slot now holds the Tuple/Unit result.

## 8.5 General helper

Recommended compiler utility:

```rust
fn collapse_scratch_locals_preserving_top(
    &mut self,
    first_slot: u16,
    scratch_count: usize,
    range: SourceRange,
)
```

or an equivalent narrowly-scoped helper.

Preconditions:

- scratch locals are contiguous and at the top of the current frame's local area;
- `first_slot` is the earliest scratch slot;
- one result value is on TOS;
- no scratch local escapes its lowering region.

This helper should update both bytecode stack shape and compiler local metadata consistently.

---

# 9. `**` expansion

`PackExpandLabels` accepts only:

```text
Tuple
Unit
Record
Map
```

## 9.1 Tuple

Append labeled lane only.

Ignore all positional values.

Preserve Tuple label order.

## 9.2 Unit

No contribution.

## 9.3 Record

Append fields in Record encounter order.

Do not sort.

## 9.4 Map

Iterate native map entries in insertion/encounter order.

For each association, in order:

1. require key to be `Symbol`;
2. reject duplicate against all earlier builder labels;
3. append `(key, value)`.

Failure occurs at the first offending association.

A valid Map with a non-Symbol key is an invalid `**` operand contribution.

Do not:

- coerce String keys to Symbol;
- invoke key `toString`;
- sort entries;
- accept Iterable-of-pairs.

## 9.5 Atomicity

`**` is incrementally committed.

If an expansion contains valid entries followed by an invalid/duplicate entry, earlier entries may already exist in the private builder when the error is raised.

That partial state is unobservable because the whole expression aborts and the builder never escapes.

There is no need to stage the entire expansion transactionally.

---

# 10. `***` expansion

`PackExpandComplete` accepts:

```text
Tuple
Unit
```

only.

## 10.1 Tuple

Append:

1. all positional-lane values;
2. all labeled-lane entries in Tuple label order.

Each label is duplicate-checked as it is appended.

## 10.2 Unit

No contribution.

## 10.3 Reject

Reject:

```text
Record
Map
List
arbitrary Iterable
```

even if a type appears structurally compatible.

Older `***record` drafts are obsolete.

## 10.4 Multiple complete expansions

Legal:

```phalcom
target(***a, later, ***b)
```

All positionals contribute to the canonical positional lane.

All labels contribute to the canonical labeled lane in label encounter order.

---

# 11. Canonical lanes versus lexical evaluation order

These are intentionally different concepts.

For:

```phalcom
target(***pack, later, ***second)
```

evaluation is lexical:

```text
evaluate pack
fully contribute pack
evaluate later
evaluate second
fully contribute second
```

Final storage/dispatch order is canonical:

```text
receiver
all positional argument values
all labeled argument values
```

Labels preserve their own encounter order.

Dynamic selector construction uses:

```text
P = number of positional values
L = labels in builder encounter order
```

It does not attempt to reconstruct source interleaving between positional and labeled contributions.

---

# 12. `*` Tuple lane and Unit lane

Tuple's ordinary iteration semantics are not the desired spread semantics because Tuple iteration may represent total product order.

`*tuple` is defined as positional-lane projection.

Lower:

```text
evaluate operand exactly once
store in hidden source local

GetLocal(source_slot)
GetLocal(builder_slot)
PackTryExpandTuplePositionals
JumpIfFalse(generic_iterable)
Jump(done)

generic_iterable:
    ... cursor loop using source_slot ...

done:
```

Unit is handled as an empty positional lane and returns `true`.

The Tuple/Unit fast lane performs no user dispatch and is always finite.

---

# 13. Generic Iterable `*`

For a non-Tuple/non-Unit operand, use the ordinary cursor protocol.

Semantics:

```text
source = operand
cursor = source.iterate(None)

while cursor != None:
    value = source.iteratorValue(cursor)
    builder.pushPositional(value)
    cursor = source.iterate(cursor)
```

Use the same selector protocol and `JumpIfNone` mechanics as `for`.

Do not call:

```text
each
```

and do not exhaust in Rust.

## 13.1 Reuse existing cursor-loop conventions

The current `for` lowering already owns important details such as:

- evaluating collection once;
- synthetic source/cursor locals;
- ordinary `iterate(_)` and `iteratorValue(_)` sends;
- `JumpIfNone`;
- loop back-edge bytecode;
- cleanup of synthetic scope.

F.2 should reuse or factor the smallest safe internal helper from those conventions.

Do not force `*` expansion through `compile_for` itself: `*` has no user loop binding, break/continue target, or statement body.

A dedicated internal "emit cursor exhaustion" helper is reasonable if it keeps both lowerings behaviorally aligned without complicating `compile_for`.

## 13.2 Per-element body

For each yielded `value`:

```text
GetLocal(builder_slot)
PackPushPositional
```

using the chosen `value,builder` opcode order.

## 13.3 Non-Iterable source

A non-Iterable that reaches the generic lane must fail as a catchable language/runtime error.

It is acceptable to let ordinary missing `iterate(_)` dispatch become the error if that matches current Iterable conventions.

If F.2 sharpens it into a dedicated `NonIterableStarOperand` error, do so without native looping or panic.

---

# 14. E.3 boundedness integration

The boundedness check applies only when the compiler would emit generic iterable exhaustion.

Required behavior:

```phalcom
target(*(0..))                    // compile error: known unbounded
target(*((0..).iter.take(3)))     // legal
target(*unknownIterator)          // legal
```

Rule:

```text
Unbounded -> reject
Bounded   -> compile
Unknown   -> compile
```

Tuple/Unit direct lane is finite and bypasses generic exhaustibility rejection.

Because the source expression's runtime type may be unknown, the compiler may need to run E.3 source-fact analysis before the runtime Tuple lane test. The semantic requirement is:

- do not reject an expression E.3 can prove is a Tuple/Unit finite direct lane;
- do reject a source E.3 proves is an unbounded generic exhaustion source;
- otherwise compile and let the runtime lane test choose Tuple/Unit versus Iterable.

Use E.3's actual landed API.

Do not duplicate:

- Range analysis;
- progression analysis;
- `take`/pipeline boundedness propagation;
- exhaustor diagnostics.

---

# 15. Evaluation timing

For every dynamic pack:

1. receiver first;
2. each pack item in lexical source order;
3. each item operand exactly once;
4. an expansion completes before the next source item begins.

For:

```phalcom
target(*sourceA(), later())
```

all effects required to fully exhaust `sourceA()` occur before `later()` begins.

For:

```phalcom
target([labelExpr()]: valueExpr())
```

order is:

```text
labelExpr
Symbol validation
duplicate reservation
valueExpr
fill
```

For:

```phalcom
target(existing: sideEffect())
```

inside a dynamic pack, if `existing` duplicates an earlier dynamic contribution, the duplicate error happens before `sideEffect()`.

---

# 16. Runtime selector encoding

Runtime selector construction must share the same canonical label escaping rules as F.1.

Do not:

- use user `toString`;
- concatenate raw label text without escaping;
- duplicate a second incompatible encoder.

## 16.1 Helper

Add a runtime-oriented helper, for example:

```rust
pub fn encode_selector_from_symbols(
    name: &str,
    positional_count: usize,
    labels: &[Symbol],
    kind: PackSendKind,
    interner: &Interner,
) -> Result<String, PackSelectorError>
```

or factor the lower-level slot renderer so static and runtime encoders share it.

The important design requirement is one canonical `encode_label_component`.

## 16.2 Method

Given:

```text
name = "send"
P = 1
labels = [#timeout, #onError]
```

derive slots:

```text
_,timeout,onError
```

then:

```text
send(_,timeout,onError)
```

after applying canonical escaping to label components.

## 16.3 Subscript get

Ignore base name and derive bracket form:

```text
[_,label]
```

from canonical positional then labeled lanes.

## 16.4 Subscript set

The builder includes compiler-owned final `#put`.

Validation before encoding:

```text
labels.last() == #put
```

Let:

```text
total_args = P + labels.len()
index_args = total_args - 1
source_labels = labels without final #put
```

Construct the bracket slots from:

```text
P positional slots
then source_labels
```

and encode:

```text
[slots]=(put)
```

The assigned value remains the final flattened argument.

This matches the existing `SignatureKind::SubscriptSet(index_arity)` architecture, where `put` is a fixed setter role outside the bracket-label list.

---

# 17. Dynamic send arity

After the builder is complete and before lookup:

```text
actual_arity = positionals.len() + labels.len()
```

Require:

```text
actual_arity <= u8::MAX
```

For subscript set, this count includes the implicit `put` value.

Tuple construction has no send-arity limit.

Do not check before expansion completes because expansion determines the actual count.

Error occurs before:

- selector lookup;
- variadic fallback;
- dNU forwarding;
- callee execution.

Static source sends continue using the existing compiler-side checked arity logic.

---

# 18. Stack rewrite and builder extraction

## 18.1 Do not clone pack vectors unnecessarily

At `InvokePack`, the builder is transient and will never be reused.

Prefer moving its vectors out:

```rust
let (positionals, labels, labeled_values) =
    self.heap.pack_builder_mut(builder_ref).take_parts();
```

using `mem::take` or equivalent.

The collector only runs at VM safepoints between opcodes, so moving `Value`s through Rust locals during one opcode is compatible with the current no-mid-opcode-GC invariant.

## 18.2 Rewrite

Starting stack:

```text
..., receiver, builder
```

Steps:

1. validate builder kind;
2. validate no pending label;
3. take parts;
4. derive/validate selector and arity;
5. remove builder;
6. append all positional values;
7. append all labeled values;
8. dispatch using `receiver_idx`.

Resulting dispatch window:

```text
receiver,
positional values...,
labeled values...
```

No builder remains in the callee argument window.

## 18.3 Failure before rewrite where practical

Failures that do not require flattening should be checked before mutating the operand window, especially:

- pending builder invariant;
- send arity limit;
- malformed SubscriptSet final `#put`;
- selector derivation failure.

This makes error cleanup easier to reason about.

A runtime error aborts the expression, so rollback of private builder contents is not required.

---

# 19. Dynamic dispatch path

Add an uncached dynamic dispatch helper, for example:

```rust
fn invoke_dynamic_selector_on_current_stack(
    &mut self,
    receiver_idx: usize,
    selector: Symbol,
    arity: usize,
    source_range: SourceRange,
) -> PhResult<()>
```

This is illustrative, not a mandated name.

It must preserve ordinary dispatch semantics:

1. exact selector lookup;
2. access authorization;
3. variadic fallback where the selector kind is eligible;
4. dNU forward on miss;
5. normal `call_method` entry;
6. no re-entrant `send_dynamic`.

## 19.1 Static hot path

Do not add:

```rust
CachePolicy::Static | CachePolicy::Dynamic
```

to a helper called unconditionally by ordinary `Invoke` unless profiling proves the branch free/beneficial.

Prefer keeping the existing cached `invoke_at` path intact.

## 19.2 Variadic fallback

Dynamic method sends must behave like ordinary method sends.

After exact miss, an all-positional Method selector may probe the canonical variadic selector according to the existing rule.

Labeled method selectors and subscript selectors do not become eligible merely because they were dynamically assembled.

## 19.3 dNU

On dynamic miss, dNU sees:

- the actual derived concrete selector Symbol;
- flattened canonical args;
- decoded raw label texts through the normal selector decoder;
- the normal single-forward behavior.

It must not receive an F.1 parser transport representation or the private pack builder.

---

# 20. Compiler-internal dynamic send authority

Current compiler-generated internal sends use a temporary VM authority mechanism.

Dynamic packing must preserve that behavior.

`InvokePack(... access: CompilerInternal)` should conceptually:

```text
increment compiler_internal_dispatch_depth
perform dynamic lookup/authorization/call_method entry
decrement compiler_internal_dispatch_depth
```

matching `InvokeCompilerInternal`.

For closure methods, authority must be gone before the newly pushed callee frame begins executing.

For primitives, preserve the existing authority lifetime semantics of compiler-internal invocation.

Ordinary source dynamic sends always use:

```text
PackAccess::Ordinary
```

The compiler remains responsible for rejecting unauthorized source references to the internal namespace before this point.

---

# 21. Dynamic super sends

Support:

```phalcom
super.foo(*args)
super.foo(**labels)
super.foo([computed]: value)
```

Lower to `SuperSendPack`.

Before assembly, perform the same constructor-name rewrite as the static super path:

```text
source constructor name
    ->
hidden "init <name>" base name
```

when applicable.

Lookup must begin above the defining class exactly like `SuperSend`.

Do not lower to ordinary receiver lookup.

Dynamic super sites are uncached in F.2 unless the existing super path already provides a safe selector-sensitive cache abstraction.

The defining class constant remains statically known.

---

# 22. Dynamic unqualified sends

`Expr::UnqualifiedCall` must branch after bare-name resolution.

## 22.1 Implicit self

For:

```phalcom
foo(*args)
```

when resolution is `ImplicitSelf`:

```text
receiver = self
base_name = "foo"
kind = Method
dynamic pack
InvokePack
```

Sacred-inliner behavior should remain consistent with current explicit/implicit call architecture. Dynamic packs are not candidates for a static sacred selector/arity fast path unless the existing recognizer can prove the equivalent shape without violating F.2 timing.

## 22.2 Local/upvalue/global callable

When `foo` resolves to a value:

```text
load callable value
base_name = "call"
kind = Method
dynamic pack
InvokePack
```

This preserves the current callable protocol; dynamic arguments do not create a separate invocation mechanism.

---

# 23. Dynamic ordinary method sends

For `Expr::MethodCall` with `needs_dynamic_pack(args)`:

1. perform existing semantic checks that must precede ordinary compilation;
2. preserve internal-namespace validation;
3. preserve constructor/special rewrites already applicable to that form;
4. allocate receiver + builder scratch locals;
5. evaluate receiver exactly once into receiver scratch;
6. allocate builder into builder scratch;
7. compile items in lexical order;
8. load receiver scratch;
9. load builder scratch;
10. emit `InvokePack`;
11. collapse scratch locals while preserving result.

Static `MethodCall` remains on existing code.

---

# 24. Dynamic subscript reads

For:

```phalcom
obj[*indices, label: x]
```

use:

```text
PackSendKind::SubscriptGet
```

No `at` desugaring.

Lower like an ordinary dynamic method send:

1. receiver scratch;
2. builder scratch;
3. lexical item assembly;
4. load receiver;
5. load builder;
6. `InvokePack(SubscriptGet)`;
7. scratch collapse preserving result.

---

# 25. Dynamic subscript writes

This path has stricter ordering.

Required language order:

```text
receiver
all source subscript pack items
implicit `put` reservation
RHS
setter send
result = original RHS
```

## 25.1 Scratch locals

Recommended adjacent order:

```text
receiver_slot
builder_slot
rhs_slot
```

Reserve all compiler metadata/placeholder slots without evaluating user expressions.

## 25.2 Lowering

1. Evaluate receiver exactly once; save to `receiver_slot`.
2. Allocate/save builder to `builder_slot`.
3. Assemble all source subscript items in lexical order.
4. Reserve compiler-owned static label `#put`:

   ```text
   GetLocal(builder_slot)
   PackReserveStaticLabel(put_symbol_constant)
   ```

   A duplicate fails here, before RHS evaluation.

5. Evaluate RHS exactly once.
6. Save RHS to `rhs_slot` while leaving/obtaining a copy suitable for fill.
7. Fill reserved `put` with the RHS:

   ```text
   GetLocal(rhs_slot)
   GetLocal(builder_slot)
   PackFillReservedLabel
   ```

8. Load receiver + builder.
9. Emit:

   ```text
   InvokePack {
       kind: SubscriptSet,
       ...
   }
   ```

10. Discard setter return.
11. Load original RHS from `rhs_slot`.
12. Collapse all three scratch locals while preserving that RHS as the expression result.

## 25.3 Static setter path

The existing static-path compile-time duplicate-`put` rejection remains unchanged.

Dynamic setter duplicates involving runtime contributions are detected by the builder reservation timing above.

---

# 26. Dynamic Tuple construction

A Tuple literal requires dynamic pack assembly if any entry has:

- `*`;
- `**`;
- `***`;
- computed label.

Tuple product labels and call pack labels use different AST types, so share lowering semantics through compiler helpers rather than forcing one AST enum into the other.

Lower:

1. allocate builder hidden local;
2. assemble entries lexically;
3. load builder;
4. `FinishTuplePack`;
5. collapse builder scratch while preserving result.

`FinishTuplePack`:

- asserts no pending reservation;
- moves/takes pack parts;
- calls A.3's product finalizer;
- yields `Unit` for zero entries;
- yields Tuple otherwise;
- preserves label encounter order;
- does not stage through List;
- is not subject to `u8` send arity.

Static Tuple literals remain on the existing `BuildTuple` path.

---

# 27. Shared compiler assembly helpers

Avoid copying the same item-lowering logic into method, super, subscript, unqualified, and Tuple paths.

Recommended internal decomposition:

```rust
fn needs_dynamic_pack(items: &[PackItem]) -> bool;

fn begin_dynamic_pack_region(...) -> Result<PackScratch, CompilerError>;

fn compile_dynamic_pack_items(
    &mut self,
    builder_slot: u16,
    items: Vec<PackItem>,
    range: SourceRange,
) -> Result<(), CompilerError>;

fn compile_dynamic_pack_item(
    &mut self,
    builder_slot: u16,
    item: PackItem,
) -> Result<(), CompilerError>;

fn collapse_scratch_locals_preserving_top(...);
```

Tuple literals may use a sibling item adapter because they carry `TupleLiteralEntry`/`ProductLabel`.

Keep helpers semantically narrow.

Do not build a large generic "call lowering framework" in F.2 unless existing duplication genuinely demands it.

---

# 28. Item lowering recipes

## 28.1 Positional

```text
compile expr
GetLocal(builder)
PackPushPositional
```

## 28.2 Static labeled

```text
GetLocal(builder)
PackReserveStaticLabel(label)

compile value

GetLocal(builder)
PackFillReservedLabel
```

Reservation precedes value evaluation.

## 28.3 Computed labeled

```text
compile label_expr
GetLocal(builder)
PackReserveComputedLabel

compile value_expr
GetLocal(builder)
PackFillReservedLabel
```

Computed label must be Symbol.

No coercion.

## 28.4 `**`

```text
compile operand
GetLocal(builder)
PackExpandLabels
```

Operand evaluated once.

## 28.5 `***`

```text
compile operand
GetLocal(builder)
PackExpandComplete
```

Operand evaluated once.

## 28.6 `*`

```text
compile operand
save to source scratch

GetLocal(source)
GetLocal(builder)
PackTryExpandTuplePositionals
JumpIfFalse(generic)
Jump(done)

generic:
    E.3-approved cursor exhaustion from source scratch
    append each value with PackPushPositional

done:
    clean source/cursor scratch locals
```

The direct Tuple path must not leave the source scratch local or Bool result behind.

---

# 29. Error taxonomy

F.2 failures are language/runtime failures unless explicitly defined as compile-time boundedness diagnostics.

Recommended variants:

```rust
ComputedLabelNotSymbol {
    found: &'static str,
}

DuplicateArgumentLabel {
    label: Symbol,
}

InvalidStarStarOperand {
    found: &'static str,
}

NonSymbolMapKeyInExpansion {
    found: &'static str,
}

InvalidStarStarStarOperand {
    found: &'static str,
}

SendArityExceedsLimit {
    found: usize,
    limit: usize,
}

NonIterableStarOperand {
    found: &'static str,
}
```

Exact names may follow repository conventions.

## 29.1 Rendering labels

If `DuplicateArgumentLabel` stores a Symbol, error rendering must resolve it through VM/interner context somewhere appropriate.

If the current `RuntimeError` display architecture cannot resolve Symbols directly, store a stable rendered label string at the error-construction boundary instead.

Do not expose numeric Symbol IDs to users.

## 29.2 Existing error conventions

If ordinary missing `iterate(_)` already produces the preferred language-level non-Iterable error, `NonIterableStarOperand` may be omitted and the existing error reused.

The normative requirement is the error category/behavior, not enum inflation.

## 29.3 No user-visible panic

User data may never trigger a Rust panic for:

- wrong computed-label type;
- duplicates;
- invalid expansion source;
- non-Symbol Map key;
- oversized dynamic send.

Internal impossible states may assert only after compiler/bytecode invariants make them unreachable from valid source.

---

# 30. GC, fibers, and safepoints

## 30.1 Builder reachability

During item compilation/execution, builder is rooted by its frame hidden local.

During fiber parking, that local lives in the parked stack and remains traced.

## 30.2 Source/cursor reachability

Generic `*` source and cursor are hidden locals, so arbitrary iteration sends and fiber switches cannot lose them.

## 30.3 No native frame around generic exhaustion

The expansion loop is bytecode.

A lazy iterator callback that calls `Fiber.yield` must have the same legality as the equivalent `for` loop.

## 30.4 Mid-opcode Rust locals

Current GC service occurs at dispatch-loop safepoints, not in the middle of an opcode.

Therefore moving builder vectors into Rust locals inside `InvokePack`/`FinishTuplePack` is safe under the current collector model.

Do not introduce a collection safepoint inside those opcodes without revisiting this assumption.

---

# 31. Disassembler and diagnostics

Update the disassembler so the new opcodes are intelligible.

At minimum, display:

```text
NewArgumentPack
PackPushPositional
PackReserveStaticLabel(<symbol>)
PackReserveComputedLabel
PackFillReservedLabel
PackExpandLabels
PackExpandComplete
PackTryExpandTuplePositionals
InvokePack(<base>, <kind>, <access>)
SuperSendPack(<base>, <defining class>)
FinishTuplePack
```

Resolve constant-backed symbols when practical, matching current `Invoke` formatting.

Do not dump private builder contents during normal disassembly.

Runtime error source spans should use the pack opcode/item source range conventions consistently:

- duplicate explicit label should point at the contribution;
- computed-label type failure should point at the computed-label expression/contribution;
- invalid expansion should point at that expansion;
- final send arity failure should point at the whole send.

If the current opcode-span model only provides one span per instruction, emit pack opcodes with the tightest useful source span.

---

# 32. Performance guidance

F.2 is a cold/less-common path relative to ordinary sends.

Optimize for correctness and isolation first.

## 32.1 Required performance properties

- ordinary static calls allocate no pack builder;
- static `Invoke` gets no new selector comparison;
- static IC structure remains unchanged;
- `PackReserve*` uses Symbol identity;
- builder vectors move into dispatch instead of being cloned;
- no user hash/equality is invoked for duplicate labels;
- generic `*` performs no per-element Rust re-entry.

## 32.2 Initial builder allocation

One heap builder allocation per dynamic pack is accepted.

Each internal `Vec` may allocate as it grows.

Do not pre-count dynamic expansion sizes by iterating sources twice.

Optional small capacity hints may use static source information only if they do not change evaluation or add complexity.

## 32.3 Future optimization seams

Leave room for:

- small-vector builder storage;
- thresholded duplicate hash acceleration;
- selector-sensitive dynamic inline cache;
- exact-size reservation for known Tuple expansion;
- specialized pack superinstructions.

None belongs in the first correctness implementation unless measurement demonstrates an immediate problem.

---

# 33. Implementation phases

## Phase 0 — prerequisite/state check

Before coding:

- confirm F.1 AST shapes currently match the assumed enums;
- confirm A.3 `finish_tuple` API;
- confirm E.3 is landed and identify its actual public compiler hook;
- confirm static checked arity remains present;
- identify current `SuperSend` lookup helper;
- identify all exhaustive `Object` matches affected by `PackBuilder`.

Do not proceed by copying obsolete function signatures from an older plan.

## Phase 1 — heap builder

Files likely include:

```text
phalcom-core/src/heap/pack_builder.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/trace.rs
memory-management documentation
```

Implement:

- boxed object;
- accessors;
- allocation;
- duplicate reservation;
- direct labeled append;
- take-parts;
- GC tracing.

Add focused Rust unit tests for builder invariants before VM integration.

## Phase 2 — bytecode definitions

Modify bytecode enum/table/index/disassembler.

Add 11 variants.

Use typed send kind/access operands.

Compile immediately after this phase so exhaustive matches identify every required update.

## Phase 3 — pack mutation opcode handlers

Implement:

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

Test stack effects independently where feasible.

Do not implement dynamic dispatch yet if keeping phases small improves reviewability.

## Phase 4 — selector runtime helper

Implement one shared runtime label-encoding path using F.1 escaping.

Add tests for:

- ordinary labels;
- reserved `#*`, `#**`, `#***`, `#_`;
- delimiter-containing labels;
- Unicode/escaped labels;
- method;
- subscript get;
- subscript set source labels excluding final `put`.

## Phase 5 — uncached dynamic dispatch

Implement:

- stack flatten;
- runtime arity check;
- exact lookup;
- variadic fallback;
- dNU;
- compiler-internal access mode;
- `InvokePack`.

Preserve static `invoke_at`.

## Phase 6 — dynamic super dispatch

Implement:

- selector derivation;
- stack flatten;
- lookup above defining class;
- normal dNU behavior;
- `SuperSendPack`.

Mirror existing `SuperSend` semantics.

## Phase 7 — compiler scratch helpers

Implement:

- `needs_dynamic_pack`;
- scratch-local reservation/storage;
- scratch collapse preserving TOS;
- shared dynamic pack item lowering;
- generic `*` cursor emission using E.1/E.3.

Unit-test compiler stack/local accounting if existing compiler tests expose max slots or disassembly.

## Phase 8 — method/unqualified/super lowering

Wire:

- ordinary method;
- implicit self;
- local/upvalue/global callable `call`;
- compiler-internal dynamic method;
- super.

Keep static path untouched.

## Phase 9 — subscript read/write

Wire:

- `SubscriptGet`;
- dynamic setter `put` reservation before RHS;
- subscript-set selector exclusion rule;
- original RHS result preservation.

## Phase 10 — Tuple literals

Wire dynamic Tuple assembly and `FinishTuplePack`.

Keep static `BuildTuple`.

## Phase 11 — errors, docs, integration tests, performance regression

Complete:

- structured runtime variants;
- error-kind mapping if applicable;
- source diagnostics;
- disassembler;
- memory-management docs;
- spec/checklist docs;
- full test suite;
- static-call benchmark comparison.

---

# 34. Test plan

## 34.1 Builder unit tests

- positional append;
- static reservation then fill;
- computed Symbol reservation;
- duplicate Symbol rejected;
- direct expansion labeled append;
- pending index targets exact placeholder;
- `take_parts` empties builder;
- `labels.len == labeled_values.len`;
- no user hash/equality path involved.

## 34.2 Evaluation order

Use explicit counters/logs proving:

- receiver executes first;
- each operand executes once;
- `*` fully exhausts before next item;
- `**` operand executes once;
- `***` operand executes once;
- static label reservation precedes value expression;
- computed label expression precedes Symbol check;
- Symbol check precedes value;
- duplicate computed label prevents value;
- duplicate from earlier `***` prevents later explicit value;
- setter duplicate `put` prevents RHS.

## 34.3 `*` positive

- `*Tuple` uses positional lane only;
- `*Unit` empty;
- `*List`;
- `*Bytes`;
- finite `*Range`;
- `*Iterator`;
- user Iterable subclass;
- lazy pipeline;
- unknown source compiles and chooses runtime path.

## 34.4 `*` negative/boundedness

- known unbounded Range compile error;
- bounded `take` legal;
- non-Iterable runtime error;
- `*Map` follows Map's actual Iterable status; if Map is non-Iterable in the current language, reject through ordinary iteration failure.

Do not make the test assume a forever-non-Iterable Map if the language later intentionally gives Map an Iterable protocol.

## 34.5 `**`

Positive:

- Tuple labeled lane;
- Unit;
- Record;
- Map;
- stable Map order;
- stable Record order.

Negative:

- List;
- non-Symbol Map key;
- duplicate against explicit earlier label;
- duplicate across two `**`;
- duplicate across `***` and `**`.

## 34.6 `***`

Positive:

- Tuple positional + labels;
- Unit;
- multiple complete expansions;
- later source positionals after a `***` with labels.

Negative:

- Record;
- Map;
- List.

## 34.7 Dynamic method dispatch

- runtime positional count selects correct overload;
- runtime label set/order selects correct selector;
- escaped `#*` label resolves concrete label selector, not variadic rest;
- exact miss uses variadic fallback only when eligible;
- labeled dynamic call does not spuriously use variadic fallback;
- dNU receives concrete selector and flattened args;
- member visibility is enforced;
- compiler-internal dynamic send preserves internal authority;
- ordinary source cannot obtain compiler-internal authority.

## 34.8 Unqualified calls

- implicit-self `foo(*args)`;
- local callable `f(*args)`;
- upvalue callable;
- global callable;
- computed labels on callable `call`;
- dynamic arity over 255.

## 34.9 Super

- dynamic exact override skips current defining class and starts above it;
- dynamic super miss/dNU;
- constructor super rewrite;
- dynamic label selector;
- dynamic arity.

## 34.10 Subscript reads

- dynamic positional count;
- dynamic labels;
- `*indices`;
- `**labels`;
- computed labels;
- dNU receives bracket selector.

## 34.11 Subscript writes

- receiver before index items;
- index items before RHS;
- implicit `put` duplicate before RHS;
- RHS evaluated once;
- setter receives canonical args;
- selector is `[source slots]=(put)`, never `[source slots,put]=(put)`;
- setter return discarded;
- expression yields original RHS;
- 255 total args accepted;
- 256 total args rejected after expansion.

## 34.12 Tuple

- dynamic Tuple equality;
- lane access;
- mixed static/dynamic entries;
- empty dynamic expansion -> Unit;
- labels preserve encounter order;
- `*Tuple` ignores labels;
- `**Tuple` ignores positionals;
- `***Tuple` contributes both;
- no send arity limit for large Tuple.

## 34.13 Fibers

Regression:

```text
lazy iterator stage
    -> callback executes
    -> Fiber.yield during `*`
    -> resumes
    -> expansion completes
```

Must not raise `CannotYieldAcrossNativeFrame` merely because spread is occurring.

Also test fiber switching with:

- builder containing already-collected values;
- source/cursor hidden locals;
- nested dynamic call during iteration.

## 34.14 GC stress

Under aggressive GC/stress mode if available:

- builder values remain live across nested sends;
- builder survives fiber park/resume;
- objects contributed before a later expansion are not collected;
- `InvokePack` take-parts/flatten does not lose roots at a safepoint;
- abandoned builder after error is collectible.

## 34.15 Static regression

Confirm unchanged disassembly/behavior for representative static calls.

At minimum:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
obj.foo(timeout: t)
obj[a, b]
```

Benchmark ordinary send microcases before/after.

F.2 is not complete if static `Invoke` materially regresses without an explicit measured tradeoff decision.

---

# 35. Verification commands

Use repository-current commands, approximately:

```bash
cargo fmt --check
cargo test -p phalcom-core
cargo test -p phalcom-ast
cargo test -p phalcom-core -- pack
```

Run any repository-standard clippy/lint target used by CI.

Run the pack integration fixture through the normal Phalcom CLI/test harness.

Disassemble representative dynamic and static samples.

If opcode histogram or VM benchmark tooling is present, compare:

```text
ordinary static send before F.2
ordinary static send after F.2
```

The expected outcome is no new pack opcodes executed and no pack allocation for static calls.

---

# 36. Correctness invariants

The implementation must satisfy all of these.

- [x] Static calls never allocate `ArgumentPackBuilderObject`.
- [x] Builder is rooted by a hidden local whenever arbitrary bytecode may run.
- [x] Temporary builder operand copies are consumed by pack opcodes.
- [x] No pack opcode depends on a fragile fixed stack depth.
- [x] Hidden scratch locals are physically collapsed after the dynamic expression.
- [x] Dynamic send result occupies exactly one stack value after scratch cleanup.
- [x] Computed label is checked before its value expression.
- [x] Duplicate explicit label is rejected before its value expression.
- [x] Duplicate setter `put` is rejected before RHS.
- [x] Duplicate checks use Symbol identity only.
- [x] `**` preserves encounter order.
- [x] `***` accepts Tuple/Unit only.
- [x] `*Tuple` projects positional lane only.
- [x] Generic `*` uses ordinary cursor bytecode.
- [x] Generic `*` reuses E.3 boundedness analysis.
- [x] Dynamic selector uses canonical F.1 label escaping.
- [x] Subscript-set final `put` is excluded from bracket label slots.
- [x] Dynamic arity is checked before lookup.
- [x] Tuple finish has no `u8` arity limit.
- [x] Dynamic method miss preserves rest-family fallback semantics.
- [x] Dynamic dNU gets concrete selector + flat args.
- [x] Dynamic super starts above defining class.
- [x] Compiler-internal dynamic sends preserve access authority.
- [x] `InvokePack` never calls re-entrant `VM::send_dynamic`.
- [x] Fiber yield in generic `*` remains legal.
- [x] Pack builder GC tracing includes every stored `Value`.
- [x] Static inline cache shape and ordinary hot path are not degraded.
- [x] No public primitive binding is added.

---

# 37. Completion checklist

F.2 closure evidence is recorded in `F.2-supplement-completion-gaps-before-F.3.md`.

- [x] `ArgumentPackBuilderObject` exists as a private boxed heap variant.
- [x] GC tracer and memory-management edge docs include it; focused forced-GC coverage traces PackBuilder-held values.
- [x] Builder has positional, label, labeled-value, pending-index state.
- [x] All 11 pack bytecodes exist and are named/indexed/disassembled.
- [x] Pack bytecodes obey the exact consuming stack contracts in this document.
- [x] Static path remains unchanged when `needs_dynamic_pack == false`; focused disassembly contains no pack machinery for static calls or Tuples.
- [x] Dynamic ordinary method sends work.
- [x] Dynamic implicit-self sends work.
- [x] Dynamic callable `call(...)` sends work.
- [x] Dynamic compiler-internal sends preserve authority; source cannot forge that authority.
- [x] Dynamic super sends work.
- [x] Dynamic subscript reads work.
- [x] Dynamic subscript setters implement pre-RHS `put` reservation.
- [x] Dynamic subscript setter selector excludes builder's final `#put` from bracket slots.
- [x] Dynamic Tuple construction uses `finish_tuple`, including construction above 255 elements.
- [x] `*Tuple`/`*Unit` direct lane works.
- [x] Generic `*` uses ordinary cursor protocol.
- [x] E.3 known-unbounded rejection is reused.
- [x] `**` and `***` semantics match the F.2 language spec.
- [x] Runtime selector encoding shares F.1 escaping.
- [x] Runtime dynamic arity check rejects >255 before lookup; setter arity includes implicit `put`.
- [x] dNU and variadic fallback match ordinary send behavior.
- [x] Scratch locals are cleaned without leaking stack slots.
- [x] Fiber regression passes.
- [ ] Every-safepoint `PHALCOM_GC_STRESS` verification: infrastructure is implemented by the completion patch; check this row only after the stress commands pass.
- [ ] Static send benchmark shows no unexplained regression. Focused static/dynamic disassembly verifies fast-path shape, not benchmark data.
- [x] Public primitive-floor delta is 0.

---

# 38. Decision record summary

| Decision | Ruling |
|---|---|
| Builder storage | Boxed private heap object |
| Builder ownership during arbitrary bytecode | Hidden local |
| Temporary builder loads | Consumed by every pack opcode |
| Invoke stack rewrite target | Top `receiver,builder` operand window |
| `stack_offset` special handling | None |
| Pack bytecodes | Dedicated variants |
| Kind representation | Typed enum, not arbitrary `u8` |
| Dynamic cache | Uncached in F.2 |
| Static dispatch refactor | Avoid hot-path policy branch |
| Duplicate labels | Linear Symbol-identity scan initially |
| Pending reservation | `Option<usize>` |
| Computed label guard | Inside reserve opcode |
| `*Tuple` | Positional lane only |
| `*Unit` | Empty, handled |
| Generic `*` | Ordinary cursor bytecode |
| E.3 | Reuse landed shared analyzer |
| `**` | Tuple/Unit/Record/Map only |
| `***` | Tuple/Unit only |
| Selector escaping | Shared F.1 encoder/component rules |
| Dynamic send arity | Runtime `<=255` |
| Tuple arity | Unlimited by send `u8` |
| Subscript setter `put` | Stored/final arg, excluded from bracket slot labels |
| Unqualified dynamic calls | Required |
| Compiler-internal dynamic calls | Preserve authority with typed access mode |
| dNU | Concrete selector + flattened canonical args |
| Public primitive bindings | None |

---

# 39. Non-goals / follow-ups

F.2 does not include:

- widening VM send arity beyond `u8`;
- arbitrary Iterable-of-pairs for `**`;
- Record/Map support for `***`;
- user-overridable spread protocol;
- public pack-builder reflection;
- dynamic-site polymorphic inline caching;
- serialization format changes for bytecode;
- native spread primitives;
- changes to Tuple's ordinary iteration semantics;
- widening compiler-internal namespace access;
- a new general stack-shuffle opcode unless scratch-collapse proves impossible with existing `SetLocal`/`Pop`.

Possible measured follow-ups:

```text
DynamicInlineCache
small-vector builder
duplicate-label hash acceleration
known-Tuple capacity reservation
pack bytecode fusion
```

Only pursue these after F.2 correctness and profiling.

---

# 40. Final implementation guidance

The most important engineering principle for F.2 is separation of responsibilities:

```text
compiler:
    preserves lexical timing
    roots transient state in hidden locals
    emits ordinary iteration bytecode

builder:
    owns canonical lanes
    enforces label identity/duplicate invariants

selector encoder:
    owns canonical selector spelling and escaping

dynamic dispatch:
    flattens completed lanes
    preserves ordinary lookup semantics
    remains non-reentrant

static dispatch:
    stays fast and unchanged
```

Do not collapse these layers into one native "spread and send" helper.

That shortcut would be simpler locally but would violate the language's fiber behavior, blur evaluation timing, duplicate selector rules, and put dynamic-pack complexity on the static call path.

The intended F.2 implementation is therefore deliberately explicit: one transient GC-traced builder, compiler-generated cursor loops, dedicated pack opcodes with precise stack effects, and a narrow uncached dynamic dispatch seam.
