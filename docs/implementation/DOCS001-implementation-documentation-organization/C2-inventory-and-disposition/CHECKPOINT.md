---
id: DOCS001.C2
program: DOCS001
kind: inventory
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# DOCS001.C2 — Inventory and disposition

## Acceptance objective

Produce a complete, reviewable disposition map for the existing implementation
and execution records before broad path migration begins.

## Baseline inventory captured before active migration

| Source root | Markdown files | Default disposition | Migration rule |
|---|---:|---|---|
| `docs/impl/` | 134 | NEEDS_REVIEW | Mixed plans, specifications, handoffs, state, and one legacy redirect; classify by file and work family. |
| `docs/work/pending/` | 125 | MIGRATE-ACTIVE | Map each bucket to a category; keep unresolved ownership explicit. |
| `docs/work/deferred/` | 14 | MIGRATE-ACTIVE | Keep under the owning program with `status: DEFERRED`; do not create a deferred root. |
| `docs/work/completed/` | 94 | MIGRATE-AS-BUILT | Preserve completion evidence and add canonical ownership metadata. |
| `docs/work/analyses/` | 47 | MOVE-TO-DESIGN | Research and analysis are not implementation plans. |
| `docs/work/logs/` | 25 | MOVE-TO-ARCHIVE | Retain important incidents through linked state records. |
| `docs/work/testing/` | 9 | NEEDS_REVIEW | Migrate durable verification programs to `TEST`; archive execution-only records. |
| `docs/superpowers/plans/` | 12 | NEEDS_REVIEW | Migrate durable plans; archive transient agent execution plans. |
| `docs/implementation/roadmap/` | 3 | RETAIN | Already canonical program-level roadmap material. |

## Plans

| Plan | Purpose | Status |
|---|---|---|
| [DOCS001.C2.P1](P1-inventory-and-disposition.md) | Create this manifest and identify review boundaries | IN_PROGRESS |

## Completion rule

This checkpoint is not complete until every source file has either an individual
manifest entry or is covered by a reviewed bucket rule whose exceptions are
listed explicitly.
