# Phalcom Language Specification — Versions

The specification is versioned. `current/` is living snapshot; historical snapshots retain version names.

| Version | Status | Contents |
|---|---|---|
| **[current](current/README.md)** | **Current** | Consolidated language snapshot. |
| v0.1 | Historical | The pre-decision draft (superseded by v0.2; not retained as a separate tree — see git history before the v0.2 consolidation). |

## Where things go

- **Reading the current language:** start at [current/README.md](current/README.md).
- **Deferred / future work:** [current/deferred-work.md](current/deferred-work.md).
- **Core, numbers, standard library, syntax, and traceback:** grouped under [current/](current/README.md).
- **Decorator design:** [`../design/decorators/`](../design/decorators/).
- **Architecture decisions:** [`../adr/`](../adr/README.md) (frozen ADRs) and [`../pdr/`](../pdr/README.md) (active records).
- **Implementation work/specs:** [`../forge/units/`](../forge/units/).

## Version policy

- A version directory is **immutable for language semantics** once the next version
  opens. Typo/link fixes are fine; a semantic change means a new version.
- ADRs are the source of truth for *why*; a version's docs are the source of truth
  for *what the language is* at that version.
