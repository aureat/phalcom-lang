---
id: LANG007.C2.GUIDANCE
category: LANG
program: LANG007
checkpoint: LANG007.C2
kind: implementer-guidance
status: ACTIVE_PLANNING_GUIDANCE
repository: aureat/phalcom-lang
branch: main
requirements_baseline_revision: 1a1e5ab022c84ad979a28994a5a135121baaaeaa
requirements_baseline_subject: "feat(lang007): continue C2 universe implementation"
prepared: 2026-09-22
requires:
  - LANG007.C2-CHECKPOINT.md
  - LANG007.C2-requirements-analysis.md
  - LANG007.C1-CHECKPOINT.md
  - LANG007.C1-GUIDANCE.md
---

# Implementer Guidance — LANG007.C2 Variadic Generic Packs and Type-Level Expansion

## 0. Purpose

This file is persistent architectural and execution guidance for **all** `LANG007.C2` plans. It is intentionally broader than a P1 patch plan. P1, P2, and P3 should each consume this guidance and then define exact dependency-ordered tasks, files, tests, and gates appropriate to that plan.

Load this document before working on:

```text
variadic generic binders
* / ** / *** generic application expansion
type-level lane projection
symbolic TypePack/LabeledTypePack composition
complete two-lane type packs
Tuple type expansion
callable-domain type unpacking
pack-aware generic inference/materialization
pack metadata/fingerprints
pack-related incremental invalidation
```

The C2 semantic sentence to keep loaded is:

> Variadicity is source binding and symbolic lane composition over C1's canonical kinded structural domains. `*`, `**`, and `***` have exactly one lane meaning; canonical application remains binder-aligned; unknown cardinality remains symbolic; and the VM never becomes a type-pack solver.

Every C2 patch must preserve that sentence.

Checkpoint lifecycle truth belongs in:

```text
LANG007.C2-CHECKPOINT.md
```

Exact implementation mechanics belong in the active numbered plan.

---

# 1. Session bootstrap

At the start of every C2 implementation session, establish the actual local state:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then read, in order:

```text
1. AGENTS.md and repository workflow instructions
2. docs/implementation/LANG007/PROGRAM.md
3. docs/implementation/LANG007/STATUS.md
4. LANG007.C2-CHECKPOINT.md
5. LANG007.C2-GUIDANCE.md
6. active C2 numbered plan
7. latest completed C2 walkthrough/handoff, if any
8. completed LANG007.C1 checkpoint/guidance/handoff
9. live accepted pack/callable specifications
10. exact production code owned by the active task
```

Answer privately before editing:

```text
Which C2 plan/task/gate am I implementing?
What is the actual local revision?
Are there unrelated edits I must preserve?
Which lane(s) does this source form own?
What is the canonical semantic term after elaboration?
Am I reusing C1 TypeId / TypePack / projection / substitution authorities?
Am I introducing a duplicate pack or generic representation?
Could this change make a symbolic labeled pack become invalid after substitution?
Does binder mode participate identity/fingerprints here?
Am I accidentally pulling runtime ArgumentPack/ABI work into C2?
What focused test distinguishes the intended semantics from the nearest wrong design?
Which STOP/CONSULT trigger applies if live source contradicts the plan?
```

Never reset or clean unrelated workspace changes merely to make the C2 tree convenient.

---

# 2. Authority and drift rules

Use this authority order:

```text
1. explicit user decisions / accepted LANG007 amendments
2. ratified current specs
3. accepted LANG007.C2 requirements analysis
4. LANG007.C2-CHECKPOINT.md
5. completed C1 architecture and C2 walkthroughs/handoffs
6. live production code for mechanical facts
7. active numbered plan
8. historical roadmaps/drafts
```

Mechanical drift examples that may be adapted locally:

```text
helper renamed
module split
private struct moved
function takes an additional context argument
test module relocated
```

Architectural drift examples that require consultation:

```text
TypeId is no longer the canonical kinded carrier
GenericInstantiation no longer owns generic parameter bindings
TypeParameterId no longer identifies a binder slot
pack terms are moved into a runtime object system
metadata no longer represents canonical kinded forms
source identity becomes token/string based rather than semantic
pack inference requires a new persistent solver identity
complete packs are already represented by another accepted exact domain
```

Do not “follow the plan” through architectural contradiction. Reconcile first.

---

# 3. The four distinctions that prevent most C2 design errors

Keep these separate at all times.

## 3.1 Domain vs binder mode

```text
P: TypePack
    domain = TypePack
    mode   = Single

*Ts
    domain = TypePack
    mode   = PositionalPack
```

Same domain, different source interface.

## 3.2 Complete pack vs Tuple

```text
CompleteTypePack(P, L)
    type-level lane product
    not a proper Type

TupleFromPacks(P, L)
    proper structural Tuple type
```

A complete pack may form a Tuple, but is not the Tuple.

## 3.3 Compile-time pack vs runtime ArgumentPack

```text
TypePack / LabeledTypePack / CompleteTypePack
    compiler type-level terms

ArgumentPack
    runtime/pre-dispatch value shape

ArgumentPackType
    reflective/callable type concept where accepted
```

Same lane algebra does not imply same object.

## 3.4 Composition vs decomposition

```text
[Int] ++ P ++ [String]
    C2 composition

solve P from P ++ [String] = [...]
    later decomposition problem
```

C2 builds symbolic sequences. It does not solve arbitrary sequence equations.

---

# 4. Live repository takeover map

The audited remote-main revision is:

```text
1a1e5ab022c84ad979a28994a5a135121baaaeaa
```

The following live structures matter to C2.

## 4.1 AST and parser

### Existing lane tokens

`phalcom-ast/src/token.rs` already contains distinct tokens for:

```rust
Asterisk
DoubleAsterisk
TripleAsterisk
DotDotDot
```

Do not add another lexical representation for C2 pack operators. Add lexer tests only where C2 grammar interaction could regress longest-match behavior.

### Existing lane mode

`phalcom-ast/src/ast.rs` already contains:

```rust
pub enum ExpansionMode {
    Positional,
    Labeled,
    Complete,
}
```

This is semantically aligned with `*` / `**` / `***` expansion. Prefer reusing it for C2 expansion-bearing AST nodes rather than inventing another three-variant lane enum.

Do **not** automatically reuse:

```rust
RestMode {
    None,
    Positional,
    Labeled,
    Complete,
}
```

