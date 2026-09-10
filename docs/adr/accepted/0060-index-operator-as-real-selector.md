# ADR-0060: `[]` Is a Real, Overridable Selector — No `at` Lowering

## Status

Accepted

Amended by [PDR-0032](../../pdr/0032-transition-1-language-surface-convergence.md)
on 2026-08-08 for setter identity. Amended again by `LANG004.C1.P2` on
2026-09-10 for the canonical setter value-lane spelling.

### Historical 2026-08-08 setter amendment

The original amendment recorded the fixed setter role as `(put)`. That spelling
is retained here as historical decision evidence only; it is superseded below.

### 2026-09-10 setter value-lane amendment

Bracket slots describe index arguments only. Assignment value occupies one
distinguished setter value lane, written `(_)`:

```text
getter: [_,default]
setter: [_,default]=(_)
```

Canonical declarations are `[_ index] { ... }` and
`[_ index, default fallback]=(_ value) { ... }`. The `_` is not appended to
the bracket selector slots and does not permit a positional argument after a
labeled index argument.

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
   - `expr[idx] = value` → send `[_]=(_)`
   - `expr[]` → send `[]`
   - `expr[] = value` → send `[]=(_)`

2. **Definition syntax** — class members declare bracket methods the same
   shape as parenthesized ones, substituting `[`/`]` for `(`/`)` and reusing
   the existing labeled-parameter grammar for the setter arm:

   ```
   class Example {
     [idx] {}
     [idx, label:] =(_ value) {}
     [] {}
     []=(_ value) {}
   }
   ```

3. **Core classes must opt in explicitly.** `List`/`Map`/`Set`/`Tuple`/`Range`
   do not automatically gain `[]` behavior from this ADR — each must define
   its own `[_]`/`[_]=(_)` (or reject via DNU, e.g. `Tuple[_]=(_)` for
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
  sends to `[_]`/`[_]=(_)`/`[]`/`[]=(_)` instead of `at`
  (`phalcom-core::method::SignatureKind::Subscript`); core collection classes
  define explicit `[]` `.ph` wrapper methods (delegating to `at`) or accept
  the DNU implementations preserve current indexing behavior.
- **Landed** (U-INDEX, `../../forge/units/U-INDEX/u28-index.md`): call-site
  `expr[args...]`/`expr[args...] = value` is arg-list-shaped, not
  single-index — `xs[i, j]` sends `[_,_]`, `cache[key, default: fallback]`
  sends `[_,default]`, generalizing this ADR's single-index examples above to
  any arity/label combination a collection author opts into, with zero
  further parser/compiler changes. `List`/`Map` define `[_]`/`[_]=(_)`;
  `Tuple` defines `[_]` only (immutable, no `[_]=(_)`, so `tup[i] = v`
  correctly `doesNotUnderstand`); `Set`/`Range` define neither (no `at`
  either, per collection-protocol.md §2).
