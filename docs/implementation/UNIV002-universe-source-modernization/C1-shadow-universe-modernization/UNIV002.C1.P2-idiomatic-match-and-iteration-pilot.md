---
id: UNIV002.C1.P2
program: UNIV002
checkpoint: UNIV002.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C1.P2 — idiomatic matching and iteration pilot

## Objective

Modernize five bounded shadow-source functions after the P1 inventory confirms
that their modules are not under active ownership migration. Preserve their
public signatures, module ownership, evaluation counts, iteration behavior,
ordering, and errors.

This is Category B behavior-preserving language modernization, with only local
Category A documentation/formatting edits. It does not change types, public
signatures, class/data representation, module paths, runtime registration, or
production source.

## Fixed scope

- `packages/universe/src/functional/chain.ph`
  - `Chain<T>.fromIterable`
  - `Chain<T>.partition`
- `packages/universe/src/collections/list.ph`
  - private `List<T>.sliceByRange`
- `packages/universe/src/collections/array.ph`
  - `Array<T>.slice`
- `packages/universe/src/collections/vector.ph`
  - `Vector<T, N>.slice`

Replace eligible `step.isSome`/manual `match` iterator consumption in the two
Chain functions with `while let` over the existing single iterator instance.
Replace the callback-based `Result.match` calls in the three range-slice
functions with exhaustive native `match` expressions. Keep the same
`rawSliceBounds` call, branch-local work, and error raising behavior.

## Required invariants

- `Iterable.iterator` is evaluated once per `fromIterable`/`partition` call.
- `iterator.next` is evaluated once per loop condition, including the final
  exhausted check; no extra `next` is introduced in the body.
- `fromIterable` consumes every item once and restores source order via its
  existing reverse step.
- `partition` calls the predicate once per item and preserves order in each
  output partition.
- Each slice function evaluates `rawSliceBounds(self.size)` once.
- Success preserves current normalized bounds and output element order;
  descending bounds still produce the current empty slice/list behavior.
- Error raises the same error exactly on the error case; success does not
  evaluate error handling.
- Public signatures and module ownership remain unchanged.

## Verification-first constraints and plan amendment

- P1 is reviewed and complete. Confirm the five functions remain outside active
  source ownership changes before editing.
- The focused validator investigation found no current test path that links or
  executes these five functions in the complete shadow Universe. The existing
  S4 validator parses shadow files and builds unlinked interfaces only; it does
  not typecheck bodies or prove exact match exhaustiveness. Synthetic semantic
  fixtures and production-Universe runtime tests do not validate these shadow
  call sites.
- Proceed as a bounded source-level pilot using the canonical `Option` and
  `Result` declarations/specifications plus causal review of the before/after
  code. Run the exact shadow parse/unlinked-interface check only if no shared
  Cargo process is active. This is a verification-plan amendment; it does not
  relax behavior, identity, evaluation, or API invariants.
- Do not report semantic linking, exact-site typechecking, or runtime behavior
  as verified. Record the source as `SOURCE_READY`; add `PARSE_VERIFIED` only if
  the focused shadow validator completes successfully. Semantic verification
  remains unavailable, and runtime parity remains `BLOCKED` until a supported
  shadow semantic and execution path exists.

## Tasks

### T1 — modernize Chain iterator consumption

Use `while let Option::Some(value) = iterator.next` while keeping the original
iterator value. Remove only the temporary Option-state binding and redundant
exhaustion branch. Document the one-pass/order contract where useful.

### T2 — modernize Result handling in sequence slicing

Use native `match` on the concrete `Result` returned by
`Range.rawSliceBounds`. Preserve success body, error propagation, and return
types in List, Array, and Vector.

### T3 — review the scoped diff and validate source

Confirm there are no public signature, import, export, or module-ownership
changes. Run the narrow existing shadow-source parse/unlinked-interface test
when no shared Cargo process is active; otherwise record the check as deferred
and do not run a competing Cargo build. The result establishes syntax and
unlinked interface shape only.

## Acceptance gate G1

- All five functions use the intended current syntax.
- The behavior invariants above hold by a documented source-level causal
  review grounded in the canonical `Option`/`Result` definitions and existing
  source operations.
- The shadow-source parser/unlinked-interface check passes if run and selects
  the intended test; otherwise it is reported incomplete/deferred.
- `git diff --check` passes.
- No production Universe, compiler, runtime, or native files are modified.
- Semantic linking and exact-site typechecking remain unverified. Runtime
  execution remains explicitly blocked unless actual shadow execution evidence
  is produced.

## Completion evidence

The source changes and evidence are recorded in
`packages/universe/docs/modernization/VALIDATION-LEDGER.md`, the `UNIV002.C1`
checkpoint, and the P2 walkthrough/handoff. The parser/interface check passed;
semantic and runtime gates remain explicitly unverified/blocked.

## Plan amendment record

The original verify-first premise that a focused validator could establish the
exact shadow match sites was disproved by a bounded repository investigation.
The available shadow check is parse/unlinked-interface only; semantic fixture
helpers analyze synthetic single modules, and production runtime fixtures load
the production Universe. P2 therefore proceeds with source-level equivalence
review and optional parse/interface validation, while explicitly withholding
semantic and runtime claims. No behavior or API acceptance criterion changed.
