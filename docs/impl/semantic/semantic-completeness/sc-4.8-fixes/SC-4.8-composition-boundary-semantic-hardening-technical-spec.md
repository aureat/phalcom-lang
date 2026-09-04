# SC-4.8 Composition-Boundary Semantic Hardening
## Technical Specification

**Project:** Phalcom  
**Subsystem:** `phalcom-semantic` typing / generic application / ADT-GADT elimination / local existential semantics  
**Baseline revision:** `e932aac4e21a5b346e719ede5a24f94e7b924ab3` (`feat(semantic): complete SC-4.8 typing integration`)  
**Document status:** Normative remediation specification  
**Purpose:** Complete SC-4.8 to full theoretical correctness at composition boundaries, preserve the successful SC-4.8 architecture, eliminate duplicated/incomplete structural logic, close the identified semantic defects, remove small avoidable allocations, and extend certification testing.

---

# 1. Executive Summary

SC-4.8 established the correct overall architecture for Phalcom's remaining type-system integration work:

- declaration-owned and callable-owned generic domains remain separate canonical products;
- ordinary generic application composes those domains only inside one query-local inference session;
- variant-local generic binders are callable-owned by deterministic variant-constructor callable identity;
- variant construction remains ordinary universal generic introduction;
- variant elimination opens constructor-local binders as fresh query-local rigid variables;
- rigid variables remain outside the canonical `TypeStore`;
- `TypeData::ExactCase { variant, enum_type }` remains the durable exact-case representation;
- applied receiver specialization remains separate from callable identity;
- generic getters, setters, index getters, index setters, constructors, variants, Families, and native surfaces reuse ordinary callable/generic machinery rather than acquiring feature-specific solvers.

Those decisions remain ratified.

The remediation specified here does **not** redesign that architecture. It completes the local existential/GADT calculus and makes the surrounding structural operations compositionally correct.

The central problem is that SC-4.8 introduced a second, query-local structural type representation (`LocalType`) and local rigid scope/proof machinery without yet making every composition boundary total over:

- nested pattern structure;
- proven GADT equalities;
- independently fresh rigid identities;
- lexical rigid scopes;
- record-row tails and non-`Type` kinds;
- type-lambda free captures;
- Family member types;
- generic application with local existential inputs;
- publication/escape boundaries;
- error recovery;
- source indexing;
- incremental comparison.

Accordingly, SC-4.8 shall remain classified as:

```text
SC-4.8 callable/generic integration:
    COMPLETE

SC-4.8 existential/GADT composition closure:
    REMEDIATION REQUIRED
```

until all requirements and acceptance gates in this document are satisfied.

After remediation, SC-4.8 may be restored to:

```text
SC-4.8 semantic implementation:
    COMPLETE
```

only under the stronger completion definition specified in §31.

---

# 2. Normative Language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

Where this document uses implementation sketches, names are illustrative unless explicitly marked as required API names. The semantic laws are authoritative; exact Rust naming may be adapted to fit the repository.

---

# 3. Preserved Architecture

The remediation MUST preserve the following SC-4.8 architectural decisions.

## 3.1 Canonical types remain rigid-free

`TypeStore` MUST NOT gain a durable rigid/skolem type variant.

Prohibited designs include:

```rust
TypeData::Rigid(...)
TypeData::GadtSkolem(...)
```

or any equivalent globally interned existential identity.

Branch-local existential identities are query-local semantic objects only.

## 3.2 `RigidArena` remains query-local

`RigidArena` remains the allocator and provenance store for:

- rigid scopes;
- rigid variables;
- rigid kind;
- rigid origin;
- lexical scope topology.

Compact integer IDs remain appropriate.

## 3.3 `CaseInstantiation` remains distinct from canonical case metadata

Construction/declaration metadata and elimination openings remain separate.

Canonical declaration products describe generic constructor signatures and GADT result equations.

`CaseInstantiation` describes **one fresh elimination opening**.

A fresh opening MUST NOT mutate canonical enum/variant metadata.

## 3.4 Exact cases remain canonical

The durable exact-case representation remains:

```rust
TypeData::ExactCase {
    variant,
    enum_type,
}
```

Hidden constructor-local variables are re-opened during elimination rather than stored in the exact case.

## 3.5 Generic ownership remains separated

Declaration-owned binders and callable-owned binders MUST NOT be merged into one publishable `GenericSignature`.

The implementation shall continue to model:

```text
declaration generic domain
+
callable generic domain
+
one query-local application solver
```

with stable owner-valid canonical metadata.

## 3.6 Variant generic ownership

Variant-local generic binders remain owned by a deterministic callable identity derived from exact `VariantId`.

No `TypeParameterOwner::Variant` is introduced.

The executable construction identity remains the variant-construction target rather than being conflated with callable semantic ownership.

## 3.7 Rigid variables remain opaque to ordinary inference

Ordinary inference MUST NOT choose an assignment for a rigid variable merely to satisfy a relation.

For example:

```text
κ ?= Int
```

does not permit ordinary inference to perform:

```text
κ := Int
```

However, §8 introduces the complementary rule that GADT evidence may **prove** `κ ≡ Int`.

---

# 4. Root Remediation Domains

The implementation work SHALL be organized around the following root problems rather than isolated symptom patches.

```text
R0  Branch-local GADT equality evidence and normalization
R1  Recursive LocalType propagation through pattern structure
R2  Alpha-equivalence and proof compatibility
R3  Operational lexical rigid scopes
R4  Local existential consumption versus scope-exiting publication
R5  Structural completeness of the local existential representation
R6  Authoritative exhaustive structural traversal
R7  Construction-side GADT equation solving
R8  Recovery and field-identity integrity
R9  Or-pattern existential joining
R10 Incremental binder-normalized semantics
R11 Source-index/tooling parity
R12 Durable generic variant metadata parity
R13 Accessor/index semantic conformance
R14 Allocation and traversal cleanup
R15 Extended semantic-law certification
```

Each individual code defect SHALL be mapped to one of these roots.

---

# 5. Core Semantic Laws

The remediation MUST make the following laws explicit and testable.

## SC48-HARD-01 — Rigid identity

A branch-local rigid is:

- fixed;
- unknown unless constrained by evidence;
- kinded;
- lexically scoped;
- query-local.

## SC48-HARD-02 — Allocation identity is non-semantic

Raw `RigidTypeVariableId` values have no semantic meaning across independent openings.

## SC48-HARD-03 — One binder, one rigid per opening

Within one constructor opening, each constructor-local generic binder maps to exactly one rigid identity, reused consistently across:

- payload types;
- result type;
- local constraints;
- branch proof.

## SC48-HARD-04 — Proven equality is not inference assignment

Ordinary inference:

```text
cannot guess κ := Int
```

but branch evidence may prove:

```text
κ ≡ Int
```

without mutating the rigid into an inference metavariable.

## SC48-HARD-05 — Recursive pattern correspondence

A binding introduced by a nested pattern receives the `LocalType` corresponding to its exact structural position.

Ancestor local types MUST NOT be bulk-applied to descendant bindings.

## SC48-HARD-06 — Proof-wide correlated alpha-renaming

Alpha-equivalence across independently opened existentials must preserve one-to-one binder correlation across the entire compared semantic product.

## SC48-HARD-07 — Scope-relative escape

Escape is defined relative to the lexical rigid scope being exited.

An outer rigid may cross an inner scope boundary.

An inner rigid may not cross the boundary that ends its own lifetime unless it is eliminated by proven equality or sound rigid-free widening.

## SC48-HARD-08 — Structural preservation

Canonical-to-local conversion MUST NOT erase semantic structure.

## SC48-HARD-09 — Kind preservation

Local existential handling MUST preserve generic kind.

In particular:

```text
Type
RecordRow
higher-kinded arrows
```

must not be conflated.

## SC48-HARD-10 — Recovery correspondence

