# Spec E.3 — Static Boundedness and Eager-Exhaustor Diagnostics

Status: implementation specification. Requires E.1 and E.2 for the first complete end-to-end behavior. The analysis should be implemented as a reusable compiler subsystem because Spec F needs the same facts for `*` expansion.

## 1. Mission

Implement the ratified static distinction:

```text
Bounded
Unbounded
Unknown
```

and reject **only** statically provable unbounded eager exhaustion.

Required examples:

```phalcom
(0..).toList
// compile error
```

```phalcom
(0..).iter
    .map { x => x * 2 }
    .toList
// compile error
```

```phalcom
(0..).iter
    .take(10)
    .toList
// legal
```

```phalcom
someIterator.toList
// legal when boundedness is Unknown
```

No runtime element limit is introduced.

Expected primitive-floor delta: **0**.

## 2. The lattice is compiler metadata, not a public type

Add a compiler-internal enum, conceptually:

```rust
pub(crate) enum Boundedness {
    Bounded,
    Unbounded,
    Unknown,
}
```

Do not add these as:

- user classes;
- Symbols;
- runtime enum variants;
- fields on Range/Iterator;
- reflection-visible annotations.

The analysis exists solely to decide whether a known eager-exhaustor expression is statically invalid.

## 3. Track receiver mode separately

Boundedness alone is insufficient because:

```phalcom
range.map(f)
```

and:

```phalcom
range.iter.map(f)
```

use the same selector name with different eager/lazy semantics.

Use facts conceptually shaped as:

```rust
pub(crate) enum IterationMode {
    Concrete,
    LazyPipeline,
    Unknown,
}

pub(crate) struct SourceFacts {
    pub mode: IterationMode,
    pub boundedness: Boundedness,
}
```

Names may follow compiler style.

`Concrete` here means "ordinary receiver using D's eager transformation semantics", not necessarily "eagerly stored in memory". A Range value is Concrete until `.iter` is sent.

## 4. Soundness posture

The checker is intentionally conservative.

Rule:

```text
reject only when Unbounded is proven
```

Never reject:

```text
Unknown
```

merely because it *might* fail to terminate.

This is a semantic nontermination diagnostic, not a linter heuristic.

False negatives are acceptable in the first implementation when the compiler lacks enough information.

False positives are not.

Do not attempt arbitrary predicate theorem proving.

Do not execute user code at compile time.

## 5. Recommended compiler module

Add a dedicated module rather than scattering string checks through expression code, e.g.:

```text
phalcom-core/src/compiler/boundedness.rs
```

Recommended public-to-compiler helpers:

```rust
infer_source_facts(expr, env) -> SourceFacts

check_exhaustor(
    operation,
    receiver_expr,
    call_range,
    env
) -> Result<(), CompilerError>
```

Spec F should later be able to call:

```rust
require_exhaustible(source_expr, expansion_range, env)
```

or the same lower-level helper.

Do not make F reimplement Range/pipeline inference.

## 6. Expression facts: finite literals

Statically finite literal products/collections are Bounded.

After A/B/D literal work, recognize as available on implementation HEAD:

```text
List literal
Tuple/product literal
Record literal
Map literal
Set literal, once its grammar branch is actually landed
Unit if iteration is ever defined for it
```

Use:

```text
mode = Concrete
boundedness = Bounded
```

only for values that actually participate in iteration.

A syntax node existing does not by itself make a value Iterable.

Do not force Record/Unit into Iterable solely for this analysis.

## 7. Range facts

Under E.2's supported traversal subset:

### 7.1 Two-sided Range

Any Range with:

```text
lower present
upper present
```

is statically Bounded **for the supported forward integer iteration path**.

Its endpoint expressions need not be compile-time constants to establish finiteness. If runtime values are invalid or reversed, iteration fails rather than becoming an unbounded traversal.

Facts:

```text
mode = Concrete
boundedness = Bounded
```

### 7.2 Lower-only Range

```phalcom
lower..
```

is the canonical:

```text
mode = Concrete
boundedness = Unbounded
```

source.

This is the primary proof anchor for compile-time rejection.

### 7.3 Lowerless forms

Until their iteration domain is ratified:

```phalcom
..upper
..=upper
..
```

classify as:

```text
mode = Concrete
boundedness = Unknown
```

They currently fail if traversed under E.2, but Unknown leaves room for the future Range iteration design without baking a false cardinality theorem into the compiler.

Do not classify them Unbounded merely from missing syntax bounds when no starting-domain traversal has been ratified.

## 8. `.iter` facts

For a receiver with known facts:

```text
receiver.iter
```

produces:

```text
mode = LazyPipeline
boundedness = receiver.boundedness
```

