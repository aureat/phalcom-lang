# F.2 Supplement — Completion Gaps Required Before F.3

Status: **implementation supplement — required completion gate before F.3**

This supplement does **not** replace or reopen the ratified F.2 specification, **F.2 — Outgoing Pack Assembly and Dynamic Send (amended)**. It records confirmed implementation gaps in the current F.2 landing state and defines the work required to close them before F.3 — Rest Capture and Rest-Pattern Dispatch begins.

The original F.2 semantics remain authoritative. Where this supplement makes an implementation choice among alternatives already permitted by F.2, that choice is recorded explicitly below.

Requires:

- F.1 pack AST and selector syntax
- A.3 `finish_tuple`
- E.1 iteration cursor protocol
- E.3 boundedness/exhaustibility analysis
- the already-landed F.2 pack builder, bytecodes, and dynamic-send machinery

Expected public primitive-floor delta: **0**.

---

# 1. Mission

Complete the currently partial F.2 implementation so that every outgoing pack form required by F.2 is reachable from the compiler, produces the ratified runtime behavior, reports structured pack-specific failures, and has enough integration/regression coverage to serve as a stable prerequisite for F.3.

The supplement closes five implementation gaps:

1. enable the `*Tuple` / `*Unit` fast lane and generic iterable `*`;
2. integrate E.3 boundedness analysis for generic `*`;
3. implement dynamic subscript-write lowering;
4. implement dynamic Tuple-literal lowering;
5. add the missing structured runtime errors and F.2 test coverage.

Do **not** redesign already-landed F.2 machinery merely because some compiler paths have not yet reached it.

The target architecture remains:

```text
compiler
    -> hidden-local pack assembly
    -> existing pack bytecodes
    -> canonical completed pack
    -> existing dynamic selector/dispatch path
```

F.3 must start from a complete canonical outgoing-pack implementation, not from an F.2 implementation where some pack forms still fail during compilation.

---

# 2. Current landing state

The following state is treated as the baseline for this supplement.

## 2.1 Implemented and retained

The current implementation already contains enough F.2 infrastructure to preserve rather than replace:

- private `ArgumentPackBuilderObject` heap representation;
- GC integration for the builder;
- all 11 F.2 bytecode definitions;
- handlers for all 11 pack bytecodes;
- dynamic ordinary method sends;
- dynamic unqualified/implicit-self sends;
- dynamic callable sends;
- dynamic super sends;
- dynamic subscript reads;
- computed labels;
- `**` expansion;
- `***` expansion;
- canonical runtime selector encoding;
- dynamic send arity guard;
- current U9 variadic fallback behavior pending F.3 replacement;
- dNU forwarding;
- compiler-internal dynamic-send authority.

The repository currently passes:

```bash
cargo check -p phalcom-core
```

This supplement must preserve that baseline while closing the missing paths.

## 2.2 Confirmed missing paths

### Positional `*`

The compiler currently rejects positional `*` before dynamic pack lowering reaches the already-existing `PackTryExpandTuplePositionals` opcode.

Consequences:

- `*Tuple` does not reach its required positional-lane fast path;
- `*Unit` does not reach its required empty fast path;
- generic iterable `*` is unavailable;
- the existing VM handler is effectively unreachable from valid compiled source.

### Generic iterable `*` and E.3

The existing E.3 boundedness seam is not connected to outgoing positional expansion.

Consequences:

- known unbounded generic spread sources are not rejected through the ratified E.3 rule;
- bounded and unknown sources cannot lower to the normal cursor protocol;
- fiber-safe bytecode iteration required by F.2 is absent.

### Dynamic subscript writes

`SetIndex` remains tied to a statically-known pack-label shape.

Dynamic forms containing expansion or computed labels therefore still fail instead of lowering through `InvokePack(SubscriptSet)`.

### Dynamic Tuple construction

Tuple literals containing expansion/computed labels still reject those forms and remain limited to the static `BuildTuple` path.

The existing `FinishTuplePack` mechanism therefore does not yet provide the full source-level F.2 Tuple semantics.

### Structured runtime failures

F.2-specific failures currently collapse into generic `ArgumentError(String)` and/or generic type failures.

Dedicated structured errors are missing for the pack-specific cases required by this supplement.

### F.2 integration/regression coverage

Only minimal builder-level coverage exists.

