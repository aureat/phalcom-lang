# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `COLL006`
- Checkpoint: `COLL006.C1`
- Plan/task: `COLL006.C1.P6`, T4 IndexedDict source API / G4
- Repository: `aureat/phalcom-lang`
- Worktree: `.worktrees/telos-collections-upgrade`
- Branch: `feat/telos-collection-upgrade`
- Current state: uncommitted P6 implementation; TDR-0092 is accepted
- Consultation status: ruling received; semantic correction implemented; P6 G4 remains open

## 2. Escalation trigger

The approved `IndexedDict.insert(entry, at:)` contract returns
`Result<Unit, Error>`, with `DuplicateKeyError` for a duplicate key and
`IndexError` for an invalid position. The source implementation cannot
currently construct the duplicate branch: `Result::Error(duplicateError)`
reports generic inference underconstraint and conflict even when the result
and error locals are explicitly annotated. Changing the public error contract,
generic variant inference, or the compiler's subtype conversion path would
cross an accepted API or compiler/semantic boundary. This meets the mandatory
consultation trigger for a cross-layer contract and for a nontrivial failure
that persisted through bounded corrective attempts.

## 3. Required invariants

- Preserve the architect-approved P6 public API in
  `docs/specs/collection-families.md`, including both error variants and the
  common `Result<Unit, Error>` result type, unless the architect explicitly
  authorizes an amendment.
- Preserve the accepted rooted-snapshot equality policy and the TDR-0092
  inventory of exactly 29 native bindings. Neither is implicated by this
  failure.
- Do not replace a recoverable `Result` contract with throwing behavior, erase
  the concrete error, or add a native binding as a workaround without
  architectural authorization.
- Keep the `Result<T,E>` error type and canonical semantic/type authority in
  the existing language/compiler layers; do not add P6-local inference or
  conversion rules.

## 4. Expected architecture and contract

`IndexedDict.insert(_ entry: (key: K, value: V), at i: Int)` validates and
normalizes the index, then performs a strict duplicate-checked native
insertion. Invalid positions return `Result::Error(IndexError.new(...))`;
duplicate keys return `Result::Error(DuplicateKeyError.new(key))`; successful
insertion returns `Result::Ok(())`. The normative contract is
`Result<Unit, Error>`, because both concrete error classes derive from `Error`.

`Result<T,E>` is declared in
`phalcom-core/core/universe/src/errors/result.ph` as a native data type with
`Error(_ error: E)`. `DuplicateKeyError` and `IndexError` both derive from
`Error` in `phalcom-core/core/universe/src/errors/argument.ph`. The accepted
collection contract is documented in §3 of
`docs/specs/collection-families.md`.

## 5. Observed behavior

The focused command

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core p6_mutable_hash_families_preserve_callback_snapshot_and_gc_contracts -- --nocapture
```

selected the requested Rust test but failed while compiling the Universe
source. The diagnostics point to the duplicate-error branch in
`phalcom-core/core/universe/src/collections/indexed_dict.ph`:

```text
GenericInferenceUnderconstrained:
  generic inference has insufficient value-producing evidence
  at CopyRange { start: 2899, end: 2928 }
GenericInferenceConflict:
  generic argument constraints conflict
  at CopyRange { start: 2913, end: 2927 }
```

The diagnostic ranges identify the `Result::Error(duplicateError)` expression
in the positional `insert` method. The invalid-index branch using
`Result::Error(IndexError.new(...))` does not produce this diagnostic. The
current compiler output establishes the failing source expression, but not
whether the root cause is variant-constructor generic inference, subclass-to-
base conversion, generic method context, or a missing canonical semantic
product.

## 6. Relevant source path

```text
docs/specs/collection-families.md §3
  -> IndexedDict.insert(entry, at:) : Result<Unit, Error>
  -> phalcom-core/core/universe/src/collections/indexed_dict.ph
       rawInsertAt(...) rejects a duplicate
       DuplicateKeyError.new(key)
       Result::Error(duplicateError)
  -> Result<T,E>::Error(E), errors/result.ph
  -> generic inference / subtype conversion in the semantic/compiler path
