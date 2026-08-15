# Semantic Identities and Resolution

Semantic analysis fails early when it cannot answer "which thing is this?" precisely. This reference defines the identity discipline beneath Phalcom binding, navigation, dispatch, incremental analysis, typing and future proving. The central rule is strict: **spelling, location, declaration identity, analysis identity and runtime object identity are different axes.**

## 1. Identity answers a semantic question

An ID is meaningful only relative to the equivalence relation it implements. Before adding an identifier, write the sentence:

> Two values of this ID are equal exactly when ...

```text
BindingId:
  same lexical declaration in this source snapshot

ClassId:
  same module-qualified class declaration under current module identity

CallableId:
  same owner class + canonical selector + dispatch side

Source occurrence:
  same occurrence in one source revision, not merely same target
```

Do not create one universal integer namespace and rely on comments to remember which entity it denotes. Rust newtypes and structured IDs make invalid comparisons harder.

## 2. Five identities commonly confused

| Concept | Example | Typical lifetime |
|---|---|---|
| Source file/revision | `file:///a.ph @ revision 14` | one document revision |
| Source occurrence | identifier token at range `80..81` | one source revision |
| Semantic declaration | module-qualified class/method/field | semantic lifetime |
| Analysis/program point | fact before assignment at node/offset | one semantic generation |
| Runtime object | a particular `Point` instance or reflective method object | execution/runtime lifetime |

Rename wants declaration identity plus occurrences. Flow wants binding identity plus program point. Reflection can connect a runtime descriptor to a declaration without making them the same identity.

## 3. CURRENT Phalcom identity model

**CURRENT:** `phalcom-lsp/src/semantic/ids.rs` defines:

```text
ModuleId     = canonical URI string wrapper
ClassId      = (ModuleId, class name)
CallableId   = (ClassId owner, canonical selector, DispatchSide)
FieldId      = (ClassId owner, field name, DispatchSide)
DispatchSide = Instance | Class
```

`ScopeId` and `BindingId` are owned by `scope.rs` and are compact identities inside the scope graph for a parsed file snapshot.

This is a good current model for a file-as-module repository. It is not automatically the permanent answer for future package identity, generated modules, multiple source files per module, package instances, or REPL/reload runtime identity.

## 4. Module identity: physical, logical and runtime identity may diverge

Today a canonical URI is enough to distinguish source modules. Future module/package work may introduce:

```text
PhysicalSourceId   file/blob/virtual source
PackageInstanceId  resolved package + version/source instance
LogicalModuleId    module namespace inside package instance
RuntimeModuleId    initialized module object/instance
```

Do not prematurely collapse these into a path string. Ask whether multiple files may contribute to one module, generated/native source may share a module, package version participates in identity, two dependency roots can instantiate the same package twice, symlinks are canonicalized, and reload preserves source identity while creating a fresh runtime object.

The semantic model should isolate module-ID construction so future decisions do not require rewriting every consumer.

## 5. Class identity is not type identity

Current class identity is module-qualified:

```text
ClassId(module, "Point")
```

A bare `Point` spelling is display/lookup text, not durable identity. A future language type system needs entities that are not classes:

```text
Point                nominal instance type
Box<String>          applied type
P & Q                intersection, if ratified
T | None             union, if ratified
Self                 context-dependent type
Dynamic / Any-like   special typing states, if ratified
protocol P           protocol/structural type identity
```

Therefore `ClassId != TypeId`. A bridge may map exact runtime-class knowledge to a nominal instance type, but identity categories stay distinct.

## 6. Callable identity and selector identity

Ordinary method identity follows Phalcom dynamic selector semantics:

```text
CallableId = (owner ClassId, canonical selector, dispatch side)
```

The selector is not base name alone, guessed arity, source formatting, parameter type annotations, source range, or declaration index. Typing metadata must not be smuggled into ordinary selector identity. Explicit future type-directed dispatch would require separately ratified selection and identity semantics.

## 7. Field identity

Instance- and class-side storage are different locations:

```text
FieldId(owner=C, name="cache", side=Instance)
FieldId(owner=C, name="cache", side=Class)
```

Field identity also differs from field evidence. A declaration initializer, constructor write and arbitrary later write can contribute to one field while having different consequences for definite initialization and diagnostics.

## 8. Scope identity and binding identity

A lexical scope has a parent, extent, ordered declarations and scope kind. A binding is one declaration introduced into it. Shadowing proves spelling is not identity:

```phalcom
let x = 1
|| {
  let x = "inner"
  x
}
x
```

