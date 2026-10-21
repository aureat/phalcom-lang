---
id: PERF003.C2.P8
program: PERF003
checkpoint: PERF003.C2
phase: PERF003.C2.P8
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
prepared_against_revision: 3ca30fca
prepared_against_working_tree: true
depends_on: [PERF003.C2.P7]
autonomous: true
execution_mode: compiler-only-build
files_modified:
  - phalcom-compiler-artifact/src/executable/
  - phalcom-core/src/modules/canonical_artifact.rs
  - phalcom-core/src/artifact/
  - phalcom-core/src/vm/bootstrap.rs
  - phalcom-core/src/modules/builtin_materialize.rs
  - phalcom-core/src/modules/materialize.rs
  - phalcom-core/src/native/install.rs
  - phalcom-core/src/interpret.rs
  - phalcom-core/bin/phalcom/cli.rs
  - docs/specs/compiler-artifacts/telosc.md
must_haves:
  truths:
    - Every canonical module has a portable initializer prototype, including later imports.
    - Executable compatibility is checked independently of semantic compatibility.
    - Fresh VMs allocate independent runtime state while preserving canonical class identity and sharing within each VM.
    - Native/primordial setup and initializers preserve source-built installation and failure order.
    - Explicit CLI compilation and VM construction share the same loaded compiler authority.
  prohibitions:
    - statement: No ObjRef, ClassId, initialized globals, captured values, metadata pool numbers or mutable caches are serialized.
      status: resolved
      verification: backstop
    - statement: Explicit artifact mode must never fall back to Universe source compilation or provider access.
      status: resolved
      verification: backstop
    - statement: P9 automatic discovery, caching, distribution and machine-native compilation are outside this plan.
      status: resolved
      verification: explicit
---

# P8 portable executable Universe implementation plan

## Objective and entry gate

Deliver `universe.telosc` containing the completed compiler image plus portable VM-bytecode recipes. A fresh process must load it, compile novel user code and initialize fresh VMs without Universe parsing, analysis, lowering or bytecode compilation. Initializer execution remains per VM.

P7 closure and independent certification are accepted in the checkpoint record.
P8.G1 is PASS: the loaded compiler interface is present and the focused P8
compiler checks listed in the checkpoint pass. P8 is now active and remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; the capture/codec/publication slice is
not the executable validation, VM materialization, or source-free execution
acceptance objective. The owner-supplied closure handoff amends this plan in
place; do not create P9. T1/T2 are complete at the structural-staging boundary,
and T3 is the active partial implementation. The architect-planning handoff is
historical and superseded by the T3 continuation handoff linked from the
checkpoint. Do not infer runtime-test authorization from focused capture
tests or a successful library check.

Read the shared P7/P8 research, P7 execution companion, existing program P8 outline, `docs/specs/compiler-artifacts/telosc.md`, `universe.md`, `vm/bytecodes.md`, `vm/vm-execution-model.md`, and the heap/rooting owners reached by materialization. The old proposed unified program specification is not adopted wholesale.

## Architecture and immutable runtime boundary

Build: finalized canonical source program → existing emitter in isolated build VM → capture each initializer and nested callable before executing that initializer → portable recipes → validate → append required EXECUTABLE → atomic artifact publication.

Consume: bounded compiler and executable validation → immutable LoadedUniverseArtifact → user-only compilation → kernel → authority-driven native floor → fresh module/class/metadata allocations → relocated chunks/callables/closures with empty caches → eager initializer execution → retained prototypes for later imports → user execution.

`phalcom-compiler-artifact` owns executable DTOs/codec/structural bounds; core owns projection, semantic linking, bytecode checks and VM allocations. No second emitter or second semantic checker is introduced. All opcode tags are explicit stable wire tags and map exhaustively to current Bytecode variants. Semantic image compatibility is independent of executable ABI.

