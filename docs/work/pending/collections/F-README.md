# Phalcom Collections — Spec F: Argument Packs, Expansion, and Rest Capture

Status: implementation-spec bundle. This is the final implementation unit in the A–F collections roadmap. It implements the ratified two-lane argument-pack model, call/Tuple expansion, runtime selector derivation, and rest capture.

Repository baseline re-checked for this plan: `aureat/phalcom-lang` main at commit `5c73279157891ca8e2fc045db5e7dff683c0be5b`. Every implementation session MUST re-inspect actual HEAD before editing. Paths and old unit names below are anchors, not frozen line numbers.

## Dependencies

Implement after:

- A.1/A.3 — Symbol/product syntax, explicit Tuple AST, Unit normalization, Tuple/Record lane access/finalizers.
- B.1/B.2 — insertion-ordered Map and Record/Map ordered-association access.
- C.1 — assignment evaluation order and hidden compiler-local technique.
- E.1 — ordinary `iterate(_)` / `iteratorValue(_)` pipeline semantics.
- E.3 — reusable `Bounded / Unbounded / Unknown` source analysis and `require_exhaustible`-style hook.

D is not otherwise load-bearing for F except where collection/Iterable methods have moved.

## Authoritative semantic model

A call-shaped pack is:

```text
ArgumentPack = <PositionalLane, LabeledLane>

PositionalLane = ordered finite sequence of Value
LabeledLane    = ordered finite sequence of (Symbol, Value)
```

Labels are unique by Symbol identity.

The semantic pipeline is:

```text
evaluate operands
    -> incrementally construct pack
    -> derive concrete selector from the completed pack
    -> method lookup
    -> rest/fixed binding
    -> execute
```

A Tuple is the canonical first-class value capable of preserving both lanes. The transient call builder introduced by F is an implementation object, not a second user-visible product type.

## Source phase rule

Call and Tuple construction share the same two source phases.

Before the labeled phase starts, legal items are:

```text
ordinary positional
*expr
***expr
```

The labeled phase starts at the first:

```text
explicit label
computed label
**expr
labeled trailing closure
```

After the labeled phase has started, these are illegal:

```text
ordinary positional
*expr
***expr
anonymous positional trailing closure
```

`***expr` may itself contribute labels without starting the *source* labeled phase. Source phase is determined by syntax, not by the runtime contents of an expansion operand.

Multiple `***` operands are legal during the positional phase.

## Lane meanings

The operator family is uniform:

```text
*     positional lane
**    labeled lane
***   complete pack
```

Built-in outgoing expansion behavior:

```text
Tuple             *   -> Tuple positional lane
Tuple             **  -> Tuple labeled lane
Tuple             *** -> both lanes

Unit              *   -> empty contribution
Unit              **  -> empty contribution
Unit              *** -> empty contribution

List              *   -> iteration
Bytes             *   -> iteration
Range             *   -> iteration
Progression       *   -> iteration once Progression lands
Set               *   -> iteration
other Iterable    *   -> iteration

Record            **  -> ordered fields
Map               **  -> ordered associations
```

`***` has one canonical built-in source: Tuple, with Unit accepted as its zero-arity normalization.

These are invalid:

```text
***record
***map
***list
*record
*map
**list
**range
```

Full user-definable expansion-capability protocols are deferred.

## Stable order is semantic

`**` preserves source association order.

This is not cosmetic because Phalcom selector identity includes the **ordered labeled argument sequence**.

For:

```phalcom
target(**m)
```

if `m` encounters:

```text
#x, #y
```

the derived selector differs from a map that encounters:

```text
#y, #x
```

even when the two maps are equal as mappings.

## Phase order

### F.1 — Pack Syntax, AST, and Selector-Slot Encoding

Replace the current call argument representation (`expr + Option<String> label`) with explicit pack-item AST variants shared by:

- ordinary method sends;
- subscript sends;
- Tuple construction.

Parse:

```text
*expr
**expr
***expr
[labelExpr]: value
```

and the ratified source-phase constraints.

Carry multiple labeled trailing closures:

```phalcom
request.send(url, timeout: 10)
    onError: { ... }
    onSuccess: { ... }
```

Also fix a repository-level selector encoding collision exposed by A/F: current U9 encodes a variadic selector as `name(*)`, but `#*` is now a legitimate Symbol call label. Quoted/computed Symbols can also contain selector delimiters. Introduce one shared reversible slot-component escaping rule that preserves ordinary safe selector spellings while escaping ambiguous labels.

Expected primitive-floor delta: **0**.

Artifact: `F.1-pack-syntax-ast-and-selector-encoding.md`.

### F.2 — Outgoing Pack Assembly and Dynamic Sends

Add one internal transient `ArgumentPackBuilderObject` plus pack-assembly bytecodes.

Static calls with only ordinary positionals/static labels MUST keep the existing `Invoke` fast path.

Only calls whose selector/arity must be determined at runtime use the pack path:

```text
*
**
***
computed labels
```

Compile contributions incrementally so:

- each operand executes once;
- source evaluation order is preserved;
- invalid expansion fails before later operands execute;
- duplicate labels fail when encountered;
- computed label validity/duplication is checked before its value expression executes.

Iterable `*` MUST be lowered as an ordinary VM cursor loop, not a Rust/native re-entrant exhaustion helper. E.3 rejects only provably unbounded expansions.

Add dynamic pack-send bytecodes that derive the concrete selector and enter the normal VM dispatch loop without calling the synchronous `VM::send_dynamic` helper from a native frame.

