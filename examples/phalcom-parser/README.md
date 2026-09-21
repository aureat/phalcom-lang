# Phalcom self-hosted parser

This package is a design/implementation draft for a parser written in Phalcom
that parses Phalcom source.

The implementation intentionally stress-tests:

- same-selector polymorphism by types and patterns;
- record-shaped enum variants;
- unqualified variant patterns (`Identifier`, `Keyword(Class)`, `Some`, `None`);
- Pratt expression parsing;
- generic/value-aware type syntax;
- pattern syntax;
- controlled refinement types;
- parser recovery and diagnostics;
- arena-based syntax storage.

The refinement grammar is deliberately restricted. It admits literals,
bindings, structural member projection, arithmetic, comparisons, and boolean
logic, but no arbitrary method calls.

The parser uses pattern-polymorphic method families such as:

```phalcom
parsePrefix(_ kind: Integer) -> ExprId { ... }
parsePrefix(_ kind: Keyword(If)) -> ExprId { ... }
parsePrefix(_ kind: TokenKind) -> ExprId { ... }
```

The unique most-specific applicable method is expected to be selected by
Phalcom dispatch.
