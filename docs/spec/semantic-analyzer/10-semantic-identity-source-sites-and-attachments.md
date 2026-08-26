# Phalcom Semantic Analyzer Specification
## 10 — Semantic Identity, Source Sites, and Attachments

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define semantic and source identities, their lifetimes, their stability boundaries, and the canonical attachments that connect source occurrences to semantic targets.

---

## 1. Identity is not location

Identity answers which semantic entity a fact concerns. Source location answers where a representation appears in one source revision. Revision identity answers which semantic world owns a fact.

These relations may coincide for simple declarations and diverge under edits, imports, generated declarations, specialization, recovery, and snapshot replacement.

The governing law is:

> Canonical identity must be carried when available. It must not be reconstructed from name, spelling, source range, or nearest-match search.

---

## 2. Identity domains

The analyzer distinguishes at least these conceptual domains:

| Domain | Representative identity | Meaning and lifetime |
|---|---|---|
| workspace | `WorkspaceId` | one persistent semantic workspace instance |
| semantic revision | `SemanticRevision` | one committed semantic world within a workspace |
| snapshot | `SnapshotId` | immutable publication identity, including required store/revision guards |
| project | `ProjectIdentity` | canonical project identity under module ownership rules |
| module | `ModuleId` | canonical module identity across semantic products |
| declaration | `DeclarationId` | canonical declared semantic entity |
| callable | `CallableId` | canonical owner, selector, and dispatch side |
| field | `FieldId` | canonical owner, name/member identity, and dispatch side |
| source owner | `SourceOwner` | namespace that owns snapshot-local source sites |
| source site | `SourceSiteId` | snapshot-local source occurrence or attachment site |
| external source reference | `SourceSiteRef` | source site guarded by owning snapshot |
| body | `BodyId` | snapshot-local callable or top-level body-analysis identity |
| binding | `BindingId` | snapshot-local local binding identity within body analysis |
| expression | `ExpressionId` | snapshot-local expression identity within body analysis |
| diagnostic cause | `DiagnosticCauseId` | snapshot-local owning contradiction identity |
| explanation | `ExplanationId` | snapshot-local explanation/provenance node identity |

Exact implementation names may differ. The domains, ownership, and non-aliasing laws must remain observable.

---

## 3. Stability matrix

| Identity | Range participates | Name participates | Stable across snapshot | Externally retainable |
|---|---:|---:|---:|---:|
| project/module/declaration/callable/field | no, except through canonical source identity inputs | only where language identity explicitly includes it | yes when canonical identity rules say entity survived | yes |
| `SourceSiteId` | no | no | no | only through snapshot guard |
| `BodyId` / `BindingId` / `ExpressionId` | no | no | no | only through owning published product/reference |
| diagnostic/explanation ID | no | no | no | only through owning snapshot/product |
| `SourceSiteRef` | carries snapshot guard, not range identity | no | rejects different snapshot | yes as guarded handle |

A source edit may move a site without changing its canonical target. Conversely, identical ranges or spellings do not make two targets identical.

---

## 4. Source sites and canonical targets

A source site describes a source occurrence owned by one immutable snapshot. A semantic target describes what that occurrence denotes when canonical resolution succeeds.

Conceptually:

```text
SourceSiteId -> SemanticTargetId

SemanticTargetId =
    Binding(declaration site)
  | Declaration(DeclarationId)
  | Callable(CallableId)
  | Field(FieldId)
  | Module(ModuleId)
```

Unresolved spelling is not a semantic target. Recovery may publish an unresolved occurrence with explanation/status, but it must not fabricate target identity.

---

## 5. Required attachments

The semantic snapshot must publish explicit canonical relationships where analysis establishes them:

```text
source binding declaration <-> body BindingId
formal ExpressionId        <-> SourceSiteId
source callable occurrence  -> CallableId
field/member occurrence     -> FieldId or canonical declaration target
import/module occurrence    -> ModuleId / DeclarationId
diagnostic-owning judgment  -> DiagnosticCauseId
semantic fact               -> ExplanationId, where explanation is published
```

Forward target lookup and reverse occurrence lookup must agree. Reverse lookup must use indexed canonical attachments, not whole-workspace name/range scans.

---

## 6. Binding and expression attachment

`BindingId` and `ExpressionId` are body-analysis identities. They are not source identities.

A binding attachment must distinguish shadowed declarations with identical names. An expression attachment must distinguish repeated identical syntax and ranges that moved between revisions.

If recovery makes attachment ambiguous, the analyzer publishes ambiguity/unmapped state. It must not select an arbitrary candidate.

---

## 7. Callable and constructor identity

`CallableId` identifies the public semantic callable through canonical owner, selector, and dispatch side. Body or execution representation is separate.

A constructor remains class-side in canonical callable identity even when its implementation body uses an instance receiver internally. Body mapping must be explicit. It must not cause a class-side miss to retry generic instance-side dispatch.

Selector identity comes from Phalcom message semantics. Inferred argument or receiver types do not redefine it.

---

## 8. Snapshot guards

Snapshot-local identities are valid only inside their owning snapshot. An externally carried source-site handle must include enough snapshot identity to reject stale reuse.

```text
SourceSiteRef(snapshot N, site K)
queried against snapshot N     -> may resolve
queried against snapshot N + 1 -> mismatch, even if local number K exists
```

Numeric coincidence is never remapping authority.

---

## 9. Identity creation and invalidation

Identity owners allocate and validate identities:

- module/project infrastructure owns project and module identity;
- semantic declaration products own declaration/callable/field identity;
- source-index construction owns source sites and attachments;
- body analysis owns body/binding/expression identities;
- diagnostic and explanation products own their local IDs;
- snapshot publication supplies revision guards.

Deletion removes the target and invalidates its reverse semantic closure. Rename follows the language/project identity rule: it either preserves an entity through an explicit canonical identity mapping or removes the old identity and creates a new one. Range/name similarity cannot decide.

---

## 10. Fingerprints and equality

Semantic fingerprints use canonical target identity where target choice is observable. Snapshot-local allocator numbering alone is not semantic identity.

Cause or source-site renumbering with equivalent semantic attachments must not force semantic inequality solely because raw local IDs differ. Normalization must preserve substantive target, causal, and explanation relationships.

---

## 11. Consumer contract

Consumers may:

- resolve exact sites and targets inside one pinned snapshot;
- retain canonical cross-revision target IDs where their identity contract permits;
- retain snapshot-local references only with owning snapshot guards;
- omit claims when attachment is unresolved or ambiguous.

Consumers must not:

- rebuild target identity from text/ranges;
- treat advisory target prediction as exact attachment;
- carry unguarded site/body/binding/expression IDs across snapshots;
- substitute URI-local or LSP-local IDs for canonical compiler IDs.

---

## 12. Conformance requirements

Tests must cover:

1. shadowed bindings attach to distinct canonical targets;
2. range-only edits do not redefine surviving target identity;
3. stale `SourceSiteRef` is rejected;
4. exact forward and reverse occurrence indexes agree;
5. unresolved/ambiguous sites do not acquire fabricated targets;
6. constructor identity remains class-side through body mapping;
7. deleted targets and reverse occurrences disappear together;
8. cold and incremental builds publish equivalent normalized attachments.