for generic binders merely because its variants look similar. `RestMode` currently belongs to runtime callable parameter capture semantics. Generic binder mode is a public generic-signature property with different lifecycle, metadata, and identity obligations. A shared lower-level `PackLaneMode` abstraction is acceptable if it genuinely improves ownership; aliasing generic semantics directly to runtime rest metadata is not required and may be misleading.

### Generic binder gap

Current:

```rust
pub struct GenericParameterSyntax {
    pub variance: VarianceSyntax,
    pub name: String,
    pub name_range: SourceRange,
    pub kind: Option<KindSyntax>,
    pub range: SourceRange,
}
```

C2 requires a binder-mode field and, ideally, a distinct operator range so diagnostics/navigation do not have to infer the prefix span from the whole parameter range.

### Generic application gap

Current:

```rust
TypeAnnotationExpr::Application {
    origin: Box<TypeAnnotation>,
    arguments: Vec<TypeAnnotation>,
    range: SourceRange,
}
```

This cannot preserve whether an argument was:

```phalcom
T
*P
**L
***Q
```

C2.P2 needs an item representation carrying ordinary arguments and lane expansion.

### Tuple type gap

Current `TypeTupleElement` carries one optional label and one type. It does not represent a pack expansion entry. Do not fake `*P` as an unlabeled ordinary element: that loses unknown cardinality and causes Tuple formation to require a concrete vector prematurely.

### Callable type gap

Current `TypeCallableParameter` stores:

```rust
rest: bool
```

This is insufficient for ratified static callable unpacking because it cannot distinguish `*`, `**`, and `***`.

## 4.2 Semantic kinds and terms

Current base kinds:

```rust
KindData::Type
KindData::RecordRow
KindData::TypePack
KindData::LabeledTypePack
KindData::Arrow { ... }
```

Current C1 pack terms:

```rust
TypeData::TypePack(...)
TypeData::LabeledTypePack(...)
TypeData::TupleFromPacks { positionals, labeled }
```

C2 requires exact complete-pack and symbolic concat forms.

## 4.3 Generic parameter/signature state

Current:

```rust
TypeParameterData {
    owner,
    index,
    name,
    kind,
    variance,
    source,
}

GenericSignature {
    owner,
    parameters,
    parameter_kinds,
    parameter_kind_shapes,
    parameter_variances,
    constraint_shapes,
    constraints,
}
```

No binder mode exists. Add one canonical source of truth; do not scatter mode only through syntax and reconstruct it later.

## 4.4 Generic instantiation

Current:

```rust
GenericInstantiation {
    type_bindings: HashMap<TypeParameterId, TypeId>,
    row_bindings: HashMap<TypeParameterId, RecordRowId>,
}
```

This is already adequate for `TypePack`, `LabeledTypePack`, and a future complete-pack term because those are kinded `TypeId` values. Keep it.

## 4.5 Inference

`GenericApplicationSession` currently allocates:

```text
RecordRowVarId for RecordRow parameters
InferVarId(kind) for every other kinded parameter
```

This is exactly the right lifetime model for whole-pack inference. A future complete-pack kind should flow through the ordinary kind-aware `InferVarId` path.

Do not create `PackVarId` merely because a pack is variadic in source.

## 4.6 Metadata

Current stable products include:

```rust
KindNode::{Type, RecordRow, TypePack, LabeledTypePack, Arrow}

TypeNode::{
    TypePack,
    LabeledTypePack,
    TupleFromPacks,
    ...
}

TypeParameterRecord {
    id,
    name,
    kind,
    variance,
    source,
}
```

C2 has durable schema work. Treat metadata as part of the semantic implementation, not cleanup after syntax is done.

---

# 5. Fixed, flexible, and verify-first decisions

| Topic | Classification | Guidance |
|---|---|---|
| `*` / `**` / `***` lane meanings | FIXED | positional / labeled / complete everywhere |
| use of `...` for packs | FIXED PROHIBITION | never repurpose it |
| `LabeledTypePack != RecordRow` | FIXED | preserve ordered labeled pack identity |
| complete pack distinct from Tuple | FIXED | complete pack is a type-level lane product |
| complete pack exact domain | FIXED | must not be typed as arbitrary `Type` or a single-lane kind |
| internal complete-pack kind | STRONGLY DETERMINED BY LIVE ARCHITECTURE | add unless an exact existing product-kind abstraction is found |
| public source name for complete kind | VERIFY / CONSULT | do not expose by assumption |
| `GenericParameterMode` exact Rust name | FLEXIBLE | semantic field is mandatory |
| reuse `ExpansionMode` for expansion AST | PREFERRED | existing semantic match; avoid duplicate enum |
| reuse `RestMode` for generic binders | VERIFY-FIRST | likely avoid direct coupling |
| concat node exact storage | FLEXIBLE | flat canonical segments, never recursive pathological tree |
| labeled-disjointness encoding | FLEXIBLE | result semantics fixed; separate theorem prover not required |
| pack variance | FIXED OUT-OF-SCOPE | invariant; reject explicit variance markers |
| elementwise constraints | FIXED OUT-OF-SCOPE | do not reinterpret existing constraints |
| explicit labeled generic args | NOT RATIFIED | do not invent |
| empty `<>` parser mechanics | VERIFY-FIRST | parser already appears to represent it; semantic binding is the gap |
| whole-pack inference | FIXED IN-SCOPE | reuse ordinary kind-aware inference |
| pack decomposition inference | FIXED OUT-OF-SCOPE | diagnose/leave unsolved, no backtracking |
| runtime ArgumentPack ABI | FIXED OUT-OF-SCOPE | C2 is static type/generic work |

---

# 6. P1 guidance — canonical pack composition and binder semantics

P1 should establish the semantic layer that P2 can consume without rediscovering pack theory.

## 6.1 P1 objective

A good bounded objective is:

> Establish exact complete-pack kinding, canonical two-lane pack terms, flat positional/labeled symbolic composition, binder-mode identity, checked normalization under substitution, and stable metadata foundations while preserving all C1 fixed-generic and structural-pack behavior.

P1 should not attempt the entire parser/application surface merely to demonstrate the semantic nodes.

## 6.2 P1 semantic kind work

If no exact existing product kind has appeared since the baseline audit, extend:

```text
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/store.rs
phalcom-type-meta/src/kind.rs
metadata exporter/validator
```

with an internal exact complete-pack domain.

Required kind laws:

```text
kind(TypePack(...))                = TypePack
kind(LabeledTypePack(...))         = LabeledTypePack
kind(CompleteTypePack(P,L))        = CompleteTypePack
kind(TupleFromPacks(P,L))          = Type
```

and generic constructor kinds remain binder-aligned:

```text
class C<T, *Ts>
    canonical constructor kind ~ (Type, TypePack) -> Type

class C<T, **Ls>
    canonical constructor kind ~ (Type, LabeledTypePack) -> Type

class C<T, ***P>
    canonical constructor kind ~ (Type, CompleteTypePack) -> Type
```

Source variadicity is in the signature mode, not in a “variadic arrow kind.”

## 6.3 P1 binder-mode work

Add binder mode to the canonical semantic parameter/signature model.

Preferred invariant:

```text
signature.parameters[i]
signature.parameter_kinds[i]
signature.parameter_modes[i]
signature.parameter_variances[i]
```

all describe the same binder slot.

Do not infer mode from kind:

```text
kind TypePack
    could mean Single or PositionalPack
```

### Critical interning rule

Audit `TypeStore::intern_type_parameter` and its semantic-version/reuse logic. Include mode when deciding whether an existing owner/index slot still denotes the same parameter semantics.

A mode-only edit must not silently reuse stale semantics.

## 6.4 P1 complete-pack constructor

Prefer an API that validates child domains:

```rust
complete_type_pack(positionals: TypeId, labeled: TypeId) -> Result<TypeId, ...>
```

with:

```text
positionals.kind == TypePack
labeled.kind     == LabeledTypePack
```

and canonical interning on the pair.

Do not auto-convert it to Tuple. The caller requesting Tuple formation must explicitly use `tuple_from_packs`.

## 6.5 P1 positional concat

The easiest architecture compatible with existing `TypeId` terms is usually a concat node whose children are `TypePack`-kinded `TypeId` segments. Concrete inline pieces can first become ordinary C1 `TypePack` nodes.

Canonicalizer sketch:

```text
normalize_positional_concat(inputs):
    flat = []
    concrete_buffer = []

    for input in inputs:
        require kind(input) == TypePack

        if input is empty concrete TypePack:
            continue

        if input is TypePackConcat(children):
            recursively consume children into this same flat pass
            continue

        if input is concrete TypePack(elements):
            append elements to concrete_buffer
            continue

        flush concrete_buffer as one TypePack segment
        append symbolic input

    flush concrete_buffer

    if flat is empty: return TypePack[]
    if flat has one segment: return that segment
    if all segments are concrete after merging: return one TypePack
    return canonical flat TypePackConcat(flat)
```

Keep construction linear. Do not recursively intern a binary concat chain.

## 6.6 P1 labeled concat

Labeled composition needs the same flatten/merge behavior plus duplicate checking.

Canonicalizer sketch:

```text
normalize_labeled_concat(inputs):
    flat = []
    concrete_buffer = []
    known_labels = set()
    has_symbolic_segment = false

    for input in inputs:
        require kind(input) == LabeledTypePack

        flatten nested concat
        drop empty concrete segment

        if concrete segment:
            for entry in segment:
                if entry.label already proven in known_labels:
                    return Invalid(DuplicateLabel(entry.label))
                record label
                append entry to concrete_buffer
        else:
            flush concrete_buffer
            append symbolic input
            has_symbolic_segment = true

    flush concrete_buffer

    // Presence of symbolic segments means cross-segment overlap may remain unknown.
    // Preserve a residual concat rather than pretending disjointness is proven.

    canonicalize empty / singleton / fully concrete cases
```

Important: a symbolic segment may later reveal a label already present in a concrete segment or another symbolic segment. The node itself is a deferred uniqueness obligation.

## 6.7 P1 checked recanonicalization

Do not force a fallible post-substitution pack operation through an infallible API that hides failure.

The repository already has a normalization discipline for projections with outcomes such as normalized/symbolic/incomplete/invalid-like states. C2 should either reuse that pattern or introduce a narrowly scoped checked pack-normalization result.

One acceptable conceptual shape:

```rust
enum PackNormalizationResult {
    Reduced(TypeId),
    Symbolic(TypeId),
    Invalid(PackNormalizationError),
}
```

The exact enum name is not important. The semantic distinction is.

A bad implementation is:

```rust
store.labeled_type_pack(...).unwrap_or(original_concat)
```

when substitution has made a duplicate concrete. That preserves an invalid term as if nothing happened.

## 6.8 P1 metadata work

P1 should add stable schema support for any new canonical term it creates. Do not postpone all metadata to P3 if P1 terms can cross module/cache boundaries.

Expected additions:

```text
KindNode::CompleteTypePack        // exact name flexible
TypeNode::CompleteTypePack
TypeNode::TypePackConcat
TypeNode::LabeledTypePackConcat
binder-mode field/enum in TypeParameterRecord or equivalent stable signature record
```

Structural fingerprints must include ordered child fingerprints and explicit node tags.

## 6.9 P1 tests

At minimum:

```text
kind/domain tests
binder mode != kind tests
mode changes parameter semantic version/identity
complete pack child validation
empty complete pack
positional concat empty/single/flatten/merge/symbolic
labeled concat empty/single/flatten/merge/symbolic
known duplicate invalid
unknown overlap symbolic
post-substitution duplicate invalid
TupleFromPacks receives projected child lanes
metadata round-trip/fingerprint for new terms
query-local infer IDs rejected from durable metadata
```

---

# 7. P2 guidance — source binding, application, Tuple/callable type expansion

P2 should be an elaboration plan, not a second semantic-pack implementation.

## 7.1 P2 objective

> Parse and elaborate `*`, `**`, and `***` generic binders and type-level expansion items into P1's canonical terms, bind generic applications deterministically by lane, preserve explicit empty specialization, wire structural Tuple and callable-domain unpacking, and provide source-oriented diagnostics/navigation without changing runtime ABI semantics.

## 7.2 Generic binder AST

Conceptually:

```rust
pub struct GenericParameterSyntax {
    pub mode: GenericParameterModeSyntax,
    pub mode_range: Option<SourceRange>,
    pub variance: VarianceSyntax,
    pub name: String,
    pub name_range: SourceRange,
    pub kind: Option<KindSyntax>,
    pub range: SourceRange,
}
```

For source pack binders, the lane prefix determines the default semantic domain:

```text
*Ts   -> mode PositionalPack, domain TypePack
**Ls  -> mode LabeledPack,    domain LabeledTypePack
***P  -> mode CompletePack,   domain CompleteTypePack
```

If syntax permits an explicit kind after a pack binder, P2 must define whether it is redundant validation or illegal. Do not accept contradictory mode/kind combinations. The simplest coherent model is that pack mode implies its domain and any explicit kind must exactly agree if such syntax is retained.

## 7.3 Binder-order validation

Perform deterministic binder-shape validation once, ideally during signature formation rather than re-deriving it at every application.

A useful state machine:

```text
state FixedPrefix
    Single -> FixedPrefix
    *      -> PositionalCaptured
    **     -> LabeledCaptured
    ***    -> CompleteTerminal

state PositionalCaptured
    **     -> SplitTerminal
    others that require positional source ownership -> error

state LabeledCaptured
    no further pack owner -> terminal/error according to accepted grammar

state SplitTerminal
    no further binder that changes lane ownership

state CompleteTerminal
    no further binder
```

Exact fixed/labeled interactions must follow accepted generic rules, but never add search/backtracking to make an otherwise ambiguous ordering work.

## 7.4 Generic argument AST

Use an item representation, not parallel ad-hoc vectors that lose source order.

Example:

```rust
enum GenericArgumentSyntax {
    Single(TypeAnnotation),
    Expansion {
        mode: ExpansionMode,
        operator_range: SourceRange,
        operand: TypeAnnotation,
        range: SourceRange,
    },
}
```

Then semantic elaboration can assemble two lane vectors/terms while retaining source-order diagnostics.

## 7.5 Generic application elaboration algorithm

Use a two-stage design:

### Stage A — evaluate source contributions into a generic argument pack

```text
positionals : ordered list/TypePack-term contributions
labeled     : ordered LabeledTypePack-term contributions
```

For each source item:

```text
Single(T)
    -> append proper Type T to positional lane

*P
    -> project/require positional lane, append TypePack contribution

**L
    -> project/require labeled lane, append LabeledTypePack contribution

***Q
    -> require complete-capable source,
       append Q.positionals to positional lane,
       append Q.labeled to labeled lane
```

Use P1 concat canonicalizers; do not materialize unknown packs into `Vec<TypeId>`.

### Stage B — bind the assembled lanes against the declaration signature

Conceptual binder algorithm:

```text
for each Single binder in fixed prefix:
    consume one positional proper Type

for * binder:
    bind remaining positional lane assigned to that binder as one TypePack

for ** binder:
    bind labeled lane as one LabeledTypePack

for *** binder:
    bind remaining positional + labeled lanes as one CompleteTypePack

if any source lane remains unowned:
    diagnose extra arguments / labels

emit exactly one canonical semantic argument per declared binder
```

Then call existing kind-aware application machinery using those canonical binder-aligned arguments.

Do not let `TypeStore::apply_type_form` become source-variadic. It should continue receiving canonical semantic arguments.

## 7.6 Explicit empty application

The live parser already appears capable of producing `Application { arguments: [] }` for `V<>`. Verify with a focused parser test before modifying parser mechanics.

The semantic distinction belongs in binder-aware application:

```text
origin alone:
    unsaturated constructor

origin <> against <*Ts>:
    one bound parameter Ts = TypePack[]
    canonical Applied(origin, [emptyPack])
```

Do not globally change `TypeStore::apply_type_form(origin, &[])` to manufacture an applied node, because fixed constructors and existing higher-kinded behavior rely on ordinary empty-application semantics. Elaborate the empty source application into nonempty **binder-aligned canonical arguments** when variadic binders require it.

## 7.7 Type-level lane projection lowering

Prefer reuse of C1 terms:

```text
*TypePackTerm
    -> identity

**LabeledTypePackTerm
    -> identity

*CompletePack
    -> complete.positionals

**CompletePack
    -> complete.labeled

***CompletePack
    -> identity

*TupleTerm
    -> C1 Positionals projection

**TupleTerm
    -> C1 Labeled projection

***TupleTerm
    -> CompleteTypePack(
           C1 Positionals projection,
           C1 Labeled projection
       )
```

This is how C2 remains an extension of C1 rather than a second projection language.

## 7.8 Tuple type expansion

Do not make `TypeTupleElement.ty` pretend an expanded pack is one element.

Use a lane-aware tuple item model, for example:

```rust
enum TypeTupleItem {
    Element(TypeTupleElement),
    Expansion {
        mode: ExpansionMode,
        operand: TypeAnnotation,
        ...
    },
}
```

Elaboration assembles positional and labeled pack terms, then calls:

```text
TypeStore::tuple_from_packs(positionals, labeled)
```

This automatically preserves C1's concrete collapse and Unit canonicalization.

## 7.9 Callable-domain static unpacking

The accepted static forms:

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

should use the same lane-projection/assembly semantics as Tuple type formation, then lower to the accepted callable-domain representation.

Do not wire this through runtime `ParameterDef::rest_mode` just because the surface operators look the same. Static callable Type formation and runtime rest capture are different compiler layers.

## 7.10 Declaration-site rollout

Audit every AST owner of `Vec<GenericParameterSyntax>`. A good P2 implementation centralizes generic-binder parsing/signature lowering so most sites gain C2 behavior automatically.

If one site is intentionally deferred, add an explicit diagnostic and record it as a checkpoint exception. Do not allow “parses but semantic layer ignores mode” behavior.

## 7.11 P2 tests

Minimum source-level matrix:

```phalcom
class V<*Ts> {}
class V<**Ls> {}
class V<***P> {}
class V<T, *Ts> {}
class V<T, *Ts, **Ls> {}
class V<T, ***P> {}
```

Applications:

```text
zero positional capture
one positional capture
many positional capture
symbolic * expansion
symbolic ** expansion
symbolic *** expansion
mixed single + * + **
mixed single + ***
explicit V<>
fixed-only empty application rejection
extra positional lane rejection
extra labeled lane rejection
***TypePack rejection
***LabeledTypePack rejection
...P rejection
```

Tuple/callable tests must include symbolic `Map<F>` operands to prove operations are kind/domain based rather than bare-variable special cases.

---

# 8. P3 guidance — inference, persistence, incremental certification, closure

P3 should prove the C2 vertical is complete. It should not become a dumping ground for unresolved semantic design.

## 8.1 P3 objective