The required dynamic-send, expansion, subscript, Tuple, fiber, GC, error, and static-regression tests are not sufficiently present to consider F.2 a stable prerequisite.

---

# 3. Supplement decisions

## S1 — Complete F.2; do not redesign landed bytecodes

**Decision:** retain the existing builder representation, 11 bytecodes, selector encoder, `InvokePack`, `SuperSendPack`, authority model, dynamic arity guard, dNU forwarding, and already-working expansion handlers.

The missing work is primarily:

```text
compiler reachability
compiler lowering
boundedness integration
error taxonomy
integration/regression verification
```

Do not introduce replacement opcodes or a second pack representation unless implementation discovers a correctness defect in the landed machinery.

Any such defect must be handled as an explicit deviation, not as incidental supplement scope growth.

---

## S2 — Positional `*` always lowers through the Tuple/Unit probe first

**Decision:** remove the blanket compiler rejection of positional expansion and lower every legal positional `*expr` using the already-ratified two-lane strategy:

```text
evaluate expr exactly once
store expr in a hidden source local

GetLocal(source)
GetLocal(builder)
PackTryExpandTuplePositionals

if true:
    Tuple/Unit contribution is complete
else:
    exhaust source through ordinary cursor bytecode
```

The VM opcode contract remains:

```text
..., operand, builder -> ..., Bool
```

with:

```text
Tuple -> append positional lane, true
Unit  -> append nothing, true
other -> no mutation, false
```

The generic path reloads the original operand from the hidden source local.

Do not make Tuple use ordinary iteration. `*Tuple` means positional-lane projection, not "whatever Tuple iteration currently yields."

Do not special-case Tuple/Unit in user-visible native methods.

---

## S3 — Generic `*` uses ordinary bytecode iteration and the landed E.3 analyzer

**Decision:** when `PackTryExpandTuplePositionals` returns false, lower generic positional expansion through the same cursor protocol used by ordinary iteration:

```text
cursor = source.iterate(None)

while cursor != None:
    value = source.iteratorValue(cursor)
    builder.pushPositional(value)
    cursor = source.iterate(cursor)
```

The loop must be compiler-generated ordinary bytecode.

Prohibited:

- exhausting the source in a Rust loop;
- routing through `each`;
- introducing a native spread helper;
- re-entering the interpreter from a native helper;
- evaluating the spread operand more than once.

This is required for fiber/yield safety.

### E.3 rule

Use the actual landed E.3 API. Do not duplicate boundedness logic in F.2.

Required semantic result:

```text
Unbounded -> compile error
Bounded   -> compile
Unknown   -> compile
```

The compiler must not reject Tuple/Unit finite direct-lane cases merely because generic exhaustion has an unboundedness rule.

No speculative E.3 function signature is mandated by this supplement.

---

## S4 — Dynamic subscript writes use the existing `SubscriptSet` pack path

**Decision:** dynamic subscript assignment must lower through the already-ratified `InvokePack(SubscriptSet)` mechanism.

A subscript write needs the dynamic path whenever its source index pack contains any F.2 dynamic-pack trigger, including:

- `*`;
- `**`;
- `***`;
- computed labels.

The required language evaluation order is:

```text
receiver
source index-pack items in lexical order
reserve implicit #put
RHS
setter dispatch
expression result = original RHS
```

The implicit `#put` must be reserved **before** evaluating the RHS so a duplicate introduced by a dynamic source contribution fails before RHS side effects occur.

The final builder contains:

```text
positionals = source positional indices
labels      = source labels..., #put
values      = source labeled values..., rhs
```

Selector derivation must exclude the final `#put` from the bracket-label list while still counting the RHS in send arity and flattening it as the final labeled value.

Correct:

```text
[source slots]=(put)
```

Incorrect:

```text
[source slots,put]=(put)
```

The setter's return value is discarded.

The overall assignment expression evaluates to the original RHS.

---

## S5 — Dynamic Tuple construction uses the builder and `FinishTuplePack`

**Decision:** Tuple literals containing any dynamic-pack trigger lower through a hidden `ArgumentPackBuilderObject` and the existing `FinishTuplePack` opcode.

Static Tuple literals remain on `BuildTuple`.

Dynamic Tuple lowering:

```text
allocate builder hidden local
assemble entries in lexical order
load builder
FinishTuplePack
collapse scratch local preserving result
```

`FinishTuplePack` must continue to:

- reject a pending unfilled reservation as an internal invariant;
- move/take the builder lanes rather than cloning them unnecessarily;
- finalize through A.3 `finish_tuple`;
- produce canonical Unit for an empty result;
- preserve labeled encounter order;
- avoid List/Map staging;
- impose no `u8` send-arity limit.

Dynamic Tuple construction is product assembly, not message dispatch.

---

## S6 — Add dedicated structured F.2 runtime errors

**Decision:** F.2-specific runtime failures must not be reduced to generic string-only `ArgumentError` values.

Add structured variants following repository naming conventions for at least:

```rust
ComputedLabelNotSymbol { ... }
DuplicateArgumentLabel { ... }
InvalidStarStarOperand { ... }
NonSymbolMapKeyInExpansion { ... }
InvalidStarStarStarOperand { ... }
SendArityExceedsLimit { found: usize, limit: usize }
NonIterableStarOperand { ... }
```

This supplement selects a dedicated `NonIterableStarOperand` as well, rather than relying on an incidental dNU/missing-`iterate` diagnostic. The generic iteration implementation may internally discover non-iterability through ordinary protocol lookup, but the surfaced failure should identify that the invalid operation was positional `*` expansion.

Exact Rust payloads may follow the current error-rendering architecture.

Requirements:

- duplicate labels must render the label text, never a raw Symbol ID;
- invalid `**`/`***` errors identify the actual source type;
- non-Symbol Map-key errors identify the offending key type;
- dynamic arity errors report the observed and maximum counts;
- no user-controlled pack failure may panic;
- source spans should point to the smallest useful contribution or expansion expression.

Do not introduce user-dispatched formatting, hashing, or equality while constructing these errors.

---

## S7 — Existing static send paths remain untouched

**Decision:** the supplement must not put dynamic-pack branching into ordinary statically-shaped call hot paths beyond the compiler's existing `needs_dynamic_pack` choice.

Static calls continue to use their existing:

- selector constants;
- inline caches;
- checked static arity;
- sacred-selector/inliner behavior;
- ordinary `Invoke`/`SuperSend` paths.

No pack builder is allocated for a statically-shaped call.

No selector comparison or dynamic-cache policy branch is added to every ordinary `Invoke`.

---

## S8 — F.2 test completion is a hard prerequisite, not optional cleanup

**Decision:** F.2 is not considered complete merely because the newly reachable compiler paths compile.

The supplement requires focused behavioral, fiber, GC, and regression tests before F.3 begins.

F.3 rewrites the existing U9 variadic fallback and argument binding behavior. Starting it before F.2's outgoing pack behavior is covered would make failures difficult to attribute to F.2 versus F.3.

---

# 4. Compiler implementation — positional `*`

## 4.1 Remove the blanket rejection

Locate the compiler path that currently rejects positional expansion before pack lowering.

Replace that rejection with dynamic-pack item lowering.

Do not change the AST representation merely to make the compiler path convenient.

## 4.2 Evaluate once and root the source

For each `*expr`:

1. run the E.3 source analysis required by §4.5;
2. evaluate `expr` exactly once;
3. save the resulting `Value` into a fresh compiler-owned hidden source local;
4. probe the Tuple/Unit direct lane;
5. if the probe fails, reload the source local for generic iteration;
6. clean all source/cursor scratch locals before continuing to the next pack item.

No source value may be kept only at a guessed operand-stack depth while arbitrary bytecode can run.

## 4.3 Direct lane

Conceptual bytecode:

```text
compile expr
save source hidden local

GetLocal(source_slot)
GetLocal(builder_slot)
PackTryExpandTuplePositionals
JumpIfFalse(generic)

direct_done:
    clean source scratch
    Jump(done)
```

Use the repository's actual branch stack contract. Do not add a new branch opcode solely for this lowering.

The direct lane performs no user dispatch.

## 4.4 Generic lane

Conceptual bytecode:

```text
generic:
    cursor = source.iterate(None)

loop:
    if cursor == None:
        goto generic_done

    value = source.iteratorValue(cursor)

    value
    GetLocal(builder_slot)
    PackPushPositional

    cursor = source.iterate(cursor)
    goto loop

generic_done:
    clean source/cursor scratch

done:
```

Reuse/factor the smallest safe cursor-loop helper from ordinary `for` lowering.

Do not call the user-level `each` abstraction.

