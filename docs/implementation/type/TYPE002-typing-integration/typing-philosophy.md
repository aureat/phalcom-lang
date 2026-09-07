# Phalcom Semantic Type Authority — Current Model vs. Proven-Fact Model

## Executive conclusion

Yes. There has been a conceptual mismatch in how we have been talking about “multiple evidence sources” and “authority.”

The right distinction is:

> **Phalcom can have many sources of semantic evidence, but there is one adjudicating authority: the compiler's semantic checker.**

A developer annotation is not an alternative truth that stands on equal footing with a compiler proof. It is a developer-authored **claim, constraint, or admissibility declaration** that the compiler consumes as evidence. When the compiler can independently establish a stronger fact, the annotation must be checked against that fact.

So if the compiler can establish:

```phalcom
CellNum.new() : CellNum
```

and the programmer writes:

```phalcom
const x: Int = CellNum.new()
```

then, assuming `CellNum` is not assignable to `Int`, the correct interpretation is:

```text
compiler-established value type = CellNum
developer declaration            = Int
relation                         = refuted

result:
    error: annotation is incompatible with the value
    current semantic fact for x remains CellNum
```

The compiler does **not** become uncertain between `Int` and `CellNum`. There is no 50/50 disagreement between two authorities.

However:

```phalcom
const x: Int = somethingWhoseTypeCannotOtherwiseBeDetermined()
```

is different. If the initializer is genuinely underconstrained, the annotation can provide contextual information that permits the checker to solve the expression as `Int`. That is ordinary bidirectional typing.

This is very close to the architecture that Phalcom's Spec 04.5 actually describes. The current implementation, however, does not consistently implement it.

The most important finding is this:

> **The current repository already has the correct architectural distinction between a binding's persistent `declared` type and its path-sensitive `current` type, but `Statement::Let` currently collapses them back together.**

Spec 04.5 explicitly gives:

```phalcom
let x: Number = 1
```

the state:

```text
declared = Number
current  = Int
```

and says that `declared` is the persistent admissibility envelope while `current` is the actual type known on the current flow path.

But the current implementation of `Statement::Let` does this after checking the initializer:

```rust
let effective_fact = if let Some(decl_k) = declared_k {
    ...
    ValueSemanticFact { knowledge: decl_k, ... }
} else {
    val_typed.fact()
};
```

In other words, if there is any annotation at all, it installs the annotation as the binding's semantic knowledge—even when the initializer was independently and precisely typed, and even after a proven mismatch.

That is the core mismatch.

---

## 1. First: disentangling all the things currently called “inference”

There are several genuinely different mechanisms in Phalcom today. Some of our previous discussions blurred them together.

| Concept                  | What it actually means                                              | Authority                               |
| ------------------------ | ------------------------------------------------------------------- | --------------------------------------- |
| Expression synthesis     | Compute a type bottom-up from an expression                         | Compiler-authoritative when sound       |
| Bidirectional checking   | Analyze an expression under an expected type                        | Compiler-authoritative                  |
| Generic inference        | Solve local type variables from argument/result constraints         | Compiler-authoritative when solved      |
| Flow typing              | Track the most precise current type of a binding at a program point | Compiler-authoritative                  |
| Declared type            | Developer-authored annotation/contract                              | Input to checker, not final adjudicator |
| `Proven` evidence        | Checker/solver has soundly established a type                       | Compiler-authoritative                  |
| `ExactSyntax` evidence   | Type follows exactly from syntax/semantic construction              | Compiler-authoritative                  |
| `TrustedNative` evidence | Type comes from trusted native metadata                             | Compiler-authoritative trust root       |
| Advisory inference       | LSP runtime-shape approximation                                     | Tooling evidence only                   |
| `Formal`                 | Presentation of compiler-owned static semantic facts                | Not an inference mechanism              |
| `Unknown`                | Static analysis cannot establish the type                           | Absence of proof                        |
| `Dynamic`                | Deliberate static escape/boundary                                   | Language-policy state, not uncertainty  |

That distinction is fundamental.

### “Formal” is not another kind of inference

`FormalPresentation` in `phalcom-semantic` is just a presentation layer over compiler-owned semantic products. It has states such as:

```text
Known
Dynamic
Unknown
Invalid
Blocked
Cancelled
BudgetExceeded
InternalFailure
Partial
```

`TypePresenter` explicitly formats existing semantic knowledge; it does not perform inference or type-relation checking.

So when the LSP says:

```text
Formal type: CellNum
```

“formal” means approximately:

> This came from the compiler/static semantic tower.

It does **not** mean “formal inference” as opposed to some other compiler inference.

---

# 2. Phalcom currently has two semantic knowledge systems

The repository is still carrying two distinct semantic layers.

## 2.1 Compiler-owned formal/static typing

This lives principally in:

```text
phalcom-semantic/
```

Its core type result is:

```rust
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

A known fact contains:

```rust
pub struct TypeEvidence {
    pub ty: TypeId,
    pub authority: EvidenceAuthority,
    pub provenance: EvidenceSet,
}
```

This is the actual static typing/checking domain.

## 2.2 LSP advisory inference

The older/live editor semantic engine still has:

```rust
InferredValue {
    shape: ValueShape,
    confidence: Confidence,
    provenance: Vec<FactOrigin>,
}
```

with confidence states:

```text
Exact
Flow
Interprocedural
Heuristic
```

and `ValueShape` variants such as:

```text
Instance(CellNum)
ClassObject(...)
List(...)
Tuple(...)
Record(...)
Union(...)
```

Crucially, the source documentation says:

> `ValueShape` is an advisory runtime value shape and deliberately is not a language type.

This engine answers questions like:

> “Given everything the IDE has observed about this code, what runtime shape does this expression probably/definitely produce?”

That can make completion and hover excellent even when the formal checker has insufficient information.

It cannot, by itself, reject a valid program.

The repository's architectural guidance explicitly distinguishes “advisory semantic inference” from the static checker and says that heuristic/advisory knowledge must not masquerade as correctness proof.

---

# 3. What the LSP calls “Formal” versus “Observed”

This explains terminology you may already have seen in hover output.

The current LSP snapshot actually contains both worlds:

```rust
pub type AdvisorySemanticSnapshot = SemanticSnapshot;
pub type StaticSemanticSnapshot = phalcom_semantic::SemanticSnapshot;
```

and the LSP's published snapshot has:

```rust
static_snapshot: Option<Arc<StaticSemanticSnapshot>>
```

alongside its advisory shape facts.

Hover deliberately prefers the formal compiler result.

For a binding, the implementation is conceptually:

```text
if compiler formal type exists:
    Formal type: ...
else if good advisory evidence exists:
    Observed type: ≈ ...
    Confidence: ...
```

Likewise selectors use the formal return state when available and fall back to advisory observed-return information otherwise. Tests explicitly enforce this priority.

So there actually **is already an authority hierarchy between the two semantic towers**:

```text
formal compiler semantics
        >
advisory LSP observations
```

That part is correct.

The confusion exists mostly *inside the formal type system*, particularly around developer declarations versus independently established facts.

---

# 4. The current formal evidence model

The heart of the current system is:

```rust
pub enum EvidenceAuthority {
    Declared,
    Proven,
    ExactSyntax,
    TrustedNative,
    Advisory,
}
```

The definitions are:

```text
Declared
    developer-authored explicit type declaration

Proven
    soundly proved by checker/static solver

ExactSyntax
    exact literal or constructor-expression fact

TrustedNative
    normalized trusted native metadata

Advisory
    LSP-level shape inference
```

And then there is this crucial operation:

```rust
pub fn is_sound_for_rejection(self) -> bool {
    matches!(
        self,
        Declared | Proven | ExactSyntax | TrustedNative
    )
}
```

So:

```text
Declared       -> can participate in hard rejection
Proven         -> can participate in hard rejection
ExactSyntax    -> can participate in hard rejection
TrustedNative  -> can participate in hard rejection
Advisory       -> cannot, by itself
```

This is not inherently wrong.

A developer declaration absolutely should be sufficiently authoritative to produce errors like:

```phalcom
let x: Int = "hello"
```

The problem is more subtle.

`EvidenceAuthority` currently conflates two different axes:

```text
WHERE DID THIS CLAIM COME FROM?
    Annotation
    Syntax
    Native metadata
    Flow
    Solver
    ...

HOW CERTAIN IS THE CHECKER THAT IT IS TRUE?
    Proven
    Assumed
    Advisory
    Unknown
    ...