The executable header contains executable schema, portable bytecode ABI, emitter ABI, native ABI BLAKE3 digest, compile profile and any VM operand-width/frame contract version needed by decoding. Native ABI digest covers ordered stable binding keys, exact selector/side/arity/contract identity and required primordial roles, never function addresses. Universe uses the existing Debug contract profile; user release/unchecked/metadata flags preserve their current meaning.

## Portable payload

| Family | Required payload | Runtime reconstruction |
|---|---|---|
| Modules | Stable module ref, initializer prototype, linked reads, source location, bootstrap role/order | Fresh ModuleObject, ordered slots/exports, registry state and closure |
| Callables | Owner/nested ordinal, instructions, constants, max slots, arity/parameter shape, locals/names, upvalue descriptors, method contracts and required callable flags | Fresh Callable/chunk/closure and method wrapper objects |
| Constants | Exact immediate/scalar bits, symbol/string recipes, immutable literal construction, prototype refs, stable class/native/module/type refs | Fresh interner/heap allocations; shared recipe relocation |
| Classes | Stable declaration/primordial role, metaclass/superclass relationships, ordered field/layout and allocation role | Resolve primordial objects; allocate required source-owned shells once |
| Semantic pool | ADT/data/anonymous construction, invocation/environment/associated/family/trait/type-parameter recipes | Reuse validated compiler authority; fresh runtime descriptors/caches |
| Native anchors | Stable owner/selector/side/native key, origin and contract attachment | Resolve compiled-in primitive registry exactly |
| Diagnostics | Logical module URI, spans and line starts; no source text dependency | Fresh SourceId/diagnostic mapping |

Census every current Bytecode variant, Value/object constant family, Callable field, ExecutableSemanticPool field and native/bootstrap side effect. Known conditional-family entries include symbol relocation and fallback-method constant indices. Reject unsupported projection with typed owner/module/prototype/constant context; never dump heap objects or captured closure values.

Canonical numbering uses module path, stable callable owner plus deterministic nested ordinal, and recursively canonical recipe keys; preserve instruction/constant/field/parameter/bootstrap order. Extend the shared P7 export context for references reachable only from executable recipes. If emission creates legitimate build-local type recipes, represent them as explicit executable-local type constructions validated against the semantic image, never cast them into its TypeStore coordinates.

## Waves

| Wave | Tasks | Outcome |
|---|---|---|
| 1 | T1, T2 | Frozen executable census/schema and complete isolated-VM projection |
| 2 | T3, T4 | Fully validated executable image and GC-safe fresh materialization |
| 3 | T5, T6 | Source-free native setup and first complete VM tracer |
| 4 | T7, T8 | Later imports and explicit CLI/API end-to-end workflow |
| 5 | T9, T10 | Deferred behavioral proof and lifecycle closure |

T1–T5 are prerequisites to T6's production tracer; do not substitute a toy Universe or serializable heap snapshot. Cargo commands remain serial and sparse. No subagent use is assumed by these waves.

## Artifacts this phase produces

ExecutableImageV1/ExecutableCompatibilityV1; portable module/callable/chunk/instruction/constant/upvalue/native/class/source-map DTOs; validated executable wrapper with private constructor; core projection, validation and materialization modules; VmArtifactRelocation; source-free native/bootstrap helpers; stored Universe prototype registry; `VM::try_new_with_universe_artifact(Arc<LoadedUniverseArtifact>)`; extended build/load APIs; `artifact build-universe --output` subcommand; `--universe-artifact` option; updated normative executable wire tables; P8 walkthrough/handoff.

## P8.T1 — Freeze executable census, ABI and complete wire schema