There are two `BindingId`s for the same text. **CURRENT:** scope/binding IDs are file-snapshot-local. Do not store them in a cache that outlives the snapshot unless the cache validity contract includes that snapshot.

## 9. Name resolution as a judgment

A useful formal view is:

```text
Γ ; M ; p ⊢ name ⇓ target
```

`Γ` is lexical scope context, `M` module/import context, `p` source/program position, and `target` is a resolved declaration/binding, ambiguity set, or unresolved state.

A simple lexical resolver is:

```text
resolve(scope, name, position):
    s = scope
    while s exists:
        candidates = declarations_named(s, name)
        candidates = candidates_visible_at(candidates, position)

        if exactly one candidate:
            return Binding(candidate.id)
        if several and language rules do not disambiguate:
            return Ambiguous(candidate.ids)

        s = parent(s)

    return resolve_nonlexical_or_unresolved(name, module_context)
```

The exact visibility rule comes from Phalcom semantics. Inference consumes the resolved target; it does not repeatedly perform string search.

## 10. Declaration order and visibility are separate from scope nesting

A declaration can be in the same scope yet unavailable before its declaration point. `same_scope(binding, occurrence)` is therefore insufficient when the language has source-order visibility. This affects completion, navigation, definite assignment, flow analysis and declaration-moving edits. Do not globally hoist names unless the language construct is normatively hoisted.

## 11. Imports have local and target identities

An import alias has at least:

```text
local binding: Provider
resolved target: ModuleId(file:///provider.ph)
```

Rename may operate on the alias; dependency analysis needs the target. Preserve both. An unresolved import should remain first-class:

```text
ImportEdge {
  source_module,
  source_range,
  requested_specifier,
  local_alias,
  target: None
}
```

Removing unresolved imports loses the dependency needed to repair them when a provider appears later. **CURRENT:** semantic tests cover unresolved-import repair after provider creation and invalidation after provider removal.

## 12. Pattern bindings are projected identities

A destructuring declaration may create several bindings from one initializer:

```text
pattern declaration
  ├─ BindingId(a) <- projection π_a(initializer)
  └─ BindingId(b) <- projection π_b(initializer)
```

Each name is a separate declaration identity; shared pattern/initializer are provenance. Future tuple/record/list pattern refinement should attach projected facts to those binding IDs.

## 13. Occurrences and source targets

An occurrence is a source event, not an entity. A useful occurrence record contains:

```text
range
role        declaration | read | write | selector-send | import | type-reference | ...
target      semantic target when resolved
recovery    whether recovery affected resolution, if needed
```

This gives a clean consumer chain:

```text
definition      occurrence -> target -> declaration location
references      target -> occurrence index
rename          target -> occurrences + conflict analysis
semantic token  occurrence -> category/modifiers
```

Do not implement references by workspace text search once semantic occurrences exist. Text search may generate candidates, but semantic identity must validate them.

## 14. Resolution results need more than `Option<T>`

`None` collapses semantically different states. A richer conceptual result is:

```rust
// Conceptual; not a required current enum.
enum Resolution<T> {
    Resolved(T),
    Unresolved(UnresolvedReason),
    Ambiguous(Vec<T>),
    RecoveryBlocked,
}
```

Possible reasons include undeclared names, missing import targets, dynamic selectors, unavailable modules and syntax recovery that prevented reliable resolution. Consumers may display several states similarly, but the semantic core should not erase distinctions needed by diagnostics, invalidation or future checking.

## 15. Source range is location, not durable identity

A byte range is excellent for locating source and poor as a long-lived declaration ID. Insert a comment and later ranges move while semantic declarations may remain conceptually unchanged.

```text
Where is it now?            -> SourceRange in this revision
Which declaration is it?    -> semantic identity
Did it survive this edit?   -> cross-revision identity matching policy
```

Range-derived IDs are legitimate when their contract is explicitly snapshot-local:

```text
NodeKey = (FileRevision, SourceRange, node-kind)
```

The error is pretending such a key is stable across edits.

## 16. Stable identity across edits is a matching problem

If future infrastructure needs declarations to survive harmless edits, stability needs an explicit matching algorithm. Common strategies are:

1. **Re-resolve each generation.** Simple and correct; caches use generation validity.
2. **Structural matching.** Match old/new declarations by semantic owner, selector/name/kind and structure.
3. **Persistent syntax identity.** Derive semantic IDs from stable incremental-syntax nodes, while still respecting semantic owner changes.
4. **Explicit declaration keys.** Use owner + selector/name when language semantics guarantee uniqueness.