Recovery may reduce type knowledge, but MUST NOT change:

- field position;
- field identity;
- binder correspondence;
- selector correspondence;
- declaration identity.

## SC48-HARD-11 — Durable products are local-variable free

No durable semantic product may contain:

- flexible inference variables;
- rigid variables;
- rigid scopes;
- branch-local proof state;
- solver-local row variables.

## SC48-HARD-12 — Complete type-form disposition

Every `TypeData` form SHALL have an explicit disposition in every correctness-sensitive structural operation.

## SC48-HARD-13 — Incremental semantics are binder-normalized

Cold versus incremental equality MUST NOT depend on rigid allocator order.

## SC48-HARD-14 — Local generic consumption is legal

Using a local existential as an argument to a polymorphic operation inside its live scope is not itself an existential escape.

## SC48-HARD-15 — No semantic laundering

`Dynamic`, `Unknown`, recovery, or failure fallback MUST NOT be used to hide:

- rigid conflicts;
- kind conflicts;
- GADT equality conflicts;
- publication violations;
- underconstrained generic applications.

## SC48-HARD-16 — Index setters return `Unit`

Both property setters and index setters have canonical language-level return type `Unit`.

## SC48-HARD-17 — Fast boolean rigid queries do not allocate

Operations such as “contains any rigid?” and “contains a rigid owned by scope S?” SHOULD run without constructing temporary sets.

---

# 6. Equality Model

A central correction is the explicit separation of three equality notions.

## 6.1 Identity equality

Identity equality answers whether two local terms refer to the same live witness.

Example:

```text
κ17 == κ17
```

This is relevant inside one query-local opening.

## 6.2 Alpha-equivalence

Alpha-equivalence answers whether two independently opened existential structures are the same modulo fresh binder renaming.

Example:

```text
F<κ17> ≡α F<κ42>
```

provided:

- kinds agree;
- binder correlation agrees;
- scope topology agrees.

## 6.3 Proof compatibility

Proof compatibility answers whether two branch proof environments can coexist or denote overlapping value space.

It is a proof-level relation, not ordinary `Eq` and not merely pairwise `LocalType::alpha_equivalent`.

The implementation SHALL expose these concepts distinctly.

---

# 7. R0 — Branch-Local Equality Evidence

## 7.1 Required semantic model

A rigid MUST remain immutable as an identity.

Established branch equality is stored externally.

Conceptually:

```rust
struct LocalProofEnvironment {
    // representation illustrative
    equalities: LocalEqualityState,
    constraints: Box<[LocalConstraint]>,
}
```

The environment SHALL be able to represent and entail:

```text
κ1 ≡ κ2
κ ≡ Int
κ ≡ List<Int>
List<κ> ≡ List<Int>
Pair<κ1, κ2> ≡ Pair<Int, String>
```

when those equalities are justified by branch evidence.

## 7.2 Equality authority

The local equality solver MUST be a bounded structural proof solver, not a general inference solver.

It SHALL support:

1. structural decomposition of identical type constructors;
2. rigid-rigid equivalence;
3. rigid-ground equivalence;
4. rigid-local-term equivalence where kind-correct;
5. contradiction detection;
6. kind validation;
7. recursive proof normalization;
8. correlation preservation.

It MUST NOT perform unrestricted higher-order unification.

## 7.3 Evidence sources

The local equality authority MAY learn equalities only from semantically justified evidence, including:

- GADT constructor result observation;
- canonical branch proof equalities;
- constructor-local `where` equivalence constraints;
- exact-case observation;
- already established local equality.

Ordinary argument assignability MUST NOT invent rigid equalities.

## 7.4 Example: concrete existential revelation

Given:

```phalcom
enum Expr<T> {
    @variant
    Wrap<U>(_ value: U) -> Expr<List<U>>
}
```

and:

```phalcom
eval(_ value: Expr<List<Int>>) -> Int {
    match value {
        Expr::Wrap(x) => x
    }
}
```

elimination creates:

```text
U ↦ κ

result observation:
    Expr<List<κ>> ≡ Expr<List<Int>>
```

which decomposes to:

```text
List<κ> ≡ List<Int>
κ ≡ Int
```

The branch is reachable.

The raw local type of `x` remains:

```text
κ
```

The effective type under proof is:

```text
Int
```

Using `x` where `Int` is required SHALL succeed.

## 7.5 Raw versus effective local type

The implementation SHOULD preserve the distinction:

```text
raw local type:
    κ

effective local type under proof:
    Int
```

Proof normalization MUST NOT rewrite query-local binding metadata destructively.

This preserves:

- witness correlation;
- provenance;
- diagnostics;
- proof reasoning.

## 7.6 Normalization

A proof-aware operation SHALL be available conceptually as:

```text
normalize_local(local_type, proof_environment)
```

It recursively replaces proven-equivalent local terms until a stable form is reached.

Normalization MUST:

- terminate;
- detect contradictory cycles;
- preserve kind;
- retain unresolved rigids.

---

# 8. R1 — Recursive Pattern Local Typing

## 8.1 Current defect

Pattern resolution recursively propagates canonical `TypeId`, while variant-local `LocalType` is attached after child recursion to every binding introduced below a variant field.

This can assign an ancestor structural type to a leaf binding.

Example:

```text
payload:
    (κ, Int)

pattern:
    (x, _)

incorrect:
    x : (κ, Int)

required:
    x : κ
```

## 8.2 Required resolver contract

Pattern recursion SHALL carry two expected-type channels:

```rust
canonical_expected: TypeId
local_expected: Option<&LocalType>
```

The exact Rust API MAY differ.

## 8.3 Binding rule

When resolving a name pattern:

```text
binding.local_type = local_expected.cloned()
```

subject to the appropriate joined/or-pattern semantics.

No ancestor bulk-assignment step is permitted.

## 8.4 Variant pattern rule

For each matched variant field:

1. derive canonical field type under declaration/GADT proof specialization;
2. obtain the corresponding local payload field type from `CaseInstantiation`;
3. pass both into the child pattern.

The post-recursion helper equivalent to:

```rust
attach_local_type_to_bindings(...)
```

SHALL be removed.

## 8.5 Tuple decomposition

If:

```text
canonical:
    (T, Int)

local:
    (κ, Int)
```

then tuple element recursion receives:

```text
element 0:
    canonical T
    local κ

element 1:
    canonical Int
    local Int
```

## 8.6 List decomposition

For local:

```text
List<κ>
```

a prefix element pattern sees:

```text
κ
```

A rest pattern sees:

```text
List<κ>
```

## 8.7 Record decomposition

For:

```text
#{ id: Int, value: κ | Rlocal }
```

the `value` child receives `κ`.

Record-row tail information remains attached to the record local type; it is not assigned to field-value bindings.

## 8.8 Nested variants

Nested variant opening SHALL create child existential scopes with correct parentage (§10) and recursively pass each child's local payload type.

---

# 9. R2 — Alpha-Equivalence and Proof Compatibility

## 9.1 Alpha-renaming context

A comparison session SHALL use one shared bijective mapping across the complete compared product.

Conceptually:

```rust
struct AlphaRenaming {
    left_to_right: ...,
    right_to_left: ...,
}
```

## 9.2 Kind requirement

A new rigid mapping:

```text
κleft ↔ κright
```

is valid only if:

```text
kind(κleft) == kind(κright)
```

## 9.3 Scope topology

Alpha-equivalence MUST preserve relative lexical scope structure.

A structure with one outer and one nested rigid is not alpha-equivalent to one where both binders belong to the same scope if that topology is semantically observable.

## 9.4 Origin metadata

`RigidOrigin` is provenance, not binder identity for alpha-equivalence.

Variant/case compatibility is checked by the containing semantic product.

Raw origin metadata SHOULD NOT by itself make two otherwise identical binder structures non-alpha-equivalent.

