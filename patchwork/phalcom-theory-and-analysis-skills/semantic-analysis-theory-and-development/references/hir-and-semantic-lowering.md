# HIR and Semantic Lowering

## HIR purpose

A High-level Intermediate Representation removes syntactic accidents while preserving source-level semantic distinctions useful to analysis.

Possible normalized nodes:

```text
ResolvedName(BindingId)
ResolvedClass(ClassId)
Send { receiver, selector, args, dispatch_mode }
SuperSend { receiver=self, lookup_owner, selector, args }
FieldRead(FieldId)
FieldWrite(FieldId, value)
Block { params, captures, home_callable, body }
Return { target, value }
Throw
```

This is conceptual; design only what consumers need.

## Preserve source

Every HIR node should map to source range and ideally source AST/node ID for diagnostics/refactorings.

Do not discard distinctions needed for formatting/source edits.

## Desugaring policy

Lower only when semantic equivalence is defined. Examples:

- operators to sends;
- shorthand fields to field ops;
- implicit self to explicit receiver metadata;
- pattern declarations to binding operations;
- loop sugar to control form.

Some sugar should remain high-level if diagnostics/prover benefit from original construct.

## Declaration surfaces versus bodies

Build shallow declaration surfaces first so recursive references can resolve without lowering every body. Then lower bodies using resolved surfaces.

## Type annotation lowering

Preserve source type expression AST for reflection/diagnostics while resolve/lower to canonical semantic `TypeId` separately.

## Attribute/decorator expansion

Attributes can change declaration product/behavior. Lowering pipeline needs explicit phases:

```text
parse attributes
classify structural/declaration-product effects
build semantic declaration
apply/retain metadata
```

Do not let LSP and compiler expand attributes differently.

## Stability

HIR IDs can be per-body revision. Incremental cache keys should include owner/body revision, not assume numeric ID remains stable after edits.
