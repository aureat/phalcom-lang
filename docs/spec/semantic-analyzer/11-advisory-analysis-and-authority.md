# Phalcom Semantic Analyzer Specification
## 11 — Advisory Analysis and Authority

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define advisory runtime-shape analysis as a compiler-owned abstract domain, its authority boundary relative to formal semantics, its resolution and fixed-point rules, and its publication contract.

---

## 1. Advisory semantics is a separate domain

Formal type knowledge and advisory runtime-shape knowledge answer different questions.

```text
formal TypeKnowledge
    what proposition is established, assumed, unknown, or dynamic

advisory ValueShape
    bounded prediction of runtime value/member/collection shape for tooling
```

Advisory facts are not language types, proof facts, assignability premises, or optimizer correctness facts.

---

## 2. Advisory product

An advisory fact/product conceptually carries:

```text
AdvisoryFact
├── ValueShape
├── AdvisoryOrigin / provenance
├── confidence or precision class
├── AdvisoryProductStatus
├── canonical source/target attachment
├── input/product fingerprint
└── bounded explanation
```

Shape and status are orthogonal. “No advisory product published” is distinct from “analysis completed and widened to unknown.” Cancelled, budget-exhausted, blocked, and internally failed advisory computations must not masquerade as ordinary unknown shapes.

---

## 3. Authority law

The direction is:

```text
formal products -> optional advisory seed/projection
advisory products -> presentation and non-authoritative tooling
```

The reverse direction is forbidden. Advisory analysis cannot:

- construct `Established` or `Assumed` formal knowledge;
- satisfy or refute a formal relation;
- repair a formal invalid/unknown/dynamic result;
- create hard compiler diagnostics from disagreement;
- authorize proof or unsafe optimization.

---

## 4. Formal/advisory composition matrix

| Formal product | Advisory product | Formal result | Permitted presentation |
|---|---|---|---|
| `Established(T), Ready` | compatible shape | unchanged | normal formal type; optional advisory detail |
| `Established(T), Ready` | incompatible shape | unchanged | formal wins; optional advisory-disagreement explanation |
| `Assumed(T)` | compatible/incompatible shape | unchanged | show formal assumption normally; advisory detail may be labeled |
| `Unknown(R)` | useful shape | remains `Unknown(R)` | advisory-only enrichment may be shown without claiming formal proof |
| `Dynamic(D)` | useful shape | remains dynamic | advisory runtime prediction may be shown as non-authoritative |
| known + `Invalid(C)` | any shape | knowledge/status/cause unchanged | preserve diagnostic and formal fact; advisory cannot repair |
| `Blocked` / `Cancelled` / budget / internal failure | available shape | terminal formal state unchanged | advisory may be omitted or shown with incomplete-formal explanation |

Presentation policy must never blend disagreement into a synthetic union type or confidence ladder between formal and advisory authority.

---

## 5. Formal-to-advisory projection

Projection is explicit and one-way. A known nominal/applied type may seed a broad compatible shape. An assumed formal proposition may guide advisory analysis but cannot create exact advisory observation merely because it is a formal premise.

Unknown or dynamic formal knowledge does not by itself supply a concrete advisory seed. Advisory analysis may still derive a shape from its own legitimate syntax/flow/runtime-shape rules.

Projection uses canonical type/declaration structure, never class-name strings or presentation text.

---

## 6. Canonical resolution

Advisory member and call resolution must use canonical compiler identities, selector rules, visibility, linked exports, and dispatch surfaces.

Required laws:

- unexported declarations do not become visible because a declaration surface exists;
- class-object dispatch uses class-side lookup;
- a class-side miss does not generically retry instance-side dispatch;
- constructor body mapping is explicit and preserves public constructor identity;
- exact formal target identity is consumed directly when available;
- advisory target prediction remains in an advisory attachment and never enters the exact target index.

Advisory code must not maintain a second module, inheritance, dispatch, or semantic dependency authority.

---

## 7. Advisory flow

Advisory transfer must preserve reachability, control-flow joins, closures/blocks, mutations, and dynamic boundaries according to its declared abstract domain.

Constructing a block is not executing it. Advisory analysis must not execute closures merely because syntax is available. Effects and captures are recorded or widened conservatively.

Joins are deterministic, monotone for one fixed input revision, and bounded by explicit widening.

---

## 8. Contributions and interprocedural summaries

Parameter advisory state is the join of all currently live call-site contributions.

```text
contribution identity =
    canonical call site
  + canonical callable
  + canonical parameter slot
```

Editing or deleting a caller removes obsolete contributions before recomputing the join. Contributions must not accumulate monotonically after their source disappears.

Callable summaries declare their dependencies. Recursive regions solve by SCC/fixed point with intentional widening; an arbitrary pass count is not a fixed-point proof.

---

## 9. Terminal advisory states

Cancellation, budget exhaustion, blockage, and internal failure leave formal products valid. The affected advisory product is absent or explicitly non-ready.

No partially mutated advisory map reaches publication. Scratch analysis state may be mutable before publication; the snapshot product is immutable and coherent.

---

## 10. Incrementality

Advisory product identity includes:

- consumed formal/source inputs;
- canonical target/dispatch identity;
- shape and status;
- live contribution set and joined result;
- declared dependencies;
- observable bounded provenance.

Advisory changes do not invalidate formal products. Formal or source changes invalidate advisory dependents only through canonical dependency ownership.

---

## 11. Publication and queries

Formal, source-attachment, and advisory products published in one snapshot must belong to the same semantic revision. Reuse is permitted only after semantic input/product equivalence is proven.

Queries distinguish:

```text
no advisory coverage
advisory unknown
advisory non-ready
advisory fact available
```

LSP and other consumers adapt these queries; they do not run advisory solving synchronously on request paths.

---

## 12. Conformance requirements

Tests must cover:

1. formal `Unknown` plus advisory known shape leaves formal product unchanged;
2. formal/advisory disagreement cannot change diagnostics or formal fingerprint;
3. assumed formal seed does not become advisory exact observation;
4. exports and class/instance dispatch follow canonical compiler rules;
5. constructor mapping preserves class-side identity;
6. caller edit/delete replaces or removes parameter contributions;
7. recursive advisory summaries converge or widen deterministically;
8. cancellation/budget/internal failure publishes no partial advisory mutation;
9. cold and incremental advisory products normalize equivalently;
10. no advisory API can create formal evidence.
