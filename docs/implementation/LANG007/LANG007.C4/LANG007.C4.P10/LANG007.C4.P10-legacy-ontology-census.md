# LANG007.C4.P10 — Legacy Ontology Census

**Audit date:** 2026-10-05

**Scope:** P1–P3 shipped semantics, consolidated specifications, LSP/REPL presentation, and current Pack/type-domain names.
**Authority:** `docs/specs/README.md`; each topic owner listed below.

This census separates canonical kinded domains and internal representations from
the public unordered-reflection vocabulary that P9 will finish migrating. It is
not a blanket replacement list.

## Findings

| Symbol / term | Path | Usage | Category | Current semantic meaning | P9 action |
|---|---|---|---|---|---|
| `TypePack` | `docs/specs/type-level-domains.md`; `phalcom-core/core/universe/src/collections/tuple.ph`; `phalcom-semantic/src/types/` | Kinded positional sequence of proper type forms, including generic capture and type-level operations | A | Canonical C3 kinded domain, distinct from first-class proper Pack types | keep |
| `LabeledTypePack` | Same paths as `TypePack` | Kinded ordered `(Symbol, Type)` lane and generic capture | A | Canonical C3 kinded domain; not unordered Record reflection | keep |
| `CompleteTypePack` | Same paths as `TypePack`; `phalcom-core/core/universe/src/collections/pack.ph` | Kinded complete two-lane sequence and Tuple/Pack generic anchor | A | Canonical C3 kinded domain, distinct from `CompletePack<T>` | keep |
| `PositionalPack<T>`, `LabeledPack<T>`, `CompletePack<T>` | `phalcom-core/core/universe/src/collections/pack.ph`; `docs/specs/type-level-domains.md` | Proper Pack specializations and element-typed rest/view results | A | Current public first-class Pack aliases; they do not replace the `*TypePack` kinded domains | keep |
| `RecordFields` | `phalcom-core/core/universe/src/collections/record.ph`; `phalcom-semantic/src/types/type_member.rs`; `docs/specs/type-level-domains.md` | Exported context-specific kinded binder domain for `Record<**Fs>`; also names the result domain of the legacy `Record::Fields` member | A | Valid Record-context kinded domain. It is not the public unordered-reflection ontology; ordered Tuple/Pack binders use their own domains | keep the binder domain; migrate reflection uses separately |
| `RecordRow` / `RecordRowId` | `phalcom-semantic/src/types/row.rs`, `types/id.rs`, `types/row_solver.rs`, and metadata export | Canonical row representation, solver state, stable internal identity, and metadata form | F | Internal unordered-row mechanism; it does not define the source-facing reflection model | keep internally |
| `Tuple::Entries` / `TupleEntries` | `phalcom-core/core/universe/src/collections/tuple.ph`; `phalcom-semantic/src/types/type_member.rs` | Public Tuple associated type backed by a compiler intrinsic operation | A | Current ordered Tuple reflection returning the complete kinded entry pack | keep |
| `PositionalPackView<T>` / `LabeledPackView<T>` | `phalcom-core/core/universe/src/collections/pack.ph` | Public one-step Pack lane views with rest Pack values | A | Current Pack view API with Symbol labels and root-backed rest slices | keep |
| `PositionalTypePackView` / `LabeledTypePackView` | `phalcom-core/core/universe/src/collections/pack.ph` | Dynamic-valued aliases of the typed Pack view families | B | Compatibility/discovery facades; not aliases for kinded TypePack domains | investigate usage before any retirement |
| `StaticPackData` backing and `Object::PackSlice` | `phalcom-semantic/src/static_value/store.rs`; `phalcom-core/src/heap/pack.rs` | Store/runtime descriptors and root-backed slice representation | F | Internal O(1) views; repeated slicing composes offsets onto the original backing | keep internally |
| `Record::Fields` / `RecordFields::Map` | `phalcom-core/core/universe/src/collections/record.ph`; `phalcom-semantic/src/types/type_member.rs` | Existing public Record reflection/member facade over row terms | C | Legacy reflection surface. It must not become the final public unordered shape vocabulary | migrate presentation and retire the facade when P4/P9 replacement APIs are available |
| `generic_kind_completion_uses_source_kind_names_not_runtime_values` | `phalcom-lsp/tests/semantic_completion.rs` | Regression test for source kind names, runtime namespace separation, and hiding internal `RecordRow` | E | Intentional tooling regression coverage; `RecordFields` is offered as a source kind, not as a runtime value | keep; revise only if P9 changes the public binder surface |
| `UnorderedShape`, `FieldKey`, `FieldInfo`, `UnorderedTail` | `docs/specs/structural-products.md`; active P4 contract | Public unordered shape, key, field, and open-tail vocabulary | A | Target ontology for unordered reflection; P10 does not implement the P4-dependent shape | keep and complete through P4/P9 |
| `let`, `let mut`, `const` | `phalcom-lsp/src/hover.rs`; `phalcom-repl/src/highlighter.rs`; P1 walkthrough/handoff | Keyword hover, contextual modifier highlighting, and shipped binding semantics | A | `let` is immutable runtime binding; `let mut` is mutable runtime binding; `const` is static phase | keep |
| Rejected old binding claims, such as “use `let` for mutation” | P1 implementation plan’s explicit stale-semantics search/prohibition section | Negative search examples and migration guardrails, not current language claims | G | Historical text that explicitly labels these claims as obsolete | keep as migration evidence; do not present as current guidance |

