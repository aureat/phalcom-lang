---
id: PERF003.C2.P7-P8.RESEARCH
status: COMPLETE
kind: planning-research
prepared_against_revision: 3ca30fca
prepared_against_working_tree: true
---

# P7/P8 remaining-work research and execution context

Date: 2026-10-14. Worktree: `.worktrees/perf-universe-characterization`; branch: `perf/universe-characterization`. The current uncommitted patch, including inherited documentation moves, is the implementation baseline. This record is planning evidence, not behavioral verification.

## Workflow adaptation

The explicitly invoked `gsd-plan-phase` research → planning → review process is applied to the repository's numbered P7/P8 records. This worktree has no `.planning/` roadmap. `CHECKPOINT.md` remains the durable state owner; no parallel roadmap, automatic GSD phase routing, automatic execution, or documentation commit is introduced. The skill's Codex spawn restriction selects inline roles because the user did not request subagents. The final review is an inline self-review, not an independent `gsd-plan-checker` result.

The existing detailed P7 plan is the implementation contract. Its task numbers and required ownership survive. The new P7 execution companion supersedes stale next-action notes and refines ordering; the new P8 plan supplies the previously missing patch-grade decomposition. These are repository execution documents with GSD task contracts, not plans registered with the GSD numeric-phase scheduler.

The approved scope supplies context: compiler plus executable Universe, BLAKE3-256, existing emitter in an isolated build VM, explicit CLI/API, default debug Universe contract profile, logical diagnostic locations, and P9 policy deferred. No context decision is silently defaulted. Compiler-only implementation feedback remains mandatory: no tests, runtime probes, benchmarking, or release certification until the user changes that instruction. Planning review does not execute any of these commands.

Tracer-first is represented by the complete source-free compiler milestone P7.T12 and the complete source-free VM milestone P8.T6. Their required transport dependencies cannot be replaced by mock products or a partly populated Universe. The original P7 task ordering takes precedence over the skill's generic requirement to put an end-to-end tracer before prerequisite schema tasks. Acceptance evidence for either tracer remains deferred under the compiler-only policy.

## Live implementation inventory

| Surface | Live state | Remaining obligation |
|---|---|---|
| P7.T0 predecessor census | P5 inventory exists, final wire support census absent | Enumerate every snapshot field, query kind, source attachment, and wire policy |
| P7.T1 container | Implemented; digest changed to BLAKE3, flag/schema guards added | Post-change malformed-input/determinism acceptance remains unrun |
| P7.T2 identity | Stable conversions and artifact provenance implemented | End-to-end cross-section identity and compatibility validation |
| P7.T3 external modules | `CanonicalExternalModuleWorld`, provider-free session/resolver paths implemented | Behavioral source-free import acceptance unrun |
| P7.T4 module world | `ModuleWorldV1`, project/load adapters and topology validation implemented | Integrate writer/loader; parity and hostile-input acceptance unrun |
| P7.T5 type graph | DTOs and fresh-store relocation implemented | Shared guarded export maps, deterministic graph numbering, primary-product cross-validation |
| P7.T6–T12 | Not implemented | Static values, primary products, indexes, queries, metadata, writer and loader |
| P7.T13–T14 | Not implemented/executed | Deferred source-free proof, corruption/parity/determinism/performance gates |
| P7.T15 | Interim records only | Final implementation/evidence disposition and normal completion records |
| P8 | Program-level outline only | Entire executable projection, validation, materialization, bootstrap and explicit workflow |

The handoff records the last compiler-only command as `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-lsp --lib`, completed in 5.30 seconds, with an unchanged unused LSP import warning. This is inherited evidence; no command was rerun for planning. The ten container test passes precede later container changes and do not certify the current patch.

## Source-grounded constraints