```

`Declared` describes provenance.

`ExactSyntax` describes provenance/derivation.

`TrustedNative` describes provenance/trust.

But `Proven` describes epistemic status.

They are not really values of the same conceptual dimension.

That becomes significant once two claims disagree.

---

# 5. How assignability currently adjudicates evidence

Current assignability essentially performs:

```text
actual   = TypeKnowledge
expected = TypeKnowledge
```

If either is `Dynamic`, it produces a dynamic boundary.

If either is `Unknown`, it does not invent a contradiction.

If both are known:

```text
check actual <: expected
```

If that relation is proved:

```text
Assignable
```

If it is refuted, it becomes a real compile-time refutation only if **both evidence authorities are sound for rejection**.

Conceptually:

```rust
if subtype(actual, expected) {
    Proven
}
else if subtype_is_refuted
     && actual.authority.is_sound_for_rejection()
     && expected.authority.is_sound_for_rejection()
{
    Refuted
}
else {
    Blocked
}
```

This is an important distinction:

### The current checker does not say “developer always wins.”

A proven/exact `CellNum` versus declared `Int` can already produce a hard mismatch.

So this:

```phalcom
let x: Int = CellNum.new()
```

can already be diagnosed.

That part agrees with your newly articulated semantics.

What it does **not** answer is:

> After discovering this contradiction, which semantic fact is retained?

And that is where today's implementation takes the wrong branch.

---

# 6. What currently happens to an annotated binding

Consider:

```phalcom
let x: Number = 1
```

The current statement checker performs these operations.

### Step 1 — resolve developer annotation

`Number` is lowered through `resolve_type_annotation()`.

A successful annotation becomes:

```rust
TypeKnowledge::Known(TypeEvidence {
    ty: number,
    authority: EvidenceAuthority::Declared,
    ...
})
```

So far, correct.

### Step 2 — use the declaration as contextual expected type

The checker constructs:

```rust
ExpectedType::from_knowledge(&declared_k)
```

and analyzes the initializer under that expectation.

This is bidirectional checking.

Again, this is correct in principle.

### Step 3 — independently analyze the expression

For a literal `1`, expression synthesis establishes `Int` using `ExactSyntax`. Primitive literals are explicitly synthesized this way in `expression.rs`.

So conceptually we now have:

```text
developer declaration: Number [Declared]
initializer fact:      Int    [ExactSyntax]
```

### Step 4 — check assignability

The checker asks:

```text
Int <: Number ?
```

If yes, no error.

This is correct.

### Step 5 — current implementation throws away the precision

And then the current code does:

```rust
let effective_fact = if let Some(decl_k) = declared_k {
    ValueSemanticFact {
        knowledge: decl_k,
        ...
    }
} else {
    val_typed.fact()
};
```

So rather than:

```text
declared = Number
current  = Int
```

it effectively gets:

```text
declared/current ≈ Number
```

This is precisely the behavior you are objecting to.

---

# 7. More seriously: it does the same thing after a mismatch

Now:

```phalcom
let x: Int = CellNum.new()
```

Assume `CellNum` and `Int` are incompatible.

The current sequence is:

```text
annotation     -> Known(Int, Declared)
initializer    -> Known(CellNum, ...)
assignability  -> Refuted
diagnostic     -> BindingInitializerMismatch
```

So the programmer is correctly told that the declaration is inconsistent with the initializer.

But then:

```rust
effective_fact = declared_k
```

still happens.

Only the initializer's denotation is dropped in the mismatch case.

Thus the semantic state after the error is effectively:

```text
diagnostic:
    CellNum cannot be assigned to Int

binding x:
    current type = Int
```

That is internally contradictory from the model you intend.

It should instead be something like:

```text
declared constraint = Int       [invalid against initializer]

current knowledge =
    CellNum [compiler-established]

binding status =
    declaration contradiction
```

The error should not cause the checker to forget what it already knows.

---

# 8. The architecture already has the representation required to fix this

This is why I do not think the semantic architecture needs to be torn down.

`BindingState` already contains:

```rust
pub struct BindingState {
    ...
    pub declared: Option<TypeId>,
    pub current: TypeKnowledge,
    ...
}
```

and its own documentation describes it as:

> “persistent declared constraint vs. current path knowledge.”

This is exactly the model we want.

Spec 04.5 is even more explicit:

```text
declared
    persistent admissibility envelope

current
    type known on this particular flow path
```

For:

```phalcom
let x: Number = 1
```

the specification literally requires:

```text
declared = Number
current  = Int
```

and later, after assigning a `Float`:

```text
declared = Number
current  = Float
```

So the design document and your newly clarified mental model are largely aligned.

The implementation is the thing that drifted.

---

# 9. There is another collapsing bug in `bind_local`

`CheckingContext` also contains a convenience operation:

```rust
pub fn bind_local(
    &mut self,
    name: ...,
    fact: ValueSemanticFact,
    ...
) {
    let declared = fact.knowledge.ty();

    self.bind_local_var(
        name,
        declared,
        fact.knowledge,
        ...
    );
}
```

That is suspicious for exactly the same reason.

Given one fact, it derives both:

```text
declared
current
```

from the same `TypeKnowledge`.

But those concepts do not mean the same thing.

For an unannotated binding:

```phalcom
let x = 1
```

the intended state is:

```text
declared = None
current  = Int
```

not:

```text
declared = Int
current  = Int
```

Spec 04.5 explicitly says the first inferred value of an unannotated mutable binding is flow knowledge, **not an implicit permanent annotation**.

So there is a second implementation seam that needs separation.

---

# 10. This also affects reassignment

There is an important consequence that makes the fix slightly larger than changing `Statement::Let`.

Imagine we correctly change:

```phalcom
let x: Number = 1
```

to:

```text
declared = Number
current  = Int
```

Now later:

```phalcom
x = 2.0
```

assuming:

```text
Float <: Number
```

this should be legal and produce:

```text
declared = Number
current  = Float
```

The assignment code already obtains `info.declared` preferentially as the contextual expected type, which is good.

But later it performs assignability against the binding's current local fact:

```rust
enforce_assignability(
    ...,
    val_k,
    &target_fact.knowledge,
    ...
);
```

and then replaces current flow knowledge with the RHS.

Today this happens to work more often because `target_fact.knowledge` was incorrectly widened to the annotation.

Once the binding initialization bug is corrected, `target_fact.knowledge` might be `Int`, so `Float` could incorrectly be checked against `Int` rather than the declared `Number`.

Therefore the proper rule is:

```text
if binding has declared envelope:
    new RHS must be assignable to declared

otherwise:
    apply unannotated-mutable-binding policy

after successful or analyzable assignment:
    current = precise RHS knowledge
