---
id: COLL006.C1.P1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: handoff
title: "Contextual Sequence Literal and Inference Foundation — Handoff to P2 / P3"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P1 — Handoff to COLL006.C1.P2 and P3

## 1. Repository anchor

- **Worktree Branch:** `feat/telos-collections-upgrade`
- **Starting Base Revision:** `61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1`
- **Completed Plan:** `COLL006.C1.P1`
- **Next Plans:** `COLL006.C1.P2` (`Array<T>`) and `COLL006.C1.P3` (`Vector<T, N>`)

## 2. Completed prerequisites

- Syntax neutralization is complete: `Expr::BracketLiteral` replaces `Expr::ListLiteral`.
- Explicit heads `<T>[...]` and `<T, N>[...]` are parsed as `BracketLiteralHeadSyntax`.
- Inference hole `GenericArgumentSyntax::InferenceHole` is parsed and supported in generic argument position.
- Annotation elaboration `AnnotationElaboration` exists in `phalcom_semantic::checker::annotation_elaboration`.
- Partial binding annotation inference is wired in `Statement::Let` and `static_constant_demand`.
- `CheckingContext.phase` explicitly tracks `EvaluationPhase::{Runtime, Static}`.
- Semantic sequence literal resolution `SequenceLiteralResolution` is published in `CallableAnalysis`, `CheckedStaticExpression`, and `CheckedStaticBody`.
- Compiler lowering consumes `ModuleLoweringSemantics.sequence_literals` and fails closed on missing resolution.

## 3. Stable interface / takeover map

| Concern | Established Interface |
|---|---|
| AST bracket literal | `phalcom_ast::ast::Expr::BracketLiteral(Box<BracketLiteralExpr>)` |
| AST literal head | `phalcom_ast::ast::BracketLiteralHeadSyntax { arguments, range }` |
| AST inference hole | `phalcom_ast::ast::GenericArgumentSyntax::InferenceHole { range }` |
| Annotation elaboration | `phalcom_semantic::checker::elaborate_annotation` / `AnnotationElaboration` |
| Evaluation phase | `phalcom_semantic::checker::EvaluationPhase::{Runtime, Static}` on `CheckingContext.phase` |
| Sequence resolution product | `phalcom_semantic::checker::SequenceLiteralResolution` / `SequenceLiteralResolutionIndex` |
| Lowering specification | `phalcom_core::modules::semantic_lowering::SequenceLiteralLoweringSpec` |
| Compiler error for missing resolution | `phalcom_core::compiler::CompilerError::MissingSequenceLiteralLowering(SourceRange)` |

## 4. Invariants that must not be redesigned

1. **Parser neutrality:** The parser records the head syntax and element syntax; it does not choose `Array`, `List`, or `Vector`. Do not add family decisions to the parser.
2. **Inference variables are query-local:** `_` is a solver variable and must never be interned into canonical `TypeStore` or published as `Dynamic`.
3. **Semantic checker owns family authority:** The compiler must only execute `spec.family`; do not re-implement family inference in the compiler.
4. **P1 bare default remains `List` until P4:** Do not flip bare bracket defaults in P2 or P3. P4 owns the atomic cutover once both `Array` and `Vector` runtime/static models are complete.

## 5. Must-read files for P2 (`Array<T>`)

1. `docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`
2. `phalcom-semantic/src/checker/expression.rs` (`synthesize_bracket_literal`)
3. `phalcom-core/src/compiler/lib/expr.rs` (`Expr::BracketLiteral` match)
4. `phalcom-core/src/modules/semantic_lowering.rs` (`SequenceLiteralLoweringSpec`)

## 6. Recommended next actions

- For P2 implementer:
  - Add canonical `Array` class identity in Universe bootstrap.
  - Implement runtime `ArrayObject` with fixed contiguous slot storage and frozen state.
  - In `synthesize_bracket_literal`, recognize explicit `<T>[...]` or expected `Array` as selecting `SequenceLiteralFamily::Array`.
  - In `compiler/lib/expr.rs`, implement `SequenceLiteralFamily::Array` code generation (`BuildArray` or target builder).
