# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `ARTF001`
- Checkpoint: `ARTF001.C1`
- Plan: `ARTF001.C1.P2.R4`
- Task/Gate: `R4.T7.2` positive compiler parity; canonical Universe compilation
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/artifact-universe-consolidation`
- Starting revision: `272c8862e` (artifact branch before source-fix integration)
- Current revision: `4f0861428d82b962a094f588450e9d0577960e68`

## 2. Escalation trigger

Mandatory trigger 8 applies: a semantic-to-runtime generic-evidence contract
relied on by compiler execution now fails during canonical Universe
compilation. Trigger 11 also applies because possible corrections have
different ownership and evidence-materialization consequences. No architecture
change has been attempted.

## 3. Required invariant

- Preserve one canonical semantic authority and the accepted associated
  projection rule: projection subjects must not be inferred backwards.
- Preserve every proven generic conformance obligation when constructing
  runtime call environments; do not silently drop evidence or accept an
  incomplete runtime recipe.
- Keep the strict P2.R4 source/artifact parity comparator and the P2/P3
  boundary unchanged.

## 4. Expected architecture

Semantic analysis publishes call specializations and their conformance proof
provenance. Compiler lowering maps each semantic evidence slot to a runtime
recipe. Canonical Universe compilation should succeed when those semantic
products are valid; a missing runtime recipe indicates either incomplete
semantic evidence or a missing compiler projection for a supported proof
form.

## 5. Observed behavior

After integrating the Universe semantic-source correction, the positive
SlotMap parity test fails before reaching the parity comparison. Its initial
`canonical_universe_program()` compilation returns:

```text
generic evidence layout error: missing runtime conformance recipe for semantic
slot 0 at call to CallableId { owner: DeclarationId {
module: Universe/collections.iterator_laws, name: "IndexedIterator" },
selector: new, side: Class }
```

This differs from the pre-integration run at `272c8862e`, which reached the
strict comparator and failed with 12 parity differences. The evidence does not
yet identify which provenance branch left the slot empty.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 \
  --config 'build.rustc-wrapper=""' \
  test -p phalcom-core --test core \
  source_backed_and_artifact_backed_compiler_program_parity -- --nocapture
```

The named test was selected and failed during canonical Universe compilation
after 97.95 seconds, before source/artifact comparison.

## 7. Relevant code path

```text
Iterable.indexed
  -> IndexedIterator<Self::Iterator>.new(self.iterator)
  -> semantic call conformance binding
  -> phalcom-core/src/modules/compile.rs runtime slot recipe construction
  -> missing recipe error
```

Relevant source locations:

- `phalcom-core/core/universe/src/collections/iterator_laws.ph:46`
- `phalcom-core/src/modules/compile.rs:2403-2438` builds caller evidence.
- `phalcom-core/src/modules/compile.rs:2585-2994` builds callee runtime slots.
- In that mapping, `Forwarded` leaves a slot empty when the caller slot lookup
  misses, while `CompilerDerived` currently has no runtime-recipe branch. The
  final error reports the slot and callee but not the source binding.
- `phalcom-semantic/src/checker/call.rs` now discharges an associated-projection
  equality through generic evidence; `checker/inference.rs` defers projection
  equality instead of solving its subject backwards.

The failing generic slot's exact `GenericCallConformanceSource` has not been
observed. Do not assume which branch is responsible.

## 8. Scoped implementation diff

The integrated source correction changes generic-call equality discharge and
associated-projection inference. No change was made to runtime recipe
construction. This incident packet and its checkpoint reference are the only
changes for the current diagnosis.

## 9. Evidence

- The focused parity test selected one test and failed before comparator
  output, with the missing-recipe error above.
- The pre-integration artifact revision `272c8862e` completed canonical
  compilation and reached the strict comparator, where it reported 12
  differences. That run did not pass positive parity.
- The source-fix proof workflow passed on implementation commit
  `bd4736743277da2dfd155e50120a48ab374b8e87` (run `39713887081`).
- The documentation-tip workflow passed on `4f0861428d82b962a094f588450e9d0577960e68`
  (run `39714394077`); its job includes the focused Universe source gates.
- The failing test's initial canonical compile names `IndexedIterator.new`
  in the `iterator_laws` Universe module.
- Runtime recipe construction has explicit paths for exact, forwarded,
  derived, and compiler-derived provenance; missing selection/caller layout
  can leave the initialized slot empty, and compiler-derived currently has no
  recipe conversion.

## 10. Implementer's causal hypothesis

The source correction may now discharge a previously unresolved projection
equality and publish a valid generic call specialization. Runtime recipe
construction may not be able to materialize the resulting conformance source,
or caller/callee slot identities may fail to align after projection
normalization. Evidence does not yet distinguish these cases.

## 11. Alternative hypotheses