For a receiver with completely unknown semantics:

```text
mode = LazyPipeline or Unknown-mode implementation choice
boundedness = Unknown
```

The important requirement is that no false Unbounded fact be invented.

For an already-known LazyPipeline:

```text
iterator.iter
```

preserves both mode and boundedness.

## 9. Lazy stage propagation

Recognize the canonical E.1 lazy stages **only when the receiver is known LazyPipeline**.

This avoids treating an unrelated user method named `map` as a cardinality theorem.

### 9.1 `map`

```text
Bounded   -> Bounded
Unbounded -> Unbounded
Unknown   -> Unknown
```

Mapping preserves source traversal cardinality/exhaustibility.

### 9.2 `filter`

```text
Bounded   -> Bounded
Unbounded -> Unbounded
Unknown   -> Unknown
```

Why Unbounded is sound for an unbounded source:

A filter cannot report stage exhaustion without eventually observing upstream exhaustion. Even if the predicate accepts only finitely many values, the stage must keep scanning the unbounded upstream forever to prove there are no later matches.

Do not mark an unbounded filter Bounded because the predicate *might* become permanently false.

### 9.3 `skip`

```text
Bounded   -> Bounded
Unbounded -> Unbounded
Unknown   -> Unknown
```

Dropping a finite prefix does not change whether upstream exhaustion can occur.

### 9.4 `take`

Any successfully constructed:

```phalcom
pipeline.take(n)
```

is Bounded.

The runtime method validates `n` as a finite non-negative Int. Therefore even when `n` is not a compile-time literal, successful execution establishes a finite upper bound.

Rule:

```text
* -> Bounded
```

Do not require literal-folding the count to recognize this.

### 9.5 `takeWhile`

Rules:

```text
Bounded   -> Bounded
Unbounded -> Unknown
Unknown   -> Unknown
```

A finite source remains finite.

An unbounded source may terminate if the predicate eventually returns false, or may continue forever. Arbitrary user predicate termination is not statically assumed.

### 9.6 `flatMap`

Minimum sound first-cut rules:

```text
Unbounded outer -> Unbounded
Bounded outer   -> Unknown
Unknown outer   -> Unknown
```

For an unbounded outer source, the flattening stage cannot observe successful total exhaustion of the outer source.

For a bounded outer source, a callback may return a finite or unbounded inner iterable, so Unknown is required unless a later type/effect analysis proves more.

Do not inspect the callback's source code in E.3.

## 10. Eager concrete transformations

When a recognized D eager transformation is sent to a known Concrete source, it is itself an eager exhaustor.

Examples:

```text
map
map(indexed:)
filter
flatMap
each
each(indexed:)
count(where:)
fold(initial:,using:)
reduce(using:)
group(by:)
partition(where:)
```

If receiver boundedness is Unbounded:

```text
compile error
```

If Bounded or Unknown:

```text
compile normally
```

If the eager transform completes, its materialized result is a finite concrete value. Therefore expression facts for generic D transforms that return List may become:

```text
mode = Concrete
boundedness = Bounded
```

after the exhaustor check.

This is true even when the input was Unknown: the call may run forever, but if it successfully returns its List result, that result is finite.

## 11. Materializers/full-source terminal exhaustors

Treat these as eager exhaustors when present on implementation HEAD:

```text
toList
toSet
toMap
toMap(merging:)
fold(initial:,using:)
reduce(using:)
count(where:)
each
each(indexed:)
group(by:)
partition(where:)
```

Also include future D.3 sorting operations when they actually land:

```text
sorted
sorted(on:)
sorted(using:)
```

and any other operation whose successful semantics require observing source exhaustion.

Do not add selector names to this catalog speculatively if the operation is not implemented yet; keep the analysis table adjacent to the actual language surface or covered by tests so it does not drift.

## 12. Short-circuit operations are not rejected solely for unboundedness

The ratified distinction is about **eager exhaustors**, not "any operation that iterates".

Do not reject these merely because the receiver is Unbounded:

```text
find(where:)
index(where:)
any(where:)
all(where:)
none(where:)
first, if/when defined for that receiver
includes, when it short-circuits
```

They may terminate after a finite prefix.

They may also fail to terminate dynamically when their stopping condition is never reached.

That possibility is legal.

Likewise, do not reject:

```phalcom
for (x in 0..) { ... }
```

because the body may `break`, `return`, or fail.

## 13. Direct `toList` examples

Reject:

```phalcom
(0..).toList
```

because:

```text
receiver facts = Concrete + Unbounded
operation       = exhaustor
```

Reject:

```phalcom
(0..).iter.toList
```

because:

```text
receiver facts = LazyPipeline + Unbounded
operation       = exhaustor
```

Allow:

```phalcom
(0..10).toList
```

Allow:

```phalcom
(0..).iter.take(10).toList
```

Allow:

```phalcom
(0..).iter.takeWhile { x => externalCondition(x) }.toList
```

because its facts are Unknown, not proven Unbounded.

## 14. Pipeline transform examples

Reject:

```phalcom
(0..).iter
    .map { x => x * 2 }
    .filter { x => x > 10 }
    .toList
```

Propagation:

```text
0..       -> Unbounded Concrete
.iter     -> Unbounded LazyPipeline
.map      -> Unbounded LazyPipeline
.filter   -> Unbounded LazyPipeline
.toList   -> reject
```

Allow:

```phalcom
(0..).iter
    .map { x => x * 2 }
    .take(10)
    .filter { x => x > 10 }
    .toList
```

Propagation:

```text
Unbounded
map    -> Unbounded
take   -> Bounded
filter -> Bounded
toList -> legal
```

## 15. Constant-binding propagation

A direct-expression-only checker is useful but too easy to evade accidentally:

```phalcom
const xs = (0..).iter
xs.toList
```

The minimum recommended environment tracks `SourceFacts` for immutable local `const` bindings whose initializer facts are known.

Conceptually extend compiler local metadata or keep a parallel boundedness environment:

```text
const initializer facts
    -> binding facts
```

Then:

```phalcom
const xs = (0..).iter
xs.toList
```

must still reject.

### 15.1 Mutable bindings

For the first implementation, mutable `let` bindings MAY conservatively become Unknown rather than requiring full control-flow dataflow.

That is sound:

```text
false negative possible
false positive avoided
```

If the existing compiler already has a straightforward sequential assignment metadata mechanism, updating facts on assignment is acceptable.

Do not build SSA solely for E.

### 15.2 Parameters, fields, imports, arbitrary globals

Default to Unknown unless the compiler already has a sound immutable-value fact proving more.

Do not infer boundedness from variable names or declared nominal type alone in this phase.

## 16. Control-flow merges

If implementation chooses to track mutable facts through branches, merge conservatively:

```text
Bounded   + Bounded   -> Bounded
Unbounded + Unbounded -> Unbounded
anything else         -> Unknown
```

Mode merging follows the same conservative posture:

```text
same known mode -> that mode
different/unknown -> Unknown
```

This section is optional for the minimal const-only implementation.

## 17. Dynamic dispatch safety

Phalcom dispatch is selector-based and user-defined methods can share names with collection operations.

Therefore this is unsound:

```text
any `.map` call on an expression inferred Unbounded
    -> automatically apply iterator map theorem
```

Apply lazy propagation only when the receiver is already known to be an E iterator pipeline.

Apply direct eager collection exhaustor rules only when the receiver facts come from a recognized built-in/collection/Range source or other sound compiler fact.

For arbitrary user values:

```text
Unknown
```

wins.

Do not inspect runtime class objects during compilation in a way that assumes reflective method tables cannot later differ unless the language's class-closure rules explicitly guarantee that fact.

## 18. Diagnostic

Add a dedicated compiler error variant, conceptually:

```rust
CompilerError::ProvablyUnboundedExhaustion {
    operation: String,
    range: SourceRange,
}
```

Preferred user message:

```text
cannot exhaust a provably unbounded source with `toList`
```

with a help note when appropriate:

```text
introduce an explicit finite bound, for example `.iter.take(n)`
```

The exact diagnostic code/name should follow current compiler-error conventions.

Point the primary span at the exhaustor selector/call, not the entire source pipeline, unless current diagnostics infrastructure strongly prefers the full expression.

The diagnostic is a compile error, not a runtime `Error` object.

## 19. No hidden runtime fallback

Do not "backstop" the static check with:

- maximum element count;
- timeout;
- maximum memory size;
- automatic truncation;
- implicit `take`;
- special RuntimeError after N iterations.

Unknown sources are explicitly legal and may run forever.

Resource exhaustion remains ordinary runtime/system behavior.

## 20. Callback failure and boundedness

Boundedness describes successful exhaustion semantics, not guaranteed success.

For example:

```phalcom
(0..).iter
    .take(10)
    .map { x => risky(x) }
    .toList
```

is statically Bounded.

It may still fail on element 3.

That does not change boundedness classification.

Likewise, a source that always throws on its first iteration step is not classified Bounded merely because execution cannot reach infinity. The lattice is not exception-path theorem proving.

## 21. Source mutation caveat

The broader language still defers mutation-during-iteration semantics.

A concrete List is structurally finite when classified Bounded, but a callback could potentially mutate the same source while it is being traversed under today's implementation.

E.3 does not attempt to prove termination in the presence of such self-mutation.