Do not force this through the entire `compile_for` statement implementation: positional expansion has no user loop variable, loop body, `break`, or `continue`.

## 4.5 Boundedness placement

E.3 analysis must happen at compilation of the `*` source expression.

Required outcomes:

```phalcom
target(*(0..))                  // reject if E.3 proves unbounded
target(*((0..).iter.take(3)))   // compile
target(*unknownIterator)        // compile
```

Tuple/Unit finite direct-lane sources must remain legal.

Use the landed boundedness/exhaustibility representation and diagnostics.

Do not create an F.2-only Range/pipeline analyzer.

## 4.6 Failure conversion

If the runtime value is neither Tuple/Unit nor iterable through the ordinary cursor protocol, surface `NonIterableStarOperand`.

The builder may contain earlier contributions at the point of failure. That state is private and the expression aborts; transactional rollback is unnecessary.

---

# 5. Compiler implementation — dynamic subscript writes

## 5.1 Dynamic-path detection

The subscript setter compiler path must use the same semantic dynamic-pack predicate as other pack-aware forms.

A dynamic setter is required when any source index item includes expansion or a computed label.

Do not keep a second inconsistent "static labels only" test inside `SetIndex`.

## 5.2 Scratch layout

Use compiler-owned hidden locals.

Recommended adjacent layout:

```text
receiver_slot
builder_slot
rhs_slot
```

Reserve metadata/placeholders without evaluating user expressions.

## 5.3 Lowering

Required sequence:

```text
1. evaluate receiver; save receiver_slot
2. allocate pack builder; save builder_slot
3. compile all source index-pack items in lexical order
4. reserve compiler-owned #put in builder
5. evaluate RHS exactly once; save rhs_slot
6. fill reserved #put from rhs_slot
7. load receiver_slot
8. load builder_slot
9. InvokePack(SubscriptSet)
10. discard setter return
11. reload rhs_slot
12. collapse scratch locals preserving RHS
```

Duplicate `#put` detection at step 4 must occur before step 5.

## 5.4 Selector and arity invariants

Before dispatch:

```text
labels.last() == #put
```

must hold.

Selector encoding uses only the source index slots.

Dynamic send arity includes the RHS and remains subject to the existing runtime `<= 255` check.

---

# 6. Compiler implementation — dynamic Tuple literals

## 6.1 Detection

A Tuple literal takes the dynamic path when any entry contains:

- positional `*`;
- labeled `**`;
- complete `***`;
- a computed label.

Keep ordinary static Tuple literals on the current `BuildTuple` lowering.

## 6.2 Shared assembly semantics

Tuple literal AST nodes may use a different entry type than call `PackItem`s.

Share semantic lowering helpers where practical, but do not distort the AST into a single generic representation merely for code reuse.

The important invariant is identical pack contribution behavior:

```text
*    -> positional lane
**   -> labeled lane
***  -> both lanes
label -> reserve/fill
```

## 6.3 Finish

After lexical assembly:

```text
GetLocal(builder_slot)
FinishTuplePack
```

Then collapse the builder scratch local while preserving the finished product value.

An empty dynamic contribution produces Unit.

A Tuple product is not subject to the method-send `u8` argument limit.

---

# 7. Runtime error integration

## 7.1 Error sites

Replace generic pack-specific failures in pack handlers with structured variants.

At minimum inspect the handlers for:

```text
PackReserveComputedLabel
PackReserveStaticLabel
PackExpandLabels
PackExpandComplete
InvokePack / SuperSendPack arity validation
generic * iteration failure conversion
```

## 7.2 Duplicate labels

Duplicate detection remains based on interned `Symbol` identity.

It must never invoke:

```text
hash
==
toString
```

or any user method.

The error renderer resolves the Symbol through VM/interner context or stores a stable rendered label string at construction time if that better matches the current architecture.

## 7.3 Invalid `**`

Distinguish:

- invalid `**` operand category;
- Map with a non-Symbol key;
- duplicate label contributed by the expansion.

Report the first offending entry in encounter order.

## 7.4 Invalid `***`

Reject non-Tuple/non-Unit values with `InvalidStarStarStarOperand`.

Do not broaden `***` support while touching the handler.

## 7.5 Dynamic arity

Retain the existing pre-lookup runtime arity check but report it through `SendArityExceedsLimit`.

Required:

```text
found = actual flattened send arity
limit = 255
```

Subscript set includes the implicit `put` value.

Tuple construction is exempt.

---

# 8. Tests required by this supplement

Tests should be added near the repository's existing compiler/VM/integration conventions rather than forcing all cases into one monolithic file.

The following coverage is a completion gate.

## 8.1 Positional `*` direct lane

- `*Unit` contributes zero positionals;
- `*Tuple` contributes positional lane only;
- Tuple labels are ignored by positional `*`;
- singleton positional Tuple contributes one value;
- multi-positional Tuple preserves order;
- the direct lane performs no user iteration dispatch;
- spread operand evaluates exactly once.

## 8.2 Generic positional `*`

Positive:

- List;
- Bytes, if currently Iterable;
- finite Range;
- Iterator;
- user-defined Iterable;
- bounded lazy pipeline;
- unknown compile-time source choosing generic runtime lane.

Ordering:

- source fully exhausts before the next pack item evaluates;
- per-element order is preserved;
- nested dynamic sends during iteration do not corrupt builder state.

Negative:

- known unbounded E.3 source -> compile error;
- non-Iterable runtime source -> structured runtime error.

## 8.3 Fiber behavior

Add a regression where generic `*` exhausts a lazy/user iterator whose callback/body yields a Fiber.

Required result:

```text
yield
resume
expansion continues
send completes
```

It must not fail merely because expansion is happening.

## 8.4 Dynamic method/unqualified/super regression

Existing implemented paths need explicit tests proving that the supplement did not regress them:

- dynamic ordinary method;
- implicit-self dynamic call;
- local/upvalue/global callable dynamic `call`;
- dynamic super exact resolution;
- dynamic super dNU;
- compiler-internal authority.

## 8.5 `**` / `***` and errors

Positive:

- Tuple/Unit `**`;
- Record/Map `**`;
- Tuple/Unit `***`;
- stable label encounter order.

Negative:

- invalid `**` operand -> structured error;
- non-Symbol Map key -> structured error;
- duplicate label -> structured error;
- invalid `***` operand -> structured error;
- computed label not Symbol -> structured error.

## 8.6 Dynamic subscript reads regression

Verify already-landed behavior:

- dynamic positional index count;
- dynamic labels;
- computed labels;
- `**`;
- `***` where valid for the pack;
- dNU receives the concrete bracket selector.

## 8.7 Dynamic subscript writes

Positive:

- positional `*indices`;
- `**labels`;
- computed labels;
- mixed dynamic index contributions;
- RHS evaluated exactly once;
- setter receives canonical flattened arguments;
- selector excludes final builder `#put`;
- setter return is discarded;
- expression result is original RHS.

Ordering/error:

- receiver before index items;
- index items before implicit `put` reservation;
- duplicate `put` fails before RHS;
- 255 total send args accepted;
- 256 total send args -> structured arity error.

## 8.8 Dynamic Tuple literals

- dynamic positional `*`;
- dynamic `**`;
- dynamic `***`;
- computed label;
- mixed static/dynamic entries;
- empty dynamic contribution -> Unit;
- positional/labeled lane behavior matches F.2;
- encounter order preserved;
- large Tuple is not rejected by send-arity limit.

## 8.9 dNU and selector behavior

- dynamic miss forwards the actual concrete selector;
- no pack-builder object leaks to dNU;
- flattened canonical args are forwarded;
- escaped labels remain canonical;
- subscript setter dNU sees `[source slots]=(put)`, not a selector containing `put` inside the bracket slots.

## 8.10 GC stress

Under the repository's stress/aggressive-GC facility, if available:

- already-collected builder values remain live during later spread iteration;
- source/cursor hidden locals remain live across nested sends;
- builder survives Fiber park/resume;
- dynamic subscript RHS remains live through setter dispatch/result restoration;
- `FinishTuplePack` does not lose moved values;
- abandoned builders after runtime failure become collectible.

## 8.11 Static regression

