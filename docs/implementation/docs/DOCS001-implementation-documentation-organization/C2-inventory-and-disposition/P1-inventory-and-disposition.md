---
id: DOCS001.C2.P1
category: DOCS
program: DOCS001
checkpoint: DOCS001.C2
kind: inventory
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on: [DOCS001.C1.P1]
follows: DOCS001.C1.P1
supersedes: null
deferred_reason: null
---

# DOCS001.C2.P1 — inventory and disposition

## Purpose

Inventory the existing implementation-plan ecosystem and define safe
dispositions before moving additional documents.

## Scope

- `docs/impl/`
- `docs/work/pending/`
- `docs/work/deferred/`
- `docs/work/completed/`
- `docs/work/analyses/`
- `docs/work/logs/`
- `docs/work/testing/`
- `docs/superpowers/plans/`
- existing `docs/implementation/roadmap/`

## Required evidence

- source-root counts;
- bucket-level disposition rules;
- list of ambiguous individual records;
- no migration based solely on filename;
- no source content changed by inventory.

## Next action

Review [MANIFEST.md](MANIFEST.md), resolve its explicit review sets, and only
then begin active-program migration.