## 9.5 Correlation law

Required:

```text
Pair<κ1, κ1> ≡α Pair<κ7, κ7>
```

Rejected:

```text
Pair<κ1, κ1> ≡α Pair<κ7, κ8>
```

## 9.6 Proof merge

`merge_branch_proofs` SHALL NOT compare local proof terms using raw `LocalType` equality.

Proof merge SHALL use one proof-wide compatibility session across:

- `local_bindings`;
- `local_equalities`;
- all nested local structures.

Replacing:

```rust
left_type != right_type
```

with independent calls to `alpha_equivalent` is insufficient and SHALL NOT be considered a complete fix.

## 9.7 Pattern-space algebra

Pattern-space `intersect` and `subtract` MUST use proof compatibility that is stable under fresh rigid renaming.

Required laws:

```text
S ∩ α(S) = S
S \ α(S) = ∅
α(S) \ S = ∅
```

---

# 10. R3 — Operational Lexical Rigid Scopes

## 10.1 Existential frame stack

`CheckingContext` SHALL model live existential scopes explicitly.

Recommended conceptual form:

```rust
struct ExistentialFrame {
    scope: RigidScopeId,
    proof: LocalProofEnvironment,
}
```

with:

```rust
existential_frames: Vec<ExistentialFrame>
```

The exact storage layout may be adjusted.

## 10.2 Consolidation

The frame stack SHOULD replace scattered ownership of:

- current rigid scope;
- active local constraints;
- local equality evidence.

`active_local_constraints` SHOULD be folded into the lexical frame model rather than remaining independent state.

## 10.3 Nested opening

Opening a constructor in scope `Souter` MUST call:

```text
fresh_scope(Some(Souter))
```

rather than always opening with `None`.

## 10.4 Example

```text
scope S0
  outer constructor opens κ0

  scope S1(parent S0)
    inner constructor opens κ1
```

Leaving S1:

- `κ1` is leaving scope;
- `κ0` remains live.

## 10.5 Scope containment

`RigidArena::scope_contains` and `variable_in_scope` remain appropriate primitives.

Escape checking SHALL use scope topology rather than merely “contains any rigid.”

---

# 11. R4 — Local Consumption Versus Publication

## 11.1 Local generic use

Passing a live existential to an ordinary generic callable inside the same lexical scope is legal.

Example:

```phalcom
class Generic {
    @class
    id<T>(_ value: T) -> T { value }
}

match packed {
    Pack(x) => {
        let y = Generic.id(x)
    }
}
```

If:

```text
x : κ
```

the call may locally solve:

```text
T := κ
```

The result remains local:

```text
y : κ
```

No escape occurs merely because a call boundary was crossed.

## 11.2 Transient inference rigid atom

The generic inference representation SHALL be able to consume fixed local rigid atoms.

Recommended conceptual extension:

```rust
InferenceTerm::Rigid(RigidTypeVariableId)
```

or an equivalent explicit fixed-local atom.

This atom:

- is query-local;
- is never interned into `TypeStore`;
- is not an inference variable;
- may be assigned to a flexible metavariable;
- may not itself be assigned by ordinary inference.

## 11.3 Flexible-to-rigid solving

Allowed:

```text
α = κ
⇒ α := κ
```

Allowed:

```text
α = List<κ>
⇒ α := List<κ>
```

Forbidden without proof evidence:

```text
κ = Int
⇒ κ := Int
```

## 11.4 Publication is a separate operation

A call argument is not automatically a publication boundary.

The checker SHALL distinguish:

- local consumption;
- scope-preserving expression flow;
- scope-exiting publication;
- durable metadata publication.

---

# 12. Publication and Escape Algorithm

When a local type crosses a scope-exiting boundary, the checker SHALL apply the following conceptual algorithm.

## 12.1 Step 1 — Proof normalization

Normalize the local type using active proven equalities.

Example:

```text
κinner ≡ κouter
```

leaving the inner scope normalizes:

```text
κinner
→ κouter
```

which may remain legal.

## 12.2 Step 2 — Scope ownership check

Determine whether the normalized local type contains any rigid owned by:

- the scope being exited;
- optionally descendant scopes whose lifetime also ends at the boundary.

Rigids belonging only to ancestors remain legal.

## 12.3 Step 3 — Rigid-free materialization

If proof normalization eliminates every leaving-scope rigid, attempt canonical materialization.

Example:

```text
κ ≡ Int
```

permits publication as `Int`.

## 12.4 Step 4 — Sound widening

If unresolved leaving-scope rigids remain, the checker MAY publish a rigid-free expected/supertype only when soundly justified.

Example:

```text
κ <: Object
```

permits widening to `Object`.

## 12.5 Step 5 — Reject

If neither proof discharge nor sound widening removes the leaving-scope existential, emit `ExistentialEscape`.

## 12.6 Required three-way law

Given local `κ`:

```text
no equality evidence:
    publish as Int
    => REJECT

bound κ <: Object:
    publish as Object
    => ACCEPT

proof κ ≡ Int:
    publish as Int
    => ACCEPT
```

---

# 13. Publication-Boundary Inventory

The implementation plan MUST audit all relevant boundaries and classify each as:

```text
not a publication boundary
scope-preserving
scope-exiting
durable hard barrier
```

The audit SHALL include at least:

- explicit return;
- implicit return;
- match result;
- conditional result;
- outer local binding write;
- field write;
- declaration-pattern binding;
- call argument;
- call result;
- tuple construction;
- list construction;
- record construction;
- map/set construction where local type is relevant;
- closure capture;
- closure return;
- flow join;
- or-pattern join;
- exact-case reconstruction;
- associated/Family capture;
- metadata export;
- incremental snapshot/fingerprint product.

No new `check_local_type_escape` call should be added merely from intuition; each call site must be justified by the boundary inventory.

---

# 14. R5 — Complete Local Structural Representation

`LocalType` SHALL faithfully represent every canonical type form capable of containing local existential information, or the language SHALL explicitly reject that combination.

Silent opacity is prohibited.

## 14.1 Existing supported structural forms

The local calculus already needs structural treatment for:

- canonical leaf;
- rigid;
- applied type;
- exact case;
- union;
- tuple;
- record;
- callable.

## 14.2 Record rows

The existing form:

```rust
LocalType::Record(Box<[LocalRecordField]>)
```

is insufficient because it loses the canonical row tail.

The local representation SHALL include an explicit row tail.

Recommended form:

```rust
LocalType::Record {
    fields: Box<[LocalRecordField]>,
    tail: LocalRecordRowTail,
}

enum LocalRecordRowTail {
    Closed,
    Parameter(TypeParameterId),
    Rigid(RigidTypeVariableId),
}
```

The exact names may differ.

## 14.3 Row-kind safety

`LocalRecordRowTail::Rigid(r)` is valid only if:

```text
kind(r) == RecordRow
```

A `Type`-kind rigid SHALL NOT occupy a row tail.

## 14.4 Row round-trip

For rigid-free local types:

```text
materialize(localize(T)) = T
```

including:

```text
#{ field: Int | R }
```

with stable canonical `R : RecordRow`.

## 14.5 Constructor-local RecordRow

Variant-local row binders are ratified as supported.

Example:

```phalcom
@variant
Pack<R: RecordRow>(_ value: #{ id: Int | R })
```

elimination opens:

```text
R ↦ κR : RecordRow
```

and preserves:

```text
#{ id: Int | κR }
```

locally.

## 14.6 Generic kind support

Any generic kind legal for an ordinary generic binder SHALL be legal for a constructor-local binder.

Existential opening preserves the declared `KindId`.

This includes:

- `Type`;
- `RecordRow`;
- legal arrow kinds.

The local calculus SHALL NOT define a second, narrower generic kind system.

---

# 15. Type-Lambda Local Captures

## 15.1 Problem

A canonical type lambda may capture free canonical types.