```

Related type declarations:

- `phalcom-core/core/universe/src/errors/result.ph`
- `phalcom-core/core/universe/src/errors/argument.ph`
- `phalcom-core/core/universe/src/collections/iterator_laws.ph` contains a
  source-level `toMap` precedent using `Result<..., DuplicateKeyError>`; that
  does not establish conversion to the broader `Error` type.

## 7. Attempts already made

The following bounded source-level forms were tried and did not clear the
duplicate branch diagnostic:

1. Direct `Result::Error(DuplicateKeyError.new(entry.key))`.
2. Widening the key to `Dynamic` and assigning the constructed error to an
   explicitly typed `Result<Unit, Error>` local.
3. Assigning the constructed duplicate error to a local typed as `Error`; the
   compiler reported an initializer type mismatch.
4. Assigning the error to a `Dynamic` local and constructing an explicitly
   typed `Result<Unit, Error>`; variant construction still reported generic
   inference conflict.
5. Assigning the error to a `DuplicateKeyError` local and constructing an
   explicitly typed `Result<Unit, Error>`; the same diagnostic remained.

No compiler, semantic, or public API changes have been made in response. The
implementer stopped at the architecture boundary rather than changing the
approved contract or inventing another conversion mechanism.

## 8. Causal hypotheses

These are hypotheses only; current evidence does not distinguish them:

1. Generic variant-constructor inference does not use the expected
   `Result<Unit, Error>` type when checking the concrete `DuplicateKeyError`
   payload.
2. The compiler lacks a supported implicit subtype upcast from a concrete
   `Error` subclass to the generic `E` parameter in this expression context.
3. The canonical semantic product for the expected generic result type is not
   reaching this source-level constructor path.
4. The approved contract is representable, but the source needs a canonical
   explicit conversion or typed constructor form not yet identified.

## 9. Decision requested

Please decide:

1. Which layer owns the failure: generic variant-constructor inference,
   subclass-to-base conversion, or another semantic/compiler boundary? What
   canonical type/semantic product must that layer consume?
2. Is there an already-supported, source-level, typed construction pattern
   that preserves both concrete errors under the approved
   `Result<Unit, Error>` contract? If so, specify the minimal pattern and
   regression.
3. If no such pattern exists, should this become a separate compiler/semantic
   implementation plan while P6 remains blocked at T4, or is a P6 API
   amendment authorized? If an amendment is authorized, specify the exact
   replacement result/error contract and preserve both duplicate and invalid-
   index behavior.
4. What is the minimum discriminating regression set, including whether to
   add a generic `Result` test outside the collection module and how to prove
   the concrete `DuplicateKeyError` and `IndexError` values are preserved?

## 10. Current disposition

This section reflects the original pre-ruling state and is superseded by the
architect's decision and execution evidence below. P6 remains
`IN_PROGRESS / PARTIAL`; the approved native-floor inventory and independent
T3/T5 evidence are unchanged.

## 11. Architect ruling and implementation status — 2026-10-11

The ruling preserves the public `Result<Unit, Error>` contract and rejects a
P6 API amendment, P6-local inference rule, native workaround, Dynamic erasure,
or Result-specific special case. The primary owner is `phalcom-semantic`.
Canonical Universe source-authored superclass edges must be published into the
immutable hierarchy; if D2 fails after ordinary subclass widening passes,
generic variant construction must receive expected-result constraints through
the existing inference graph before payload analysis.

The discriminator established that D1 passes and D2 failed before the
correction. The implemented change shares source superclass resolution between
workspace hierarchy queries and canonical baseline construction, and seeds
expected-result evidence for variant constructors before payload analysis.
The source census found 68 explicit Universe `is` clauses and no applied
generic superclass forms; generic child edges continue to use
`GenericSupertypeTemplate`. No new SEMA plan identifier was created. The C1
checkpoint records this as an external semantic dependency closure and notes
the available historical SEMA001 C9/C16 references.

### Focused evidence

| Gate | Result |
|---|---|
| `canonical_universe_source_hierarchy` | 5 passed: ordinary widening, contextual `Result<Unit, Base>` construction, both canonical error-subclass relations, concrete `DuplicateKeyError` payload, unrelated-class rejection |
| `generic_application` | 15 passed |
| P6 combined callback/snapshot/GC source test | Compiled and executed; `duplicateErrorPreserved`, `indexErrorPreserved`, and `dictCollision` were true. Aggregate failed because `orderedCollision` and `mutableKeyPolicy` were false. |

The combined P6 test is not a G4 pass. The two new runtime checks establish
that the duplicate and invalid-index branches preserve concrete error classes,
but they do not yet establish that either failed insertion leaves dictionary
size/order/indexes unchanged. Key/positional coherence after mutation and the
full G4 gate remain open. The OrderedSet and mutable-key-policy failures were
not investigated further after the user requested a stop.

### Lifecycle and next action

P6 T4 may resume without another architectural consultation, but G4 must remain
unpassed until a focused gate establishes both error-path state preservation
and key/positional coherence. Keep P6 `IN_PROGRESS / PARTIAL`; do not create a
P6 walkthrough or handoff yet. P7 remains gated on P6 closure and a stable
handoff. P5 G6.B/aggregate G6 and the P4 D1/D2/D4 residuals remain separate
from this semantic correction.