<task id="P8.T1" type="auto" wave="1">
  <name>Freeze executable census, ABI and complete wire schema</name>
  <depends_on>P8.G1 complete P7 compiler interface and targeted compiler check</depends_on>
  <reversibility rating="one-way">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>bytecode.rs; callable.rs; chunk.rs; value/heap object types; native registry; telosc.md</read_first>
  <files>phalcom-compiler-artifact/src/executable/{mod,instruction,prototype,constant,recipes}.rs (new); limits.rs; docs/specs/compiler-artifacts/telosc.md</files>
  <action>Enumerate every opcode operand domain and every emitted object/semantic-pool/bootstrap product. Define explicit tagged DTOs and checked conversions, independent executable compatibility header and native ABI digest. Add executable-specific maximum prototypes, total instructions, constants, semantic entries, frame slots, upvalues and literal/source-map bytes under a cumulative load budget. Define class/native refs by stable identities, module init coverage and profile. Retain parameter shape, names/contracts/source spans and exact scalar encodings. No Rust enum discriminants, usize widths, runtime pool numbers or allocation coordinates. Approved portable schema decisions are retained; unapproved conflicting ABI changes require consultation before freeze.</action>
  <acceptance_criteria>Exhaustive compile-time matches and manifest rows cover all current executable inputs and validation operand classes; schema tables are precise.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-compiler-artifact --lib. No tests.</verify>
  <done>Exhaustive compile-time matches and manifest rows cover all current executable inputs and validation operand classes; schema tables are precise. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T2 — Capture canonical prototypes through the existing emitter

<task id="P8.T2" type="auto" wave="1">
  <name>Capture canonical prototypes through the existing emitter</name>
  <depends_on>P8.T1; source-built canonical program and P7 export context</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>vm/bootstrap.rs::compile_universe_module/run_universe_modules; compiler/lib; NativeSourceIndex; canonical_universe.rs</read_first>
  <files>phalcom-core/src/artifact/project.rs (new); modules/canonical_artifact.rs; narrow existing emitter hooks; vm/bootstrap.rs build-only helper</files>
  <action>Create an isolated build VM with existing source-built authority and normal native substrate. Compile modules with the current emitter, then recursively capture initializer/nested methods and callables immediately before that initializer executes. If later emission requires earlier initialization, execute earlier captured modules in the build VM only; never persist resulting mutable globals/captured values. Root capture graphs with existing VM temporary-root discipline. Compile every canonical module, including modules outside eager bootstrap; dependency scheduling for build capture must not alter stored runtime bootstrap order. Class objects become stable allocation/primordial recipes, callable objects become prototype refs, strings/symbols/literals become construction recipes. Preserve identity sharing by recipe maps, then discard the build VM. Unsupported constants or source-dependent legal products return typed build errors.</action>
  <acceptance_criteria>Every canonical module has validated portable code; none of the isolated VM heap/state escapes; capture precedes each initializer mutation.</acceptance_criteria>
  <verify>Compiler feedback after T2/T3 coherent seam: cargo check -p phalcom-core --lib with flags/wrapper cleared.</verify>
  <done>Every canonical module has validated portable code; none of the isolated VM heap/state escapes; capture precedes each initializer mutation. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T3 — Validate instructions and cross-section executable references