If one such free type corresponds to a constructor-local binder, treating `TypeData::Lambda` as an opaque canonical leaf hides the rigid from:

- free-rigid traversal;
- escape checking;
- equality;
- alpha comparison.

## 15.2 Ratified representation direction

The remediation SHOULD represent a local lambda as:

```text
canonical lambda identity
+
localized free-capture overlay
```

rather than duplicating the complete scoped lambda AST.

Illustrative form:

```rust
LocalType::Lambda {
    lambda: TypeLambdaId,
    captures: LocalCaptureSubstitution,
}
```

## 15.3 Example

Canonical declaration:

```text
<X> =>> Pair<X, U>
```

after opening:

```text
U ↦ κ
```

shall become a local lambda view whose free capture for `U` is `κ`.

## 15.4 Traversal

Free-rigid traversal SHALL inspect local lambda captures.

## 15.5 Materialization

A local lambda may materialize only when all local captures materialize canonically.

## 15.6 Equality

Alpha comparison and proof normalization SHALL include local lambda capture structure while preserving lambda-bound variables through existing lambda identity/scoped representation.

---

# 16. Families Containing Local Types

## 16.1 Problem

Family members contain member types. Those member types may transitively contain constructor-local binders.

Treating the whole Family as an opaque canonical leaf can hide local existential structure.

## 16.2 Ratified semantic rule

Family member types MUST remain visible to:

- free-rigid traversal;
- local equality;
- alpha comparison;
- publication checks;
- materialization.

## 16.3 Representation

The implementation MAY choose either:

1. fully structural local Family members; or
2. canonical Family identity plus local member overlays.

The latter is preferred if it remains simple and avoids copying unchanged members.

## 16.4 Family member identity

Localizing a Family MUST NOT change:

- operation shape;
- member kind;
- invocation target identity;
- callable semantic identity.

Only type terms are localized.

---

# 17. R6 — Authoritative Structural Traversal

## 17.1 Problem statement

The current repository contains multiple recursive type walkers with differing coverage.

For example:

- one walker detects Family member type parameters but misses record-row tails;
- another detects row-tail parameters but may not inspect Family members.

This is a systemic correctness risk.

## 17.2 One authority per semantic question

The repository SHALL establish authoritative operations for at least:

```text
contains one parameter
contains any parameter from a set
visit canonical child types
visit parameter occurrences
canonical -> local conversion
free-rigid visitation
scope-rigid visitation
local materialization
alpha comparison
local proof normalization
```

## 17.3 Exhaustive handling

Correctness-sensitive operations SHOULD use exhaustive `match` over `TypeData`.

A new `TypeData` variant SHOULD cause compilation failure or explicit test failure until its semantic disposition is specified.

Avoid:

```rust
_ => false
```

for operations whose correctness depends on structural completeness.

## 17.4 Row-tail domain

A generic traversal API MUST NOT pretend `RecordRowTail::Parameter` is an ordinary child `TypeId`.

Row-tail parameter visitation shall remain domain-aware.

## 17.5 Family traversal

Parameter occurrence and child traversal MUST descend into Family member types where the operation's semantics require it.

## 17.6 Lambda traversal

Parameter/capture-sensitive operations MUST account for type-lambda free captures.

---

# 18. R7 — Construction-Side GADT Equations

## 18.1 Current defect

Explicitly applied generic variant construction and associated/Family specialization can compare fixed owner type arguments with GADT case types using mutual subtype checks before variant-local generic inference has solved the local binder.

Example:

```phalcom
Expr<List<Int>>::Wrap(1)
```

with:

```phalcom
Wrap<U>(_ value: U) -> Expr<List<U>>
```

must solve:

```text
T = List<Int>
U = Int
T ≡ List<U>
```

coherently.

## 18.2 Ratified rule

GADT case equations SHALL enter the same query-local application constraint session as:

- fixed declaration generics;
- callable-local generics;
- argument constraints;
- expected result constraints;
- generic bounds.

## 18.3 Equality, not mutual subtyping

A GADT result equation is an equality constraint.

It SHALL NOT be implemented as:

```text
A <: B && B <: A
```

when unsolved generic terms remain.

## 18.4 Ground fast path

An early contradiction check MAY run only after all participating terms are ground under already-fixed substitutions.

Example:

```text
Int ≡ String
```

may be rejected immediately.

Example:

```text
List<Int> ≡ List<U>
```

must enter generic solving while `U` is still inferable.

## 18.5 Direct and Family parity

The same equation semantics MUST apply to:

```text
Expr<List<Int>>::Wrap(1)
```

and:

```text
let wrap = Expr<List<Int>>::Wrap::*
wrap(1)
```

---

# 19. Residual Generic Domains for Captured Applied Families

## 19.1 Ratified rule

When fixed receiver/GADT evidence determines some callable-local generic binders, the captured Family member SHALL expose only the residual unsolved generic domain.

Example:

```text
original:
    ∀U. U -> Expr<List<U>>

applied owner:
    Expr<List<Int>>

owner equation proves:
    U ≡ Int
```

The effective captured callable type SHOULD be:

```text
Int -> Expr<List<Int>>
```

while retaining the stable underlying callable/construction identity.

## 19.2 Partial solving

If only some callable-local binders are determined, only those binders are removed.

Example:

```text
∀U, V
```

with fixed evidence proving only:

```text
U ≡ Int
```

leaves residual:

```text
∀V
```

## 19.3 Metadata

Residual specialization is a query-local/applied semantic view. It MUST NOT mutate the canonical declaration's original `GenericSignature`.

---

# 20. R8 — CaseInstantiation Payload Identity

## 20.1 Current defect

Using `filter_map` to construct `payload_types` can change cardinality and shift positional correspondence.

## 20.2 Ratified representation

Prefer ordered identity-preserving storage:

```rust
struct LocalPayloadField {
    field: VariantFieldId,
    ty: Option<LocalType>,
}

payload_fields: Box<[LocalPayloadField]>
```

## 20.3 Requirements

The product MUST preserve:

- declaration order;
- exact `VariantFieldId`;
- absence/unavailability of a local type;
- one entry per declared field.

## 20.4 Lookup

Pattern resolution SHOULD carry the field index/identity discovered during field matching rather than rescanning `variant.fields` later.

## 20.5 Recovery

A field with unavailable canonical type remains represented at its original position with `None` or equivalent recovery state.

Later fields MUST NOT shift.

---

# 21. R9 — Or-Pattern Existential Join

## 21.1 Problem

Independent alternatives may open alpha-equivalent existential witnesses that do not have shared identity.

Example:

```text
alternative A:
    x : κA

alternative B:
    x : κB
```

with `κA ≡α κB`.

## 21.2 Ratified conservative semantics

SC-4.8 remediation SHALL NOT introduce implicit existential repackaging at or-pattern joins.

The join SHALL retain a local existential type only if:

1. it is rigid-free; or
2. all alternatives refer to genuinely shared still-live rigid identities compatible under the same lexical environment.

Independently opened alpha-equivalent rigids do not satisfy condition 2.

## 21.3 Canonical knowledge

Failure to retain a local existential type does not imply losing all type knowledge.

The joined binding may retain the ordinary safe canonical join:

```text
local_type = None
knowledge = canonical join
```

where sound.

## 21.4 Future work

Fresh existential repackaging:

```text
κA | κB
→ ∃κjoin
```

is explicitly deferred as a future precision feature.

---

# 22. R10 — Binder-Normalized Incremental Semantics

## 22.1 Live alpha comparison

Live checker operations SHALL use an `AlphaRenaming`-style correlated comparison session.

## 22.2 Durable comparison

Cold/incremental parity SHOULD use a deterministic binder-normalized semantic form rather than raw IDs.

Conceptual normalization:

```text
scope 0:
    binder 0 : Type
    binder 1 : RecordRow

scope 1(parent = 0):
    binder 0 : Type
```

