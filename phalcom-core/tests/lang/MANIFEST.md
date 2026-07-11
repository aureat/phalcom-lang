# Phalcom Language Corpus Manifest

Preliminary acceptance corpus for end-to-end CLI validation.

## Summary

- Areas: lexical/literals, arithmetic/operators, bindings, messages/selectors, control flow, classes, blocks, errors.
- Status counts: PASS 15, PENDING 7, NEGATIVE 2.
- Baseline recorded on 2026-07-11 against `./target/debug/phalcom`.

## Test Matrix

| File | Area | Spec anchor | Status | Current behavior |
|---|---|---|---|---|
| arithmetic_add.ph | arithmetic/operators | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `3`; exit 0 |
| arithmetic_precedence.ph | arithmetic/operators | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `7`; exit 0 |
| arithmetic_unary_minus.ph | arithmetic/operators | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `-1`; exit 0 |
| arithmetic_nested.ph | arithmetic/operators | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `9`; exit 0 |
| arithmetic_div_zero.ph | arithmetic/operators | object-model.md; values-and-absence.md | PASS | stdout `inf`; exit 0 |
| arithmetic_equality.ph | arithmetic/operators | messages-and-selectors.md; object-model.md | PASS | stdout `true`; exit 0 |
| literals_zero.ph | lexical/literals | lexical-structure.md; object-model.md | PASS | stdout `0`; exit 0 |
| literals_empty_string.ph | lexical/literals | lexical-structure.md; object-model.md | PASS | stdout blank line; exit 0 |
| literals_string.ph | lexical/literals | lexical-structure.md; object-model.md | PASS | stdout `hello, world`; exit 0 |
| literals_true.ph | lexical/literals | values-and-absence.md; object-model.md | PASS | stdout `true`; exit 0 |
| literals_false.ph | lexical/literals | values-and-absence.md; object-model.md | PASS | stdout `false`; exit 0 |
| comments_inline.ph | lexical/comments | lexical-structure.md | PASS | stdout `1`; exit 0 |
| binding_let.ph | bindings | values-and-absence.md; open-questions.md; ADR-0014 | PASS | stdout `1`; exit 0 |
| send_system_new.ph | messages/selectors | messages-and-selectors.md; object-model.md | PASS | stdout `<class System>`; exit 0 |
| class_static_pi.ph | classes | classes.md; object-model.md | PASS | stdout `3.1415`; exit 0 |
| literals_escape.ph | lexical/literals | lexical-structure.md; object-model.md | PENDING | current stdout is literal `a\n b`; exit 0 |
| binding_var_uninitialized.ph | bindings | values-and-absence.md; open-questions.md; ADR-0014 | PENDING | current parser rejects `var`; exit 1 with syntax error |
| bool_short_circuit_and.ph | control flow | control-flow.md | PENDING | current stdout is `Binary operation '&&' not supported for false and inf`; exit 0 |
| bool_short_circuit_or.ph | control flow | control-flow.md | PENDING | current stdout is `Binary operation '||' not supported for true and inf`; exit 0 |
| class_construct_name.ph | classes | classes.md; object-model.md | PENDING | current parser errors at `construct`; exit 1 |
| blocks_literal_call.ph | blocks | blocks.md; functions.md | PENDING | current parser errors at `=>`; exit 1 |
| syntax_unclosed_string.ph | errors | lexical-structure.md; implementation-status.md | NEGATIVE | clean diagnostic `Unterminated string`; exit 1 |
| syntax_missing_paren.ph | errors | lexical-structure.md; implementation-status.md | NEGATIVE | clean diagnostic `Expected ")"`; exit 1 |

## Notes

- The corpus intentionally mixes already-working regression guards with spec-target cases that are still pending.
- Pending tests are valid spec targets even when the current tree rejects them.
- Negative tests are malformed inputs that should always fail cleanly, not panic.