```

That distinction needs to be repaired consistently across declaration, assignment, joins and widening.

---

# 11. Constructors prove your point particularly well

The repository now has an explicit integration test for constructor `Self` specialization:

```phalcom
class Base {
  @constructor
  new() {}

  @class
  ordinary() -> Base {
    Base.new()
  }
}

class Derived is Base {}

...
let b = Base.new()
let d = Derived.new()
let o = Derived.ordinary()
```

The test requires:

```text
Base.new()        -> Base
Derived.new()     -> Derived
Derived.ordinary()-> Base
```

This is exactly the semantics you are describing.

There is no ambiguity about:

```phalcom
Derived.new()
```

The type is not something the developer gets to vote on.

It follows from Phalcom's constructor semantics.

---

# 12. But constructors expose another authority-model problem

Look at how constructor signatures are currently registered.

For an `@constructor`, `register_class_surface()` creates a `SelfType` return:

```rust
let self_type = ctx.store.self_type(...);

TypeKnowledge::known(
    self_type,
    EvidenceAuthority::Declared
)
```

Then dispatch specializes that `SelfType` against the actual receiver, so an inherited `Derived.new()` correctly gets a specialized `Derived` return type.

But notice the authority:

```text
Declared
```

That is conceptually wrong for the model you just described.

Nobody declared:

```phalcom
new() -> Self
```

The compiler imposed it because `@constructor` has language semantics.

The fact:

```text
@constructor return = Self
```

is a **language semantic rule**.

It should therefore be compiler-established evidence.

At minimum it should be `Proven`.

I would actually prefer the evidence model to eventually distinguish something like:

```text
origin = LanguageRule(ConstructorReturnsSelf)
status = Proven
```

rather than overloading `EvidenceAuthority`.

This is a concrete example of why the current enum is not expressive enough.

---

# 13. The correct way to understand `Declared`

I would stop thinking of `Declared` as an “authority level.”

Think of it instead as:

> **A developer-authored static requirement offered to the checker.**

For:

```phalcom
let x: Number = 1
```

two different propositions exist simultaneously:

```text
P1: this binding admits values assignable to Number
    source = developer declaration

P2: at this program point, x contains an Int
    source = compiler analysis
```

Both can be true.

There is no need to choose between them.

That is precisely why:

```text
declared
current
```

must remain separate.

---

# 14. The checker is the authority; evidence is plural

This gives us a much cleaner formulation than “there is no single authority.”

The proper statement is:

> **There is no single source of evidence. There is a single semantic adjudicator.**

Evidence can come from:

```text
developer annotations
literal syntax
constructor semantics
resolved callable signatures
generic constraints
native universe metadata
flow facts
branch predicates
return bodies
module/interface resolution
proof rules
```

The compiler then decides what follows from those facts.

That distinction matters enormously.

The developer is not an authority peer of the compiler.

The developer supplies part of the program.

The semantic checker interprets that program according to the language's rules.

---

# 15. Where developer annotations genuinely should influence inference

Your second point is equally important:

> Sometimes the compiler cannot independently establish a unique type, and the annotation should then participate in reasoning.

Correct.

Phalcom already has bidirectional machinery for exactly that.

`ExpectedType` has:

```rust
None
Proper(TypeId)
Inference(InferenceTerm)
```

and is explicitly described as a contextual expectation propagated downward through expression analysis.

So:

```phalcom
let xs: List<Int> = []
```

could use the declaration to give the empty literal an element type.

Likewise a generic factory:

```phalcom
let xs: List<Int> = List.empty()
```

where:

```phalcom
@class
empty<T>() -> List<T>
```

has no arguments from which `T` can be solved, may legitimately use the expected result `List<Int>` to solve `T = Int`.

That is not “trusting the developer over the compiler.”

The compiler is doing this:

```text
Known constraints:
    result type is List<T>
    declaration requires result <: List<Int>

Solve:
    T = Int
```

The annotation supplies a constraint.

The solver remains the authority.

---

# 16. Generic inference is a separate mechanism

This is where `InferenceSession` enters.

It uses temporary, solver-local terms:

```text
Canonical(TypeId)
Var(InferVarId)
Applied(...)
Union(...)
Tuple(...)
Callable(...)
```

and constraints:

```text
Equivalent(A, B)
Subtype(A, B)
```

with origins such as:

```text
Argument
ExpectedResult
BlockParameter
BlockResult
CollectionElement
GenericWhere
Explicit
```

This is actual formal type-variable inference.

Importantly, inference variables are deliberately session-local and do not become canonical `TypeId`s.

A generic call gathers constraints from arguments and from its expected result. If the solver succeeds, the specialized result is published with `EvidenceAuthority::Proven`; if it is underconstrained, Phalcom reports an `Unknown(UnderconstrainedTypeVariable)` state rather than manufacturing a type.

This mechanism is almost exactly the sort of “annotation as evidence where necessary” that you are describing.

---

# 17. A useful classification: synthesis versus checking versus inference

I would use these three terms much more rigorously going forward.

## Synthesis

```text
Γ ⊢ e ⇒ T
```

Question:

> Without being told what this expression is supposed to be, what can we determine it is?

Examples:

```phalcom
1
    => Int

