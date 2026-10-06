## Verdict

The implementer has the **right high-level correction**, but I would **not approve the plan unchanged**. There are several semantic/architectural errors in Steps 1–4 that could recreate exactly the kind of TypePack/Pack conflation we are trying to remove.

The plan's central distinction is correct: `*args: T` is an element contract, `Pack<*Goals>` is a proper first-class Pack type, and whole-lane typing needs a distinct lane-level form. :chatgpt-content-reference{index="0"}

Your note, however, exposes the terminology problem at the center of this.

# `Goals` is a pack — but **not a `Pack`**

In:

```telos
data GoalQuery<*Goals> {
    goals: Pack<*Goals>
}
```

there are two completely different semantic categories both containing the word “pack.”

Given:

```telos
<*Goals>
```

`Goals` is a **type-level pack term**:

```text
Goals : TypePack
```

For example, after inference:

```text
Goals = TypePack[Int, String]
```

It is a compile-time ordered sequence of types.

It is **not a proper runtime/value type**. You cannot have a runtime value whose type is simply `Goals`.

By contrast:

```telos
Pack<*Goals>
```

is a **proper type**:

```text
Pack<*Goals> : Type
```

It describes actual first-class Pack values.

If:

```text
Goals = TypePack[Int, String]
```

then:

```telos
Pack<*Goals>
```

essentially means the proper structural type:

```telos
Pack<Int, String>
```

and a corresponding value might be:

```telos
Pack(42, "hello")
```

Internally the compiler already models exactly this distinction:

```text
Goals
  ↓
TypeData::TypePack([Int, String])
kind = TypePack

Pack<*Goals>
  ↓
TypeData::Pack(...)
or, while Goals remains symbolic:
TypeData::PackFromPacks {
    positionals: Goals,
    labeled: []
}
kind = Type
```

This distinction is **explicitly normative** in Telos now:

```text
TypePack / LabeledTypePack / CompleteTypePack
    = compile-time/kinded sequences of Types

Pack<...>
    = proper Type of a first-class Telos Pack value
```

So the answer to:

> Isn't `Goals` a `Pack` anyway?

is:

**It is a type pack, but it is not a first-class `Pack` type.** The overloaded word “pack” is unfortunately doing a lot of work here.

A useful analogy is:

```telos
<Ts...>             // hypothetical C++ type parameter pack
Tuple<Ts...>        // concrete value type made from that type pack
```

In Telos:

```telos
<*Goals>            // Goals : TypePack
Pack<*Goals>        // proper Pack value type formed from Goals
```

---

# Then what exactly does the `*` in `Pack<*Goals>` mean?

It means **expand/contribute the positional type lane `Goals` into the structural Pack type**.

Suppose:

```text
Goals = [Int, String, Bool] : TypePack
```

Then:

```telos
Pack<*Goals>
```

means approximately:

```telos
Pack<Int, String, Bool>
```

This is the same structural idea as:

```telos
Tuple<*Goals>
```

or:

```telos
SomeGeneric<*Goals>
```

when the destination accepts a positional generic pack.

This is already how current `annotation.rs` works. A `*X` contribution doesn't merely insist that `X` itself has `TypePack` kind; it resolves `X` and invokes the canonical:

```rust
store.project_positionals(X)
```

Likewise:

```text
**X  → store.project_labeled(X)
***X → store.project_complete(X)
```

That observation exposes the **first concrete error in the implementation plan**.

---

# 1. Step 2's expansion semantics are wrong

The plan says:

> `*P` requires `P` to have kind `TYPE_PACK`
> `**P` requires `P` to have kind `LABELED_TYPE_PACK`
> `***P` requires `P` to have kind `COMPLETE_TYPE_PACK` :chatgpt-content-reference{index="1"}

That is too restrictive and disagrees with existing Telos expansion semantics.

Telos already accepts meaningful forms such as:

```telos
*(Int, String)
```

where:

```telos
(Int, String)
```

is a proper Tuple type, **not** a `TypePack` term.

The existing expansion machinery does:

```text
resolve operand
    ↓
project requested lane
```

Conceptually:

```rust
match mode {
    Positional => store.project_positionals(operand),
    Labeled    => store.project_labeled(operand),
    Complete   => store.project_complete(operand),
}
```

That is what the new rest-lane annotation must reuse.

So:

```telos
*goals: *Goals
```

works because:

```text
Goals : TypePack
project_positionals(Goals) == Goals
```

But this should also potentially work:

```telos
*args: *(Int, String)
```

because:

```text
(Int, String) : Type
project_positionals((Int, String))
    = TypePack[Int, String]
```

### Required correction

Do **not** implement:

```text
*P => operand must already be TypePack
```

Implement:

```text
*P
    resolve P
    project_positionals(P)

**P
    resolve P
    project_labeled(P)

***P
    resolve P
    project_complete(P)
```

This reuses the existing canonical lane-projection authority.

---

# 2. I would not add `TypeAnnotationExpr::Expansion` globally

The plan proposes:

```rust
pub enum TypeAnnotationExpr {
    ...
    Expansion(Box<TypePackExpansionSyntax>),
}
```

and then permits parsing `*P` as a general type annotation, only to reject it later in proper-Type positions. :chatgpt-content-reference{index="2"}

This is unnecessarily broad.

An expansion is not normally a proper type expression. It is a **structural contribution expression**.

That is why the AST already has:

```rust
GenericArgumentSyntax::Expansion(TypePackExpansionSyntax)
TypeTupleItem::Expansion(TypePackExpansionSyntax)
TypeCallableItem::Expansion(TypePackExpansionSyntax)
```

rather than making expansion an unrestricted `TypeAnnotationExpr`.

The new location is another such contextual site:

```text
rest parameter annotation
```

I recommend something conceptually like:

```rust
enum ParameterTypeSyntax {
    Type(TypeAnnotation),

    RestLane(TypePackExpansionSyntax),
}
```

or, more narrowly:

```rust
enum RestParameterAnnotation {
    Element(TypeAnnotation),
    Lane(TypePackExpansionSyntax),
}
```

Then:

```telos
*x: Int
```

parses as:

```text
Element(Int)
```

while:

```telos
*x: *Goals
```

parses as:

```text
Lane(
    mode = Positional,
    operand = Goals
)
```

This prevents nonsensical general syntax such as:

```telos
let x: *Goals
field: **Fields
fn f() -> ***Everything
```

from entering the ordinary type-annotation grammar in the first place.

### Recommendation

**Do not implement Step 1 exactly as written.**

Reuse `TypePackExpansionSyntax`, but introduce it specifically into parameter/rest-annotation syntax.

---

# 3. The proposed `RestContract` idea is right, but its representation needs tightening

The plan proposes:

```rust
pub enum RestContract {
    Element(DeclaredTypeFact),
    Lane(TypeId),
}
```

and adds a separate `body_capture_type`. :chatgpt-content-reference{index="3"}

The conceptual separation is correct:

```text
call-side contract
≠
body-local captured Pack type
```

But I would not make `declared_type`, `rest_contract`, and `body_capture_type` three potentially competing authorities.

A cleaner semantic model is:

```rust
pub enum ParameterContract {
    Fixed(DeclaredTypeFact),

    Rest {
        mode: RestMode,
        contract: RestValueContract,
    },
}

pub enum RestValueContract {
    Unconstrained,
    Element(DeclaredTypeFact),
    Lane(TypeId),
}
```

The body capture type is then a **derived semantic product** of that contract.

For example:

```telos
*args: Int
```

becomes:

```text
Rest {
    mode: Positional,
    contract: Element(Int)
}
```

and derives:

```text
body type =
    Pack {
        positional remainder: Typed(Int)
        labeled remainder: None
    }
```

Whereas:

```telos
*args: *Goals
```

becomes:

```text
Rest {
    mode: Positional,
    contract: Lane(Goals)
}
```

and derives:

```text
body type =
    PackFromPacks(
        Goals,
        LabeledTypePack[]
    )
```

This gives one canonical contract authority rather than layering a new meaning on top of `declared_type`.

---

# 4. `Element(Unknown)` is semantically wrong for unannotated rest

The plan says:

```text
*args
→ RestContract::Element(Unknown)
```

and derives an unconstrained Pack body type. :chatgpt-content-reference{index="4"}

Don't do that.

**Unknown is epistemic. Unconstrained is semantic.**

These mean different things:

```text
Unknown
    "the checker does not know what the contract is"

Unconstrained
    "the declaration intentionally imposes no element-type constraint"
```

For:

```telos
fn forward(*args)
```

the contract is not unknown. We know precisely what the declaration says:

```text
accept any positional remainder
```

So introduce:

```rust
RestValueContract::Unconstrained
```

Do not encode it as:

```rust
Element(DeclaredTypeFact::unknown(...))
```

Otherwise ordinary calls risk acquiring bogus blocked/underconstrained evidence.

---

# 5. The plan's `Subtype(captured_lane, Goals)` relation is incorrect

This is the most important checker error.

The proposed algorithm says:

```rust
session.add_constraint(
    InferenceRelation::Subtype(
        captured_lane_term,
        lane_param_term,
    ),
    ...
);
```

:chatgpt-content-reference{index="5"}

That conflicts with the existing C2 whole-pack inference laws.

`TypePack`, `LabeledTypePack`, and `CompleteTypePack` are **kinded/static domains**. Telos deliberately does **not** define an ordinary subtype lattice over them.

Existing C2 explicitly establishes:

```text
whole TypePack inference = structural unification/equivalence

not TypePack subtyping
```

For:

```telos
GoalQuery.of(1, "two")
```

and:

```telos
<*Goals>
```

we want:

```text
Goals ≡ TypePack[Int, String]
```

not:

```text
TypePack[Int, String] <: Goals
```

So for a fresh pack variable the constraint should be:

```rust
InferenceRelation::Equivalent(
    captured_lane,
    goals_term,
)
```

Existing inference already knows how to solve:

```text
Var(TypePack)
    ≡ TypePack[Int, String]
```

as one whole pack variable.

That capability should be reused.

---

## But concrete lane annotations need one extra distinction

Consider:

```telos
*args: *(Number, String)
```

and:

```telos
f(42, "hello")
```

We should **not** require:

```text
TypePack[Int, String]
==
TypePack[Number, String]
```

because ordinary callable parameter typing permits:

```text
Int <: Number
```

So the correct application algorithm is not simply “always Equivalent” either.

There are two cases:

### Generic whole-lane inference

```telos
<*P>
*args: *P
```

Captured:

```text
[Int, String]
```

gives:

```text
P ≡ TypePack[Int, String]
```

Exact whole-lane inference.

### Declared concrete lane contract

```telos
*args: *(Number, String)
```

requires:

```text
same structural lane shape
argument0 <: Number
argument1 <: String
```

Thus the checker needs a **lane-contract application operation**, not a fake TypePack subtype relation.

Conceptually:

```text
apply_rest_lane_contract(captured, expected):

    if expected is a whole inference variable:
        unify whole lane

    else:
        structurally align the already-determined lane
        require exact cardinality / labels
        apply ordinary argument subtype constraints pointwise
```

No sequence partitioning. No TypePack subtype lattice.

---

# 6. Preserve source evaluation order