<task id="P8.T3" type="auto" wave="2">
  <name>Validate instructions and cross-section executable references</name>
  <depends_on>P8.T2 DTO projection; P7 loaded semantic/module authority</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>vm execution opcode consumers; bytecode.rs; chunk.rs operand methods; P7 validation context</read_first>
  <files>phalcom-core/src/artifact/validate.rs (new); artifact executable codec; modules/canonical_artifact.rs</files>
  <action>Before any initializer or artifact-specific native side effect, verify executable ABI/profile/native anchors, exact module coverage and initializer ownership, all instruction opcodes/operand widths, constant/prototype/recipe references and tags, branch targets, reachable control-flow/frame-slot safety, register ranges and call conventions. Check closure-site upvalues against parent locals/upvalues, matching parameter/arity shapes and semantic-pool variant indices. For ReturnNonLocal, classify every activation edge for each closure template from Closure and MethodRecipe::Closure sites; admit the opcode only when every reachable activation is block-only and carries the home-frame token. A template with block and method activation, an unknown route, or a conflicting route is rejected if its body uses ReturnNonLocal.

    Prove ordinary Method and VariantMethod installation identity from the bytecode graph without changing the executable schema. Run a bounded abstract dataflow over each prototype's CFG with stack and local-slot provenance for method values and class values. Seed facts only from validated producers (including closure/prototype references, native anchors, stable class/primordial references, and class construction recipes); propagate only through opcode transfers whose runtime semantics preserve identity. Model stack effects and branch edges explicitly. A join retains a fact only when every incoming reachable edge agrees on the same exact provenance; unknown, conflicting, unsupported, or out-of-range facts remain unknown. Writes, calls, returns, aliases or mutations that can change or obscure a tracked value invalidate affected facts unless the opcode's contract proves identity preservation. At each actual Bytecode::Method or Bytecode::VariantMethod instruction, read the method and class values from the instruction's verified operand/stack positions and require singleton provenance. Tie that evidence to the corresponding MethodRecipe/native anchor, exact selector and side, validated body prototype/signature, and the stable class recipe/owner actually consumed by the instruction. Reject unknown or conflicting installations before runtime; do not infer the installation holder or access owner from the callable owner. Apply the same operand-tied proof to variant installation and its variant/class authority. Do not accept a recipe merely because a matching selector, owner, or class exists elsewhere in the image.

    Validate class/super/metaclass roles, native selector/side/arity and anchor agreement, linked reads/export slots, checked type/static/metadata refs, ordered bootstrap coverage and diagnostics mappings. Reject unsafe cycles and arithmetic/length overflow under shared budgets.

    Preserve runtime object identity across the proof: `Bytecode::Class` allocates a fresh class on each execution, so its abstract provenance must include the allocation site and exact `SourceDeclared` class row. Reject a site unless it is unique for that row and proven not to execute repeatedly or alias another producer; loops, cyclic routes, and unsupported dynamic class producers fail closed. Track method object identity by canonical method recipe/constant because installation mutates holder/access-owner and closure anchors; reject a recipe reused at conflicting class/selector/side identities. Do not treat equal stable class references or equal method signatures as proof of equal runtime objects. Return a private ValidatedExecutableImage only after every proof succeeds; consumers cannot construct it directly.</action>

  <acceptance_criteria>Every operand interpretation is covered by a structural or control-flow rule; every reachable Method/VariantMethod install has singleton operand-tied method and exact runtime class/variant provenance; unknown, conflicting, repeated-allocation, and incompatible install identities fail closed; malformed/incompatible bytes cannot reach initializer execution.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-core --lib; hostile-byte execution deferred.</verify>
  <done>Every operand interpretation is covered by a structural or control-flow rule; every reachable method installation is proved against P7 identity and its actual bytecode operands; unsupported provenance fails closed; malformed/incompatible bytes cannot reach initializer execution. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T4 — Materialize fresh objects with explicit relocation and rooting

<task id="P8.T4" type="auto" wave="2">
  <name>Materialize fresh objects with explicit relocation and rooting</name>
  <depends_on>P8.T3 validated image</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>heap/rooting APIs; callable.rs; chunk.rs; module registry/materialize; ADT/data/runtime typing registries</read_first>
  <files>phalcom-core/src/artifact/materialize.rs (new); modules/materialize.rs; owning runtime registry constructors</files>
  <action>Allocate fresh symbols, ordered module slots/exports, class/metaclass shells, metadata pools, callable/chunk/method objects and closure shells through existing VM allocation APIs. Use a per-VM relocation map keyed only by artifact recipe/prototype/stable identity. Allocate declarations before wiring edges; resolve primordial roles to the VM kernel/native objects without shadow classes. Recipes contain only allocation-time fields: methods, fields or globals installed by initializer instructions must not also be copied from initialized build objects. Initialize every runtime inline/global/associated/data cache empty; rebuild local semantic-pool indexes. Top-level initializer closures have no persisted captured values; nested captures bind at ordinary runtime closure creation. Keep all in-progress objects rooted across allocations with existing guard discipline; publish module prototypes only after relocation succeeds. No shared mutable state, global relocation cache or new GC policy.</action>
  <acceptance_criteria>Fresh allocations preserve required within-VM sharing and canonical identity; relocation does not expose partial modules or stale build-VM refs.</acceptance_criteria>
  <verify>Batch with T5/T6: cargo check -p phalcom-core --lib with flags/wrapper cleared; GC/two-VM runtime checks deferred.</verify>
  <done>Fresh allocations preserve required within-VM sharing and canonical identity; relocation does not expose partial modules or stale build-VM refs. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T5 — Replace source-backed native module setup and binding anchors

