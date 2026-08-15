# Binding, Scope Graphs, Name Resolution, and Captures

## 1. Bind before infer

A source occurrence of `x` is not semantically the string `"x"`; it denotes a binding according to lexical, module, class, and language-specific lookup rules. Flow facts, rename, references, type facts, capture analysis, and diagnostics must therefore attach to semantic identities.

Use the judgment

```text
Σ ; s ⊢ x ⇓ b
```

where `Σ` is the module/project environment, `s` the starting scope, `x` the source name, and `b` the resolved semantic target. Resolution may instead produce `Unresolved`, `Ambiguous(candidates)`, or a recovery-only target; do not flatten these to the same state.

## 2. Scope graphs

A practical scope graph separates scope structure from occurrences:

```rust
struct ScopeInfo {
    parent: Option<ScopeId>,
    owner: ScopeOwner,
    range: SourceRange,
    bindings: Vec<BindingId>,
}

struct BindingInfo {
    name: Symbol,
    kind: BindingKind,
    scope: ScopeId,
    declaration: SourceRange,
}
```

Indexes should answer both directions:

```text
source occurrence -> SemanticTarget
SemanticTarget    -> source occurrences
offset            -> containing scope chain
scope             -> visible bindings
```

**CURRENT:** Phalcom's LSP semantic layer already has `ScopeGraph`, `ScopeId`, `BindingId`, `BindingInfo`, `NameResolution`, semantic occurrences, and queries for visible bindings. Preserve this identity-first architecture when generalizing it beyond LSP.

## 3. Lexical resolution and shadowing

For an ordinary lexical parent chain:

```text
resolve(scope, name):
    if scope declares name:
        return declaration chosen by language rule
    if scope has parent:
        return resolve(parent, name)
    return unresolved
```

This simple algorithm is insufficient if imports, class members, implicit receivers, or separate namespaces participate; represent those as explicit edges or later resolution stages instead of adding ad hoc fallback string searches.

Example:

```phalcom
let x = 1
{
  let x = "inner"
  consume(x)
}
consume(x)
```

The two declarations must have distinct `BindingId`s. The inner `x` occurrence resolves to the inner binding; the final occurrence resolves to the outer one. A fact table keyed by `(name, offset)` is merely a lookup optimization and cannot replace the binding identity.

## 4. Declaration order, hoisting, and temporal visibility

Scope membership and visibility are different concepts. A declaration may belong to a scope yet not be usable before its declaration if Phalcom's normative semantics say so. Model visibility explicitly:

```text
visible(binding b, program point p)
```

rather than assuming `b ∈ declarations(scope(p))` implies usability.

This distinction becomes essential for definite assignment, mutually recursive declarations, module initialization, and editor recovery. If Phalcom later allows selected declaration kinds to be mutually visible, encode that per declaration category rather than globally hoisting everything.

## 5. Patterns bind multiple identities

Pattern declarations should lower to a set of binding introductions plus match/destructure semantics. For a hypothetical product pattern:

```phalcom
let (x, y) = point
```

there are at least two distinct concerns:

1. scope construction creates bindings for `x` and `y` according to the pattern's binding rule;
2. flow analysis assigns facts to those `BindingId`s only on paths where destructuring succeeds, if failure is possible.

Do not let a recursive AST visitor infer bindings opportunistically while evaluating the initializer; that entangles declaration identity with flow.

## 6. Captures and closure environments

A use of binding `b` inside nested callable/block `c₂` captures `b` when `b` is owned by an enclosing callable/block `c₁` and resolution crosses that callable boundary.

```text
captures(c₂, b) iff
    resolve(use in c₂) = b
    and owner(b) is outside c₂'s local frame
```

Capture analysis should be derived from resolved uses, not spelling. The compiler can then choose upvalue/cell representation, while static analyses can reason about aliasing and mutation.

Example:

```phalcom
let n = 0
let next = || {
  n = n + 1
  n
}
```

The closure does not capture a snapshot of the abstract value `0`; it captures the storage semantics defined by Phalcom. If a captured binding is mutable, analyses must account for writes across closure invocations. If the closure escapes or crosses fiber suspension, the lifetime/effect implications become stronger.

