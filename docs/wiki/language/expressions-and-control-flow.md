# Expressions and control flow

> Sources: `docs/spec/current/syntax/expressions.md`, `control-flow.md`
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

Expression syntax groups sends; it does not replace the message model. Binary and unary operators are surface sugar for selectors, while the compiler may later recognize selected sends without changing their semantic contract.

## Precedence

Assignment is right-associative and sits below Option coalescing. The middle tiers are `or`, `and`, equality, comparison/type-test, bitwise operators, additive and multiplicative arithmetic, power, unary operators, then postfix sends/calls/trailing blocks.

`??` is right-associative and short-circuits its right operand. `?.` is a postfix member-access form with the same chaining behavior as a send. Power groups its unary right operand, so `2 ** -2` is `2 ** (-2)`.

## Sends and expansion

Dot notation names a selector; labels become selector slots. A non-dot callee with arguments is call sugar. A spread argument expands a collection into positional slots, and a trailing block supplies the final argument without changing selector identity. The [argument expansion](../collections/argument-expansion.md) article owns the pack semantics.

## Control flow

Conditionals and loops are expression-oriented. `break` and `continue` belong to loop sugar; blocks use [non-local return](blocks-and-closures.md#returns). Type tests and flow predicates produce semantic evidence only after the semantic analyzer reconciles their relation outcome.

## Boundary

Parsing establishes grouping and source ranges. [Semantic capability and flow](../semantic/capability-and-flow.md) owns narrowing and control outcomes; [compiler/VM execution](../runtime/compiler-vm-boundary.md) owns lowering.
