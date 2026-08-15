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

## Formal resolution judgment

Name resolution should be describable independently of the concrete scope-map implementation. A useful judgment is:

```text
Γ, p ⊢ name ⇓ target
```

where:

- `Γ` is the scope/module/class environment;
- `p` is the source/program point, needed for declaration-order visibility;
- `name` is the syntactic reference;
- `target` is a semantic identity or a structured unresolved result.

For lexical scopes, define a visibility predicate:

```text
visible(binding, p) = declaration_rule(binding, p)
```

and nearest-scope lookup:

```text
resolve_lexical(scope, name, p) =
    first visible declaration named `name` in scope
    otherwise resolve_lexical(parent(scope), name, p)
```

The important point is that “present in the scope's declaration table” and “visible at program point `p`” are separate predicates.

## Declaration identity versus source location

A `BindingId` identifies one declaration within the semantic generation/file model. Its source range locates the spelling. Do not make byte offset the sole semantic identity because offsets move after preceding edits, and do not assume a file-local ID survives reparsing unless the architecture explicitly stabilizes it.

Think in layers:

```text
source occurrence identity  = (revision, range, role)
semantic declaration identity = BindingId within snapshot/generation
cross-module identity = ModuleId/ClassId/CallableId/etc.
runtime object identity = unrelated execution-time concept
```

Consumers must use the identity appropriate to their question.

## Resolution namespaces

Do not assume Phalcom has one universal namespace merely because source syntax reuses identifier tokens. Model each lookup category explicitly from the language specification:

```text
lexical variables/parameters
classes/types (if/when type syntax exists)
modules/import aliases
globals
members/selectors
labels
attributes
```

If two categories share a namespace normatively, encode that rule once. If they are distinct, a generic `HashMap<String, Target>` risks false collisions and incorrect rename.

## Declaration groups and recursive visibility

Some language constructs may eventually declare a group whose members are mutually visible. Represent this deliberately:

```text
phase 1: allocate declaration identities/shells
phase 2: resolve bodies/initializers in the specified group environment
```

Do not obtain recursion accidentally by collecting every declaration in a file before resolving all expressions. That changes source-order semantics for non-recursive constructs.

## Capture analysis

A use resolves to a lexical declaration, then capture is derived from scope ownership:

```text
captures(use, binding) iff
    defining_callable(binding) != using_callable(use)
    and binding is lexically reachable
```

Useful capture facts:

```text
read capture
write capture
captured by which closure/callable
escapes through which closure (future)
```

Capture identity should reference `BindingId`, not copied names. Shadowing then behaves correctly by construction.

## Occurrence index as semantic source map

The occurrence index is more than navigation metadata. It is the bridge:

```text
source range -> semantic target/role
semantic target -> source occurrences
```

This supports hover, references, rename, diagnostics, code actions, and eventually type/proof explanation. Preserve narrow token ranges and enough role information that consumers do not need to re-parse the AST.

A malformed reference may still deserve an occurrence with an unresolved/recovery target if doing so is structurally safe. This enables the editor to recover when imports/declarations appear later.

## Ambiguity is not unknown

If two semantically viable declarations exist because source/project state is ambiguous, represent ambiguity explicitly where it affects correctness:

```text
Resolved(target)
Unresolved(reason)
Ambiguous(candidates)
Recovered(candidate?, recovery_reason)
```

Collapsing ambiguity into `Unresolved` loses useful diagnostics; choosing one candidate silently makes rename/navigation dangerous.

## Rename as a semantics-preservation operation

A correct rename should preserve the resolution relation after edits. If declaration `d` is renamed from `x` to `y`, for every renamed occurrence `o`:

```text
resolve_after(o[y]) = d'
```

and for every unaffected occurrence `u`:

```text
resolve_after(u) = resolve_before(u)
```

unless the user explicitly accepts broader semantic change.

Collision analysis therefore must simulate lookup, including declaration order, shadowing, selector identity, imports, and any reflection caveats. Text replacement is only the final edit mechanism.

## Incremental resolution invariants

An edit that inserts a new declaration can affect uses after the declaration point in nested scopes without affecting unrelated modules. The invalidation key should therefore be semantic dependency/scope contribution, not “every identifier with this text in workspace.”

At minimum, after incremental update:

```text
all occurrences refer to targets from the same published generation
removed declarations have no live references
new shadowing is reflected in downstream uses
unchanged module-qualified identities remain stable where architecture promises it
```

## Worked shadowing example

```phalcom
let x = 1
let block = |y| {
    use(x)       // outer x
    let x = y
    use(x)       // inner x
}
use(x)           // outer x
```

Expected semantic identities:

```text
x_outer != x_inner
use#1 -> x_outer
use#2 -> x_inner
use#3 -> x_outer
```

If `let x = y` does not allow self-reference in its initializer, the `y` expression resolves in the pre-declaration environment and any `x` inside that initializer would resolve to `x_outer`.

A scope builder that inserts `x_inner` before visiting its initializer gets this wrong.

## Incomplete-source example

```phalcom
let user = fetchUser()
user.
```

The incomplete member selector should not destroy the exact binding occurrence for `user`. The parser/source model can recover a member-access prefix; semantic analysis can still resolve receiver `user` and publish its advisory fact, while the missing selector remains a source-recovery condition rather than a language-level resolved member.

## Review questions

- What exact judgment is resolution implementing?
- Is source position part of visibility?
- Which namespace is searched at each stage?
- Does the declaration exist before its initializer/body by specification?
- Is ambiguity represented or silently picked?
- What source target does reflection refer to, if any?
- Are captures identified semantically rather than by name?
- Does rename preserve resolution for renamed and unaffected occurrences?
- Can malformed source preserve unaffected binding/use facts?
- After an edit, can a reference point to an identity from an older snapshot?
