---
id: LANG007
category: LANG
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
prepared: 2026-09-21
---

# LANG007 — Type-Level Structural Domains and Intrinsic Type Operations

LANG007 owns the implementation of Phalcom's compiler-assisted type-level structural operations: explicit type-level sequence domains, structural Tuple/Record type-member projection, typed compile-time type-member arguments, intrinsic normalization, and the semantic infrastructure required to preserve those operations through inference, metadata, and canonical Universe source.

The program exists to make structural type computation a coherent part of Phalcom's type/value model without turning the VM into a type solver, without treating compile-time constants as fake types, and without collapsing distinct structural domains such as `TypePack`, `LabeledTypePack`, and `RecordRow`.

## Program ownership

LANG007 owns implementation work for:

- type-level structural domains such as `TypePack` and `LabeledTypePack`;
- compiler-native type members and their canonical semantic identity;
- typed compile-time arguments to type members where required by the accepted language design;
- Tuple structural projection and indexing;
- structural `Map<F>` over Tuple/Record-related domains;
- stable symbolic projections when a type-level computation is well formed but not yet reducible;
- substitution, inference, metadata, source indexing, and Universe integration needed to preserve those semantics;
- future LANG007 checkpoints that extend this substrate into broader variadic/type-level structural programming when separately accepted and planned.

LANG007 does not own ordinary runtime collection iteration, general VM dispatch, arbitrary compile-time evaluation, unrelated trait/conformance architecture, or general-purpose metaprogramming unless a later checkpoint explicitly expands the program's bounded objective.

## Current checkpoints

### C1 — Semantic domains and compiler-intrinsic structural type operations

**State:** `IN_PROGRESS / PARTIAL / UNVERIFIED`

C1 is complete and focused-tested across all three numbered plans (P1, P2, P3).

### C2 — Variadic Generic Pack Binding and Type-Level Pack Expansion

**State:** `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`

C2 is complete and focused-tested across all three numbered plans (P1, P2, P3). It establishes variadic generic binding (`*`, `**`, `***`), flat canonical pack compositions (`TypePackConcat`, `LabeledTypePackConcat`, `CompleteTypePack`), Tuple/callable structural expansions, whole-pack inference, durable scoped metadata persistence under schema v4, and incremental parity.

Current numbered plans for C2:

- `LANG007.C2.P1` — canonical pack composition and variadic binder semantics — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C2.P2` — source variadic generic binding and structural/callable type expansion — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C2.P3` — whole-pack inference, persistence, incremental certification, and checkpoint closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

### C3 — Type Data and Unified Algebraic Static Domains

**State:** `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`

C3 establishes unified algebraic declarations (`data` for runtime products
and sums, `type data` for type-level static domains), nominal static-domain
identity, canonical static terms, heterogeneous static applications, and
intrinsic structural-domain exposure. P3.R1 completed the corrective work that
separates intrinsic semantic application from runtime/native owner projection.
The active P4 completes structural kinds, product-shape expansion, and
Tuple/Record semantic behavior.

Current numbered plans for C3:

- `LANG007.C3.P1` — unified algebraic data declarations and type-data foundation — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P2` — general static domains, heterogeneous application/inference, associated type data — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P3` — intrinsic structural surface, Universe/corpus migration, certification and closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P3.R1` — intrinsic semantic application and runtime/native owner projections — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P4` — intrinsic structural data semantic completion — `IN_PROGRESS / PARTIAL / UNVERIFIED`.

The active objective and durable progress ledger are in
[`LANG007.C3-CHECKPOINT.md`](LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md).
No successor checkpoint or plan is currently accepted; no C4 is implied.

C1 is the first implementation checkpoint. It establishes the canonical domains, type-member identity/projection machinery, structural operations, persistence/inference integration, and runtime boundary required for:

```phalcom
(Int, String)::Positionals
(Int, String, name: Symbol)::Labeled

(Int, String)[1]
(Int, String)::At<1>
(Int, String)::At<-1>

(Int, String)::Map<Option>
(Int, name: String)::Map<Option>

#{name: String, age: Int}::Map<Option>
```

and the corresponding abstract/symbolic operations over accepted pack/row domains.

Current numbered plans:

- `LANG007.C1.P1` — semantic domains and compiler-intrinsic structural type operations — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C1.P2` — authoritative specification and `intrinsic` keyword convergence — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C1.P3` — architecture baseline, cross-layer certification, roadmap re-baselining, and C1 closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

C1 is not release certification for all future type-level programming.

## Re-baselined capability map

### Capabilities owned by C1 (DO NOT REIMPLEMENT)