> Certify whole-pack inference and materialization through all C2 forms, durable cross-module metadata and public fingerprints, incremental/cold equivalence, source identity, hostile symbolic duplicate-label transitions, fixed-generic compatibility, performance bounds, and the compile-time/runtime boundary; then close the checkpoint and rebaseline future LANG007 work.

## 8.2 Whole-pack inference

Because pack terms are ordinary kinded `TypeId` forms, the desired inference path is:

```text
TypeParameterId(kind TypePack)
    -> InferVarId(kind TypePack)

TypeParameterId(kind LabeledTypePack)
    -> InferVarId(kind LabeledTypePack)

TypeParameterId(kind CompleteTypePack)
    -> InferVarId(kind CompleteTypePack)
```

Then bind the whole term.

P3 must explicitly prove that no path accidentally attempts to decompose a concat equation. If a constraint would require decomposition, it should remain unsupported/blocked with a targeted diagnostic or inference outcome rather than enumerate partitions.

## 8.3 Materialization closure

Audit all `TypeData` exhaustive matches after P1/P2. Common missed paths include:

```text
contains_type_parameter
call-site “contains any generic parameter” helpers
inference conversion
recursive normalizers
metadata walkers
pretty printers
source-index type traversals
```

C1 already added pack variants in many central paths. New C2 variants must be included everywhere semantically reachable.

## 8.4 Metadata round-trip

P3 should prove:

```text
canonical C2 term
    -> stable TypeNode/KindNode/parameter records
    -> serialize/deserialize
    -> validate
    -> same structural fingerprint / semantic shape
```

Cross-module examples should include abstract symbolic concat, not only fully concrete packs.

## 8.5 Incremental parity

Required edit scenarios:

```text
Single TypePack parameter -> * binder
* binder -> *** binder
add/remove ** binder
add/remove source * expansion segment
replace concrete labeled pack with conflicting label
resolve previously symbolic label overlap into duplicate
change Tuple expansion while preserving final concrete type
```

For each, compare incremental final products against a clean/cold build.

## 8.6 Performance certification

Use large enough symbolic/concrete packs to expose:

```text
nested-concat failure to flatten
quadratic repeated copying
duplicate-label scans implemented as repeated linear search
infinite/repeated normalization cycles
workspace-wide invalidation
```

Do not turn C2 into a benchmark program; use focused scaling tests sufficient to catch architecture mistakes.

---

# 9. Canonical source-to-semantic examples

These examples should guide both implementation and tests.

## 9.1 Positional variadic binder

```phalcom
class V<Head, *Tail> {}
```

Source:

```phalcom
V<Int, String, Bool>
```

Canonical binding:

```text
Head = Int
Tail = TypePack[String, Bool]
```

Canonical applied form:

```text
Applied(V, [Int, TypePack[String, Bool]])
```

## 9.2 Empty positional capture

```phalcom
V<Int>
```

becomes:

```text
Head = Int
Tail = TypePack[]
```

not an unsaturated constructor.

## 9.3 Symbolic positional expansion

```text
P : TypePack
```

Source concept:

```phalcom
V<Int, *P, String>
```

If the binder owns the composed positional tail, canonical pack is:

```text
TypePackConcat(P, [String])
```

with the fixed prefix consumed according to signature ownership.

No `Vec<TypeId>` length for `P` is required.

## 9.4 Labeled binder

```phalcom
class L<**Fields> {}
```

with:

```text
Names : LabeledTypePack
```

and source:

```phalcom
L<**Names>
```

binds:

```text
Fields = Names
```

A second symbolic labeled contribution forms `LabeledTypePackConcat`, preserving order and deferred uniqueness.

## 9.5 Complete binder

```phalcom
class F<***Args> {}
```

with complete source `Q = ⟨P,L⟩`:

```phalcom
F<***Q>
```

binds exactly:

```text
Args = CompleteTypePack(P,L)
```

not `TupleFromPacks(P,L)`.

## 9.6 Tuple type assembly

```text
P : TypePack
L : LabeledTypePack
```

```phalcom
(Int, *P, **L)
```

becomes conceptually:

```text
TupleFromPacks(
    TypePackConcat([Int], P),
    L
)
```

When `P` and `L` later become concrete, C1 canonicalization yields the ordinary Tuple type.

## 9.7 Complete Tuple unpack

For Tuple type `T`:

```phalcom
(***T,)
```

should lower through C1 lane projections:

```text
CompleteTypePack(
    T::Positionals,
    T::Labeled
)
    ↓ tuple context
TupleFromPacks(T::Positionals, T::Labeled)
```

not through a new Tuple deconstructor.

---

# 10. Duplicate-label residual semantics in detail

Labeled composition is the part most likely to be implemented unsafely.

Consider:

```text
L1 = [(x, A)]
L2 = symbolic LabeledTypePack

ConcatLabeled(L1, L2)
```

At formation time there are three possible semantic states:

```text
1. L2 concrete and lacks x
   -> reduce to concrete merged pack

2. L2 symbolic
   -> keep canonical symbolic concat

3. L2 concrete and contains x
   -> Invalid(DuplicateLabel(x))
```

Later substitution can transition case 2 into case 1 or 3.

Therefore C2 needs an explicit place where this transition is checked. It is insufficient to validate only when parsing source literals.

A sound flow is:

```text
symbolic concat term
    ↓ substitution/materialization
children rewritten
    ↓ checked pack normalization
Reduced(new concrete/less-symbolic term)
    or Symbolic(residual)
    or Invalid(duplicate label)
```

Do not encode “unknown disjointness” as a boolean permanently set to true or false. The obligation is structural and must be re-evaluated after child substitution.

Do not introduce a general theorem prover. For C2, equality of concrete label symbols and residual unknown segments is sufficient.

---

# 11. Generic parameter identity and incremental safety

This deserves separate emphasis because it crosses semantic correctness and incremental correctness.

Current parameter identity is owner/index based with semantic versioning in the persistent TypeStore. C2 binder mode changes the public meaning of a slot without necessarily changing its kind.

Example edit:

```diff
-class Box<P: TypePack> {}
+class Box<*P> {}
```

Both forms may have:

```text
owner = Box
index = 0
name = P
kind = TypePack
variance = Invariant
```

but they are not the same interface.

Therefore mode must participate at least:

```text
TypeParameterData semantic equality/versioning
GenericSignature structural identity
metadata parameter/signature records
public/declaration fingerprint
incremental dependency key or fingerprint path
source presentation
```