Tuple construction reuses the same builder and finishes through A's Tuple finalizer; empty result normalizes to Unit.

Expected architectural delta:

- one internal heap object variant;
- a small pack bytecode family;
- zero public native primitive bindings.

Artifact: `F.2-outgoing-pack-assembly-and-dynamic-send.md`.

### F.3 — Incoming Rest Capture and Generalized Rest Dispatch

Replace U9's positional-only:

```text
ParameterDef.is_rest: bool
SignatureKind::Variadic
List rest binding
```

with lane-aware rest metadata:

```text
*rest      positional residual lane
**rest     labeled residual lane
***rest    complete residual pack
```

Capture results are Tuple products:

```text
*rest     -> positional-only Tuple
**rest    -> labeled-only Tuple
***rest   -> complete Tuple
```

Zero-arity captures normalize to Unit.

Fixed parameters consume lane prefixes; rest captures the residual suffixes. No label reordering occurs during binding.

Exact selector lookup always runs before rest-pattern lookup.

For the first F implementation, preserve a deliberate simplification compatible with today's U9 architecture:

> At most one rest-capable method per base selector family per class.

A subclass may replace that family's rest fallback for itself; if its pattern does not accept a call, lookup may continue to the superclass fallback. This avoids inventing a multi-rest-overload specificity lattice during the core-language build. Lifting this restriction is a separate selector-dispatch decision, not part of collections.

Expected public primitive-floor delta: **0**.

Artifact: `F.3-rest-capture-and-dispatch.md`.

## Static fast-path invariant

Do **not** route all calls through argument-pack allocation.

These remain exactly the current compiler/VM path:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
obj.foo(a, timeout: t)
obj[0]
```

when every label is statically known and no expansion occurs.

They still compile to a statically interned selector plus `Invoke`/`SuperSend`.

This protects the existing inline-cache and superinstruction hot path.

## Dynamic-pack examples

```phalcom
const args = (1, 2)
const opts = #{timeout: 10}

target(*args, **opts)
```

conceptually assembles:

```text
positionals = [1, 2]
labels      = [(#timeout, 10)]
selector    = target(_,_,timeout)
```

This:

```phalcom
const first = (1, x: 2)
const second = (3, y: 4)

target(***first, 5, ***second, z: 6)
```

is legal because both `***` operands occur in the positional source phase.

The completed pack is:

```text
positionals = [1, 5, 3]
labels      = [(#x, 2), (#y, 4), (#z, 6)]
```

The labels contributed by `***first` do not prevent the later source positional `5`.

## Duplicate-label rule

All contributions share one duplicate check:

```phalcom
target(x: 1, x: 2)
target(**first, **second)
target(***pack, x: 2)
target([labelExpr]: value)
```

No contribution overrides another.

For statically provable duplicates, reject during parse/compile.

For dynamic duplicates, fail at runtime at the contribution point.

Duplicate identity is Symbol identity after canonical interning, not source spelling.

## Computed labels

```phalcom
target([labelExpr]: valueExpr)
```

evaluation order is:

```text
evaluate labelExpr
require Symbol
reserve/check label in builder
evaluate valueExpr
fill reserved entry
```

If `labelExpr` is not a Symbol or duplicates an earlier label, `valueExpr` does not execute.

Use the same rule for Tuple construction.

## Eager expansion and boundedness

Positional iterable expansion is an eager exhaustor because selector arity cannot be known before source exhaustion.

Therefore:

```phalcom
target(*(0..))
```

is a compile error through E.3.

This is legal:

```phalcom
target(*((0..).iter.take(3)))
```

Unknown boundedness remains legal:

```phalcom
target(*someIterator)
```

and may run forever dynamically. F adds no hidden element limit.

## Deliberate exclusions

F does not implement or decide:

- typing/generic `ArgumentPackType`;
- type-level `*` / `**` / `***` unpacking;
- reflective pack-projection values;
- arbitrary user-defined expansion protocols beyond Iterable positional expansion;
- `***Record` compatibility from older drafts;
- source-order override behavior for labels;
- unordered `**` sources;
- a specificity lattice for multiple rest-capable overloads in one class family;
- keyword/default parameters;
- destructuring rest;
- Progression semantics themselves;
- Set/ImmutableSet completion;
- public first-class mutable argument-pack objects;
- a public builder API;
- widening the VM's current maximum concrete send arity beyond 255;
- rest-aware typing/reflection beyond minimal existing `arity` compatibility.

## Cross-unit verification

At the end of F, verify at minimum:

1. ordinary static sends still emit ordinary `Invoke`;
2. selector cache behavior for static sends is unchanged;
3. `#*` as a literal call label does not collide with a rest selector;
4. weird quoted/computed Symbol labels round-trip through encode/decode;
5. `*` over an Iterable executes through VM cursor sends and permits fiber yield;
6. `*(0..)` is rejected by E.3;
7. `*((0..).iter.take(3))` terminates;
8. `**Map` preserves insertion order in selector identity;
9. multiple `***Tuple` expansions work;
10. `***Record` fails;
11. duplicate dynamic labels fail before later source operands;
12. zero `*`, `**`, and `***` rest captures bind Unit;
13. non-empty rest captures bind Tuple, never legacy List;
14. exact methods beat the family's rest fallback;
15. a static subscript setter still returns the original RHS per C.1;
16. dynamic subscript packing reserves implicit `put` before evaluating RHS;
17. no new public primitive binding was added.
