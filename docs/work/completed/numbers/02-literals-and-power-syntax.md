# U-NUMBERS-02 — numeric literals and `**`

## Outcome

Implement PDR-0026 typed numeric literals and PDR-0027 power syntax without weakening lexical
diagnostics or source spans.

## Write set

- `phalcom-ast/src/{token.rs,lexer.rs,ast.rs,parser.rs}`: typed literal tokens, `**`, AST binary op.
- `phalcom-core/src/compiler/lib/{expr.rs,patterns.rs}` plus span-bearing bytecode emission.
- lexer/parser/compiler tests and negative fixtures.

## Steps

1. Preserve literal kind through token, AST, and compiler: decimal/radix Int payloads must never
   pass through f64; decimal point/exponent selects Float. Oversized Int token payload is digits +
   radix, parsed to BigInt only in the compiler.
2. Enforce PDR-0026 underscore, radix, and malformed-token rules atomically. One malformed numeric
   literal emits one `numeric.literal` diagnostic with a primary span over the complete lexeme.
3. Lex `**` by longest match before `*`. Add it to operator declarations, selector-symbol parsing,
   AST, and compiler send emission. Do not create `**=`.
4. Use grammar `power := postfix [ "**" unary ]`, with unary's base `power`. Preserve the power
   token span in the binary AST/bytecode so later runtime errors can underline it.

## Required fixtures

- literals: `0b1010`, `0o755`, `0xFF`, `1_000`, `.5`, `6e2`, huge decimal/radix Int;
  invalid separators, invalid digit, trailing dot, and malformed exponent.
- parse tree/behavior: `2 ** 3 ** 2 == 512`, `-2 ** 2 == -4`, `2 ** -2 == 0.25`,
  `(-2) ** 2 == 4`.
- source-span regression: a failing `0 ** -1` later reports the `**` span, not either operand or
  an enclosing statement.
