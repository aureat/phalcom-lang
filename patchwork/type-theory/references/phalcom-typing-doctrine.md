# Phalcom Typing Doctrine and Repository Status Discipline

## Purpose

This reference records Phalcom-specific guardrails that type-theory agents must preserve. It is not a substitute for reading the repository. Current specifications, ADRs/PDRs, code, and tests are authoritative.

Every repository-specific statement in a task should be labeled:

```text
CURRENT IMPLEMENTATION
NORMATIVE / RATIFIED DESIGN
PROPOSED
EXPERIMENTAL
FUTURE / PLANNED
RECOMMENDATION
```

## 1. Snapshot observed while deepening this skill

Repository inspection date: 2026-08-14.

Observed documents include:

- `docs/spec/typing/01-core-type-lattice-and-unit.md` — marked **Normative design specification**.
- `docs/spec/typing/02-type-expression-foundation.md` — marked **Proposed normative design; not a claim of current compiler or VM support**.
- `docs/spec/typing/03-type-parameters-and-generic-signatures.md` — same proposed-normative/non-implementation status.
- `docs/spec/typing/STATUS.md` — records Documents 01–03 as completed design work and assigns applied types, substitution, complete relations, variance validation, inference, structural conformance, checker modes, etc. to later documents.
- `docs/spec/typing/README.md` — describes the series as normative incremental design and reiterates that type metadata must not implicitly alter ordinary dispatch/allocation/layout.

This snapshot can become stale. Future agents must re-read current files.

## 2. Dynamic semantics remain baseline

Phalcom is dynamic-first and message/class based. Static typing adds contracts/information about actual execution.

Core architectural form:

```text
dynamic semantics              baseline execution truth
semantic model                 identities/scopes/control facts
canonical type domain          static contracts
checker                        validates type obligations
LSP                            renders/query facts
runtime reflection             observes type metadata where reified
typed runner                   optional runtime enforcement mode
prover                         stronger logical obligations
optimizer                      uses guarded proven facts
```

No consumer gets to invent contradictory semantics.

## 3. Type metadata is non-dispatching unless explicitly designed otherwise

Current typing design repeatedly preserves:

```text
type annotations do not implicitly change
  selector identity
  ordinary method lookup
  overload resolution
  allocation
  instance layout
  ordinary inline-cache identity
  automatic validation
```

If future `@typecase`/multimethod semantics use types, that must be an explicit feature layered on/alongside ordinary dispatch, not a checker hack.

## 4. Domain separation

Keep at least:

```text
ClassId / runtime class
ProtocolId / protocol descriptor identity
ValueShape / advisory runtime possibility
TypeId / canonical language type
TypeParamId / declaration binder
InferenceVarId / solver-local metavariable
Constraint / relation obligation
ProofFact / proposition/path theorem
RuntimeContract / executable validation obligation
EffectFact / mutation/control summary
```

Bridges can map facts across domains. IDs are not interchangeable.

## 5. Normative core lattice facts

The normative core lattice specification currently establishes, among other rules:

```text
Never <: T <: Any
Object <: Any
() is unit / exact empty tuple type
T | Never = T
T & Never = Never
T | Any = Any
T & Any = T
Dynamic is separate from ordinary top Any
None is nullary Option variant
None's unconstrained principal type is Option<Never> under covariance
ordinary brace-bodied fallthrough returns ()
missing return annotation is not equivalent to explicit ()
```

Before relying on these, confirm current spec has not changed.

## 6. Type expressions as reflective objects

Proposed type-expression design makes existing `Class` and `Protocol` descriptors participate directly in reflective `Type` behavior rather than exposing `ClassType`/`ProtocolType` wrappers.

Important consequences:

- bare descriptor identity matters;
- reflection sees actual class/protocol objects;
- synthetic descriptors need trusted immutable representation;
- type equivalence is explicitly separated from subtyping, consistency, acceptance, conformance, and ordinary value equality;
- missing annotations remain absent metadata.

Do not call this current compiler/VM support unless repository code/tests show implementation.

## 7. Generic parameter doctrine

Proposed generic-signature design establishes:

```text
TypeParameter identity = owner descriptor identity + declaration index
name is descriptive
unmarked class/protocol params invariant
out/in explicit for class/protocol parameters
method-owned params invariant in first version
T: Bound = upper bound
T in (A,B) = finite exact constraint set
bound and finite constraints mutually exclusive
nested shadowing allowed; nearest scope wins
same-signature recursive restrictions deferred
```

This is exactly the kind of identity discipline this type-theory skill reinforces.

## 8. Bare generic origin versus applied type

Proposed type-expression design treats bare `Box` as declaration/type-constructor object, not implicit open `Box<T>` application.

Future applied type design is assigned separately. Do not assume partial application/defaults/raw-generic semantics beyond current spec.

## 9. Absence

Phalcom intentionally uses `Option` semantics for explicit absence. Do not reintroduce hidden nullability as checker convenience.

Different facts:

```text
None                         value/variant
Option<Never>                principal unconstrained type of None under current core design
Option<T>                    contextual widened type
missing annotation           source metadata absence
Dynamic                      gradual escape
```

## 10. Unit and return behavior

Current core design makes `()` the unit/empty tuple and ordinary callable fallthrough returns `()`.

Therefore checker result analysis must distinguish:

```text
no annotation written
explicit -> ()
fallthrough path produces ()
explicit return expression
abrupt path produces no normal result (Never at expression point)
```

Do not infer `()` merely because annotation is absent.

## 11. Object model typing obligations

A complete Phalcom type system must eventually type:

- ordinary instances;
- class objects;
- metaclass/class-side behavior;
- `Self` in instance and class-side contexts;
- protocol descriptors;
- method descriptors;
- bound methods;
- blocks/closures;
- selector/family reflection objects;
- modules/packages;
- type-expression descriptors themselves.

A design that only handles ordinary class instances is incomplete.

## 12. Semantic engine reuse

Future checker should reuse current semantic infrastructure where sound:

```text
binding/scope identities
member identities
source spans
CFG/program points
callable summaries
dependency graph/module graph
occurrence/reference index
```

It should introduce a distinct canonical type domain and constraint/relation solver rather than rename `ValueShape` to `Type`.

One semantic truth does not mean one data structure for every abstraction.

## 13. LSP bridge

LSP should consume semantic/type queries:

```text
hover          -> target identity + declared/inferred facts + provenance
completion     -> receiver type/shape + safe/possible member surface
definition     -> semantic identity
references     -> occurrence index
signature help -> dispatch/callable candidates
inlay hints    -> rendered type facts
```

Handlers should not independently re-infer types.

## 14. Typed runner

A typed runner can validate declared/inferred contracts on executed paths using same metadata. It is not a proof of unexecuted paths.

Define:

- which contracts are checked;
- where checks occur;
- whether ordinary runtime omits them;
- how dynamic boundaries are handled;
- how violations are reported/blamed;
- native/FFI trust.

Do not claim test execution + typed runner statically proves a whole program.

## 15. Static proving

Contracts/refinements should share semantic IDs and CFG points with checker but proof status stays separate:

```text
TypeId says: x : Int
Flow fact says: x is nonzero here
Proof says: denominator != 0 proven on this path
```

A solver timeout is `Unknown`, not acceptance/proof.

## 16. Reflection and optimization

Because Phalcom is reflective/open, optimizations based on type/member facts need validity guards.

Examples:

- inline cache keyed by runtime class/selector;
- specialized code based on inferred type guarded by class/version assumptions;
- conformance cache invalidated by member-surface mutation;
- no type metadata silently inserted into ordinary selector cache key.

## 17. FFI/native boundaries

Rust FFI must bridge:

```text
Phalcom reflective/static Type
runtime value representation
Rust ABI/concrete type/trait contract
ownership/lifetime/GC rooting
error/control effects
```

Rust's compiler type identity is not Phalcom type identity. Native declarations are trusted only under explicit authority policy.

## 18. Annotation economy

Optional but correctness-participating typing should maximize information without requiring every local to be annotated.

Annotations are especially valuable at:

- public API boundaries;
- FFI/native boundaries;
- recursive SCCs;
- ambiguous inference points;
- mutable state boundaries;
- protocol/generic contracts;
- semantic intent that inference cannot recover.

Inference should carry local obvious information. Evaluate annotation burden on real Phalcom code, not contrived examples.

## 19. Open-world caveat

Class/member surfaces may change through development/reflection. Static facts depending on them need generation/dependency tracking.

Never freeze:

```text
C does not conform P
```

or:

```text
selector s absent on C
```

without knowing what mutations/module changes can invalidate it.

## 20. Repository workflow for future agent

1. Read `AGENTS.md`.
2. For codebase questions, follow repository's graphify guidance when available.
3. Read typing `README.md`, `STATUS.md`, `CHANGELOG.md`.
4. Read relevant numbered spec entirely, including status header/out-of-scope.
5. Inspect newer decisions that supersede it.
6. Inspect code/tests before saying feature is current.
7. Label current/design/proposed/recommendation in output.
8. Only then design/implement.

## 21. Failure modes

- Proposed design described as current VM behavior.
- `ValueShape` renamed to checker `Type`.
- Type annotations enter selector identity.
- Missing annotations rewritten to `Dynamic` in reflection.
- Applied type assumed to create runtime subclass/per-specialization class state.
- `Option` replaced by hidden nullable pointer semantics.
- LSP handlers build independent inference.
- Typed-runner executions described as proof.

## 22. Competency questions

1. Which current core lattice facts are normative design, and which later typing areas remain deferred?
2. Why can class/protocol descriptors be reflective type expressions without making runtime class identity equal to all type identity?
3. What is the semantic difference between absent annotation and explicit `Dynamic`?
4. Why must type metadata remain outside ordinary selector identity?
5. Which existing semantic-engine structures should a future checker reuse, and which distinct domain must it add?
