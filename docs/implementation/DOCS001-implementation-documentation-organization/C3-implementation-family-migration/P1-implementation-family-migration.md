---
id: DOCS001.C3.P1
category: DOCS
program: DOCS001
checkpoint: DOCS001.C3
kind: migration
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on: [DOCS001.C1.P1]
follows: DOCS001.C1.P1
supersedes: null
deferred_reason: null
---

# DOCS001.C3.P1 — implementation family migration

## Scope

Move the coherent tracked families under `docs/impl/` into canonical program
directories and normalize every plan/spec filename and heading during the
relocation.

## Non-goals

- Do not retain nested historical directories or version/stage labels.
- Keep genuine technical specifications as named `*-spec.md` companions.
- Do not infer completion from a former directory name.
- Do not claim that links are repaired until the link audit completes.

## Program mapping

| Source family | Destination | Classification |
|---|---|---|
| `gadt/` | `type/TYPE001-adt-gadt/` | ADT/GADT completion and correction |
| `semantic/typing-integration/` and `typing-philosophy.md` | `type/TYPE002-typing-integration/` | Typing integration |
| `semantic/semantic-completeness/` | `sema/SEMA001`–`SEMA009` | Split by type formation, authority, workspace, capability, products, integration, and focused closure programs |
| `semantic/semantic-correctness/` | `sema/SEMA002`–`SEMA006` | Split by authority, products, integration, and verification ownership |
| `lsp/` and callable LSP review | `lspx/LSPX001-lsp-architecture/` | LSP/editor architecture |
| semantic analyzer invariants | `sema/SEMA003-semantic-analyzer-invariants/` | Semantic hardening |
| recursive pattern coverage | `sema/SEMA004-recursive-pattern-coverage/` | Semantic correction |
| print-family intrinsic | `univ/UNIV001-canonical-universe-surface/` | Universe surface design |

## Completion evidence required

- canonical program directories exist;
- substantive plan/spec content is preserved;
- program metadata exists for each destination;
- legacy root contains no implementation record;
- no plan/spec filename contains a version, staged-part, SC, handoff, or implementation-variant label;
- link and plan-metadata follow-up remains explicitly open.