"foo"
    => String

CellNum.new()
    => CellNum
```

This is where the compiler's independent evidence comes from.

## Checking

```text
Γ ⊢ e ⇐ T
```

Question:

> Does this expression satisfy an expected type T?

Example:

```phalcom
let x: Number = 1
```

The checker synthesizes:

```text
1 ⇒ Int
```

and checks:

```text
Int <: Number
```

The expected type does not magically transform the literal into a `Number`.

## Inference

Question:

> There are unknown type variables in the problem. What substitutions satisfy all constraints?

Example:

```text
foo<T>(T) -> List<T>

foo(1)

constraint:
    Int <: T

solution:
    T = Int

result:
    List<Int>
```

These operations interact, but they are not synonyms.

---

# 18. `Proven` also needs to be understood carefully

Another potential source of terminology confusion:

`Proven` in `EvidenceAuthority` does **not** mean “Phalcom's future theorem prover proved a theorem.”

Currently it means roughly:

> Static type reasoning has soundly established this type result.

For example, a solved generic return is marked `Proven`.

The broader semantic architecture also distinguishes actual proposition/proof facts from language types, correctly warning that:

```text
language type
!= proof proposition
!= runtime shape
```

Longer term, I would probably rename/restructure these concepts to avoid overloading “proof.”

---

# 19. The exact reconciliation model I think Phalcom should use

For an annotated binding:

```phalcom
let x: D = e
```

first independently derive as much as possible about:

```text
e -> A
```

while allowing `D` to participate contextually in places where the typing rules explicitly allow bidirectional information flow.

Then reconcile.

### Case A — compiler establishes `A`, and `A <: D`

Example:

```phalcom
let x: Number = 1
```

Result:

```text
declared = Number
current  = Int

status = valid
```

The annotation acts as an admissibility envelope.

It does not erase precision.

### Case B — compiler establishes `A`, and `A <: D` is refuted

Example:

```phalcom
let x: Int = CellNum.new()
```

Result:

```text
declared = Int
current  = CellNum

annotation consistency = Refuted

diagnostic:
    declared Int contradicts proven initializer CellNum
```

The declaration is wrong.

The compiler does not mutate its knowledge into `Int`.

### Case C — initializer is genuinely underconstrained

Example:

```phalcom
let x: List<Int> = empty()
```

and `empty<T>() -> List<T>` has no argument evidence.

Then:

```text
annotation provides expected-result constraint
solver derives T = Int
initializer becomes List<Int>
```

Result:

```text
declared = List<Int>
current  = List<Int>
```

But provenance should distinguish:

```text
current type derived using declared contextual constraint
```

from:

```text
current type independently established from syntax
```

### Case D — checker cannot determine the initializer at all

Suppose:

```text
initializer = Unknown(UnannotatedDeclaration)
```

and the declaration is valid.

The annotation may be used as the binding's best static assumption:

```text
declared = T
current  = Known(T, Declared)
```

But that does not become `Proven`.

This distinction is useful later if contradictory evidence appears.

### Case E — initializer is unknown because source is broken

For:

```text
Unknown(SyntaxError)
Unknown(UnresolvedName(...))
SuppressedByInvalidCause
```

we should **not** blindly turn the expression into the declared type.

Those states say:

> Analysis could not reliably establish semantics.

The annotation can remain known as a declaration, but the expression itself remains invalid/unknown.

This is one reason the existing `UnknownReason` taxonomy is valuable.

### Case F — initializer is `Dynamic`

That is yet another case.

`Dynamic` is deliberately not `Unknown`.

It represents an explicit static escape/boundary.

A declaration around a dynamic expression may create:

```text
static assumption
runtime obligation
or accepted dynamic boundary
```

depending on the eventual typed-runner/contract policy.

It must not be silently treated as static proof.

---

# 20. Why `let x: Number = 1` matters so much

This example reveals the deeper semantics.

There are actually three questions:

```text
1. What values may legally be stored in x?
2. What value/type is currently known to be in x?
3. What should consumers display as "the type of x"?
```

For:

```phalcom
let x: Number = 1
```

the answers can legitimately be:

```text
admissibility / declared type: Number
current flow type:            Int
```

If you ask:

> Can I later write a Float?

use `declared = Number`.

If you ask:

> Which methods can I safely suggest right here?

you may use precise `current = Int`, while respecting mutation/flow.

If you ask:

> What contract did the programmer specify?

show `Number`.

Those are not contradictory answers.

This is why forcing every semantic question through one “type of x” slot creates problems.

The current `BindingState` design already recognizes this.

---

# 21. Your `CellNum.of()` example reveals another current gap

There is a second issue beyond authority reconciliation.

Suppose:

```phalcom
class CellNum {
  @constructor
  new() {
    ...
  }

  @class
  of() {
    CellNum.new()
  }
}

const x: Int = CellNum.of()
```

The desired formal chain is:

```text
CellNum.new()
    ↓ constructor semantic rule
CellNum
    ↓ tail-expression/return inference
