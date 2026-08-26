# 06 — Phalcom Language Comparisons, Design Rationale, and Decision Audit

**Date:** 2026-08-23
**Revision:** post-Spec-01.5 canonical generic calculus; post-Spec-03.5 canonical native surface; post-Spec-04 source syntax; incorporates the reviewed 04.5 expression/inference/flow decisions and revised Spec 05
**Status:** Ratified engineering decision audit. Precedents explain decisions; they do not override normative Phalcom semantics or turn current implementation accidents into language rules.
**Authority:** rationale for decisions owned normatively by Specs 01, 01.5, 02, 03, 03.5, 04, reviewed 04.5 decisions, and revised 05. This document explains *why* those choices fit together. It does not create a parallel semantic specification.
**Primary owners:** language design, `phalcom-semantic`, compiler/runtime boundary in `phalcom-core`, metadata/reflection, `phalcom-lsp`, native semantic surface infrastructure, future proof platform
**Repository snapshot used for current-state observations:** `aureat/phalcom-lang@dd89c2f6f2021b0458e2a03e5bcb5ac5c0e7a3e2` (`main`, 2026-08-23)
**Verification note:** repository files on `main` were inspected. No fresh local build, REPL run, or test run is claimed by this document. The working checkout supplied for review contains dirty semantic changes that currently prevent a clean fresh REPL build, so runtime observations below are source/test-grounded rather than represented as newly executed behavior.
**Non-goals:** language survey for its own sake; compatibility promise with another language; implementation work plan; proof-backend selection; runtime object-model rewrite; new syntax; inventing semantics to explain current bugs

---

# 0. Revision contract

The previous Spec 06 was written before the latest ownership corrections in Spec 01.5, the revised Spec 05, the 03.5 native-surface convergence, and the reviewed 04.5 expression-typing decisions. This revision replaces the previous rationale wherever those documents or decisions disagree.

The central correction is methodological:

> **A design-rationale document must compare the intended semantic architecture, not rationalize whatever the current implementation happens to do.**

That distinction is especially important for runtime reflection. A semantic fact may be correct while its runtime projection is incomplete or wrong. Conversely, a runtime convenience API must not be generalized into semantic authority merely because it is implemented.

## 0.1 Superseded rationale

| Previous or tempting rationale | Revised decision |
|---|---|
| Ordinary higher-kinded programming requires prenex kind polymorphism | Ordinary HKTs use explicit arrow kinds and type lambdas from Spec 01.5. Kind polymorphism remains an optional gated extension in revised Spec 05. |
| `prenex kind schemes` are part of the normal generic foundation | Not required. Public kind variables/generalization remain gated. |
| Solver metavariables may live as canonical `TypeData::Infer` nodes | Rejected. `InferVarId != TypeId`; inference variables are query/session-local and must disappear before publication. |
| A monolithic mutable checker context is merely an implementation style | Rejected. Analysis environment, path-sensitive flow state, and inference-session state have different ownership/lifetimes and should be separate. |
| First inferred value of an unannotated mutable local can act as an implicit permanent annotation | Rejected. It is path-local flow knowledge. Sequential assignment replaces current knowledge; control-flow joins may form unions. |
| LSP flow analysis may remain an independent formal checker | Rejected. `phalcom-semantic` owns formal flow; LSP flow/shape logic becomes advisory during migration. |
| `for` needs a type-system-only iteration element rule | Rejected. Static typing follows the same `iterate(_)` / `iteratorValue(_)` protocol used by runtime/compiler lowering. |
| Source and native methods can use separate call-typing paths | Rejected. Implementation provenance differs; callable semantic typing does not. |
| Hand-maintained native semantic tables are an acceptable permanent source | Rejected. Generated canonical native surface is formal authority; legacy runtime installation may remain temporarily for migration only. |
| Proof assumptions are themselves a proof-trust tier | Rejected as a conflation. Assumptions are first-class inputs to an obligation; accepted proof evidence has a distinct trust classification such as `KernelChecked` or `TrustedBackend`. |
| `Never`, effects, exits, termination, contracts, and proof can share one callable correctness state | Rejected. Revised Spec 05 makes them independent semantic products. |
| A passed runtime contract is proof evidence | Rejected. Runtime enforcement and static proof are independent projections of one source contract. |
| A reflected `FunctionKind` means arbitrary standalone `Type -> Type` is constructible at runtime | Rejected. A `FunctionKind` is a reflected value produced from an actual semantic kind, e.g. `List.kind`. There is no public standalone arrow-kind constructor in the current runtime model. |
| `TypingContext.applyKind` is part of the current public API | Stale. The live public registration intentionally omits `applyKind`; kind descriptors are observational rather than a general user-constructed kind calculus. |
| Runtime `Behavior#kind` may hard-code a list of generic builtins | Rejected as architecture. The current hard-coded implementation causes `Option.kind` to be wrong even though semantic metadata correctly says `Option :: Type -> Type`. Runtime projection must be metadata-driven. |

## 0.2 Authority order

When evidence conflicts, use this order:

```text
normative semantic specification
        ↓
canonical compiler semantic metadata
        ↓
runtime metadata / reflection projection
        ↓
presentation
```

A defect in a lower layer does not rewrite a higher layer.

The current `Option.kind` mismatch is the canonical example:

```text
semantic declaration:
    Option :: Type -> Type

runtime projection today:
    Option.kind -> Type          // defect

correct conclusion:
    runtime projection is incomplete
```

not:

```text
Option is semantically nongeneric
```

## 0.3 What this revision deliberately does not do

This document does not:

- patch `behavior_kind`;
- add `applyKind`;
- make arrow kinds constructible from arbitrary runtime values;
- repair the dirty local checkout;
- claim a fresh REPL run;
- redefine Spec 03 merely to match an implementation omission;
- make 04.5 an advanced-semantics document;
- move ordinary generics back into Spec 05;
- select a proof solver or certificate format.

Those belong to their owning implementation/specification work.

---

# 1. Audit method

Every comparison in this document answers nine questions:

1. **Problem** — what design pressure exists in Phalcom?
2. **Alternatives** — what plausible choices could be made?
3. **Precedent** — what did another language or implementation demonstrate?
4. **Phalcom decision** — what is actually selected?
5. **Take** — which mechanism or lesson transfers directly?
6. **Adapt** — what must change to fit Phalcom's object/runtime model?
7. **Reject** — which attractive mechanism is intentionally not imported?
8. **Operational consequence** — what does the decision imply for runtime, compiler, LSP, metadata, diagnostics, and proof?
9. **Failure avoided** — what class of accidental complexity or unsoundness is excluded?

Precedent is evidence, not authority. A mature language may prove that a mechanism is viable without proving that its surrounding semantics fit Phalcom.

## 1.1 Evidence categories

This audit uses four categories deliberately:

**Ratified/normative design.** Fixed by the authoritative specification series.

**Observed current implementation.** Verified in current repository source. It may be incomplete or wrong relative to the target.

**Observed test requirement.** A test states a semantic invariant expected by the repository. A test is evidence of intended behavior, not proof that all runtime projections currently satisfy it.

