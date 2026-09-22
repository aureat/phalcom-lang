---
id: LANG007.C2
category: LANG
program: LANG007
checkpoint: LANG007.C2
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
requirements_baseline_revision: 1a1e5ab022c84ad979a28994a5a135121baaaeaa
requirements_baseline_subject: "feat(lang007): continue C2 universe implementation"
previous_checkpoint: LANG007.C1
previous_checkpoint_status: COMPLETE
active_plan: null
next_plan: LANG007.C2.P2
prepared: 2026-09-22
requires:
  - docs/implementation/LANG007/PROGRAM.md
  - docs/implementation/LANG007/STATUS.md
  - docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
  - docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
  - docs/implementation/LANG007/LANG007.C2/LANG007.C2-requirements-analysis.md
  - docs/spec/collections-next/argument-packs-and-expansion-spec.md
  - docs/spec/collections/06-rest-spread-and-pack-operators.md
  - docs/spec/collections/07-callable-domains.md
---

# LANG007.C2 — Variadic Generic Pack Binding and Type-Level Pack Expansion

## 0. Checkpoint role

This file is the proposed durable checkpoint authority for the **entire `LANG007.C2` implementation**, not merely the first numbered plan. It converts the accepted C2 requirements analysis, the completed C1 architecture, the ratified pack-operator specifications, and a fresh audit of remote `main` into one bounded implementation contract.

It does **not** mark C2 implemented or complete. `LANG007.C1` is complete and closed; P1 established the C2 semantic/metadata substrate at the focused-tested level, while P2/P3 remain necessary for checkpoint acceptance.

P1 started from the planning baseline `1a1e5ab022c84ad979a28994a5a135121baaaeaa` on local `main`. The supplied checkpoint, guidance, and P1 plan were inherited as untracked planning artifacts; no competing C2 semantic architecture is present in the live source.

Current lifecycle truth for this document is therefore:

```yaml
checkpoint: LANG007.C2
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active_plan: null
next_plan: LANG007.C2.P2
```

## P1 completion state

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C2.P1` | canonical pack composition and variadic binder semantics | `COMPLETE` | `IMPLEMENTED` | `FOCUSED_TESTED` | exact complete-pack kind and term, flat concat, mode-sensitive signatures, checked rewrite, and schema-v3 persistence established |
| `LANG007.C2.P2` | source variadic generic binding and structural/callable type expansion | `PROPOSED` | `NOT_STARTED` | `UNVERIFIED` | next action |

### Durable P1 interfaces

- `KindId::COMPLETE_TYPE_PACK` / `KindData::CompleteTypePack` is the exact non-proper complete two-lane domain.
- `TypeStore::{complete_type_pack, project_positionals, project_labeled, project_complete}` owns complete-pack construction and lane projection. `TupleFromPacks` remains the only Tuple formation authority.
- `TypeData::{TypePackConcat, LabeledTypePackConcat}` are flat canonical residual compositions. Concrete duplicate labels fail; symbolic overlap remains residual until checked rewriting proves validity or invalidity.
- `GenericParameterMode` is stored on `TypeParameterData` and slot-aligned in `GenericSignature`; parameter semantic versioning and interface fingerprints include it. Pack modes require their exact kind and invariant variance.
- `TypeSubstitution::try_apply` and `materialize_type` re-canonicalize P1 terms and surface concrete labeled duplication as an error. The legacy unchecked APIs retain their pre-C2-safe caller precondition.
- Metadata schema v3 serializes complete/concat terms and binder mode. Schema v1/v2 remains decodable for historical forms and rejects C2-only forms/modes.

### Focused verification

| Gate | Scope | Result |
|---|---|---|
| G1-G4 focused semantic regressions | `foundations::{type_model,generics_core,substitution,generic_application}` | PASS: 16, 8, 4, and 10 selected tests |
| G5 metadata and C1 closure | `integration::{metadata,type_member_closure}` and `phalcom-type-meta` | PASS: 16, 9, and 6 tests |
| Affected crate checks | `phalcom-type-meta`, `phalcom-semantic`, `phalcom-core` | PASS |
| Hygiene | `cargo fmt --all -- --check`, `git diff --check` | PASS |

### Deferred verification

- Workspace tests, workspace clippy, full language corpus, VM/LSP suites, and release certification remain deliberately deferred.
- Inherited `BASELINE-001` and `BASELINE-002` trait failures were not exercised because P1 did not touch the trait/default-body dependency architecture.

### P2 takeover obligation

P2 must elaborate source `*` / `**` / `***` directly into P1 canonical terms. It must migrate all source-reachable C2 rewrite paths to `TypeSubstitution::try_apply` or the corresponding fallible materialization path before enabling syntax that can make symbolic labeled overlap concrete. It must not infer mode from kind, use `...` for packs, add pack decomposition, or introduce runtime pack ABI work.

### Next action

Execute `LANG007.C2.P2` source binder/application elaboration against the P1 handoff. C2 remains open.

Future implementers must re-read `PROGRAM.md`, `STATUS.md`, this checkpoint, the P1 handoff, and the current local Git state before starting P2.

---

# 1. Repository audit baseline

## 1.1 Audited remote `main`

The preceding handoff named revision:

```text
19e6b555c999e920aeca43cbdbec250617f4e1c9
```

as the most recently audited main revision, while explicitly requiring a live re-check.

The fresh remote-main audit found:

```text
1a1e5ab022c84ad979a28994a5a135121baaaeaa
feat(lang007): continue C2 universe implementation
```

Therefore the live repository is one commit ahead of the handoff anchor. The new commit adds the accepted C2 requirements analysis and accompanying Universe work. This checkpoint is grounded in `1a1e5ab...`, not the older `19e6b555...` assumption.

The implementer must still run local checkout commands before editing:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

A remote-main audit does not prove that a future local workspace is clean or at the same revision.

## 1.2 Current LANG007 lifecycle state

The live lifecycle remains:

```text
LANG007.C1
    COMPLETE
    IMPLEMENTED
    FOCUSED_TESTED

LANG007.C2
    next checkpoint
    no active plan
```

C1 is not reopened by C2. C2 consumes C1 as frozen substrate.

## 1.3 C2 repository records already present

The audited tree contains:

```text
docs/implementation/LANG007/LANG007.C2/
    LANG007.C2-requirements-analysis.md