The plan says to group arguments into positional/labeled lane terms and then analyze them. :chatgpt-content-reference{index="6"}

The implementation must be very careful here.

For:

```telos
f(
    first(),
    second(),
    label: third(),
)
```

Telos requires:

```text
first()
second()
third()
```

in source order.

Do **not** implement:

```text
collect positional rest
analyze positional group

collect labeled rest
analyze labeled group
```

because complete/split rest can reorder expression checking.

Instead:

```text
1. bind structural roles
2. analyze every source argument exactly once, in source order
3. retain typed results
4. assemble positional/labeled inference lanes afterward
5. add whole-lane constraints
```

This should be an explicit invariant in the plan.

---

# 7. Zero-length capture needs a real inference origin

The plan correctly notices:

```telos
GoalQuery.of()
```

must infer:

```text
Goals = TypePack[]
```

This is an important advantage of whole-lane inference. :chatgpt-content-reference{index="7"}

But there is no argument expression from which to build:

```rust
ConstraintOrigin::Argument {
    argument: ...
}
```

for an empty lane.

Add an origin such as:

```rust
ConstraintOrigin::RestLane {
    call: ExpressionId,
    parameter_index: u16,
}
```

This works for zero or N elements and accurately describes causality.

For non-empty lanes, proof/support should be the join of the contributing argument evidence rather than arbitrarily choosing one argument.

---

# 8. Complete capture construction in the plan is too informal

The plan says, conceptually:

```text
Complete:
    PackFromPacks(P.positionals, P.labeled)
```

:chatgpt-content-reference{index="8"}

There should be no direct structural peeking into `P`.

Use the existing projection authorities:

```rust
let positionals = store.project_positionals(p)?;
let labeled = store.project_labeled(p)?;

let capture =
    store.pack_from_packs(
        positionals,
        labeled,
    )?;
```

That correctly handles:

```text
concrete complete packs
symbolic CompleteTypePack variables
projection-backed terms
canonical normalization
```

---

# 9. Labeled lane generic defaults need care

This future test from the plan:

```telos
**kwargs: **K
```

is not sufficient by itself. :chatgpt-content-reference{index="9"}

Telos deliberately distinguishes:

```text
LabeledTypePack
RecordFields / RecordRow
```

and `**` generic binders have contextual/default-domain history.

For the initial regression test, make the intended domain explicit:

```telos
impl<**K: LabeledTypePack> ...
```

or whatever current generic syntax canonically spells the explicit kind.

Do **not** accidentally make:

```text
RecordRow == LabeledTypePack
```

to get the test through.

---

# 10. The plan understates signature/incremental fallout

Adding a new rest-contract semantic dimension means more than changing:

```text
signature.rs
dispatch.rs
declaration_signature.rs
checker/context.rs
```

The plan currently scopes this too narrowly. :chatgpt-content-reference{index="10"}

At minimum the implementer must audit all consumers of `CallableParameterSemantic` and projected callable parameters for:

```text
fingerprinting / incremental invalidation
metadata publication/import if applicable
signature equivalence
override compatibility
trait witness compatibility
presentation / LSP signature help
reflection parameter metadata
native/source contract comparison
specialization/substitution
```

The important invariant is:

> Changing `*args: Int` to `*args: *P` must change the canonical callable semantic product and invalidate all consumers whose behavior depends on the parameter contract.

And:

> Override/conformance comparison must compare call contracts, not merely the derived body-local Pack type.

---

# Correct interpretation of your example

With all that clarified, this source is semantically coherent:

```telos
data GoalQuery<*Goals> {
    goals: Pack<*Goals>
}

impl<*Goals> GoalQuery<*Goals> {
    @class
    of(*goals: *Goals) -> Self {
        GoalQuery {
            goals: goals,
        }
    }
}
```

It means:

```text
<*Goals>
    Goals is a compile-time TypePack

*goals
    capture all residual positional runtime arguments

: *Goals
    their complete positional type lane is governed by Goals

inside the body:
    goals is one first-class Pack value

type(goals)
    = Pack<*Goals>
```

For:

```telos
GoalQuery.of()
```

infer:

```text
Goals = TypePack[]
type(goals) = Pack<>
result = GoalQuery<*TypePack[]>
```

Conceptually.

For:

```telos
GoalQuery.of(1, "two")
```

infer:

```text
Goals = TypePack[Int, String]

type(goals)
    = Pack<*Goals>
    = Pack<Int, String>
```

and the actual local value is conceptually:

```telos
Pack(1, "two")
```

So this field:

```telos
goals: Pack<*Goals>
```

does **not** redundantly wrap a Pack in another Pack.

It converts:

```text
compile-time sequence of types
        Goals
```

into:

```text
proper type of a runtime Pack value
        Pack<*Goals>
```

That distinction is exactly why both constructs exist.

---

## One subtle design question: could we write `*goals: Goals`?

Technically, **we could design it that way**.

Since:

```text
Goals : TypePack
```

and `*goals` already tells us this is a positional-rest parameter, the compiler could interpret:

```telos
*goals: Goals
```

as a lane contract based on the annotation's kind.

But I would **not** recommend it now.

It would mean parameter annotations have a special contextual rule where:

```telos
Goals
```

means “consume the contents of this TypePack,” whereas everywhere else Telos writes that operation explicitly:

```telos
Tuple<*Goals>
Pack<*Goals>
Foo<*Goals>
```

The explicit form:

```telos
*goals: *Goals
```

is therefore more regular:

```text
first *
    runtime rest capture

second *
    type-level positional lane expansion
```

They happen to look symmetrical because they describe the same lane at two different semantic levels.

---

# Final ruling on the plan

**Approve the architectural objective, reject the plan as currently executable.**

The following amendments are required before implementation:

1. **Keep the `TypePack` vs proper `Pack` distinction explicit.**
2. **Prefer a rest-specific lane-annotation AST carrier** rather than globally adding `TypeAnnotationExpr::Expansion`.
3. **Reuse `project_positionals` / `project_labeled` / `project_complete`**; do not require expansion operands themselves to already have the corresponding pack kind.
4. Replace `RestContract::Element(Unknown)` with **`Unconstrained`**.
5. Make one canonical **parameter contract** authority; don't allow `declared_type`, `rest_contract`, and `body_capture_type` to independently disagree.
6. **Do not use `Subtype(TypePack, TypePack)`** for whole-lane inference. Fresh pack variables use whole-pack equivalence/unification; concrete lane contracts align structurally and apply ordinary value-type subtype checks elementwise.
7. Analyze arguments **exactly once and in source order**, then assemble lane terms.
8. Add a **rest-lane constraint origin** that works for the empty capture.
9. Build complete capture types through canonical lane projection + `pack_from_packs`.
10. Audit fingerprinting, metadata, reflection, conformance/override compatibility, presentation, and substitution consumers of callable parameters.
11. Keep `LabeledTypePack` distinct from Record row domains.

With those corrections, the design becomes both semantically clean and very useful: `*args: T` gives homogeneous variadic contracts, while `*args: *P` gives exact heterogeneous variadic shape capture without changing what a runtime `Pack` actually is.

# Whole-Lane Rest Parameter Semantics — Comprehensive Verification Requirements

## 1. Verification objective

The implementation is not complete merely when the motivating `GoalQuery` regression passes.

The verification suite must establish that Telos has two distinct, compositional rest-parameter annotation semantics:

```telos
*args: T
```

means an **element contract** over the selected positional rest lane:

```text
∀ captured positional argument a:
    type(a) <: T
```

while:

```telos
*args: *P
```

means a **whole positional lane contract**.

The corresponding labeled and complete forms are:

```telos
**args: T
**args: **P

***args: T
***args: ***P
```

No context-sensitive implicit conversion between those forms is permitted.

The test suite must prove all of the following independently:

```text
binder lane selection
annotation semantic category
kind correctness
argument acceptance
whole-lane inference
body-local capture typing
generic specialization
substitution/materialization
diagnostic correctness
call evaluation order
signature identity
override/conformance behavior
metadata/incremental persistence
runtime lowering compatibility
absence of accidental TypePack subtyping
```

A single integration test cannot establish these properties.

---

# 2. Core language invariants

The implementation must be considered incorrect if any test demonstrates a violation of these invariants.

## REST-01 — Binder and annotation operators are independent

In:

```telos
*args: *P
```

the two `*` operators have distinct grammatical roles:

```text
first *
    positional runtime rest capture

second *
    positional type-lane contribution/contract
```

The implementation must never infer the second operation merely from the first.

Therefore:

```telos
*args: P
```

and:

```telos
*args: *P
```

must remain observably distinct.

---

## REST-02 — Plain annotations are element contracts

For any proper type `T`:

```telos
fn f(*args: T)
```

means:

```text
∀ a ∈ captured positionals:
    type(a) <: T
```

It does not mean:

```text
type(args) <: T
```

and does not mean:

```text
captured_type_lane <: T
```

---

## REST-03 — Explicit lane syntax is required for lane semantics

Only:

```telos
*P
**P
***P
```

select positional, labeled, and complete type lanes respectively.

A bare pack-domain variable is never automatically expanded.

Therefore, given:

```telos
<*P>
```

this must be rejected:

```telos
fn f(*args: P)
```

because:

```text
P : TypePack
```

but the ordinary element-contract annotation position requires a proper `Type`.

The compiler must not reinterpret it as:

```telos
fn f(*args: *P)
```

---

## REST-04 — TypePack and Pack remain different semantic categories

The suite must preserve the normative distinction:

```text
TypePack
    compile-time/kinded sequence of Types

Pack<...>
    proper Type of a first-class Pack value
```

For:

```telos
<*P>
```

the following must hold:

```text
P         : TypePack
Pack<*P>  : Type
```

No test or implementation shortcut may collapse them.

---

## REST-05 — `Pack<*P>` retains ordinary type-formation semantics

For a positional type pack `P`:

```telos
Pack<*P>
```

forms a proper first-class Pack type from the positional lane `P`.

It must **not** acquire a different meaning merely because it appears after a rest binder.

Consequently:

```telos
fn f<*P>(*args: Pack<*P>)
```

means:

```text
every captured positional argument must itself conform to Pack<*P>
```

It does **not** mean:

```text
the outer captured lane has shape P
```

This case is mandatory because it catches contextual reinterpretation bugs immediately.

---

## REST-06 — Whole-lane inference is equality/unification, not TypePack subtyping

For a fresh pack parameter:

```telos
fn f<*P>(*args: *P)
```

a call:

```telos
f(1, "x")
```

must infer:

```text
P ≡ TypePack[Int, String]
```

The implementation must not invent a relation such as:

```text
TypePack[Int, String] <: P
```

Telos does not acquire an ordinary subtype lattice for `TypePack`, `LabeledTypePack`, or `CompleteTypePack` as a side effect of this feature.

---

## REST-07 — Concrete lane contracts retain ordinary element conformance

For:

```telos
fn f(*args: *(Number, String))
```

a call with:

```text
Int, String
```

is valid when:

```text
Int <: Number
```

The contract is:

```text
same positional topology
same arity
corresponding argument types conform to corresponding expected types
```

It is not literal canonical-TypeId equality.

---

## REST-08 — Rest capture locals are proper Pack values

For:

```telos
fn f(*args: Int)
```

the local identifier:

```telos
args
```

must not have type `Int`.

It must have the appropriate proper Pack type representing an open positional sequence constrained to `Int`, conceptually:

```telos
Pack<*: Int>
```

For:

```telos
fn f<*P>(*args: *P)
```

the local identifier must have the proper Pack type:

```telos
Pack<*P>
```

or its canonical semantic equivalent.

---

# 3. Parser and AST tests

These tests must exist at the parser/AST layer independently of semantic checking.

## 3.1 Positive parsing

Verify all three lane forms:

```telos
fn f<*P>(*args: *P) {}
fn f<**P: LabeledTypePack>(**args: **P) {}
fn f<***P: CompleteTypePack>(***args: ***P) {}
```

The AST must preserve separately:

```text
parameter.rest_mode
lane-annotation mode
lane-annotation operand
operator source range
operand source range
whole annotation source range
```

Expected pairs:

| Source | Rest mode | Lane annotation mode |
|---|---|---|
| `*args: *P` | Positional | Positional |
| `**args: **P` | Labeled | Labeled |
| `***args: ***P` | Complete | Complete |

---

## 3.2 Element annotation remains unchanged

Verify:

```telos
*args: Int
**args: String
***args: Object
```

continue to parse as ordinary element contracts rather than lane annotations.

No existing proper-type annotation grammar should regress.

---

## 3.3 No implicit expansion AST

Given:

```telos
<*P>
fn f(*args: P) {}
```

the AST must represent:

```text
rest = Positional
annotation = ordinary reference P
```

It must **not** rewrite the annotation into a positional expansion.

The semantic layer is responsible for subsequently rejecting the kind mismatch.

---

## 3.4 Mismatched lane syntax parses faithfully

The parser should preserve, rather than silently normalize:

```telos
*args: **P
**args: *P
***args: *P
```

Semantic validation can then report the precise lane mismatch.

Do not make the parser “fix” the spelling.

---

# 4. Lane projection tests

Whole-lane annotation operands must use the same canonical projection rules as existing type expansion.

## 4.1 Direct TypePack variable

```telos
<*P>
*args: *P
```

must project `P` positionally and obtain `P` itself.

---

## 4.2 Structural Tuple operand

A concrete source form such as:

```telos
*args: *(Int, String)
```

must derive:

```text
TypePack[Int, String]
```

through the existing positional projection authority.

The implementation must not require the operand expression itself to have kind `TypePack`.

---

## 4.3 Pack proper-type operand

Given a proper Pack type whose positional lane is known:

```telos
*args: *SomePackType
```

the same canonical positional projection machinery must apply.

---

## 4.4 Invalid projection

An operand from which the requested lane cannot legally be projected must produce the canonical projection diagnostic.

It must not:

```text
panic
return Dynamic
silently become Unknown
fabricate an empty lane
```

---

# 5. Element-contract call tests

These are essential regressions protecting the original semantics.

## 5.1 Homogeneous positive

```telos
fn ints(*xs: Int) {}

ints()
ints(1)
ints(1, 2, 3)
```

All must succeed.

---

## 5.2 Homogeneous negative

```telos
ints(1, "two", 3)
```

must fail specifically because:

```text
String !<: Int
```

The diagnostic must point at the offending argument, not at the entire captured Pack.

---

## 5.3 Subtyping

Given:

```telos
Cat <: Animal
```

then:

```telos
fn animals(*xs: Animal) {}
animals(Cat(), Cat())
```

must succeed.

This confirms `<:` semantics rather than exact equality.

---

## 5.4 Empty element rest

```telos
fn animals(*xs: Animal) {}
animals()
```

must succeed without inventing an argument or requiring inference evidence from an absent argument.

---

## 5.5 Body-local type

Inside:

```telos
fn ints(*xs: Int) {
    ...
}
```

semantic inspection must show:

```text
xs : proper Pack type
```

with positional remainder `Typed(Int)`.

It must not show:

```text
xs : Int
```

---

# 6. Explicit-vs-implicit lane distinction tests

This section is mandatory.

## 6.1 Bare pack parameter is rejected as an element contract

```telos
fn f<*P>(*xs: P) {}
```

must fail because:

```text
P : TypePack
```

while an ordinary element annotation requires:

```text
Type
```

Expected diagnostic category:

```text
KindExpectedType
```

or the repository's exact canonical equivalent.

The diagnostic should explain that explicit positional lane syntax is available:

```telos
*xs: *P
```

if diagnostic architecture permits actionable notes.

---

## 6.2 Explicit lane succeeds

```telos
fn f<*P>(*xs: *P) {}
```

must succeed.

---

## 6.3 Proper-Type generic still means element contract

This must remain valid:

```telos
fn f<T: Type>(*xs: T) {}
```

and:

```telos
f<Int>(1, 2, 3)
```

must continue to constrain every item against `Int`.

This prevents an overly broad rule such as:

```text
generic name after rest binder => lane contract
```

---

# 7. Whole positional lane inference tests

## 7.1 Empty capture

```telos
fn collect<*P>(*xs: *P) -> Box<*P> { ... }

collect()
```

or the corresponding repository-valid fixture must infer:

```text
P = TypePack[]
```

An empty lane is positive type information.

`P` must not remain unsolved.

---

## 7.2 Singleton capture

```telos
collect(1)
```

must infer:

```text
P = TypePack[Int]
```

---

## 7.3 Heterogeneous capture

```telos
collect(1, "x", true)
```

must infer exactly:

```text
P = TypePack[Int, String, Bool]
```

with order preserved.

---

## 7.4 Repeated types

```telos
collect(1, 2, "x", "y")
```

must preserve multiplicity:

```text
P = TypePack[Int, Int, String, String]
```