<task id="P8.T5" type="auto" wave="3">
  <name>Replace source-backed native module setup and binding anchors</name>
  <depends_on>P8.T4; validated module authority/native recipes</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>builtin_materialize.rs; native/install.rs; bootstrap.rs::install_native_runtime/prepare_universe_source_bindings/prepare_universe_static_constants</read_first>
  <files>phalcom-core/src/modules/builtin_materialize.rs; native/install.rs; vm/bootstrap.rs</files>
  <action>Split native setup so artifact VMs enter from new_kernel and install the existing native substrate using loaded interfaces/topology and portable binding records. Do not call new_native unchanged: its builtin materialization uses UniverseSourceProvider. Replace provider node/interface/source-ID enumeration and NativeSourceIndex anchors with stable module/declaration/native records, including direct child-package exports, primordial aliases, None immediate/global behavior, sealing and field reservations. Reconstruct required static constants through loaded semantic/static authority. Preserve registered primitive origin and source-owned primitive contract attachment ordering. Existing source startup may call the shared owner helpers with its own authority.</action>
  <acceptance_criteria>Artifact native-floor setup has no provider/parsed-source call edge; native and primordial binding identity remains singly owned.</acceptance_criteria>
  <verify>One compiler check with T6; native binding/class parity execution remains deferred.</verify>
  <done>Artifact native-floor setup has no provider/parsed-source call edge; native and primordial binding identity remains singly owned. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T6 — Bootstrap a VM from one validated loaded artifact

<task id="P8.T6" type="tracer" wave="3">
  <name>Bootstrap a VM from one validated loaded artifact</name>
  <depends_on>P8.T5; executable validated before VM allocation</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>vm/bootstrap.rs full source bootstrap; error.rs; P7 LoadedUniverseArtifact</read_first>
  <files>phalcom-core/src/vm/bootstrap.rs; artifact/materialize.rs; modules/canonical_artifact.rs; error.rs</files>
  <action>Expose VM::try_new_with_universe_artifact(Arc&lt;LoadedUniverseArtifact&gt;) returning typed bootstrap error. Require PORTABLE_EXECUTABLE and Debug Universe profile; reject compiler-only artifacts for VM construction. Install authority-driven native floor, reserve classes/constants, relocate metadata and anchors, materialize initializer closures, execute exactly the stored eager bootstrap order, attach remaining source-owned primitive bindings, synchronize aliases and establish semantic roots/invariants in existing order. Preserve module initialization state and failure semantics. Retain the same immutable artifact Arc in the VM for later prototypes/diagnostics; allocate runtime state afresh. The explicit path cannot acquire canonical_universe_program or NativeSourceIndex.</action>
  <acceptance_criteria>Complete artifact bootstrap entry exists with no Universe compile call; initializer execution is per VM and follows existing failure behavior.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-core --lib. Source-free runtime tracer acceptance deferred.</verify>
  <done>Complete artifact bootstrap entry exists with no Universe compile call; initializer execution is per VM and follows existing failure behavior. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T7 — Supply stored code for every later Universe import

