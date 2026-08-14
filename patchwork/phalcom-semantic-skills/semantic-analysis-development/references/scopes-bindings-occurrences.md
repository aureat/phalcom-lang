# Scopes, Bindings, Name Resolution, and Semantic Occurrences

## Scope construction workflow

For every new syntax construct that can introduce names:

1. identify whether it creates a new lexical scope;
2. identify parent scope;
3. define exact source range of the scope;
4. visit initializer/default expressions in the correct *pre-declaration* environment;
5. declare bindings at the language-defined point;
6. assign binding kind/mutability;
7. visit nested body;
8. add semantic occurrences for declaration and uses.

## Source order

Do not assume "binding exists in scope map" means it is visible before its declaration.
Current resolution uses declaration range compared with query offset.

This matters for:

```phalcom
use(x)
let x = 1
```

and for shadowing during initializers.

## Initializer ordering

A binding typically is not visible in its own initializer unless Phalcom explicitly permits
recursive definitions for that construct.

Correct pattern:

```text
visit initializer using existing scope
then declare new binding
```

For recursive declarations, use an explicit declaration shell/indexing phase rather than
accidentally exposing every local early.

## Shadowing

Resolution walks nearest lexical scope outward and stops at first visible declaration.

Tests should cover:

- parameter shadows outer local;
- local shadows top-level;
- closure binding shadows method parameter;
- for binding shadows outer;
- destructuring names;
- import alias shadowing/being shadowed according to spec.

## Binding categories

Keep binding kind because consumers care:

- completion detail;
- semantic token category;
- mutability/assignment diagnostics;
- refactoring rules;
- future type inference/generalization policy.

When adding a new category, ask whether it truly has distinct semantics or only display
wording.

## Mutability

Mutability is semantic. Future flow refinements/smart casts may depend on stability:

- immutable binding can often retain a refinement safely;
- mutable binding may invalidate refinement on assignment;
- captured mutable binding may be invalidated by invoked closures;
- aliasing/field mutation complicates refinement further.

Do not add flow typing without consulting mutability/capture facts.

## Patterns

A pattern can introduce multiple bindings. For each binding record:

- individual `BindingId`;
- exact name range;
- shared pattern/initializer provenance if needed;
- projection path if future precise destructuring inference requires it.

Example projection metadata (future):

```text
TupleIndex(0)
TupleIndex(1)
ListRest(after=2)
RecordField("name")
VariantPayload(...)
```

Avoid re-parsing the pattern at every query to determine projection.

## Imports

An import may create:

```text
local BindingId -> import declaration -> resolved ModuleId/declaration target
```

Keep unresolved imports in occurrence/module data so editor diagnostics/navigation can recover
when the target later appears.

## Classes and globals

Lexical binding resolution and class/global lookup are distinct. Do not force classes into the
same local-binding namespace unless the language spec says the namespace is unified.

The current scope model can return `Binding`, `Class`, `Module`, `ImplicitSelf`, `Global`,
`Unresolved`. Preserve that semantic distinction as the language grows.

## Implicit self

A bare name that is not a lexical binding may become an implicit-self send only where syntax
and current class context permit.

Resolution should not prematurely turn every unknown identifier into a member. Suggested
staging:

```text
lexical resolution
  -> class/module/global rules
  -> implicit-self dispatch candidate where allowed
  -> unresolved/dynamic
```

## Occurrence roles

Useful occurrence roles include:

- declaration;
- read/reference;
- write/assignment target;
- import alias/path;
- class/member declaration;
- selector send;
- field access;
- type annotation reference (future);
- protocol/type parameter reference (future).

Role matters for rename and diagnostics.

## Targeted source behavior

Every occurrence should use the narrowest meaningful range:

- identifier token, not whole method body;
- selector part, not whole send expression;
- parameter name, not whole declaration;
- field name, not whole assignment.

This prevents "hover highlights entire method" failures and makes edits precise.

## Rename algorithm

Semantic rename should:

1. resolve occurrence to target identity;
2. collect occurrences by identity;
3. simulate new spelling in affected scopes/selectors;
4. detect collisions/capture;
5. preserve selector label/arity/kind semantics;
6. reject or warn for reflection/dynamic references that cannot be proven;
7. emit edits.

Never use workspace text search as the authoritative reference set.

## Tests

Minimum for a new binding form:

```text
single declaration/use
use before declaration
nested shadow
outer use after inner shadow ends
assignment/read roles
closure capture
malformed/incomplete declaration
rename collision
edit that inserts/removes declaration
```
