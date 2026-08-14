# Review Checklist and Agent Pressure Scenarios

## Purpose

Use these scenarios to test whether an agent can apply the skill under tempting shortcuts. A correct response must identify the exact relation/domain being confused, state the invariant, and propose a test or safer algorithm.

## Core review checklist

Before approving a type-theory change, verify:

- relation implemented is named precisely;
- current/proposed/future Phalcom status is labeled;
- type equality is not runtime class/layout equality;
- source annotation absence is preserved;
- inference variables never escape into reflective type identity;
- substitution uses semantic binder IDs;
- variance is validated through nested polarity;
- recursive relations terminate by memoization/guarded policy;
- recursive inference has SCC/fixed-point or annotation policy;
- union/intersection normalization is deterministic and only uses ratified stable laws;
- `Any`, `Dynamic`, analysis unknown, error recovery, and `Never` are distinct;
- protocol conformance does not alter ordinary dispatch;
- metatype/class-side semantics are handled explicitly;
- `Self` is not erased prematurely;
- runtime reification policy is separate from dispatch/layout;
- FFI/native trust/validation is explicit;
- constraints retain provenance;
- LSP `ValueShape` is not checker `Type`;
- proof unknown/timeout is not acceptance;
- caches state key, dependencies, invalidation, concurrency, and memory bound;
- every new positive rule has a nearby negative test.

## Scenario 1 — "Just use class IDs as types"

Task: add annotations for ordinary class names quickly.

Temptation:

```text
type Type = ClassId
```

Expected response: a nominal instance type can reference `ClassId`, but complete type domain needs protocols, type parameters, applied types, unions/intersections, callables, `Self`, special types, possibly metatypes/existentials. Keep runtime class identity as one constructor inside semantic type algebra.

Required negative test: `Option<Never>` or `A | B` cannot be represented by one `ClassId` without ad-hoc escape.

## Scenario 2 — typed overload by annotation

Task: two methods share selector surface but have different parameter annotations; make checker select one by types.

Expected response: reject as silent semantic change. Ordinary Phalcom selector identity/dispatch remains type-independent under current doctrine. If typed dispatch is desired, design explicit feature such as typecase/multimethod with runtime semantics, reflection, ambiguity rules, cache keys, and tests.

## Scenario 3 — unknown becomes bottom

Task: branch analysis cannot infer expression type; use `Never` to suppress later errors.

Expected response: `Never` means no normal value/path. Unknown means insufficient information. Using bottom makes unreachable proofs and joins unsound.

## Scenario 4 — unknown becomes `Dynamic`

Task: inference fails, so return `Dynamic` to keep editor/checker moving.

Expected response: explicit dynamic escape and inference failure are distinct. LSP may carry advisory unknown; checker must report underconstrained/blocked/unsupported according to mode rather than authorize arbitrary sends.

## Scenario 5 — mutable covariance

Task: make `List<Cat> <: List<Animal>`.

Expected response: demonstrate write counterexample `animals.add(Dog)`. Mutable read/write parameter is invariant unless writes are restricted/checked.

## Scenario 6 — method parameter covariance

Task: subclass override narrows `Animal` parameter to `Cat`.

Expected response: ordinary function subtyping requires contravariant parameters. A base-typed caller can pass `Dog`; narrowed override is unsafe unless dispatch/contract rules intentionally change.

## Scenario 7 — variance scanner only looks one level

Task: `out T` appears inside a callback parameter and reviewer says "T is in method parameter, so invalid".

Expected response: compute nested polarity through method parameter and callback parameter; two flips may make occurrence positive. Use compositional variance traversal.

## Scenario 8 — substitute by name

Task: implementation stores `{ "T" -> Int }`.

Expected response: nested generic method can shadow class `T`; substitution must use owner/index `TypeParamId`.

Test: `class C<T> { m<T>(x:T)->T }` under `C<Int>` leaves method `T` untouched.

## Scenario 9 — occurs check removed

Task: unifier hits `?α = List<?α>`; engineer disables occurs check because recursive types are planned.

Expected response: ordinary metavariable equation would create infinite substitution. Recursive types require explicit fixed-point/nominal representation; keep occurs check.

