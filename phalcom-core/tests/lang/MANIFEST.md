# Phalcom Language Corpus Manifest

Acceptance corpus for end-to-end CLI validation, organized by label directory.

## Summary

- Labels: arithmetic, bindings, blocks, booleans, classes, lexical, messages, syntax-errors.
- Status counts: PASS 15, PENDING 6, NEGATIVE 2.
- Baseline recorded on 2026-07-11 against `./target/debug/phalcom`.

## Test Matrix

| File | Label | Spec anchor | Status | Current behavior |
|---|---|---|---|---|
| arithmetic/arithmetic_add.ph | arithmetic | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `3`; exit 0 |
| arithmetic/arithmetic_precedence.ph | arithmetic | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `7`; exit 0 |
| arithmetic/arithmetic_unary_minus.ph | arithmetic | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `-1`; exit 0 |
| arithmetic/arithmetic_nested.ph | arithmetic | values-and-absence.md; messages-and-selectors.md; control-flow.md | PASS | stdout `9`; exit 0 |
| arithmetic/arithmetic_div_zero.ph | arithmetic | object-model.md; values-and-absence.md | PASS | stdout `inf`; exit 0 |
| arithmetic/arithmetic_equality.ph | arithmetic | messages-and-selectors.md; object-model.md | PASS | stdout `true`; exit 0 |
| lexical/literals_zero.ph | lexical | lexical-structure.md; object-model.md | PASS | stdout `0`; exit 0 |
| lexical/literals_empty_string.ph | lexical | lexical-structure.md; object-model.md | PASS | stdout blank line; exit 0 |
| lexical/literals_string.ph | lexical | lexical-structure.md; object-model.md | PASS | stdout `hello, world`; exit 0 |
| lexical/literals_true.ph | lexical | values-and-absence.md; object-model.md | PASS | stdout `true`; exit 0 |
| lexical/literals_false.ph | lexical | values-and-absence.md; object-model.md | PASS | stdout `false`; exit 0 |
| lexical/comments_inline.ph | lexical | lexical-structure.md | PASS | stdout `1`; exit 0 |
| bindings/binding_let.ph | bindings | values-and-absence.md; open-questions.md; ADR-0014 | PASS | stdout `1`; exit 0 |
| messages/send_system_new.ph | messages | messages-and-selectors.md; object-model.md | PASS | stdout `<class System>`; exit 0 |
| classes/class_static_pi.ph | classes | classes.md; object-model.md | PASS | stdout `3.1415`; exit 0 |
| lexical/pending/literals_escape.ph | lexical | lexical-structure.md; object-model.md | PENDING | current stdout is literal `a\n b`; exit 0 |
| bindings/pending/binding_var_uninitialized.ph | bindings | values-and-absence.md; open-questions.md; ADR-0014 | PENDING | current parser rejects `var`; exit 1 with syntax error |
| booleans/pending/bool_short_circuit_and.ph | booleans | control-flow.md | PENDING | current stdout is `Binary operation '&&' not supported for false and inf`; exit 0 |
| booleans/pending/bool_short_circuit_or.ph | booleans | control-flow.md | PENDING | current stdout is `Binary operation '||' not supported for true and inf`; exit 0 |
| classes/pending/class_construct_name.ph | classes | classes.md; object-model.md | PENDING | current parser errors at `construct`; exit 1 |
| blocks/pending/blocks_literal_call.ph | blocks | blocks.md; functions.md | PENDING | current parser errors at `=>`; exit 1 |
| syntax-errors/syntax_unclosed_string.ph | syntax-errors | lexical-structure.md; implementation-status.md | NEGATIVE | clean diagnostic `Unterminated string`; exit 1 |
| syntax-errors/syntax_missing_paren.ph | syntax-errors | lexical-structure.md; implementation-status.md | NEGATIVE | clean diagnostic `Expected ")"`; exit 1 |

## Notes

- The corpus intentionally mixes already-working regression guards with spec-target cases that are still pending.
- Pending tests are valid spec targets even when the current tree rejects them.
- Negative tests are malformed inputs that should always fail cleanly, not panic.