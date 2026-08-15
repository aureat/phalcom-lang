# Judgments, Contexts, and Metatheory

## Purpose

Use this reference when a typing proposal sounds plausible but its inputs, outputs, or safety claim are vague. A type checker is an executable procedure for semantic judgments. Formalization is useful because it forces hidden assumptions—scope, expected type, effects, subtyping, recovery, dynamic boundaries—to become explicit.

The goal is not to prove all of Phalcom sound at once. Phalcom is dynamic-first and includes reflection/native boundaries. The goal is to state **scoped guarantees** precisely enough that implementation and tests can preserve them.

## 1. Judgments are typed interfaces to semantic questions

A judgment has inputs to the left of `⊢` and a proposition/result to the right.

```text
Γ ⊢ e : T
```

means: in environment `Γ`, expression `e` has type `T` according to a specified declarative typing relation.

Bidirectional checking splits the relation:

```text
Γ ⊢ e ⇒ T       synthesis: compute a type from e
Γ ⊢ e ⇐ T       checking: verify e against expected T
```

Type formation and kinding are separate:

```text
Δ ⊢ T : Type
Δ ⊢ F : Type -> Type
```

Relations are judgments too:

```text
Δ ⊢ A <: B
Δ ⊢ A ≡ B
Δ ⊢ A ~ B
```

A checker implementation may return diagnostics and recovery values, but the normative judgment should not be defined in terms of its error-recovery sentinel.

## 2. What belongs in the context?

A mathematical paper often writes one `Γ`. A production Phalcom implementation should use explicit semantic components rather than a single string map.

A conceptual context can be decomposed as:

```text
Δ    type binders / kinds / generic restrictions
Γ    value binding types keyed by BindingId
Σ    declarations and canonical type descriptors keyed by semantic IDs
R    current receiver / Self / class-side context
E    expected control/effect context
M    module/import visibility context
P    optional path propositions/refinements
```

Then a detailed judgment may be written:

```text
Δ; Σ; Γ; R; P ⊢ e ⇒ T
```

Do not force all of these into notation unless the rule needs them. The implementation, however, must know which inputs affect the result so dependency tracking and caching are correct.

### Semantic identity, not spelling

If source contains:

```text
class Outer<T> {
  method<T>(x: T) -> T
}
```

the two `T`s are different binders. Context lookup should use identities like:

```text
TypeParamId(owner=Outer, index=0)
TypeParamId(owner=Outer.method, index=0)
```

not the string `"T"` after resolution.

This is a foundational invariant for substitution, shadowing, diagnostics, and incremental rename stability.

## 3. Formation before typing

Typing an expression against a malformed type expression is undefined unless recovery semantics are specified.

A formation judgment can be:

```text
Δ ⊢ T type
```

Representative rules:

### Nominal descriptor

```text
C is a recognized type descriptor
───────────────────────────────
Δ ⊢ C type
```

### Type parameter

```text
α : Type ∈ Δ
────────────
Δ ⊢ α type
```

### Generic application

```text
Δ ⊢ F : Type^n -> Type
Δ ⊢ T1 type ... Δ ⊢ Tn type
restrictions(F, T1...Tn) satisfied
──────────────────────────────────
Δ ⊢ F<T1,...,Tn> type
```

Formation must state arity, kind, restriction checking, recursive legality, and trust/metadata constraints. An `arity mismatch` is a formation failure, not a mysterious subtype failure.

## 4. Declarative versus algorithmic typing

A declarative system says what is valid. An algorithmic system says how the implementation decides validity.

Example declarative subsumption:

```text
Γ ⊢ e : S     S <: T
────────────────────
Γ ⊢ e : T
```

If applied everywhere naively, this rule gives many possible derivation paths and poor diagnostics. A bidirectional algorithm typically localizes subsumption:

```text
Γ ⊢ e ⇒ S     S <: T
────────────────────
Γ ⊢ e ⇐ T
```

The implementation can then synthesize once and compare once.

The key proof obligations are:

- **algorithmic soundness**: if the algorithm accepts, the declarative relation holds;
- **algorithmic completeness**: if the declarative relation holds, the algorithm accepts, within the supported fragment;
- **termination**: the algorithm returns for every finite input accepted by its domain.

