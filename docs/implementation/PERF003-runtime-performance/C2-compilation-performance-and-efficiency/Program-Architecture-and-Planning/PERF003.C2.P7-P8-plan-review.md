---
id: PERF003.C2.P7-P8.PLAN-REVIEW
kind: inline-plan-review
status: COMPLETE
implementation_verification: UNVERIFIED
prepared_against_revision: 3ca30fca
prepared_against_working_tree: true
---

# P7/P8 execution-plan review

This is an inline self-review of planning documents. No independent checker agent ran; no compiler, test, runtime or performance command ran in this planning pass. Textual XML task parsing confirmed all 11 P7 and 10 P8 contracts contain name, dependencies, first reads, file ownership, action, acceptance, feedback and done fields. These plans are repository-native, not registered numeric GSD phases.

## Coverage disposition

| User requirement | Execution owner |
|---|---|
| Preserve existing P7 contract and dirty work | P7 companion contract, T15; checkpoint/handoff routing |
| Provider-free completed module authority | Inherited T3/T4; integrated through P7.T11/T12 |
| Type/static graph relocation and stable identities | P7.T0/T5, T7.A, T6 |
| Exhaustive semantic persistence and source attachments | P7.T0/T5 manifest and T7.B |
| P6 indexes without relinking/analysis/witness rebuild | P7.T8 |
| Sole query eligibility and immutable reusable reads | P7.T9 |
| Existing RuntimePublic metadata once | P7.T10 |
| BLAKE3, deterministic encoding, bounded validation, atomic writes | Inherited container plus P7.T0/T5, T10–T12; P8.T1–T3 |
| Immutable LoadedUniverseArtifact and compiler injection | P7.T12 |
| Existing emitter in isolated build VM | P8.T2 |
| Explicit instructions/constants/upvalues/semantic recipes/native ABI/profile | P8.T1–T3 |
| Fresh allocation and sharing, GC rooting, empty caches | P8.T4 |
| Source-free native bindings and primordial/class identity/order | P8.T5/T6 |
| Eager and later-imported Universe code | P8.T2/T6/T7 |
| Explicit build/run commands and same loaded image in compiler/VM | P8.T8 |
| Logical diagnostic locations with no text dependency | P8 payload/T1/T7 |
| Fresh-process denial of embedded/provider access and zero compilation counts | P7.T13/T14; P8.T9, deferred |
| Runtime parity, two-VM independence, corrupt-load retry | P8.T9, deferred |
| Different physical roots/allocation histories yield identical bytes | P7.T0/T5/T11/T14; P8.T2/T9, deferred |
| Separate load/validate/relocate/allocate/initialize measurements | P7.T13/T14; P8.T8/T9, deferred |
| Separate lifecycle records and truthful closure | P7.T15; P8.T10 |
| P9 automatic policy deferred | Both contracts and P8.T8/T10 |

## Revisions applied during review

1. **Gate-number collision corrected.** The first draft used companion P7.G labels. The final companion uses the original G7.0–G7.11 identifiers and their exact objectives.
2. **BUILD and acceptance separated.** The first draft made P8 implementation wait for P7 behavioral proof even under the user's no-test instruction. Final P8 BUILD depends on complete P7 APIs/targeted compilation; behavioral proof still controls P7/P8 COMPLETE.
3. **Canonical numbering made concrete.** Shared export assigns per-arena IDs using deterministic traversal of stable-keyed required roots and ordered child edges. Native IDs are private memo keys only; unreachable history is omitted, referenced historical versions remain distinct.
4. **Static ADT dependency ordered correctly.** Type-only DataInfo/VariantInfo and constructors precede ordinary static-value factory validation, with private pending refs and final completeness checks for genuine cycles.
5. **Query omission does not erase dependency evidence.** ParsedModule reusable roots/dependents are pruned, while aggregate-backed invalidation evidence is restored separately and never promoted into reusable query eligibility.
6. **Metadata validation budget moved before allocation.** Existing metadata authority/decoder is reused, with bounded preflight if collection allocation occurs before existing validation; all sections charge one artifact allocation budget.
7. **No initialized class contents copied.** Fresh allocation recipes contain construction-time state only; fields/methods/globals installed by bytecode must not also be copied from initialized build objects.
8. **Current route reconciled.** Stale checkpoint/handoff next actions were replaced; historical predecessor evidence and documentation moves remain preserved.

## Preconditions and residual uncertainty

- The first P7 work unit must close the field/reader manifest. Required compiler source attachments currently include checked static products whose roots retain syntax. Exact source-independent checked operational-template support is a VERIFY-FIRST contract, not an assertion that transport already exists. If it requires a material evaluator/semantic representation change, stop the affected work and use the existing consultation protocol with exact consumers and owner evidence. Other independently valid tasks may proceed. Never omit the family or reclassify it to make publication complete.
- The isolated emitter's full constant/object census and build-time initialization prerequisites must be inspected before freezing executable adapters. Typed unsupported-projection errors are a development boundary; a final Universe build cannot pass with required modules omitted.
- Opcode and rooting rules must be grounded in current VM owners. Compiler checks alone do not prove hostile-byte safety, semantic parity or GC correctness.
- The latest core/LSP compiler result is inherited from the handoff (5.30 seconds), not acquired here. The ten historical container passes do not apply to all subsequent changes.

## Review result and next action

The plans provide an ordered, owner-scoped BUILD route and explicit stop conditions for unresolved representation contracts. P7 implementation remains IN_PROGRESS/PARTIAL/UNVERIFIED; P8 is PLANNED/NOT_STARTED/UNVERIFIED. No fresh-process or runtime acceptance is claimed.

Next implementation work is P7.T0/T5: establish the exhaustive persistence/consumer manifest and shared guarded deterministic type projection context; then T7.A structural products, T6 static values, and remaining T7–T15. P8 follows implemented P7 interfaces. This review authorizes no automatic execution and changes no existing test-execution policy.