Every strategy has edge cases around moves, duplicates, renames and malformed source. Do not promise stronger stability than the matcher can justify.

## 17. Identity stability matrix

For any proposed stable ID, fill a matrix like this:

| Edit/event | Should semantic declaration identity survive? | Reason |
|---|---:|---|
| whitespace before declaration | usually yes | no semantic change |
| rename declaration | policy-dependent | name may participate in current key |
| move method within same owner | often yes | source order may not define identity |
| move method to another owner | no | dispatch owner changed |
| change selector labels | no | ordinary callable selector changed |
| edit method body only | yes | declaration identity unchanged |
| delete then recreate | generally no without persistent identity | avoid accidental resurrection |
| runtime module reload | source identity may survive; runtime object identity may not | different lifetime |

This exposes accidental coupling between representation and semantics.

## 18. Program-point identity

Flow facts require a location in the semantic execution model. Source offset may be sufficient for current ordered local facts; a future CFG might use:

```text
ProgramPoint = (CallableId, BlockId, instruction-or-edge position)
```

Source provenance maps program points back to ranges. This permits desugaring or semantic IR normalization without losing diagnostics.

## 19. Runtime reflection and declaration identity

Phalcom has first-class classes/methods/reflection. Distinguish:

```text
source declaration identity
runtime class/method descriptor identity
runtime instance identity
```

Reflection can connect these domains without making their lifetimes equal. A REPL reload or reflective replacement can change a runtime method object while source declaration identity remains conceptually stable. Static caches that assume source declaration identity uniquely determines live runtime behavior need a runtime dispatch-state guard/version if mutation is observable.

## 20. Identity and incremental invalidation

Dependencies should point at the narrowest correct semantic identity:

```text
caller summary -> callee CallableId
completion surface -> ClassId / hierarchy surface
binding fact -> BindingId within file generation
import resolution -> ModuleId or unresolved import request
```

If everything depends on whole-file strings, invalidation is coarse. If identities are falsely stable, invalidation is stale. Correct incrementality begins with correct identity contracts.

## 21. Rust representation guidance

Prefer typed structured identities. Conceptually:

```rust
struct CallableId {
    owner: ClassId,
    selector: CanonicalSelector,
    side: DispatchSide,
}
```

A dedicated canonical-selector newtype may eventually prevent display strings from crossing identity APIs. Whether that refactor is worthwhile belongs to implementation/Rust engineering; the semantic rule is that canonical identity must be unambiguous at boundaries.

Avoid long-lived raw references into mutable semantic maps. Store owned/interned IDs and resolve through coherent snapshots. If IDs become arena indices, document arena/generation lifetime and use generational identity where stale index reuse is possible.

## 22. Testing obligations

Identity/resolution changes should test:

- nested shadowing with identical spellings;
- declarations before/after use where order matters;
- two modules declaring the same class name;
- two classes declaring the same selector;
- instance/class-side same-named members;
- import alias versus target identity;
- unresolved import repair and provider removal;
- pattern declarations creating multiple bindings;
- same-spelled unrelated names excluded from references;
- malformed/incomplete source around declarations;
- whitespace/comment edits preserving semantic answers;
- move/rename behavior according to the documented stability policy;
- incremental results equal to a clean full rebuild.

Useful metamorphic properties:

```text
alpha-renaming one local binding and all of its uses preserves behavior

semantically irrelevant whitespace preserves resolved targets

adding an unrelated module with the same class name does not change existing resolution
```

## 23. Failure modes to reject

Reject designs that key project classes by bare name, key methods by base name, add parameter types to ordinary selector identity, compare declarations by ranges across revisions, let snapshot-local `BindingId` escape its lifetime, treat an import alias as its target, erase unresolved import edges, use global text search as semantic reference resolution, conflate source declaration identity with runtime object identity, infer identity from a value shape, or claim an ID is stable without defining which edits preserve it.

## 24. Review questions

Before approving an identity/resolution design, answer:

- What equivalence relation does this ID implement?
- Is it source, semantic, analysis, type or runtime identity?
- Is it module-qualified and dispatch-side-qualified where required?
- What is its lifetime and which edits preserve it?
- Is range merely location or intentionally snapshot-local identity?
- Can unresolved and ambiguous states be represented separately?
- Does lexical resolution honor declaration-order rules?
- Are import alias and target both preserved?
- Can navigation/refactoring use identity rather than text replacement?
- Does future package/module evolution fit behind the identity abstraction?
- Does reflection/runtime mutation require a separate runtime version or guard?