Production checkers sometimes intentionally sacrifice completeness for predictability. If so, state it. Never describe a heuristic as the definition of the language relation unless the language design chooses that.

## 5. Basic bidirectional rules

### Variable synthesis

```text
x : T ∈ Γ
─────────
Γ ⊢ x ⇒ T
```

Implementation: resolve `x` to `BindingId` first; query the type fact for that identity.

### Literal synthesis

```text
──────────
Γ ⊢ 42 ⇒ Int
```

The literal may later be accepted by a broader expected type through subtyping/acceptance.

### Checking through subsumption

```text
Γ ⊢ e ⇒ S     S <: T
────────────────────
Γ ⊢ e ⇐ T
```

### Block checking

For a block with parameters `x1...xn` and expected callable type `(A1...An) -> R`:

```text
Γ, x1:A1,...,xn:An ⊢ body ⇐ R
────────────────────────────────
Γ ⊢ |x1,...,xn| { body } ⇐ (A1,...,An) -> R
```

Phalcom block semantics may include non-local return, mutation, throw, or yield; those belong in a separate computation/effect component if the type system tracks them. See `effects-control-and-computation-types.md`.

### Why blocks often check better than synthesize

An unannotated block parameter has no local source type. An expected callable type can flow inward. Requiring every block to synthesize parameter types independently creates an annotation wall or guesses.

This is a practical reason for bidirectionality.

## 6. Substitution lemma: the implementation meaning

A classic substitution property is:

```text
Γ, x:S ⊢ e : T
Γ ⊢ v : S
────────────────
Γ ⊢ e[v/x] : T
```

It says that replacing a value variable with a value of the declared type preserves typing under suitable side conditions.

For generic types, a corresponding property is:

```text
Δ, α:κ ⊢ T type
Δ ⊢ U : κ
────────────────
Δ ⊢ T[U/α] type
```

Implementation consequences:

- substitution must target binder identity, not text;
- traversal must respect nested binders;
- a substituted result must be re-normalized/canonicalized according to semantic rules;
- recursive descriptors need memoized traversal;
- source annotation syntax and normalized semantic views may differ: reflection may need both.

If substitution can create an ill-formed type, either the original formation rules were insufficient or the substitution preconditions are missing.

## 7. Weakening, exchange, and contraction

These structural properties explain which context changes should be irrelevant.

### Weakening

If `Γ ⊢ e : T`, then adding an unused ordinary binding should not invalidate the judgment:

```text
Γ, y:U ⊢ e : T
```

This supports incremental reasoning: an unrelated local declaration should not force a typing result to change.

### Exchange

Independent assumptions may be reordered mathematically. Runtime evaluation order is a different matter. Do not infer that source expressions can be reordered because typing contexts are exchangeable.

### Contraction

Ordinary Phalcom value bindings are reusable; the type system is not currently linear by default. A future affine/linear capability system would require revisiting this assumption.

## 8. Preservation and progress, scoped for dynamic Phalcom

Classic type safety is often decomposed as:

### Preservation

```text
Γ ⊢ e : T
⟨e, σ⟩ → ⟨e', σ'⟩
────────────────────
Γ ⊢ e' : T'     where T' <: T or T' is equivalent under the chosen theorem
```

### Progress

A closed well-typed expression is either a value or can take a semantic step.

Phalcom has dynamic sends, reflection, native boundaries, thrown errors, and future gradual escapes. A global theorem like "well-typed programs never get stuck" may be false or too broad.

Instead define a safety envelope, for example:

> In checker-accepted code whose relevant path contains no `Dynamic` escape, no unchecked native/FFI contract, no reflection that bypasses trusted metadata, and no concurrent mutation invalidating the checked member surface, a send accepted because receiver type guarantees selector `s` will not fail solely because `s` is absent.

This theorem is useful because it identifies exactly what must be proven by:

- member conformance/subtyping;
- dynamic-boundary checks;
- cache invalidation;
- native trust specifications.

The exact Phalcom theorem must follow ratified semantics.

## 9. Soundness, completeness, precision, usefulness

