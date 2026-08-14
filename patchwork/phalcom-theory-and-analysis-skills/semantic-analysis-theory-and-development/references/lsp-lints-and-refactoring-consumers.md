# LSP, Lints, and Refactoring Consumers

## LSP as adapter

Semantic layer returns:

```text
SemanticTarget
Definition/Binding/Member info
Value/Type facts
Completion candidate surface
Signature/call contract
References
Diagnostics facts
```

LSP converts to protocol ranges/items/markup.

## Hover

Target exact semantic occurrence at cursor, then render relevant declaration, runtime shape, type, docs, provenance/confidence. Do not infer by walking enclosing method independently.

## Completion

Use receiver/context semantic facts and member-resolution APIs. Rank/filter in LSP layer; member truth belongs in semantics.

## Definition/references/rename

Use semantic identity. Rename must perform capture/conflict analysis and edit exact occurrences; selectors and labels require syntax-aware edits.

## Semantic tokens

Can use resolved occurrence category to distinguish class/local/field/method beyond lexer token kinds.

## Lints

Each lint declares minimum analysis tier:

```text
syntax
binding
CFG/flow
type
effect
proof
whole project
```

Avoid forcing expensive proof for stylistic lint.

## Code actions/refactorings

Transformation preconditions are semantic facts. A refactoring should query whether operation preserves binding/dispatch/type behavior before producing edits.

## Diagnostics ownership

Core semantic/checker layers produce structured diagnostic facts/codes; LSP renders them. Do not make semantic engine depend on `lsp_types`.