No deduplication.

---

## 7.5 Fixed prefix plus lane rest

```telos
fn f<*Tail>(_ head: Int, *tail: *Tail) {}
```

Verify:

```telos
f(1)
```

gives:

```text
Tail = TypePack[]
```

and:

```telos
f(1, "x", true)
```

gives:

```text
Tail = TypePack[String, Bool]
```

The fixed argument must never leak into the inferred tail.

---

# 8. GoalQuery acceptance regression

The motivating API must have a dedicated integration fixture:

```telos
data GoalQuery<*Goals> {
    goals: Pack<*Goals>,
}

impl<*Goals> GoalQuery<*Goals> {
    @class
    of(*goals: *Goals) -> Self {
        GoalQuery {
            goals: goals,
        }
    }
}
```

## Required calls

```telos
GoalQuery.of()
GoalQuery.of(goal1)
GoalQuery.of(goal1, goal2)
GoalQuery.of(goal1, goal2, goal3)
```

For heterogeneous goal types `G1`, `G2`, `G3`, verify exact specialization:

```text
GoalQuery<TypePack[]>
GoalQuery<TypePack[G1]>
GoalQuery<TypePack[G1, G2]>
GoalQuery<TypePack[G1, G2, G3]>
```

using canonical repository representation rather than pretty-printed spelling alone.

---

## 8.1 Field/body agreement

Inside `of`:

```telos
goals
```

must have the same proper Pack type required by the field:

```telos
Pack<*Goals>
```

The constructor assignment:

```telos
goals: goals
```

must succeed without:

```text
special-case coercion
synthetic reinterpretation
Dynamic fallback
generic-inference conflict
```

---

# 9. Nested Pack regression

This is one of the strongest tests against the old mistaken interpretation.

Define:

```telos
fn consume<*P>(*packs: Pack<*P>) {}
```

Supply actual Pack values:

```telos
let a: Pack<Int, String> = ...
let b: Pack<Int, String> = ...

consume(a, b)
```

This should succeed according to the normal element contract.

But:

```telos
consume(1, "x")
```

must fail because each supplied argument is expected to itself be:

```telos
Pack<*P>
```

The compiler must not interpret the outer captured argument lane as `P`.

---

# 10. Concrete positional shape contracts

Verify:

```telos
fn f(*xs: *(Number, String)) {}
```

## Positive

```telos
f(1, "x")
```

when:

```text
Int <: Number
```

must succeed.

## Wrong arity

```telos
f()
f(1)
f(1, "x", true)
```

must fail.

## Wrong corresponding type

```telos
f("not-number", "x")
f(1, false)
```

must fail at the corresponding argument.

## Order sensitivity

If:

```text
Int <: Number
```

then:

```telos
f("x", 1)
```

must not be accepted merely because the same two types are present in different order.

---

# 11. Labeled-lane tests

Use explicit domain annotations where necessary to avoid `RecordFields` ambiguity.

Conceptually:

```telos
fn f<**K: LabeledTypePack>(**kwargs: **K) {}
```

## Required cases

### Empty

```telos
f()
```

infers empty labeled pack.

### Singleton

```telos
f(name: "x")
```

infers the exact label and value type.

### Multiple

```telos
f(name: "x", age: 42)
```

preserves:

```text
label order
label identity
value types
```

### Wrong label

A concrete contract requiring `name:` must not accept `title:` merely because the value type is equal.

### Duplicate labels

If duplicate labels are source-illegal, verify normal call diagnostics remain authoritative.

Do not silently use:

```text
first wins
last wins
unordered map semantics
```

---

# 12. Complete-lane tests

For:

```telos
fn f<***P: CompleteTypePack>(***args: ***P) {}
```

verify simultaneous preservation of:

```text
positional lane
labeled lane
positional order
labeled order
labels
element types
```

Example:

```telos
f(1, "x", debug: true, count: 2)
```

must infer a complete pack whose lanes are exactly:

```text
positionals = TypePack[Int, String]

labeled =
    [debug: Bool, count: Int]
```

No lane may disappear.

---

# 13. Lane mismatch diagnostics

Explicitly test:

```telos
fn f<**K: LabeledTypePack>(*xs: **K) {}
fn g<*P>(**xs: *P) {}
fn h<*P>(***xs: *P) {}
```

These must produce deterministic lane mismatch diagnostics.

The implementation must not:

```text
project a different lane automatically
reinterpret binder mode
discard labels
coerce positional to complete
```

---

# 14. Inference engine tests

These should operate below source integration level.

## 14.1 Whole pack variable unification

Verify:

```text
Var(TypePack)
≡
TypePack[Int, String]
```

solves exactly to the concrete pack.

---

## 14.2 Empty pack unification

Verify:

```text
Var(TypePack)
≡
TypePack[]
```

solves rather than remaining underconstrained.

---

## 14.3 Labeled pack unification

Verify exact labels and corresponding terms.

---

## 14.4 Complete pack unification

Verify both lanes are solved together without losing either lane.

---

## 14.5 No TypePack subtype lattice

Add explicit negative/invariant tests proving that ordinary proper-type subtype APIs do not suddenly report:

```text
TypePack[Int] <: TypePack[Number]
```

merely because:

```text
Int <: Number
```

Whole-lane rest matching must use its dedicated structural contract machinery.

---

## 14.6 No pack sequence partitioning

Retain all C2 no-decomposition invariants.

For example:

```text
Concat(P, TypePack[String])
≡
TypePack[Int, Bool, String]
```

must not infer:

```text
P = TypePack[Int, Bool]
```

unless an independently canonical lane correspondence already establishes that split.

This feature must not introduce general sequence solving.

---

# 15. Body capture type derivation tests

