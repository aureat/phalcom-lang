# Testing, Property Testing, Metamorphic Testing, and Fuzzing

## Test semantic core directly

For a semantic feature, first test:

```text
parse fixture
update SemanticDb/Engine
query semantic fact
assert identity/domain/provenance
```

Then test LSP rendering. This isolates semantic bugs from protocol formatting.

## Unit tests

Good targets:

- selector canonicalization;
- join algebra;
- scope lookup;
- union widening;
- argument-to-parameter mapping;
- module/path identity normalization;
- summary equality/effects.

## Fixture tests

Use readable `.ph` fixtures for whole semantic behavior. Include source markers/ranges where
possible so targeted query tests are stable.

## Golden/snapshot tests

Useful for:

- semantic occurrence dump;
- scope graph dump;
- inferred facts per marker;
- diagnostic rendering;
- completion lists/hover text.

Keep snapshots semantic/deterministic; avoid incidental hash iteration order.

## Negative tests

Every positive inference should have cases where it must *not* infer:

- dynamic selector;
- unresolved class;
- conflicting branch;
- unknown call;
- module alias collision;
- malformed source.

False certainty is more dangerous than missing completion.

## Incremental tests

A good incremental test performs a sequence:

```text
load A+B
query fact
edit A
query changed fact in B
assert generation advanced
assert rebuild frontier
remove A
assert unresolved/unknown
restore A
assert recovery
```

## Metamorphic tests

These are unusually effective for language tooling.

### Alpha rename

Rename a local consistently without capture -> semantic shapes/types/results should remain
isomorphic except source names/ranges.

### Formatting

Format source -> semantic identities/facts should be equivalent modulo ranges/trivia.

### Parentheses/syntactic sugar

Add redundant grouping or equivalent sugar -> semantics unchanged where language defines
same meaning.

### Explicit inferred annotation

Once typing exists: insert an annotation equal to synthesized type -> program should still
check and runtime semantics unchanged.

### Reordering independent declarations

Where semantics permits, reordering independent declarations should not change unrelated facts.

## Property tests

Potential properties:

```text
join idempotent
join commutative/associative for domain
union size never exceeds cap unless widened to Unknown
scope visible bindings contain no duplicate spelling
all occurrence target ranges inside file bounds
all CallableId owners exist in class surface or trusted native set
published snapshot generation internally consistent
```

Use proptest/quickcheck style tooling if already in workspace or justified.

## Fuzz parser + semantic pipeline

Fuzzing target should not only parse. Feed arbitrary/recovered AST source through semantic
construction/query to catch:

- panics;
- recursion blowups;
- pathological union/provenance growth;
- invalid range assumptions;
- module graph corner cases.

Assertions:

```text
no panic/UB
bounded memory/time policy
all ranges valid or recovery-safe
snapshot query does not deadlock
```

## Differential testing

When replacing one semantic implementation with another (structured flow -> CFG, legacy index ->
semantic occurrence index), run both on corpus and compare agreed observable facts during migration.

Delete dual implementation after confidence; do not make permanent dual truth.

## Runtime differential tests

For exact semantic claims that predict runtime behavior, generate/execute small programs and
compare:

- dispatch target/returned class where introspection permits;
- selector construction;
- constructor result;
- collection literal class;
- field side behavior.

Do not expect advisory unions/unknowns to exactly match one runtime execution.

## Performance regression tests

Track:

- number of flow passes;
- modules/callables recomputed after local edit;
- update latency on representative workspace;
- query latency;
- snapshot size/memory if measurable.

A semantic feature that passes correctness tests but invalidates every file on each keystroke is
not finished.

## Test names

Name tests by invariant/failure, not implementation function:

Good:

```text
same_named_classes_in_distinct_modules_do_not_merge
branch_assignments_join_receiver_shapes
removing_callee_invalidates_dependent_return_summary
dynamic_pack_does_not_fabricate_static_selector
```

This preserves intent across refactors.