## 7. Blocks and non-local control

Phalcom blocks/closures can interact with non-local control. Name resolution must identify not only captured data bindings but also control ownership when a `return` can target an enclosing callable.

A useful normalized representation distinguishes:

```text
ReturnLocal(value, target_callable)
ReturnNonLocal(value, home_callable)
```

The parser may use one surface syntax; semantic lowering resolves the target. This prevents every CFG/checker/prover pass from rediscovering lexical return ownership.

Constructing a block is not executing it. Capture analysis occurs at construction; block body effects become conditional on invocation unless semantics guarantee immediate execution.

## 8. Members are not lexical locals

Message/member lookup must not be disguised as lexical resolution. For:

```phalcom
receiver.foo(arg)
```

`receiver` may resolve lexically, but `foo` is a selector/member operation governed by dispatch semantics. Likewise an implicit receiver form, if supported, may require a staged rule:

```text
1. attempt lexical binding according to syntax category
2. if syntax denotes/send requires implicit receiver, construct selector
3. resolve/approximate member through dispatch model
```

Do not make a single `HashMap<String, BindingId>` responsible for both lexical and runtime member identity.

## 9. Imports and qualified names

Qualified names should resolve through module identities, not textual concatenation. **CURRENT:** current semantic class identities are module-qualified and `resolve_named_class` can interpret imported bindings and core fallback. Future package/project resolution should preserve canonical `ModuleId` identity even if path aliases or registry coordinates become richer.

An unresolved import is not the same as an absent local declaration. Keep a diagnostic/recovery state that can later be re-resolved when the dependency appears.

## 10. Rename correctness

Rename is a strong test of the resolver. A sound rename is not “replace equal text.” It requires:

```text
selected occurrence -> target identity
all occurrences(target) -> candidate edits
for each edit:
    simulate/new-name conflict rules
    ensure occurrence still resolves to intended target
```

Potential conflicts include shadowing, import collision, member selector changes, labels/selector canonicalization, and generated/core names. If the analysis cannot establish safety, the refactoring should refuse or narrow its scope rather than guess.

## 11. Recovery-aware resolution

In an editor, half-written source can temporarily violate grammar or declaration uniqueness. Separate:

- `Resolved(b)` — semantic resolution under recoverable structure;
- `Unresolved` — no matching declaration;
- `Ambiguous(candidates)` — multiple plausible declarations;
- `Blocked(dependency)` — cannot decide because an import/module is unavailable;
- `RecoverySynthetic(id)` — parser/semantic recovery introduced a temporary anchor.

Recovery-generated bindings should not silently become valid language declarations in batch checking.

## 12. Complexity and indexing

Naively walking lexical parents is usually cheap for one query but can become expensive across all occurrences or repeated LSP requests. Build once, query many. Typical indexes:

```text
scope_at_offset: interval index
bindings_by_scope: compact vector/map
occurrence_at_offset: sorted range index
occurrences_by_target: target -> compact ranges
```

Intern names/selectors only if profiling justifies it. More important is avoiding repeated whole-AST resolution per hover/completion request.

## 13. Testing and properties

Test at least:

- nested shadowing and same-name parameters/locals;
- declaration-before-use rules;
- pattern bindings;
- closure capture/read/write;
- non-local return ownership;
- imported and qualified names;
- same class name in different modules;
- unresolved/missing imports;
- malformed declaration recovery;
- rename conflict detection;
- alpha-renaming: consistently renaming a local binding should preserve behavior and semantic relationships;
- incremental/full equivalence after inserting/removing a shadowing declaration.

## 14. Review questions

1. Which namespace/lookup rule owns this source token?
2. Is the result a lexical binding, class/module identity, selector/member, or recovery artifact?
3. Does visibility depend on program point, not only scope membership?
4. Are captures derived from resolved bindings?
5. Can a nested closure write this binding or escape across suspension?
6. Does a rename preserve resolution after the edit?
7. Can missing dependencies be distinguished from a genuine unresolved-name error?
8. Is any consumer still re-resolving names independently?
