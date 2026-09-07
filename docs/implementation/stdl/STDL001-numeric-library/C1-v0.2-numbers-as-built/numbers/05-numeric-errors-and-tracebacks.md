# U-NUMBERS-05 — numeric errors and rich tracebacks

## Outcome

Every ratified numeric failure produces structured `Error.kind`, stable text, and an innermost
traceback caret when current span/source architecture can represent it.

## Write set

- numeric primitive and constructor error sites.
- compiler/bytecode span propagation for numeric sends and constructors.
- `phalcom-core/src/diagnostics/{traceback.rs,caret.rs}` only if an existing hook cannot carry
  labels; do not build another renderer.
- runtime and JSON traceback tests; parse-diagnostic fixtures.

## Steps

1. Create one numeric-error builder mapping every condition in
   [text-and-errors](text-and-errors.md#3-numeric-error-contract)
   to `Error.kind` and its exact message template. Do not introduce new native exception classes.
2. Carry source range with numeric binary operations, unary conversion selectors, constructor
   arguments, shifts, bit-index calls, and allocation-triggering powers. Reuse instruction spans;
   do not derive ranges from rendered text.
3. On runtime raise, populate existing `RuntimeError::Raise` traceback data. Its innermost frame
   must use `caret::LabelKind::Primary` on the operator/call argument. Add a secondary operand
   label only for shift/index cases where it identifies the invalid input.
4. Preserve graceful fallback: generated/native/REPL frames lacking source or span show structured
   error and frames without a fabricated source line, caret, column, or byte range.
5. Keep literal failures in lexer/parser diagnostic machinery, one primary malformed-literal
   span. Do not route syntax failures through runtime traceback.

## Gates

- Human tests assert source line and operator caret for divide/mod/floor-divide by zero,
  nonfinite `~/`, `0 ** -1`, invalid shift/index, malformed numeric text, conversion, abstract
  allocation, invalid hash, and numeric limit.
- JSON trace tests assert `error.kind`, message, frame source location, and primary range fields;
  no byte-for-byte dependence on pretty layout.
- Source-less native and REPL cases assert no bogus caret block.
