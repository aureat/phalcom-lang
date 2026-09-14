# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG005
- Checkpoint: LANG005.C2
- Plan: LANG005.C2.P3 constrained/specialized inherent-impl applicability
- Task/Gate: T6 bound callable references, class-side execution, source tooling, and incrementality
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `986568da`
- Current revision: `986568da` (working tree only; no commit)

## 2. Escalation trigger

Mandatory RED triggers 5, 8, 11, 14, and 15 fired while implementing T6:

- the runtime representation must change to carry conditional-family execution evidence;
- a compiler/runtime/semantic contract used by later work is changing;
- fallback method lifetime and GC rooting are nonlocal;
- the proposed editor projection risks becoming a second applicability-resolution path;
- generic applicability, runtime dispatch, and incremental editor products cross subsystem boundaries.

Implementation is stopped pending architectural advice.

## 3. Required invariant

- Semantic lowering remains the sole authority for conditional inherent-impl applicability.
- A bound family must retain canonical conditional selection evidence without retaining generic arguments or re-solving constraints in the VM.
- Runtime lookup must use exact declaring identity and the correct dispatch side, while allowing a strict runtime subclass override.
- Conditional fallback methods must remain GC-live through an ordinary rooted path.
- Source index and LSP completion must consume canonical semantic products and must not implement a parser-side or LSP-side solver.
- Incremental updates must invalidate affected receiver-effective products without leaking unrelated target state.

## 4. Expected architecture

T6 requires `&receiver.member` and family applications to use the same semantic conditional selection domain as direct sends, including getters, setters, indexers, and class-side receivers. Lowering should carry immutable canonical provenance into execution; the VM may select an already-compiled fallback and inspect live subclass methods, but must not inspect erased generic arguments or reconstruct impl identity from names.

The source/editor path should expose the receiver-effective member set from the semantic snapshot. It should preserve applied receiver identity for `Box<Int>` versus `Box<String>`, while leaving eligibility and canonical callable identity owned by semantic analysis.

## 5. Observed behavior

No behavioral test was run, per the explicit task instruction.

During the T6 implementation boundary, the existing bound-family lowering emitted only `MakeFamily { spec, kind }`. The semantic `BoundBehavioralMember` product had no retained conditional selection field, so a conditional method selected for a direct call could not be represented in a reified bound family. The existing runtime `FamilyObject` therefore had no path to execute that selected fallback after `&receiver.member` was formed.

The existing editor `ReceiverAlternative` retained only `(DeclarationId, ReceiverMode)`. It discarded the formal `TypeId`, so editor member projection could not distinguish an applied receiver such as `Box<Int>` from another application of `Box`, and could not safely decide which conditional members were applicable.

## 6. Minimal reproduction

Exact command/test/input:

```sh
Not run: testing and verification were explicitly deferred by the user.
```

Observed output:

```text
No runtime output was collected. The incident is an architecture/lifetime boundary discovered during implementation.
```

Expected output/behavior:

```text
&Box<Int>.value and &Box<String>.value must retain the semantic selection made
for their receiver types; the VM must execute only the applicable fallback,
preserve a strict subclass override, and keep the fallback method rooted.
```

## 7. Relevant code path

```text
CheckingContext::resolve_dispatch_target_with_specialization
  -> ResolvedDispatch::conditional
  -> resolve_bound_behavioral_family
  -> CallableReferenceResolution::BoundFamily
  -> project_callable_reference_resolution
  -> CallableReferenceLoweringSpec::MakeBoundFamily
  -> Compiler::compile_callable_reference
  -> Bytecode::MakeConditionalFamily
  -> VM::MakeConditionalFamily
  -> FamilyObject::conditional
  -> VM::activate_family_with_kind
  -> conditional fallback/strict-subclass dispatch
```

```text
EditorSemanticQuery::resolve_receiver_at
  -> ReceiverAlternative
  -> EditorSemanticQuery::members_for_receiver
  -> conditional applicability projection
  -> LSP compiler_class_completions
```

