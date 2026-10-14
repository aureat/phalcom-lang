# Telos contextual enum-variant inference — implementation requirements and live-branch audit

**Date:** 2026-10-14  
**Target:** `aureat/phalcom-lang`, `feat/integration-universe`  
**Basis:** attached 2,387-line architectural handoff and fresh read-only GitHub connector examination of the named branch.  
**Scope:** checker architecture, owner resolution, bidirectional typing, exact semantic identities, generic flow, diagnostics, regression gates. No code was modified and no builds/tests were executed.

## Executive ruling

**Implement an owner-first, context-provided variant resolver, shared between expression and pattern elaboration.** A bare unresolved name does not become an enum-variant candidate merely because of its spelling or capitalization. Only a semantically established expected type or pattern domain may supply eligible enum owners. Qualified syntax remains an explicit escape hatch. Existing exact `VariantId`, associated-member and invocation-target products must remain authoritative through lowering.

The live branch verifies three independent defects: (1) expected-type propagation exists at annotated declarations, but `Expr::Var` does not consult it for unresolved variants; (2) `synthesize_unqualified_call` takes an expected type but has no matching owner-first variant-constructor fallback; (3) `resolve_variant_pattern` derives a union-aware owner candidate list, but its unqualified constructor branch still selects only `nominal_origin_declaration(subject.canonical)`. This differs from the union-aware singleton helper. A fourth, companion issue is preserving useful generic/local-type knowledge across RHS synthesis, flow joins and pattern-boundary normalization.

### Critical correction to the attached handoff

The handoff's sections 9 and 31 describe `if let` and `while let` RHS checking as invariably `ExpectedType::None`, and claim `while let` has no whole-scrutinee annotation. **That is not true of the live branch retrieved for this audit.** `expression.rs:893–915` checks `if_let.annotation` and passes an elaborated `expected_scrutinee` to RHS analysis; `expression.rs:964–991` likewise handles `while_let.annotation` and passes that expectation into each loop-step RHS evaluation. Without an annotation, those paths still fall back to `ExpectedType::None`; their generic RHS synthesis risks remain. Before changing grammar or AST, inspect and test the existing annotation feature. The original handoff is retained below as historical baseline, but these live-branch observations supersede its contradictory assertions.

## Live source evidence and code locations

