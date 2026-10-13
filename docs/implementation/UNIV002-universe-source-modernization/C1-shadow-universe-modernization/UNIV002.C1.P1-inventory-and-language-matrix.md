---
id: UNIV002.C1.P1
program: UNIV002
checkpoint: UNIV002.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 18aaf8b568a0b45dd450befc35932bb7a1fdd442
---

# UNIV002.C1.P1 — shadow source inventory and language-feature baseline

## Objective

Produce the required MOD1 baseline for every `.ph` file in
`packages/universe/src/`, document language-feature evidence and source-style
rules, and select a low-risk modernization pilot. This plan creates evidence
and documentation only; source edits are deferred to P2.

## Fixed constraints

- Preserve production ownership and leave `phalcom-core/core/universe/**`
  untouched.
- Do not modify the active MODL001.P4 semantic-linking implementation or its
  uncommitted Rust files.
- Do not treat counts from textual scans as proof of semantic eligibility.
- Report uncertain parsing/classification rather than silently rewriting or
  classifying it.
- The cohesive `collections/iteration` source relocation is complete; do not
  split it or change module ownership in P1.
- Do not convert classes to data in P1.

## Mechanically flexible

- Inventory generation may use a scanner or parser already present in the
  repository. If it cannot obtain AST-aware counts for a field, mark its count
  as a textual indicator or `needs review` and state the limitation.
- Document organization may add helper files beneath
  `packages/universe/docs/modernization/`.

## Verify first

- Confirm branch, HEAD, and dirty-file scope before writing.
- Confirm the current source-file count and the S4 iteration/module ownership
  state against the live tree.
- Confirm exact accepted-language evidence for native `match`, `if let`,
  `while let`, `for` patterns/destructuring, and trailing closures.
- Confirm the shadow parser/interface validator's actual coverage before
  recording it as verification.

## Tasks

### T1 — capture current source baseline

Record branch, revision, dirty scope, and the exact set/count of shadow `.ph`
files. Record production/legacy status and known source-module conflicts.

### T2 — inventory every shadow `.ph` source

For each source file record: module, path, line count, public declarations,
module documentation, missing declaration documentation, `.match` calls and
receiver categories, `isSome`/`unwrap`, manual iterator loops, `if let`,
`while let`, `for`, closure style, compact control flow, unnecessary `mut`,
untypeable public APIs, candidate data types, native integration risk, and
status. Distinguish observed syntax from reviewed opportunities.

### T3 — establish supported-language evidence

Read the named specifications, parser/checker implementation, and executable
fixtures in the implementer packet. Record the exact distinction between
parser acceptance, semantic support, and executable-runtime coverage.

### T4 — establish guidelines and rank a pilot

Create the source-modernization guidelines and a feature matrix. Rank concrete
pilot functions that preserve public signatures and module ownership, avoiding
active structural migrations and any native/runtime changes.

## Acceptance gate G1

- Every live shadow `.ph` path appears exactly once in `INVENTORY.md`.
- Counts and classifier limits are stated and reproducible.
- Supported-language entries link to direct evidence and separate parse,
  semantic, and runtime confidence.
- Guidelines preserve the packet's category boundaries and do not imply that
  parser/interface checks prove behavior.
- A pilot proposal identifies exact files/functions and observable behavior
  that must remain unchanged.
- `git diff --check` passes for documentation changes.

## Required records

- `packages/universe/docs/modernization/GUIDELINES.md`
- `packages/universe/docs/modernization/INVENTORY.md`
- `packages/universe/docs/modernization/LANGUAGE-FEATURE-MATRIX.md`
- `packages/universe/docs/modernization/STATUS.md`
- Update the C1 checkpoint and create the P1 walkthrough/handoff at completion.

## Verification

No broad compiler/runtime test is required for this documentation-only plan.
Use direct parser/fixture evidence to populate the matrix; run no test suite.

## Completion evidence

- All four required modernization documents were created under
  `packages/universe/docs/modernization/`.
- An independent path/hash audit compared the inventory table against the live
  source tree: 174 live paths, 174 unique inventory rows, no missing/extra/
  duplicate paths, and zero SHA-256 mismatches.
- Reviewed the feature matrix, its direct parser/spec/fixture references, and
  the explicit limits on prior S4 parser/unlinked-interface evidence.
- `git diff --check` passed for the documentation changes.
- No test suite, parser, compiler, semantic, or runtime command was run.
- The first three `Result` slice sites remain a candidate only. P2 must first
  establish whether a focused shadow validator can resolve the exact match
  sites; do not infer eligibility from textual receiver labels.
