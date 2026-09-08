---
id: DOCS001.C1.P1
category: DOCS
program: DOCS001
checkpoint: DOCS001.C1
kind: migration
status: COMPLETE
completion: IMPLEMENTED
verification: UNVERIFIED
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---

# DOCS001.C1.P1 — canonical contract and corpus pilot

## Delivered

- Added the canonical implementation-documentation contract.
- Registered `DOCS001` as the documentation migration program.
- Registered `TEST001` as the language-corpus conformance program.
- Migrated the corpus remediation plan to `TEST001.C1.P1`.
- Added checkpoint and state metadata for the pilot.
- Retained the former path as a temporary redirect.
- Preserved the pre-existing `docs/.obsidian/workspace.json` change.

## Verification

- The migrated plan retained its original content with only canonical metadata
  added at the top.
- The legacy path points to the canonical plan.
- Scoped whitespace validation passed.

## Follow-up

The next plan must inventory all plan-like documents and assign each a
disposition before additional broad moves are made.
