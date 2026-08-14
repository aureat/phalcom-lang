# Semantic Identities and Resolution

## Identity principle

Names are not entities. Ranges are not entities. AST pointers are not necessarily
stable entities. Give semantically distinct things distinct identities.

## Module identity

Current LSP uses canonical URI-backed `ModuleId`. This is appropriate for today's
"file as module" world, but future package/project work may require separating:

- physical source file;
- logical module name;
- package identity/version;
- module instance at runtime;
- generated/virtual modules;
- core/native pseudo-modules.

Do not bake file-system coincidence into every downstream identity. Prefer a module
identity abstraction whose construction policy can evolve.

Questions for future modules:

- Can multiple files contribute to one logical module?
- Can one file contain nested modules?
- Are aliases identity-preserving?
- Are imports namespace edges, initialization edges, compilation edges, or several?
- Can the same package version be instantiated twice under different dependency roots?
- How are symlinks/canonical paths handled?
- What is the identity of embedded/core source?

## Class identity

A class must be module-qualified. Bare class names are presentation text.

Current conceptual form:

```text
ClassId = (ModuleId, class name)
```

Future declaration IDs may become preferable if declaration products can be generated,
aliased, or rebound independently of name. Keep APIs sufficiently abstract to migrate.

Class identity is not necessarily identical to a future language `TypeId`:

- a class object is a runtime value;
- instance type may be parameterized/applied;
- protocols and unions are types but not classes;
- `Dynamic`, `Any`, `Nothing`, `Self` are not ordinary classes.

## Callable identity

Ordinary method identity follows Phalcom selector semantics and dispatch side.

Conceptually:

```text
CallableId = (owner class identity, canonical selector, dispatch side)
```

Do not replace canonical selector with base name + guessed arity. Labels and member
kind semantics matter. Do not add parameter types to selector identity merely because
the checker sees annotations.

If multiple-dispatch/typecase is added explicitly in the future, its identity and
selection semantics must be specified separately.

## Field identity

Use class-qualified + side-qualified identity. Instance field `x` and class-side field
`x` are not interchangeable facts.

Field analysis must also distinguish evidence kind:

- declaration initializer;
- constructor initialization;
- general write;
- future native/trusted metadata.

This supports definite initialization and better diagnostics later.

## Scope identity

A scope is a lexical region with a parent and direct declarations. Scope containment
is source-sensitive. The current file-snapshot-local `ScopeId` is a good compact index.

Do not assume it remains stable after a reparse. Stable cross-edit references should
resolve again from source/semantic targets or use a future stable declaration identity.

## Binding identity

A `BindingId` identifies one declaration, not a spelling.

This is what makes these distinct:

```phalcom
let x = 1
|| {
  let x = "inner"
  x
}
x
```

A name-resolution query should first identify the scope at the occurrence and then walk
lexical parents, honoring declaration-order rules.

## Pattern bindings

Destructuring patterns may introduce several bindings from one source declaration.
Each name needs its own `BindingId`, while provenance can point back to the shared pattern
and initializer.

Future tuple/record/list pattern refinement should attach projected facts to those
binding IDs rather than to source spellings.

## Imports

An import binding has at least two identities:

- local lexical binding/alias;
- resolved target module/declaration.

Keep both. Rename and navigation may care about the local alias; type/dispatch/module
graph analysis cares about the target identity.

Unresolved imports should remain representable rather than disappearing from the graph.
They are dependencies whose target is currently unknown.

## Occurrences and source targets

An occurrence should carry:

- exact source range;
- semantic target when known;
- occurrence role (declaration, read, write, selector send, etc.);
- source kind if useful to consumers.

Do not infer references by global text search once occurrences can be indexed by target.

## Declaration identity versus runtime object identity

Phalcom has first-class reflective objects. Be precise:

- source declaration identity describes *which declaration*;
- runtime object identity describes a particular object produced during execution;
- reflection metadata may connect them but they are not automatically identical.

For example, re-evaluating a declaration in a REPL may produce a fresh runtime descriptor
even when source spelling is unchanged. Tooling should not assume source and runtime
identity have the same lifetime.

## Equality rules

Every ID type should define what equality means, and that equality should match the
semantic question.

Avoid:

- comparing display strings;
- comparing source ranges for declaration identity;
- interning unrelated categories into one untyped integer namespace;
- relying on pointer equality of cloned snapshots.

Prefer newtypes with typed keys.

## Stable versus ephemeral IDs

Classify IDs explicitly:

| Lifetime | Example | Safe uses |
|---|---|---|
| file-snapshot local | current `BindingId`, `ScopeId` | flow facts/query within snapshot |
| semantic-generation stable | occurrence/internal analysis IDs | one published generation |
| module/project stable | `ModuleId`, class/callable identities where declaration unchanged | dependency maps, summaries |
| runtime stable | object identity/handles | VM/reflection only |

Do not persist an ephemeral ID into a cache whose lifetime exceeds its contract.