- `TypePack` and `LabeledTypePack` kinds, domains, and canonical terms (`TypeData::TypePack`, `TypeData::LabeledTypePack`);
- Parameterized compiler-intrinsic type members and typed compile-time `Const(Int)` arguments;
- Canonical type-member identity, selectors, and domain registry (`types/type_member.rs`);
- Single canonical Subscript member identity for `T[I]` and `T::At<I>`;
- Symbolic type-member projections (`TypeLevelProjection`, `TypeData::TypeMemberProjection`);
- Closed compiler-side structural normalization for `Positionals`, `Labeled`, `At`, and `Map<F>`;
- Shape-preserving `Map<F>` over `Tuple`, `Record`, `TypePack`, `LabeledTypePack`, and `RecordRow`;
- Open-row mapped tails (`RecordRowTail::Projection` / `RecordRowMap`);
- Substitution, materialization, and type-inference preservation (no query-local solver variable leakage);
- Stable metadata nodes and validation (`KindNode`, `TypeNode::TypePack`, `TypeNode::LabeledTypePack`, `TypeNode::TypeMemberProjection`);
- Source-target identity and alias convergence;
- Incremental cold/incremental final-state parity for structural type edits;
- Zero-product static distinctions (`Tuple([], []) ≡ Unit`, `Record({})` distinct static Record);
- Zero VM type-member bytecode / solver execution / runtime object model.

### Remaining capability gaps for future checkpoints

- **Variadic generic syntax & expansion:** Pack expansion syntax (e.g. `...P`), variadic type parameters on functions/types;
- **Variadic inference:** Inferring packs from call arguments, pack splitting, prefix/suffix decomposition;
- **Variadic callable lowering & ABI:** Bytecode generation and calling convention for variadic parameter packs;
- **Broader const generics:** User-declared const parameters beyond intrinsic member arguments;
- **Additional structural operations:** Additional intrinsic type-level operators if accepted (e.g. pack concatenation, slicing);
- **Tooling & LSP integration:** Editor hover, inlay hints, and navigation for symbolic structural projections;
- **Release certification:** Program-wide release gates and performance benchmarking.

## Future checkpoint allocation

Future LANG007 work will extend the canonical C1 substrate above. A candidate follow-on checkpoint `LANG007.C2` would focus on variadic generic parameters and pack expansion syntax once its bounded acceptance objective is formally specified and planned.

## Program dependency model

```text
accepted language/specification decisions
        ↓
LANG007.C1
    canonical domains
    type-member identity
    intrinsic structural normalization
    inference/persistence integration
    source/runtime boundary
        ↓
future LANG007 checkpoints
    only after C1 establishes the reusable substrate
```

Later checkpoints must reuse C1 semantic identities and canonicalization rules rather than introducing parallel pack, projection, or intrinsic-operation systems.

## Durable program invariants

These are architectural constraints inherited by LANG007 work unless a later accepted specification or checkpoint amendment explicitly changes them.

1. Type-level structure and runtime values are distinct semantic categories even where Phalcom intentionally unifies type-level and value-level language concepts.
2. `TypePack`, `LabeledTypePack`, and `RecordRow` must not be collapsed merely because all can contain type information.
3. Ordered Tuple labeled structure is not a `RecordRow`; Tuple label order participates in structural identity while Record key order does not.
4. Compile-time value arguments such as an integer index are typed compile-time terms, not kinds and not fake proper types.
5. Compiler-native type operations are semantic/type-normalization operations, not runtime method sends.
6. Canonical semantic identity must survive inference, metadata export, and incremental analysis without serializing query-local IDs.
7. Existing trait associated-type semantics remain a distinct authority and must not be weakened to make intrinsic type members easier to implement.
8. Future variadic/pack work should extend the C1 canonical domains rather than replace them.

## Current lifecycle truth

```yaml
program: LANG007
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active checkpoint: none
active_plan: none
next checkpoint: none accepted
next task: Await the next scoped LANG007 objective.
```

`LANG007.C1`, `C2`, and `C3` are complete at focused-tested level. P3 fixed the
generic associated static-domain payload signature defect; P4 corrected and
focused-tested the declaration projection contract. The checkpoint records
inherited AST, impl-query, and runtime baselines, formatter drift, and the
incomplete full semantic test-binary run. LANG007 remains in progress for
future scoped work; this does not claim release certification.

## Canonical records

Expected repository layout:

```text
docs/implementation/LANG007/
    PROGRAM.md
    STATUS.md
    LANG007.C1/
        LANG007.C1-CHECKPOINT.md
        LANG007.C1-GUIDANCE.md
        LANG007.C1.P1-patch-grade-implementation-plan.md
        LANG007.C1.P1-walkthrough.md
        LANG007.C1.P1-handoff.md
        LANG007.C1.P2-authoritative-specification-and-intrinsic-keyword-convergence.md
        LANG007.C1.P2-walkthrough.md
        LANG007.C1.P2-handoff.md
        LANG007.C1.P3-architecture-baseline-cross-layer-certification-and-roadmap-rebaseline.md
        LANG007.C1.P3-walkthrough.md
        LANG007.C1.P3-handoff.md
```

`PROGRAM.md` owns the long-lived program purpose and checkpoint map.

`STATUS.md` is a concise program-level status view.

`LANG007.C1-CHECKPOINT.md` is the single durable source of truth for C1 implementation state.

`LANG007.C1-GUIDANCE.md` owns persistent C1 execution and architecture guidance that should survive individual plan turnover.

Numbered plans describe intended work; they do not replace checkpoint state.

## Next action

No successor LANG007 checkpoint or plan is currently accepted. Await the next
scoped objective; do not infer C4/P4 from C3 closure.
