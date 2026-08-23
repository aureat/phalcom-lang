# Control-header braces no longer parse as trailing closures

- Date: 2026-08-23
- Scope: `phalcom-ast` parser control headers and focused parser regressions
- Trigger: `cargo run -- check examples/sheetcalc/` reported `Expected "{"` at
  `examples/sheetcalc/src/support/num.ph`, after an unparenthesized member getter:
  `if nums.isEmpty { ... }`

## 1. Cause

Phalcom permits bare braced trailing closures after member sends. The parser therefore
could consume the brace after `nums.isEmpty` as an argument to the `isEmpty` getter.
When `parse_if` then looked for the control body, it had already consumed that brace and
reported a misleading missing-`{` error near the closing brace.

## 2. Change

Added `parse_expr_without_trailing_closures` in
[`phalcom-ast/src/parser.rs`](../../../phalcom-ast/src/parser.rs), which temporarily
disables trailing-closure recognition while parsing expressions owned by `if`, `while`,
`if let`, `while let`, and `for` headers. The previous parser state is restored before
returning the expression result, including error paths.

This keeps the intended surface syntax unchanged:

```phalcom
if nums.isEmpty {
  return nil
}

for n in nums {
  process(n)
}
```

Parenthesized conditions remain valid. The following brace is now reserved for the
control construct's body.

## 3. Verification

- `cargo test -p phalcom-ast --lib parser::tests::` — 41 passed.
- `cargo test -p phalcom-ast --test integration parser` — 85 passed.
- `git diff --check -- phalcom-ast/src/parser.rs` — passed.
- `graphify update .` — passed; HTML visualization skipped because graph has more than
  5,000 nodes.

Full `cargo run -- check examples/sheetcalc/` remains unverified in this dirty checkout;
existing `phalcom-semantic` compilation errors stop the command before source checking.
Unrelated working-tree changes were preserved.