Representative statically-shaped calls must continue to compile without pack bytecodes or builder allocation:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
obj.foo(timeout: t)
obj[a, b]
```

Also verify ordinary static Tuple literals remain on `BuildTuple`.

If benchmark tooling exists, compare ordinary send microcases before/after the supplement. No unexplained static hot-path regression is acceptable.

---

# 9. Recommended implementation phases

## Phase 0 — baseline and invariant check

Before edits:

- confirm the current 11 opcode handlers and their exact stack contracts;
- confirm the hidden-local scratch helper(s) available today;
- identify the current generic positional-`*` rejection;
- identify the landed E.3 API;
- identify ordinary `for` cursor lowering;
- identify dynamic-pack detection used by method/subscript-read/Tuple paths;
- identify current runtime pack-error construction sites;
- identify current integration/fiber/GC test harnesses.

Do not copy obsolete function signatures from the original plan.

## Phase 1 — structured runtime errors

Add the F.2-specific error variants and migrate already-landed handler failures.

Why first:

- subsequent feature tests can assert stable error categories;
- error conversion is easier to review independently from compiler control flow.

Run focused error/handler tests.

## Phase 2 — positional `*` direct lane

Remove the compiler rejection.

Wire:

```text
hidden source local
PackTryExpandTuplePositionals
direct completion branch
```

Initially verify Tuple/Unit reachability and exact-once evaluation.

## Phase 3 — generic iterable `*` + E.3

Add cursor-loop lowering on probe failure.

Reuse E.1 iteration conventions and E.3 analysis.

Add finite/unbounded/non-Iterable/fiber tests immediately.

This phase is not complete until a yielding iterator works through spread.

## Phase 4 — dynamic subscript writes

Replace the static-only `SetIndex` assumption for dynamic packs.

Implement:

```text
receiver scratch
builder scratch
source pack assembly
pre-RHS #put reservation
rhs scratch
InvokePack(SubscriptSet)
RHS result restoration
```

Add ordering, duplicate-`put`, selector, and arity tests.

## Phase 5 — dynamic Tuple literals

Wire dynamic Tuple detection, builder assembly, and `FinishTuplePack`.

Add mixed-lane, empty Unit, computed-label, and large-product tests.

## Phase 6 — integration/regression completion

Fill all remaining F.2 coverage:

- dynamic methods;
- unqualified/callable;
- super;
- subscript reads;
- `**`/`***`;
- dNU;
- compiler-internal authority;
- GC stress;
- static disassembly/performance regression.

Only after this phase should F.2 be marked complete.

---

# 10. Verification commands

Use repository-current commands. At minimum:

```bash
cargo fmt --check
cargo check -p phalcom-core
cargo test -p phalcom-core
cargo test -p phalcom-ast
```

Add focused filters matching the tests introduced by the supplement, for example:

```bash
cargo test -p phalcom-core pack
cargo test -p phalcom-core dynamic
cargo test -p phalcom-core subscript
cargo test -p phalcom-core tuple
cargo test -p phalcom-core fiber
```

Exact test names should follow the repository's actual test organization.

Run any repository-standard clippy/lint target used by CI.

Disassemble representative static and dynamic pack programs and verify:

```text
static send
    -> no pack builder / pack opcodes

dynamic send
    -> expected pack assembly opcodes

static Tuple
    -> BuildTuple

dynamic Tuple
    -> pack assembly + FinishTuplePack
```

---

# 11. Completion checklist

This supplement is complete when all of the following are true.

## Positional `*`

- [ ] Compiler no longer blanket-rejects positional `*`.
- [ ] `*Tuple` reaches `PackTryExpandTuplePositionals`.
- [ ] `*Tuple` projects only the positional lane.
- [ ] `*Unit` contributes nothing and succeeds.
- [ ] Tuple/Unit direct lane performs no user iteration dispatch.
- [ ] Generic non-Tuple/non-Unit `*` lowers through ordinary cursor bytecode.
- [ ] Generic spread operand evaluates exactly once.
- [ ] Generic spread fully exhausts before the next source pack item.
- [ ] E.3 known-unbounded rejection is reused.
- [ ] Bounded/unknown generic sources compile.
- [ ] Non-Iterable positional spread produces a structured runtime failure.
- [ ] Fiber yield during generic `*` works.

## Dynamic subscript writes

- [ ] Dynamic `SetIndex` forms lower through the pack path.
- [ ] Receiver evaluates before index items.
- [ ] Source index items evaluate before RHS.
- [ ] Compiler-owned `#put` is reserved before RHS.
- [ ] Duplicate `put` prevents RHS evaluation.
- [ ] RHS evaluates once.
- [ ] Final `#put` is excluded from bracket selector slots.
- [ ] RHS remains the final flattened setter argument.
- [ ] Setter return is discarded.
- [ ] Assignment expression returns original RHS.
- [ ] Dynamic arity guard includes `put`.