## 22.3 Non-semantic data

The following MUST NOT affect durable semantic equivalence:

- raw rigid allocator index;
- allocation order;
- query-local addresses.

## 22.4 Semantic data

Normalization MUST preserve:

- kind;
- scope topology;
- binder correlation;
- local proof structure required for semantic comparison.

## 22.5 Provenance

Source provenance such as `RigidOrigin` may be retained for diagnostics but SHOULD NOT become a durable equality discriminator unless a containing semantic identity requires it.

---

# 23. R11 — Source-Index and Tooling Parity

SC-4.8 added language-visible generic binders on syntax forms beyond ordinary methods.

The compiler-owned source index MUST understand those binders.

## 23.1 Generic accessors

Source type-reference collection and source-scope construction SHALL account for callable-local generics and `where` clauses on:

- getters;
- setters;
- index getters;
- index setters.

## 23.2 Enums and variants

Enum declarations SHALL no longer be skipped where source-index semantics require traversal.

The index SHALL process:

- enum generic parameters;
- variant-local generic parameters;
- variant payload type annotations;
- variant result type annotations;
- variant `where` clauses;
- enum behavior members;
- their generic binders and constraints.

## 23.3 Shadowing

A source generic binder MUST shadow a same-spelled nominal type during source reference resolution.

Example:

```phalcom
value<T> -> T
```

must treat `T` as the callable-local binder, not a declaration reference.

## 23.4 Scope

This is part of the completion remediation, but SHOULD be scheduled after the core proof/equality repairs.

---

# 24. R12 — Durable Variant Generic Metadata

## 24.1 Required audit

The implementation SHALL verify whether canonical variant-constructor generic metadata is fully represented in the durable metadata/export path.

## 24.2 Required exported semantics

If missing, export canonical declaration information sufficient to reconstruct the generic constructor contract:

- stable constructor callable identity;
- generic parameter list;
- parameter kinds;
- bounds/equality constraints;
- constructor parameter types;
- constructor result type;
- stable variant identity.

## 24.3 Forbidden exported state

Durable export MUST NOT contain:

- `RigidTypeVariableId`;
- `RigidScopeId`;
- `LocalType`;
- branch proof environment;
- local equality solver state.

## 24.4 Already-complete case

If the repository already exports the required canonical metadata, no redesign is needed; the remediation SHALL add parity/round-trip tests proving the path.

---

# 25. R13 — Accessor and Index Conformance

## 25.1 Property setter

Canonical property setter type:

```text
(T) -> Unit
```

unless future language design explicitly ratifies another rule.

## 25.2 Index setter

The newly ratified rule is:

```text
(IndexParameters..., Put) -> Unit
```

The declaration-signature builder MUST NOT use the put parameter type as `declared_return`.

## 25.3 Expression semantics

Index assignment expression synthesis must remain consistent with the callable semantic signature:

```text
result = Unit
```

## 25.4 Tests

Tests SHALL inspect:

- expression result;
- `CallableSemanticSignature.declared_return`;
- projected callable surface return;
- generic ownership;
- selector stability.

---

# 26. Existential Diagnostics

## 26.1 Provenance

`RigidOrigin::VariantParameter` already provides source-semantic provenance.

Diagnostics SHOULD report:

- constructor/variant;
- local generic binder name;
- escape boundary;
- outward attempted type;
- relevant proof/widening context where useful.

## 26.2 Boundary category

Escape checking SHOULD accept or derive an explicit boundary category, e.g.:

```text
Return
OuterAssignment
FieldWrite
MatchJoin
ClosureCapture
MetadataPublication
```

This improves consistency and testing.

## 26.3 Raw κ IDs

Raw `κN` identifiers MAY appear in debug details but SHOULD NOT be the primary user-facing explanation.

---

# 27. R14 — Performance and Allocation Cleanup

Correctness repairs take priority. The remediation SHALL additionally remove the small avoidable allocations and repeated traversals identified during review.

## 27.1 Non-allocating rigid predicates

Current boolean queries SHOULD NOT call:

```text
free_rigids() -> BTreeSet
```

first.

Provide operations conceptually equivalent to:

```rust
has_free_rigid()
contains_rigid_from_scope(...)
for_each_free_rigid(...)
```

Set allocation is reserved for callers that actually need a collection.

## 27.2 Reuse local replacement views

`CaseInstantiation` SHOULD avoid repeatedly reconstructing `HashMap<TypeParameterId, LocalType>` for the same opening.

Prefer:

- direct lookup over `local_rigids`;
- borrowed substitution view;
- compact dedicated local replacement object.

## 27.3 Or-pattern bindings

Avoid cloning complete alternative-binding vectors solely to form an “active alternatives” subset.

Use borrowed references/slices where possible.

## 27.4 Variant-field lookup

Carry field index and identity through pattern resolution.

Avoid repeated `.position(...)` scans over `variant.fields`.

## 27.5 Active proof/constraint aggregation

Escape/publication checking SHOULD consume active frame constraints through iterators/borrowed views where practical instead of cloning all active constraints for every check.

## 27.6 Alpha mapping reuse

Proof-wide comparison SHALL naturally reuse one alpha-renaming context, eliminating repeated per-term mapping allocations.

## 27.7 No speculative caches

Do NOT add persistent caches for:

- free-rigid sets;
- normalized local types;
- branch proof results;

without benchmark evidence.

The intended performance style remains:

```text
compact IDs
monotonic query-local arenas
canonical interning
transient proof state
minimal cloning
```

---

# 28. R15 — Testing Extension

The remediation test program is normative. Regression tests are not merely supporting evidence; they define the repaired semantic laws.

---

## 28.1 Test organization

Recommended organization:

```text
phalcom-semantic/tests/semantic/adts/
├── existentials.rs
├── exact_cases.rs
├── generics.rs
├── matching/
│   ├── gadt_refinement.rs
│   ├── nested_existentials.rs
│   ├── existential_equality.rs
│   ├── existential_or_patterns.rs
│   ├── pattern_space_alpha.rs
│   └── proof_merging.rs
├── scope/
│   ├── rigid_scope_nesting.rs
│   └── existential_escape.rs
├── integration/
│   ├── generic_application.rs
│   ├── record_rows.rs
│   ├── type_lambdas.rs
│   ├── families.rs
│   ├── applied_class_generics.rs
│   └── generic_accessors.rs
├── recovery/
│   ├── payload_identity.rs
│   └── malformed_patterns.rs
└── incremental/
    └── existential_semantics.rs
```

Exact placement MAY adapt to existing test-module naming, but concerns SHOULD remain separated.

---

# 29. Required Test Families

## 29.1 `SC48-EQ-*` — Proven local equality

### EQ-01 — Concrete GADT existential revelation

```phalcom
enum Expr<T> {
    @variant
    Wrap<U>(_ value: U) -> Expr<List<U>>
}

class Eval {
    @class
    eval(_ value: Expr<List<Int>>) -> Int {
        match value {
            Expr::Wrap(x) => x
        }
    }
}
```

Assert:

```text
branch reachable
raw local x = κ
proof entails κ ≡ Int
x usable as Int
return accepted
```

### EQ-02 — No guessing without evidence

A local `κ` passed to an `Int`-only sink with no GADT equality evidence remains rejected.

### EQ-03 — Rigid-rigid equality

GADT evidence may establish:

```text
κ1 ≡ κ2
```

without merging allocator identity.

### EQ-04 — Structural equality decomposition

```text
Pair<κ1, κ2> ≡ Pair<Int, String>
```

entails the corresponding component equalities.

### EQ-05 — Contradiction

Evidence leading to:

```text
κ ≡ Int
κ ≡ String
```

shall refute the branch when `Int` and `String` are not equal.

---

## 29.2 `SC48-NEST-*` — Recursive pattern local typing

