# PDR-0029 — Complete string interpolation; defer multiline literals

- Status: Accepted
- Date: 2026-07-22
- Amends: ADR-0022's completion details while preserving its `\(expr)` sigil
- Related: [string interpolation specification](../spec/current/string-interpolation.md), [interpolation completion contract](../work/pending/string-interpolation-completion.md), [multiline-literal deferral](../work/deferred/multiline-string-literals.md)

## Context

ADR-0022 chose the `\(expr)` interpolation sigil and later amended its
stringification target to the `toString` getter. It did not settle the scanner
modes, full escape set, diagnostic-code surface, test placement, or synthetic
AST ranges. The implementation contract collected those completion choices,
but left them as an open ledger while the canonical specification was still a
normative draft.

The lexer already treats a raw `NEWLINE` as outside `string_char` in the
consolidated grammar. Admitting physical newlines now would change a source
boundary rule and expand the first completion unit beyond interpolation.

## Decision

This record makes `docs/spec/current/string-interpolation.md` the sole
accepted normative specification for string interpolation. Do not create a
second canonical copy. Its implementation contract is
`docs/work/pending/string-interpolation-completion.md`.

1. Double-quoted strings accept `\"`, `\\`, `\n`, `\t`, and `\r`; `\(` opens
   interpolation; every other escape is invalid.
2. A physical LF or CRLF remains illegal inside a double-quoted string. Use
   `\n` or `\r\n` for embedded line breaks. Dedicated multiline string literal
   syntax is deferred in the linked work item.
3. Interpolation bodies use the ordinary string and comment scanners. Parenthesis
   depth changes only in expression-code mode; nested strings and comments do
   not affect the outer interpolation boundary.
4. An interpolation body contains exactly one expression after surrounding
   trivia. Empty bodies, extra expressions, and statement separators fail.
5. The stable diagnostic codes are `string.invalid_escape`,
   `string.interpolation.unterminated`, and `string.interpolation.empty`.
6. Tests extend the established lexer/parser modules and runtime fixture
   harness. Use property tests only where existing workspace convention and
   dependencies support them; otherwise use the deterministic UTF-8 adversarial
   corpus in the implementation contract.
7. Parser lowering remains ordinary AST lowering: each expression segment gets
   exactly one `GetProperty("toString")`, expression-leading interpolation starts
   with an empty String accumulator, and concatenation is left-associative.
   `String#+` supplies the ordinary String-required failure when `toString`
   yields a non-String; interpolation adds no special coercion or validation
   operation.
8. Synthetic `Add` and `GetProperty("toString")` nodes use the full outer-string
   range. Inner-expression ranges remain their own source ranges.
9. Update active specifications and relevant front-end comments; preserve ADR
   history. Add a clarification only where an unqualified historical statement
   would otherwise mislead.

## Consequences

- The implementation has one normative source, stable external diagnostic
  names, and test locations that retain existing discovery behavior.
- Interpolation stays parser sugar over the existing AST and ordinary message
  semantics; it does not create an interpolation-specific runtime path.
- Raw newlines remain statement boundaries rather than string content. The cost,
  named plainly: source with visually multiline text must use escapes until a
  separate multiline-literal design decides delimiters, indentation, newline
  normalization, interpolation, and diagnostics.
- Full outer-string ranges make synthetic-node snapshots uniform; range-sensitive
  tools cannot infer a narrower literal-segment origin from those nodes.
- This record does not change ADR-0022's sigil choice, block-comment
  non-nesting, or selector identity for the `toString` getter.

## Alternatives rejected

- **Second canonical specification:** rejected because duplicated normative text
  drifts and leaves implementers without an authority.
- **Physical newlines in ordinary double-quoted strings now:** deferred because
  it expands this corrective interpolation unit into a separate literal-design
  axis.
- **Interpolation-specific string validation or repeated coercion:** rejected;
  it would make interpolation disagree with ordinary `String#+` semantics.
- **Rust error variants without stable codes:** rejected; callers need a durable
  diagnostic surface.
- **New test infrastructure or dependency solely for this feature:** rejected;
  coverage must fit the established harness unless it already supports property
  tests.
- **Narrow synthetic-node ranges:** rejected; they blur synthetic lowering and
  user-written inner-expression ranges.
