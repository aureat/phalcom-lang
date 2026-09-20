# AUD-RUNTIME-010 - String raw byte accessor contract mismatch

## Classification

- Severity: Medium
- Category: Native surface / library declaration / result representation
- Confidence: Confirmed source-contract mismatch and repaired with focused
  publication/runtime evidence.

## Observed

`String.rawByteAt(_)` is declared as `(Int) -> Option<Int>` in
`phalcom-core/core/universe/src/scalar/string.ph`, while
`phalcom-core/src/primitive/string.rs::string_raw_byte_at` returns a bare
`Value::int` for an in-range byte and kernel `None` only for an out-of-range
index. Existing library callers compare the result with `None` and then use it
as a numeric value, reinforcing the bare-value convention.

## Resolution

The effective contract is bare `Int` on success and `None` out of range. The
`.ph` declaration and Rust native metadata now publish `(Int) -> Int`, matching
the unchanged native body and existing callers. Generated native surface data
was refreshed.

Focused publication and raw-byte regressions pass, including direct runtime
result shape and native-source census checks.
