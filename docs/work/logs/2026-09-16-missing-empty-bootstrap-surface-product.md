# Missing Empty Bootstrap Surface Product

- Date: 2026-09-16
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Status: Fixed locally and focused-tested; not committed
- Affected checkpoint: `LANG005.C5`

## Symptom

The concrete associated-iterator regression reached runtime as an ordinary
dynamic getter send and failed with:

```text
<C5List instance> does not understand 'iterator'
```

Its compiled lowering contained no trait invocation for `source.iterator`.

## Root cause

The failure was not in associated-type binding or conformance witness
selection. Semantic analysis aborted while constructing a declaration surface
because it recorded a dependency on the canonical `Iterable` declaration
surface, but no query product existed for that empty built-in surface:

```text
query dependency DeclarationSurface(... Iterable ...) is not Ready
```

Only built-in declarations with registered dispatch members had bootstrap
surface products. The update failure then returned the session's empty fallback
snapshot, which removed all trait surfaces, conformances, and invocation
evidence from downstream compilation.

## Fix

Bootstrap declaration-surface products are now published for every built-in
declaration. Declarations without dispatch members receive a canonical empty
surface. This preserves fine-grained dependency tracking without adding
runtime conformance lookup or iterator-specific behavior.

## Verification

Focused regression:

```text
language::traits_p3_closure::concrete_associated_iterator_preserves_state_freshness_identity_and_exhaustion
```

Result: **1 passed, 0 failed** (`562` filtered out).

No broader verification was run. Existing unrelated working-tree changes were
left untouched.