```text
NEST-01 tuple leaf gets κ, not Tuple<κ, Int>
NEST-02 repeated tuple leaves preserve same κ identity
NEST-03 nested variant leaf receives inner rigid
NEST-04 deep nested tuple/application/variant structure
NEST-05 record field receives corresponding local field type
NEST-06 list element gets element local type
NEST-07 list rest gets local list type
```

---

## 29.3 `SC48-ALPHA-*` — Alpha-equivalence

```text
ALPHA-01 independently fresh openings are alpha-equivalent
ALPHA-02 repeated binder correlation preserved
ALPHA-03 one left binder cannot map to two right binders
ALPHA-04 two left binders cannot collapse to one right binder
ALPHA-05 Type-kind and RecordRow-kind rigids are not alpha-equivalent
ALPHA-06 nested scope topology preserved
ALPHA-07 reflexivity
ALPHA-08 symmetry
ALPHA-09 transitivity
```

Core laws:

```text
Pair<κ1, κ1> ≡α Pair<κ7, κ7>
Pair<κ1, κ1> ≢α Pair<κ7, κ8>
```

---

## 29.4 `SC48-SPACE-*` — Pattern-space algebra

```text
SPACE-01 S ∩ α(S) = S
SPACE-02 S \ α(S) = ∅
SPACE-03 α(S) \ S = ∅
SPACE-04 independently opened duplicate generic GADT arm is redundant
SPACE-05 genuinely different existential constraints remain distinguishable
```

---

## 29.5 `SC48-SCOPE-*` — Lexical rigid scopes

```text
SCOPE-01 outer rigid survives inner match boundary
SCOPE-02 inner rigid cannot escape inner match
SCOPE-03 sibling branch rigids are invisible to each other
SCOPE-04 inner scope parent is outer scope
SCOPE-05 three-level nesting exits independently
SCOPE-06 rigid-free values cross all lexical boundaries
SCOPE-07 κinner ≡ κouter normalizes to outer witness before inner exit
```

---

## 29.6 `SC48-LOCAL-GEN-*` — Local generic consumption

### LOCAL-GEN-01

```text
x : κ
id<T>(x)
```

Assert:

```text
T locally specializes to κ
result local type = κ
no ExistentialEscape
```

### LOCAL-GEN-02

```text
x : List<κ>
id<T>(x)
```

solves:

```text
T := List<κ>
```

### LOCAL-GEN-03

Ordinary inference cannot solve:

```text
κ := Int
```

merely to satisfy a concrete parameter.

### LOCAL-GEN-04

A local generic result containing a live rigid may continue within the same branch.

### LOCAL-GEN-05

That result is rejected when subsequently crossing the rigid's scope without discharge/widening.

---

## 29.7 `SC48-ESC-*` — Publication matrix

Boundary axis:

```text
explicit return
implicit return
outer assignment
field write
match join
conditional join
closure capture
aggregate publication
metadata publication
```

Type-shape axis:

```text
κ
List<κ>
Tuple<κ, Int>
Option<κ>
record containing κ
record with rigid row tail
callable containing κ
Family containing κ
type lambda capturing κ
deep nested composite
```

Expected classification:

```text
SAFE_LOCAL
SAFE_PROOF_DISCHARGE
SAFE_WIDENING
ILLEGAL_ESCAPE
```

Mandatory proof-discharge cases:

```text
κ ≡ Int -> publish as Int
κinner ≡ κouter -> inner exit legal
```

---

## 29.8 `SC48-ROW-*` — Record row integration

```text
ROW-01 declaration generic used only in row tail remains in effective domain
ROW-02 canonical -> local preserves open stable row tail
ROW-03 local -> canonical round-trip preserves rigid-free open tail
ROW-04 constructor-local RecordRow opens as row-kind rigid tail
ROW-05 Type rigid cannot occupy RecordRow tail
ROW-06 List<#{x:T | R}> traversal finds both T and R
ROW-07 row-tail-only generic participates in application underconstraint detection
ROW-08 alpha-equivalent row-tail rigids compare correctly
ROW-09 row-tail rigid escape is detected
```

---

## 29.9 `SC48-LAMBDA-*` — Type-lambda captures

```text
LAMBDA-01 constructor-local U captured free inside type lambda is localized
LAMBDA-02 free-rigid traversal sees the captured κ
LAMBDA-03 proof κ ≡ Int can discharge lambda capture
LAMBDA-04 unresolved captured κ cannot escape
LAMBDA-05 rigid-free lambda localization/materialization round-trips
```

---

## 29.10 `SC48-FAMILY-*` — Families

```text
FAMILY-01 Family member type containing local κ remains visible to rigid traversal
FAMILY-02 Family local type participates in alpha comparison
FAMILY-03 Family containing unresolved leaving-scope κ cannot escape
FAMILY-04 proof-discharge can materialize a rigid-free Family view
FAMILY-05 applied generic variant Family retains owner-derived GADT evidence
```

---

## 29.11 `SC48-CONSTRUCT-*` — Construction-side GADT equations

### CONSTRUCT-01

```phalcom
Expr<List<Int>>::Wrap(1)
```

Assert:

```text
T fixed as List<Int>
U inferred as Int
GADT equation T ≡ List<U> accepted
no AssociatedGadtOwnerConflict
```

### CONSTRUCT-02

Captured Family equivalent:

```phalcom
let wrap = Expr<List<Int>>::Wrap::*
wrap(1)
```

### CONSTRUCT-03

A truly ground contradiction still rejects.

### CONSTRUCT-04

GADT case equation is solved in the same application session as callable and declaration generics.

### CONSTRUCT-05

Residual callable domain removes owner-proven generic binders.

---

## 29.12 `SC48-REC-*` — Recovery integrity

```text
REC-01 missing field 0 type does not shift field 1
REC-02 malformed middle field preserves later field identity/index
REC-03 recovery preserves generic binder correspondence
REC-04 malformed nested pattern does not overwrite unaffected sibling local types
REC-05 Unknown/Dynamic recovery cannot erase a real existential contradiction
REC-06 payload recovery product retains exact VariantFieldId
```

---

## 29.13 `SC48-OR-*` — Or-pattern semantics

```text
OR-01 rigid-free local types join
OR-02 genuinely shared outer rigid may remain local
OR-03 independently opened alpha-equivalent rigids do not retain one alternative's witness
OR-04 correlation mismatch does not join locally
OR-05 kind mismatch does not join
OR-06 canonical common type remains available when local witness is discarded
```

---

## 29.14 `SC48-GENAPP-*` — Generic application composition

```text
GENAPP-01 applied receiver + callable generic + rigid argument
GENAPP-02 declaration generic only in row tail + callable generic
GENAPP-03 flexible α may solve to List<κ>
GENAPP-04 rigid κ is not solved by ordinary inference
GENAPP-05 generic setter accepts legal local existential consumption
GENAPP-06 generic index setter/getter follows same rule
GENAPP-07 Family invocation composes with local existential input
```

---

## 29.15 `SC48-INDEX-*` — Index setter conformance

```text
INDEX-01 canonical index-setter declared return is Unit
INDEX-02 projected callable return is Unit
INDEX-03 assignment expression result is Unit
INDEX-04 generic put binder remains callable-owned
INDEX-05 selector identity unchanged
```

---

## 29.16 `SC48-SOURCE-*` — Source-index parity

```text
SOURCE-01 generic getter binder shadows same-name nominal type
SOURCE-02 generic setter binder
SOURCE-03 generic index binder
SOURCE-04 accessor where-clause external nominal reference indexed
SOURCE-05 variant-local binder shadows nominal type
SOURCE-06 variant payload/result annotations indexed
SOURCE-07 variant where-clause external reference indexed
SOURCE-08 enum behavior generic members indexed
```

---

## 29.17 `SC48-META-*` — Metadata/export parity