**Architectural transfer.** A mechanism learned from another implementation—especially Pyrefly—adapted to Phalcom rather than copied semantically.

## 1.2 Repository defects are not precedents

A current implementation defect is useful architectural evidence if it demonstrates why a particular abstraction is wrong.

For example:

```rust
match class_name {
    "List" | "Set" => 1,
    "Map" => 2,
    _ => 0,
}
```

does not teach that four generic classes are special. It teaches that **reflection of declaration kinds must consume canonical declaration metadata instead of duplicating generic arity in runtime code**.

The bug is evidence for the architecture, not part of the architecture.

---

# 2. Protected Phalcom invariants

Any mechanism borrowed from another language must survive these invariants.

## 2.1 Message semantics remain runtime authority

The following remain ordinary runtime concepts:

```text
object
class
metaclass
selector
method dictionary
message send
super lookup
DNU
reflection
```

Static typing may predict, validate, summarize, or explain them. It does not silently replace them.

No type argument, inference variable, proof result, effect set, or annotation participates in selector identity.

## 2.2 Static semantics are compiler-owned

Canonical facts live in compiler semantic products:

```text
TypeId / proper-type forms
kind
generic signature
callable signature
flow facts
relations
effects
exits
termination
contracts
proof results
```

Runtime reflection and LSP are projections/consumers, not alternate semantic engines.

## 2.3 Canonical and temporary state are different ontological categories

Publishable semantic identity and temporary solving state must never share an ID domain merely for convenience.

Examples:

```text
TypeId          != InferVarId
RecordRowId     != RowVarId
stable binder   != solver metavariable
ProofArtifactId != backend process handle
```

This is a correctness boundary, not naming hygiene.

## 2.4 Unknown is not Dynamic

`Dynamic` is a semantic boundary intentionally admitting runtime uncertainty.

`Unknown` means the analyzer lacks sufficient information.

These are not interchangeable:

```text
opaque metadata         -> Unknown / blocked reason
explicit Dynamic value  -> Dynamic boundary
budget exhausted        -> BudgetExceeded
cancelled               -> Cancelled
invalid program         -> Invalid
```

The analyzer failing to know something never rewrites the user's program to `Dynamic`.

## 2.5 Runtime reflection observes; it does not legislate

A reflected `FunctionKind` is a runtime representation of a semantic arrow kind. It does not define kind semantics.

`List.kind` can legitimately produce a `FunctionKind` representing:

```text
Type -> Type
```

without implying that users can construct arbitrary standalone kind expressions as runtime values through `Type -> Type`, `applyKind`, or an `ArrowKind.new(...)` constructor.

## 2.6 Advanced callable products remain orthogonal

Revised Spec 05 establishes:

```text
normal return type
effect summary
exit summary
termination knowledge
contract set
proof evidence/status
```

as distinct products.

No convenience API or external precedent may collapse them.

---

# 3. Repository reality that constrains the rationale

This section records current implementation evidence so the rationale does not accidentally describe an already-fixed world.

## 3.1 `Option.kind` is currently wrong

Semantic tests require:

```text
Option :: Type -> Type
Some   :: Type -> Type
```

The runtime `Behavior#kind` implementation currently determines generic arity by runtime class name and recognizes only:

```text
List -> 1
Set  -> 1
Map  -> 2
```

Everything else gets zero parameters and therefore reflects `Type`.

Consequently, current runtime source implies:

```text
Option.kind -> Type
```

even while semantic metadata requires:

```text
Option :: Type -> Type
```

This is a projection bug.

### Rationale consequence

Generic arity/kind must not be re-authored in `phalcom-core` runtime reflection.

Correct architecture:

```text
canonical declaration metadata
          ↓
declaration kind
          ↓
runtime reification
          ↓
Type or FunctionKind descriptor
```

not:

```text
runtime class name
          ↓
hand-written switch
          ↓
guessed generic arity
```

The current bug is precisely the kind of drift Spec 03.5 exists to eliminate elsewhere.

## 3.2 `FunctionKind` is observational

Current runtime reification has a `FunctionKind` descriptor class for arrow kinds.

That means this is valid conceptually:

```text
List.kind
    -> reflected FunctionKind
    -> display "Type -> Type"
```

It does **not** imply:

```phalcom
const k = Type -> Type
```

is currently a runtime kind-construction expression.

The semantic language may write:

```text
F: Type -> Type
```

in a kind position. The runtime value surface is a separate matter.

This distinction prevents a common category error:

```text
semantic syntax exists
    ≠
the same syntax is executable as a runtime constructor
```

## 3.3 `applyKind` is intentionally absent from the live public API

The current `TypingContext` runtime primitive registration includes:

```text
apply
unionOf
tupleOf
recordOf
callable
equivalent
subtype
assignable
consistent
conforms
member
...
```

and does not register `applyKind`.

That omission is consistent with the current reflection posture:

> reflected kinds are inspectable values; arbitrary runtime kind calculus is not a public requirement.

Older reflection drafts mentioning `applyKind` should therefore not be cited as current public behavior.

Future kind-construction APIs, if motivated, require their own capability, semantics, budget, and use-case review rather than being inferred from the existence of `FunctionKind`.

## 3.4 Semantic truth already distinguishes constructor kinds

The semantic declaration tests explicitly distinguish:

```text
List   :: Type -> Type
Set    :: Type -> Type
Map    :: Type -> Type -> Type
Option :: Type -> Type
Some   :: Type -> Type
```

That is the correct semantic source of truth for runtime projection.

The reflection issue is not a missing type-theory decision. It is missing integration.

## 3.5 Canonical native surfaces have materially converged

The semantic workspace now imports generated canonical native surfaces instead of treating a hand-written `register_standard_surfaces` table as permanent formal truth.

This matters to the comparison document because it changes the rationale for native code:

```text
native implementation
    ≠
special type-system path
```

A native method may differ in:

```text
implementation provenance
intrinsic tag
trusted metadata source
ABI/runtime entry point
```

while sharing:

```text
selector identity
generic signature model
parameter/return type model
call resolution
argument checking
inference
diagnostics
```

with source methods.

## 3.6 Runtime native installation still has transitional duplication

Runtime installation may still retain legacy and descriptor paths during census/parity migration.

That does not create two formal native semantic authorities.

The distinction is:

```text
formal static authority:
    canonical generated surface

runtime migration detail:
    dual installer until descriptor coverage parity
```

This is a useful general lesson: migration duplication is tolerable when authority remains singular and deletion criteria are explicit.

---

# 4. Haskell and ML — kinds, constructors, local inference, and the limit of transfer

## Problem

Phalcom needs:

- generic type constructors;
- higher-kinded parameters;
- type lambdas;
- reusable generic algorithms;
- useful local inference;
- predictable checking and diagnostics.

It does not want every program to participate in global principal-type inference or a dependent type-level programming language.

## Alternatives

1. atomic `Type` only, no constructor kinds;
2. explicit arrow kinds such as `Type -> Type`;
3. mandatory kind polymorphism;
4. `Type :: Type`;
5. universe levels;
6. dependent kinds/types;
7. global Hindley–Milner-style inference;
8. local/bidirectional inference around published signatures.