- The generic call's conformance source may be `Forwarded`, but the caller
  evidence layout may use a structurally equivalent yet nonidentical key.
- The proof may be `CompilerDerived`; in that case the current runtime mapping
  intentionally leaves it without a recipe, and the relevant compiler/runtime
  contract needs an explicit rule.
- The proof may be `Exact` or `Derived` with a requirement-selection mismatch.
- The new semantic specialization may be invalid for runtime lowering despite
  satisfying the source-level constraint; if so, the semantic rule or test
  oracle needs review rather than a recipe workaround.

## 12. Attempts already made

No corrective edit was attempted. The failure was reproduced once with the
exact focused test after the source-fix integration. Read-only inspection
confirmed the runtime mapping branches and the source-fix diff; it did not
identify the actual binding provenance. A second behavior-changing attempt
would cross the semantic/runtime contract without that evidence.

## 13. Why local authority is insufficient

The next step must choose whether the missing product is a canonical semantic
proof, a caller-to-callee evidence identity mapping, or a runtime
representation of an already-valid proof. Each choice affects semantic
authority or the compiler/runtime generic-evidence contract. A local fallback
could invent runtime authority or weaken fail-closed behavior.

## 14. Decision requested

What is the narrowest principled way to identify the exact semantic proof
source for the failing `IndexedIterator.new` slot, and how should a valid
proof of that form map to runtime authority while preserving canonical
identity and fail-closed recipe construction? Please specify the focused
regression evidence required before resuming implementation.

## 15. Advisory decision — 2026-10-22

The failure does not establish whether the missing slot is an invalid
semantic product, a forwarded-key lookup miss, or an unimplemented runtime
projection of a supported proof form. Before changing runtime recipe behavior,
add a failure-only trace at recipe construction that reports:

- the semantic slot key and requirement;
- whether a matching conformance binding exists and its proof-source kind;
- for forwarded evidence, the caller slot key and lookup result.

Then add or extend a focused `Iterable.indexed` regression to exercise the
identified evidence path and runtime recipe construction. Keep compilation
fail-closed, preserve the non-injective projection rule, and retain strict
source/artifact parity. A forwarded-key repair is justified only if the trace
proves that the assumed conformance is valid but the current lookup misses it.
A compiler-derived recipe is justified only if its runtime requirement is
established. Revisit semantic proof publication only if the published proof
violates its semantic contract. No plan amendment is required for this
diagnostic trace; defer any behavioral architecture decision until the trace
identifies the proof source.

## 16. Trace result — 2026-10-22

The focused parity test was rerun after adding the failure-only details. The
trace short-circuits on the first unresolved slot, so it identifies an earlier
failure than the prior aggregate error:

```text
call: Universe/collections.iterator_laws::SkipIterator.new
slot: GenericConformanceKey { subject: Canonical(TypeId(938)), trait: Iterator }
requirement: Iterator.chain
binding lookup: no matching semantic conformance binding
call specialization: present, with zero conformance bindings
conditional capability: zero conformance bindings
```

The source call is `Iterator.skip`, which invokes `SkipIterator.new(self, n)`;
the constructor declares `where I |= Iterator`. This is evidence that the
compiler has a callee runtime evidence slot but the published call
specialization provides no corresponding semantic binding. It does not yet
decide whether that absence is a semantic publication defect or whether the
runtime layout is including a constraint outside the call's executable
contract. The failing command completed after 105.35 seconds with one selected
test, zero passed, and one failed. No runtime authority was synthesized.

The implementation diff from the pre-integration artifact revision contains
semantic checker changes in `call.rs`, `context.rs`, `inference.rs`,
`generic_evidence.rs`, and `impls.rs`, but no `phalcom-core/src/modules/compile.rs`
change. The source integration is therefore not by itself proven to cause the
missing binding.

## 17. Follow-up decision requested

Given that a `CallSpecialization` exists but contains no conformance binding
for the constructor's `I |= Iterator` slot, should the next focused step
inspect/correct semantic call-specialization publication for this constructor,
or should the runtime evidence layout exclude this slot because it is not part
of the constructor's executable contract? Specify the invariant and the
minimal semantic/runtime regression that distinguishes those cases. Do not
weaken source checking or synthesize an unproven runtime recipe.

## 18. Advisory ruling and focused implementation discriminator — 2026-10-22

The adviser ruled that the constructor's explicit `I |= Iterator` constraint
must remain in its executable contract. Repair semantic call-specialization
publication so the `Iterator.skip -> SkipIterator.new` specialization carries
a `Forwarded` binding from the caller's `Self |= Iterator` evidence. Do not
narrow the runtime evidence layout unless a separate executable-contract
analysis proves the slot is not consumed.

