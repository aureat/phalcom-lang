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

**State:** `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`

P1 is complete and focused-tested. P2 is the active specification/source
convergence plan; C1 is not yet release-complete.

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

C1 is not release certification for all future type-level programming.

## Future checkpoint allocation

No later LANG007 checkpoint identifier is assigned by this record.

Expected follow-on concerns may include:

- variadic pack expansion syntax and structural formation;
- bounded sequence/pack inference;
- broader compile-time generic argument support;
- additional closed intrinsic structural operations;
- richer reflection/tooling for type-level members and domains;
- broader checkpoint/release certification.

These are **program roadmap topics, not accepted checkpoint scope**. Create a new checkpoint only when its bounded acceptance objective is explicitly defined and differs from C1.

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
active checkpoint: LANG007.C1
active plan: LANG007.C1.P2
next checkpoint: LANG007.C1
next plan: LANG007.C1.P2
```

The program is `IN_PROGRESS` because P1 implementation and focused test
results are recorded while C1's P2 specification/source convergence remains
active. `FOCUSED_TESTED` does not claim workspace-wide or release certification.

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
```

`PROGRAM.md` owns the long-lived program purpose and checkpoint map.

`STATUS.md` is a concise program-level status view.

`LANG007.C1-CHECKPOINT.md` is the single durable source of truth for C1 implementation state.

`LANG007.C1-GUIDANCE.md` owns persistent C1 execution and architecture guidance that should survive individual plan turnover.

Numbered plans describe intended work; they do not replace checkpoint state.

## Next action

Execute `LANG007.C1.P2` from T0. P1 lifecycle closure is recorded in the
checkpoint, walkthrough, and handoff; the P2 start lock is satisfied.

At P2 start:

1. record the actual local repository revision and working-tree state;
2. consume the P1 walkthrough and handoff rather than reconstructing P1;
3. verify the live decision/specification authority and run the named
   `@intrinsic`/`intrinsic` census;
4. proceed through P2's ordered tasks and gates;
5. update the checkpoint only at durable architecture/verification events;
6. create the P2 walkthrough and handoff at P2 completion.
