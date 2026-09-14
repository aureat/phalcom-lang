---
plan: LANG005.C3.P1
checkpoint: LANG005.C3
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C4.P1
---

# Walkthrough — LANG005.C3.P1 First-Class Traits and Abstract Trait Surfaces

## Result

Closed the C3 checkpoint with first-class, non-storage `trait` declarations and
abstract semantic contract surfaces. Trait declarations now have canonical
module declaration identity and generic headers; `TraitRef`,
`TraitRequirementId`, `TraitSurface`, and trait-owned callable identities are
published through the semantic snapshot. Bodyless members are requirements;
bodyful members retain the same requirement and add a trait-local default.

Defaults are checked once against the complete abstract trait `Self` surface.
Abstract calls retain semantic callable identity but no executable invocation
target. Traits do not create classes, fields, layouts, superclass edges,
runtime method installation, conformance registries, or VM dispatch scans.

## Implementation slices

- Shared behavior members now represent declaration-only index bodies through
  `MemberBody`, while ordinary class and impl index bodies remain executable.
- The lexer/parser/AST and module declaration table recognize `trait` as its
  own declaration category, including generic binders, `where` constraints,
  methods, getters, setters, and index requirements.
- Semantic products provide DB-owned `TraitHeader` and `TraitSurface` queries,
  canonical `TraitRef` formation, stable requirement/source identity, complete
  signature publication before default analysis, and trait-specific dependency
  edges.
- Default checking uses abstract owner-relative `Self`, terminal trait-surface
  lookup, canonical argument checking, and early rejection of storage and
  `super` access. External nominal dependencies remain replayable; bootstrap
  Universe declaration shells are filtered at capture time.
- Incremental invalidation and source indexing preserve canonical declaration
  and callable targets. Compiler/product visitors accept traits as
  compile-time/type-level declarations only.
- The effective specification is now canonicalized in
  `docs/spec/extensions/traits.md`; protocol-era competing documents are
  explicitly historical or deferred, with a migration note and preserved
  history.

## Focused verification

All final filters selected nonzero tests.

| Surface | Result |
|---|---:|
| AST `trait_syntax` | 3 passed |
| Module declaration shell | 1 passed |
| Semantic trait capabilities | 13 passed |
| Incremental trait products | 3 passed |
| Semantic source index | 1 passed |
| Core trait compiler/runtime boundary | 2 passed |
| Core inherent-impl regression lane | 15 passed |
| Adjacent algebraic-data lane | 44 passed, 19 ignored |
| `phalcom-semantic` crate check | passed earlier in T9 |
| `cargo fmt --all -- --check` | passed |
| `git diff --check` and negative architecture searches | passed |

The 19 ignored algebraic-data tests are pre-existing gated/RED cases recorded
outside C3 scope. No A/B failure or blocking C failure remains. Full semantic,
LSP, workspace, and release certification was not run.

## Consultation

The T9 incident was sent to the advisory task and returned `Decision:
PROCEED`, with no plan amendment. The applied corrections were capture-time
Universe shell filtering, non-nominal default contexts, terminal active trait
`Self` lookup, early trait-`super` rejection, and preservation of generic and
external dependency edges.

## Remaining boundary

`C2-F03` applicability proof-state granularity remains a mandatory prerequisite
before C4 conformance or witness implementation. C3 does not claim explicit
conformance, witness selection, associated types, trait constraints, trait
objects, runtime trait descriptors, or trait-driven dispatch.

The shared worktree remains dirty with unrelated user/agent changes. This plan
did not clean, commit, or push those changes.