Important owning files:

- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/associated.rs`
- `phalcom-semantic/src/dispatch.rs`
- `phalcom-semantic/src/types/denotation.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/associated.rs`
- `phalcom-core/src/chunk.rs`
- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/vm/send.rs`
- `phalcom-core/src/heap/object.rs`
- `phalcom-core/src/heap/trace.rs`
- `phalcom-semantic/src/editor.rs`

## 8. Scoped implementation diff

The current-turn implementation delta, not including unrelated pre-existing work in the dirty checkout, is:

- Added canonical `ConditionalDispatchSelection` to `ResolvedDispatch` and propagated it through bound-family semantic denotation.
- Added conditional candidate discovery for pattern bound families through canonical dispatch.
- Added `ExecutableConditionalFamilyEntry`, a typed executable descriptor pool, and `MakeConditionalFamily` bytecode.
- Added runtime family entries containing exact declaring class identity, dispatch side materialized through the metaclass for class-side entries, and a compiled fallback method.
- Added VM conditional-family activation that prefers a method defined by a strict subclass of the declaring class and otherwise invokes the selected fallback.
- Moved descriptor fallback references to closure constant-pool indices so the method handles use normal closure GC roots.
- Added formal `TypeId` retention to editor receiver alternatives and semantic conditional-member projection for concrete receiver types.
- Updated affected compiler/LSP/test struct literals and heap tracing.

The working tree also contains substantial earlier LANG005 changes in the same files. They were preserved and are not attributed to this incident.

## 9. Evidence

- `ResolvedDispatch` previously had no conditional provenance field; the ordinary resolver returns `conditional: None` and checker conditional branches now populate it.
- `resolve_bound_behavioral_family` previously scanned unconditional surfaces for pattern candidates; it now discovers conditional selectors and re-runs the canonical checker dispatch query for each selector.
- `CallableReferenceLoweringSpec::MakeBoundFamily` previously carried only `BehavioralFamilySpec`; it now carries conditional executable entries.
- `FamilyObject` previously contained only `receiver` and `spec`; it now contains an immutable conditional entry slice.
- `Object::Family(family) => *family` required correction after `FamilyObject` stopped being `Copy`.
- Conditional fallback method handles are now represented by chunk constant indices, because descriptor-only handles would not automatically be visited by ordinary closure constant tracing.
- Editor receiver alternatives previously retained no `TypeId`; the new field is optional so declaration-only recovery remains fail-closed.
- No test, check, formatter, lint, or verification command was executed in this turn.

## 10. Implementer's causal hypothesis

The plan's semantic provenance direction appears sound, but T6 exposes a missing representation contract: a bound family is a reified runtime capability, while conditional applicability is a compile-time proof. The capability needs a stable, GC-rooted runtime projection of the selected proof result. The same distinction applies to editor queries: they need applied receiver identity, but must not become an independent generic constraint solver.

The current implementation hypothesis is to carry only selected callable provenance, exact declaring identity, dispatch side, and rooted fallback handles. However, the exact ownership and lifetime contract for that descriptor, and whether the editor may use a snapshot-local cloned `TypeStore` with empty ambient constraints, require advisor confirmation.

## 11. Alternative hypotheses

1. Bound families may need to retain a semantic family descriptor keyed by canonical callable identity, with runtime handles materialized lazily through an existing semantic executable pool rather than a new `FamilyObject` field.
2. The runtime may need to reuse the direct conditional invocation opcode/descriptor instead of introducing a second family-specific opcode.
3. Editor applicability may need a dedicated immutable snapshot query product produced during semantic analysis, rather than evaluating `check_impl_domain_applicability` in `EditorSemanticQuery`.
4. Class-side applied receiver evidence may require a distinct semantic type/form pair; the current public `ReceiverAlternative { declaration, mode, receiver_type }` may not encode that distinction completely.

## 12. Attempts already made