CellNum.of() return = CellNum
    ↓ call resolution
initializer x = CellNum
    ↓ relation against annotation Int
Refuted
```

That is exactly what the formal compiler eventually should do.

But the current implementation is not fully there.

For ordinary source methods without a return annotation, `register_class_surface()` currently gives the callable:

```text
Unknown(UnannotatedDeclaration)
```

as its return type.

The body checker analyzes the tail expression, but it does not currently feed that inferred return back into the declaration surface's callable signature.

More decisively, the canonical `CallableSignature` DB query only publishes a signature if all parameter and return types are complete. An incomplete unannotated return is blocked with:

```text
UnknownType(UnannotatedDeclaration)
```

So today:

```phalcom
CellNum.new()
```

is formally known.

But:

```phalcom
CellNum.of()
```

where `of()` has no explicit return and its body produces `CellNum`, is not yet reliably promoted into a compiler-owned interprocedural formal return type.

That is an implementation gap.

Spec 04.5 actually anticipates the desired behavior: reachable return sites and tail expressions are supposed to contribute to inferred callable return knowledge where no explicit return annotation exists and inference policy allows it.

The older advisory LSP inference engine can already perform interprocedural return-shape propagation, which may be why some existing IDE behavior looks more capable here than the formal checker.

But an advisory `ValueShape::Instance(CellNum)` is not yet equivalent to:

```text
formal return type = CellNum [Proven]
```

That bridge still needs completion.

---

# 22. So how close is the current model to yours?

I would characterize it as follows.

### Already aligned

The current architecture already understands that:

* static facts have explicit epistemic state;
* `Unknown` is different from `Dynamic`;
* advisory shape evidence is different from formal language typing;
* advisory evidence cannot by itself reject code;
* annotations are represented with explicit provenance/authority;
* expression analysis can independently synthesize types;
* expected annotations can participate in bidirectional checking;
* generic expected-result information can solve otherwise unknown type variables;
* hard diagnostics require an actual refutation;
* binding state has separate `declared` and `current` fields;
* constructor `Self` specialization is now implemented and tested;
* formal compiler results dominate advisory LSP observations.

Those are strong foundations.

### Partially aligned

The relation system already treats:

```text
Known(CellNum, compiler evidence)
versus
Known(Int, Declared)
```

as a possible hard contradiction.

So it does not blindly trust the annotation.

But it has no explicit notion of:

```text
which side is the erroneous claim?
which knowledge survives the contradiction?
```

It only decides:

```text
relation proven?
relation refuted?
relation unknown/blocked?
```

### Not aligned

The current binding reconciliation policy then installs the annotation as the effective binding fact.

That is the main conceptual violation.

### Also incomplete

Formal return inference across unannotated source callable bodies is not yet sufficiently published for your `CellNum.of()` chain.

---

# 23. The most important conceptual correction to our previous language

I would retire this statement:

> “There is no single authority; compiler inference and developer inference are evidence that must agree.”

It is misleading.

Replace it with:

> **The semantic checker is authoritative, but it reasons from multiple sources of evidence. Developer annotations are source-level constraints/evidence, not unquestionable truth. Compiler-established facts, language semantic rules, trusted native contracts, and developer declarations must be reconciled according to their semantic roles. A proven contradiction is an error in the conflicting program claim, not uncertainty in the checker.**

And specifically:

> **A developer annotation constrains a program; it does not overwrite what the compiler can prove about a value.**

That is the model I think matches what you actually want.

---

# 24. I would also change our terminology from “developer inference”

I do not think:

```text
developer inference
```

is a useful term.

The developer isn't performing an inference operation when writing:

```phalcom
x: Int
```

They are making a declaration.

Better vocabulary:

```text
declared type
developer claim
declared constraint
declared admissibility envelope
annotation evidence
```

versus:

```text
synthesized type
inferred generic substitution
current flow type
compiler-established fact
proven relation
```

This immediately removes much of the ambiguity.

---

# 25. A better evidence representation

I would seriously consider eventually replacing the one-dimensional `EvidenceAuthority` with two or three orthogonal dimensions.

Conceptually:

```rust
struct TypeEvidence {
    ty: TypeId,
    origin: EvidenceOrigin,
    strength: EvidenceStrength,
    provenance: EvidenceSet,
}
```

Something approximately like:

```rust
enum EvidenceOrigin {
    DeveloperAnnotation,
    LiteralSyntax,
    ConstructorRule,
    CallableSignature,
    GenericSolver,
    FlowAnalysis,
    NativeMetadata,
    AdvisoryShapeBridge,
}

