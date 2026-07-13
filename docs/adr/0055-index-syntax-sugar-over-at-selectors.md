# ADR-0055: Subscript Indexing Syntax Sugar over `at` Selectors

## Status

Accepted

## Context

Phalcom's benchmark suite (specifically `benchmarks/math/vectors.ph` and `stats.ph`) and Wren-compatibility porting notes require a postfix subscript syntax `xs[i]` and `xs[i] = v` for readable list and map indexing. However, we do not want to introduce new opcodes, new VM types, or separate sacred inline cached selectors for this syntax in the initial implementation.

## Decision

We implement `expr[idx]` and `expr[idx] = value` as pure parser-level and compiler-level syntactic sugar over the existing `at(_)` and `at(_,put:)` message pairs.

1. **AST Representation**:
   - `Expr::Index(Box<IndexExpr>)` for subscript reads.
   - `Expr::SetIndex(Box<SetIndexExpr>)` for subscript writes.
   - Distinct AST nodes are preserved (rather than immediate method call desugaring in the parser) so `parse_assignment` can distinguish left-hand side targets for setter assignment.

2. **Parser Desugaring**:
   - `parse_call` recognizes postfix `[` as a postfix operator (same newline-termination rules as `.`, `(`, etc., avoiding ASI hazard). It produces `Expr::Index`.
   - `parse_assignment` matches `Expr::Index` on the left-hand side of `=` and transforms it into `Expr::SetIndex`.

3. **Compiler Lowering**:
   - `Expr::Index(ix)` compiles to an ordinary method send `Invoke(1, at_idx)` to the selector `at(_)`.
   - `Expr::SetIndex(six)` compiles to an ordinary method send `Invoke(2, at_put_idx)` to the selector `at(_,put:)` with index as a positional argument and value under the `put:` keyword label.

4. **DEC-INDEX-B (Compound Assignment)**:
   - Compound assignment (`xs[i] += 1`) is descoped from this design block. We verified that `obj.prop += 1` also does not work on HEAD today, so compound assignments will be addressed in a generic follow-up for both property and index targets.

## Consequences

- Postfix `[]` and `[]=` syntax carries zero new VM floor changes (no new opcodes, no new primitives).
- Out-of-bounds index reads/writes inherit the existing collection protocols: reads return `None` (total operation), and writes raise an out-of-range catchable `RuntimeError`.
- Once inline caches (U-IC) land, indexing benefits from method-dispatch cache speedups automatically, as it desugars to standard `Invoke` calls.