## Precedent

ML/Haskell traditions establish important facts:

- constructor kinds are useful;
- substitution and unification should be formal rather than ad hoc;
- inference variables require occurs checks and disciplined scoping;
- polymorphic generalization has specific safe boundaries;
- higher-order type constructors are practical;
- type-level expressiveness directly increases elaboration complexity.

They also show that global inference and powerful type-level computation can become architectural commitments rather than isolated features.

## Phalcom decision

**Ratified/normative design.**

Ordinary generic programming uses:

```text
Type
Type -> Type
Type -> Type -> Type
```

and source type lambdas:

```phalcom
<T> =>> Result<T, Error>
```

Higher-kinded parameters are explicit:

```phalcom
class Functor<F: Type -> Type> { ... }
```

Ordinary method generic inference is local/bidirectional.

Public kind polymorphism is **not** required for this system and remains gated in revised Spec 05.

### Critical correction from the old rationale

Do not say:

> Phalcom's ordinary HKT foundation uses prenex kind schemes.

Say:

> Phalcom's ordinary HKT foundation uses explicit monomorphic arrow kinds and type lambdas. Optional future kind polymorphism may quantify over kinds after a separate ratification gate.

## Take

- constructor/proper-type distinction;
- explicit kind checking before type relation checking;
- local constraint solving;
- capture-avoiding substitution;
- occurs checks;
- zonking/materialization before publication;
- bidirectional checking;
- partial-correctness versus total-correctness distinction.

## Adapt

Global inference becomes local callable/body inference.

The key judgment shape is:

```text
Γ ⊢ expression ⇒ TypeKnowledge
Γ ⊢ expression ⇐ ExpectedType
```

Expected types may constrain literals, blocks, method-generic arguments, and return expressions, but inference does not own public API design.

## Reject

- `Type :: Type`;
- universe polymorphism as a default;
- dependent escalation;
- arbitrary type-level runtime evaluation;
- whole-workspace principal-type inference;
- publishing unsolved metavariables;
- kind polymorphism merely because HKTs exist.

## Runtime consequence

The runtime need not evaluate kind expressions.

A semantic declaration may have kind:

```text
Type -> Type
```

and runtime reflection may *observe* that as a `FunctionKind`.

No runtime `applyKind` or standalone arrow-kind constructor is required.

## Failure avoided

Phalcom obtains HKT expressiveness without committing every typing operation to:

- global generalization;
- runtime type-level evaluation;
- escaping metavariables;
- a universe hierarchy;
- an unnecessarily powerful kind solver.

---

# 5. Scala — variance and HKTs without ambient type-directed machinery

## Problem

Phalcom wants concise variance and higher-kinded abstractions while preserving a message-oriented runtime whose dispatch meaning is not silently altered by static inference.

## Alternatives

- invariant generics only;
- declaration-site variance;
- use-site variance;
- keyword variance (`out T`, `in T`);
- sign variance (`+T`, `-T`);
- placeholder HKTs such as `F<_>`;
- explicit kind annotations;
- implicit/given/type-class search;
- type-directed overload or dispatch selection.

## Precedent

Scala demonstrates that declaration-site variance and higher-kinded abstractions can be ergonomic in a nominal OO language.

It also demonstrates the complexity that emerges when:

- implicit/given search;
- path-dependent types;
- type members;
- context resolution;
- overload resolution

interact.

## Phalcom decision

Use:

```phalcom
+T
-T
T
```

on nominal declaration parameters.

Use explicit constructor kinds:

```phalcom
F: Type -> Type
```

Use type lambdas:

```phalcom
<T> =>> Result<T, Error>
```

Use `where` for constraints:

```phalcom
where T <: Number
where T == U
where Number <: T
```

Reject:

```phalcom
F<_>
```

as a second HKT notation.

## Take

- concise sign variance;
- compositional variance validation;
- explicit constructor-kinded parameter patterns;
- F-bound caution;
- higher-kinded library design patterns.

## Adapt

Phalcom's `where` constraints are signature-owned semantic relations, not per-parameter bags.

The source surface does not import Scala's ambient contextual resolution.

## Reject

- `out`/`in` spellings;
- `F<_>` placeholders;
- implicit/given search as hidden call input;
- path-dependent type identity;
- static type information changing selector identity;
- overload resolution that causes inference to choose a different runtime method.

## Operational consequence

Type inference specializes a selected semantic callable; it does not choose a differently named/encoded selector.

A method send is conceptually:

```text
runtime selector
      +
static candidate semantic surface
      +
local generic inference
```

not:

```text
type inference
      -> invent/select runtime selector
```

## Failure avoided

Generic expressiveness cannot silently fork Phalcom into two call semantics: one for typed code and another for ordinary message sends.

---

# 6. OCaml — rows, domain separation, and generalization discipline

## Problem

Open structural records require row-like solving. It is tempting to invent one universal `Row` abstraction and reuse it for records, variants, and effects.

## Alternatives

- closed records only;
- one universal public row domain;
- independent row domains sharing implementation utilities;
- nominal class layouts treated as structural rows;
- unrestricted generalization;
- conservative boundary-based generalization.

## Precedent

OCaml demonstrates:

- practical row polymorphism;
- open structural shapes;
- subtraction/unification techniques;
- importance of occurs checks;
- value/generalization restrictions around effects and mutation.

## Phalcom decision

Revised Spec 05 owns a distinct:

```text
RecordRow
```

domain.

Future:

```text
EffectRow
VariantRow
```

may share generic solver algorithms, but they are not the same semantic IDs or policies.

Record rows describe structural record values.

They do not expose nominal class field layout.

## Take

- open/closed tail distinction;
- sorted/canonical known labels;
- row-variable occurs checks;
- field subtraction;
- lacks constraints where needed;
- caution around generalizing mutable/effectful facts.

## Adapt

Stable row binders reuse stable generic owner/index identities when appropriate, but query-local row variables remain a distinct ID domain.

## Reject

- one public `Row` kind;
- record/effect/variant rows sharing untyped IDs;
- structural reflection of class object layout;
- publishing incomplete row solver state;
- treating mutation flow facts as globally generalized signatures.

## Operational consequence

Row solving can be sophisticated without contaminating canonical type identity with query-local row metavariables.

## Failure avoided

An implementation utility cannot accidentally make:

```text
record field
effect atom
variant case
```

interchangeable merely because each happens to be labeled.

---

# 7. Rust — stable identities, solver-local variables, obligations, and trust boundaries

## Problem

Phalcom needs rigorous identity and proof boundaries without adopting Rust's runtime or borrow semantics.

## Alternatives

- identify generic parameters by name;
- identify by owner/index;
- put inference variables in canonical `TypeStore`;
- separate solver-local IDs;
- represent constraints as ad hoc booleans;
- represent obligations/results explicitly;
- treat solver/backend output as trusted;
- distinguish proof evidence/trust and assumptions.

## Precedent

Rust demonstrates the implementation value of:

- owner-scoped generic identity;
- explicit obligations;
- compiler-local inference variables;
- disciplined substitution;
- clear trusted/unsafe boundaries;
- refusing to make textual names semantic identity.