enum EvidenceStrength {
    Proven,
    TrustedPremise,
    ContextualAssumption,
    Advisory,
}
```

Then:

```text
DeveloperAnnotation + TrustedPremise
```

means:

> This is a valid premise/constraint supplied by the program.

While:

```text
ConstructorRule + Proven
```

means:

> Language semantics establish this independently.

And:

```text
GenericSolver + Proven
```

means:

> This follows from solved constraints.

And:

```text
AdvisoryShapeBridge + Advisory
```

means:

> Useful editor evidence, not sufficient for rejection.

This lets provenance and authority stop competing inside one enum.

I would not necessarily make this refactor before fixing the behavioral bugs, but this is the direction I would take.

---

# 26. The checker should reconcile claims, not select a winner by enum ordering

I would **not** solve this by creating something simplistic like:

```text
Proven > ExactSyntax > Native > Declared > Advisory
```

and always picking the highest.

That would introduce new bugs.

For example:

```phalcom
let x: Number = 1
```

should not “pick Int over Number.”

Both facts matter.

They answer different questions.

Instead, reconciliation needs domain-specific roles:

```text
declaration:
    defines admissibility requirement

initializer fact:
    defines current value knowledge

relation:
    verifies current value satisfies admissibility

flow state:
    keeps both
```

Similarly, a method parameter annotation defines an input contract, while call-site evidence describes a particular argument.

The checker should not combine them into one confidence contest.

---

# 27. What should happen on mismatch?

I recommend this exact semantic law:

```text
Given:
    declared constraint D
    independently/currently established value knowledge A

If A <: D is Proven:
    accept
    retain declared = D
    retain current = A

If A <: D is Refuted:
    diagnose declaration/value contradiction
    retain declared = D as authored source metadata
    retain current = A as compiler knowledge
    mark binding/declaration relation invalid

If A <: D is Unknown/Blocked:
    do not diagnose mismatch
    retain both epistemic states
    use D contextually only where typing policy permits

If A is Dynamic:
    create dynamic-boundary semantics according to typing mode
```

Notice that even after a mismatch I would probably retain the authored `declared = D` in semantic metadata.

It is still what the programmer wrote.

But it is not the binding's current truth.

That means an IDE could eventually display something excellent like:

```text
x

Current type:   CellNum
Declared type:  Int  ✕ incompatible

CellNum established because:
  CellNum.of()
  → returns CellNum
  → tail expression calls @constructor CellNum.new()
  → constructors return receiver Self
```

That is much more useful than simply turning `x` into `Int`.

---

# 28. This would also make inference provenance far more meaningful

The current checker already stores:

```text
EvidenceSet
ExplanationArena
ExplanationId
```

and `TypedExpression` carries type knowledge, constraints and provenance.

Expression analysis records derivation steps for literals and method calls, and formal analysis products contain explanation IDs and diagnostic causality.

So the desired diagnostic:

```text
Expected Int
  because x was annotated Int

Found CellNum
  because CellNum.of() returns CellNum
  because its tail expression invokes CellNum.new()
  because @constructor returns Self
```

is completely aligned with the architecture we have been designing.

The critical prerequisite is to stop discarding the winning semantic fact at reconciliation time.

---

# 29. One subtle point: “compiler authority” does not mean annotations are weak

I would not go as far as saying:

> annotations are merely hints.

That would also be wrong.

For:

```phalcom
fun f(x: Int) { ... }
```

`Int` is part of the program's static contract.

The compiler is supposed to enforce it.

Likewise:

```phalcom
let x: Number = ...
```

means assignments incompatible with `Number` are invalid.

So annotation evidence is *normative program input*.

But normative input can itself be inconsistent.

For example:

```phalcom
let x: Int = "hello"
```

contains two incompatible requirements induced by the program.

The compiler's job is to diagnose the inconsistency.

The compiler does not defer to whichever piece of source syntax contains a colon.

---

# 30. The crucial distinction is “contract” versus “fact”

This is perhaps the cleanest way to phrase the whole model.

A declaration often expresses a **contract**:

```text
x must satisfy Int
```

Expression analysis establishes a **fact**:

```text
this initializer has CellNum
```

The checker asks whether:

```text
fact satisfies contract
```

If yes:

```text
valid
```

If no:

```text
program error
```

It should never implement:

```text
contract says Int
therefore initializer is Int
```

except in genuinely contextual/underconstrained positions where the typing rules allow the contract to contribute information.

That qualification is what bidirectional typing is for.

---

# 31. Where I think the current implementation should change

There are several concrete changes implied by this analysis.

### 1. Fix binding initialization reconciliation

`Statement::Let` must stop making:

```rust
effective_fact.knowledge = declared_k
```

unconditionally.

It needs to preserve:

```text
declared
current
```

separately.

### 2. Stop `bind_local()` from deriving `declared` from arbitrary current knowledge

The API should force callers to distinguish:

```rust
declared: Option<TypeId>
current: TypeKnowledge
```

rather than guessing that one implies the other.

### 3. Repair assignment checking simultaneously

Assignments to an annotated mutable binding should be checked against the persistent declared envelope, not its currently refined type.

Then the RHS becomes the new `current`.

### 4. Correct constructor evidence authority

The compiler-generated:

```text
@constructor -> Self
```

return is not `Declared`.

It comes from a language semantic rule.

It should be represented as compiler-established evidence.

### 5. Split evidence origin from epistemic strength

`EvidenceAuthority` currently mixes these concepts.

This will become increasingly painful once proof, contracts, native trust and flow provenance become richer.

### 6. Complete formal callable return inference

An unannotated method whose return/tail type can be established should publish that compiler-owned result so:

```text
constructor
→ factory method return
→ caller expression
→ binding
```

forms one formal proof chain.

This is necessary for the `CellNum.of()` case.

### 7. Preserve exact/current knowledge after diagnostic refutation

A mismatch should invalidate the relationship, not destroy correctly established semantic facts.

### 8. Improve formal presentation to expose declared versus current when useful

Right now formal binding presentation primarily projects `state.current`.

Once the model is corrected, hover/diagnostics should be capable of showing both dimensions when they differ.

---

# 32. How I would define Phalcom's semantic authority model from now on

I propose we treat the following as the canonical mental model:

```text
                         PROGRAM
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
     annotations        syntax/body      native/core
          │                 │                 │
          ▼                 ▼                 ▼
    declared claims    semantic facts     trusted facts
          │                 │                 │
          └────────────┬────┴─────────┬───────┘
                       │              │
                       ▼              ▼
                constraint generation
                       │
                       ▼
               compiler semantic checker
                       │
             ┌─────────┼──────────┐
             ▼         ▼          ▼
           Proven    Refuted    Unknown
             │         │          │
             ▼         ▼          ▼
       precise facts  errors    preserve
       + contracts             uncertainty
