# ADR-0060: `[]` Is a Real, Overridable Selector — No `at` Lowering

## Status

Accepted

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
- Parser must gain a bracket arm in method-name/class-member parsing
  (`parse_method_name`/`parse_class_member`, `phalcom-ast/src/parser.rs`);
  compiler must emit sends to `[_]`/`[_,put]`/`[]`/`[put]` instead of `at`;
  core collection classes need explicit `[]` primitives or DNU implementations
  to preserve current indexing behavior. None of this is implemented yet.