It also demonstrates how monomorphization and ownership semantics are broader language commitments that should not be copied accidentally.

## Phalcom decision

Stable generic binders use semantic owner + index.

Solver variables are not canonical types:

```text
InferVarId != TypeId
```

`TypeData::Infer` is transitional and is deleted after all inference paths use `InferenceSession`.

Proof acceptance distinguishes evidence trust from assumptions.

Initial proof evidence trust includes categories such as:

```text
KernelChecked
TrustedBackend
```

while assumptions remain first-class obligation inputs:

```text
preconditions
class invariants
callee contracts
native semantic declarations
closed-world assumptions
arithmetic model
world/version assumptions
```

A proof that uses assumptions is still a proof *under those assumptions*; “assumed” is not a substitute for evidence trust.

## Take

- owner-qualified IDs;
- newtypes for distinct identity domains;
- explicit obligation/result state;
- publishability validation;
- visible trust boundary;
- deterministic metadata/evidence fingerprints.

## Adapt

Phalcom generic specialization remains semantic metadata, not runtime class monomorphization.

## Reject

- Rust borrow checking as the default Phalcom effect model;
- specialized runtime class identity for generic instantiations;
- trait coherence/orphan rules before Phalcom protocol/coherence semantics exist;
- inference variables interned as publishable canonical type nodes;
- backend success silently granting proof authority.

## Operational consequence

The canonical store can make a strong promise:

> Every canonical published type form is meaningful independent of one local inference session.

That promise simplifies:

- snapshots;
- metadata;
- reflection;
- hashing;
- incremental equality;
- cross-process artifacts;
- diagnostics.

## Failure avoided

No ephemeral solver variable can escape merely because it happened to occupy the same enum as stable semantic types.

---

# 8. Smalltalk — runtime coherence, flow typing, and why first assignment is not a declaration

## Problem

Phalcom's runtime tradition is message-oriented and live. Static analysis should add knowledge without rewriting that runtime into a conventional nominally statically typed language.

This pressure is especially visible for mutable locals.

## Alternatives for an unannotated mutable binding

Given:

```phalcom
let x = 1
x = "hello"
```

possible semantics include:

1. infer `Int` once and reject later assignment;
2. infer an invisible permanent union `Int | String`;
3. treat the current path fact as changing from `Int` to `String`;
4. treat all unannotated mutable locals as `Dynamic`.

## Phalcom decision

Use flow knowledge, not hidden declaration.

Straight line:

```phalcom
let x = 1
// x : Int here

x = "hello"
// x : String here
```

At a join:

```phalcom
let x = 1

if condition {
    x = "hello"
}

use(x)
// x : Int | String here
```

An explicit annotation is different:

```phalcom
let x: Number = 1
```

`Number` is a persistent declared constraint; current flow knowledge may be more precise.

## Precedent

Smalltalk's core lesson is not “everything must be dynamically typed.” It is that runtime object/message semantics should remain coherent and late-bound rather than acquiring invisible compile-time state that changes meaning.

Flow-sensitive static knowledge can coexist with that runtime.

## Take

- preserve runtime message identity;
- classes remain ordinary runtime objects;
- DNU/open-world behavior remains real;
- static analysis reports what it knows rather than pretending runtime closure;
- mutable values can change runtime classes naturally.

## Adapt

Phalcom adds path-sensitive compiler knowledge on top:

```text
binding declaration constraint
        +
current flow fact
        +
branch refinement
```

## Reject

- first assignment silently becoming an annotation;
- every mutable local degrading immediately to `Dynamic`;
- static analysis closing the world around DNU/reflection;
- ambient generic state on frames/fibers;
- specialized generic runtime class objects.

## Operational consequence

`FlowState` must be a first-class checker product rather than hidden mutation inside a monolithic `CheckingContext`.

## Failure avoided

Inference does not smuggle a new source-language rule into code that never wrote an annotation.

---

# 9. Swift, Java, Kotlin, and C# — reification is a cost model, not a purity test

## Problem

Generic information can be:

- erased;
- partially retained;
- stored per instance;
- specialized into runtime types;
- reified only under explicit contexts.

Each choice changes performance, identity, and reflection.

## Precedent

Java exposes costs/limits of erasure.

Kotlin shows targeted reification can be useful without making all objects carry all generic arguments.

C# shows richer runtime generic metadata and specialized runtime machinery.

Swift shows careful distinction among values, types, metatypes, and runtime metadata.

## Phalcom decision

Ordinary objects carry no mandatory per-instance generic token.

Generic applications do not create new runtime class/metaclass identities.

Nominal reification returns the existing class object.

Synthetic forms use descriptors only when explicitly reflected/materialized.

`TypingContext` carries explicit reflection/construction context.

## Take

- truthful APIs about what generic information exists;
- versioned runtime metadata;
- explicit reification boundaries;
- declaration-site variance lessons;
- careful metatype/type distinction.

## Adapt

Phalcom keeps:

```text
value.class
value : T
T :: K
reified type descriptor
```

separate.

## Reject

- mandatory generic token in every instance;
- one new class object per `List<Int>`;
- pretending erased generic information can be reconstructed from arbitrary values;
- strong immortal descriptor caches.

## Runtime consequence

Adding typing metadata should not change:

```text
ordinary object size
selector
method table
class result
allocation layout
```

## Failure avoided

Typing does not impose universal runtime tax merely to make reflection maximally convenient.

---

# 10. TypeScript — flow precision without permissive `any`

## Problem

Gradual/dynamic boundaries and editor responsiveness encourage broad compatibility relations and “best effort” fallback types.

## Alternatives

- one `compatible(A,B)` relation;
- separate equivalence/subtyping/assignability/consistency;
- use `Dynamic` when analysis gives up;
- distinguish explicit Dynamic from epistemic failure;
- let editor analysis define semantics;
- make compiler semantics authoritative.

## Precedent

TypeScript demonstrates the practical value of:

- structural typing;
- union-based flow narrowing;
- excellent editor feedback;
- incremental analysis.

It also demonstrates trade-offs from an intentionally permissive `any` and compatibility-oriented design.

## Phalcom decision

Keep separate:

```text
equivalent
subtype
assignable
consistent
conforms
```

and separate:

```text
Dynamic
Unknown
Missing
Invalid
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

The LSP consumes compiler-published formal facts.

## Take

- flow-sensitive narrowing;
- useful union formation at joins;
- structural record ergonomics;
- fast editor feedback;
- dynamic escape boundaries as explicit language features.

## Adapt

Flow facts are compiler-owned and snapshot/query scoped.

## Reject

- one permissive compatibility relation;
- analysis failure becoming `Dynamic`;
- editor convenience overriding compiler semantics;
- treating erased/unknown facts as runtime reflection truth;
- importing TypeScript's `any` semantics.

## Failure avoided

A timeout, opaque native, or missing annotation cannot silently validate an otherwise unverified program.

---

# 11. Python and Pyrefly — transfer the architecture, not Python's semantics

## Problem

Phalcom needs a fast incremental analyzer for:

- whole projects;
- recursive definitions;
- live editor updates;
- flow analysis;
- generic inference;
- diagnostics;
- proofs/effects later.

A naive whole-workspace recomputation would be correct but increasingly expensive.

## Precedent

The Pyrefly transfer analysis provides implementation evidence around:

- dense identities;
- staged semantic products;
- query/answer cells;
- recursive publication;
- dependency fingerprints;
- reverse invalidation;
- immutable generation snapshots;
- cancellation;
- bounded work;
- observability;
- differential incremental testing.

## Phalcom decision

Transfer those execution ideas into `phalcom-semantic`.

Do not transfer:

- Python `Any`;
- Python attribute/descriptor semantics;
- overload resolution rules;
- Python import fallback semantics;
- Python protocol assumptions;
- Python's runtime object model.

## 11.1 Body-local expression identity

Current AST nodes do not carry stable global node IDs.

04.5 therefore assigns deterministic expression IDs within one body analysis product:

```rust
struct ExpressionId {
    body: BodyId,
    local: LocalExpressionId,
}
```

These IDs key:

```text
type fact
flow fact
call resolution
inference provenance
diagnostic derivation
```

They are not promised stable after arbitrary text edits.

### Why this fits the query architecture

Incremental identity should be attached at the level with semantic stability:

```text
CallableId / BodyId
```

The body query may recompute local expression numbering after a structural edit.

This is preferable to pretending source-order `#17` is globally persistent identity.

## 11.2 Compiler-owned formal flow

Current LSP flow work contains valuable algorithms and presentation knowledge.

The target is not to throw that work away blindly.

Migration rule:

```text
phalcom-semantic:
    formal flow truth

phalcom-lsp:
    request scheduling
    rendering
    advisory shape heuristics
```

During migration, both may coexist for parity testing.

No new formal rejection rule belongs in the LSP copy.

## 11.3 Inference sessions are local query state

The same architecture motivates:

```text
InferenceSession
```

instead of canonical `TypeData::Infer`.

An inference session owns:

- fresh metavariables;
- constraints;
- union-find/representatives;
- lower/upper bounds;
- expected-type constraints;
- provenance;
- budgets;
- cancellation.

It publishes only solved/canonical forms or explicit non-success outcomes.

## Take

- staged queries;
- dependency fingerprints;
- reverse invalidation;
- bounded recursion/fixed points;
- immutable snapshots;
- clean-vs-incremental differential tests;
- cancellation and stale-result rejection;
- performance metrics.

## Reject

- copying Python type rules;
- LSP as independent formal semantic owner;
- pointer-tag tricks before profiling/evidence;
- complexity widening to `Dynamic`.

## Failure avoided

Performance architecture improves without importing a second language's type semantics.

---

# 12. Effect systems — composition without turning effects into control-flow truth

## Problem

Phalcom wants useful static effect summaries for:

- mutation;
- I/O;
- scheduling;
- reflection;
- nondeterminism;
- blocking;

while preserving a separate account of exceptional/control exits and termination.

## Alternatives

- no effect analysis;
- one boolean `pure`;
- a small closed effect-set domain;
- immediate public effect rows/handlers;
- encode exceptions/divergence as effects;
- keep effects and exits separate.

## Precedent

Effect systems across ML-family languages and modern research languages demonstrate that effect summaries can make local reasoning, APIs, optimization, and diagnostics much more informative.

They also demonstrate that effect polymorphism/handlers are a language feature in their own right, not merely a nicer internal set representation.

## Phalcom decision

Start with an explicit, bounded effect-summary domain as revised Spec 05 specifies.

Keep separate:

```text
effects
exits
termination
```

This is valid:

```text
@total + IO
@total + Mutation
@total + raises Error
pure + non-total
```

## Take

- compositional effect summaries;
- known-versus-opaque effect knowledge;
- call-graph/SCC propagation;
- future effect-variable extension seams.

## Adapt

Current native `EffectSpec`, `RaisesSpec`, and `ReturnFlowSpec` are authoritative inputs where declared, but they are adapted into compiler-owned semantic summaries.

## Reject

- `pure` implying total;
- empty effects implying no raise;
- `Never` encoding divergence;
- effect rows/handlers before their source/operational semantics are separately ratified;
- missing effect metadata meaning pure.

## Failure avoided

A convenient summary field cannot become a false proof about termination or control flow.

---

# 13. Proof-oriented languages and tools — evidence without dependent-programming takeover

## Problem

Phalcom contracts should support both runtime enforcement and static verification without transforming ordinary Phalcom APIs into a dependent proof language.

## Alternatives

- runtime guards only;
- ephemeral solver verdicts;
- persistent fingerprinted proof evidence;
- proof terms in ordinary user values/types;
- trusted backend;
- local kernel checking;
- assumptions treated as proof trust;
- assumptions modeled separately.

## Precedent

Refinement/proof systems demonstrate:

- canonical obligations;
- verification conditions;
- counterexamples;
- explicit assumptions;
- trust/certificate boundaries;
- partial vs total correctness;
- value of persistent evidence.

They also show how quickly proof terms/dependent types reshape the entire language.

## Phalcom decision

One source contract gets one canonical semantic identity and two consumers:

```text
source contract
      │
      ▼
canonical ContractDecl
      ├──> runtime guard lowering
      └──> proof / VC lowering
```

Runtime success is not proof.

A proof result has explicit terminal state:

```text
Proven
Disproven
Unknown
Cancelled
BudgetExceeded
InternalFailure
```

Accepted evidence records trust such as:

```text
KernelChecked
TrustedBackend
```

Assumptions are recorded independently.

## Take

- VC generation;
- counterexample models;
- explicit trust;
- persistent fingerprints;
- partial/total correctness distinction;
- dependency invalidation;
- unsupported theory becoming reasoned unknown.

## Adapt

Phalcom proofs are side products keyed to ordinary callable/declaration identities.

They do not become dispatch inputs or runtime class identity.

## Reject

- `Prop`/proof terms as a default public kind;
- Curry–Howard-style proof programming as an implicit consequence;
- runtime guard success as theorem;
- solver process exit code as sufficient trusted evidence;
- assumptions hidden inside an undifferentiated “trusted proof” label;
- proof-guided selector dispatch.

## Failure avoided

The verification platform can grow independently of the core object model and can state exactly what was proven, under which assumptions, by which trust route.

---

# 14. Direct alternative decisions

This section is the fast audit surface. Each decision names the tempting alternative and why Phalcom chooses otherwise.

## 14.1 `Type` atomic kind versus `Type :: Type`

**Decision:** `Type` is an atomic kind. `Type.kind` is not introduced to create a self-classifying universe.

**Reason:** `Type :: Type` collapses a useful stratification and creates normalization/consistency obligations Phalcom does not need.

**Runtime implication:** the runtime `Type` singleton being an object does not imply static `Type :: Type`.

---

## 14.2 Explicit arrow kinds versus mandatory kind polymorphism

**Decision:** ordinary HKTs use:

```text
Type -> Type
```

and type lambdas.

Kind polymorphism remains gated.

**Reason:** HKT use cases do not require quantifying over kinds.