1. **Static loading has a real dependency on primary products.** `static_value/store.rs::intern_data` requires `DataInfo`; `intern_variant` requires `VariantInfo`. Both check exact owner/type/component identity. Load type-only declaration, constructor, data and variant products before static ADT values. P7's recommended order explicitly permits validated two-phase construction; publication stays atomic. Do not bypass factories to preserve the old illustrative ordering.
2. **Type export currently enumerates arenas.** `types/artifact.rs::ExportIds` is private and maps existing ordinals. T6/T7 need one shared export context with a `TypeStoreId` guard; import already supplies `TypeRelocation`. Stable outer row sorting alone does not prove allocation-history-independent bytes. Add canonical reachable graph ordering and preserve referenced historical parameter versions, current parameter selection, variance overrides, scoped union order/repetition, and within-graph sharing.
3. **Compiler attachments cannot be erased.** `CANONICAL_PERSISTENCE_INVENTORY` classifies `CompilerSourceAttachments` as required pending source-independent regeneration. `EffectiveCallableDefinition` carries accepted-definition provenance and generic body context. `StaticDefinitionProduct.checked_body` contains checked operations and facts but its root currently contains syntax; `StaticDefinitionSignature` also retains a syntactic where-clause. Inventory exact compiler/evaluator reads. Persist required checked operational templates and normalized constraints, rather than raw parsed modules or source text. If an evaluator still requires reparsing/rechecking dependency syntax, stop that affected task and consult; empty tables are not a substitute.
4. **Immutable query products and aggregate evidence differ.** `canonical_query_persistence` includes ParsedModule as structural but marks body/generic-conformance queries open-world and some aggregate-backed dependency keys unsupported. Omit ParsedModule roots and reverse-prune their reusable query dependents. Preserve separate aggregate-derived dependency evidence such as LinkedName/PublicExport/static-binding facts, so omission does not lose invalidation authority. Recompute fingerprints after relocation; do not promote an unsupported key into an eligible reusable query.
5. **Core's old identity is not wire compatibility.** `CanonicalCompilerImageView.compatibility` still includes process-local extent/lineage. The artifact crate's portable compatibility identity is distinct. File loading must compare against independently defined reader ABI constants and required capabilities, not build a source image for comparison or compare an artifact header with itself.
6. **Native-floor setup reads source today.** `modules/builtin_materialize.rs::initialize_canonical_universe` uses `UniverseSourceProvider`; `install_universe_native_bindings` iterates its nodes. Therefore `VM::new_native()` cannot be reused unchanged for a source-free consumer. Artifact setup must enter through the kernel and an authority-driven native installation path.
7. **Bootstrap contains several source-dependent steps.** `vm/bootstrap.rs` reserves source class bindings, prepares constants, refreshes primitive origins from `NativeSourceIndex`, installs metadata, compiles/executes initializers, installs source-owned primitive anchors, and establishes semantic roots. Replace all required source reads with portable records; preserve their ordering and primordial identity.
8. **Lazy imports need code too.** `interpret.rs::run_compiled` compiles a missing module closure from `CompiledModule.source_text`. Capture every canonical module initializer, including non-eager modules, and attach validated stored prototypes at module materialization. Explicit artifact mode must reject missing Universe code rather than use that fallback.
9. **Executable payload is larger than bytecode.** `chunk.rs::ExecutableSemanticPool` includes ADT/data/anonymous products, call-environment and associated-target recipes, family/conditional-family entries, trait invocations/requirements, and type-parameter receivers. Conditional family entries contain interned symbols and fallback-method constant indices. Every primary entry needs explicit transport; associated/data caches and pool indexes are reconstructed locally.
10. **Existing CLI check is insufficient proof.** `cli.rs::cmd_check` is a parser check. Add explicit compiler injection APIs; fresh-process semantic proof must compile novel user code through them, not rely on `phalcom check`.

## Build and acceptance separation

Implementation checks run serially at coherent owning-layer seams, using pinned toolchain and cleared flags/wrapper. Do not run compilation for these planning-only edits. P8 BUILD entry requires implemented P7 compiler APIs, while actual P7 source-free acceptance is retained as a closure requirement; deferred tests do not force an artificial stop in authorized code work. Plans list future test files and acceptance cases for coverage; they do not authorize executing them now. No fresh-process proof, two-VM independence, corruption rejection, determinism, GC stress, or performance outcome is currently established.

## Authority

- Existing P7 detailed implementation plan, §§16–25 and predecessor inventory task.
- `P7/PERF003.C2.P7-handoff.md` and walkthrough, including latest T4/T5 continuation.
- `Program-Architecture-and-Planning/PERF003.C2-P1-P9-universe-telosc-program-handoff.md`, P8 outline.
- `docs/specs/compiler-artifacts/telosc.md`, `docs/specs/README.md`, Universe and VM topic owners.
- `docs/workflow/implementation-record-lifecycle-convention.md`, principal architect doctrine, and patch-grade plan schema.