Before the single focused semantic correction, add a regression for the exact
positive call and require the `Forwarded` binding. Pair it with an unconstrained
caller invoking the same constructor; that call must receive
`GenericConstraintUnsatisfied` and publish no specialization. Then run a core
compile/runtime regression through `skip(...).next`. If semantic publication
passes but lowering still fails, investigate runtime identity separately.

**Observed:** the trace shows a present specialization with no conformance
bindings at a constructor slot requiring `I |= Iterator`.

**Causal hypothesis:** constructor call-specialization publication omits the
callee's owner-parameter conformance obligation or fails to discharge it from
the trait method's ambient evidence. The runtime layout represents a real
callee contract and must remain unchanged.

**Prediction:** the new positive semantic assertion will expose the missing
binding, while the negative caller will continue to reject the unproved call.
If the positive binding is already present, this hypothesis is refuted and
runtime slot identity becomes the next focused question.

## 19. First semantic discriminator and new escalation — 2026-10-22

The positive regression confirms that both the canonical constructor
signature and the raw class-side dispatch signature retain its conformance
constraint. Before the semantic edit, however, the published specialization
has the inferred class parameter and zero conformance bindings.

One focused correction in `CheckingContext::specialize_dispatch_signature`
now preserves constraint-only generic signatures through receiver
specialization. This prevents a `where` clause on a declaration-owned
parameter from disappearing just because the callable signature declares no
callable-owned parameters. With that change, the exact positive semantic test
reports `GenericConstraintUnsatisfied` at `Iterator.skip -> SkipIterator.new`
instead of publishing the expected `Forwarded` binding. The negative fixture
has not yet been reached.

**Observed:** trait default-method body setup passes the trait surface and
`Self` receiver type, but constructs `ctx.generic_evidence` from only the
explicit owner, callable, and impl constraints. The compiler caller evidence
layout likewise composes those published signatures. The required
`Self |= Iterator` authority is not visible in that environment.

**Updated hypothesis:** preserving the constructor contract is necessary but
insufficient. The call needs the canonical assumed receiver conformance for a
trait default method to be represented as caller evidence and mapped to a
runtime caller slot. The previous adviser ruling names this expected
`Forwarded` source, but the live body and compiler setup do not show its owning
publication path.

**Escalation:** this changes the source of semantic evidence for trait
receivers and its compiler/runtime layout. Stop edits to this path pending a
decision on whether implicit `Self |= Trait` belongs in the canonical body
evidence environment, a separate trait-default evidence product, or another
existing authority. Preserve the constructor constraint and fail-closed
runtime mapping. The focused semantic regression is retained as the
discriminator; a core `skip(...).next` gate remains pending.

**Decision requested:** identify the canonical owner and representation for a
trait default method's implicit `Self |= Trait` assumption; specify how that
same identity becomes a caller runtime evidence slot; and confirm the minimum
positive and negative semantic/compiler tests. Do not add a second ad hoc
source of conformance authority or synthesize a slot only in compiler code.

## 20. Follow-up advisory ruling — 2026-10-22

The adviser ruled that a trait default method's implicit receiver conformance
belongs in its lexical `GenericEvidenceEnvironment`, keyed by the existing
symbolic `SelfTypeTerm` for the owning trait and the exact `TraitRef`. The
assumption is owned by the trait declaration and remains local to trait
default bodies; it must not be published as an explicit source generic
constraint or seeded into unrelated bodies.

Use one shared semantic constructor to seed this assumption for body checking
and compiler caller-layout construction. Reuse the existing assumed-proof,
forwarded-call-provenance, and evidence-slot paths. Keep constraint-only
generic-signature preservation in receiver specialization.

Focused gates: the positive semantic call publishes a `Forwarded` binding
from the exact trait `Self` slot; the unconstrained caller is rejected with
`GenericConstraintUnsatisfied` and has no specialization; and the core
`skip(...).next` path constructs its runtime recipe and preserves observable
iterator behavior. This is the accepted scoped continuation; no second
authority or compiler-only synthesized slot is introduced.

## 21. Receiver-key identity follow-up — 2026-10-22

**Escalation trigger:** canonical identity/canonicalization and the semantic
failure budget (shared consultation protocol triggers 4 and 10). Stop edits to
the conformance proof boundary until the identity rule is confirmed.

**Observed:** the shared environment constructor now seeds the prescribed
symbolic `TypeTerm::SelfType(SelfTypeTerm { owner: Iterator, side: Instance,
role: InstanceType })` with `TraitRef(Iterator, [])`. The selected semantic
regression still emits `GenericConstraintUnsatisfied` at the constructor call
range (49..74), so the positive fixture fails before its forwarded-binding
assertion or negative fixture is reached. The helper currently runs only for
an instance callable whose declaration owner equals the supplied trait
surface. Compiler caller layout and semantic lowering use the same helper.