---

## 14.3 Reflected `FunctionKind` versus standalone runtime arrow-kind construction

**Decision:** a reflected constructor kind may materialize as `FunctionKind`, but there is no requirement for arbitrary runtime syntax/construction:

```text
Type -> Type
```

as an independently evaluated runtime constructor.

**Reason:** observation and construction are separate capabilities.

**Current implementation:** `reify_kind` can produce `FunctionKind`; `TypingContext` does not expose `applyKind`.

---

## 14.4 Metadata-driven declaration kinds versus runtime class-name switches

**Decision:** `Behavior#kind` must derive from canonical declaration metadata.

**Reject:** hard-coded:

```text
List -> 1
Set -> 1
Map -> 2
else -> 0
```

**Evidence:** this currently misprojects `Option`, whose semantic test requires `Type -> Type`.

**Failure avoided:** every newly generic core/user declaration requiring a runtime code edit.

---

## 14.5 Canonical inference node versus solver-local inference variable

**Decision:**

```text
InferVarId != TypeId
```

`TypeData::Infer` is transitional and deleted after migration.

**Reason:** a temporary question is not a canonical type.

---

## 14.6 Monolithic `CheckingContext` versus analysis context + flow state + inference session

**Decision:**

```text
BodyAnalysisContext
FlowState
InferenceSession
```

are distinct.

**Reason:** they have different mutation, lifetime, branching, and caching semantics.

---

## 14.7 Global inference versus local/bidirectional inference

**Decision:** infer within bounded local problems around declared/published surfaces.

**Reason:** preserves predictable API boundaries and incremental dependencies.

---

## 14.8 First assignment freezes a local versus flow-sensitive knowledge

**Decision:** no hidden annotation.

Straight-line assignment replaces current fact.

Branch/loop joins combine predecessor facts.

Explicit annotation remains a persistent constraint.

---

## 14.9 Global AST node identity versus body-local `ExpressionId`

**Decision:** deterministic body-local IDs until AST owns stable IDs.

**Reason:** avoids pretending source-order numbering survives arbitrary edits.

---

## 14.10 Magic `for` element type versus iteration protocol typing

**Decision:** type the real protocol:

```text
iterate(_)
iteratorValue(_)
```

Element knowledge comes from the specialized `iteratorValue` result.

Insufficient metadata yields `Unknown`, not `Dynamic`.

---

## 14.11 Separate native/source call checkers versus one callable algorithm

**Decision:** implementation provenance is orthogonal to signature semantics.

Call algorithm conceptually:

```text
type receiver
   ↓
member lookup
   ↓
specialize owner environment
   ↓
instantiate method generics
   ↓
collect argument/expected constraints
   ↓
solve
   ↓
check
   ↓
specialize result
```

The same path applies to source and native callables.

---

## 14.12 Handwritten native semantic catalog versus generated canonical surface

**Decision:** authored native declarations generate the VM-free semantic surface.

**Reason:** duplicated catalogs drift.

**Current migration:** legacy runtime installer may coexist until census parity, but formal semantic authority is singular.

---

## 14.13 LSP formal checker versus compiler-owned formal flow

**Decision:** `phalcom-semantic` is formal authority.

LSP may keep advisory facts during migration.

**Reason:** compiler and editor must not disagree on what program is valid.

---

## 14.14 Flattened diagnostic strings versus structured semantic derivation

**Decision:** checker owns diagnostic truth and provenance; renderer owns presentation.

Semantic record should retain:

```text
code
severity
primary/secondary spans
expected/found
reason chain
notes/help
fixes
```

**Reason:** CLI, LSP, JSON, code actions, and explanations need the same facts.

---

## 14.15 One compatibility predicate versus distinct relations

**Decision:** keep equivalence, subtyping, assignability, consistency, and conformance distinct.

**Reason:** each answers a different semantic question.

---

## 14.16 Unknown versus Dynamic

**Decision:** never conflate.

**Reason:** `Dynamic` is a user/program semantic boundary; `Unknown` is analyzer knowledge state.

---

## 14.17 Universal row domain versus `RecordRow`

**Decision:** separate row domains.

**Reason:** record labels, effects, and variants obey different laws.

---

## 14.18 `Never` as divergence versus normal-return bottom

**Decision:** `Never` says no normal returned value.

It does not say why.

---

## 14.19 Total-by-default versus explicit totality

**Decision:** partial correctness is ordinary default; totality is separately proven/required.

**Reason:** recursion, blocking, dynamic calls, FFI, and loops make universal termination assumptions unsound.

---

## 14.20 Effects, exits, and termination collapsed versus independent products

**Decision:** independent.

**Reason:** all combinations can be semantically meaningful.

---

## 14.21 Runtime guard versus proof

**Decision:** independent.

A runtime guard checks one execution; static proof establishes an obligation under modeled assumptions.

---

## 14.22 Assumptions versus evidence trust

**Decision:** assumptions are obligation inputs; evidence trust describes why the proof result is accepted.

Do not encode:

```text
Assumed
```

as if it were equivalent to:

```text
KernelChecked
TrustedBackend
```

An axiom may be an assumption in the proof. The trust route for the proof evidence remains separately visible.

---

## 14.23 Wrapper around every class versus direct nominal reification

**Decision:** a nominal type form reifies as its existing class object.

Synthetic forms use descriptors.

**Reason:** avoids duplicate nominal identity and wrapper cache complexity.

---

## 14.24 Strong global descriptor cache versus weak context-owned cache

**Decision:** bounded weak canonicalization.

**Reason:** avoids immortal synthetic descriptor graphs and stale cross-world retention.

---

## 14.25 `Type.currentApplication` versus explicit `TypingContext`

**Decision:** explicit context.

**Reason:** ambient generic application is ambiguous under fibers, reentrancy, reflection, and callbacks.

---

## 14.26 Specialized runtime generic classes versus erased ordinary runtime classes

**Decision:** no runtime `List<Int>` class identity.

**Reason:** static application must not multiply class/metaclass graphs or alter `value.class`.

---

## 14.27 Boolean proof verdict versus reasoned proof-result algebra

**Decision:** rich terminal states.

**Reason:** timeout, unsupported theory, internal failure, counterexample, and proof are not booleans.

---

## 14.28 Human diagnostic layout versus stable semantic diagnostic contract

**Decision:** human layout may evolve; semantic code/status/spans/reasons remain structured.

**Reason:** attractive CLI diagnostics and machine tooling need different projections of the same truth.

---

# 15. Cross-decision consequences

The architecture is strongest when viewed as a set of reinforcing decisions rather than isolated preferences.

## 15.1 Solver-local inference protects every downstream representation

```text
InferVarId != TypeId
        ↓
canonical TypeStore contains publishable forms only
        ↓
SemanticSnapshot can be immutable/stable
        ↓
metadata export cannot accidentally serialize metavariables
        ↓
runtime reflection never sees one
        ↓
cross-run fingerprints remain meaningful
```

Keeping `TypeData::Infer` because it is convenient would weaken every layer below it.

## 15.2 Runtime message invariance constrains call inference

