# IMPLEMENTATION INCIDENT — MODL001.C2.P4 phase-aware source dependencies

## 1. Identity

- **Program:** MODL001 — module architecture
- **Checkpoint:** MODL001.C2 — module project model
- **Plan:** MODL001.C2.P4 — phase-aware source dependencies (blocked draft; not started)
- **Task/gate:** P4 preflight, especially T1/T2 and CV-01/CV-05/CV-07
- **Repository/worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections`
- **Branch:** `feat/telos-collections`
- **P4 plan's recorded repository basis:** `e5988ad80a04769740462de98c51d58b56bd314` (the plan's repository grounding; not the live branch head).
- **Inspected commit at incident preparation:** `da436a6c6b02c98f237a810b2e39f8d4fdc4cf58` (`HEAD`).
- **Working-tree state before delivery:** dirty with shadow Universe source/specification/inventory edits and three supplementary P4 analysis drafts (`C2P4-SLICE-1.md`, `C2P4-SLICE-2.md`, and `C2P4-SLICE-3.md`). The shadow paths do not modify the module linker, semantic analysis, compiler, or runtime evidence cited below. The supplementary drafts are committed in `48099fbc` and `bd2cbccc` as review inputs, not adopted architecture; the architect response attached by the user remains the decision authority. No P4 implementation was attempted.

## 2. Escalation trigger

This is a mandatory architecture escalation under triggers 1, 8, 11, and 16 in `docs/workflow/shared-consultation-escalation-protocol.md`:

1. The proposed P4 design requires semantic usage evidence that has no defined owner or recording path in the live tree.
2. P4 crosses module linking, semantic analysis, compiler lowering, and runtime materialization, beyond the shadow-source migration boundary.
3. Source phase, canonical semantic evidence, and executable runtime-link contracts affect identity, semantic authority, and runtime initialization.
4. The proposed plan, P2/P3 design documents, live code, and the initial Universe migration brief do not yet define one executable source-to-runtime contract.
5. The P4 front matter depends on and follows P2/P3, while the checkpoint still marks P2 and P3 proposed. The incident must resolve whether those predecessors must be completed first or whether P4's dependency/order should be amended before it can become executable work.

No production implementation was attempted. The current source linker’s eager behavior is established behavior; this report does not classify it as a production defect.

## 3. Required invariants

- A shadow import must resolve to the existing canonical declaration identity; it must not create a duplicate protocol, class, or Universe project identity.
- Interface analysis must retain every imported declaration needed to resolve types, constraints, and conformances.
- Runtime initialization remains a strict DAG. A runtime cycle must still fail before execution; no partial-module or tolerant-execution fallback is acceptable.
- Every runtime read emitted by lowering, including generated declaration references, must map to a materialized linked read and a runtime dependency.
- Unknown, unsupported, or incomplete phase evidence must preserve the runtime dependency.
- `packages/universe` remains an unregistered shadow tree. This incident does not authorize production source-provider, compiler, prelude, native, or bootstrap cutover.

## 4. Expected architecture and authority

The S4 incident defers the six-child iteration split until a supported interface-only dependency design preserves canonical identity and avoids eager runtime cycles. P2 proposes separate semantic/interface and runtime graphs, with runtime cycles rejected; P3 expects execution to consume a closed program with a validated initialization order. Those documents are proposed implementation designs, not normative source-import phase rules.

The original Universe migration brief says that compiler support needed for shadow verification should be recorded as an integration prerequisite rather than introduced into production during the initial shadow migration. Any P4 implementation therefore needs explicit architectural scope and authority before code work begins.

The current P4 draft proposes semantic-derived phase classification and a provisional/final link boundary. Those are candidate choices only; this incident asks the architect to accept, amend, or replace them.

## 5. Observed behavior

1. The source AST and `phalcom-modules::ImportSurface` represent ordinary module imports, selective imports, and re-exports. They do not carry an import phase.
2. `ModuleLinker::collect_imports_and_graphs` adds runtime edges for all three source import forms and creates linked reads/import bindings for imported names. `ReferenceKind::InterfaceOnly` and `DependencyPhase::InterfaceOnly` exist, but the live source-link path does not emit them.
3. Executable compile paths call strict `ModuleLinker::link_all` before semantic analysis. The linker computes the runtime initialization order during `LinkContext::build`, so a cycle is rejected before semantic analysis can prove any edge interface-only.
4. A tolerant component-link API exists for workspace/editor analysis, but it is not a finalized executable link product and does not provide the missing per-import phase evidence or runtime-read projection.
5. The semantic occurrence roles are `Declaration`, `Read`, `Write`, `Call`, and `Reference`; they do not encode “type-only use” versus “runtime value required.” `declaration_is_type_only` classifies a declaration’s static kind, not the phase of each use of an imported binding.
6. Runtime read layout and runtime materialization are coupled: the linker assigns `ImportBindingId` slots to `LinkedReadSpec` values, compiler lowering emits `GetLinked` for imported and generated declaration references, and core materialization resolves every linked-read slot.

## 6. Minimal reproduction

There is no supported source-level interface-only import fixture to reproduce. No tests were run for this report. The current tree therefore has no test that proves either (a) imported names remain symbolically resolvable before phase finalization or (b) a provisional link product cannot reach executable initialization.

Existing tests establish only neighboring behavior:

- `phalcom-modules/tests/graph.rs::reference_graph_preserves_kind_and_phase_separately` constructs an `InterfaceOnly` reference directly; it does not exercise source parsing through semantic analysis and executable linking.
- `phalcom-modules/tests/graph.rs::runtime_self_and_multi_node_cycles_are_rejected` tests the graph algorithm.
- `phalcom-core/tests/core/modules/linking.rs::comp_07_cycle_ab_detected_and_rejected` tests strict compile/link rejection of a runtime source cycle.
- `phalcom-modules/tests/linker.rs::link_01_selective_import_produces_binding` tests a normal selective import and linked read.

The P4 draft's T1 points to module-linker tests for strict runtime-cycle rejection. The strict source compile/link regression is in the core test above; the module graph test is the lower-level graph discriminator. T1 must add an assertion for symbolic resolution before phase finalization and an explicit fail-closed test for executable compile/bootstrap receiving a provisional product; these stage-boundary tests are acceptance evidence, not coverage to defer until the later semantic/runtime tasks.

## 7. Relevant code path

```text
phalcom-core/src/modules/compile.rs
  -> ModuleLinker::link_all
  -> LinkContext::build
  -> RuntimeDependencyGraph::initialization_order (strict cycle rejection)
  -> phalcom_semantic::try_analyze_workspace

