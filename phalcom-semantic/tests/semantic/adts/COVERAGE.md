# ADT / GADT / match test coverage ledger

This ledger gives every scenario in comprehensive catalog and Part 05.1
amendment executable owner. `READY` means test code exists and owning target
compiles; it does not claim every runtime assertion passes. `RED` preserves
expected implementation failure. `GATED` marks ignored scenario whose
required language or product surface is not available.

## Semantic products

| Scenario IDs | Owner | Status |
| --- | --- | --- |
| ADT-DECL-01, ADT-DECL-02, ADT-DECL-03, ADT-DECL-04, ADT-DECL-05, ADT-DECL-06, ADT-DECL-07, ADT-DECL-08 | `declarations.rs`, `variants.rs` | READY |
| ADT-VARIANT-01, ADT-VARIANT-02, ADT-VARIANT-03, ADT-VARIANT-04, ADT-VARIANT-05, ADT-VARIANT-06, ADT-VARIANT-07 | `variants.rs` | READY |
| ADT-VARIANT-08, ADT-VARIANT-09, ADT-VARIANT-10 | `variants.rs` | GATED |
| ADT-CONSTR-01, ADT-CONSTR-02, ADT-CONSTR-03, ADT-CONSTR-04, ADT-CONSTR-05, ADT-CONSTR-06 | `constructors.rs` | READY |
| ADT-EXACT-01, ADT-EXACT-02, ADT-EXACT-03, ADT-EXACT-04, ADT-EXACT-05 | `exact_cases.rs`, `declarations.rs` | READY |
| ADT-EXACT-06 | `exact_cases.rs` | GATED |
| ADT-GEN-01, ADT-GEN-02, ADT-GEN-03, ADT-GADT-01, ADT-GADT-02 | `generics.rs`, `declarations.rs` | READY |
| ADT-GADT-03 | `generics.rs` | RED |
| ADT-BEH-01, ADT-BEH-02, ADT-BEH-03 | `behavior.rs`, `phalcom-core/tests/core/language/algebraic_data/behavior.rs` | RED |
| ADT-REQ-01, ADT-REQ-02, ADT-REQ-03, ADT-REQ-04, ADT-REQ-05 | `declarations.rs`, `requirements.rs` | READY |
| ADT-ASSOC-01, ADT-ASSOC-02, ADT-ASSOC-03, ADT-ASSOC-04, ADT-ASSOC-10, ADT-ASSOC-12, ADT-ASSOC-13 | `associated/scenarios.rs`, `associated/lookup.rs` | READY |
| ADT-ASSOC-05, ADT-ASSOC-06, ADT-ASSOC-07, ADT-ASSOC-08, ADT-ASSOC-09 | `associated/scenarios.rs` | RED |
| ADT-ASSOC-11, ADT-ASSOC-14, ADT-ASSOC-15, ADT-ASSOC-16 | `associated/scenarios.rs` | GATED |

## Match semantics