| Location | Verified behavior | Required intervention |
|---|---|---|
| [`checker/statement.rs:45–75`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/statement.rs#L45-L75) | Annotated binding elaborates a canonical or inference expectation, then calls `analyze_expression` with it. | Preserve pipeline; test both canonical and symbolic applied contexts. |
| [`checker/expression.rs:344–411`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/expression.rs#L344-L411) | `Expr::Var` resolves locals, types, type parameters, then returns `UnknownReason::UnresolvedName`; no contextual enum-member fallback. | Invoke shared resolver only after ordinary resolution fails and a valid enum domain exists. |
| [`checker/expression.rs:4504+`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/expression.rs#L4504) | `synthesize_unqualified_call` already receives `expected`, starts with ordinary local callable resolution. | Add owner-restricted variant family/selector fallback; preserve lexical precedence and callable pipeline. |
| [`checker/expected.rs:28–76`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/expected.rs#L28-L76) | `ExpectedType::{None,Proper,Inference}`; `ty()` only handles canonical terms, not general symbolic `InferenceTerm::Applied`. | Expose domain extraction retaining declaration origin, generic args, solver context. |
| [`checker/pattern.rs:496–550`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/pattern.rs#L496-L550) | Singleton pattern resolution enumerates owner types via `nominal_owner_types`, validates singleton shape and calls `open_variant_case`. | Reuse owner enumeration and GADT reachability for all pattern shapes. |
| [`checker/pattern.rs:634–657`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/pattern.rs#L634-L657) | `nominal_owner_types` recursively decomposes canonical union type members. | Extend to local/symbolic terms without discarding member-specific specializations. |
| [`checker/pattern.rs:739–791`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/pattern.rs#L739-L791) | `resolve_variant_pattern` computes `expected_owners` but unqualified path falls back only to `expected_nominal_decl`. | Resolve candidates over all domain owners with explicit ambiguity/blocked/refuted outcomes. |
| [`checker/expression.rs:893–915, 964–991`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/expression.rs#L893-L991) | **Live correction:** `if let` and `while let` both elaborate optional RHS annotations; `while let` uses that expectation each iteration. | Test current syntax; do not implement redundant AST extension. |
| [`checker/expression.rs:5963–6011`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/expression.rs#L5963-L6011) | Coverage subject starts from `typed.knowledge.ty()` or Unit fallback, tries generic normalization, carries `typed.local_type`, and retains blocked evidence. | Separate absent type knowledge from actual Unit; improve local owner extraction and proof preservation. |
| [`checker/statement.rs:922–975`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/statement.rs#L922-L975) | Declaration validation explicitly rejects `Pattern::Variant` as refutable. | Treat required `let Some(x)` semantics as separate language change. |
| [`checker/expression.rs:2566+`](https://github.com/aureat/phalcom-lang/blob/feat/integration-universe/phalcom-semantic/src/checker/expression.rs#L2566) | Qualified `synthesize_associated_invoke` resolves owner/family/member and constructor metadata. | Reuse this authoritative resolution and callable specialization rather than inventing parallel targets. |

## Implementation contract

### I. Context sources and lookup precedence

A contextual variant lookup is legal *only* if independently obtained type evidence supplies one or more enum owner declarations. Eligible sources: explicit local annotations, return contracts, assignment target contracts, callable parameter contracts, candidate-specific overloaded-call expectations, known pattern scrutinee types, and supported `if let` / `while let` annotations. Run ordinary lexical/value/type resolution first for expressions, then contextual fallback. For bare `let x = None` and `let x = Some(1)` without a resolved ordinary symbol or expected type, preserve normal unresolved-name/call diagnostics. Never globally search variants by spelling; neither capitalization nor built-in `Option`/`Result` naming is evidence.

### II. Domain representation

Use an abstract *contextual algebraic domain* carrying canonical nominal instances, union members, and inference-applied nominal shells. Each candidate must preserve `(DeclarationId, expected specialization, inference context, local rigid/projection evidence, origin)`. The resolver must accept incomplete generics (e.g., `Option<α>`) when the owner declaration is already known; it must not demand `expected.ty(): Some(TypeId)`. Keep `Blocked`, `Underconstrained`, `Refuted`, `Missing`, and `Ambiguous` distinct. A missing owner is not an enum-inference error unless a context independently establishes one.

### III. Candidate selection and semantic identity

Within eligible owners only, resolve the appropriate getter or method/selector family using base name, argument count, labels, visibility, evaluation phase and constructor shape; specialize against the contextual owner and solve GADT/exact-case/local-rigid constraints before committing. Constructor arguments must be checked against candidate-specific parameter types; no arbitrary first-candidate or declaration-order preference. On success, publish the same declaration-backed `VariantId`, `AssociatedMemberId`, `InvocationTargetId`, specialized result and argument mapping as corresponding qualified syntax. No fresh runtime spelling-based owner inference.

### IV. Expression integration and overloads

Connect the service at `Expr::Var` and `synthesize_unqualified_call` after ordinary resolution. `Some(42)` under `Option<Int>` must select the owner before using existing `apply_resolved_callable` generic result constraints and payload analysis. Under `Option<_>`, `Some(42)` may solve `α=Int`; `None` may resolve owner and variant yet leave α underconstrained. Overloaded `consume(None)` must defer contextual checking to each call candidate's own parameter contract, then rank/diagnose overloads without leaking constraints or diagnostics from rejected candidates.

### V. Pattern integration

Unify bare singleton, constructor-shaped, family and selector-pattern paths. Fix union behavior first: `Same` and `Same(x)` must use equivalent owner-domain candidate discovery. Preserve reachability/coverage, blocked reasons, GADT specialization, exhaustiveness and binding projection. For a bare name pattern, a matching singleton in the independently established subject domain may take contextual precedence under the existing rule; otherwise the name remains a binder. Never interpret capitalization as binding-versus-variant syntax.

### VI. Generic RHS and mutable local preservation

Audit `coverage_subject_for_expression`, `LocalType`, inference substitutions, projection normalization, loop flow joins and mutable assignments as a *separate companion workstream*. The branch's generic `Chain<T>` annotations are symptoms/evidence, not proof of one particular bug. An iterator's `Option<<I as Iterable>::Item>` nominal shell should survive even while the element projection remains symbolic or blocked. `while let Some(v) = iterator.next` must bind `v` at the projection/local type that follows from the iterator contract, and not erase evidence at each loop backedge. A blocked associated projection must not be diagnosed as a nonexistent variant.

### VII. Diagnostics and dependent features

Without contextual owner: ordinary unresolved identifier or callable. With an owner but no case: expected-domain missing member/selector. More than one viable owner: candidate ambiguity with qualification advice. Selected case but unsolved generics: generic underconstraint. Impossible specialized case: GADT/coverage contradiction. Preserve ordinary lexical collisions. `let Some(x) = value` remains rejected today, irrespective of name inference; required destructuring needs a separate language ruling, binding/lowering behavior and runtime failure semantics. Nested typed patterns should likewise be planned separately unless already supported by the exact grammar under test.

## Ordered implementation sequence and exit gates

1. **Baseline/golden tests:** lock current qualified resolution and semantic target identity; add failing contextual-expression and union-constructor pattern tests; capture current `if let`/`while let` annotation syntax.
2. **Domain service:** canonical, union and symbolic-applied owner extraction with proof-carrying generics, rigid locals, projection status and no name search.
3. **Shared candidate service:** declaration-backed candidate collection, selector filtering, visibility, exact case/GADT proof, blocked/refuted/ambiguous outcomes; migrate pattern singleton/constructor paths and fix union parity.
4. **Expression hooks:** wire bare identifiers and calls to candidate service, reuse callable specialization and resolution publication; verify no-context and lexical-collision negatives.
5. **Overload isolation:** candidate-local expected parameter checking, solver snapshot/rollback or equivalent speculative isolation; no premature chosen owner.
6. **Generic-flow companion:** `Chain<T>` mutable locals, generic iterator associated item projections, `if let` / `while let` loop backedges; isolate independent flow defects instead of masking with explicit qualification.
7. **Linked end-to-end gate:** semantic checking + linked lowering + executable matching/constructing for user-defined enums as well as Universe enums. Only then de-qualify a small representative `packages/universe` slice.
8. **Corpus and performance gate:** broad semantic and integration regressions, deterministic diagnostics, incremental dependency invalidation on enum changes; do not recompile unrelated owners or scan every enum for contextual names.

**Minimum regression matrix:** `None`, `Some(1)` in annotated let, expression returns, assignments, parameters, `Option<_>`, nested `Some(Ok(x))`, singleton and payload union ambiguity, case narrowing/GADTs, `if let`, `while let` with/without annotation, generic `Iterable::Item`, mutable locals, shadowed lexical `Some`, unconstrained shorthand, incorrectly shaped selector, qualified/unqualified semantic product equivalence, and exact runtime lowering. Add user-defined sum types and identifiers with lowercase variants to prove no naming conventions are used.

## Verification limits

The live GitHub source paths above were fetched from `feat/integration-universe` during this report. No live branch SHA was fetched in this pass, so the handoff's `38def701...` is a **historical reference**, not independently confirmed as current HEAD here. No tests were executed. The 2,387-line handoff below includes further historical comparisons, illustrative pseudo-code and proposals; they are not all independently reverified in this pass. In the event of conflict, the live-source corrections and requirements above take precedence.

---

# Appendix A — attached architectural handoff (preserved verbatim)

# Enum variant inference audit — `feat/integration-universe`

I inspected the current remote `feat/integration-universe` branch, its recent history, the semantic checker paths, pattern lowering, and the newer `feat/telos-collections` checkpoint that has not yet been merged into integration.

The central finding is clear:

> **Telos does not have one broken “enum inference” feature. It currently has several partially independent mechanisms, and only the pattern-side mechanism has true contextual variant-owner inference.**
>
> Fully qualified forms such as `Option::Some`, `Option::None`, `Result::Ok`, and `Result::Error` bypass the missing layer by supplying the owner before specialization begins.

That explains why qualification has spread through fixtures and Universe source even though the repository already contains tests proving that contextual shorthand is intended language behavior.

There is also a **second, adjacent inference problem** visible in `packages/universe`: generic/local type information becomes insufficient in some iterative and mutable-flow contexts. That is why the branch has accumulated annotations such as:

```telos
let mut remaining: Chain<T> = self
let mut inner: Chain<U> = f(head)
let mut step: Option<T> = iterator.next
```

That problem intersects `while let Some(...)`, but it is not itself variant-name resolution. Both need to be repaired for the final ergonomic result.

---

# 1. Repository state I inspected

Current remote integration head:

```text
branch: feat/integration-universe
HEAD:   38def7014ad9f93e8f7569b70bcf1426cdba7888
commit: fix(universe): align collection iterator APIs
```

Relevant recent history:

```text
38def7014a  fix(universe): align collection iterator APIs
264140764b  feat(intervals): add interval relations, views, and functional loop typing
6efa6188bd  fix(universe): annotate drop loop variable type in Chain
bbfe790578  feat(LANG007.C5): start C5.P1 interval algebra and relational matching
1c9e7920f9  docs(LANG007.C4): complete P9 ...
5685a34071  Merge branch 'feat/telos-collections' into feat/integration-universe
a09de379d9  feat(LANG007.C4): complete P7 ... and P8 ...
```

There is an important branch-state wrinkle:

```text
feat/integration-universe
    contains feat/telos-collections through a4385926

feat/telos-collections
    subsequently advanced to 60e6a3db
```

So integration is currently one collections checkpoint behind. I inspected `60e6a3db` separately because that checkpoint contains a substantial source/fixture migration and makes the qualification trend particularly obvious.

This matters because the problem cannot be understood by looking only at current integration diffs.

---

# 2. What the repository says the language is supposed to do

The semantic test suite already establishes contextual pattern variants as intended behavior.

For example, `phalcom-semantic/tests/semantic/adts/matching/resolution.rs` contains:

```telos
data Option<T> {
    Some(_ value: T) -> Option<T>
    None -> Option<T>
}

class Test {
    run(_ value: Option<Int>) {
        match value {
            Some(x) => x
            None => 0
        }
    }
}
```

There are also tests for:

```telos
Some(Ok(x))
Some(Error(x))
None
```

and ambiguity across:

```telos
Left | Right
```

where both owners expose:

```telos
Same
```

The test explicitly requires that the checker **not arbitrarily choose an owner**.

That matches the existing match-semantics specification. Contextual variant resolution is supposed to be:

```text
known pattern domain
    ↓
eligible enum/sum owner(s)
    ↓
matching variant family/selector
    ↓
specialization / GADT applicability
    ↓
one semantic interpretation or ambiguity
```

It is explicitly **not** supposed to be:

```text
name "Some"
    ↓
search the entire program
    ↓
guess Option
```

That is exactly consistent with the design decision we reached in the preceding discussion.

---

# 3. Why fully qualified forms work

Consider:

```telos
Option<Int>::Some(42)
```

or:

```telos
Result::Error(error)
```

The syntax has already established the owner.

The checker can therefore go directly into the mature associated-member machinery:

```text
Option<Int>
    ↓
resolve associated namespace
    ↓
find Some family
    ↓
find exact Some(_) selector
    ↓
specialize constructor against Option<Int>
    ↓
check argument
    ↓
publish exact variant/call target
```

The relevant implementation is already substantial:

- `checker/expression.rs`
  - `synthesize_associated_lookup`
  - `synthesize_associated_invoke`
- `checker/associated.rs`
  - associated-family/member lookup
  - specialization
- `checker/call.rs`
  - generic callable application
  - expected-result constraints
  - payload checking
- `EnumSemanticTable`
  - owner-indexed enum metadata
  - declaration-backed variant identity

This is why qualification is such an effective workaround.

It skips precisely the missing question:

> **Given an unqualified `Some`, which declaration-owned associated namespace should I search?**

Everything after that question is substantially more developed.

---

# 4. The first major defect: expression inference never asks the expected type for the owner

This is the clearest defect in the repository.

Consider:

```telos
let x: Option<Int> = None
```

The compiler already propagates the annotation correctly.

In `phalcom-semantic/src/checker/statement.rs`, a `let` annotation becomes:

```rust
ExpectedType::proper_from(
    ty,
    ExpectationOrigin::DeclarationContract,
)
```

and the initializer is analyzed using it:

```rust
analyze_expression(ctx, expr, &expected_init)
```

So the information is present:

```text
initializer: None
expected:    Option<Int>
```

The problem is what happens next.

## `Expr::Var` throws the contextual information away

Current `checker/expression.rs` around the `Expr::Var` branch performs:

```rust
if let Some(fact) = ctx.lookup_local(value) {
    ...
} else if let Some(decl) = ctx.resolve_type_name(value) {
    ...
} else if let Some(param_ty) = ctx.resolve_type_parameter(value) {
    ...
} else {
    TypedExpression::unknown(
        UnknownReason::UnresolvedName(value.as_str().into())
    )
}
```

Notice what is absent.

There is no:

```rust
else if let Some(domain) = contextual_algebraic_domain(expected) {
    resolve_contextual_variant(domain, value)
}
```

Therefore:

```telos
let x: Option<Int> = None
```

currently behaves semantically like:

```text
expected = Option<Int>           ← checker has this
        ↓
Expr::Var("None")
        ↓
local? no
type? no
type parameter? no
        ↓
UnresolvedName("None")
```

The expected type is simply never consulted.

This is not difficult enum inference failing.

**The inference path does not exist.**

---

# 5. The same missing bridge exists for `Some(...)`

Now consider:

```telos
let x: Option<Int> = Some(42)
```

The parser produces an unqualified call.

It eventually reaches:

```rust
synthesize_unqualified_call(
    ctx,
    call,
    expected,
)
```

The function receives the expected type.

But its lookup progression is currently approximately:

```text
1. local callable
2. send on self
3. named data constructor / nominal declaration
4. unresolved name
```

Its terminal path is literally:

```rust
TypedExpression::unknown(
    UnknownReason::UnresolvedName(call.name.as_str().into())
)
```

There is again no transition:

```text
ordinary unqualified callable resolution failed
        +
expected = Option<Int>
        ↓
look for matching variant constructor Some(_)
in Option's associated variant namespace
```

Consequently:

```telos
Option<Int>::Some(42)
```

works because `Option<Int>` is syntactically supplied.

But:

```telos
let x: Option<Int> = Some(42)
```

does not have an expression-side mechanism that extracts `Option` from the expected result type.

That is the second manifestation of the same architectural hole.

---

# 6. Pattern inference is substantially further ahead

Patterns have a mechanism expressions simply do not have.

`checker/pattern.rs` currently contains:

```rust
Pattern::Name { name, range } => {
    if let Some(var_res) =
        try_resolve_contextual_singleton(
            ctx,
            name,
            *range,
            subject,
        )
    {
        PatternResolution::Variant(var_res)
    } else {
        bind_name_pattern(...)
    }
}
```

And `try_resolve_contextual_singleton` does the right fundamental thing:

```rust
let owner_types =
    nominal_owner_types(ctx.store, subject.canonical);
```

Then for each eligible owner it consults:

```rust
enum_table.enums.get(&owner)
```

and searches for the appropriate selector.

This is important architecturally.

The compiler is **not** asking:

```text
"What enum somewhere has a member called None?"
```

It asks:

```text
"What enum owners are represented by this expected pattern subject?"

then:

"Does one of those owners have this member?"
```

That is exactly the desired direction of inference.

---

# 7. Nested contextual patterns already demonstrate the correct model

The existing tests go further than simple `Some`.

For:

```telos
value: Option<Result<Int, String>>

match value {
    Some(Ok(x)) => ...
    Some(Error(e)) => ...
    None => ...
}
```

the conceptual inference is:

```text
pattern domain
Option<Result<Int, String>>
        │
        ├── Some(...)
        │      ↓ payload domain
        │   Result<Int, String>
        │      │
        │      ├── Ok(...)
        │      │      ↓
        │      │     Int
        │      │
        │      └── Error(...)
        │             ↓
        │           String
        │
        └── None
```

This already exists as a semantic design and regression target.

So the full repair should **generalize an existing correct pattern principle**, not invent a new inference philosophy.

---

# 8. But even pattern inference currently has holes

Pattern inference is ahead of expressions, but it is not complete.

## 8.1 Singleton names and constructor patterns use different owner-selection machinery

Bare singleton contextual resolution uses:

```rust
nominal_owner_types(ctx.store, subject.canonical)
```

That handles union subjects.

For example:

```telos
value: Left | Right

match value {
    Same => ...
}
```

can retain:

```text
Left::Same
Right::Same
```

and diagnose ambiguity.

Good.

But `resolve_variant_pattern`, which handles syntax such as:

```telos
Some(x)
Case(...)
Case*
```

starts with:

```rust
let expected_nominal_decl =
    ctx.store
        .nominal_origin_declaration(subject.canonical)
        .cloned();

let expected_owners =
    nominal_owner_types(ctx.store, subject.canonical);
```

Then the unqualified path uses:

```rust
} else if let Some(decl) = expected_nominal_decl {
    decl
} else {
    // unresolved owner
}
```

That means it computes the union-aware `expected_owners`, but does **not use them to choose unqualified constructor-shaped variants**.

For a union:

```text
A | B
```

`nominal_origin_declaration(A | B)` is not one declaration.

So:

```telos
Same(x)
```

does not get the same multi-owner contextual treatment as:

```telos
Same
```

That should be unified.

---

# 9. `while let` exposes another boundary

Current `Expr::WhileLet` does:

```rust
let val_typed =
    analyze_expression(
        step_ctx,
        &while_let.value,
        &ExpectedType::None,
    );

let (subject, refinement_block) =
    coverage_subject_for_expression(
        step_ctx,
        &val_typed,
    );

...

resolve_pattern(
    ctx,
    &while_let.pattern,
    &subject,
    ...
)
```

So for:

```telos
while let Some(value) = iterator.next {
    ...
}
```

the direction is:

```text
iterator.next
    ↓ synthesize with no expected type
infer complete RHS type
    ↓
construct CoverageSubject
    ↓
resolve Some(...) from RHS domain
```

This is logically sound when:

```text
iterator.next => Option<T>
```

is already sufficiently established.

But it explains why generic iterator-heavy Universe code is a stress case.

If the RHS is only:

- symbolically known,
- projection-dependent,
- underconstrained,
- locally rigid,
- or loses enough information during flow analysis,

then the pattern does not have a proper canonical owner to inspect.

The variant resolver itself may be fine; its **input domain is deficient**.

---

# 10. `coverage_subject_for_expression` reveals the canonicalization pressure

The current implementation does approximately:

```rust
let original_ty =
    typed.knowledge.ty()
        .unwrap_or_else(|| ctx.store.unit());
```

It then attempts generic refinement and ultimately constructs:

```rust
CoverageSubject::from_parts(
    scrutinee_ty,
    scrutinee_local_ty.unwrap_or(
        LocalType::Canonical(scrutinee_ty)
    ),
)
```

The presence of `LocalType` is good: the pattern engine has evolved to retain local rigid information.

But owner discovery still fundamentally starts from:

```rust
subject.canonical
```

and `nominal_owner_types`.

So there is an important distinction:

```text
LocalType / symbolic knowledge
    can preserve useful specialization evidence

but

contextual owner enumeration
    currently wants a canonical TypeId domain
```

That becomes relevant with generic iterators, associated-type projections, and partially solved types.

---

# 11. The branch history gives concrete evidence of this second problem

This is one of the more revealing findings.

Commit:

```text
8d168c0ec9
feat(universe): modernize shadow match and iterator code
```

modernized `Chain.fromIterable` from a manual loop to:

```telos
while let Option::Some(value) = iterator.next {
    result = result.prepend(value)
}
```

and similarly modernized `Chain.partition`.

So the desired source style was actively being introduced.

Then `264140764b` later changed `Chain.fromIterable` back to:

```telos
let mut step: Option<T> = iterator.next

while step.isSome {
    match step {
        Option::Some(value) => {
            result = result.prepend(value)
        }
        Option::None => {}
    }

    step = iterator.next
}
```

and similarly altered other code.

At the same time that commit added explicit generic annotations throughout `Chain`:

```telos
let mut remaining: Chain<T> = self
```

instead of:

```telos
let mut remaining = self
```

and:

```telos
let mut inner: Chain<U> = f(head)
```

instead of:

```telos
let mut inner = f(head)
```

and:

```telos
let mut left: Chain<T> = self
let mut right: Chain<U> = other
```

Commit `6efa6188bd` then added yet another:

```telos
let mut remaining: Chain<T> = self
```

in `Chain.drop`.

This is not coincidence.

It says that integration work is encountering a broader problem with preserving sufficiently exact generic/local types through mutable iteration.

---

# 12. But do not conflate that with contextual variant-name inference

There are therefore two different defects:

| Defect | Example | State |
|---|---|---|
| Contextual owner lookup missing | `let x: Option<Int> = None` | expression path absent |
| Contextual constructor lookup missing | `let x: Option<Int> = Some(1)` | expression path absent |
| Pattern contextual lookup | `match x { Some(v) => ... }` | substantially implemented |
| Generic/mutable local type retention | `let mut remaining = self` in `Chain<T>` | branch currently needs annotations in several sites |
| Generic RHS → pattern domain | `while let Some(v) = iterator.next` | depends on sufficiently good RHS inference |
| Required variant destructuring | `let Some(x) = value` | explicitly rejected today |

A comprehensive repair should address the first three inference boundaries without hiding the fourth behind qualification.

---

# 13. Why the Universe migration moved toward qualification

The source modernization history also explains an apparent contradiction.

The Universe modernization plan explicitly chose forms such as:

```telos
while let Option::Some(value) = iterator.next
```

and:

```telos
match result {
    Result::Ok(value) => ...
    Result::Error(error) => ...
}
```

But its validation at that stage was substantially parser/interface-oriented rather than full semantic-linked execution.

So the modernization established **syntax/style migration**, not proof that contextual inference worked throughout the semantic compiler.

That distinction is important.

And the current source continues to contain many explicit forms:

```telos
Option::Some(...)
Option::None

Result::Ok(...)
Result::Error(...)

Chain::Empty
Chain::Link(...)
```

The newer `60e6a3db` collections checkpoint extends this pattern into newly added packages such as UUID, NanoID, and Snowflake code.

That means qualification is now masking inference defects in real source.

---

# 14. There is an especially important historical reason for the current state

The repository used to have unsafe fallback behavior.

Commit:

```text
55bf6a5e7c
fix(core): require semantic identity for variant matches
```

removed code that effectively guessed owners from spellings.

The removed fallback contained logic equivalent to:

```rust
if v.base == "Some" || v.base == "None" {
    "Option"
} else if v.base == "Ok"
       || v.base == "Error"
       || v.base == "Err"
{
    "Result"
}
```

That is precisely the kind of behavior we do **not** want.

The commit correctly established:

> Variant identity is semantic and cannot be guessed during fallback lowering.

This was the correct architectural move.

But it created a requirement:

```text
remove guessed identity
        +
provide semantic contextual identity everywhere variants are legal
```

The first half was completed.

The second half is only partially complete.

That is, in my view, the historical root of the present inconsistency.

---

# 15. Why old fixtures can be misleading

There are still fixtures using forms such as:

```telos
let mut opt = Some(42)

if let Some(x) = opt {
    ...
}
```

and:

```telos
while let Some(value) = cursor.next {
    ...
}
```

Those fixtures should not be taken as proof that the current canonical semantic pipeline correctly supports the desired inference.

The repository has evolved through:

- older global/generated variant-class behavior,
- unlinked compiler paths,
- semantic lowering,
- declaration-backed enum identities,
- associated families,
- GADT specialization,
- exact-case types.

A fixture can remain syntactically present while the modern semantic path lacks the corresponding contextual inference.

For the repair, the authoritative tests need to be in the **semantic checker and linked lowering path**, not merely legacy source fixtures.

---

# 16. Work already happening that helps — but does not solve owner inference

The current branch contains important generic inference improvements.

One particularly relevant section in `checker/call.rs` now says:

```rust
// Seed contextual result evidence before checking data-constructor
// arguments. A generic variant's result parameters are invariant: the
// expected result must select those parameters before payload arguments
// contribute lower bounds ...
```

Conceptually, it does:

```text
known constructor target
      ↓
constructor returns Result<T,E>
      +
expected Result<Unit, Base>
      ↓
constrain constructor result first
      ↓
solve T/E
      ↓
then type-check payload arguments
```

This is exactly the right ordering.

For example:

```telos
let x: Result<Unit, Base> =
    Result::Error(Derived.new())
```

the expected result should first establish the generic result specialization.

This work is very useful for contextual variant inference.

But notice the prerequisite:

> **`Result::Error` has already been resolved to a particular variant constructor.**

It improves:

```text
variant selected
    ↓
generic specialization
```

It does not solve:

```text
bare Error(...)
    ↓
which owner?
```

The full repair should reuse this existing machinery rather than duplicate it.

---

# 17. The missing abstraction: contextual algebraic domain

`ExpectedType` today is:

```rust
pub enum ExpectedType {
    None,

    Proper {
        ty: TypeId,
        origin: ExpectationOrigin,
    },

    Inference {
        context: InferenceContextId,
        term: InferenceTerm,
        origin: ExpectationOrigin,
    },
}
```

Its convenient:

```rust
expected.ty()
```

only succeeds for:

```text
Proper(TypeId)
Inference(Canonical(TypeId))
```

It does **not** return a type for a symbolic applied inference term.

That is insufficient for the desired language.

Consider:

```telos
let x: Option<_> = Some(42)
```

The checker may know:

```text
owner = Option
argument = inference variable α
```

before it knows a canonical:

```text
Option<Int>
```

That is already enough to search only `Option` for `Some`.

Therefore contextual variant resolution must not require:

```rust
expected.ty(): Option<TypeId>
```

as its sole entry point.

---

# 18. Required representation

I would introduce an abstraction along these lines:

```rust
pub enum ContextualAlgebraicDomain {
    Canonical {
        ty: TypeId,
    },

    InferenceApplied {
        context: InferenceContextId,
        owner: DeclarationId,
        arguments: Box<[InferenceStaticArgument]>,
        term: InferenceTerm,
    },

    Union {
        members: Box<[ContextualAlgebraicDomain]>,
    },
}
```

The exact representation can vary, but it needs to answer:

```text
Which declaration-owned sum namespaces
are legitimately supplied by this context?
```

without prematurely requiring all generic arguments to be canonical.

Existing `ExpectedType` collection helpers already demonstrate this technique.

For example, `collection_element_type` and `map_key_val_types` inspect:

```rust
InferenceTerm::Applied { ... }
```

directly.

So this would not introduce a foreign inference model. It generalizes a pattern that is already present.

---

# 19. The shared resolver should be owner-first

The central new semantic service should look conceptually like this:

```rust
enum ContextualVariantSyntax {
    Singleton {
        name: Box<str>,
    },

    ExactConstructor {
        name: Box<str>,
        selector: Selector,
    },

    PatternSelector {
        name: Box<str>,
        selector: SelectorPattern,
    },

    WholeFamily {
        name: Box<str>,
    },
}
```

with an input:

```rust
struct ContextualVariantRequest {
    domain: ContextualAlgebraicDomain,
    syntax: ContextualVariantSyntax,
    site: ContextualVariantSite,
}
```

and outcomes richer than `Option<T>`:

```rust
enum ContextualVariantResolution {
    Resolved(ResolvedContextualVariant),

    Ambiguous {
        candidates: Box<[ContextualVariantCandidate]>,
    },

    Missing,

    Contradictory,

    Blocked(BlockReason),

    Underconstrained,
}
```

The important property is that this API **requires a domain**.

There should be no:

```rust
resolve_some_variant_by_name("Some")
```

API.

---

# 20. Required resolution algorithm

The full algorithm should be:

```text
1. Ordinary language name resolution runs first where applicable.

2. Obtain contextual algebraic domain independently from:
      - expected expression type,
      - expected callable parameter type,
      - return contract,
      - assignment contract,
      - pattern scrutinee,
      - explicit whole-pattern annotation,
      - candidate-specific overload expectation.

3. If no algebraic domain exists:
      contextual variant inference is not applicable.

4. Enumerate only declaration owners represented by that domain.

5. For each owner:
      inspect its EnumSemanticTable entry.

6. Filter by:
      - base name,
      - getter vs method,
      - arity,
      - argument labels,
      - exact selector / selector pattern,
      - whole-family mode,
      - visibility,
      - evaluation phase if relevant.

7. Specialize each surviving variant against its
   expected owner specialization.

8. Apply:
      - generic substitutions,
      - exact-case constraints,
      - GADT branch constraints,
      - local-rigid constraints.

9. Refute impossible candidates.

10. If expression constructor:
       check arguments under candidate-specific
       parameter expectations.

11. Results:
       0 viable → missing/contradictory
       1 viable → resolved
       >1 viable → ambiguity

12. Never pick based on source order.
```

---

# 21. Expression resolution requirements

## 21.1 Bare singleton

For:

```telos
let x: Option<Int> = None
```

the desired path is:

```text
ordinary lookup None
    ↓ unavailable
expected type Option<Int>
    ↓
contextual domain [Option<Int>]
    ↓
Option enum metadata
    ↓
getter selector None
    ↓
Option::None
```

But for:

```telos
let x = None
```

there is no contextual domain.

Therefore:

```text
ordinary unresolved name
```

No enum-specific guessing.

---

## 21.2 Constructor call

For:

```telos
let x: Option<Int> = Some(42)
```

the desired path is:

```text
ordinary callable lookup Some
    ↓ unavailable
expected Option<Int>
    ↓
Option variant family Some
    ↓
selector Some(_)
    ↓
specialize constructor
    ↓
check 42 : Int
```

For:

```telos
let x = Some(42)
```

if ordinary callable/name resolution cannot find `Some`:

```text
cannot resolve callable `Some`
```

Again, no global enum search.

---

# 22. Generic contextual construction

This is essential, not optional.

The implementation must support:

```telos
let x: Option<_> = Some(42)
```

as:

```text
expected owner = Option<α>
        ↓
resolve Option::Some
        ↓
payload parameter = α
        ↓
argument = Int
        ↓
α = Int
        ↓
Option<Int>
```

This is where the current expected-result seeding work in `call.rs` becomes extremely useful.

Conversely:

```telos
let x: Option<_> = None
```

should produce:

```text
owner:   Option
variant: None
generic: α remains unsolved
```

The error is not:

```text
cannot resolve None
```

because the variant was resolved.

It is an underconstrained generic:

```text
`None` resolves to `Option::None`,
but the element type of `Option<_>` cannot be inferred
```

or the equivalent existing generic-inference diagnostic.

---

# 23. Function returns

This must also work:

```telos
fn lookup() -> Option<Int> {
    None
}
```

and:

```telos
fn lookup() -> Option<Int> {
    Some(42)
}
```

The expected return contract already belongs in bidirectional checking.

The contextual resolver should not care whether `Option<Int>` came from:

- a local annotation,
- a return annotation,
- an assignment,
- a parameter.

Those are all simply different `ExpectationOrigin`s.

---

# 24. Assignments

Given:

```telos
var current: Option<Int>

current = None
current = Some(42)
```

the assignment target should provide:

```text
ExpectedType(
    Option<Int>,
    AssignmentContract
)
```

and therefore contextual variants should behave identically to annotated initialization.

If assignment currently fails to propagate that expected type all the way into `Expr::Var` or `UnqualifiedCall`, that is part of the same repair.

---

# 25. Callable arguments

This must work:

```telos
consume(Some(42))
```

where:

```telos
consume(_ value: Option<Int>)
```

The parameter expectation supplies the domain.

This becomes more subtle with overloads:

```telos
consume(_ x: Option<Int>)
consume(_ x: Result<Int, Error>)
```

Suppose both sum types have a case called `None`.

Then:

```telos
consume(None)
```

must not resolve `None` globally before overload resolution.

Instead:

```text
candidate A
parameter expectation Option<Int>
    → try None in Option

candidate B
parameter expectation Result<Int, Error>
    → try None in Result
```

Contextual variants therefore participate in **candidate-local checking**.

If both overloads remain viable, the result is an overload ambiguity.

This is a critical implementation requirement.

---

# 26. Pattern resolution requirements

The current pattern machinery should be retained, but refactored onto the same owner-candidate service.

That means replacing the current split between:

```rust
try_resolve_contextual_singleton(...)
```

and:

```rust
resolve_variant_pattern(...)
```

with shared owner enumeration and specialization.

Then all of these behave consistently:

```telos
match value {
    None => ...
}

match value {
    Some(x) => ...
}

match value {
    Case(...) => ...
}

match value {
    Case* => ...
}
```

including union domains.

---

# 27. The bare-name pattern ambiguity should remain context-based

Per your decision, there should be **no capitalization semantics**.

Therefore:

```telos
match value {
    foo => ...
}
```

can mean a binding unless the contextual sum domain has a singleton `foo` that wins under the defined pattern rule.

The current semantic model is effectively:

```text
Pattern::Name

if expected algebraic domain contains
a matching singleton variant:
    resolve variant
else:
    bind
```

That is a legitimate language rule.

It does have an inherent ambiguity if an enum deliberately defines a variant with exactly the same spelling a programmer wanted to use as a binding:

```telos
data State {
    value
}
```

Then:

```telos
match state {
    value => ...
}
```

has to choose one interpretation.

Contextual variant-first is defensible and matches current implementation.

If Telos later wants explicit disambiguation for a binding, that should be syntactic—not casing-based.

---

# 28. `if let` and `while let`

These should share exactly the same pattern-domain resolver as `match`.

For:

```telos
while let Some(value) = iterator.next {
    ...
}
```

the semantic requirement is:

```text
synthesize iterator.next
        ↓
establish Option<Element>
        ↓
construct contextual algebraic domain
        ↓
resolve Some within Option
        ↓
project Element
        ↓
bind value: Element
```

The same applies to:

```telos
if let Ok(value) = operation() {
    ...
}
```

No explicit owner should be necessary if the RHS yields:

```text
Result<T, E>
```

unambiguously.

---

# 29. The generic iterator problem must be repaired alongside this

The Universe branch demonstrates that `while let` success depends on good RHS typing.

The final acceptance target should therefore include:

```telos
run<I>(_ source: I)
    where I |= Iterable,
          <I as Iterable>::Item == T
{
    let iterator = source.iterator

    while let Some(value) = iterator.next {
        ...
    }
}
```

and not merely:

```telos
fn simple(_ x: Option<Int>) {
    while let Some(v) = x { ... }
}
```

The generic associated-type case is what real Universe code needs.

That means fixing contextual variants alone is insufficient if:

```text
iterator.next
```

continues to degrade into unknown/underconstrained knowledge at the pattern boundary.

---

# 30. Required local/generic-type preservation work

The branch's added annotations strongly suggest this deserves an explicit companion audit.

At minimum, the repair should verify that these retain local types without manual annotation:

```telos
let mut remaining = self
```

inside:

```telos
impl<T> Chain<T>
```

should retain:

```text
Chain<T>
```

not a weaker canonical shell incapable of specializing `Chain::Link`.

Similarly:

```telos
let mut inner = f(head)
```

with:

```telos
f: (T) -> Chain<U>
```

should preserve:

```text
Chain<U>
```

across mutable flow updates.

And:

```telos
let step = iterator.next
```

where:

```text
iterator.next: Option<<I as Iterable>::Item>
```

must retain enough associated-type evidence for `Some(value)` to project the correct payload.

I would treat this as a **companion inference correctness requirement**, not fold it invisibly into the variant resolver.

---

# 31. Pattern annotations are currently much narrower than the desired eventual model

The AST already has:

```rust
pub struct LetBinding {
    pub pattern: Pattern,
    pub annotation: Option<TypeAnnotation>,
    ...
}
```

and the parser accepts:

```text
let <pattern> : Type = expression
```

So this is already possible at the **whole binding** level:

```telos
let pattern: Option<Int> = expression
```

But `Pattern` itself has no recursive:

```rust
Pattern::Typed { ... }
```

node.

Therefore:

```telos
Some(x: Int)
```

is not currently represented as a general typed nested pattern.

Nor does `while let` presently have a separate whole-pattern annotation field analogous to `LetBinding.annotation`.

For the immediate variant-inference repair this does not need to block progress.

But the complete pattern model we discussed earlier would eventually benefit from:

```rust
Pattern::Typed {
    pattern: Box<Pattern>,
    annotation: TypeAnnotation,
    range: SourceRange,
}
```

so that type constraints can live recursively inside patterns.

That should be a separate grammar/type-system extension, not smuggled into the variant repair.

---

# 32. `let Some(x) = ...` is currently a different blocker

Even if contextual resolution were perfect, this is explicitly rejected today:

```telos
let Some(x) = option
```

`statement.rs::validate_declaration_pattern` emits:

```rust
DiagnosticCode::DeclarationPatternRefutable
```

for:

```rust
Pattern::Variant(...)
```

with the message:

```text
variant patterns are refutable and cannot be used
in declaration destructuring
```

And `bind_declaration_pattern` has no variant-binding case.

So this feature has two separate requirements:

```text
A. Resolve Some contextually
B. Permit a refutable variant pattern in a declaration
```

A alone will not make the construct legal.

If Telos adopts the model we discussed:

```telos
let Some(x) = option
```

= **required destructuring**, then this needs a deliberate semantic/lowering amendment:

```text
evaluate RHS once
    ↓
check pattern
    ↓
success → bind
failure → required-pattern failure / exception
```

It should not be disguised as ordinary `if let`.

---

# 33. Lowering is already capable of carrying semantic variant identity

This is good news.

`phalcom-core/src/modules/semantic_lowering.rs` converts:

```rust
PatternResolution::Variant(...)
```

into:

```rust
ExecutablePattern::Variant {
    candidates: ...
}
```

using exact `VariantId`s from the semantic snapshot.

And `compile_if_let` / `compile_while_let` demand the corresponding semantic pattern-site lowering products.

So once the semantic checker resolves:

```telos
Some(x)
```

to:

```text
Option::Some
```

the lowering layer does not need to guess.

This is exactly what commit `55bf6a5e7c` was trying to enforce.

The architecture is therefore fundamentally sound:

```text
source shorthand
        ↓
semantic contextual resolution
        ↓
exact declaration-backed VariantId
        ↓
semantic lowering
        ↓
runtime pattern test
```

The missing work is in the semantic middle, not bytecode guessing.

---

# 34. Diagnostic requirements

The diagnostics should reflect how far resolution actually got.

### No expected owner

```telos
let x = None
```

Correct:

```text
cannot resolve `None`
```

Not:

```text
cannot infer enum owner of `None`
```

because the compiler has no independent basis for assuming variant intent.

Likewise:

```telos
let x = Some(1)
```

should use the normal unresolved callable diagnostic if nothing ordinary named `Some` exists.

---

### Expected owner exists, member missing

```telos
let x: Option<Int> = Nnoe
```

Now the compiler has an independently established contextual domain.

A diagnostic can say:

```text
cannot resolve `Nnoe` in the expected `Option<Int>` domain
```

or more specifically:

```text
`Option<Int>` has no associated variant `Nnoe`
```

depending on how variant/member namespaces are presented.

---

### Wrong selector shape

```telos
let x: Option<Int> = Some(1, 2)
```

Should say approximately:

```text
`Option<Int>` has no `Some` variant matching `Some(_, _)`
```

not merely unresolved name.

---

### Ambiguous owner set

Given:

```text
expected: Left | Right
```

with both defining matching `Same`:

```text
contextual variant `Same` is ambiguous

candidates:
    Left::Same
    Right::Same
```

No arbitrary selection.

---

### Generic underconstraint

```telos
let x: Option<_> = None
```

should report generic inference, because `Option::None` itself was successfully identified.

---

### GADT contradiction

If:

```telos
value: Expr<Int>
```

and contextual `BoolValue` can only produce:

```text
Expr<Bool>
```

the diagnostic should say the case cannot inhabit the expected specialization.

This should use the existing GADT/case machinery.

---

# 35. Testing requirements

The current tests cover contextual patterns better than contextual construction. The repaired implementation needs a matrix that proves every expected-type source independently.

| Context | Required regression |
|---|---|
| annotated initializer | `let x: Option<Int> = None` |
| annotated constructor | `let x: Option<Int> = Some(1)` |
| symbolic annotation | `let x: Option<_> = Some(1)` |
| symbolic singleton | `let x: Option<_> = None` → underconstrained generic |
| return contract | `fn f() -> Option<Int> { None }` |
| return constructor | `fn f() -> Option<Int> { Some(1) }` |
| assignment | `x = None` for `x: Option<Int>` |
| assignment constructor | `x = Some(1)` |
| callable parameter | `consume(Some(1))` |
| match | `Some(x)`, `None` |
| nested match | `Some(Ok(x))` |
| `if let` | `if let Some(x) = expr` |
| `while let` | `while let Some(x) = iterator.next` |
| generic iterator | associated `Iterable::Item` case from Universe |
| expected union | unique owner succeeds |
| expected union | two matching owners diagnose ambiguity |
| GADT | impossible contextual case rejected |
| no context | `let x = None` remains ordinary unresolved name |
| no context call | `let x = Some(1)` remains ordinary unresolved callable |
| lexical collision | ordinary lexical resolution wins before contextual fallback |
| explicit qualification | existing `Option::Some` behavior unchanged |
| lowering | shorthand produces exact same `VariantId` as qualified syntax |

I would make the qualified and contextual versions **semantic equivalence tests** where possible.

For example:

```text
Some(42) under Option<Int>
```

and:

```text
Option<Int>::Some(42)
```

should publish the same:

```text
VariantId
InvocationTargetId
specialized result type
argument mapping
```

That provides a much stronger invariant than merely “compiles.”

---

# 36. Universe-specific acceptance gate

After semantic tests are green, the repair should deliberately remove unnecessary qualifications from a small, high-signal slice of `packages/universe`.

I would use `functional/chain.ph` and `functional/folds.ph`.

The acceptance source should end up looking like:

```telos
while let Some(value) = iterator.next {
    ...
}
```

rather than:

```telos
while let Option::Some(value) = iterator.next {
    ...
}
```

and:

```telos
match f(value) {
    Ok(item) => ...
    Error(error) => ...
}
```

where the scrutinee establishes `Result<T,E>` unambiguously.

Then verify that those sites compile through semantic linking and executable lowering—not merely parsing.

Only after that proof should there be a wider corpus de-qualification.

---

# 37. What should **not** be done

Several tempting fixes would be architectural regressions:

1. **Do not restore global lookup by variant spelling.**
   `Some → Option` and `Ok → Result` was already correctly removed.

2. **Do not make capitalization semantic.**
   We explicitly rejected that, and nothing in the inference architecture requires it.

3. **Do not inject `Option`/`Result` special cases.**
   User-defined sums must behave exactly like Universe sums.

4. **Do not make `Expr::Var` independently scan `EnumSemanticTable`.**
   It should consume a contextual domain through a shared resolver.

5. **Do not implement one resolver for expressions and another for patterns.**
   Owner enumeration, selector filtering, specialization, and ambiguity rules need one authority.

6. **Do not use qualification as the permanent “fix.”**
   Qualification is the escape hatch, not the language's only usable constructor syntax.

7. **Do not treat unknown/blocked generic evidence as a false mismatch.**
   `Blocked`, `Refuted`, `Missing`, and `Ambiguous` must remain distinct outcomes.

---

# 38. Recommended implementation decomposition

I would implement this in four tightly ordered layers.

## Phase A — establish one contextual algebraic-domain abstraction

Primary areas:

```text
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/types/*
```

Requirements:

- canonical nominal owner extraction;
- union owner extraction;
- symbolic `InferenceTerm::Applied` owner extraction;
- retain specialization arguments;
- no global scanning;
- no spelling heuristics.

This establishes the common input representation.

---

## Phase B — centralize variant/member resolution

Primary areas:

```text
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/associated.rs
new contextual-variant helper/module if appropriate
```

Move common logic out of:

```text
try_resolve_contextual_singleton
resolve_variant_pattern
```

into a resolver that handles:

```text
singleton
exact constructor
selector pattern
whole family
```

and produces declaration-backed candidates.

Fix the union inconsistency here.

---

## Phase C — connect expressions to it

Primary area:

```text
phalcom-semantic/src/checker/expression.rs
```

Add contextual fallback after normal name resolution for:

```rust
Expr::Var
```

and:

```rust
synthesize_unqualified_call
```

Flow:

```text
ordinary lookup succeeds
    → use it

ordinary lookup fails
    +
expected algebraic domain exists
    → contextual variant lookup

neither
    → existing unresolved-name behavior
```

Once a constructor candidate is selected, reuse the existing:

```text
specialize associated member
CallableApplicationTarget
apply_resolved_callable
```

machinery.

Do not duplicate constructor inference.

---

## Phase D — generic pattern-domain robustness

Primary areas:

```text
expression.rs
coverage_subject_for_expression
types/rigid.rs
generic evidence / projection normalization
flow typing
```

Prove that:

```telos
while let Some(value) = iterator.next
```

continues to know its algebraic owner when `next` is derived from:

```text
Option<<I as Iterable>::Item>
```

and that mutable local flow does not unnecessarily erase:

```text
Chain<T>
Option<T>
Result<T,E>
```

This is the phase that should allow removing the explicit annotations that have accumulated in `Chain`.

---

# 39. Separate follow-on: required declaration patterns

After the above is solid, address:

```telos
let Some(x) = value
```

as a distinct language-semantic amendment.

Current blockers are explicitly:

```text
validate_declaration_pattern
    rejects Pattern::Variant

bind_declaration_pattern
    has no Variant implementation
```

If required destructuring is ratified, it should reuse the exact same contextual resolver and executable pattern semantics rather than introducing another variant implementation.

---

# 40. Final architectural model

The end state should be extremely simple to state:

```text
                     CONTEXT
                        │
       ┌────────────────┼─────────────────┐
       │                │                 │
 expected expr     scrutinee type    candidate param
    type                                type
       │                │                 │
       └────────────────┼─────────────────┘
                        ▼
             ContextualAlgebraicDomain
                        │
                        ▼
              owner candidate set
                        │
                        ▼
                EnumSemanticTable
                        │
                        ▼
          selector / family filtering
                        │
                        ▼
          specialization + GADT proof
                        │
             ┌──────────┼──────────┐
             │          │          │
          missing    ambiguous   resolved
                                   │
                    ┌──────────────┴────────────┐
                    │                           │
                 pattern                    expression
                    │                           │
             field projection          callable application
             coverage proof            argument inference
                    │                           │
                    └──────────────┬────────────┘
                                   ▼
                         exact VariantId
                                   │
                                   ▼
                         semantic lowering
```

There should be **one semantic notion of contextual variant identity**.

`match`, `if let`, `while let`, annotated expressions, returns, assignments, and callable arguments should merely be different sources of the context that feeds it.

---

# Conclusion

The current behavior is the consequence of an incomplete architectural transition.

The repository correctly moved away from an old, unsound model where variant identity could be guessed from names such as `Some` and `Ok`. It also built a strong declaration-backed variant model, associated-member specialization, exact-case typing, GADT-aware pattern resolution, and semantic lowering.

But contextual owner inference was completed primarily on the **pattern side**. The ordinary expression paths never acquired the corresponding bridge from:

```text
ExpectedType
```

to:

```text
expected enum/sum owner
```

to:

```text
variant lookup
```

Meanwhile, generic mutable-flow and associated-type inference remain weak enough in some Universe loops that even pattern inference can lose the domain it needs.

So I would characterize the repair as:

> **Complete the bidirectional checker by making algebraic owner context a first-class semantic input to variant resolution, then use that same resolver everywhere variants occur.**

The governing rule should be:

> **Ordinary resolution first. If and only if surrounding semantics independently provides an algebraic domain, unresolved names/calls may be resolved as members of that domain. No global variant search, no spelling heuristics, no capitalization semantics.**

Once that is implemented, `Option::Some` and `Result::Error` remain useful explicit disambiguation syntax rather than being required boilerplate throughout `packages/universe`.