```

The accepted requirements analysis already establishes the intended checkpoint title and the central semantic thesis:

> Variadicity is source-level binding and composition over C1's canonical structural domains, while `*`, `**`, and `***` retain exactly one lane meaning throughout the language.

C2 planning and implementation must build on that accepted analysis rather than recreate an older pre-C1 roadmap.

---

# 2. Normative authority and conflict resolution

Use the following authority order for C2:

```text
1. explicit user rulings / accepted LANG007 amendments
2. current ratified or authoritative specifications
3. accepted LANG007.C2 requirements analysis
4. this C2 checkpoint record
5. completed C1 checkpoint, walkthroughs, and handoffs
6. live production architecture for mechanical facts
7. C2 numbered implementation plans for execution mechanics
8. historical plans, drafts, and superseded prose
```

Three rules are load-bearing.

First, stale `...P` pack examples in older LANG007 roadmap prose are not authority. C2 uses `*`, `**`, and `***` exclusively for the pack lane algebra. `...` retains its independent language roles.

Second, C1 capabilities are frozen architecture. C2 may extend them, but may not duplicate them with a parallel pack arena, projection system, generic-instantiation system, row system, or runtime solver.

Third, a live-source implementation detail may drift mechanically from a plan. Mechanical adaptation is allowed. A change in semantic ownership, canonical identity, solver lifetime, metadata authority, source identity, or runtime responsibility is not mechanical drift and requires consultation.

---

# 3. Bounded C2 acceptance objective

`LANG007.C2` is accepted when Phalcom can statically represent, bind, project, compose, forward, expand, substitute, persist, and incrementally re-evaluate variable-length **positional**, **labeled**, and **complete two-lane type packs** using the already-ratified `*` / `**` / `***` algebra, while preserving C1's canonical structural type model and keeping type-level computation out of the VM.

The end-to-end C2 vertical is:

```text
source generic/type syntax
        │
        ▼
lane-aware binders and expansion items
    Single / * / ** / ***
        │
        ▼
canonical pack-domain terms
    TypePack
    LabeledTypePack
    CompleteTypePack
        │
        ▼
symbolic lane composition
    TypePackConcat
    LabeledTypePackConcat
        │
        ▼
binder-aware generic application
    one canonical semantic argument per declared binder
        │
        ▼
existing C1 substitution / materialization / inference lifetime
        │
        ▼
TupleFromPacks / callable-domain type formation / type projections
        │
        ▼
stable metadata / fingerprints / source identity / incremental parity
        │
        ▼
no runtime type-pack objects, no VM pack solver, no pack decomposition search
```

C2 is **not** accepted merely because `<*Ts>` parses, because a concrete pack example works, or because runtime rest/spread syntax already exists. The checkpoint requires the complete bounded static vertical, including symbolic unknown-cardinality packs, metadata, incremental identity, diagnostics, and fixed-generic compatibility.

---

# 4. Frozen C1 substrate — DO NOT REIMPLEMENT

The following are C1-owned and must be reused.

| C1 substrate | Live owner / representation | C2 rule |
|---|---|---|
| proper type vs kinded form | `TypeId`, `KindId`, `ProperTypeId` | pack terms stay in the existing kinded `TypeId` universe |
| positional type packs | `KindData::TypePack`, `TypeData::TypePack` | extend with symbolic composition; do not add a `PackId` arena |
| labeled type packs | `KindData::LabeledTypePack`, `TypeData::LabeledTypePack` | preserve label order and uniqueness |
| records / record rows | `TypeData::Record`, `RecordRowId` | never collapse labeled packs into rows |
| structural Tuple | `TypeData::Tuple` | remains the concrete Tuple proper-type identity |
| symbolic Tuple formation | `TypeData::TupleFromPacks` | C2 expansion must lower into this authority |
| generic parameter identity | `TypeParameterId` | one identity per binder, variadic or not |
| generic instantiation | `GenericInstantiation` | retain one binding authority for kinded non-row terms |
| substitution | `TypeSubstitution` | extend; do not create `PackSubstitution` |
| whole generic inference | `InferenceSession`, `GenericApplicationSession` | pack kinds use ordinary kind-aware inference variables |
| row inference | `RecordRowSolver`, `RecordRowVarId` | remains row-specific; no pack-row unification |
| type-member projections | C1 `TypeLevelProjection` + normalizer | use for Tuple lane projections; do not add a second projection framework |
| structural `Map<F>` | TypePack/LabeledTypePack/Tuple/Record/RecordRow | C2 accepts mapped pack terms by semantic domain |
| stable metadata | `KindNode`, `TypeNode`, metadata exporter/validator | extend existing graph |
| source identity | existing semantic source index | binder mode modifies the existing parameter identity record, not a second symbol |
| runtime boundary | compile-time semantic normalization | no runtime TypePack/CompletePack object or opcode |

C2 also inherits C1's declaration rule: `intrinsic` is a first-class source keyword / implementation kind; the retired `@intrinsic` attribute must not be reintroduced. `ImplementationKind::Intrinsic` remains distinct from internal `NativeIntrinsicId` metadata.

C2 must specifically preserve:

```text
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
TypePack != Tuple
TypeId != ProperTypeId
InferVarId != TypeId
RecordRowVarId != RecordRowId
compile-time pack term != runtime ArgumentPack
```

---

# 5. Ratified pack operator algebra

The operator semantics are fixed:

```text
*     positional lane
**    labeled lane
***   complete two-lane pack
```

For a complete lane product `Q = ⟨Qp, Ql⟩`:

```text
π*(Q)   = Qp
π**(Q)  = Ql
π***(Q) = Q
```

The surrounding grammar changes whether the operation means **capture**, **expansion**, or **type unpacking**. It does not change the lane meaning.

Therefore:

```phalcom
*P
**P
***P
```

must never be implemented as three unrelated language features that only happen to share tokens.

Likewise:

```text
***single-lane-source
```

must not be treated as convenient spelling for the corresponding one-lane operator. In particular:

```text
***TypePack        != *TypePack
***LabeledTypePack != **LabeledTypePack
```

unless a future explicit language decision changes the ratified algebra.

`...` is outside this algebra. C2 must not recognize either of the following as generic/type-pack syntax:

```phalcom
...P
<...Ts>
```

---

# 6. Exact complete type-pack domain

## 6.1 Semantic requirement

C2 requires a canonical semantic value representing:

```text
⟨P, L⟩