| Scenario IDs | Owner | Status |
| --- | --- | --- |
| MATCH-RES-01, MATCH-RES-02, MATCH-RES-03, MATCH-RES-04, MATCH-RES-05, MATCH-RES-06, MATCH-RES-07, MATCH-RES-09, MATCH-RES-14, MATCH-RES-15 | `matching/resolution.rs` | READY |
| MATCH-RES-08 | `matching/resolution.rs` | GATED |
| MATCH-RES-10, MATCH-RES-11, MATCH-RES-12, MATCH-RES-13 | `matching/resolution.rs` | RED |
| MATCH-PAT-01, MATCH-PAT-02, MATCH-PAT-03, MATCH-PAT-04, MATCH-PAT-05, MATCH-PAT-07 | `matching/patterns.rs` | READY |
| MATCH-PAT-06 | `matching/patterns.rs` | RED |
| MATCH-BIND-01, MATCH-BIND-02, MATCH-BIND-03, MATCH-BIND-04, MATCH-BIND-05, MATCH-BIND-06, MATCH-BIND-07, MATCH-BIND-08 | `matching/bindings.rs` | READY |
| MATCH-SPACE-01, MATCH-SPACE-02, MATCH-SPACE-03, MATCH-SPACE-04, MATCH-SPACE-05, MATCH-SPACE-06, MATCH-SPACE-07, MATCH-SPACE-08, MATCH-SPACE-09, MATCH-SPACE-10, MATCH-SPACE-11, MATCH-SPACE-12, MATCH-SPACE-13, MATCH-SPACE-14, MATCH-SPACE-15, MATCH-SPACE-16, MATCH-SPACE-17, MATCH-SPACE-18 | `matching/pattern_space.rs` | READY |
| MATCH-EXH-01, MATCH-EXH-02, MATCH-EXH-03, MATCH-EXH-04, MATCH-EXH-05, MATCH-EXH-07 | `matching/exhaustiveness.rs` | READY |
| MATCH-EXH-06, MATCH-EXH-11, MATCH-EXH-12 | `matching/exhaustiveness.rs` | RED |
| MATCH-EXH-08, MATCH-EXH-09, MATCH-EXH-10, MATCH-EXH-13, MATCH-EXH-14, MATCH-EXH-15 | `matching/exhaustiveness.rs` | GATED |
| MATCH-USE-01, MATCH-USE-02 | `matching/exhaustiveness.rs` | READY |
| MATCH-USE-03, MATCH-USE-04, MATCH-USE-05 | `matching/exhaustiveness.rs` | RED |
| MATCH-IMP-01 | `matching/exhaustiveness.rs` | READY |
| MATCH-IMP-02 | `matching/exhaustiveness.rs` | RED |
| MATCH-GADT-01, MATCH-GADT-02, MATCH-GADT-03, MATCH-GADT-09 | `matching/gadt_refinement.rs` | READY |
| MATCH-GADT-04, MATCH-GADT-08 | `matching/gadt_refinement.rs` | RED |
| MATCH-GADT-05, MATCH-GADT-06, MATCH-GADT-07, MATCH-GADT-10, MATCH-GADT-11 | `matching/gadt_refinement.rs` | GATED |
| MATCH-FLOW-01, MATCH-FLOW-02, MATCH-FLOW-07, MATCH-FLOW-08, MATCH-FLOW-09, MATCH-FLOW-11 | `matching/flow.rs` | READY |
| MATCH-FLOW-03, MATCH-FLOW-04, MATCH-FLOW-05, MATCH-FLOW-06, MATCH-FLOW-10, MATCH-FLOW-12 | `matching/flow.rs` | GATED |
| MATCH-DIAG-01, MATCH-DIAG-06, MATCH-DIAG-07, MATCH-DIAG-08, MATCH-DIAG-09, MATCH-DIAG-10, MATCH-DIAG-13, MATCH-DIAG-14 | `matching/diagnostics.rs` | READY |
| MATCH-DIAG-02, MATCH-DIAG-03, MATCH-DIAG-04, MATCH-DIAG-05, MATCH-DIAG-11, MATCH-DIAG-15 | `matching/diagnostics.rs` | GATED |
| MATCH-DIAG-12 | `matching/diagnostics.rs` | READY |

## Incremental and executable boundaries

| Scenario IDs | Owner | Status |
| --- | --- | --- |
| ADT-INCR-01, ADT-INCR-02, ADT-INCR-03, ADT-INCR-05, ADT-INCR-06, ADT-INCR-10, ADT-INCR-11, ADT-INCR-12 | `../../incremental/adts.rs` | READY |
| ADT-INCR-04, ADT-INCR-07, ADT-INCR-08, ADT-INCR-09 | `../../incremental/adts.rs`, `../../incremental/match_analysis.rs` | GATED |
| ADT-LOWER-01, ADT-LOWER-02, ADT-LOWER-03, ADT-LOWER-04, ADT-LOWER-06, ADT-LOWER-09, ADT-LOWER-10 | `phalcom-core/tests/core/language/compiler/lowering_scenarios.rs` | READY |
| ADT-LOWER-05, ADT-LOWER-07, ADT-LOWER-12 | `phalcom-core/tests/core/language/compiler/lowering_scenarios.rs` | RED |
| ADT-LOWER-08, ADT-LOWER-11 | `phalcom-core/tests/core/language/compiler/lowering_scenarios.rs` | GATED |
| ADT-RUN-01, ADT-RUN-02, ADT-RUN-03, ADT-RUN-04, ADT-RUN-05, ADT-RUN-06, ADT-RUN-07, ADT-RUN-08, ADT-RUN-09, ADT-RUN-11, ADT-RUN-17 | `phalcom-core/tests/core/language/algebraic_data/scenarios.rs` | READY |
| ADT-RUN-10, ADT-RUN-13, ADT-RUN-14, ADT-RUN-15, ADT-RUN-16 | `phalcom-core/tests/core/language/algebraic_data/scenarios.rs` | GATED |
| PAT-CTX-01, PAT-CTX-02, PAT-CTX-03, PAT-CTX-04, PAT-CTX-05, PAT-CTX-06, PAT-CTX-07, PAT-CTX-08 | `phalcom-core/tests/core/language/algebraic_data/pattern_context.rs` | GATED |
| ADT-GC-01, ADT-GC-02, ADT-GC-03, ADT-GC-04, ADT-GC-05 | `phalcom-core/tests/core/language/algebraic_data/gc_scenarios.rs` | READY |
| ADT-GC-06, ADT-GC-07 | `phalcom-core/tests/core/language/algebraic_data/gc_scenarios.rs` | GATED |
| ADT-VERT-01, ADT-VERT-02 | `phalcom-core/tests/core/language/algebraic_data/scenarios.rs` | READY |
| ADT-VERT-03, ADT-VERT-04 | `phalcom-core/tests/core/language/algebraic_data/scenarios.rs` | RED |
| ADT-VERT-05, ADT-VERT-06 | `phalcom-core/tests/core/language/algebraic_data/scenarios.rs` | GATED |