```text
META-01 generic variant constructor exports stable callable identity
META-02 generic parameter kind exported
META-03 generic bounds/equalities exported
META-04 payload/result types exported
META-05 no rigid IDs or LocalType in durable export
META-06 cold/incremental metadata parity
```

---

## 29.18 `SC48-INCR-*` — Incremental semantics

```text
INCR-01 simple constructor-local existential cold/incremental parity
INCR-02 nested GADT pattern
INCR-03 proven κ ≡ Int equality
INCR-04 or-pattern
INCR-05 open row
INCR-06 type-lambda capture
INCR-07 Family local member
INCR-08 applied receiver + callable generic
INCR-09 body-only edit preserves binder-normalized semantics
INCR-10 constructor generic edit invalidates affected products
INCR-11 row-tail contract edit invalidates dependents
```

Raw rigid IDs MUST NOT be compared.

---

# 30. Property and Algebraic Tests

## 30.1 Strategy

Use deterministic bounded generators first.

No new property-testing dependency is required for the initial remediation.

Generate bounded `LocalType` structures and proof environments to validate laws.

## 30.2 Alpha laws

```text
T ≡α T
A ≡α B => B ≡α A
A ≡α B && B ≡α C => A ≡α C
T ≡α alphaRename(T)
```

## 30.3 Correlation law

Binder renaming MUST NOT merge distinct binders.

## 30.4 Kind law

Binder renaming MUST NOT cross kinds.

## 30.5 Scope law

Normalization MUST preserve valid ancestor-scope visibility.

## 30.6 Pattern-space laws

```text
S ∩ S = S
S \ S = ∅
S ∩ α(S) = S
S \ α(S) = ∅
```

## 30.7 Round-trip laws

For rigid-free localizable types:

```text
materialize(localize(T)) = T
```

including open rows and supported lambda/Family structure.

## 30.8 Recovery law

Recovery never changes declaration correspondence.

---

# 31. Required Code Areas

The implementation plan SHALL inspect and likely modify at least the following areas.

## 31.1 Local rigid/type representation

```text
phalcom-semantic/src/types/rigid.rs
```

Expected responsibilities after remediation:

- rigid arena;
- scope topology;
- local structural types;
- row-tail local representation;
- type-lambda/Family local overlays as needed;
- non-allocating rigid visitation;
- alpha-renaming primitives;
- binder-normalization support.

## 31.2 Case opening

```text
phalcom-semantic/src/types/case_instantiation.rs
```

Expected changes:

- parent scope propagation;
- identity-preserving payload products;
- local replacement view cleanup;
- complete structural localization.

## 31.3 Pattern recursion

```text
phalcom-semantic/src/checker/pattern.rs
```

Expected changes:

- canonical + local expected-type recursion;
- remove bulk descendant local-type attachment;
- proper nested existential scope opening;
- conservative or-pattern local join.

## 31.4 GADT proof engine

```text
phalcom-semantic/src/checker/gadt_proof.rs
```

Expected changes:

- branch-local equality proof solver;
- proof normalization;
- alpha-compatible proof merging;
- remove raw-rigid equality dependence.

## 31.5 Pattern-space algebra

```text
phalcom-semantic/src/checker/pattern_space.rs
```

Expected changes:

- consume proof compatibility;
- preserve alpha laws.

## 31.6 Checker context

```text
phalcom-semantic/src/checker/context.rs
```

Expected changes:

- existential frame stack;
- scope-relative publication guard;
- proof-aware normalization;
- constraint borrowing rather than repeated clones where practical;
- improved diagnostics.

## 31.7 Call application

```text
phalcom-semantic/src/checker/call.rs
```

Expected changes:

- local rigid atoms in transient inference/application terms;
- remove premature “call argument equals escape” behavior;
- authoritative parameter occurrence traversal;
- generic application integration with local terms.

## 31.8 Associated/variant construction

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
```

Expected changes:

- remove premature mutual-subtype GADT prechecks;
- send GADT case equations into the ordinary application solver;
- preserve direct/Family parity;
- residual generic specialization.

## 31.9 Canonical type traversal

Likely areas:

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/family.rs
```

Expected work:

- central exhaustive traversal helpers;
- row-tail coverage;
- Family member coverage;
- lambda free-capture disposition.

## 31.10 Declaration signatures

```text
phalcom-semantic/src/checker/declaration_signature.rs
```

Required correction:

```text
index setter declared return = Unit
```

## 31.11 Source index

```text
phalcom-semantic/src/source_index/builder.rs
```

Required extension:

- generic accessors;
- enums;
- variants;
- local generic binder scopes;
- `where` constraints.

## 31.12 Metadata/export

Audit and possibly modify:

```text
phalcom-semantic/src/metadata/
phalcom-semantic/src/export/
```

to ensure generic variant constructor metadata parity.

---

# 32. Structural Completeness Audit

Before implementation is declared complete, the engineer SHALL create a table covering every current `TypeData` variant and every relevant operation.

Required columns:

```text
TypeData variant
parameter traversal
canonical -> local
local rigid visibility
materialization
alpha comparison
proof normalization
publication behavior
```

At minimum enumerate:

```text
Never
Unit
ClassObject
Nominal
Applied
ExactCase
Union
Tuple
Record
Callable
Family
Parameter
Lambda
SelfType
```

Each cell must be one of:

```text
structurally preserved
canonical leaf by proof
explicitly unsupported with diagnostic
not applicable
```

No implicit catch-all disposition is accepted.

---

# 33. Recovery Audit

The remediation SHALL perform a similar inventory of recovery paths.

Normative law:

> Recovery may reduce knowledge, but never changes identity correspondence.

Audit at least:

- unresolved variant payload field type;
- malformed type annotation;
- malformed nested pattern;
- blocked generic inference;
- row materialization failure;
- associated lookup failure;
- unknown expected type;
- incremental stale/recomputed products.

Each recovery path must preserve stable identity/index mapping.

---

# 34. Performance Verification

The remediation is not required to introduce new optimization architecture.

It SHALL, however, benchmark the relevant semantic paths after correctness is restored.

Suggested benchmarks/fixtures:

```text
cold semantic analysis
no-op incremental analysis
large nested GADT match
many constructor-local binders
large or-pattern
open-row generic application
Family capture/invocation
large non-GADT source as regression control
```

Watch specifically:

- `CheckingContext` clones;
- proof-frame clones;
- `RigidArena` clones;
- repeated `LocalType` walks;
- repeated allocation of free-rigid sets;
- repeated field scans;
- generic-domain structural scans.

The acceptance criterion is no obvious regression attributable to avoidable new copying; exact numeric thresholds may be established after baseline measurement.

---

# 35. Migration and Compatibility

## 35.1 Canonical identities

The remediation MUST preserve:

- `TypeId` canonical semantics;
- `CallableId`;
- `VariantId`;
- `VariantFieldId`;
- selector identity;
- declaration generic ownership;
- callable generic ownership.

## 35.2 Runtime

No new runtime representation is required.

The work is semantic/static unless metadata parity exposes already-canonical generic constructor information.

## 35.3 Exact cases

No exact-case metadata migration is required.

## 35.4 Incremental database

If binder-normalized local comparison is added only to transient semantic comparison, no durable schema migration should be necessary.

Any persisted/fingerprinted representation changes must be documented explicitly.

---

# 36. Non-Goals

This remediation SHALL NOT expand into:

```text
general first-class existential packages
implicit existential repackaging at or-pattern joins
runtime existential boxes
runtime GADT proof witnesses
global rigid interning
TypeData::Rigid
monomorphization
per-applied-class runtime storage
rank-N polymorphism
a second generic solver
a constructor-specific mini-solver
a setter/indexer-specific generic solver
general higher-order local unification
```

---

# 37. Implementation Order Constraints

The later patch-grade implementation plan SHALL preserve the following dependency order.