<task id="P8.T7" type="auto" wave="4">
  <name>Supply stored code for every later Universe import</name>
  <depends_on>P8.T6; complete non-eager prototype coverage</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>interpret.rs::run_compiled; modules/materialize.rs; module initialization registry</read_first>
  <files>phalcom-core/src/interpret.rs; modules/materialize.rs; artifact/materialize.rs; module runtime records</files>
  <action>Attach or lazily materialize the stored initializer prototype for any canonical Universe module requested by a user program. Preserve current dependency initialization order, once-only state and error propagation; eager initialized modules are not rerun. Recognize explicit artifact Universe authority before the source_text compilation fallback. Missing canonical prototype is a typed error, never a signal to compile Universe from source. User modules continue using their existing compilation path. Keep semantic/module identity aligned with the loaded image and preserve logical runtime diagnostics without source text.</action>
  <acceptance_criteria>Non-eager canonical imports cannot use AST-to-bytecode fallback and preserve initialization state semantics.</acceptance_criteria>
  <verify>Batch with T8 compiler check; late-import execution deferred.</verify>
  <done>Non-eager canonical imports cannot use AST-to-bytecode fallback and preserve initialization state semantics. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T8 — Expose deterministic full build and explicit CLI workflow

<task id="P8.T8" type="auto" wave="4">
  <name>Expose deterministic full build and explicit CLI workflow</name>
  <depends_on>P8.T7; P7 writer/loader/compiler injection</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>bin/phalcom/{main,cli}.rs; ProgramCompiler loaded-image APIs; modules/canonical_artifact.rs</read_first>
  <files>phalcom-core/bin/phalcom/{main,cli}.rs; modules/canonical_artifact.rs; public API exports; docs/specs/compiler-artifacts/telosc.md</files>
  <action>Add phalcom artifact build-universe --output universe.telosc, selecting the full compiler+executable build and atomically publishing only after whole-image validation. Add phalcom --universe-artifact universe.telosc app.ph to the existing default run path; load exactly once, pass the same artifact compiler authority into user compilation and VM construction. Missing/incompatible explicit paths fail typed, without fallback. Preserve existing startup without the option and existing user-code flags; Universe remains Debug independently. Allow semantic-only loading APIs for compiler tools while the explicit run requires executable compatibility. Report build summary/stage measurements using existing observability boundaries; no automatic discovery or cache policy.</action>
  <acceptance_criteria>Both explicit commands and the VM API use one loaded immutable image; current no-option behavior remains intact.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-core --lib --bin phalcom. Help/CLI smoke execution deferred.</verify>
  <done>Both explicit commands and the VM API use one loaded immutable image; current no-option behavior remains intact. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T9 — Prepare deferred executable and fresh-process acceptance

<task id="P8.T9" type="auto" wave="5">
  <name>Prepare deferred executable and fresh-process acceptance</name>
  <depends_on>P8.T8 implementation; test-execution restriction remains</depends_on>
  <reversibility rating="reversible">Private implementation can change within the owning boundary.</reversibility>
  <read_first>core tests README; original P7 proof; canonical bootstrap/native/reflection/compiler tests; telos-observability metrics</read_first>
  <files>Future core artifact execution integration tests and source-access-denial hooks; artifact executable corruption tests</files>
  <action>Cover source/artifact parity for natives/classes, closure upvalues, constants/ADTs, generic/trait/default dispatch, reflection/contracts, initializer order and exceptions. Process A builds; Process B denies every physical/embedded/provider source access before loading, compiles novel user code, bootstraps and imports a non-eager Universe module. Require zero Universe parse/interface/link/update/analysis/witness/export/lowering/AST-bytecode work. Load one Arc into two VMs and prove independent globals/heaps/caches with within-VM sharing. Reject corrupt/incompatible code before initializer counters change; failed load/bootstrap state remains retryable as permitted by current semantics. Compare fresh-build bytes under changed physical roots/allocation histories. Measure read/validation/relocation/VM allocation/initialization separately. Do not execute any of these gates now.</action>
  <acceptance_criteria>Coverage maps every acceptance dimension to a focused harness and explicit deferred evidence; no PASS is inferred from compilation.</acceptance_criteria>
  <verify>DO NOT RUN tests/probes/benchmarks. Targeted compiler-only test-target checks may be used when harness code is later implemented.</verify>
  <done>Coverage maps every acceptance dimension to a focused harness and explicit deferred evidence; no PASS is inferred from compilation. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P8.T10 — Close lifecycle only with source-free execution evidence