## Part 05.1 review amendment

| Scenario IDs | Owner | Status |
| --- | --- | --- |
| REVIEW-C1-01, REVIEW-C1-02 | `matching/bindings.rs` | READY |
| REVIEW-C1-03, REVIEW-C1-04, REVIEW-C1-05, REVIEW-C1-06 | `matching/bindings.rs` | GATED |
| REVIEW-C2-01, REVIEW-C2-02, REVIEW-C2-03, REVIEW-C2-04, REVIEW-C2-05, REVIEW-C2-06 | `matching/pattern_space.rs` | READY |
| REVIEW-C3-01, REVIEW-C3-02, REVIEW-C3-03, REVIEW-C3-04, REVIEW-C3-07, REVIEW-C3-08 | `matching/gadt_refinement.rs` | RED |
| REVIEW-C3-05 | `matching/gadt_refinement.rs` | GATED |
| REVIEW-C3-06 | `matching/gadt_refinement.rs` | READY |
| REVIEW-C4-01, REVIEW-C4-02, REVIEW-C4-03, REVIEW-C4-04, REVIEW-C4-05 | `matching/patterns.rs` | GATED |
| REVIEW-M1-01, REVIEW-M1-02, REVIEW-M1-03, REVIEW-M1-04, REVIEW-M1-05 | `phalcom-lsp/src/inlay_hints.rs` | RED |
| REVIEW-M2-01, REVIEW-M2-02 | `matching/pattern_space.rs` | READY |
| REVIEW-M2-03, REVIEW-M2-04, REVIEW-M2-05 | `matching/pattern_space.rs` | GATED |
| REVIEW-M3-01, REVIEW-M3-02, REVIEW-M3-03 | `phalcom-ast/tests/match_patterns.rs` | READY |
| REVIEW-M3-04 | `phalcom-ast/tests/match_patterns.rs` | GATED |
| REVIEW-M4-01, REVIEW-M4-02, REVIEW-M4-05 | `matching/diagnostics.rs` | READY |
| REVIEW-M4-03 | `matching/diagnostics.rs` | GATED |
| REVIEW-M4-04 | `matching/diagnostics.rs` | RED |
| REVIEW-M5-01, REVIEW-M5-05, REVIEW-M5-06 | `matching/exhaustiveness.rs` | READY |
| REVIEW-M5-02, REVIEW-M5-03 | `matching/exhaustiveness.rs` | RED |
| REVIEW-M5-04 | `matching/exhaustiveness.rs` | GATED |
| REVIEW-M6-01, REVIEW-M6-05, REVIEW-M6-06 | `matching/bindings.rs` | READY |
| REVIEW-M6-02, REVIEW-M6-03, REVIEW-M6-04, REVIEW-M6-07 | `matching/bindings.rs` | GATED |
| REVIEW-X-01, REVIEW-X-03, REVIEW-X-04 | `matching/conformance.rs` | READY |
| REVIEW-X-02 | `matching/conformance.rs` | GATED |
| REVIEW-AST-01 | `phalcom-ast/tests/match_syntax.rs` | READY |
| REVIEW-AST-02 | `phalcom-ast/tests/match_patterns.rs` | READY |
| REVIEW-AST-03 | `phalcom-ast/tests/match_syntax.rs` | READY |

Separate `phalcom-ast` and `phalcom-lsp` targets own AST and editor-safety
products. Core tests own bytecode/runtime consequences only; they do not
recreate compiler semantic authority.
