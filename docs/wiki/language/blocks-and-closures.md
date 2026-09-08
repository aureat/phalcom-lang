# Blocks and closures

> Sources: `docs/spec/current/blocks.md`
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

Blocks, lambdas, method bodies, and getter bodies share one closure representation, but a method remains distinct because it carries selector, holder, and receiver identity. `Block` and `Method` are siblings under the callable root.

## Forms and arity

An unbraced arrow has one parameter and an expression body. Braced blocks may have zero or more parameters and statement bodies. The comma is therefore unambiguous inside braces while `n, x => ...` is invalid. Trailing-block syntax is argument sugar and preserves the underlying selector.

## Returns

The final expression is the block value. `return` exits the enclosing method frame, not merely the block. An escaping block carries a frame token; a stale generation must become a `DeadFrameError` rather than an unsafe unwind. There is no `break` or `continue` inside a block.

## Object and error boundary

Blocks are objects and can receive `call`, `arity`, and protected-execution messages such as `on`, `ensure`, and `attempt`. The language contract is specified here; [runtime lifecycle](../runtime/lifecycle-and-memory.md) owns frame/heap implementation and cleanup.