## Phase A — Equality and local structural foundations

1. local proof equality authority;
2. proof normalization;
3. `LocalType` row/lambda/Family completeness;
4. non-allocating rigid traversal;
5. proof-wide alpha machinery.

## Phase B — Elimination correctness

6. lexical existential frame stack;
7. recursive pattern local typing;
8. identity-preserving `CaseInstantiation`;
9. alpha-compatible proof merge;
10. pattern-space laws;
11. proof-aware publication/escape.

## Phase C — Generic application composition

12. transient rigid atoms in inference terms;
13. local generic consumption;
14. construction-side GADT equation integration;
15. applied Family residual generic specialization.

## Phase D — Completeness and parity

16. authoritative canonical walkers;
17. index-setter `Unit` signature;
18. source-index parity;
19. metadata/export parity.

## Phase E — Cleanup and certification

20. allocation cleanup;
21. full extended testing;
22. incremental normalization tests;
23. benchmarks;
24. final gate execution.

The plan MAY split these further, but SHALL NOT implement later consumers before the foundational equality/scope model is stable.

---

# 38. Acceptance Gates

## 38.1 Focused semantic gates

All new test families in §29 MUST pass.

## 38.2 Existing semantic regressions

At minimum rerun:

```text
semantic::adts::generics
semantic::adts::existentials
semantic::adts::matching
semantic record-row suites
generic getter/setter/index suites
applied class-side suites
Family/associated suites
incremental semantic suites
```

## 38.3 Full semantic suite

```bash
cargo +stable test -p phalcom-semantic --test semantic
```

MUST pass with no new failure.

## 38.4 Core typing-integration regressions

Existing Monad and Either integration tests SHALL remain green.

## 38.5 Workspace check

```bash
cargo +stable check --workspace --all-targets
```

MUST pass.

## 38.6 Workspace fmt/clippy/test baselines

Known unrelated baseline failures MAY remain separately documented if still unrelated.

The remediation MUST NOT claim repository-wide release completion unless the repository-wide gates actually complete successfully.

## 38.7 Negative authority searches

The implementation SHALL verify absence of regressions such as:

```text
TypeData::Rigid
TypeParameterOwner::Variant
mixed-owner synthetic constructor GenericSignature
constructor-specific generic solver
setter/indexer-specific generic solver
```

---

# 39. Completion Definition

SC-4.8 may return to `COMPLETE` only when:

1. a rigid can be concretely known through GADT evidence without becoming an inference variable;
2. ordinary inference still cannot guess rigid identities;
3. nested patterns receive exact recursively decomposed local types;
4. branch proof comparison is stable under fresh alpha-renaming;
5. proof correlation and kind are preserved;
6. lexical rigid parent/child scopes are operational;
7. escape is relative to the actual exiting scope;
8. proven equality can discharge a local existential before publication;
9. bound-based rigid-free widening remains sound;
10. local generic calls may consume live existential values;
11. construction-side GADT equations participate in ordinary application solving;
12. direct and captured-Family variant application agree;
13. row tails survive canonical/local conversion;
14. constructor-local `RecordRow` binders work;
15. type-lambda free captures cannot hide rigids;
16. Family member types cannot hide rigids;
17. every relevant canonical type form has an explicit local-calculus disposition;
18. recovery never shifts payload correspondence;
19. or-pattern joins do not invent cross-alternative witness identity;
20. incremental comparison is independent of rigid allocation order;
21. generic accessors/variants have source-index parity;
22. durable generic variant metadata is either verified complete or completed;
23. index setters canonically return `Unit`;
24. avoidable allocations/duplicate scans identified in review are removed;
25. all extended and existing semantic gates pass.

The stronger completion criterion is:

> **Every canonical type form, generic kind, pattern form, branch proof relation, lexical scope transition, local generic-consumption path, and scope-exiting publication boundary reachable by SC-4.8 has an explicit and tested disposition for branch-local existential structure.**

This is the normative definition of full theoretical correctness at SC-4.8 composition boundaries.

---

# 40. Ratified Decisions Register

The following decisions are considered ratified for the implementation plan.

| ID | Decision | Ratified choice |
|---|---|---|
| D-01 | Proven rigid equality representation | External branch-local proof environment |
| D-02 | Local equality engine | Bounded structural equality proof solver |
| D-03 | Stored local binding after proof | Preserve raw rigid; normalize through proof |
| D-04 | Generic calls with local rigids | Transient rigid atom in inference terms |
| D-05 | Construction-side GADT equations | Same ordinary application constraint session |
| D-06 | Lexical existential state | Explicit existential frame stack |
| D-07 | Escape algorithm | Proof-normalize → scope-check → rigid-free widening |
| D-08 | Open row local representation | Explicit local record-row tail |
| D-09 | Constructor-local generic kinds | All kinds legal for ordinary generic binders |
| D-10 | Type-lambda local existential representation | Canonical lambda + localized free-capture overlay |
| D-11 | Family local existential visibility | Localized/overlaid member types; never opaque |
| D-12 | Owner-determined variant generics | Remove solved binders from residual applied view |
| D-13 | Case payload representation | Ordered field-identity product with optional local type |
| D-14 | Alpha semantics | Kind + correlation + scope topology; ignore raw IDs/provenance |
| D-15 | Incremental comparison | Binder-normalized local semantic form |
| D-16 | Or-pattern existential join | Conservative; no implicit repackaging |
| D-17 | Structural traversal architecture | Shared exhaustive targeted operations |
| D-18 | Source-index parity | Included in remediation |
| D-19 | Variant generic metadata | Canonical durable export / explicit parity audit |
| D-20 | Performance cleanup | Remove obvious allocations/scans; no speculative caches |
| D-21 | Property testing | Deterministic bounded generators first |
| D-22 | Index setter return | `Unit` |

---

# 41. Final Architectural Model

After remediation, the SC-4.8 semantic flow shall be understood as:

```text
                     CANONICAL DECLARATION WORLD
────────────────────────────────────────────────────────────────

GenericSignature(declaration-owned)
GenericSignature(callable-owned)
VariantInfo
CaseTypeEnvironment
TypeStore
ExactCase
Family metadata
Applied receiver

                    universal introduction
                             │
                             ▼

                   QUERY-LOCAL APPLICATION
────────────────────────────────────────────────────────────────

flexible inference variables
fixed receiver arguments
callable/declaration domains
GADT result equations
local rigid atoms when arguments come from active existential scopes

                             │
                             ▼

                    CANONICAL CONSTRUCTION
────────────────────────────────────────────────────────────────

canonical exact-case result
no rigid stored durably

                             │
                             ▼

                       ELIMINATION
────────────────────────────────────────────────────────────────

CaseInstantiation
fresh lexical rigid scope
U ↦ κ : Kind(U)
LocalType payload/result
LocalProofEnvironment

                             │
                    GADT observation/evidence
                             ▼

κ remains a rigid identity
proof may establish:
    κ ≡ Int
    κ1 ≡ κ2
    List<κ> ≡ List<Int>

                             │
                             ▼

                BRANCH-LOCAL TYPE CHECKING
────────────────────────────────────────────────────────────────

recursive local pattern typing
generic local consumption
proof-aware normalization
proof-aware relations
alpha-aware proof algebra

                             │
                     lexical scope exit
                             ▼

                  PUBLICATION / ESCAPE
────────────────────────────────────────────────────────────────

normalize under proof
        │
        ├── all leaving rigids discharged
        │       → materialize canonical type
        │
        ├── remaining rigids soundly widen
        │       → publish rigid-free canonical supertype
        │
        └── otherwise
                → ExistentialEscape

                             │
                             ▼

                       DURABLE WORLD
────────────────────────────────────────────────────────────────

TypeStore / metadata / incremental products
contain no local rigid identity or solver-local state
```

This model is the normative target of the SC-4.8 composition-boundary remediation.
