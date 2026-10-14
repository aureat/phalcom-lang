# Canonical Universe generated-requirement cycle incident

Status: architectural decision required. Consolidation remains PARTIAL / UNVERIFIED.

## Scope and required invariant

Consolidation of integration `52838aa5`, main `a09de379` (already inherited), and
collections `60e6a3db`. Worktree: `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom`;
branch: `feat/consolidation-universe`. PERF and concurrent enum repairs are excluded.

MODL001.C2.P4 requires immutable finalized reads, every generated runtime owner
represented before finalization, and strict runtime DAG rejection. Compiler-side
late insertion and relaxed cycle validation are prohibited.

## Evidence and attempted correction

1. Conservative Universe wrapping failed `UncoveredRuntimeRead` for a root
   re-export to `errors.argument`. The owning linker was corrected to project
   canonical re-export owners before DAG validation. All 18 linker tests pass.
2. Runtime bootstrap then reached `callable.function`, whose `attempt()` reads
   prelude `Error`, and failed because the conservative wrapper had no generated
   read. The canonical builder did not pass symbolic linkage into semantic
   analysis and therefore omitted phase/generated products.
3. One serious correction applies existing staged APIs to canonical construction:
   `link_semantic` → fallible semantic analysis → generated requirements plus
   existing metadata owner projection → `finalize_executable_with_requirements`.
   Core and LSP checks pass. Exact runtime discriminator fails strict DAG validation.

Reproducer (pinned toolchain; clear local flags):

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
```

Completed result: 0 passed, 1 failed, 989 filtered, 140.05 seconds.

Reported cyclic initialization set:

```text
collections.bytes, collections.iterator_laws, collections.list,
collections.map, collections.range, collections.record, collections.set,
collections.tuple, errors.argument, errors.result, option.option,
scalar.bool, scalar.char, scalar.number, scalar.string,
scalar.string_index, text.builder, text.substring
```

This is a reported cycle set, not an independently established single SCC or
an edge-by-edge root-cause proof.

## Causal hypothesis and decision requested

Canonical generated declaration/reification reads now participate in the eager
source-module DAG. Some may be satisfied by already allocated native-floor
objects rather than source initialization, or the legacy source graph may need
structural migration. No bounded evidence yet distinguishes these alternatives.

Decide the canonical contract between native-floor availability, generated
runtime reads, metadata-owner requirements, and source initialization. If floor
satisfaction is valid, define the owning canonical product and exact proof for
that classification. If source cycles are real, amend the source architecture
and consolidation scope. Do not introduce a blanket Universe exemption.

Triggers: nonlocal compiler/runtime/bootstrap contract; materially different
architectural consequences between floor classification and source migration;
active P4 strict-DAG requirement conflicts with canonical source acceptance.
See `docs/workflow/shared-consultation-escalation-protocol.md`.

## Takeover

Inspect only:

- `phalcom-core/src/modules/canonical_universe.rs`: canonical staged candidate.
- `phalcom-core/src/modules/compile.rs`: shared runtime metadata owner collector.
- `phalcom-semantic/src/phase_evidence.rs`: generated requirement producers.
- `phalcom-modules/src/linker.rs`: strict executable finalization.
- `phalcom-core/src/vm/bootstrap.rs`: native-floor and source bootstrap boundary.
- MODL001.C2 checkpoint/ruling and this integration record.

The staged canonical candidate is saved for review; it is not accepted runtime
closure. No further corrective attempt or relaxed validator was applied.
