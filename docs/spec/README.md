# Phalcom Language Specification — Versions

The specification is **versioned**. Each version is a self-contained snapshot of
the language under `spec/v<N>/`; the version reflects the language as decided at
that point, with the resolved [open questions](v0.2/open-questions.md) woven in.

| Version | Status | Contents |
|---|---|---|
| **[v0.2](v0.2/README.md)** | **Current** | The consolidated, decision-complete snapshot: all 14 open questions resolved (ADR-0022/0024/0025/0027), numeric `Int`/`Float` split, `\(expr)` interpolation, external/internal parameter names, file-as-module imports. |
| v0.1 | Historical | The pre-decision draft (superseded by v0.2; not retained as a separate tree — see git history before the v0.2 consolidation). |

## Where things go

- **Reading the current language:** start at [v0.2/README.md](v0.2/README.md).
- **Making the next round of edits:** they are housed in a **new** `spec/v0.3/`
  directory (copy `v0.2/` forward, then edit), so `v0.2/` stays a frozen snapshot.
  Never edit a shipped version in place for a language change — cut a new version.
- **Deferred / future / still-open work:** [v0.2/deferred-work.md](v0.2/deferred-work.md).
- **Architecture decisions (ADRs):** live outside the version tree at
  [`../adr/`](../adr/README.md) — they are cross-version rationale.
- **Per-unit implementation specs:** [`../forge/units/`](../forge/units/) — one folder
  per unit (`U1/`, `U-CORE-1/`, …), each holding `as-built.md`.

## Version policy

- A version directory is **immutable for language semantics** once the next version
  opens. Typo/link fixes are fine; a semantic change means a new version.
- ADRs are the source of truth for *why*; a version's docs are the source of truth
  for *what the language is* at that version.
