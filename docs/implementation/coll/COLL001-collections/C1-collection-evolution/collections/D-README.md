# Phalcom Collections — Spec D: Core Eager Collection Operations

Status: implementation-spec bundle. This continues Specs A–C and implements the ratified eager collection-operation layer without pulling forward Spec E's lazy iterator/boundedness machinery or Spec F's expansion/argument-pack machinery.

The repository baseline inspected while producing this plan is `aureat/phalcom-lang` main at commit `5c73279157891ca8e2fc045db5e7dff683c0be5b`. Every implementation session MUST re-inspect actual HEAD before editing. Paths below are architectural anchors, not frozen line numbers.

## Dependencies

Implement after Spec C in the intended sequence.

- D.1 requires canonical Unit from A and the existing cursor iteration substrate. B.2 should be landed first if Map is to participate through its `keys`/`values`/`entries` model, because D.1 removes the old special two-argument `Map#each` behavior in favor of the ordinary one-value Iterable contract.
- D.2 requires C.1's negative-index/error substrate and C.3's `List::replaceSlice_` primitive. It deliberately reuses that one splice seam instead of adding native insert/remove/pop primitives.
- D.3 requires D.1, B.1's duplicate-safe Map contract, and B.2's `Entry`/Map view layer. Its grouping/partition/conversion lane is executable. The sorting lane is explicitly gated by still-open ordering-protocol details described in D.3; do not invent them during implementation.

## Repository diagnosis

HEAD has substantial pre-existing collection functionality, but several parts contradict the ratified collection design:

1. `Iterable#map` was deliberately changed by U-SEQ to return a lazy `MapView`. The ratified design now says operations sent directly to concrete collection receivers are eager; laziness starts through an explicit iterator/pipeline receiver.
2. `Iterable#reduce(init, f)` is today's explicit-initial accumulator. The ratified vocabulary splits that operation into `fold(initial:using:)` and reserves `reduce(using:)` for no-initial reduction returning `Option`.
3. Predicate queries are currently positional (`all(f)`, `any(f)`, `count(f)`, `find(f)`). The ratified selector vocabulary uses `where:` for predicate-qualified queries.
4. U-SEQ exposes `where`, `skip`, and `take` directly on `Iterable`, creating lazy views without an explicit `.iter` receiver. That surface belongs to Spec E's explicit lazy pipeline.
5. `Map#each(f)` currently invokes its callback with `(key, value)` even though the same selector on `Iterable` is a one-value traversal. Selector identity does not encode callback arity, and the new design forbids choosing semantic behavior by closure arity. Entry traversal belongs on `map.entries`.
6. `List#add` and several other historical mutations return the receiver for chaining. Ratified mutation commands return Unit unless they carry a semantic payload.
7. List literal parsing currently relies on the chainability of `add`: `[a,b]` lowers through `List.new().add(a).add(b)`. The public mutation contract cannot be corrected cleanly until literal construction is decoupled from that method.
8. C.3 already provides the right native structural seam for List mutation: a bulk `replaceSlice_(start,end,replacements)` splice. D should reuse it instead of growing a native primitive for every mutation verb.
9. Map grouping and duplicate-safe `toMap` are expressible over B's public Map/Entry substrate and should remain `.ph` code.
10. Sorting is not fully implementable from the ratified text alone: the semantic `Ordering` type is named, but exact case surface, the default comparison protocol, and `sorted on:` key-evaluation timing are not yet pinned. D records that gate rather than silently choosing semantics.

## Phase order

### D.1 — Eager Traversal, Queries, and Reduction

Replace the obsolete direct-lazy `Iterable#map` behavior with eager concrete materialization; add eager `flatMap`; correct `each` to Unit; add selector-distinct indexed variants; migrate predicate queries to `where:`; implement short-circuit identities; split `fold` from `reduce`; retire the old direct lazy-view sugar/classes that bypass `.iter`; and remove Map's incompatible two-argument `each` override.

Initial eager result family for generic transforms is `List`. This is an executable minimum, not a final cross-family preservation matrix: the ratified specification explicitly defers the complete return-family table. Per-family overrides may refine this later without changing selector identities.

Expected primitive-floor delta: **0**.

Artifact: `D.1-eager-traversal-queries-and-reduction.md`.

### D.2 — List Mutation Commands and Literal Decoupling

Move List literals off the chainable `add` desugaring, then install the ratified command/payload mutation surface: `append`, `prepend`, `clear`, recoverable `insert`, indexed removal, pops, `removeAll where:`, and `swap`. Reuse C.3's splice primitive for structural edits and C.1's index/error rules.

Two underspecified operations are intentionally gated inside this phase rather than guessed:

- `remove(value)` — the ratified document says first-match behavior is expected but MUST be confirmed if not otherwise stated;
- `move(from:to:)` — the document does not pin whether `to:` denotes a pre-removal or post-removal coordinate when moving forward.

Do not ship either selector until those semantics are ratified. Everything else in D.2 is executable independently.

Expected primitive-floor delta: **0** beyond C.3. D.2 adds a List literal AST/build opcode but no native method binding.

Artifact: `D.2-list-mutation-and-literal-construction.md`.

### D.3 — Grouping, Partitioning, Conversions, and Sorting Gate

Implement eager `group by:`, `partition where:`, `toList`, `toSet`, duplicate-rejecting `toMap`, and `toMap merging:` using B's ordered Map and Entry substrate. Add `DuplicateKeyError` as an ordinary language error class unless already present on implementation HEAD.

The same document contains the implementation plan for `sorted`/`sort`, but marks shipping as blocked until three semantic details are pinned: Ordering case surface, default comparison protocol, and `on:` key-extraction evaluation count. This keeps the next design conversation narrowly scoped and prevents D's executable grouping/conversion work from being blocked by sorting.

Expected primitive-floor delta for the executable lane: **0**.

Artifact: `D.3-grouping-conversions-and-sorting.md`.

## Cross-phase invariants

After D.1:

- sending `map`, `filter`, or `flatMap` directly to a current concrete Iterable performs work immediately and returns a concrete List;
- direct collection `map` never returns `MapView` or another lazy stage;
- lazy-pipeline selectors are not reachable directly as `collection.where/skip/take`; Spec E owns their replacement under `.iter`;
- `each` invokes the callback once per encountered value in encounter order and returns Unit;
- indexed callbacks receive encounter ordinals `0,1,2,...`, never opaque cursor objects;
- `find where:`, `any where:`, `all where:`, and `none where:` short-circuit;
- empty `any/all/none` yield `false/true/true`;
- `fold(initial:using:)` returns the initial value on empty input without invoking the callback;
- `reduce(using:)` returns `None` on empty input, `Some(element)` on singleton input without invoking the callback, and `Some(result)` otherwise;
- callback failures propagate through the ordinary language error model;
- no callback-arity inspection selects collection semantics;
- Map entry-pair traversal is expressed through `map.entries`, not a special callback convention hidden behind `Map#each`.

After D.2:

- `[]` creates a fresh List without sending public mutation selectors;
- nonempty List literals evaluate elements exactly once in lexical order;
- canonical List command mutations no longer return the receiver for fluent chaining;
- successful `append`, `prepend`, `clear`, `removeAll where:`, and `swap` return Unit except where the ratified API specifies another payload;
- `insert(value, at:)` returns `Result<Unit, IndexError>` and accepts insertion position `size`;
- indexed removal returns `Result<removed, IndexError>`;
- pops return Option and never raise merely because the List is empty;
- retained-element order after `removeAll where:` is stable;
- no new insert/remove/pop native primitive exists: C.3's splice is the single structural edit seam.

After D.3 executable work:

- grouping preserves source encounter order within every group and first-seen group-key order in the Map;
- partition returns `(accepted, rejected)` in that exact order and preserves encounter order on both sides;
- `toMap` rejects duplicate equivalent keys instead of overwriting;
- `toMap merging:` invokes conflict resolution in source encounter order and preserves the Map position of the first occurrence;
- stored `None` values remain distinguishable from missing keys throughout conversion because B's Map insertion/lookup returns real Option;
- conversions do not inspect callback arity and do not auto-wrap callback failures in Result.

## Deliberate exclusions

Spec D does **not** implement or decide:

- the lazy iterator/pipeline object model — E;
- `.iter` itself and boundedness propagation — E;
- compile-time rejection of provably unbounded eager exhaustors — E;
- Range/Progression direction/step semantics beyond what C supplied;
- `*`/`**`/`***` expansion — F;
- full Set/ImmutableSet API, order, equality, or hashing;
- mutation-during-iteration or Map-view fail-fast/live/snapshot policy;
- exact eager result family for every concrete collection;
- generic collection capability/protocol hierarchy;
- destructuring;
- printing/debug policy;
- the final `associate key:value:` API, whose exact shape remains subject to the broader conversion specification;
- sort stability;
- `remove(value)` duplicate-occurrence policy until ratified;
- `move(from:to:)` destination-coordinate semantics until ratified;
- numeric-tower migration; D inherits C's temporary integral-Number index seam until Int/Float split lands.

## Verification gate

Each phase must re-read implementation HEAD before work and update migration assumptions accordingly. Run focused language/core tests during development. Completion requires:

```sh
./scripts/verify.sh --full
```

D.1 and D.2 are breaking surface migrations. Search the full repository, including benchmarks and examples, for the retired/changed selectors before declaring completion. Do not limit migration audit to `phalcom-core/tests`.