```

The annotation is an input to this process.

The checker owns the judgment.

---

# 33. Applied directly to your example

Suppose:

```phalcom
class CellNum {
  @constructor
  new() {
    ...
  }

  @class
  of() {
    CellNum.new()
  }
}

const x: Int = CellNum.of()
```

The target analysis should be:

```text
1. @constructor semantic rule

   CellNum.new() -> Self
   receiver Self = CellNum

   therefore:
   CellNum.new() : CellNum
   status = Proven / exact semantic rule


2. Analyze CellNum.of()

   tail expression:
   CellNum.new() : CellNum

   every reachable return agrees:
   return CellNum

   therefore:
   CellNum.of() : () -> CellNum
   status = Proven


3. Analyze initializer

   CellNum.of() : CellNum

   therefore:
   initializer type = CellNum


4. Resolve developer annotation

   annotation:
   Int

   declared constraint = Int


5. Check relation

   CellNum assignable-to Int?

   Refuted


6. Diagnostic

   error:
   declared type `Int` is incompatible with initializer type `CellNum`

   evidence:
   `CellNum.of()` returns `CellNum`
   because its tail expression calls `CellNum.new()`
   and `@constructor` returns `Self`


7. Binding semantic state

   declared = Int
   current  = CellNum
   status   = invalid declaration/initializer relation
```

That is, I think, the exact semantics you were trying to describe.

And I agree with it.

---

# 34. Contrast: where the annotation should genuinely resolve uncertainty

Now:

```phalcom
@class
empty<T>() -> List<T> {
    []
}

const xs: List<Int> = List.empty()
```

Suppose nothing inside the call independently determines `T`.

Then:

```text
call return = List<?T>

expected result from annotation = List<Int>

constraint:
    List<?T> <: List<Int>

solver:
    ?T = Int

result:
    List.empty() : List<Int>
```

Here the developer annotation was necessary evidence.

But the compiler still made the judgment.

If another argument simultaneously proved:

```text
?T = String
```

then the annotation `List<Int>` would not override that constraint.

The solver would find a conflict.

This is the distinction I think we were missing.

---

# 35. Final assessment

The current Phalcom semantic model is not fundamentally incompatible with your model.

In fact, much of the difficult architecture already supports it:

```text
TypeKnowledge
EvidenceAuthority
ExpectedType
InferenceSession
RelationOutcome
declared/current BindingState
flow-sensitive current knowledge
constructor Self specialization
structured explanations
formal/advisory separation
```

The core conceptual mistake is much narrower but very consequential:

> **The implementation currently sometimes treats a developer declaration not merely as a constraint but as the resulting semantic fact.**

That is visible most clearly in `Statement::Let`, `bind_local()`, and the current assignment relationship.

There is also an independent incompleteness:

> **Compiler-owned unannotated callable-return inference is not yet fully promoted into canonical callable signatures, so a chain such as `@constructor → unannotated factory method → caller` is not yet formally authoritative end-to-end.**

And there is a modeling smell:

> **`EvidenceAuthority` mixes provenance/source with epistemic strength, making “who supplied this claim?” and “how strongly has the compiler established it?” unnecessarily hard to represent.**

So I would revise our semantic principle to this:

> **Phalcom's compiler is the final authority on static semantic judgments. It accepts evidence from many sources—including developer declarations—but does not treat all evidence as equivalent. Developer annotations establish constraints and may provide contextual information when inference is underconstrained. When language semantics, exact syntax, trusted metadata, flow analysis, or the type solver establish a fact that contradicts an annotation, the checker diagnoses the annotation/program constraint as inconsistent and preserves the strongest sound semantic knowledge it has.**

And for bindings specifically:

```text
declared type = what values the program says the binding may admit
current type  = what the compiler currently knows the binding contains
```

Those two facts should never again be collapsed into a single “type of the binding.”

That should become one of the central laws of Phalcom's semantic analyzer.