**Code evidence:** in `checker/call.rs`, a canonical conformance subject is
materialized and unconditionally wrapped as `TypeTerm::Canonical(m_ty)` before
`query_generic_conformance_proof_with_structural`. The new assumption is keyed
by symbolic `TypeTerm::SelfType`. The test trait is nongeneric, so its exact
trait reference has no arguments on both sides.

**Causal hypothesis:** the proof query misses the new assumption because the
materialized canonical type is the canonical store form of `SelfType`, while
the environment key is its symbolic `TypeTerm::SelfType` identity.

**Alternative hypotheses:** the trait body may not match the new seeding
predicate at runtime, or another call-site substitution detail may change the
subject before the proof query. The current failure output does not print the
runtime proof key.

**Attempt and result:** added the shared lexical assumption and exact positive
semantic assertion, while preserving the constructor's constraint-only
generic signature and adding compiler/semantic-lowering slot seeding. Predicted
the proof query would return `Assumed`; observed the same generic-constraint
diagnostic. No slot or proof rule was weakened.

**Why local authority is insufficient:** the ruling requires the canonical
symbolic receiver identity, but changing call-site canonicalization can affect
semantic proof keys beyond this trait default. Adding a second canonical
assumption key would violate the one-identity direction. Confirm the owning
normalization boundary before making that cross-cutting correction.

**Decision requested:** should the generic-call proof path preserve a
materialized `TypeData::SelfType` as its matching symbolic `TypeTerm::SelfType`
before lookup, or is a different canonical identity mapping required? Specify
the narrowest owner and a discriminator that confirms the runtime key matches
the trait-default assumption without adding a duplicate key or weakening
fail-closed proof behavior.

## 22. Advisory decision — 2026-10-22

The adviser confirmed that the generic-evidence proof path owns this identity
bridge. Before memoization and lexical-assumption lookup, a
`TypeTerm::Canonical(id)` whose exact canonical type is `TypeData::SelfType`
must be represented as the matching symbolic `TypeTerm::SelfType`, preserving
owner, side, role, and exact `TraitRef`. This is one representation bridge for
the same identity, not a second assumption. Do not normalize arbitrary nominal
types or add canonical and symbolic duplicate keys.

Required discriminator: a focused generic-evidence unit test queries the
assumption using both its symbolic form and the exact canonical SelfType form
and gets the same assumed proof; changed owner, side, role, or trait remain
unproven. Then require the call-site `Forwarded` assertion and the core
`skip(...).next` runtime gate. No plan amendment is required; the implementer
may resume on this scoped correction.

## 23. Runtime gate blocked by canonical source bootstrap cycle — 2026-10-22

**Observed:** the proof normalization unit test passed (1/1), and the focused
semantic call regression passed (1/1), including the unconstrained negative
case. The `language-corpus iteration` command selected three tests; all three
failed before executing their fixtures because `VM::new()` reported
`canonical Universe source initialization cycle`. The direct core
`boot_01_bootstrap_measurement_separates_catalog_closure_and_execution` test
also passed its explicit Universe-root reachability/order assertions, then
failed when it constructed `VM::new()`.

A temporary source-index-only probe (removed after use) showed the expanded
`bootstrap_roots()` order is blocked by the mutual dependency
`reflection.module -> reflection.package_object -> reflection.module`.
`reflection.project`, `reflection.resolved_project_dependency`,
`reflection.module_dependency`, `reflection.export_table`, `reflection.export`,
and `reflection.child_module_table` remain unscheduled behind this pair. The
source declarations are native class surfaces; for example, `Module` refers to
`Package` in signatures while `Package` extends `Module`. The imports are in
the checked-in Universe source and were not changed by the T7 patch.

**Causal hypothesis:** the canonical eager-root order treats every module
import as a runtime initializer dependency, so it rejects the declaration
cycle even though the explicit `universe` root closure itself orders
successfully. The probe confirms the graph and failing boundary; it does not
establish whether the correct owner is source imports, runtime dependency
extraction, or loader support for declaration-only strongly connected
components.

**Attempt and result:** ran the smallest existing runtime discriminators and
then isolated the root-order graph without changing Universe sources or
weakening the cycle check. The temporary graph probe and diagnostic changes
were removed. No runtime-ordering or import edit has been made.

**Escalation trigger:** resolving this requires choosing between changing
accepted source dependency semantics and changing the compiler/runtime
initialization contract. This is a cross-layer initialization-order decision;
the T7 implementer must not silently choose one.

**Decision requested:** should declaration-only module SCCs such as the
`Module`/`Package` pair be supported by canonical bootstrap ordering, or should
the Universe source imports/declarations be reshaped? Identify the owning
layer, invariants for executable top-level initializers, and the narrowest
regression that distinguishes a safe declaration SCC from a genuine runtime
initialization cycle. Confirm whether this blocker belongs within P2.R4's
runtime acceptance or should remain a separately owned baseline blocker.
