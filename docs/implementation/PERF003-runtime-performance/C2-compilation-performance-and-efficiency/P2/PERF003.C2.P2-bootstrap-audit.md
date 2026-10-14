# PERF003.C2.P2 — bootstrap/ownership audit

Starting revision: 820514801e0de0480ab8bd03cf676c1e0c137b5a. Worktree:
`.worktrees/perf-universe-characterization`; clean at entry. No source-ownership
or one-shot overlay cutover is part of this patch.

## Source-backed call graph

```
semantic::canonical_image::canonical_universe_formal_image
 -> canonical_baseline::canonical_universe_semantic_baseline
 -> CanonicalModuleProducts::capture(provider, foundation.linked)
 -> SemanticWorkspaceSession::from_baseline(foundation, workspace)
 -> session.update(complete canonical sources, full product request)
 -> update_candidate_with_budget_and_cancel_and_delta
 -> SemanticDb::commit_transaction
 -> returned update.snapshot
 -> committed query freeze
 -> successful image publication

core::build_canonical_universe_program
 -> NativeSourceIndex::build + verify_native_contracts (core authority)
 -> semantic::canonical_universe_formal_image
 -> retained final snapshot + linked module products
 -> existing executable-match validation
 -> existing compiler projection
```

The explicit foundation constructor does not call the image cache. Session
`last_snapshot` and `last_known_good` begin empty. Neither receives the image's
snapshot. `phalcom-semantic` depends on `phalcom-modules` and native metadata /
surface crates, not `phalcom-core`. The image constructor does not call runtime
or compiler projection. Reentry is detected before acquiring the initializer
mutex; failures leave the cache empty and retryable.

## Module prerequisite and provenance

P1 has no complete external-module image. The immutable module-owned
`CanonicalModuleProducts` is the bounded adapter permitted by P2 §6.2. It retains
all provider parsed units and provider-cached interfaces alongside the existing
foundation's linked program. No AST-to-interface reconstruction occurs in image
capture. Topology construction still materializes interface values once.

World identity includes exact module/text provenance, its deterministic source
digest and this running binary's semantic/module/native compatibility contract.
The builtin provider is sealed embedded source in the current binary. Overrides
never use the builtin cache. This is in-memory provenance, not a disk ABI.
The completed snapshot owns final types and static values; reader binding checks
all arena prefixes, lambda meaning, parameter variance and every static arena.

## Query policy

Export occurs after `session.update` returns and rejects an open transaction.
Only explicitly audited structural/positive families with a matching typed
payload and committing-revision validation may be retained. Missing, foreign,
stale or changed dependency observations reject transitively. Cycles reject
until independently certified. Body/proof/candidate-set and static-evaluation
families are deliberately ineligible, and advisory/presentation families remain
unsupported. All rejected keys/reasons are inspectable in the census. Snapshot
products/diagnostics are not removed by query eligibility decisions.

The read API borrows validated consumer stores and returns canonical origin;
`SemanticDb::query_state`, `local_query_state`, invalidation and discard remain
local. There is no automatic workspace read-through in this patch.

## Reference status and limits

The ordinary complete canonical publication currently retains internal semantic
incidents. The prior core acquisition tolerated these and separately rejected
native-signature mismatches and unproven executable matches. This patch retains
that contract, the incidents and diagnostics; it does not repair the analyzer or
claim incident-free semantic certification. The first focused run exposed an
extra incident-free builder check introduced by this patch; that check was
removed to match the plan's requirement to preserve tolerated reference status.
The accepted source/native and executable-match guards remain in core.

Build/check and parity results belong in the walkthrough, not this call-graph
audit. No broad consumer cutover is implemented or claimed.