A regression test should edit only the mode and prove a dependent generic application is invalidated/recomputed.

---

# 12. Generic signature and constructor-kind guidance

Do not make variadicity a new kind constructor calculus.

After source lowering, a generic declaration still has one canonical semantic parameter per binder.

Conceptually:

```phalcom
class C<T, *Ts> {}
```

becomes a constructor accepting:

```text
(Type, TypePack) -> Type
```

while its signature records:

```text
modes = [Single, PositionalPack]
```

This separation has major benefits:

```text
existing KindData::Arrow machinery remains useful
existing apply_kind remains useful
existing TypeData::Applied remains useful
existing GenericInstantiation remains useful
existing substitution remains binder-oriented
metadata has one parameter identity per source binder
```

Do not encode source-expanded arity into `Applied` identity.

---

# 13. Constraint handling guidance

Pack parameters are kinded semantic values. That does not make every ordinary generic constraint meaningful on them.

Keep existing constraint semantics.

Bad C2 shortcut:

```text
Ts |= Printable
=> assume all elements of Ts conform to Printable
```

That is a quantified constraint system and is not part of C2.

Likewise do not invent:

```text
Ts <: Us elementwise
length(Ts) == length(Us)
```

unless separately accepted.

If the existing constraint validator requires a proper `Type` subject, reject a pack-kinded subject clearly.

---

# 14. Formatting and diagnostics guidance

Internal representation should not leak into source diagnostics.

Prefer:

```text
expected a complete type-pack expansion after `***`, but this expression has positional pack domain `TypePack`
```

not:

```text
KindMismatch expected KindId(4) got KindId(2)
```

Prefer:

```text
duplicate labeled type argument `name` introduced by pack expansion
```

not:

```text
LabeledTypePackConcat normalization failed
```

Prefer source syntax in type display when possible:

```phalcom
C<Int, String, Bool>
```

rather than canonical storage display:

```text
Applied(C, [Int, [String, Bool]])
```

A debug formatter may show canonical nodes; user diagnostics should not depend on debug formatting for semantic clarity.

---

# 15. Source-index guidance

The prefix operator and the binder name are related but not identical source spans.

For:

```phalcom
class C<***Args> {}
```

source indexing should preserve:

```text
Args name range -> canonical TypeParameterId target
*** operator range -> syntax/operator occurrence associated with binder mode
whole binder range -> diagnostic/refactoring envelope
```

References to `Args` elsewhere resolve to the parameter identity, not to a synthetic “pack variable” symbol.

If the current source-index schema cannot represent the operator occurrence directly, it is acceptable for C2 to keep name navigation canonical and treat the prefix as diagnostic syntax. Do not create a second semantic symbol merely to index `***`.

---

# 16. Metadata guidance

## 16.1 Stable nodes

Every durable canonical term needs a stable structural representation.

Recommended conceptual nodes:

```rust
KindNode::CompleteTypePack

TypeNode::CompleteTypePack {
    positionals: TypeNodeId,
    labeled: TypeNodeId,
}

TypeNode::TypePackConcat(Box<[TypeNodeId]>)
TypeNode::LabeledTypePackConcat(Box<[TypeNodeId]>)
```

Names may differ. Ordered child identity may not.

## 16.2 Stable binder mode

One robust schema is:

```rust
enum GenericParameterModeRef {
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}

TypeParameterRecord {
    ...
    mode: GenericParameterModeRef,
}
```

Putting mode on the parameter record naturally makes it part of a stable parameter's public semantics. If the repository instead owns parameter mode in `GenericSignatureRecord`, that is acceptable only if a stable parameter cannot be misinterpreted independently and all fingerprints incorporate the mode.

## 16.3 Fingerprint tags

Do not recycle existing node tags in metadata fingerprinting. New structural variants need unique tagged encodings.

For concat nodes, fingerprint:

```text
node tag
segment count
ordered child structural fingerprints
```

For complete packs:

```text
node tag
positional child fingerprint
labeled child fingerprint
```

## 16.4 Schema compatibility

If metadata schema/versioning requires a feature/version bump for new serialized variants, do it deliberately. Do not rely on serde enum ordinal accidents as a compatibility story.

---

# 17. Inference guidance

## 17.1 Reuse existing query-local inference

Pack cardinality is not a reason for a separate metavariable class. A whole unknown pack is one unknown kinded term.

Desired:

```text
InferVarId(kind TypePack)
InferVarId(kind LabeledTypePack)
InferVarId(kind CompleteTypePack)
```

## 17.2 Do not leak inference variables

Before durable publication:

```text
InferVarId -> materialized canonical TypeId
```

or the product is rejected/blocked. Never serialize an inference variable in `TypePackConcat` merely because the concat node can be symbolic.

Symbolic canonical parameter/projection terms are stable. Query-local inference IDs are not.

## 17.3 Decomposition boundary tests

Add a negative solver test that would only succeed if the implementation accidentally starts solving pack partitions. It should remain unsupported.

Example conceptual equation:

```text
Concat(P, [String]) = [Int, Bool, String]
```

C2 must not infer `P = [Int, Bool]` by sequence subtraction.

---

# 18. Testing doctrine and budget

Use tests as architecture probes, not only coverage.

## 18.1 Tier 1 — owner-local unit/foundation tests

Run after each coherent semantic patch:

```text
phalcom-semantic type-store/kind tests
pack formation/canonicalization tests
generic signature/parameter tests
substitution/materialization tests
metadata node tests
phalcom-ast parser tests for modified productions
```

## 18.2 Tier 2 — focused semantic integration tests

Run after a complete task/gate:

```text
foundations::type_annotations
foundations::type_members
foundations::generic_application
relevant metadata integration tests
relevant incremental tests
```

Use exact current test module names from the live tree; do not cargo-run guessed filters without checking.

## 18.3 Tier 3 — affected-crate checks

Typical bounded checks:

```sh
cargo check -p phalcom-ast
cargo check -p phalcom-type-meta
cargo check -p phalcom-semantic
```

Add `phalcom-core` only when a C2 change actually touches or compiles through its semantic metadata/runtime typing consumers.

## 18.4 Tier 4 — broader regression suites

Run when P2/P3 integration justifies them, not after every helper edit.

Do not make P1 absorb release certification.

## 18.5 Mandatory hostile cases

Maintain explicit negative tests for:

```text
*** applied to TypePack
*** applied to LabeledTypePack
* applied to labeled-only domain
** applied to positional-only domain
multiple * binders
multiple ** binders
split capture plus ***
binder after ***
variance marker on pack binder
duplicate concrete label
symbolic duplicate becoming concrete after substitution
pack decomposition equation
... generic-pack syntax
pack-kinded term used where proper Type is required
query-local inference ID at metadata boundary
labeled pack accidentally becoming RecordRow
complete pack accidentally becoming Tuple/TypePack
```

---

# 19. Baseline failure policy

Inherited from C1:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

If a broad C2 test encounters them unchanged, record as baseline and continue focused C2 verification.

If the failure signature changes or C2 touches the same dependency/evidence path, reclassify with evidence before dismissing it.

Never fix an unrelated baseline failure solely to make a test count zero.

---

# 20. Debugging budget

When a C2 test fails, spend debugging effort in this order:

```text
1. confirm source syntax/AST shape
2. confirm binder mode and inferred semantic domain
3. inspect assembled positional/labeled contributions
4. inspect canonical concat/complete term
5. inspect binder-aligned arguments passed to apply_type_form
6. inspect substitution/materialization
7. inspect metadata/fingerprint only if semantic state is already correct
8. inspect incremental invalidation only after cold path is correct
9. inspect runtime/VM only if evidence proves C2 crossed the boundary unexpectedly
```

This ordering avoids wasting time in the VM for a compile-time type formation bug.

Do not compensate for a semantic error by widening to `Dynamic`, `Unknown`, `Never`, or an empty pack unless that is the actual language meaning.

---

# 21. STOP / CONSULT triggers

Stop and consult when any of these appears:

```text
1. An accepted spec gives a different * / ** / *** lane meaning.
2. Live main already has another exact complete-pack domain not covered by this audit.
3. Supporting complete packs appears to require representing them as proper Type/Tuple.
4. Generic application appears to require arbitrary pack partitioning/backtracking.
5. A proposed fix needs a persistent PackVarId or PackId arena.
6. Labeled symbolic composition cannot preserve duplicate uncertainty without changing a major normalizer API.
7. A user-visible public name for the complete-pack kind is required but not ratified.
8. A generic site needs explicit named generic arguments not covered by accepted syntax.
9. Pack variance becomes necessary to satisfy an accepted example.
10. An existing constraint is claimed to quantify over pack elements without a specification.
11. C2 type formation appears to require runtime ArgumentPack ABI or VM changes.
12. Metadata cannot encode binder mode without a compatibility/schema decision.
13. A local baseline failure is causally entangled with C2 rather than merely co-occurring.
14. Source pack semantics conflict with the accepted runtime pack lane ordering in a way not resolvable by grammar context.
```

Consultation packet template:

```markdown
## C2 consultation

### Observed
<live behavior/code/spec evidence>

### Required invariant
<what C2 must preserve>

### Competing interpretations
1. ...
2. ...

### Repository evidence
- path/symbol ...
- accepted spec section ...
- test ...

### Architectural consequences
<identity / inference / metadata / source / runtime effects>

### Recommended resolution
<one recommendation, or explicit no-preference if truly undecidable>

### Can implementation proceed around it?
<yes/no and bounded unaffected work>
```

Do not bury semantic uncertainty in a code comment and continue.

---

# 22. Anti-patterns explicitly forbidden

## 22.1 Parallel pack arena

Do not add:

```text
PackId
LabeledPackId
CompletePackArena
```

for canonical type-level identity while C1's kinded `TypeId` architecture remains suitable.

## 22.2 Variadic applied-type identity

Do not add:

```text
TypeData::VariadicApplied {
    source_arguments: ...
}
```

Canonical application is binder-aligned ordinary `Applied`.

## 22.3 Flatten labeled packs into positionals

Never implement complete expansion as:

```text
positionals ++ labeled payload types
```

Labels remain in the labeled lane.

## 22.4 Convert labeled packs to RecordRow

`RecordRow` sorts/key-canonicalizes fields for row semantics. `LabeledTypePack` preserves order. They are not interchangeable.

## 22.5 Eager concrete-vector expansion

Do not reject or approximate:

```text
P : TypePack
*P
```

merely because `P` is not yet a concrete `Vec<TypeId>`.

## 22.6 Last-wins duplicate labels

Duplicate labels are invalid, not overwrite semantics.

## 22.7 Hide invalidity behind old symbolic node

If substitution proves a duplicate label, report invalidity; do not preserve the pre-substitution concat as if it remained unknown.

## 22.8 Reuse `...` for convenience

The language has explicitly separated `...` from the pack lane algebra. Do not accept it as an alias.

## 22.9 Runtime pack solver

No VM opcode/message send/native class should be necessary for C2 type-level composition.

C2 also must not resurrect the retired `@intrinsic` attribute. Compiler-owned intrinsic declarations continue to use the first-class `intrinsic` keyword; internal `NativeIntrinsicId` remains a separate concept.

## 22.10 Elementwise constraint magic

Do not interpret a pack constraint as a loop over its elements without a future explicit feature.

---

# 23. Plan-writing requirements

Each numbered C2 plan should be patch-grade. Every task should include:

```text
Purpose
Preconditions
Consumes
Produces
Files/symbols
Required implementation shape
Forbidden approaches
Tests required
Tests to run now
Tests explicitly deferred
Acceptance
Local STOP/CONSULT triggers
Checkpoint update
```

Each plan should also include:

```text
front matter / identity
repository grounding revision
goal and bounded objective
normative authority
stable takeover map
global invariants
non-goals
expected impact map
fixed / flexible / verify-first decisions
STOP/CONSULT triggers
debugging budget
testing surface analysis
verification budget
baseline failure policy
dependency-ordered tasks
gates
acceptance checklist
resource expectations
lifecycle bookkeeping
walkthrough requirements
handoff requirements
completion truth table
self-review
compact executor directive
```

Do not copy P1's tasks into P2/P3 simply to preserve a three-plan shape. The plans must follow dependency ownership.

---

# 24. P1/P2/P3 gate suggestions

These are guidance, not immutable task numbering.

## P1 suggested gates

```text
G1  exact complete-pack kind + canonical term
G2  binder mode + TypeParameterId/signature identity
G3  positional/labeled concat canonicalization
G4  checked substitution/materialization + duplicate residuals
G5  metadata/fingerprint foundations + focused compatibility
```