The compile-time rule's critical safety property is one-way:

```text
known unbounded full exhaustion -> reject
```

Do not market `Bounded` as a hard runtime termination guarantee under behaviors whose iteration-mutation semantics remain unspecified.

## 22. Spec F integration

Positional expansion through iteration is an eager exhaustor because selector arity cannot be derived until the source is exhausted.

F must call E.3's analyzer before compiling:

```phalcom
foo(*source)
```

When source facts are Unbounded:

```text
compile error
```

When Bounded or Unknown:

```text
compile expansion normally
```

Required F-era examples:

```phalcom
foo(*(0..))
// reject
```

```phalcom
foo(*((0..).iter.take(3)))
// legal
```

```phalcom
foo(*someIterator)
// legal if Unknown
```

Similarly, collection literal spread such as:

```phalcom
[*(0..)]
```

must reuse the same service when F activates that syntax.

E.3 should add unit tests for the inference helper now, while the source-level expansion fixtures remain pending until F.

## 23. Compiler integration points

Likely touch points after re-inspecting HEAD:

```text
phalcom-core/src/compiler/
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib.rs or compiler module registry
phalcom-core/src/compiler/error.rs (or current CompilerError location)
```

If AST expressions are consumed by value during compilation, run boundedness inference before moving the receiver/arguments or pass immutable references into the checker.

Do not clone whole large AST subtrees repeatedly just to classify them if a small pre-send helper can borrow the expression.

Keep selector matching canonical:

- use the same selector encoder/signature representation as normal dispatch where practical;
- do not compare pretty source spelling when labels/arity matter.

## 24. Tests: inference unit lane

Rust unit tests for the analyzer should pin at least:

```text
two-sided Range               -> Concrete/Bounded
lower-only Range              -> Concrete/Unbounded
lowerless Range               -> Concrete/Unknown
finiteRange.iter              -> Lazy/Bounded
unboundedRange.iter           -> Lazy/Unbounded
unbounded.iter.map            -> Lazy/Unbounded
unbounded.iter.filter         -> Lazy/Unbounded
unbounded.iter.skip           -> Lazy/Unbounded
unbounded.iter.take           -> Lazy/Bounded
unbounded.iter.takeWhile      -> Lazy/Unknown
bounded.iter.flatMap          -> Lazy/Unknown
unbounded.iter.flatMap        -> Lazy/Unbounded
unknown expression            -> Unknown
```

## 25. Tests: compile-error lane

Active negative fixtures:

- `boundedness_unbounded_range_tolist.ph`
- `boundedness_unbounded_iter_tolist.ph`
- `boundedness_unbounded_iter_map_tolist.ph`
- `boundedness_unbounded_iter_filter_tolist.ph`
- `boundedness_unbounded_iter_skip_tolist.ph`
- `boundedness_unbounded_direct_map.ph`
- `boundedness_unbounded_fold.ph`
- `boundedness_const_binding_propagates.ph`

Each must fail at compile time before executing any source callback.

## 26. Tests: legal lane

Positive fixtures:

- `boundedness_finite_range_tolist.ph`
- `boundedness_take_bounds_unbounded.ph`
- `boundedness_take_then_filter_stays_bounded.ph`
- `boundedness_takewhile_unknown_allowed.ph`
- `boundedness_unknown_parameter_tolist_allowed.ph`
- `boundedness_unbounded_any_allowed.ph`
- `boundedness_unbounded_find_allowed_with_early_hit.ph`
- `boundedness_unbounded_for_break_allowed.ph`

Do not write a positive test that actually tries to fully exhaust an Unknown source known by the test author to be infinite; legality and termination are separate questions.

## 27. Pending F fixtures

Keep pending/ignored until F parser/compiler work lands:

- `boundedness_spread_unbounded_rejected.ph`
- `boundedness_spread_take_allowed.ph`
- `boundedness_list_spread_unbounded_rejected.ph`

The unit-level analyzer hook must nevertheless be present and tested in E.

## 28. Completion checklist

E.3 is complete when:

- the three-state compiler lattice exists;
- mode and boundedness are not conflated;
- direct recognized lower-only Range is Unbounded;
- `.iter` preserves boundedness and enters lazy mode;
- lazy propagation follows §§9.1–9.6;
- `take` converts an unbounded pipeline to Bounded;
- `takeWhile` over unbounded becomes Unknown;
- known eager exhaustors reject only Unbounded sources;
- short-circuit operations remain legal;
- unknown sources remain legal;
- immutable local const propagation catches obvious aliases;
- a dedicated compile diagnostic exists;
- no runtime boundedness field/cap exists;
- a reusable API is ready for F;
- all E tests and the normal verification gate pass.