## Dynamic Tuple

- [ ] Tuple literals with `*` lower dynamically.
- [ ] Tuple literals with `**` lower dynamically.
- [ ] Tuple literals with `***` lower dynamically.
- [ ] Tuple literals with computed labels lower dynamically.
- [ ] Dynamic Tuple assembly uses `FinishTuplePack`.
- [ ] Empty dynamic Tuple result is Unit.
- [ ] Dynamic Tuple finalization uses A.3 `finish_tuple`.
- [ ] Tuple construction has no send-arity limit.
- [ ] Static Tuple lowering remains unchanged.

## Errors

- [ ] Computed non-Symbol label has a structured error.
- [ ] Duplicate argument label has a structured error.
- [ ] Invalid `**` operand has a structured error.
- [ ] Non-Symbol Map key during `**` has a structured error.
- [ ] Invalid `***` operand has a structured error.
- [ ] Dynamic send arity overflow has a structured error.
- [ ] Non-Iterable `*` has a structured error.
- [ ] User-controlled pack failures do not panic.
- [ ] Error rendering never exposes raw Symbol IDs.

## Regression and safety

- [ ] Existing builder/GC integration remains intact.
- [ ] All 11 pack opcodes remain the canonical F.2 opcode family.
- [ ] Dynamic ordinary method sends still work.
- [ ] Dynamic unqualified/callable sends still work.
- [ ] Dynamic super sends still work.
- [ ] Dynamic subscript reads still work.
- [ ] Computed labels still work.
- [ ] `**` semantics remain unchanged.
- [ ] `***` semantics remain unchanged.
- [ ] Canonical selector encoding remains unchanged.
- [ ] dNU still receives concrete selector + flat args.
- [ ] Compiler-internal authority remains correct.
- [ ] GC stress regressions pass.
- [ ] Fiber regression passes.
- [ ] Static calls allocate no pack builder.
- [ ] Static `Invoke` hot path is not degraded.
- [ ] Public primitive-floor delta remains 0.

---

# 12. F.3 start gate

**F.3 must not begin until this supplement is complete.**

In particular, the following are hard prerequisites for F.3:

```text
*Tuple / *Unit source lowering is reachable
generic iterable * is implemented
E.3 boundedness is connected
dynamic subscript writes work
dynamic Tuple construction works
pack-specific runtime failures are structured
F.2 integration/fiber/GC/static regressions are covered
```

Reason:

F.3 changes incoming rest binding and replaces U9 variadic fallback semantics. It assumes F.2 already provides one stable, canonical outgoing pack model across static-equivalent and dynamically assembled calls.

Leaving any outgoing pack form compiler-incomplete while changing incoming dispatch would create two moving boundaries at once and make selector, binding, fiber, and dNU failures substantially harder to isolate.

The sequencing is therefore:

```text
F.2 amended core implementation
        ↓
this F.2 completion supplement
        ↓
F.2 declared complete
        ↓
F.3 rest capture and rest-pattern dispatch
```

---

# 13. Non-goals

This supplement does not include:

- F.3 rest capture or rest-pattern dispatch;
- block rest capture;
- native/primitive rest ABI design;
- multiple rest-pattern specificity;
- dynamic-site inline caching;
- widening send arity beyond `u8`;
- new spread protocols;
- Iterable-of-pairs support for `**`;
- Record/Map support for `***`;
- public pack-builder reflection;
- new public primitives;
- redesign of Tuple's ordinary iteration semantics;
- redesign of already-landed F.2 bytecodes.

Those remain separate work.

---

# 14. Final implementation guidance

Treat the supplement as completion work, not architecture exploration.

The intended end state is:

```text
all F.2 source pack forms
        ↓
shared compiler pack assembly
        ↓
already-landed builder/opcodes
        ↓
canonical positional + labeled lanes
        ↓
existing dynamic selector/dispatch or Tuple finalization
```

The most important correctness properties are:

```text
evaluate once
preserve lexical evaluation order
root transient values in hidden locals
never exhaust generic * in native Rust
preserve canonical lane order
report pack failures structurally
keep the static hot path unchanged
```

When these properties and the completion checklist are satisfied, F.2 is a sufficiently stable substrate for F.3.
