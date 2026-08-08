# Specification reorganization manifest

**Status:** Active migration record. Not normative.

**Scope frozen:** 2026-08-08. The repository has 177 Markdown files under
`docs/spec/`. This manifest assigns every specification source to a final role
before any content moves. It does not move, rewrite, ratify, or retire a source
by itself.

## Dispositions

| Mark | Meaning |
|---|---|
| **P** | Promote by `git mv`: source is already a coherent authoritative chapter. |
| **M** | Merge: extract and reconcile effective rules into the named canonical chapter; do not blindly move the source. |
| **D** | Keep as active design or research, outside the normative specification. |
| **I** | Keep as implementation, execution, status, or verification material, outside the normative specification. |
| **A** | Archive: stale, superseded, duplicate, closed, or legacy-only material. Preserve its path and replacement note. |

Promotion requires the gate in [`../../spec/README.md`](../../spec/README.md).
No path becomes authoritative merely because this manifest calls it a candidate.

## `docs/spec/` source map

| Source | Mark | Destination or action | Reason |
|---|:--:|---|---|
| `README.md` | P | `docs/spec/README.md` | Root governance charter; already landed. |
| `current/README.md` | A | `archive/spec-reorg-2026-08/spec/current/README.md` | Legacy index replaced by root topic indexes. |
| `current/{lexical-structure,string-interpolation}.md` | M | `spec/syntax/` | Canonical lexical and string chapters; remove migration and implementation status. |
| `current/syntax/*.md` | M | `spec/syntax/` | Reconcile grammar, lexical, expression, and declaration rules into one syntax module. |
| `current/{values-and-absence,object-model,messages-and-selectors,selectors,classes,method-lookup,functions,blocks}.md` | M | `spec/foundations/` | Core language semantics; split target rules from historical and implementation prose. |
| `current/{control-flow,iteration,destructuring,error-handling,result,modules}.md` | M | `spec/semantics/` and `spec/library/result.md` | Cross-cutting language behavior; each needs one topic owner. |
| `current/{concurrency,memory-management,performance}.md` | M | `spec/runtime/` | Retain only effective execution and memory rules. Move HEAD status and work ordering out. |
| `current/system.md` | M | `spec/library/system.md` | Public runtime-service contract. |
| `current/is-tests.md` | I | `implementation/as-built/is/` | Test and landing evidence, not language semantics. |
| `current/{open-questions,deferred-work}.md` | I | `implementation/roadmap/` | Decision closure and deferred-work ledgers. |
| `current/core/{bootstrap-phases,catalog-delta,floor-census,overview,pending-retirement}.md` | I | `implementation/as-built/core/` | Bootstrap phase, census, delta, and retirement evidence. |
| `current/core/{core-classes,collection-protocol,forward-compat,invariant-requirements}.md` | M | `spec/foundations/` or `spec/library/collections/` | Split canonical contract from as-built baseline and verification evidence. |
| `current/core/index.md` | A | `archive/spec-reorg-2026-08/spec/current/core/index.md` | Legacy index; replace with topic indexes. |
| `current/decorators/index.md` | M | `spec/extensions/decorators/README.md` + `implementation/as-built/decorators/` | Ratified tier model mixes directly with built/not-built claims. |
| `current/decorators/{accessors,data,ensures,ignore,invariant,native,on,requires,sealed}.md` | M | `spec/extensions/decorators/` | Reconcile effective decorator semantics; extract per-expander implementation evidence. |
| `current/decorators/constructor.md` | M | `spec/foundations/classes.md` + `spec/extensions/decorators/constructor.md` | Canonical constructor rule, reconciled with PDR-0028. |
| `current/decorators/construct.md` | A | `archive/spec-reorg-2026-08/spec/current/decorators/construct.md` | Legacy derivation surface after constructor migration; retain replacement link. |
| `current/stdlib/index.md` | I | `implementation/roadmap/stdlib.md` | Declares itself a program index, not a normative spec. |
| `current/stdlib/{bytes,cancellation,filesystem,map-and-set,net,process,reactor,stream-protocol,tuple-and-range}.md` | M | `spec/library/` | Promote only ratified contracts; leave proposals in design. |
| `current/traceback/{index,color,output-catalog}.md` | M | `spec/semantics/traceback/` + `implementation/roadmap/traceback.md` | Separate diagnostic contract from renderer design target and implementation plan. |
| `numerical/README.md` | P | `spec/library/numbers/README.md` | Existing authoritative numeric-module index. |
| `numerical/{numeric-tower,float-protocol,numeric-literals,text-and-errors,bitwise}.md` | P | `spec/library/numbers/` | Existing authoritative public numeric contract. |
| `numerical/conformance.md` | P | `spec/conformance/numbers.md` | Existing authoritative proof requirements. |
| `numerical/{implementation,migration}.md` | M | `implementation/as-built/numbers/` and `spec/library/numbers/compatibility.md` | Required implementation architecture and compatibility policy need separate homes. |
| `collections-next/*.md` | M | `spec/library/collections/` and `spec/syntax/argument-expansion.md` | Ratified design material, but overlapping sources need one reconciled collection contract. |
| `collections/**` | D | `design/proposals/collections/` | Explicit draft suite with provisional and open rules. |
| `typing/**` | D | `design/proposals/typing/` | Incremental proposed typing design; no promotion before ratification. |
| `next/**` | D | `design/proposals/next/` | Mixed provisional and draft design records, including documents labelled “normative design.” |
| `design/decorators/canonical/{mechanism,placement,interception,dispatch-tier,runtime-tier,behavioral,compiler-directives,metadata-and-docs,subtractive}.md` | M | `spec/extensions/decorators/` | Mine ratified effective rules; preserve proposals and source history separately. |
| `design/decorators/canonical/proposals.md` | D | `design/proposals/decorators/` | Explicit proposal material. |
| `design/decorators/canonical/{README,contracts,derives,effect,frameworks,frameworks-design,reactive,concurrency}.md` | D | `design/proposals/decorators/` | Remaining canonical-tree material contains proposals, framework design, or a design index. |
| `design/decorators/decorators-*.md` | D | `design/proposals/decorators/` | Non-canonical decorator proposals. |
| `design/{drafts,experimental}/**` | D | `design/proposals/` or `design/research/` | Draft and experimental material cannot coexist with canonical rules. |
| `design/{bound-callable-unification,bytes,fiber-ensure-and-limits,iteration-protocol,scheduler-unit}.md` | D | `design/proposals/` | Future or unresolved design work. |
| `design-notes/**` | D | `design/research/` | Technique and feasibility notes, not a language contract. |