## T7 — Specification and implementation-record reconciliation

The current P1 walkthrough and handoff agree on the binding phase/mutability
split. The P2 walkthrough records `const` as a checked static constant, with
runtime `let`/`let mut` excluded from static reads, budgeted evaluation, and
fail-closed structural materialization. The P3 walkthrough and handoff preserve
the kinded `TypePack` family while naming first-class proper aliases
`PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>`.

The audit found stale proper-alias spellings in
`docs/specs/generic-constraints.md` and `docs/specs/type-members.md`. Those
references now use the aliases shipped by Universe and owned by
`type-level-domains.md`. The generic-constraints summary also now distinguishes
the valid `RecordFields` binder kind from the public unordered-reflection
ontology. These edits synchronize summaries with the owning specifications and
current Universe source; they add no language behavior.

P3’s walkthrough previously described a Pack slice as able to reference a
prior slice. It now records the shipped invariant: slice-of-slice offsets are
composed and the resulting view points directly at the original root.

## T8 — Tooling presentation audit

The current LSP keyword hover describes the canonical `let`, `let mut`, and
`const` semantics. Kind completion offers source kind names and excludes
internal `RecordRow`; runtime-value completion excludes kind-only names.
Semantic tokens and REPL highlighting treat `mut` contextually, preserving an
identifier named `mut`. The TextMate grammar already has corresponding
context-sensitive patterns and its JSON parses successfully. No LSP, REPL, or
grammar source correction was indicated by this audit.
No obsolete P1–P3 LSP/REPL presentation entry requiring category D was found.

Focused evidence acquired:

- `phalcom-lsp` integration: generic kind completion, canonical binding hover,
  and contextual `mut` semantic-token tests passed.
- `phalcom-repl` `repl_session_intelligence`: contextual `mut` highlighting
  test passed.
- `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json`: JSON parse passed.
- `phalcom-semantic` Pack storage test: repeated 512-step alternating lane
  slicing retains the same backing and composed lane offsets.
- `phalcom-semantic` static evaluator resource test: increasing sequence,
  selected nested-if, nested static-application, early-return, budget, and
  in-evaluation cancellation discriminators passed.

P4 heterogeneous Record-key UX and P9 public reflection cutover remain outside
this audit’s implementation scope.