```text
selector identity fixed by source/runtime
        ↓
semantic member candidate located
        ↓
generic parameters instantiated locally
        ↓
arguments/expected type constrain substitutions
        ↓
result specialized
```

Inference is therefore *inside* the static description of one runtime send, not a mechanism that changes which message is sent.

## 15.3 Checker decomposition enables true flow semantics

```text
BodyAnalysisContext
        +
FlowState
        +
InferenceSession
        ↓
sequential assignment updates current fact
        ↓
branches fork states
        ↓
joins combine reachable facts
        ↓
loops compute bounded fixed point
        ↓
formal facts published by phalcom-semantic
        ↓
LSP presents the same facts
```

A monolithic context obscures which mutation is semantic environment mutation versus path evolution.

## 15.4 Canonical native surface completes source/native convergence

```text
#[primitive(...)] authored truth
        ↓
generated native surface
        ↓
canonical callable signature
        ↓
same member/call/inference engine
        ↓
same diagnostic model
```

Only implementation provenance remains different.

## 15.5 `Option.kind` demonstrates why reflection must project metadata

Correct desired chain:

```text
Option declaration
    :: Type -> Type
        ↓
semantic metadata
        ↓
runtime reify kind
        ↓
FunctionKind(Type -> Type)
```

Current buggy chain:

```text
Option class name
        ↓
not in hard-coded switch
        ↓
parameter_count = 0
        ↓
Type
```

The lesson generalizes: runtime reflection should not duplicate type-system declaration knowledge.

## 15.6 Advanced callable facts compose without merging identities

```text
CallableId
   ├── signature / return type
   ├── effects
   ├── exits
   ├── termination
   ├── contracts
   └── proof results
```

This preserves independent:

- query budgets;
- invalidation;
- metadata profiles;
- reflection access;
- proof trust;
- runtime cost.

---

# 16. Failure patterns this design explicitly guards against

## 16.1 “One enum is simpler”

Putting:

```text
canonical type
inference variable
unknown
dynamic
invalid
```

into one convenient enum may make local code shorter.

It destroys semantic distinctions at boundaries.

## 16.2 “Reflection already has it, so reflection defines it”

A runtime descriptor may be missing, stale, stripped, or buggy.

Semantic identity cannot depend on its existence.

`Option.kind` is a direct warning against this inversion.

## 16.3 “The editor knows enough”

Editor heuristics are valuable but cannot become independent program validity rules.

## 16.4 “Use Dynamic when precision is hard”

That converts an implementation limitation into a language semantic.

The correct result is often `Unknown`, `Blocked`, or `BudgetExceeded`.

## 16.5 “Generic specialization should be a class”

That makes typing alter runtime identity, allocation, metaclass topology, caches, and reflection.

## 16.6 “Core builtins are few; hard-code them”

This appears inexpensive until a second semantic authoring source inevitably drifts.

Again, `Option.kind` is the concrete failure.

## 16.7 “The solver said unsat, therefore proven”

Only active trust policy decides whether backend evidence is accepted and at what trust tier.

## 16.8 “The contract passed, therefore verified”

A runtime sample is not a universal proof.

---

# 17. Future flexibility deliberately preserved

The current decisions leave room for:

- public kind polymorphism after a dedicated gate;
- higher-rank kinds if a demonstrated use case justifies them;
- `Any` as a proper top type after lattice policy;
- intersections;
- protocol declarations/coherence;
- guarded recursive aliases/ADTs;
- effect rows and handlers;
- explicit termination measures;
- richer proof logic;
- a certificate kernel;
- multiple proof backends;
- heap/frame reasoning;
- floating-point proof semantics;
- concurrency proof semantics;
- package-carried proof artifacts;
- safe parallel semantic query execution;
- optimizer specialization that preserves observable runtime identity.

They intentionally do **not** preserve compatibility with:

- `Type :: Type`;
- hidden ambient generic application;
- type-bearing selector identity;
- canonical inference metavariables;
- LSP-owned formal semantics;
- universal row identity;
- backend-result-as-proof;
- runtime-guard-as-proof;
- hard-coded generic class lists as reflection authority.

---

# 18. Review gates for future changes

A future typing feature should answer the following before implementation.

## Identity gate

What is stable identity?

What is query-local state?

Can any temporary variable escape into snapshots/metadata?

## Runtime gate

Does the feature change:

```text
selector
dispatch
class identity
metaclass identity
layout
allocation
```

If yes, it is not “just typing.”

## Dynamic/open-world gate

What happens under:

- DNU;
- `perform`;
- reflective method replacement;
- opaque native/FFI;
- missing metadata?

## Incrementality gate

What exact dependency invalidates the result?

Can clean and incremental analysis be compared structurally?

## Reflection gate

What is semantically knowable versus retained versus reifiable?

What does stripped metadata return?

## Diagnostics gate

Can the failure explain:

```text
what happened
what was expected
why it was expected
which source fact caused it
what can be fixed
```

without dumping solver internals?

## Proof gate

Is the claim:

- a type fact;
- effect fact;
- exit fact;
- termination fact;
- assumption;
- proof result;
- trust statement?

If more than one, are they represented separately?

## Performance gate

Does the feature add cost to ordinary execution when unused?

If so, is that cost measured and justified?

---

# 19. Take directly / Adapt / Reject

## 19.1 Take directly

- owner/index binder identity;
- explicit proper-type/constructor/kind distinction;
- `+T` / `-T` declaration-site variance;
- explicit arrow kinds;
- type lambdas;
- local solver variables with occurs checks;
- bidirectional checking;
- result-rich relation outcomes;
- record-row occurs/lacks discipline;
- path-sensitive flow and joins;
- partial versus total correctness;
- verification conditions and counterexamples;
- explicit proof evidence/trust;
- staged semantic queries;
- reverse invalidation;
- immutable published snapshots;
- bounded/cancellable solving;
- native/source canonical signature convergence;
- differential clean/incremental testing.

## 19.2 Adapt

- ML/Haskell inference -> local declared-surface inference;
- Haskell/Scala HKTs -> explicit Phalcom arrow kinds/type lambdas without mandatory kind polymorphism;
- Scala variance -> `+T`/`-T` without ambient implicits;
- OCaml rows -> distinct `RecordRow` domain;
- Rust identity/trust discipline -> without borrow semantics or monomorphized runtime classes;
- Smalltalk runtime coherence -> with compiler-owned static flow facts;
- Swift/JVM/.NET reification lessons -> explicit context/profile-controlled metadata;
- TypeScript flow ergonomics -> without `any`-style semantic collapse;
- Pyrefly execution architecture -> without Python semantics;
- proof-system artifacts -> without dependent public programming.

## 19.3 Reject

- `Type :: Type`;
- universe/dependent escalation as default;
- mandatory public kind polymorphism for HKTs;
- `F<_>` / `F<_, _>`;
- canonical `TypeData::Infer`;
- first assignment as hidden permanent annotation;
- analyzer failure becoming `Dynamic`;
- one compatibility predicate;
- LSP-owned formal checker;
- type-directed selector identity;
- type-directed runtime overload selection;
- specialized runtime generic classes;
- per-instance generic tokens;
- nominal wrapper object around every class;
- `Type.currentApplication`;
- arbitrary runtime kind construction inferred from `FunctionKind` existence;
- current public `applyKind` assumption;
- hard-coded core generic arity lists;
- universal row domain;
- proof/effect/termination folded into callable type identity;
- runtime guard success as proof;
- backend process result as automatically trusted proof;
- assumptions hidden as an evidence trust tier;
- strong immortal descriptor caches.