P : TypePack
L : LabeledTypePack
```

This is conceptually `CompleteTypePack(P, L)`.

It is not a Tuple proper type. Tuple formation remains:

```text
CompleteTypePack(P, L)
        │
        └── when a Tuple type is requested ──> TupleFromPacks(P, L)
```

and when both lanes are concrete and empty:

```text
TupleFromPacks([], []) -> Unit
```

through C1's existing canonicalization.

## 6.2 Live architecture conclusion

The current `KindData` contains:

```rust
Type
RecordRow
TypePack
LabeledTypePack
Arrow { ... }
```

and there is no existing product kind that can exactly classify a two-lane complete pack. Encoding the complete pack as `Type`, `TypePack`, or `LabeledTypePack` would erase a semantic distinction required by the accepted C2 model. Encoding it as a Tuple would confuse a type-level lane product with a proper structural value type.

Therefore the live architecture strongly implies that C2.P1 must add an **exact internal complete-pack kind/domain**, most naturally as a new `KindData`/`KindId` base kind plus a corresponding canonical `TypeData` term. The exact Rust and source-facing name is flexible, but the domain distinction is not.

If an implementer proposes avoiding a new base kind, the alternative must prove all of the following without semantic loss:

```text
- exact discrimination from Type / TypePack / LabeledTypePack
- ordinary kind-aware inference-variable allocation
- correct arrow-kind generic constructor formation
- stable metadata serialization
- stable hashing/fingerprinting
- no proper-Type coercion
- no Tuple identity conflation
```

Absent such an existing exact abstraction, a new complete-pack kind is the architecturally correct extension.

## 6.3 Source visibility is separate

Adding an internal complete-pack kind does **not** automatically ratify a public source spelling such as:

```phalcom
P: CompleteTypePack
```

The current `KindSyntax` exposes `Type`, `RecordRow`, `TypePack`, and `LabeledTypePack`, not a complete-pack name. C2 can bind `***P` to the internal exact kind without exposing a new named kind. Public naming/reflection of that domain remains a separate user-visible language decision unless already accepted elsewhere.

---

# 7. Generic binder mode is orthogonal to kind

C2 must encode the distinction between:

```phalcom
P: TypePack
```

and:

```phalcom
*P
```

The first declares **one fixed generic parameter** whose argument is itself a `TypePack` value. The second declares a **variadic positional source binder** whose variable-length captured source sequence is assembled into one canonical `TypePack` binding.

The same principle applies to `**` and `***`.

Conceptually:

```rust
enum GenericParameterMode {
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}
```

Exact names are flexible. The semantic dimensions are not:

```text
parameter identity
    TypeParameterId

parameter domain
    KindId

source binding shape
    GenericParameterMode

variance
    Variance

constraints
    existing GenericConstraint set
