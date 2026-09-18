Proposed root:

```text
docs/
  spec/
    README.md                 # authority rules, release/version, reading map
    foundations/              # values, object model, messages, classes, lookup
    syntax/                   # lexical, grammar, expressions, declarations
    semantics/                # blocks, control flow, errors, modules, iteration
    runtime/                  # bootstrap, memory model, execution/concurrency
    library/                  # numbers, collections, Result, System, stdlib APIs
    extensions/               # decorators, typing, reactivity; only ratified parts
    conformance/              # normative test and compatibility requirements

  decisions/
    accepted/
    proposed/
    retired/
    README.md                 # merged ADR/PDR registry; decision history, not spec

  design/
    proposals/                # active but unratified language work
    research/                 # theory, experiments, external technique notes
```

Authority rule:

- `docs/spec/` states one effective language rule. No alternatives, implementation progress, “currently uses,” work-unit links, or historical migration detail.
- `docs/decisions/` records why rule exists.
- `docs/implementation/` records HEAD reality.
- `docs/design/` holds proposals and research. It cannot override spec.
- `docs/archive/` holds retired, duplicate, superseded, and closed artifacts. Never delete provenance.

Key moves and merges:

| Current material | Destination | Action |
|---|---|---|
| `docs/spec/current` semantic chapters | `spec/foundations`, `syntax`, `semantics`, `runtime`, `library` | Re-home and remove implementation/status prose. |
| `docs/spec/numerical` | `spec/library/numbers` | Promote intact first. It already claims full authority. |
| `docs/spec/collections-next` | `spec/library/collections` and `spec/syntax/argument-expansion` | Promote only after conflict review against accepted decisions. |
| `docs/spec/current/decorators` plus canonical decorator material | `spec/extensions/decorators` | Synthesize ratified semantics. Move per-tier “built/not built” claims to implementation status. Archive retired `@construct` material after TDR-0073 migration audit. |
| `docs/spec/typing` | `design/proposals/typing` | Proposed normative design, not yet language authority. Promote sections only after ratification. |
| `docs/spec/collections`, most `next/`, `design/`, `design-notes/` | `design/proposals` or `design/research` | Keep live thinking; do not call it spec. |
| `current/core/*`, `forge/spec-status.md`, `deferred-work.md`, `stdlib/index.md` | `implementation/status`, `implementation/as-built`, `implementation/roadmap` | These are useful, but non-normative. |
| ADR + PDR trees | `decisions/` | Merge registries eventually; preserve status and supersession edges. |
| Forge phases, completed plans, logs, old numeric copies, scratch drafts | `archive/` | Preserve original relative path below dated archive root. |

Important cleanup findings:

- [`docs/spec/current/README.md`](/Users/altunhasanli/dev/phalcom/phalcom/docs/spec/current/README.md) claims authority but links to missing `numbers/`, `bitwise.md`, `stdlib/README.md`, `units/`, and `core/README.md`.
- [`docs/spec/current/decorators/index.md`](/Users/altunhasanli/dev/phalcom/phalcom/docs/spec/current/decorators/index.md) is largely an implementation report, while [`constructor.md`](/Users/altunhasanli/dev/phalcom/phalcom/docs/spec/current/decorators/constructor.md) is target semantics. Split them.
- [`docs/spec/typing/README.md`](/Users/altunhasanli/dev/phalcom/phalcom/docs/spec/typing/README.md) points at nonexistent `design/typing` paths while its actual files live locally.
- [`docs/spec/collections/`](/Users/altunhasanli/dev/phalcom/phalcom/docs/spec/collections) declares itself a draft candidate with provisional/open rules. Keep outside authority.
- [`docs/spec/numerical/README.md`](/Users/altunhasanli/dev/phalcom/phalcom/docs/spec/numerical/README.md) is strongest existing module. Use its hierarchy as root-model template.

Execution plan:

1. Write `docs/spec/README.md` charter: normative vocabulary, decision precedence, implementation separation, promotion and retirement rules.
2. Produce source-to-destination manifest. Every existing spec-like file gets one disposition: promote, merge, design, implementation, or archive.
3. Create canonical topic indexes. Migrate stable material with `git mv`; merge duplicate topic variants into one destination document.
4. Extract status, as-built evidence, work plans, open questions, and migration history from promoted specs.
5. Reconcile each promoted rule against accepted decision records. Conflict means block promotion; retain source under `design/` until ruled.
6. Move stale and superseded artifacts to dated archive paths. Leave a short archive manifest with replacement links.
7. Rewrite inbound links across `docs/`, code comments, and tests. Add a link check plus a “no `Proposed`/`Partially implemented` status banner under `docs/spec`” check.
8. Review each top-level spec module against one question: “Can language user implement correct program from this alone?” If no, complete it or keep it out of spec.

I would do this as small commits: charter/tree, numeric+core consolidation, syntax/semantics, extensions/library, then archive/link cleanup. No files changed yet.

---

```
decisions/
accepted/
proposed/
retired/
README.md                 # merged ADR/PDR registry; decision history, not spec

design/
proposals/                # active but unratified language work
research/                 # theory, experiments, external technique notes
```
- `docs/decisions/` records why rule exists.
- `docs/design/` holds proposals and research. It cannot override spec.
  | ADR + PDR trees | `decisions/` | Merge registries eventually; preserve status and supersession edges. |