<task id="P8.T10" type="auto" wave="5">
  <name>Close lifecycle only with source-free execution evidence</name>
  <depends_on>P8.T9 code and actual evidence only after user policy change</depends_on>
  <reversibility rating="reversible">Private implementation can change within the owning boundary.</reversibility>
  <read_first>implementation-record lifecycle; CHECKPOINT.md; P7/P8 plans</read_first>
  <files>CHECKPOINT.md; P8/PERF003.C2.P8-{walkthrough,handoff}.md; telosc executable schema tables</files>
  <action>Record implemented contracts and exact compiler results separately from behavioral acceptance. P8 cannot become COMPLETE or a fully compiled Universe artifact until the source-free execution, lazy import, determinism and two-VM gates pass with recorded evidence. Under current policy retain IN_PROGRESS/PARTIAL-or-IMPLEMENTED/UNVERIFIED as appropriate. Preserve P7 closure evidence as its own record; list residual typed projection failures rather than omit code. At P8 closure, record the stable explicit build/load/runtime APIs and format ABIs in the plan completion records. Do not create P9 or another sibling plan as part of this closure; defer any separate follow-on planning until explicitly requested. Do not commit/push or certify release without instruction.</action>
  <acceptance_criteria>P7/P8 status reflects actual completed gates; the P8 records expose no hidden source fallback or unexplained missing payload; no successor plan was created.</acceptance_criteria>
  <verify>Prose-only closure record: no Cargo command.</verify>
  <done>P7/P8 status reflects actual completed gates, no successor plan was created, and the P8 records expose no hidden source fallback or unexplained missing payload. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## Acceptance gates and execution budget

### Current implementation reconciliation — 2026-10-21

This table describes the live tree. A partial implementation does not satisfy
the task's full acceptance criteria.

| Task | Current state | Evidence or remaining work |
|---|---|---|
| P8.T1 | IMPLEMENTED / FOCUSED_TESTED | The portable executable representation, opcode manifest, bounded structural codec, and field census are complete. Semantic execution safety remains T3 work. |
| P8.T2 | IMPLEMENTED / FOCUSED_TESTED | The existing compiler/emitter produces complete deterministic portable staging data. Capture remains non-authoritative and does not prove runtime consumption. |
| P8.T3 | PARTIAL / UNVERIFIED | `phalcom-core/src/artifact/validate.rs` now has exhaustive stack transfers for all portable opcodes, checked underflow and joins, edge-sensitive transfers, the callable/nested receiver-plus-parameter entry floor, reachable local-slot checks, and the independent static `max_slots` bound. Tuple/record builders account for runtime key/value pairs; static products use semantic-pool shape; fused local/constant invokes retain their result. `ReturnNonLocal` requires a graph-classified block-only activation with a home-frame token; the user selected this schema-preserving proof, including rejection of mixed or unknown activation routes that use the opcode. Method recipes validate selector/signature shape against the body prototype or native-anchor calling shape; detached closure rows require their target prototype to have the exact stable callable owner. The user selected conservative schema-preserving provenance proof for ordinary `Method` and `VariantMethod`: the validator must tie singleton method/class provenance from bytecode/CFG facts to actual installation operands, recipes/anchors, selector/side and class/variant authority, and reject unknown or conflicting installations. It must not infer holder/access owner from callable owner. P7, class, closure-template, and native-anchor checks remain in their existing owners. `validate_executable_image` still returns `Result<(), _>` and does not construct authority. The authorized core library check passes with existing warnings; behavior tests and runtime gates remain unrun. Compilation is not behavior evidence. |
| P8.T4 | NOT IMPLEMENTED | No executable-to-VM allocation/relocation path with rooting and failure atomicity exists. |
| P8.T5 | NOT IMPLEMENTED | Artifact VM native/bootstrap setup still depends on the source-backed canonical Universe path. |
| P8.T6 | NOT IMPLEMENTED | No VM entry point accepts one validated executable artifact and initializes it source-free. |
| P8.T7 | NOT IMPLEMENTED | Later canonical imports do not have a persisted prototype registry with a demonstrated no-source path. |
| P8.T8 | PARTIAL | The staged capture API exists; full compiler-plus-executable publication and explicit CLI/API use of one loaded artifact remain open. |
| P8.T9 | PARTIAL | The persistence census and capture determinism tests exist. The source/artifact parity, denial, retry, and two-VM harnesses and evidence remain open. |
| P8.T10 | OPEN | Final P8 closure requires the remaining behavior and gate evidence. P8 remains the active plan; no successor plan was created. |