```

### 7.1 Critical live-store identity requirement

`TypeStore::intern_type_parameter` currently versions a parameter slot using semantic equality over approximately:

```text
name + kind + variance
```

C2 must include binder mode in that semantic identity/versioning decision. Otherwise these two public interfaces:

```phalcom
class C<P: TypePack> {}
class C<*P> {}
```

could reuse the same parameter identity because both have kind `TypePack`, even though they have different application shape and API semantics.

The same binder mode must survive in:

```text
TypeParameterData
GenericSignature
stable metadata TypeParameterRecord / signature products
fingerprints
incremental dependency identity
source presentation/tooling
```

Changing `Single -> *`, `* -> **`, or `* -> ***` is an interface change.

---

# 8. Binder legality and deterministic ownership

C2 must not require arbitrary sequence partitioning to bind a generic application.

The accepted deterministic model is:

1. ordinary single binders consume their required fixed positional prefix;
2. a positional `*` binder owns the remaining positional lane assigned to it;
3. a labeled `**` binder owns the labeled lane assigned to it;
4. a complete `***` binder owns both remaining lanes and is terminal;
5. no source application requires trial partitions or backtracking.

At minimum C2 must enforce:

```text
- at most one positional-pack owner
- at most one labeled-pack owner
- at most one complete-pack owner
- complete capture conflicts with split * / ** capture over the same remaining lanes
- *** is terminal
- no ordinary fixed binder may appear after a binder that makes its source position ambiguous
```

Canonical positive shapes include:

```phalcom
class A<T> {}
class B<T, *Ts> {}
class C<T, **Ls> {}
class D<T, *Ts, **Ls> {}
class E<T, ***P> {}
```

C2 does not introduce explicit named generic-argument syntax merely to populate the labeled lane. Unless separately ratified, the labeled lane of a generic application is contributed by admitted `**` / `***` expansions, not by inventing a new `name: Type` generic argument form.

---

# 9. Pack binder variance and constraints

## 9.1 Variance

C2 does not define pack covariance/contravariance, elementwise variance, length variance, or complete-pack subtyping. Those are independent type-theoretic features.

Therefore C2's safe contract is:

```text
pack binders are invariant in C2
explicit + / - variance markers on pack binders are rejected
```

This is a scope boundary, not a claim that future pack variance is impossible.

## 9.2 Constraints

C2 must not reinterpret an existing constraint such as:

```text
Ts |= Printable
```

as quantified shorthand for:

```text
for every T in Ts: T |= Printable
```

unless a separate accepted specification introduces quantified pack constraints.

Existing constraint-domain validation remains authoritative. A pack-kinded term is not silently coerced to a proper `Type` merely to make an ordinary constraint compile.

---

# 10. Canonical positional pack composition

C2 needs a durable symbolic term for ordered positional composition when one or more source contributions have unknown cardinality.

For:

```text
P : TypePack
Q : TypePack
```

an assembly equivalent to:

```text
[Int] ++ P ++ [String] ++ Q
```

must remain representable without knowing the sizes of `P` or `Q`.

The canonical concept is:

```text
TypePackConcat([Int], P, [String], Q)
```

where all concrete singleton/multi-element segments are themselves ordinary C1 `TypePack` terms or equivalent normalized inline segments.

Required laws:

```text
Concat()                 ≡ []
Concat(P)                ≡ P
Concat([], P)            ≡ P
Concat(P, [])            ≡ P
Concat([A,B], [C])       ≡ [A,B,C]
Concat(Concat(A,B), C)   ≡ Concat(A,B,C)
```

Additional requirements:

- nested concat terms flatten at construction/interning time;
- adjacent concrete segments merge;
- no deep left/right concat tree accumulates across substitutions;
- once substitution makes all segments concrete, the result becomes the existing `TypeData::TypePack` identity;
- no second concrete-pack identity survives alongside C1 `TypePack`;
- all operands are checked by semantic kind/domain, not source spelling;
- `P::Map<Option>` and any other stable `TypePack`-kinded projection are valid operands.

No public `Concat` operator or type member is implied by this internal term.

---

# 11. Canonical labeled pack composition and deferred disjointness

C2 likewise needs symbolic ordered composition for `LabeledTypePack`.

Conceptually:

```text
LabeledTypePackConcat(L1, L2, ...)
```

Required laws include:

```text
ConcatLabeled()               ≡ []
ConcatLabeled(L)              ≡ L
ConcatLabeled([], L)          ≡ L
ConcatLabeled(L, [])          ≡ L
```

with adjacent concrete segments merged when valid.

The key additional invariant is label uniqueness:

```text
keys(Li) ∩ keys(Lj) = ∅
```

for every pair of concrete contributions in the final pack.

Therefore:

```text
[(name, String)] ++ [(name, Int)]
```

is statically invalid.

For symbolic composition:

```text
L1 ++ L2
```

unknown overlap must remain a stable symbolic residual. The compiler must not:

```text
assume disjointness
reject all symbolic composition
use last-wins semantics
sort labels and lose source order
collapse the pack into RecordRow
```

The semantic states are:

```text
known disjoint  -> reducible/canonical
unknown overlap -> symbolic/stuck
known duplicate -> invalid
```

### 11.1 Checked normalization requirement

This obligation can become invalid **after substitution**. The current `TypeSubstitution::apply` API returns a `TypeId` and in several existing branches uses `unwrap_or(original)` for operations assumed to preserve validity. C2 must not hide a newly discovered duplicate-label error by returning the old symbolic term.

P1 must therefore establish a checked normalization/materialization route for C2 terms. Acceptable shapes include:

```text
- a checked pack-normalization API returning Reduced / Symbolic / Invalid;
- a checked substitution/materialization variant for terms whose validity can change;
- reuse/generalization of the existing projection-normalization result discipline.
```

The architectural requirement is that post-substitution invalidity becomes an actual semantic error. The exact helper/API spelling is flexible.

---

# 12. Complete pack composition is lane-wise

For a complete pack:

```text
Q = ⟨Qp, Ql⟩
```

contributions:

```text
*P
***Q
**L
```

assemble as:

```text
positionals = TypePackConcat(P, Qp)
labeled     = LabeledTypePackConcat(Ql, L)
```

They do not assemble as:

```text
P ++ Qp ++ payloads(Ql) ++ payloads(L)
```

because labels are semantic structure, not positional decoration.

The canonical complete term should therefore structurally preserve two children:

```text
CompleteTypePack {
    positionals: TypeId, // kind TypePack
    labeled: TypeId,     // kind LabeledTypePack
}
```

or an equivalent exact representation.

---

# 13. Type-level lane projection

C2 type unpacking must reuse C1 projection semantics wherever possible.

Required structural behavior:

| Subject | `*subject` | `**subject` | `***subject` |
|---|---|---|---|
| `TypePack` | identity | invalid | invalid |
| `LabeledTypePack` | invalid | identity | invalid |
| complete pack `⟨P,L⟩` | `P` | `L` | identity |
| Tuple `Tuple(P,L)` | C1 positional projection | C1 labeled projection | complete pack of both C1 projections |
| `Unit` | empty `TypePack` | empty `LabeledTypePack` | empty complete pack |
| unrelated proper type | invalid unless accepted structural evidence proves the required lane interface | invalid | invalid |

For Tuple subjects, C2 must not create a new Tuple-lane projection system. It should lower to or otherwise share C1's canonical `Positionals` / `Labeled` projection identities and normalizer. `***Tuple` is then the complete pair of those two lane results.

For abstract operands, validity depends on semantic domain/evidence, not syntactic bare-variable recognition. A `TypePack`-kinded `Map` result is as valid as a direct pack parameter.

---

# 14. Generic argument assembly and binder-aligned application

## 14.1 Source representation

The current AST stores a generic application as:

```rust
TypeAnnotationExpr::Application {
    origin,
    arguments: Vec<TypeAnnotation>,
    ...
}
```

C2 needs lane-aware items equivalent to:

```rust
enum GenericArgumentItem {
    Single(TypeAnnotation),
    Expansion {
        mode: ExpansionMode,
        operand: TypeAnnotation,
    },
}
```

The repository already has:

```rust
ExpansionMode {
    Positional,
    Labeled,
    Complete,
}
```

for pack contribution lanes. C2 should reuse or deliberately generalize that existing lane enum rather than add another semantically duplicate `GenericPackOperator`.

## 14.2 Canonical binder alignment

Source arity is not canonical semantic arity.

For:

```phalcom
class C<T, *Ts> {}
```

and source:

```phalcom
C<Int, String, Bool>
```

canonical application must be:

```text
T  -> Int
Ts -> TypePack[String, Bool]
```

and the resulting `Applied` form must contain one canonical argument per declaration binder:

```text
Applied(C, [Int, TypePack[String, Bool]])
```

This preserves the existing `TypeData::Applied`, arrow-kind application, `GenericInstantiation`, and `TypeSubstitution` architecture.

For:

```phalcom
class C<T, ***P> {}
```

canonical application similarly becomes:

```text
Applied(C, [Int, CompleteTypePack(remainingPositionals, labeledLane)])
```

No `VariadicAppliedType` is permitted.

---

# 15. Empty specialization semantics

Variadic binders can capture zero elements. Therefore:

```phalcom
class V<*Ts> {}
```

must distinguish:

```phalcom
V      // unsaturated constructor
V<>    // saturated specialization with Ts = []
```

The same applies to `**Ls` and `***P`.

A critical live-tree observation is that the parser's generic-application loop already appears capable of constructing an application node with an empty argument list. The missing semantics are downstream: `TypeStore::apply_type_form(origin, &[])` currently returns `origin` unchanged.

C2 therefore must not blindly “add empty-`<>` parser support” if the parser already accepts it. The required correction is binder-aware elaboration:

```text
source V<>
    ↓ consult signature <*Ts>