Test body typing independently of call inference.

## Element contract

```telos
*xs: Int
```

derives a proper Pack with:

```text
positional_remainder = Typed(Int)
labeled_remainder = None
```

## Positional lane contract

```telos
*xs: *P
```

derives canonically:

```text
PackFromPacks(
    P,
    EmptyLabeledTypePack
)
```

## Labeled lane

```telos
**xs: **K
```

derives:

```text
PackFromPacks(
    EmptyTypePack,
    K
)
```

## Complete lane

```telos
***xs: ***C
```

must derive its proper Pack type through canonical:

```text
project_positionals(C)
project_labeled(C)
pack_from_packs(...)
```

rather than manually inspecting `CompleteTypePack` representation.

---

# 16. Unannotated rest tests

For:

```telos
fn f(*xs) {}
```

verify the semantic contract is explicitly:

```text
Unconstrained
```

rather than epistemically:

```text
Unknown
```

Required checks:

```telos
f()
f(1)
f("x", true, 3)
```

must not become blocked or underconstrained because the rest element type is unspecified.

The body local should have a proper unconstrained positional Pack type.

---

# 17. Source evaluation-order tests

Lane grouping must not reorder expression checking or execution.

Use side-effecting calls:

```telos
fn mark(_ n: Int) -> Int { ... }

target(
    mark(1),
    mark(2),
    label: mark(3),
    other: mark(4),
)
```

For complete or mixed rest capture, assert runtime observation remains:

```text
1, 2, 3, 4
```

in source order.

The checker should also analyze each argument expression exactly once.

This test catches implementations that:

```text
analyze positional arguments first
then labeled arguments
```

or reconstruct typed expressions during lane formation.

---

# 18. Constraint-causality tests

## Empty lane

A zero-element lane inference must still have a deterministic semantic origin attached to its constraint.

No fabricated argument `ExpressionId` should be required.

## Non-empty lane

The whole-lane proof/support state should correctly join evidence from every contributing argument.

## Failure diagnostics

A failure involving one concrete element should identify the element when possible.

A failure involving lane topology should identify the rest parameter/lane contract.

---

# 19. Generic specialization tests

After inference:

```telos
GoalQuery.of(1, "x")
```

must specialize all uses of `Goals` consistently.

Test specialization in:

```text
return type
data fields
nested generic applications
associated projections
method-local proper Pack type
subsequent calls using the specialized result
```

No unresolved `InferVarId` may leak into a published semantic product.

---

# 20. Substitution and materialization tests

Given:

```text
P -> TypePack[Int, String]
```

verify substitution through:

```text
PackFromPacks
TupleFromPacks
CallableFromPacks
TypePackConcat
type member projections
applied generic types
Self specialization
```

For the motivating example:

```text
PackFromPacks(P, [])
```

must normalize after substitution to the canonical proper:

```text
Pack[Int, String]
```

rather than remain unnecessarily symbolic.

---

# 21. Signature identity tests

These parameter forms must produce semantically different callable signatures:

```telos
*xs
*xs: Int
*xs: *P
```

Likewise:

```telos
**xs: T
**xs: **K
```

and:

```telos
***xs: T
***xs: ***C
```

Fingerprint/signature equality must observe this distinction.

A semantic cache must never consider:

```telos
*xs: Int
```

and:

```telos
*xs: *P
```

equivalent solely because they have the same runtime `RestMode`.

---

# 22. Override and conformance tests

Test methods participating in:

```text
class override compatibility
trait requirements
impl witness matching
native/source signature comparison
```

The contract comparison must reason about:

```text
rest lane
element-vs-lane contract category
actual contract
```

not merely derived body-local Pack type.

At minimum add negative coverage where:

```telos
base:    *args: Animal
derived: *args: *P
```

or the reverse would otherwise appear structurally similar after body-capture derivation.

The implementation must not silently treat different callable acceptance contracts as equal.

---

# 23. Metadata round-trip tests

If callable parameter contracts are exported across semantic/module metadata, add encode/decode tests covering:

```text
unconstrained rest
element rest
positional lane rest
labeled lane rest
complete lane rest
```

Round-trip must preserve:

```text
rest mode
contract category
lane identity/kind
canonical type/static identities
```

It must not preserve source spelling as semantic identity.

---

# 24. Incremental recomputation tests

Required edit scenarios:

## Element → lane

Edit:

```telos
*args: Int
```

to:

```telos
*args: *P
```

Dependent call sites must recompute.

## Lane operand edit

Edit:

```telos
*args: *P
```

to:

```telos
*args: *Q
```

where semantic contracts differ.

Relevant signatures and calls must invalidate.

## Body-only edit

Change method body while preserving the parameter contract.

Call-site signature products should not unnecessarily change.

## Source movement

Whitespace/range-only movement must not change canonical semantic identity where repository policy requires range-free products.

Every incremental result must equal a cold analysis of the same source.

---

# 25. Presentation and LSP tests

Where semantic signatures are surfaced, verify the language service can distinguish and display:

```telos
*args: Int
*args: *P
```

without collapsing both to the derived local type:

```telos
Pack<...>
```

Hover/signature information should preserve the declaration contract.

Body-local hover on `args`, however, should expose the proper Pack type.

That gives two intentionally different views:

```text
parameter declaration:
    *args: *P

body local:
    args: Pack<*P>
```

---

# 26. Runtime/lowering tests

This feature primarily changes semantic typing, not runtime rest capture representation.

Verify existing runtime behavior remains:

```text
*     captures positional Pack
**    captures labeled Pack
***   captures complete Pack
```

The compiler must not add a second runtime representation depending on whether the source used:

```telos
*args: Int
```

or:

```telos
*args: *P
```