## P2 suggested gates

```text
G1  generic binder/application AST + parser
G2  lane-aware generic argument assembly and binding
G3  explicit empty specialization + declaration-site rollout
G4  Tuple type expansion + callable-domain unpacking
G5  source diagnostics/index/presentation + focused integration
```

## P3 suggested gates

```text
G1  whole-pack inference/materialization closure
G2  stable metadata round-trip/cross-module products
G3  incremental cold parity and mode-only invalidation
G4  hostile/performance/fixed-generic compatibility matrix
G5  runtime-boundary certification + lifecycle closure
```

A plan may merge/split gates if the live dependency graph justifies it, but must not invert the basic order “canonical substrate -> source elaboration -> certification.”

---

# 25. Walkthrough requirements

Every completed C2 plan walkthrough should record:

```text
starting revision
ending revision
actual files changed
architecture implemented
important deviations from plan
consultations and resolutions
focused commands/tests and results
baseline failures encountered
performance/resource observations
lifecycle updates
remaining work for next plan
```

For P1 specifically, the walkthrough must state the final internal representation chosen for:

```text
complete-pack kind
complete-pack term
positional concat
labeled concat
deferred label uniqueness
binder mode storage
checked normalization after substitution
```

For P2, record the final AST/application-item representation and exact declaration-site rollout.

For P3, record the incremental parity matrix and C2 closure truth table.

---

# 26. Handoff requirements

A C2 handoff must not force the next agent to rediscover semantics.

It should include:

```text
current revision
workspace state expectations
completed gates
deferred baseline failures
new stable symbols/APIs introduced
canonical term examples
open consultations
exact next bounded objective
focused test commands that matter
anti-patterns that were explicitly avoided
```

P1 handoff must tell P2 exactly which canonical APIs to consume for:

```text
pack lane projection
concat formation
complete-pack formation
binder mode queries
checked normalization
```

P2 handoff must tell P3 exactly which canonical source elaboration/products need inference/persistence certification.

---

# 27. PROGRAM / STATUS maintenance

At C2 activation, update the lifecycle files according to repository convention. Do not claim implementation before work exists.

As active guidance is updated, replace obsolete future-facing pack syntax such as:

```text
...P
```

with ratified:

```text
*P
**P
***P
```

where the text is current roadmap guidance.

Do not rewrite closed C1 historical walkthroughs merely to make all old prose look current.

At C2 closure:

```text
PROGRAM.md
    records C2 COMPLETE / IMPLEMENTED / verified state
    names next checkpoint accurately

STATUS.md
    matches checkpoint truth

LANG007.C2-CHECKPOINT.md
    has completed plan ledger and verification evidence

LANG007.C2-GUIDANCE.md
    is either frozen as closed guidance or updated according to program convention
```

---

# 28. Self-review checklist for every C2 patch

Before calling a task complete, ask:

```text
[ ] Did I preserve * / ** / *** lane meanings exactly?
[ ] Did I avoid using ... as pack syntax?
[ ] Did I reuse TypeId instead of creating a second canonical pack arena?
[ ] Did I preserve LabeledTypePack order and keep it distinct from RecordRow?
[ ] Did I keep complete pack distinct from Tuple?
[ ] Does binder mode survive beyond syntax into semantic identity?
[ ] Does a mode-only edit affect fingerprints/invalidation?
[ ] Does canonical application contain one argument per binder?
[ ] Can symbolic unknown-cardinality packs remain symbolic?
[ ] Are concat terms flat/canonical rather than nested pathological trees?
[ ] Can duplicate-label invalidity surface after substitution?
[ ] Did I avoid pack decomposition/backtracking?
[ ] Did I reuse C1 Tuple projections / TupleFromPacks?
[ ] Did I preserve GenericInstantiation as binding authority?
[ ] Did I keep query-local inference variables out of stable metadata?
[ ] Did I avoid runtime ArgumentPack/VM changes?
[ ] Did I add focused hostile tests for the nearest wrong design?
[ ] Did I classify inherited baseline failures with evidence?
[ ] Did I update lifecycle state only to what was actually completed?
```

Any unchecked semantic item is a reason to keep the task open.

---

# 29. Compact executor directive

```text
Implement LANG007.C2 as a source-binding/composition extension over the completed C1 type system.

Keep * positional, ** labeled, and *** complete everywhere. Never repurpose ....

Reuse TypeId, TypePack, LabeledTypePack, TupleFromPacks, TypeParameterId, GenericInstantiation, ordinary kind-aware inference, type-member projections, and stable metadata architecture.

Add an exact complete-pack domain/term, flat symbolic positional/labeled concat, and binder mode as durable semantic identity. Make post-substitution labeled duplicates become errors rather than hidden residuals.

Elaborate source generic arguments into two lanes, then bind deterministically to one canonical semantic argument per declared binder. Do not preserve source variadic arity in Applied identity.

Support symbolic unknown-cardinality expansion, Tuple type expansion, and ratified callable-domain type unpacking without a pack-decomposition solver or runtime ABI work.

Extend stable metadata, fingerprints, source identity, and incremental products as first-class parts of the feature. Prove cold/incremental parity and fixed-generic compatibility before closing C2.

STOP if a proposed solution needs a second pack arena, a persistent pack solver, Tuple/RecordRow conflation, user-visible semantics not in authority, or VM responsibility for type-level pack computation.
```

---

# 30. Architectural north star

The intended final architecture is:

```text
source
    *  **  ***
        │
        ├── generic binder mode
        ├── generic argument expansion
        └── Tuple/callable type unpacking
                │
                ▼
       one shared lane algebra
                │
                ▼
       canonical C1-compatible domains
        TypePack
        LabeledTypePack
        CompleteTypePack
                │
                ▼
       flat symbolic composition
        TypePackConcat
        LabeledTypePackConcat
                │
                ▼
       binder-aligned application
        TypeData::Applied
        GenericInstantiation
                │
                ▼
       existing substitution / inference
        InferVarId(kind)
        no PackVarId
                │
                ▼
       TupleFromPacks / callable Type formation
                │
                ▼
       stable metadata / fingerprints / incremental state
                │
                ▼
       runtime VM receives no type-pack-solving responsibility
```

If an implementation starts to look materially different from this flow, determine whether the live repository has revealed a better accepted architecture or whether the patch has drifted. Do not normalize architectural drift by accumulating adapters around it.
