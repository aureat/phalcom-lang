# Language surface

> Sources: `docs/spec/current/`, LANG001–LANG003
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

The language domain defines Phalcom source notation, lexical boundaries, expression grouping, control flow, blocks, strings, patterns, and documentation-facing syntax. It neighbors [type-system](../type-system/overview.md) for type notation, [semantic](../semantic/overview.md) for meaning after parsing, and [tooling](../tooling/overview.md) for interactive/documentation consumers.

Normative behavior comes from `docs/spec/`; implementation programs annotate whether a rule is implemented, partial, proposed, or unverified. A parser acceptance fact is not by itself a semantic or runtime guarantee.

## Reading order

- [Lexical structure](lexical-structure.md) — token classes, newlines, literals, and brace disambiguation
- [Expressions and control flow](expressions-and-control-flow.md) — precedence, sends, conditionals, loops, and flow
- [Blocks and closures](blocks-and-closures.md) — the shared closure representation and non-local return
- [Strings and interpolation](strings-and-interpolation.md) — UTF-8, escapes, interpolation, and source ranges
- [Patterns and matching](patterns-and-matching.md) — destructuring, constructor patterns, and exhaustiveness boundaries
- [Annotations and Phaldoc](annotations-and-phaldoc.md) — documentation syntax and its tooling boundary

## Status boundary

LANG001 is in progress and partial; LANG003 is proposed and not started. Those labels describe implementation programs, not changes to the normative grammar.