Both are runtime positional rest capture; only their static contracts differ.

---

# 27. Backward-compatibility tests

Existing valid programs using:

```telos
*args: Int
**args: String
***args: Object
```

must retain their previous argument-acceptance semantics.

Existing pack expansion behavior in:

```telos
Tuple<*P>
Pack<*P>
Callable<...>
generic applications
```

must remain unchanged.

The implementation must not globally alter expansion semantics merely to support the new parameter context.

---

# 28. Diagnostic requirements

Required diagnostic categories should cover at least:

### Element annotation wrong kind

```telos
<*P>
fn f(*xs: P)
```

Diagnostic:

```text
rest element contract requires a proper Type;
`P` has domain TypePack
```

Prefer an optional note:

```text
use `*P` to describe the positional rest lane
```

### Rest/lane mode mismatch

```telos
*xs: **K
```

Diagnostic must mention:

```text
positional rest binder
labeled lane annotation
```

### Invalid lane projection

```telos
*xs: *NonProjectable
```

Use the canonical projection failure.

### Concrete lane arity mismatch

Report expected and supplied positional cardinality.

### Concrete labeled-shape mismatch

Identify missing/unexpected label where deterministically known.

Diagnostics must not claim that the captured proper `Pack` failed to subtype a `TypePack`.

---

# 29. Property-based / generative tests

The implementation would benefit strongly from bounded generative testing.

Generate finite positional type sequences:

```text
[]
[A]
[A, B]
[A, B, C]
...
```

and assert:

```text
capture → inference → substitution → Pack proper type
```

round-trips to the same ordered sequence.

Generate labeled lanes with unique labels and assert:

```text
labels preserved
order preserved
payload types preserved
```

Generate complete packs and assert projecting:

```text
positionals
labeled
```

and reassembling the complete lane is canonical.

These tests should be bounded; no arbitrary pack decomposition/search is required.

---

# 30. Stress and complexity tests

Whole-lane grouping must remain linear in argument count.

Test large but reasonable argument packs to catch:

```text
quadratic repeated concatenation
repeated lane reconstruction
repeated duplicate-label scans
solver split-point searches
```

Expected architecture:

```text
analyze N arguments      O(N)
assemble lane terms      O(N)
whole-lane unification   O(N)
```

No exponential or combinatorial partitioning is acceptable.

---

# 31. Required focused test groups

The implementation should not be accepted until at least these logical groups are green:

```text
parser/rest_lane_annotations
foundations/pack_types
foundations/inference
capabilities/generics
call/rest_element_contracts
call/rest_lane_contracts
call/rest_evaluation_order
signatures/rest_contract_identity
incremental/rest_contract_changes
metadata/rest_contract_roundtrip
```

Use the repository's actual existing module organization rather than creating duplicate test harnesses merely to match these names.

---

# 32. Required acceptance matrix

| Capability | Positive | Negative | Empty | Heterogeneous | Body type | Incremental |
|---|---:|---:|---:|---:|---:|---:|
| `*args: T` | ✓ | ✓ | ✓ | N/A/homogeneous | ✓ | ✓ |
| `*args: *P` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `**args: T` | ✓ | ✓ | ✓ | N/A/homogeneous | ✓ | ✓ |
| `**args: **P` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `***args: T` | ✓ | ✓ | ✓ | N/A/homogeneous | ✓ | ✓ |
| `***args: ***P` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| nested `Pack<*P>` element contract | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| concrete lane shape | ✓ | ✓ | ✓ | ✓ | ✓ | — |

No row in this matrix should be certified solely through another row's implementation.

---

# 33. Mandatory anti-regressions

These deserve dedicated named tests because they guard the conceptual boundaries most likely to regress.

```text
rest_plain_annotation_remains_element_contract

rest_typepack_name_is_not_implicitly_expanded

rest_typepack_element_contract_rejects_non_type_kind

rest_explicit_positional_lane_infers_whole_typepack

rest_empty_lane_infers_empty_typepack

rest_capture_local_is_proper_pack_not_element_type

rest_lane_capture_local_is_pack_from_inferred_lane

rest_pack_of_expansion_remains_proper_pack_element_contract

rest_concrete_lane_uses_pointwise_value_conformance

rest_whole_pack_inference_does_not_create_typepack_subtyping

rest_lane_inference_does_not_partition_pack_concat

rest_argument_analysis_preserves_source_order

rest_complete_capture_preserves_both_lanes

rest_labeled_lane_preserves_labels_and_order

rest_contract_change_invalidates_dependent_calls
```

---

# 34. Final acceptance condition

The implementation may be marked complete only if the tests establish all three layers independently:

```text
CALL CONTRACT
    what incoming arguments are accepted

TYPE-LEVEL LANE
    what compile-time sequence of argument types is inferred/constrained

BODY CAPTURE
    what proper first-class Pack type the local rest binding has
```

For the canonical example:

```telos
data GoalQuery<*Goals> {
    goals: Pack<*Goals>
}

impl<*Goals> GoalQuery<*Goals> {
    @class
    of(*goals: *Goals) -> Self {
        GoalQuery { goals: goals }
    }
}
```

the suite must prove:

```text
Goals
    : TypePack

*Goals
    : explicit positional lane contract

goals
    : first-class runtime Pack value

type(goals)
    : Pack<*Goals>
```

and for:

```telos
GoalQuery.of(1, "x")
```

it must establish:

```text
Goals
    ≡ TypePack[Int, String]

type(goals)
    ≡ Pack<Int, String>

result
    ≡ GoalQuery instantiated with that exact positional type pack
```

without contextual reinterpretation, Dynamic fallback, pack sequence partitioning, invented TypePack subtyping, or unresolved inference variables escaping into published semantics.