These words are not synonyms.

- **Sound typing rule:** accepted programs satisfy the stated safety property.
- **Complete decision procedure:** every case valid under the declarative relation is accepted.
- **Precise analysis:** produces a narrow approximation; precision is relative to an abstract domain/order.
- **Useful checker:** gives actionable results on real code.

A checker may be sound but incomplete. An LSP hint may be useful but intentionally not sound enough for program rejection. Do not use one result as another without an explicit bridge.

## 10. Error recovery is outside normative algebra

A compiler may introduce:

```text
Type::Error(ErrorId)
```

so later operations do not cascade. Typical implementation behavior is to make `Error` absorb many relation queries internally.

Do **not** conclude:

```text
Error <: T
```

as a language rule. `Error` may not be a user-denotable type at all.

Similarly, parser recovery can create missing nodes. A missing annotation is a source recovery/absence fact, not `Dynamic` unless language semantics explicitly convert it later.

## 11. Example: typing a conditional

Suppose:

```phalcom
choose(flag: Bool, x: Int, y: String) {
  if flag {
    return x
  }
  return y
}
```

A result-inference rule might collect reachable normal returns:

```text
R = {Int, String}
result = join_type(R)
```

If explicit unions are part of the normative type algebra:

```text
join_type(Int, String) = Int | String
```

If they are not, the join might be the least nominal/common safe supertype. That is a language-design choice.

A `return fail()` with `fail() : Never` should not widen the join if the core lattice says `T | Never = T`.

This example demonstrates why control-flow reachability and type join policy are inputs to result inference rather than guesses from syntax.

## 12. Example: generic member substitution

Declaration:

```text
class Box<T> {
  value -> T
}
```

Given application `Box<Int>`, member view should satisfy:

```text
subst = { Box.T ↦ Int }
return_type(Box.value, subst)
= T[Int/Box.T]
= Int
```

A nested method-owned `T` must not be replaced:

```text
class Box<T> {
  choose<T>(x: T) -> T
}
```

Owner-qualified identity makes this automatic.

## 13. Dependency and cache consequences

A cached judgment is a function of its semantic inputs. A conceptual key may include:

```text
(query kind,
 expression/semantic identity,
 expected TypeId if checking,
 substitution environment,
 receiver/Self context,
 relevant declaration generation,
 dynamic/checker mode)
```

Do not key solely by source position. Do not store a result without recording which declaration/type relation generations make it valid.

This is where metatheory meets incremental architecture: weakening says unrelated facts should not matter, while dependency tracking identifies the facts that do.

## 14. Failure modes

- One `HashMap<String, Type>` for all value/type namespaces.
- A type parameter shadow creates accidental substitution because strings match.
- The algorithm implements only a convenient subset but documentation calls it complete.
- A recovery type leaks into reflection as if user wrote it.
- A dynamic send result is treated as statically proven member existence.
- A recursion depth limit becomes the meaning of recursive subtyping.
- A checker theorem ignores reflection/native mutation that can invalidate it.
- A block is typed as pure `A -> B` despite non-local return semantics relevant to the task.

## 15. Verification obligations

For every new typing rule, ask:

1. What is the declarative judgment?
2. What are all context inputs?
3. What is the formation precondition?
4. Is the rule syntax-directed or does it rely on subsumption/search?
5. How does the algorithm terminate?
6. Is it sound? Complete? Intentionally incomplete?
7. What dynamic behavior does the guarantee cover?
8. What boundary invalidates the guarantee?
9. What semantic IDs are involved?
10. What provenance is required for a failure?

## 16. Competency questions

1. Why is `Γ ⊢ e ⇒ T` different from `Γ ⊢ e ⇐ T` in implementation, not just notation?
2. Give a Phalcom form that benefits from checking mode because it lacks enough information to synthesize.
3. Why does substitution require owner-qualified type-parameter identity?
4. State a useful scoped safety theorem for sends in checked Phalcom code.
5. Why is a compiler `ErrorType` not automatically top or bottom?
6. What does algorithmic completeness mean relative to a declarative subtyping relation?
7. Which context inputs would make a cached block-checking result stale?