- Added semantic conditional provenance and propagated it into callable-reference lowering. Prediction: bound-family lowering would have enough canonical evidence to avoid runtime constraint solving. Result: the representation gap moved to the compiler/runtime boundary, requiring a new typed descriptor and runtime family field.
- Added `MakeConditionalFamily` and runtime dispatch using exact declaring class identity plus strict-subclass override selection. Prediction: family applications would execute selected conditional methods. Result: lifetime analysis showed descriptor-stored `ObjRef` values would not automatically be traced, so fallback references were changed to closure constant-pool indices.
- Added optional receiver `TypeId` retention and snapshot-local conditional editor projection. Prediction: completion could distinguish concrete applications while staying in the semantic layer. Result: this raises a second architectural question about whether editor queries may perform canonical matching on a cloned store or must consume precomputed snapshot products.

## 13. Why local authority is insufficient

The decision crosses canonical semantic authority, compiler/runtime representation, GC rooting, class-side identity, and editor/incremental product ownership. Choosing between a family-specific executable descriptor, reuse of the direct conditional-call representation, or a precomputed editor product changes interfaces relied upon by later T6–T8 work. A local mechanical choice could create a second source of applicability truth or an unrooted runtime handle while appearing locally correct.

## 14. Decision requested

Please decide the following single architectural question:

> Is the required T6 architecture to carry selected conditional callable provenance into a GC-rooted bound-family runtime descriptor (with exact declaring class/side and strict-subclass override probing), and to expose concrete receiver `TypeId` to the semantic editor query; or should either boundary consume a precomputed semantic executable/editor product instead? Please specify the ownership, lifetime/rooting, class-side applied-form, and no-runtime-resolving constraints that T6–T8 must preserve.

## 15. Advisory decision

### Root cause

T6 exposed two gaps: bound families were reified capabilities without an executable/rooted projection of a selected conditional implementation, and conditional applicability was being independently re-derived by direct-call lowering and the provisional editor path.

### Architecture verdict

`PLAN SOUND — ARCHITECTURAL CLARIFICATION REQUIRED`

### Required correction

- Keep `ConditionalDispatchSelection` as the canonical semantic proof and make it complete for consumers: selected `ImplId`, target-owned `CallableId`, declaring `DeclarationId`, `DispatchSide`, and specialized signature/environment where required.
- Attach the selection to the formal expression/call resolution product. Direct-call lowering must project that selection and must not scan conditional-member sets to rediscover it. Bound-family lowering may project the `BoundBehavioralMember.conditional` result already returned by canonical dispatch; pattern enumeration may discover selector names but eligibility remains canonical dispatch.
- Retain the approved family runtime design: a family-specific descriptor is allowed, fallback methods live in the enclosing closure constant pool, and the descriptor copies exact declaring behavior identity/side into the traced `FamilyObject`.
- Replace the provisional editor matcher loop with a semantic receiver-effective lookup API/product. The editor may pass the exact formal `TypeId` and ambient evidence, but must not call `check_impl_domain_applicability` from presentation code.
- Preserve class-side mode and applied receiver form as independent facts. If required evidence is unavailable, completion fails closed.

### Invariants and forbidden shortcuts

Semantic checking remains the sole applicability/coherence authority. The compiler projects immutable selected evidence; the VM receives no domains, constraints, substitutions, or erased generic arguments. Same-module canonical identity, metaclass class-side dispatch, strict-subclass override behavior, shared-class nonpollution, explicit GC tracing, and cold/incremental agreement remain mandatory.

Do not scan AST impl blocks in compiler/VM/source index/LSP; infer selection from conditional index membership; call the domain matcher from editor presentation; store descriptor-only `ObjRef` values; install conditional methods on shared classes; resolve by bare name; or maintain separate override algorithms.

### Verification and resume

No tests or verification were run by explicit instruction. The advisor listed focused semantic, core GC/runtime, and incremental/editor lanes for later execution. No plan amendment is required. The implementer may resume only after applying this correction.
