# Binding and Name Resolution

## Identity before value

A name occurrence must first answer "which declaration?" before "what value/type?"

Use semantic identities:

```text
BindingId
ClassId
ModuleId
TypeParamId
```

rather than source spelling.

## Namespace policy

Phalcom has multiple naming mechanisms:

- lexical locals/upvalues;
- parameters;
- module/global bindings;
- class declarations;
- imports/aliases;
- receiver-local fields;
- privileged implementation fields;
- implicit-self sends;
- selectors/symbols.

Do not flatten into one map with ad-hoc precedence.

## Resolution result

A rich resolution enum can encode:

```text
Lexical(BindingId)
ModuleBinding(...)
Class(ClassId)
Module(ModuleId)
Field(FieldId)
ImplicitSelf(selector/name)
Global(...)
Unresolved(reason)
Ambiguous(candidates)
```

The current implementation has a simpler `NameResolution`; extend deliberately.

## Declaration order

Specify forward-reference rules per declaration kind. Locals may be visible only after declaration while class/protocol shells may support recursion.

## Shadowing

Shadowing changes which binding future occurrences resolve to. Existing occurrences keep their semantic target. Rename/refactoring must detect capture/conflicts through scope graph, not textual replacement.

## Imports

An import alias is a local binding to a module/export semantic entity. Avoid copying remote inferred data into local maps; resolve by semantic IDs/snapshot.

## Implicit self

Resolution path should distinguish unresolved lexical/global name from confirmed implicit-self selector possibility. Incomplete editor code may keep an unresolved/implicit-self candidate rather than claiming one exact target.

## Field namespaces

Receiver-local fields are semantic storage, not sends and not lexical variables. Resolve `_field`/implementation forms according to current object/access spec.

## Type namespace

Future type expressions may reuse value/class bindings or have dedicated descriptors. Specify whether type/value namespaces are unified, contextual, or separate before implementing lookup.
