---
id: LANG007.C3.P1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
---

# LANG007.C3.P1 — Walkthrough

## 1. Identity and delivery state

- Plan: `LANG007.C3.P1`
- Starting planning revision: `62a6082ab463387c812d558e5561508267ecc63c`
- Live verification revision: `60a27f79d69aa3e79c22e3d19df221466c7db920`
- Delivery: working-tree changes only; no commit or push was requested.
- Final classification: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- Checkpoint classification: `LANG007.C3` remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

## 2. Implemented architecture

### A. Canonical algebraic declaration path

- Runtime `data` products and sums use one canonical algebraic semantic builder.
- Legacy `enum` remains accepted compatibility syntax and converges to the same
  semantic declaration/case identity as equivalent `data` source.
- `DataInfo` and `EnumInfo` remain compatibility projections of the shared
  algebraic product; no second declaration authority was introduced.
- `type data` declarations publish nominal `StaticDomainId` identity with
  canonical static constructors/cases.

### B. Static-term and kind foundation

- `KindData::StaticDomain` distinguishes static domains from `Type`, `RecordRow`,
  and pack domains.
- `TypeData::ConstInt` and `TypeData::StaticData` are canonical store terms with
  checked domain, constructor, arity, and argument-kind formation.
- `ProperTypeId`/proper-type checks reject static terms; substitution, GADT
  environments, rows, and checked instantiation preserve them recursively.
- Runtime lowering keeps TypeOnly declarations out of runtime declaration
  emission, and runtime expression use of TypeOnly associated terms is rejected
  with `TypeOnlyRuntimeValue`.

### C. Stable metadata and fingerprints

- Metadata schema advances monotonically from v4 to v5.
- Static-domain kinds, nominal declaration references, `ConstInt`, and
  `StaticData` terms use structural stable identity rather than local arena IDs
  or rendered source strings.
- Declaration metadata records algebraic phase (`Runtime`/`TypeOnly`) and shape
  (`Product`/`Sum`). Interface fingerprints are sensitive to those facts and to
  static/algebraic structure, while equivalent legacy `enum` and `data` spelling
  remains identity-neutral.
- Schema validation rejects unsupported static features, malformed domain/kind
  combinations, unknown constructor references, and invalid static-data forms.

## 3. Coverage verification

| Coverage | Evidence | Result |
|---|---|---|
| CV-01/CV-02 canonical data syntax and body discrimination | `phalcom-ast` integration `data_syntax` — 25 tests | PASS |
| CV-03 enum/data compatibility identity | semantic `equivalent_legacy_enum_and_data_produce_equal_semantic_facts`; metadata identity-neutral regression | PASS |
| CV-04/CV-05 runtime sum, record payload, exact-case, and GADT preservation | semantic `adts` — 281 passed, 28 gated/ignored | PASS |
| CV-06 one semantic authority | shared builder plus semantic declaration/constructor/variant suites | PASS |
| CV-07 nominal static domain | `foundations::kinds::source_type_data_publishes_a_static_domain_kind` | PASS |
| CV-08 canonical static terms | `foundations::kinds` static-domain/ConstInt/StaticData tests | PASS |
| CV-09 proper-type boundary | `foundations::type_model` — 17 tests and static kind checks | PASS |
| CV-10 static payload validity | `static_data_is_canonical_and_rejects_wrong_domain_or_payload_kind`; schema validation | PASS |
| CV-11 runtime-negative boundary | `type_data_case_cannot_be_used_as_a_runtime_value`; compiler/runtime lowering audit | PASS |
| CV-12 intrinsic orthogonality | AST intrinsic data/type-data syntax tests and phase/provenance preservation | PASS |
| CV-13 metadata stability | semantic `integration::metadata` — 21 tests; `schema_compat` — 9 tests | PASS |
| CV-14 fingerprint sensitivity/neutrality | metadata phase/shape and enum/data identity-neutral regressions; fingerprint suite — 33 tests | PASS |
| CV-15 cold/incremental parity | `incremental::adts` — 13 passed/3 gated; `type_store_revisions` — 20 tests | PASS |
| CV-16 C2 regression | `foundations::type_model` and affected semantic suites | PASS |
| CV-17 runtime representation freedom | core `language::algebraic_data` — 44 passed/19 gated | PASS |
| CV-18 no static VM work | core source/lowering inspection and runtime-negative tests | PASS |

## 4. Gate summary

- G1 parser/AST algebraic surface: PASS — 25 `data_syntax` and 12 `enum_syntax`
  integration tests.
- G2 runtime algebraic semantic preservation: PASS — full `adts` focused suite.
- G3 static-domain/type-data foundation: PASS — kinds, type model, data
  declarations, static formation, and runtime-negative evidence.
- G4 persistence/incremental/runtime representation: PASS — schema compatibility,
  metadata, fingerprints, incremental parity, and core runtime suite.
- G5 final focused stabilization: PASS for the live replacement filters and
  directly affected suites. The plan target `native_adt_runtime` does not exist
  in the live package; its runtime coverage is supplied by `core`'s
  `language::algebraic_data` suite (44 passed, 19 gated).

## 5. Deferred and baseline evidence

- `cargo test -p phalcom-semantic static_data` remains a known baseline compile
  failure from a pre-existing test initializer missing `generic_binder_header`;
  it is outside the focused integration/G5 matrix and was not repaired here.
- Gated/ignored ADT tests remain explicitly gated in the existing suites. They
  are not promoted to P1 evidence.
- Workspace-wide build/test/clippy, full corpus, LSP, and release certification
  remain deferred by the plan.

## 6. P2 boundary

P1 deliberately does not implement heterogeneous `Applied`, domain-directed
static inference, static aliases/result domains, associated `type data` lookup,
or cross-module static application. P2 must consume the canonical static-domain
and static-term products established here without weakening the runtime-negative
boundary or introducing a second identity engine.