## Scenario 10 — finite constraint set treated as union bound

Task: `T in (A,B)` is implemented as `T <: A | B`.

Expected response: finite exact membership and upper bound differ; subtypes below `A` could satisfy union bound but not exact constraint. Follow current generic spec semantics.

## Scenario 11 — recursive protocol depth 20

Task: stop structural comparison after depth 20 and return true.

Expected response: arbitrary cutoff is neither sound nor complete. Use memoized obligation pairs and guarded coinductive cycle rule.

## Scenario 12 — every in-progress relation is true

Task: simplify recursive subtype cache: if `(A,B)` already active, return true.

Expected response: only justified under relation-specific guarded coinduction. Contravariant/unguarded cycles can be unsound. Track obligation kind/polarity/guardedness.

## Scenario 13 — isomorphism collapse

Task: canonicalize `Result<Unit,Never>` to `Unit`.

Expected response: they can be isomorphic in value content, but named constructor/reflection/API meaning may remain distinct. Only definitional equality rules may canonicalize.

## Scenario 14 — `Option` becomes nullable pointer

Task: VM optimization represents `None` as null pointer; checker models `T | null`.

Expected response: representation optimization must preserve semantic `Option<T>` sum/variant behavior and explicit handling. Runtime layout is not type algebra.

## Scenario 15 — `Dynamic` as top

Task: implement `T <: Dynamic` for all T and use regular transitive subtype engine.

Expected response: current normative core distinguishes `Dynamic` from safe top `Any`. Gradual consistency/acceptance needs separate relation; transitivity through Dynamic launders incompatibilities.

## Scenario 16 — missing annotation normalized to `Dynamic`

Task: reflection wants one field, so absent annotation stores Dynamic.

Expected response: source absence is semantically observable and current proposed design preserves `None`/absent metadata. Checker policy can later infer/treat boundary without rewriting source fact.

## Scenario 17 — class object typed as instance

Task: expression `Person` gets static type `Person` and class-side sends are looked up as instance members.

Expected response: distinguish class object/metatype from ordinary instance type, while preserving that `Person` descriptor also denotes a type expression in annotation/reflection context.

## Scenario 18 — erase `Self` to lexical class

Task: parser resolves `Self` inside `Animal.clone()->Self` to `Animal`.

Expected response: loses dynamic self behavior for subclasses/generic applications. Preserve binder-like Self until receiver/application context.

## Scenario 19 — binary method `Self` specialization

Task: inherited `merge(other: Self)` on `Animal` is specialized to `Cat.merge(Cat)` automatically.

Expected response: flag binary-method contravariance hazard. Base-typed caller may pass another Animal subtype. Requires explicit self-type semantics/restrictions.

## Scenario 20 — generic application clones class

Task: every `Box<Int>` creates fresh runtime subclass/member graph.

Expected response: semantic applied descriptor, member substitution, runtime class, code specialization, and class-side state are independent. Prefer canonical applied views/lazy substituted member views unless runtime design requires specialization.

## Scenario 21 — conformance cache no invalidation

Task: cache `(C,P)->false` forever.

Expected response: open member surfaces/module edits can make result stale. Key/dependencies need candidate/protocol generations and substitution/access context.

## Scenario 22 — source span in TypeId hash

Task: include annotation span so diagnostics can find source later.

Expected response: identical semantic type at two locations becomes unequal/non-shareable. Keep occurrence/provenance side table mapping span to canonical TypeId.

## Scenario 23 — LSP union cap copied into checker

Task: more than eight inferred types => `Unknown`, checker accepts.

Expected response: advisory analysis widening is not correctness type algebra. Checker needs exact/defined union policy or explicit failure/dynamic boundary.

## Scenario 24 — guarded pattern counted exhaustive

Task:

```text
Some(x) if x > 0
None
```

is considered exhaustive for `Option<Int>`.

Expected response: `Some(nonpositive)` remains uncovered unless guard is proven tautological over payload space.

## Scenario 25 — current subclasses counted exhaustive

Task: match `Cat` and `Dog` over open `Animal`; repo has no other subclasses.

Expected response: open-world hierarchy is not closed proof. Require wildcard or sealed hierarchy semantics.