canonical argument pack = positional [], labeled []
    ↓ bind
Ts = TypePack[]
    ↓ ordinary kind application
Applied(V, [TypePack[]])
```

For fixed-only declarations, explicit empty application remains invalid when required arguments are missing.

---

# 16. Structural Tuple type expansion

C2 must make type Tuple formation consume lane-aware expansion entries while preserving C1 Tuple identity.

Examples:

```phalcom
(*P,)
(**L,)
(***Q,)
```

Conceptually mixed formation:

```text
Int
*P
***Q
**L
```

lowers to:

```text
TupleFromPacks(
    TypePackConcat([Int], P, π*(Q)),
    LabeledTypePackConcat(π**(Q), L)
)
```

When all lanes become concrete, C1's `tuple_from_packs` path must canonicalize to the existing `TypeData::Tuple`, including `Unit` for the empty product.

Tuple expansion must preserve:

```text
- all positional ordering
- positional-before-labeled lane boundary
- label identity
- label order
- duplicate-label rejection
```

---

# 17. Callable-domain type unpacking

The ratified callable-domain specs make the static forms:

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

part of C2.

C2 owns **formation and normalization of the callable Type**, not runtime rest capture or ABI transport.

The current AST `TypeCallableParameter` has only:

```rust
rest: bool
```

which is insufficient to distinguish positional/labeled/complete type unpacking. C2.P2 must replace or generalize this with lane-aware static type syntax while keeping runtime `ParameterDef::rest_mode` ownership separate.

The result of static callable-domain unpacking must feed the existing canonical callable type model or its accepted lane-aware extension. It must not require a VM `ArgumentPack` object merely to represent a type.

---

# 18. Generic declaration-site coverage

`GenericParameterSyntax` is reused across many declaration sites in the live AST, including nominal declarations, traits, data, enums/variants, type aliases, impls, methods, getters, setters, and index members.

C2 must define one declaration-site policy rather than accidentally supporting packs in one parser path and not another.

Default checkpoint requirement:

> Every declaration site that already supports generic parameters must either accept the C2 binder modes with the same semantic rules or reject them intentionally with a targeted diagnostic. No silent parser/semantic mismatch is acceptable.

P2 should audit at least:

```text
ClassDef
EnumDef
VariantDecl
DataDef
TraitDef
TypeAliasDef
ImplDef
MethodDef
GetterDef
SetterDef
IndexMethodDef
```

Callable-specific limitations may be phased only if explicitly recorded as a bounded C2 exception; they must not emerge accidentally from missing plumbing.

---

# 19. Whole-pack inference boundary

C2 may infer/bind a pack **as one value**:

```text
P = [Int, String]
L = [(name, String)]
Q = ⟨[Int], [(name, String)]⟩
```

because `GenericApplicationSession` already gives every non-row parameter a kind-aware `InferVarId`, and pack-valued forms already live in the kinded `TypeId` domain.

C2 must not solve equations that require splitting an unknown sequence:

```text
Concat(P, [String]) = [Int, Bool, String]
    => solve P = [Int, Bool]          // NOT C2

Concat(P, Q) = [A, B, C]
    => enumerate partitions          // NOT C2