## Specification sources outside `docs/spec/`

| Source | Mark | Destination or action | Reason |
|---|:--:|---|---|
| `docs/adr/**` and `docs/pdr/**` | M | `docs/decisions/{accepted,proposed,retired}/` | Durable decision history; not specification chapters. Merge indexes only after status reconciliation. |
| `docs/forge/spec-status.md`, `docs/forge/STATE.md`, `docs/forge/UNITS-TRACKER.md`, `docs/forge/DEFERRED.md`, `docs/forge/units/**` | I | `docs/implementation/{status,roadmap,as-built}/` | Execution state, unit plans, and implementation evidence. |
| `docs/forge/archive/**`, `docs/work/completed/**`, `docs/work/logs/**` | A | `docs/archive/{forge,work}/` | Closed-phase, completed, and historical records. |
| `docs/work/pending/**`, `docs/work/deferred/**`, `docs/work/testing/**` | I | `docs/implementation/roadmap/` and `docs/implementation/verification/` | Active plans and verification design. |
| `docs/work/analyses/**`, `docs/theory/**`, `docs/learn/**` | D | `docs/design/research/` | Analysis, theory, and learning material. |
| `docs/guide/**` | D | `docs/guide/` | User guide derived from spec; it is neither a decision record nor a specification source. |
| `docs/apps/**`, `docs/visualization/**` | D | `docs/projects/` | Product-specific documentation, outside language authority. |
| `docs/drafts.md` | A | `docs/archive/spec-reorg-2026-08/drafts.md` | Unstructured scratch material. |
| `docs/archive/**` | A | Preserve in place until archive normalization pass. | Already historical; do not churn before replacement references exist. |

## First migration batches

1. **Numeric module.** Promote the numerical public contract and conformance
   chapter. Split implementation and migration records. Archive duplicate numeric
   copies only after links point at `spec/library/numbers/`.
2. **Foundations and syntax.** Reconcile `current/` core semantics into topic
   modules. Move implementation baseline and status prose out in the same batch.
3. **Collections and extensions.** Reconcile only ratified collection and
   decorator rules. Keep typing and other proposals outside `spec/`.
4. **Decision and execution records.** Normalize `adr`/`pdr`, Forge, and work
   directories after their inbound links no longer point to legacy spec paths.
5. **Archive and link closure.** Move stale sources with manifests; fail the
   migration if any active document still names a legacy canonical source.

## Per-batch evidence

Each batch adds a short entry here containing:

- old path and final path;
- disposition actually applied;
- governing accepted decisions;
- links rewritten; and
- archived replacement note, if any.

This manifest moves to `docs/archive/spec-reorg-2026-08/` when the reorganization
is complete.

### 2026-08-08 — Numeric module

- `docs/spec/numerical/README.md` and its five public chapters moved to
  `docs/spec/library/numbers/`.
- `docs/spec/numerical/conformance.md` moved to
  `docs/spec/conformance/numbers.md`.
- `implementation.md` and `migration.md` moved to
  `docs/implementation/roadmap/` as non-normative landing records.
- Rewrote live numeric references in PDR, current-spec, work, and transition
  documents. Existing archive copies were intentionally left unchanged.
- Governing records retained: ADR-0024 and PDR-0012, PDR-0020, PDR-0026, and
  PDR-0027.
