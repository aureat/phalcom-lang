# Environments, Stores, and Binding

Correct semantics for lexical scope require separating names, binding identities, runtime locations, and values. This becomes essential with shadowing, assignment, closures, modules, and incremental analysis.

## 1. Four different things

```text
name      textual spelling: "x"
binding   declaration identity: BindingId(17)
location  mutable cell: ℓ42
value     current content: 3
```

Never use spelling alone as semantic identity.

## 2. Lexical resolution

Static resolution can be modeled:

```text
resolve(scope, "x", position) = BindingId(17)
```

A nested declaration creates a distinct binding:

```phalcom
let x = 1
|| {
  let x = 2
  use(x)   // inner binding
}
use(x)     // outer binding
```

The occurrences have identical text and different targets.

## 3. Runtime environment

For immutable variables:

```text
ρ : BindingId -> Value
```

For mutable variables:

```text
ρ : BindingId -> Loc
σ : Loc -> Value
```

Assignment changes store content rather than source binding identity.

## 4. Why locations matter

Suppose:

```phalcom
let n = 0
const inc = || { n = n + 1 }
const read = || { n }
inc()
read()
```

Under shared mutable capture:

```text
ρ(inc.capture[n]) = ℓn
ρ(read.capture[n]) = ℓn
```

After `inc`, `σ(ℓn)=1`, so `read` sees `1`. Copying the current integer into each closure would implement different semantics.

## 5. Environment extension

Invocation/declaration extends an environment without mutating lexical parents conceptually:

```text
ρ' = ρ[b ↦ ℓ]
σ' = σ[ℓ ↦ v]
```

This makes shadowing natural: the inner environment contains a new binding/location while outer environment remains reachable through lexical ancestry.

## 6. Immutable capture

An implementation may copy an immutable value into a closure if observationally equivalent. The semantic model can still describe the closure as retaining the binding's value. Optimization is separate from meaning.

## 7. Declaration timing

Specify whether a declaration is visible:

- throughout scope;
- only after declaration point;
- during its own initializer;
- in mutually recursive declaration groups.

Do not let parser traversal order accidentally decide the language rule.

## 8. Initialization state

If a binding can exist before initialization, model that state:

```text
Cell = Uninitialized | Initialized(Value)
```

Reading `Uninitialized` yields a defined error or is statically forbidden. Do not confuse private VM sentinels with surface absence values.

## 9. Parameters

Invocation creates parameter bindings. Selector labels determine argument association; binding identity determines occurrences inside body.

```text
parameter declaration p
 -> BindingId bp
 -> fresh location ℓp
 -> argument value
```

Defaults, rest packs, and destructuring add evaluation/binding rules but should preserve this distinction.

## 10. `self`

`self` is best modeled as a distinguished activation binding/value, normally immutable. Nested blocks retain lexical receiver access according to Phalcom rules. Do not resolve `self` from whichever method dynamically invokes the block.

## 11. Fields are not lexical bindings

A field such as `_x` has owner/layout identity:

```text
FieldId(ownerClass, name, side)
```

`self._x` denotes receiver-local storage. A local `x` and a field `_x` are different namespaces.

## 12. Implicit-self fallback

If ordinary unresolved names use a priority such as:

```text
local -> upvalue -> known global/module -> implicit self send
```

then implicit-self is not a lexical binding. It is a dispatch fallback and therefore has selector/access/reference semantics.

## 13. Modules and global bindings

Top-level identity should be module-qualified:

```text
(ModuleId, BindingId/name)
```

Two modules can both declare `Point` without merging identities. Imports may create local aliases whose target is another module/entity identity; alias identity and target identity are distinct for rename/reference semantics.

## 14. Recursive declarations

Classes/protocols/types may require shells so declarations can refer to themselves or mutually recursive peers before all metadata is resolved. Separate:

```text
declaration identity creation
metadata/type resolution
runtime initialization
```

Do not fake recursion by repeated lookups by bare name.

## 15. Upvalues and lowering

The compiler may classify captured locals as upvalues and allocate closure cells. This is an implementation of lexical storage sharing. Semantic rules should not depend on exact upvalue indices.

Correspondence:

```text
source BindingId -> semantic location
compiler local/upvalue slot -> same logical value evolution
```

## 16. Lifetimes

A location remains semantically live while reachable through an active environment/closure/module/object, even after its original stack frame is gone. This is why captured mutable locals may need heap promotion/cells.

## 17. Reflection and debugging

If locals are not reflectable, environment records remain specification mechanisms. If future debugger/reflection APIs expose lexical variables, lifetime/identity/source-name policies become observable and should be added deliberately.

## 18. Incremental tooling IDs

LSP `BindingId`s may be snapshot-local while language identity is source-declaration-based. Do not treat editor IDs as permanent runtime identities. Stable cross-revision identity requires an explicit policy.

## 19. Common mistakes

- mapping variable name directly to inferred type/value across nested scopes;
- treating assignment as creation of a new lexical binding;
- copying mutable captured values into each closure;
- letting parser traversal order decide declaration visibility;
- merging same-named module classes;
- resolving implicit-self sends as local variables.

## 20. Competency checks

1. Draw environment/store after creating two closures over one mutable local.
2. What differs between shadowing and reassignment?
3. Why is a field identity not a `BindingId`?
4. How should import alias and imported class identity be represented separately?
5. Which semantics changes if a declaration becomes visible during its own initializer?