```

No:

```text
PackVarId
prefix solver
suffix solver
length arithmetic
partition enumeration
backtracking
```

is required by C2.

A specialized pack solver is a later checkpoint only if future pack-decomposition semantics justify one.

---

# 20. Substitution, materialization, and occurrence analysis

Every new C2 term must participate in all existing semantic traversals that can reach a canonical `TypeData` node.

At minimum audit and extend:

```text
TypeSubstitution::apply
specialize_self_type
GenericInstantiation materialization
parameter-occurrence checks
inference lifting / TypeId -> InferenceTerm conversion
normalization recursion
metadata export/import validation
formatting/presentation
source indexing where type traversal matters
```

Required examples:

```text
TypePackConcat([A], P, [B])
P -> [X, Y]
=> [A, X, Y, B]
```

and:

```text
LabeledConcat([(x,A)], L)
L -> [(y,B)]
=> [(x,A), (y,B)]
```

whereas:

```text
L -> [(x,B)]
```

must become a duplicate-label error rather than silently retaining the old symbolic concat.

Any query-local inference variable must be materialized or rejected before canonical metadata publication.

---

# 21. Stable metadata and fingerprint requirements

The live metadata graph currently contains stable C1 forms:

```text
KindNode::{Type, RecordRow, TypePack, LabeledTypePack, Arrow}
TypeNode::{TypePack, LabeledTypePack, TupleFromPacks, ...}
TypeParameterRecord { id, name, kind, variance, source }
GenericSignatureRecord { owner, parameters, constraints }
```

C2 must extend these durable products for every new durable concept.

At minimum, if the semantic implementation contains them, stable metadata needs equivalents of:

```text
complete-pack kind/domain
CompleteTypePack(positionals, labeled)
TypePackConcat(segments)
LabeledTypePackConcat(segments)
generic parameter binder mode
```

Structural fingerprints must use stable child fingerprints and explicit tagged node identities. They must not use:

```text
raw TypeId indexes
InferVarId
RecordRowVarId
source byte offsets as semantic identity
formatted source strings as canonical identity
```

Public-interface fingerprint examples that must differ:

```phalcom
P: TypePack
```

vs:

```phalcom
*P
```

and:

```phalcom
*P
```

vs:

```phalcom
***P
```

The current metadata exporter's `GenericSignatureKey` is based on owner, parameters, and constraints. C2 must audit whether binder mode is already structurally reachable through stable parameter records; if not, the key/fingerprint path must be extended so mode-only changes cannot alias.

---

# 22. Incremental and cache correctness

C2 extends C1's rule:

```text
cold final semantic state == incremental final semantic state
```

for equivalent source.

Representative edit transitions include:

```text
<P: TypePack> -> <*P>
<*P>           -> <***P>
<T>            -> <T, *Ts>
<*Ts>          -> <*Ts, **Ls>
pack concat gains/removes symbolic segment
symbolic labeled pack becomes concretely conflicting
```

Parity must cover:

```text
canonical type term
canonical applied arguments
diagnostics
public metadata fingerprint
source targets
incremental dependency invalidation
downstream dependent types
```

C2 must not use a whole-workspace invalidation/reset as the normal way to make these edits correct.

---

# 23. Source identity and presentation

A binder such as:

```phalcom
class C<*Ts> {}
```

owns exactly one canonical generic parameter identity: `Ts`.

The `*` token modifies source binding mode; it is not a separate variable or symbol target.

References through:

```text
Ts
*Ts
**Ls
***P
```

must navigate through the existing parameter identity model.

Presentation must remain source-oriented. Internally:

```text
Applied(C, [Int, TypePack[String, Bool]])
```

may be canonical, but diagnostics for source `C<Int, String, Bool>` must not casually print `C<Int, [String, Bool]>` unless that is actual accepted source syntax.

Similarly, internal complete-pack terms should not leak enum/debug names into user diagnostics.

---

# 24. Runtime and callable-ABI boundary

C2 is a compiler/type-system checkpoint.

It must not introduce:

```text
runtime TypePack values
runtime LabeledTypePack values
runtime CompleteTypePack values
VM pack-solving bytecode
VM dispatch for type-level * / ** / ***
runtime generic pack inference
new call-frame pack ABI
selector calling-convention redesign
runtime F.3 rest-capture migration
```

The AST already contains runtime-facing `RestMode` and value-level `ExpansionMode` concepts because the language has related argument-pack syntax. Shared tokens and lane laws do not imply one runtime representation.

The invariant is:

```text
same lane algebra
≠ same semantic layer
≠ same storage representation
```

---

# 25. Diagnostics required by C2

C2 should expose targeted failure classes rather than collapsing all errors into generic arity/kind failures.

| Failure class | Required diagnostic concept |
|---|---|
| illegal binder ordering | variadic binder violates deterministic capture order |
| second positional pack owner | positional lane already captured |
| second labeled pack owner | labeled lane already captured |
| split capture plus complete capture | complete pack conflicts with existing lane owner |
| binder after `***` | complete pack binder must be terminal |
| variance marker on pack binder | pack variance is not defined in C2 |
| `*X` wrong domain | positional expansion requires a positional-capable pack source |
| `**X` wrong domain | labeled expansion requires a labeled-capable pack source |
| `***X` single-lane domain | complete expansion requires a two-lane complete-capable source |
| extra positional source data | declaration has no remaining positional owner |
| extra labeled source data | declaration has no remaining labeled owner |
| concrete duplicate label | labeled pack contains the same label more than once |
| deferred duplicate becomes concrete | instantiated symbolic labeled composition overlaps |
| fixed-only empty application | required fixed generic argument is missing |
| pack decomposition needed | requested inference requires unsupported pack splitting |
| `...` as pack operator | `...` is not generic/type-pack syntax |
| constraint on wrong domain | constraint requires proper Type but subject is pack-kinded |
| unsupported explicit labeled generic syntax | no such generic-argument syntax is ratified |

Operator span and operand span should remain separately available where the AST/source-index architecture permits it.

---

# 26. Performance and resource requirements

C2 must remain deterministic and approximately linear in the amount of pack material processed.

Target complexity:

```text
source pack assembly:
    O(number of source items)

positional concat canonicalization:
    O(segments + merged concrete elements)

labeled concat canonicalization:
    O(entries) with expected O(1) duplicate lookup per label

complete composition:
    O(positional contributions + labeled contributions)

binder matching:
    O(number of binders + assembled contributions)
