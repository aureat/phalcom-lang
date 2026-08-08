# ADR-0060: `[]` Is a Real, Overridable Selector — No `at` Lowering

## Status

Accepted

Amended by [PDR-0032](../../pdr/0032-transition-1-language-surface-convergence.md)
on 2026-08-08 for setter identity.

### 2026-08-08 setter amendment

Bracket slots now describe index arguments only. Assignment value occupies the fixed
setter role `(put)`:

```text
getter: [_,default]
setter: [_,default]=(put)
```

Canonical declarations are `[_ index] { ... }` and
`[_ index, default fallback]=(put value) { ... }`. Historical `[_,put]` examples below
record the original motivation but no longer define current selector identity.

## Context

ADR-0055 lowered `expr[idx]` / `expr[idx] = value` to ordinary `at(_)` /
`at(_,put:)` sends, explicitly to avoid a new selector. That decision is
reversed: `[]` already exists as a reservable selector slot in the dispatch
encoding, and no core class (`List`, `Map`, `Set`, `Tuple`, `Range`) currently
defines it — the slot is free, unclaimed by the floor.

## Decision

Index syntax compiles to direct sends against dedicated bracket selectors.
No `at`/`at(_,put:)` lowering occurs.

1. **Expression → selector mapping**:
   - `expr[idx]` → send `[_]`
   - `expr[idx] = value` → send `[_,put]`
   - `expr[]` → send `[]`
   - `expr[] = value` → send `[put]`

2. **Definition syntax** — class members declare bracket methods the same
   shape as parenthesized ones, substituting `[`/`]` for `(`/`)` and reusing
   the existing labeled-parameter grammar for the setter arm:

   ```
   class Example {
     [idx] {}
     [idx, put:] {}
     [] {}
     [put:] {}
   }
   ```

3. **Core classes must opt in explicitly.** `List`/`Map`/`Set`/`Tuple`/`Range`
   do not automatically gain `[]` behavior from this ADR — each must define
   its own `[_]`/`[_,put]` (or reject via DNU, e.g. `Tuple#[_,put]` for
   immutability) to keep working under direct dispatch.

## Consequences

- Supersedes ADR-0055 in full (not partial) — the `at` lowering path is
  removed, not kept as a fallback.
- User classes are free to define `[]`/`[]=` today since no core class has
  claimed the selector yet.
- Parser gains a dedicated `[...]` class-member production
  (`Parser::parse_index_member`, dispatched from `parse_class_member` —
  *not* `parse_method_name`, since a bracket method carries no separate name
  token at all, unlike `==`/`+`/other operator selectors); compiler emits
  sends to `[_]`/`[_,put]`/`[]`/`[put]` instead of `at`
  (`phalcom-core::method::SignatureKind::Subscript`); core collection classes
  define explicit `[]` `.ph` wrapper methods (delegating to `at`) or accept
  the DNU implementations preserve current indexing behavior.
- **Landed** (U-INDEX, `docs/forge/units/U-INDEX/plan.md`): call-site
  `expr[args...]`/`expr[args...] = value` is arg-list-shaped, not
  single-index — `xs[i, j]` sends `[_,_]`, `cache[key, default: fallback]`
  sends `[_,default]`, generalizing this ADR's single-index examples above to
  any arity/label combination a collection author opts into, with zero
  further parser/compiler changes. `List`/`Map` define `[_]`/`[_,put]`;
  `Tuple` defines `[_]` only (immutable, no `[_,put]`, so `tup[i] = v`
  correctly `doesNotUnderstand`); `Set`/`Range` define neither (no `at`
  either, per collection-protocol.md §2).