---

# 20. Phalcom typing philosophy

Phalcom typing is **compiler-owned, flow-sensitive knowledge about a live message-oriented runtime**.

That sentence has several precise consequences.

Canonical types represent publishable semantic forms. Temporary inference variables do not.

A mutable local without an annotation acquires path-sensitive knowledge, not an invisible declaration.

An explicit `Dynamic` boundary is different from the compiler failing to know something.

A static call analysis describes the same runtime message send that execution will perform. It does not invent a type-directed selector.

Native and source implementations share one callable semantic model. Their implementation provenance is different; their type system is not.

A class object may denote a nominal type form without ceasing to be an ordinary runtime class object. A synthetic applied/union/tuple/callable/lambda form may be reflected lazily without creating a new runtime class hierarchy.

Kinds classify semantic type forms. Runtime `FunctionKind` values are reflections of actual arrow kinds; they do not establish an unrestricted runtime kind-construction language. In particular:

```text
List.kind
    may reflect Type -> Type
```

while:

```text
standalone Type -> Type as a runtime constructor
```

is not thereby part of the public runtime semantics.

Likewise, semantic truth is not rewritten by a reflection bug:

```text
Option :: Type -> Type
```

remains the semantic requirement even while current `Behavior#kind` incorrectly reflects `Option` as `Type`.

Effects, exits, termination, contracts, and proof evidence remain separate from return typing and from one another. This lets Phalcom state useful facts precisely:

```text
a total function may perform I/O
a pure function may diverge
a Never-returning function may terminate by raising
a runtime-checked contract may remain statically unproven
a proof may be valid only under explicit assumptions
```

Finally, the runtime remains recognizably Phalcom:

```text
objects
classes
metaclasses
selectors
messages
DNU
reflection
```

Typing adds increasingly precise answers about that system without replacing it.

The long-term objective is therefore not “make Phalcom statically typed” in the sense of building a second, closed-world language over the runtime.

It is:

> **make Phalcom's runtime semantics increasingly knowable, checkable, explainable, reflectable, and provable—without allowing the mechanisms used to know those semantics to redefine them.**

---

# 21. Evidence and correction ledger

This revision was produced from:

1. the previous Spec 06 as the structural baseline;
2. revised Spec 05 as the authority for advanced-domain ownership, effect/exit/termination separation, contract/proof semantics, proof outcomes, and optional gated kind polymorphism;
3. revised Spec 04 for explicit arrow kinds, type lambdas, `where` constraints, and source/runtime separation;
4. reviewed 04.5 decisions for expression identity, inference-variable lifetime, checker decomposition, flow semantics, iteration typing, diagnostics, and LSP migration;
5. current repository `main` for runtime reflection/native-surface implementation state.

## 21.1 Current repository anchors

### `phalcom-core/src/primitive/typing.rs::behavior_kind`

Observed current implementation hard-codes:

```rust
"List" | "Set" => 1,
"Map" => 2,
_ => 0,
```

This is insufficient and causes the current `Option.kind` defect.

### `phalcom-semantic/tests/declaration_types.rs`

Observed test requirement explicitly requires:

```text
Option :: Type -> Type
Some   :: Type -> Type
```

This is the semantic requirement.

### `phalcom-core/src/primitive/typing.rs::reify_kind`

Observed current runtime reification chooses between the atomic `Type` class and `FunctionKind` for arrow-kind values.

This supports reflected arrow kinds without implying standalone runtime arrow-kind construction.

### `phalcom-core/src/primitive/typing.rs::install`

Observed live `TypingContext` registration includes `apply(...)` for type-form application but does not expose `applyKind`.

The rationale therefore treats `applyKind` in older drafts as stale, not as an API that is mysteriously missing from current code.

### Canonical native surface

Current semantic native registration consumes generated canonical surfaces. The formal type checker should build on that rather than revive hand-written standard-surface tables.

## 21.2 Verification limitation

No fix to `Option.kind` is claimed here.

No new `applyKind` is claimed or recommended by implication.

No fresh REPL behavior is claimed.

The supplied dirty local checkout currently fails compilation in semantic changes, so this rationale records source/test evidence and separates that from executable verification.

---

# 22. Final audit checklist

An implementation or later specification remains aligned with this rationale only if all of the following remain true:

1. `Type` remains atomic; no accidental `Type :: Type`.
2. Explicit arrow kinds/type lambdas are sufficient for ordinary HKT use; public kind polymorphism stays separately gated.
3. `InferVarId` never becomes published `TypeId` identity.
4. Solver-local variables do not cross snapshot, metadata, or reflection boundaries.
5. `BodyAnalysisContext`, `FlowState`, and `InferenceSession` remain conceptually separate even if compatibility wrappers temporarily exist.
6. Unannotated mutable bindings use flow knowledge, not hidden inferred declarations.
7. Sequential assignments replace current path knowledge; unions arise from joins, not from historical accumulation on a single path.
8. Formal flow lives in `phalcom-semantic`; LSP advisory logic cannot independently reject programs.
9. `for` element typing follows `iterate(_)` / `iteratorValue(_)`.
10. Missing static iteration precision produces `Unknown`, not fabricated `Dynamic`.
11. Source and native callables use the same semantic call-resolution/inference algorithm.
12. Generated canonical native metadata remains formal authority.
13. Runtime migration duplication does not create a second semantic authority.
14. `Behavior#kind` ultimately consumes canonical declaration metadata rather than a name-based arity switch.
15. `Option` reflects `Type -> Type` once the projection defect is repaired.
16. A reflected `FunctionKind` does not imply a public arbitrary kind-construction language.
17. `applyKind` is not documented as a current public operation unless a later ratified API actually adds it.
18. `Dynamic`, `Unknown`, missing, invalid, cancelled, budgeted, and internal-failure states remain distinct.
19. Equivalence, subtyping, assignability, consistency, and conformance remain distinct relations.
20. Runtime type metadata never changes selector identity, dispatch keys, class/metaclass identity, or ordinary object layout.
21. `Never` remains a normal-return bottom fact, not divergence or totality evidence.
22. Effects, exits, and termination remain independent.
23. `@total` means termination only.
24. Runtime contract execution never becomes static proof evidence.
25. Proof assumptions remain first-class and separate from proof-evidence trust.
26. Only evidence accepted under explicit policy yields `Proven`.
27. Proof/runtime metadata is demand-driven and does not add ordinary per-call runtime cost.
28. Diagnostic semantics stay structured; visual presentation may evolve independently.
29. Clean and incremental formal analysis remain required to agree for identical inputs/policy.
30. Any future feature that violates runtime invariance must be reviewed as a runtime-language change, not smuggled in as “typing metadata.”