```

Forbidden complexity patterns:

```text
partition enumeration
permutation search
recursive pack splitting
exponential normalization
repeated rebuilding of nested concat chains
whole-workspace invalidation for ordinary pack edits
```

Canonical concat terms should store/normalize flat segment sequences rather than recursive binary trees.

---

# 27. C2 source/semantic impact map

The following live areas are expected C2 owners. Exact helper names may drift, but ownership should remain stable.

| Layer | Live path / symbol | C2 requirement |
|---|---|---|
| tokenization | `phalcom-ast/src/token.rs` — `Asterisk`, `DoubleAsterisk`, `TripleAsterisk`, `DotDotDot` | reuse tokens; do not make `...` a pack token |
| shared lane mode | `phalcom-ast/src/ast.rs` — `ExpansionMode` | reuse/generalize for expansion syntax where semantically identical |
| runtime rest mode | `phalcom-ast/src/ast.rs` — `RestMode` | do not conflate runtime capture ownership with generic binder semantics accidentally |
| generic binder AST | `GenericParameterSyntax` | add/represent binder mode and operator span |
| kind syntax | `KindSyntax` | internal complete kind need not imply public syntax; add only if ratified |
| generic application AST | `TypeAnnotationExpr::Application` | replace flat argument assumption with lane-aware item representation |
| Tuple type AST | `TypeTupleElement` | admit pack expansion entries without losing ordinary labels |
| callable type AST | `TypeCallableParameter` | replace `rest: bool` with lane-aware static unpack semantics |
| generic parser | `parse_generic_parameters` | parse `*` / `**` / `***` binder modes and enforce syntax constraints |
| type application parser | `parse_postfix_type` | parse lane-aware expansion items; preserve explicit empty application |
| semantic kinds | `types/kind.rs`, `types/id.rs` | add exact complete-pack domain if no existing exact domain appears |
| canonical types | `types/store.rs` — `TypeData` | add complete/concat terms and canonicalizers |
| parameter identity | `types/parameter.rs`, `TypeStore::intern_type_parameter` | include binder mode in semantics/versioning |
| generic signature | `GenericSignature` | persist parameter modes aligned with parameters |
| source type formation | `types/annotation.rs` | assemble source lanes, bind against signature, create binder-aligned canonical arguments |
| substitution | `types/substitution.rs` | traverse and normalize all new terms; surface post-substitution invalidity |
| materialization | `types/instantiation.rs` | same; retain `GenericInstantiation` authority |
| inference | `checker/inference.rs`, `checker/row_inference.rs` | whole-pack kinded inference only; no decomposition solver |
| projection | `types/projection.rs` | reuse C1 Tuple projections and normalization outcome discipline |
| metadata schema | `phalcom-type-meta/src/{kind,type_node,generic}.rs` | stable nodes + binder mode |
| metadata export | `phalcom-semantic/src/metadata/export.rs` | export/fingerprint/validate new durable forms |
| source index | `phalcom-semantic/src/source_index/*` | parameter/operator references preserve one canonical target |
| incremental products | semantic snapshot/invalidation/fingerprint paths | cold/incremental parity |
| tests | `phalcom-ast/tests`, `phalcom-semantic/tests` | focused syntax, semantics, metadata, incremental, negative coverage |

---

# 28. C2 plan decomposition

C2 should use three dependency-ordered plans unless live implementation evidence proves a materially better cut.

## 28.1 `LANG007.C2.P1` — Canonical Pack Composition and Variadic Binder Semantics

P1 owns the semantic substrate required before broad source wiring:

```text
exact complete-pack kind/domain
complete-pack canonical term
GenericParameterMode / binder-mode identity
TypePackConcat
LabeledTypePackConcat
lane projection semantic helpers
canonicalization / duplicate-label residual discipline
substitution and materialization support for new terms
metadata schema/export foundations
focused semantic-unit tests
```

P1 should not need the full generic application parser/type-unpacking vertical to prove its canonical model.

## 28.2 `LANG007.C2.P2` — Source Variadic Generic Binding and Structural/Callable Type Expansion

P2 owns broad source elaboration:

```text
* / ** / *** generic binders
generic application expansion items
binder-aware source lane assembly
explicit empty specialization semantics
Tuple type expansion
callable-domain type unpacking
declaration-site rollout
source diagnostics
source target identity
source-oriented formatting
```

P2 consumes P1 canonical terms. It must not invent alternate source-only semantic pack structures that bypass the TypeStore.

## 28.3 `LANG007.C2.P3` — Whole-Pack Inference, Persistence/Incremental Certification, and Checkpoint Closure

P3 owns checkpoint-wide integration and closure:

```text
whole-pack inference certification
materialization through all generic paths
metadata round-trip / validation / schema compatibility
public fingerprints
cross-module persistence
incremental/cold parity
source-index parity
hostile duplicate-label substitutions
large/dense symbolic pack behavior
fixed-generic compatibility
runtime-boundary certification
roadmap re-baselining
C2 walkthrough/handoff/closure
```

P3 must not smuggle pack-decomposition inference or runtime ABI work into C2 merely to fill a third plan.

---

# 29. Verification strategy

C2 verification should be progressive rather than reflexively workspace-wide.

## 29.1 P1 gates

Required focused proof:

```text
complete-pack kind/domain is exact
binder mode is distinct from kind
parameter semantic identity changes when mode changes
positional concat canonical laws
labeled concat canonical laws
concrete duplicate rejection
symbolic duplicate residual preservation
post-substitution duplicate invalidation
complete lane-wise composition
TupleFromPacks reuse
metadata node/fingerprint representation
no query-local IDs in canonical terms
```

## 29.2 P2 gates

Required source proof:

```text
binder syntax parses on all owned declaration sites
* / ** / *** generic expansion parses
... is rejected as pack syntax
empty <> distinguishes saturation from unsaturated constructor
binder-aware application is deterministic
canonical Applied arguments are binder-aligned
Tuple expansion lowers through P1 pack terms
callable-domain type unpacking forms correct static domain
operator/operand diagnostics are targeted
source references target one canonical parameter identity
```

## 29.3 P3 gates

Required checkpoint proof:

```text
whole-pack inference succeeds without decomposition
cross-module metadata round-trip
incremental mode-only edits invalidate dependents
incremental concat edits equal cold build
fixed generic programs retain semantics
C1 TypePack/LabeledTypePack/Map/Projection tests remain green
LabeledTypePack remains distinct from RecordRow
no VM opcode/runtime pack type added by C2
resource behavior remains linear/bounded
```

Affected-crate checks should precede any broader suite. Full workspace/release testing is not a default P1 obligation.

---

# 30. Inherited baseline failure policy

C1 recorded two unrelated trait baseline failures:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Treat them as inherited baseline failures unless C2 changes the relevant trait/dependency architecture or new evidence shows causal interaction.

Do not repair them merely to make a broad suite aesthetically green.

If a C2 change causes a new failure in the same area:

1. reproduce on the C2 predecessor revision;
2. classify baseline vs regression with evidence;
3. only widen C2 scope when the failure is causally owned by C2.

---

# 31. STOP / CONSULT matrix

The following require explicit consultation if the live accepted corpus does not already settle them.

| Question | Current analysis | Can C2 planning proceed? |
|---|---|---|
| Does the complete pack need an exact semantic domain? | **Yes, fixed.** | yes |
| Does the live architecture require a new internal complete-pack kind? | **Strongly yes**: no existing exact product kind exists. Exact Rust name remains flexible. | yes, unless a contrary exact existing abstraction appears |
| Is `CompleteTypePack` a public source kind name? | Not established by current `KindSyntax`; do not expose by assumption. | yes, keep internal |
| May pack binders carry `+` / `-` variance? | No C2 variance semantics accepted; reject in C2. | yes |
| Are `Ts |= Trait` constraints elementwise? | No. Quantified pack constraints are out of scope. | yes |
| Is explicit labeled generic argument syntax introduced? | No current authority found. Do not invent it. | yes |
| May `***` alias a single-lane source? | No; ratified algebra forbids this. | yes |
| May generic binding backtrack to partition packs? | No; C2 explicitly excludes decomposition. | yes |
| How is symbolic labeled disjointness represented internally? | Semantic outcome fixed; representation flexible. | yes; P1 chooses within existing normalization architecture |
| Must a mode-only parameter edit allocate/version semantic parameter identity? | Yes; public interface changes. | yes |
| Should `ExpansionMode` be reused for generic expansion syntax? | Existing enum is semantically aligned; reuse/generalize preferred. | yes, unless ownership coupling makes it unsafe |
| Should generic binder mode reuse runtime `RestMode`? | Not automatically; runtime/rest ownership differs. | yes; P1/P2 may introduce binder-specific mode |
| Does explicit empty `<>` need parser invention? | Live parser appears already capable; semantic binder-aware handling is the real gap. | yes |
| Are all generic-capable declaration sites enabled at once? | Default yes, or intentionally diagnose unsupported sites. Silent partial support is forbidden. | yes |

A consultation packet should state:

```text
Observed
Competing interpretations
Repository evidence
Architectural consequences
Recommended resolution
Whether work can continue around the issue
```

---

# 32. Explicit non-goals

C2 does not implement:

```text
pack decomposition solver
prefix/suffix pack inference
pack-length arithmetic
public Pack::Length
public Pack::Slice
public generic pack-concat operator unless separately ratified
arbitrary user-defined expansion protocols
general nominal const generics
trait const generics
GATs
open type families
implicit union distribution
pack-wide quantified trait constraints
pack covariance/contravariance
runtime TypePack/LabeledTypePack/CompleteTypePack values
runtime pack-solving bytecode
runtime callable ABI redesign
runtime F.3 rest-capture migration
general reflection UI for packs
release certification unrelated to the checkpoint
```

Internal concat terms are required implementation machinery and do not imply public concat syntax.

---

# 33. C2 durable invariants

The following should be treated as checkpoint-level acceptance laws.

```text
C2-01  * always denotes the positional lane.
C2-02  ** always denotes the labeled lane.
C2-03  *** always denotes the complete two-lane pack.
C2-04  Grammar context changes capture/expansion/unpacking, not lane meaning.
C2-05  ... is not generic/type-pack syntax.
C2-06  * binders capture into TypePack.
C2-07  ** binders capture into LabeledTypePack.
C2-08  *** binders capture into an exact complete-pack domain.
C2-09  Complete packs are not Tuple proper types.
C2-10  Complete packs are not TypePack or LabeledTypePack.
C2-11  Generic binder mode is orthogonal to parameter kind.
C2-12  One variadic binder still owns one TypeParameterId.
C2-13  Binder mode participates semantic/public interface identity.
C2-14  Canonical generic applications contain one argument per declared binder.
C2-15  Positional symbolic composition is flat, ordered, and canonical.
C2-16  Labeled symbolic composition preserves order and uniqueness obligations.
C2-17  Known duplicate labels are invalid; unknown overlap remains symbolic.
C2-18  Substitution cannot hide a duplicate-label error.
C2-19  Complete composition is lane-wise and never flattens labeled payloads.
C2-20  Tuple formation reuses C1 TupleFromPacks and Unit canonicalization.
C2-21  C1 Map/projection results are expandable according to semantic domain.
C2-22  Whole-pack inference is allowed; pack decomposition is not.
C2-23  No PackVarId is introduced without a distinct solver-lifetime need.
C2-24  GenericInstantiation remains the canonical non-row binding authority.
C2-25  LabeledTypePack remains distinct from RecordRow.
C2-26  Static complete packs remain distinct from runtime ArgumentPack/ArgumentPackType.
C2-27  No C2 type-pack operation executes in the VM.
C2-28  Stable metadata contains no raw/query-local IDs.
C2-29  Mode-only changes affect public fingerprints/invalidation.
C2-30  Cold and incremental compilation converge to the same semantic state.
C2-31  Existing fixed-arity generic programs retain pre-C2 semantics.
C2-32  Pack assembly and normalization are deterministic and non-backtracking.
```

---

# 34. Completion truth table

C2 may be closed only when every row below is true.

| Area | Required completion truth |
|---|---|
| semantic domains | complete two-lane pack has exact canonical domain and term |
| binder model | Single / `*` / `**` / `***` modes are represented independently of kind |
| canonicalization | positional/labeled concat laws and duplicate-label semantics proven |
| generic application | source packs bind deterministically into one canonical argument per binder |
| empty capture | explicit zero-pack specialization is distinct from unsaturated constructor |
| Tuple type expansion | symbolic and concrete expansion lowers through C1 TupleFromPacks |
| callable type unpacking | ratified static `*` / `**` / `***` domain forms implemented |
| substitution | all C2 terms substitute/materialize and recanonicalize correctly |
| inference | whole-pack inference works; decomposition remains unsupported and diagnosed |
| metadata | complete/concat/mode state serializes, validates, fingerprints, and round-trips |
| source identity | binder references target canonical parameter identity |
| incremental | cold/incremental parity established for mode and pack edits |
| diagnostics | invalid ordering/domain/duplicates/decomposition produce targeted errors |
| compatibility | C1 pack, Tuple, Map, projection, row distinctions remain green |
| runtime boundary | no runtime pack solver/value/opcode introduced by C2 |
| performance | no backtracking or concat-depth pathologies |
| lifecycle | P1/P2/P3 walkthroughs/handoffs and PROGRAM/STATUS updates accurately reflect reality |

If any row remains false, C2 is not complete even if the headline syntax works.

---

# 35. Lifecycle and documentation obligations

C2 execution should maintain:

```text
docs/implementation/LANG007/LANG007.C2/
    LANG007.C2-CHECKPOINT.md
    LANG007.C2-GUIDANCE.md
    LANG007.C2.P1-<name>.md
    LANG007.C2.P1-walkthrough.md
    LANG007.C2.P1-handoff.md
    LANG007.C2.P2-<name>.md
    LANG007.C2.P2-walkthrough.md
    LANG007.C2.P2-handoff.md
    LANG007.C2.P3-<name>.md
    LANG007.C2.P3-walkthrough.md
    LANG007.C2.P3-handoff.md
```

and update, at appropriate lifecycle events:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
```

Current active roadmap prose that still uses obsolete `...P` pack syntax should be corrected when C2 lifecycle work is committed. Historical completed C1 records should remain historical rather than being rewritten to erase prior planning context.

---

# 36. Final checkpoint requirement

The architectural success criterion is concise:

> `LANG007.C2` must make variadic generic/type-level source syntax a deterministic binder-and-composition layer over C1's canonical kinded structural terms. Unknown pack cardinality remains symbolic; labels remain labels; public generic identity records binder mode; concrete results collapse to existing canonical forms; whole-pack inference reuses the existing solver; persistence and incremental products remain stable; and no runtime pack solver or second type system is introduced.

That is the boundary every C2 plan and implementation patch must preserve.
