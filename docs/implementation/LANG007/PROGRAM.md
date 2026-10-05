---
id: LANG007
category: LANG
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
prepared: 2026-09-21
last_updated: 2026-10-05
---

# LANG007 — Type-Level Structural Domains and Intrinsic Type Operations

LANG007 owns the implementation of Telos compiler-assisted structural and static
computation: the canonical type/static semantic substrate established by C1–C3 and the
C4 structural/static upgrade, first-class Pack transport, Product/Sum reflection,
generalized `FieldKey`, static evaluation, and binding-phase semantics.

The program exists to make structural computation a coherent part of Telos's type/value
model without turning the VM into a type solver or compile-time evaluator, and without
conflating ordered Pack lanes with unordered row structure. Under `INC-P3-2`, C3 kinded
domains (`TypePack`, `LabeledTypePack`, `CompleteTypePack`) remain canonical compiler
domains, while `Pack` provides first-class proper structural transport.

## Program ownership

LANG007 owns implementation work for:

- type-level structural domains such as `TypePack`, `LabeledTypePack`, and `CompleteTypePack`;
- compiler-native type members and their canonical semantic identity;
- typed compile-time arguments to type members where required by the accepted language design;
- Tuple structural projection and indexing;
- structural `Map<F>` over Tuple/Record-related domains;
- stable symbolic projections when a type-level computation is well formed but not yet reducible;
- substitution, inference, metadata, source indexing, and Universe integration needed to preserve those semantics;
- C4 structural/static evaluation, first-class Pack transport, Product/Sum reflection, generalized unordered `FieldKey`, and compile-time `const` semantics.

LANG007 does not own ordinary runtime collection iteration, general VM dispatch, arbitrary compile-time evaluation, unrelated trait/conformance architecture, or general-purpose metaprogramming unless a later checkpoint explicitly expands the program's bounded objective.

## Checkpoint ledger

### LANG007.C1 — Semantic domains and compiler-intrinsic structural type operations

**State:** `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`

C1 established the canonical domains, type-member identity/projection machinery,
structural operations, persistence/inference integration, and runtime boundary across
plans P1, P2, and P3.

- `LANG007.C1.P1` — semantic domains and compiler-intrinsic structural type operations — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C1.P2` — authoritative specification and `intrinsic` keyword convergence — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C1.P3` — architecture baseline, cross-layer certification, roadmap re-baselining, and C1 closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

### LANG007.C2 — Variadic Generic Pack Binding and Type-Level Pack Expansion

**State:** `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`

C2 established variadic generic binding (`*`, `**`, `***`), flat canonical pack
compositions (`TypePackConcat`, `LabeledTypePackConcat`, `CompleteTypePack`),
Tuple/callable structural expansions, whole-pack inference, durable scoped metadata
persistence under schema v4, and incremental parity across plans P1, P2, and P3.

- `LANG007.C2.P1` — canonical pack composition and variadic binder semantics — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C2.P2` — source variadic generic binding and structural/callable type expansion — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C2.P3` — whole-pack inference, persistence, incremental certification, and checkpoint closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

### LANG007.C3 — Type Data and Unified Algebraic Static Domains

**State:** `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`

C3 established unified algebraic declarations (`data` for runtime products and sums,
`type data` for type-level static domains), nominal static-domain identity, canonical
static terms, heterogeneous static applications, intrinsic structural-domain exposure,
dynamic generic slots, witness specialization, and full iteration integration.
Closed and verified under adopted P5 rulings.

- `LANG007.C3.P1` — unified algebraic data declarations and type-data foundation — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P2` — general static domains, heterogeneous application/inference, associated type data — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P3` — intrinsic structural surface, Universe/corpus migration, certification and closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P3.R1` — intrinsic semantic application and runtime/native owner projections — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P4` — intrinsic structural data semantic completion — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3.P5` — full implementation audit, dynamic slots, witness specialization, and closure — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

### LANG007.C4 — Structural Static Upgrade

**State:** `IN_PROGRESS / PARTIAL / UNVERIFIED`

Authoritative decision: [TDR-0090](../../decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md).
Owning checkpoint record: [`LANG007.C4-CHECKPOINT.md`](LANG007.C4/LANG007.C4-CHECKPOINT.md).

C4 upgrades Telos into a coherent structural and compile-time semantic system:
- runtime immutable `let`, mutable `let mut`, and compile-time `const`;
- explicit immutable-by-default fields and definite initialization lifecycle;
- first-class `Pack` structural transport, O(1) rest slicing, and call/rest convergence;
- generalized `FieldKey` (`Symbol | String | Int | Bool`) for unordered records;
- compiler-derived sealed `Product`, `OrderedProduct`, `UnorderedProduct`, and `Sum` capabilities;
- bounded deterministic semantic static evaluation without a compile-time VM;
- normalized semantic residual IR;
- single pattern/GADT authority for static and runtime matching.

Current plans:
- `LANG007.C4.P1` — Binding, mutability, field lifecycle, corpus cutover — `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`.
- `LANG007.C4.P2` — Static value model, static definitions, evaluator foundation, closed `const` — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C4.P3` — First-class Pack and call/rest convergence — `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C4.P4` — Generalized `FieldKey` and unordered-product calculus — `IN_PROGRESS / PARTIAL / UNVERIFIED`.

## Current program lifecycle truth

```yaml
program: LANG007
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_checkpoint: LANG007.C4
active_plan: LANG007.C4.P4
latest_completed_plan: LANG007.C4.P3
next_action: >
  Continue LANG007.C4.P4 execution from its active checkpoint state.
```

Detailed P4 execution progress is maintained in `LANG007.C4-CHECKPOINT.md`.

## Durable program invariants

1. Type-level structure and runtime values are distinct semantic categories even where Telos intentionally unifies type-level and value-level language concepts.
2. `TypePack`, `LabeledTypePack`, and `RecordRow` must not be collapsed merely because all can contain type information.
3. Ordered Tuple labeled structure is not a `RecordRow`; Tuple label order participates in structural identity while Record key order does not.
4. Pack and Tuple labels remain `Symbol`-only; `FieldKey` belongs exclusively to unordered Record/product structure.
5. Compile-time value arguments are typed compile-time terms, not kinds and not fake proper types.
6. Compiler-native type operations are semantic/type-normalization operations, not runtime method sends.
7. Canonical semantic identity must survive inference, metadata export, and incremental analysis without serializing query-local IDs.
8. Existing trait associated-type semantics remain a distinct authority and must not be weakened to make intrinsic type members easier to implement.
9. First-class `Pack` transport preserves representation freedom (flat, slice, virtual) while adhering to exact structural semantics.

## Canonical records layout

```text
docs/implementation/LANG007/
    PROGRAM.md
    STATUS.md
    LANG007.C1/
        LANG007.C1-CHECKPOINT.md
        ...
    LANG007.C2/
        LANG007.C2-CHECKPOINT.md
        ...
    LANG007.C3/
        LANG007.C3-CHECKPOINT.md
        ...
    LANG007.C4/
        LANG007.C4-CHECKPOINT.md
        LANG007.C4.P1/
        LANG007.C4.P2/
        LANG007.C4.P3/
        LANG007.C4.P4/
```