## Scenario 26 — solver timeout means proof

Task: SMT solver times out on contract, no counterexample found; mark proven.

Expected response: proof status `Unknown(timeout)`. Lack of counterexample is not proof.

## Scenario 27 — typed runner test suite means program proven

Task: all tests pass in typed-runner mode; claim declared contracts verified.

Expected response: runtime checking covers executed paths. Static proof/checking of all paths is separate.

## Scenario 28 — `Result` used for throw type

Task: method that throws `IOError` gets result type `Result<T, IOError>` without source/API change.

Expected response: thrown error is control effect; `Result` is returned sum value. Converting is semantic transformation.

## Scenario 29 — non-local return checked against block result

Task: block inside method `foo()->String` does `return 42`; checker validates against block's local type.

Expected response: non-local return targets home method return context. Payload must satisfy home `String` contract.

## Scenario 30 — yield preserves field smart cast

Task: field refined to `Some`, fiber yields, another fiber may mutate it, then code uses value.

Expected response: refinement stability depends on concurrency/effect model. Yield can invalidate shared mutable field fact.

## Scenario 31 — arbitrary user object as authoritative type descriptor

Task: any object with `displayName`/`equivalentTo` accepted in bytecode metadata.

Expected response: type normalization must be pure/stable/trusted. Current proposed design restricts authoritative normalized metadata to recognized descriptor kinds; arbitrary user behavior can mutate/throw/nondeterministically define identity.

## Scenario 32 — Rust `TypeId` bridge

Task: store Rust `std::any::TypeId` as Phalcom `TypeId` for native packages.

Expected response: unrelated identity domains/semantics. Define explicit FFI adapter and metadata mapping.

## Scenario 33 — inferred type used as dispatch key

Task: optimizer/checker includes static `TypeId` in normal inline-cache key.

Expected response: ordinary dispatch is runtime receiver class/selector under current doctrine. Static type can guide guarded specialization but must not redefine dispatch identity.

## Scenario 34 — empty intersection from observation

Task: no known class conforms to `P & Q`; normalize to `Never`.

Expected response: open world. "No known inhabitant" is not "provably uninhabited".

## Scenario 35 — eager distributive normalization

Task: normalize all intersections of unions to full DNF for canonical equality.

Expected response: exponential blow-up. Use local algebraic normalization/relation reasoning unless advanced set-theoretic representation is deliberately chosen.

## Scenario 36 — cache proposed without validity policy

Task: "cache subtype results for speed".

Expected response must request/specify:

```text
key
value
validity condition
dependency set
invalidation event
concurrency/snapshot policy
memory bound
```

No cache design is complete without these.

## Scenario 37 — rename changes generic meaning

Task: type parameter renamed `T` to `U`; implementation considers all dependent type metadata new because name is identity.

Expected response: owner/index identity means spelling is descriptive; source revision may change spans/display but semantic binder identity within declaration-generation policy should be stable as specified.

## Scenario 38 — proof fact interned as type

Task: branch proves `x > 0`; create canonical global `PositiveInt` TypeId for each path automatically.

Expected response: keep proposition/refinement fact separate unless language explicitly has refinement types/canonical predicate logic. Path fact has program-point/mutation dependencies.

## Scenario 39 — type checker owns scopes

Task: checker walks AST and builds its own `HashMap<String,Type>`.

Expected response: reuse semantic binding/scope identities; type checker attaches type facts to resolved entities. A second scope engine will drift on shadowing/modules/recovery.

## Scenario 40 — incomplete editor source becomes valid complete semantics

Task: half-written `Map<Int,` recovered as raw `Map` type and diagnostics disappear.

Expected response: recovery can preserve partial node/facts for LSP, but complete program semantics still has formation error. Source recovery fact differs from valid partial type application.

## How to use these scenarios

For a new/modified skill version, sample at least one scenario from each family:

```text
relation separation
generics/substitution
variance/callables
recursion
metatypes/Self
gradual boundaries
ADT/exhaustiveness
proof/refinement
effects/control
representation/caching
Phalcom status/dispatch doctrine
```

If an agent answers only with terminology and cannot derive the rule/counterexample, deepen the relevant reference.
