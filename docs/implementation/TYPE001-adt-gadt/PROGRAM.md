---
id: TYPE001
category: TYPE
kind: completion-and-correction
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# TYPE001 — adt gadt

This program contains the ADT/GADT implementation, associated-family
resolution, pattern semantics, runtime representation, reflection, tooling,
testing, and remaining correctness remediation.

## Checkpoints

| Checkpoint | Acceptance area |
|---|---|
| `TYPE001.C1` | Surface syntax, parser AST, selectors, and family grammar |
| `TYPE001.C2` | Declaration identity and exact-case contracts |
| `TYPE001.C3` | Associated resolution and generic specialization |
| `TYPE001.C4` | Forward compatibility and family source design |
| `TYPE001.C5` | Runtime representation, reification, and lowering |
| `TYPE001.C6` | Match semantics, exhaustiveness, and pattern projection |
| `TYPE001.C7` | Core integration, reflection, and verification |
| `TYPE001.C8` | ADT/GADT/match test architecture |
| `TYPE001.C9` | Completion remediation and handoff |
| `TYPE001.C10` | Historical family LSP and selector-runtime records |
| `TYPE001.C11` | Historical immediate-Option runtime record |

The checkpoint names replace the former `part-*` and `testing/` labels. The
technical documents remain supporting records beside their implementation
plans; they are not additional phases.
