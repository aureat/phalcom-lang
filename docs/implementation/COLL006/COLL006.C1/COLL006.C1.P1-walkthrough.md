---
id: COLL006.C1.P1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
title: "Contextual Sequence Literal and Inference Foundation — Walkthrough"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P1 — Contextual Sequence Literal and Inference Foundation — Walkthrough

## 1. Plan identity and final state

- **Plan ID:** `COLL006.C1.P1`
- **Owning Checkpoint:** `COLL006.C1`
- **Status:** `COMPLETE`
- **Completion:** `IMPLEMENTED`
- **Verification:** `FOCUSED_TESTED`

## 2. Objective and outcome

P1 established the semantic foundation for the Telos collection upgrade without prematurely activating runtime `Array<T>` or `Vector<T, N>` families.

Outcomes achieved:
1. Neutralized bracket literal syntax from `Expr::ListLiteral` to `Expr::BracketLiteral`, preserving backwards-compatible bare literal behavior as `List<T>`.
2. Added first-class generic/static argument inference hole syntax `GenericArgumentSyntax::InferenceHole { range }`.
3. Added parser support for explicit generic/static argument heads `<T>[...]` and `<T,N>[...]` on bracket literals without hardcoding family selection in the parser.
4. Added kind-aware annotation elaboration (`AnnotationElaboration`) in the semantic checker, allocating solver-local inference variables with target parameter kinds for holes.
5. Connected partial annotation inference to `Statement::Let` and `static_constant_demand`, deriving `ExpectedType::Inference` and solving holes from initializers before publishing canonical `BindingContract`s.
6. Added explicit `EvaluationPhase` (`Runtime | Static`) to `CheckingContext`, propagating across derived contexts and set to `Static` in static checking entry points.
7. Generalized literal synthesis to publish `SequenceLiteralResolution` products on `CheckingContext`, `CallableAnalysis`, `CheckedStaticExpression`, and `CheckedStaticBody`.
8. Extended `ModuleLoweringSemantics` with `SequenceLiteralLoweringSpec` and updated compiler lowering to consume checked semantic sequence specs, failing closed when missing.

## 3. Architecture implemented

- **Neutral Syntax (`phalcom-ast`):**
  - `Expr::BracketLiteral(Box<BracketLiteralExpr>)`
  - `BracketLiteralExpr` carries `head: Option<BracketLiteralHeadSyntax>`, `elements: Vec<BracketLiteralElement>`, and `range: SourceRange`.
  - `GenericArgumentSyntax::InferenceHole { range }`.
- **Annotation Elaboration (`phalcom-semantic`):**
  - `AnnotationElaboration::Canonical(TypeId)` vs `AnnotationElaboration::Inference { context, frame, term, holes }`.
  - Destination binder's `KindId` directly governs the fresh inference variable's kind.
  - Solving via `solve_inference_in_frame` and materialization via `session.materialize(&term, store)`.
- **Evaluation Phase (`phalcom-semantic`):**
  - `EvaluationPhase::{Runtime, Static}` stored on `CheckingContext`.
  - Static entry points establish `EvaluationPhase::Static`.
- **Semantic Product (`phalcom-semantic`):**
  - `SequenceLiteralResolution` carries `family`, `result_type`, `element_type`, `extent`, and `construction`.
  - Recorded in `CheckingContext.sequence_literals` and published in `CallableAnalysis`, `CheckedStaticExpression`, and `CheckedStaticBody`.
  - Included in incremental product fingerprints in `db/fingerprint.rs`.
- **Compiler Boundary (`phalcom-core`):**
  - `ModuleLoweringSemantics.sequence_literals` maps `SourceRange -> SequenceLiteralLoweringSpec`.
  - Compiler matches `spec.family`, emitting `BuildList` / `BeginListLiteral` for `List`, and rejecting missing lowering with `CompilerError::MissingSequenceLiteralLowering`.

## 4. Important files changed

- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/src/bin/c4_migrate.rs`
- `phalcom-semantic/src/checker/annotation_elaboration.rs` (new module)
- `phalcom-semantic/src/checker/mod.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/analysis.rs`
- `phalcom-semantic/src/checker/expected.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/statement.rs`
- `phalcom-semantic/src/checker/static_body.rs`
- `phalcom-semantic/src/static_constant.rs`
- `phalcom-semantic/src/static_constant_demand.rs`
- `phalcom-semantic/src/static_definition.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- `phalcom-semantic/src/advisory/analyzer.rs`
- `phalcom-semantic/src/static_eval/machine.rs`
- `phalcom-semantic/src/static_key.rs`
- `phalcom-semantic/src/impls.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-core/src/compiler/lib/expr.rs`
- `phalcom-core/src/compiler/lib/error.rs`
- `phalcom-core/src/compiler/attributes.rs`
- `phalcom-core/src/compiler/boundedness.rs`
- `phalcom-core/src/compiler/lib/product_opt.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-lsp/src/semantic_tokens.rs`

## 5. Durable interfaces and invariants established

- `P1-INV-SYN-01` to `P1-INV-SYN-05`: Bracket literals are neutral AST nodes; explicit heads parse and retain arguments; `_` is a distinct generic argument node.
- `P1-INV-INF-01` to `P1-INV-INF-07`: Holes use canonical binder kinds; inference variables remain query-local; no `InferVarId` escapes into canonical types.
- `P1-INV-PHASE-01` to `P1-INV-PHASE-04`: Scoped `EvaluationPhase` is explicit in checking context; bare bracket default remains `List` during P1.
- `P1-INV-SEQ-01` to `P1-INV-SEQ-05`: Semantic checker publishes canonical `SequenceLiteralResolution`.
- `P1-INV-LOWER-01` to `P1-INV-LOWER-04`: Compiler lowering consumes checked sequence specs and fails closed if missing.
- `P1-INV-INCR-01` to `P1-INV-INCR-03`: Sequence literal resolutions participate in semantic body fingerprints.

## 6. Deliberate deviations from plan

None. All architectural requirements, invariants, and scope limits were observed without deviation.

## 7. Consultations and adopted decisions

No escalation was triggered. All implementation work fell within the planned architectural boundaries.

## 8. Tests added

- `phalcom-ast/src/parser.rs`: `bracket_literal_parses_explicit_heads_and_inference_holes`
- `phalcom-semantic/tests/semantic/foundations/coll006_c1_p1_contextual_literals.rs`:
  - `test_list_hole_inference_solves_element_type`
  - `test_list_hole_empty_is_underconstrained`
  - `test_strict_type_formation_rejects_holes_in_declarations`
  - `test_spread_list_literal_publishes_expanded_construction`
  - `test_static_constant_demand_with_hole`

## 9. Verification classification

`FOCUSED_TESTED`.
The entire workspace and test suites compile cleanly without warnings (`cargo check --workspace --all-targets`).
Baseline tests passed during initial setup. Per instructions, full workspace test suites were deferred to the checkpoint release gate.
