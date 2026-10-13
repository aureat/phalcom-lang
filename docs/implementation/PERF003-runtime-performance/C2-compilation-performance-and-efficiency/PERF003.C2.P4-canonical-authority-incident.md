# IMPLEMENTATION INCIDENT

## 1. Identity

- Program/checkpoint/plan: `PERF003` / `PERF003.C2` / `PERF003.C2.P4`
- Task/gate: T1 end-to-end options test; T6 canonical authority prerequisite
- Repository/branch: `phalcom` / `perf/universe-characterization`
- Starting/current committed revision: `a82dfe32ab744a16a7f24fce1cca85201372df19`
- Current increment: uncommitted compiler options plumbing, 2026-10-13

## 2. Escalation trigger

Repository mandatory trigger 2: a core plan premise is disproved by live
evidence. The canonical compiled RuntimePublic metadata authority required by
T6 cannot currently be built. This is an inherited D baseline, but now blocks
proof of the active P4 authority model. No permission to weaken the source-index
contract or absorb a separate bootstrap repair is inferred.

## 3. Required invariant

INV-P4-06 requires immutable external canonical metadata. T6 names
`canonical_universe_program().program().semantic_metadata` as its authority.
Local canonical record elision cannot be certified while that authority is
unavailable. INV-P4-08 forbids stripping required reflection records merely to
make export succeed.

## 4. Expected architecture

Canonical source index -> native contract verification -> canonical formal
image -> compiler projection -> canonical RuntimePublic metadata bundle.
Ordinary compilation then exports only its retained local closure.

## 5. Observed behavior

The existing exact canonical singleton test fails at `NativeSourceIndex::build`
before compiler projection, with a `SourceIndex` error rejecting `ProductKind`.
Independently, the new retained-session options test reaches ordinary metadata
export and fails on an unrepresentable non-Symbol Record remainder before its
first header assertion. These are distinct failure paths.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' cargo test -p phalcom-core --lib modules::canonical_universe::tests::canonical_universe_program_is_process_singleton -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --lib metadata_profile_is_explicit_and_independent_of_semantic_publication
```

Both select one test and fail. Canonical output:

```text
SourceIndex("intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration")
```

Session output:

```text
Io("metadata export error: legacy metadata cannot represent a Record remainder outside the Symbol key domain")
```

## 7. Relevant code path

- `modules/canonical_universe.rs::build_canonical_universe_program` first calls
  `NativeSourceIndex::build`; options are consumed only in later projection.
- `native/source.rs::index_program` sends every intrinsic runtime data declaration
  to `index_intrinsic_runtime_data`.
- `index_intrinsic_runtime_data` accepts only canonical Tuple/Record/Pack keys.
- `modules/compile_session.rs::compile_fixture_with_options` independently calls
  `ProgramCompiler::compile_analyzed_with_options`.
- Ordinary projection still calls aggregate `MetadataExporter::build_bundle`;
  T5 cutover has not been implemented in this increment.

## 8. Scoped implementation diff

Adds `ProgramCompilationOptions`, default RuntimePublic, explicit analyzed and
retained-session entry points, a single options path to the exporter, and
compiler/session regression tests. No bootstrap/export schema/runtime changes.

## 9. Evidence

Both baseline messages already appear in CHECKPOINT section 5. The unchanged
source-index guard also exists at the starting revision. The canonical failure
precedes every modified compiler function, so it is not caused by this increment.
The ordinary default export path retains the predecessor's profile and tables.

## 10. Implementer's causal hypothesis

The canonical source index's Tuple/Record/Pack-only intrinsic-data contract no
longer covers the authored ProductKind declaration. Ordinary aggregate export
independently reaches legacy-unrepresentable forms; reducing local reachability
may resolve that incident, but does not repair canonical source indexing.

## 11. Alternative hypotheses

The intended ownership/representation of ProductKind may instead require
reconciliation of authored source and native declarations. Adding it to an
allowlist without that ruling would assume the answer.

## 12. Attempts already made

No bootstrap correction attempted. The only production change is T1 plumbing.
No failing assertion, profile default, or accepted semantic rule was weakened.

## 13. Why local authority is insufficient

A ProductKind repair enters canonical source/native ownership outside the
planned metadata files. The repository protocol requires a ruling when the
active external-authority premise is unavailable; a known baseline becomes a
consultation topic only because it blocks P4's canonical/runtime proof.

## 14. Decision requested

Should P4 acquire a prerequisite, separately bounded canonical ProductKind
source-index/ownership repair, or continue only detached metadata work while a
separate repair restores the canonical compiled authority? Preserve the current
default profile and the pending end-to-end assertions in either case.