phalcom-modules/src/linker.rs::collect_imports_and_graphs
  -> runtime edge + LinkedReadSpec + ImportBindingId for each import/re-export
  -> phalcom-core/src/compiler/lib/expr.rs emits Bytecode::GetLinked
  -> phalcom-core/src/modules/materialize.rs resolves linked-read slots
```

For a phase-aware design, semantic analysis needs canonical imported identities before the final executable graph exists; executable compilation must receive only the finalized runtime projection with a validated DAG. The live API boundary between those products is not defined.

## 8. Scoped changes and execution state

- No Rust, parser, linker, compiler, or runtime implementation was changed for this incident. The concurrent shadow Universe source moves are separate source-only work and are outside the P4 incident scope.
- A blocked P4 plan draft exists at `docs/implementation/MODL001-module-architecture/C2-module-project-model/MODL001.C2.P4-phase-aware-source-dependencies.md`. It must not be treated as an approved implementation contract until amended from the architect’s decision.
- No tests or compilation were run. The current documentation diff passes `git diff --check`; the earlier plan draft’s whitespace check is not behavioral evidence.
- The S3 Option/Result/Unit shadow moves and normative target paths are already present on this branch. They are not part of the unresolved phase-aware import design.

## 9. Direct evidence

- `phalcom-modules/src/linker.rs:278-286` — `link_all` builds the linked program through `LinkContext::build`.
- `phalcom-modules/src/linker.rs:580-634` — strict mode returns a runtime-cycle error while computing initialization order.
- `phalcom-core/src/modules/compile.rs:616-618,724-726` — executable compile paths link before semantic workspace analysis.
- `phalcom-modules/src/session.rs:2373` — workspace component analysis uses the tolerant interface-link API.
- `phalcom-modules/src/linker.rs:694-790` — import and re-export paths add runtime edges and linked reads.
- `phalcom-modules/src/linker.rs:1011-1015` — `InterfaceOnly` maps to that phase only when already present as a reference kind.
- `phalcom-semantic/src/source_index/occurrence.rs:28-36` — occurrence roles contain no type/value phase distinction.
- `phalcom-semantic/src/checker/context.rs:2730-2742` — `declaration_is_type_only` inspects canonical declaration kind.
- `phalcom-core/src/compiler/lib/expr.rs:335-365` — cross-module declaration references can emit `GetLinked`.
- `phalcom-core/src/modules/materialize.rs:220-260` — linked-read slots are materialized to module, binding, or static-constant runtime reads.
- `docs/implementation/MODL001-module-architecture/C2-module-project-model/P2-module-linking-graphs-and-lsp.md:1732-1740` — P2 names a runtime/interface phase-classification task but does not specify a complete semantic use-evidence product.

## 10. Causal hypothesis

The blocker is not the absence of an enum value. The system currently represents a single executable import surface: the same import creates the semantic name binding, a runtime dependency, a linked-read slot, and ultimately a materialized runtime read. The proposed P4 separates these responsibilities but does not yet define (a) how every semantic use of one exact imported binding is classified, or (b) how semantic linking can proceed before strict executable-cycle validation without allowing a provisional product to reach execution.

Until both contracts are explicit, an implementation could either reject the very cycle it is meant to resolve or remove a graph edge while leaving executable reads/materialization that still require the dependency.

## 11. Alternative architectures for the architect to evaluate

1. **Semantic-derived phase product:** canonical resolution records all uses per imported local binding and joins them conservatively. This needs a precise owner and coverage of type annotations, constraints, protocol conformances, class/superclass use, re-exports, value bodies, generated declaration references, generic/runtime type materialization, aliases, and incremental invalidation.
2. **Authored interface-only import form:** source syntax declares the intended phase, with a specified rule for mixed type/value use and diagnostics. This changes language syntax/specification and must be explicitly authorized as an integration capability.
3. **Keep the source modules cohesive:** defer the six-child iteration split while the shadow package continues. This preserves current runtime rules and canonical identities without requiring compiler changes now.
4. **Another representation or module design:** the architect may identify a design that preserves declaration identity and member behavior without introducing the predicted eager runtime cycle.

No option is selected in this incident.

## 12. Safe containment and recommendation

Do not implement the current P4 draft or reopen the six-child iteration split yet. Keep `universe.collections.iteration` cohesive and preserve strict runtime-cycle rejection. The P4 draft should be amended only after the architect chooses the phase model, the semantic evidence owner, and the provisional-to-final link boundary.

## 13. Decisions requested from the architect

1. Should interface-only status be authored in source or derived from canonical semantic use facts? If derived, identify the exact product owner and recording hooks that cover all semantic and compiler-generated uses.
2. What is the staged linker contract? Specify which product semantic analysis may consume, which product executable compilation may consume, when runtime cycle validation occurs, and how APIs make it impossible to execute a provisional product.
3. Which import forms may ever be interface-only? Decide treatment of whole-module imports, direct re-exports, aliases, unused imports, class/superclass references, protocol conformances, generic/runtime type materialization, and unsupported uses.
4. How must runtime import slots, `GetLinked` emission, static constants, declaration materialization, and runtime dependency edges be projected together so every emitted read is covered?
5. What incremental fingerprints/invalidation guarantee phase changes match cold analysis?
6. Is compiler/semantic/core implementation authorized as a separate integration increment, or should this branch record the prerequisite and continue only with source changes that preserve current linker behavior?
7. Correct P4’s strict-cycle test target and split its acceptance tests by owner: semantic phase proof, final runtime graph, linked-read/materialization coverage, and the S4 iteration integration fixture.
8. P4 depends on and follows P2/P3, but both remain proposed in the checkpoint. Must P2/P3 be implemented first, or should the dependency/order and scope be amended so the architectural prerequisite can be resolved without treating those proposed plans as completed contracts?
9. Require T1 tests proving pre-finalization symbolic import resolution and fail-closed rejection of provisional products by executable compile/bootstrap APIs. Confirm where each test belongs and keep phase classification, runtime-read projection, and Universe integration assertions in the later owner-specific gates.

## 14. Acceptance requirements for a revised plan

- Name a canonical per-import evidence owner and define the stable key, phase join, conservative default, and incremental invalidation.
- Cover every use site that can require a runtime object or linked read; declaration kind alone must not prove a use is interface-only.
- Define a provisional semantic-link product and a distinct finalized executable-link product, or an equivalent type-safe stage boundary.
- Add owner-specific tests proving symbolic import resolution before phase finalization and proving that provisional products cannot reach executable initialization; tests already present cover neither property.
- Preserve canonical `SymbolId`/`DeclarationId` identity across both products.
- Prove that every runtime read/materialization requirement has a corresponding runtime edge and slot; interface-only imports remain available to semantic resolution without requiring a runtime value slot.
- Demonstrate a source-level interface-only case, mixed type/value use, unknown-use fallback, semantic cycle acceptance, strict runtime-cycle rejection, and cold/incremental parity.
- Include a minimal iteration protocol/adapter fixture that proves identity and member contracts without modifying the Universe protocols or production bootstrap.
- Keep production Universe authority, native associations, prelude, and bootstrap unchanged unless a separate cutover is explicitly approved.