The current gate results below are authoritative for verification. The T3
continuation handoff records the next implementation slice and the still-
restricted runtime gates.

| Gate | Evidence | Current state |
|---|---|---|
| P8.G1 | Complete P7 loaded compiler interface and targeted compiler check | PASS: P7 loaded compiler interface is present; focused `phalcom-core` compilation and tests pass |
| P8.G2 | Opcode/constant/semantic/native validator rejects malformed/incompatible payload before initialization | NOT RUN |
| P8.G3 | Source/artifact runtime parity across native/classes/closures/static/ADT/generic/trait/reflection/error/order | NOT RUN |
| P8.G4 | Fresh-process source-unavailable eager and later-import execution; all Universe compiler counters zero | NOT RUN |
| P8.G5 | Two-VM independent mutable state, allocation/rooting correctness and within-VM sharing | NOT RUN |
| P8.G6 | Fresh-process byte determinism, failed-load retry and separate startup timings | PARTIAL: byte determinism PASS in two independent capture processes (4,999,601 bytes; digest `c2af943406376eae636979f1223ea58895db3265a2d43dcf48c5da759267efea`); failed-load retry and startup timings NOT RUN |
| P8.G7 | P7 source-free compiler gate plus P8 focused closure records; broader certification separately authorized | DEFERRED |

For the current implementation continuation, verification is limited to
`RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib`; this check
passes with warnings on the current worktree and establishes compilation only.
Do not run validator tests, other Cargo tests, runtime probes, failed-load
retry, startup timing, two-VM checks, workspace suites, Clippy, or benchmarks
unless the user authorizes them. The earlier focused capture and
semantic-projection results remain historical evidence recorded in the
checkpoint.

FIXED: existing emitter, no live heap persistence, stable symbolic ownership, fresh VM state, exact observable ordering, compiler/executable compatibility separation, validation before initializer execution and no explicit-path fallback. MECHANICALLY FLEXIBLE: private adapter names/files, equivalent API parameter packaging and bounded decoder decomposition. VERIFY-FIRST: emitted object census, build-time initialization prerequisites, all closure capture sites, native installation side effects and current rooting APIs.

Stop and consult when a legal emitter output lacks a portable recipe, compilation depends on runtime captured state that cannot be represented, primordial identity/order would change, verification requires executing untrusted bytecode to establish safety, or GC safety/canonical owner semantics need redesign. Do not create a second compiler, relaxed validator or special-case Universe source provider. Allow at most three causal mechanical compiler cleanup cycles; classify unrelated failures and avoid broad repairs.
