# Random R0 implementation incident

Current state: stable-owner and method-generic recipe prerequisites are
IMPLEMENTED / FOCUSED_TESTED. The original R0 syntax probe passes. The API slice
and canonical Universe fixes are present; the plan is IN_PROGRESS / PARTIAL /
FOCUSED_TESTED at the user-directed delivery stop. Performance, metadata acceptance
and remaining broad gates are pending. The completion handoff owns the next steps;
earlier entries below record the investigation history.

## Recipe repair outcome (2026-10-08)

The user explicitly renewed investigation and fix authority, requiring a sound,
clean general trait implementation. Main advanced independently to `800474d8`;
the stable owner repair was already committed as `fbf057ea`. Subsequent delivery
was explicitly requested on main; its commit includes this incident and handoff.

Instrumentation proved why the earlier correction was inert:
`CheckingContext.callable_signatures` was absent for top-level application,
while `conformance_semantics.callable_signatures` already held the canonical
selected witness. `resolve_callable_signature` now uses those existing canonical
inputs and records the normal signature dependency.

After requirement checking discharges the bounds, publication translates the
alpha-equivalent requirement method binders positionally to the selected source
callable's binders. The exact/forwarded proof products remain unchanged. This
fixed recipe compilation. Execution then proved a second transport omission:
`InvokeTraitSelected` ignored the already-projected call-entry recipe, entering
the witness with only trait/default evidence.

`TraitInvocationSpec` now retains that existing recipe. Pool remapping includes
its type roots and nested proof recipes. Lowering offsets abstract trait slots
after the selected callable's generic prefix and uses that complete prefix
instead of independently reconstructing trait-header antecedents. VM entry
merges checked owner/method type bindings, materializes the existing conformance
recipe, and enters the selected method with both environments. No new opcode,
trait discovery, runtime proof inference, identity scheme, or witness selection
was introduced. Existing method identity validation applies to the recipe.

Completed gates:

- `RUSTFLAGS='' cargo check -p phalcom-core`: PASS.
- Original `--test language-corpus random -- --nocapture`: PASS, 1 selected,
  unchanged `random_syntax.ph` and expected result.
- `--test core language::traits -- --nocapture`: PASS, 16 selected.
- Semantic `generic_trait_call_publishes_selected`: PASS, 1 selected.
- Semantic `generic_trait_witness_binding_transport`: PASS, 1 selected;
  missing conformance is still rejected before publication.
- Semantic `generic_witness_call_bindings_and_proofs`: PASS, 1 selected;
  source-only edits preserve cold/incremental publication parity.
- New core witness regression: PASS; two binders with renamed/reordered bounds,
  top-level and member-body exact calls, forwarded generic proofs, nested defaults,
  typed `List<X>` allocation, and forced GC.
- Scoped Rust formatting and `git diff --check`: PASS.

Semantic trait suite: 39 PASS / 4 FAIL before the additional negative test. A
predecessor-behavior comparison temporarily removed both semantic correction
sites, retained the new tests, and reproduced all four existing failures plus
the expected new binder regression failure. The four baseline cases are:
`exact_projection_integration_normalizes_parameter_and_return_witness_contracts`,
`normalized_associated_projection_rejects_incompatible_witness` (both
RecursiveFixpoint), `trait_default_replays_external_nominal_callable_dependencies`,
and `trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`
(both pre-existing Universe DeclarationShell dependencies).

## Subsequent random continuation gates and resolution

A source slice reached the canonical Universe gates and exposed:

1. Private field authority: a conformance method cannot acquire target private
   fields. Plan pseudocode for xoshiro must use an ordinary class-owned
   `nextU64` and inherent-witness conformance, per `objects/traits.md:253-257`.
2. `const WORD_MASK: Int = (1 << 64) - 1` failed static evaluation because the
   current intrinsic allowlist does not include shift. The owning static-value
   contract accepts exact large integer literals, so the production constants use
   `18446744073709551616` and `18446744073709551615`; their descriptors are
   materialized and tested. No mutable runtime constant was introduced.
3. Qualified `random.Xoshiro256PlusPlus` was initially treated as a module value;
   the checker now resolves qualified exported class declarations through the
   canonical type-name path, preserving class-object/type-form denotation.
4. Trait default slots were initially numbered across all snapshot traits. The
   lowering helper now numbers each trait surface locally, making canonical
   bootstrap and smaller client import closures agree. The client and full
   random corpus both pass.

The deterministic source modules, typed distributions, secure source capability,
OS `getrandom` primitive, Universe identity, effect metadata, generated native
surface, and focused acceptance corpus are retained and tested.

## Consultation ruling and prerequisite execution

The user authorized a lossless owner sum type for declaration, variant, and
conformance callables, schema v11 with legacy declaration-owner decoding,
owner-kind fingerprints, and partial declaration-dependent consumers. Exact
specialization remains evidence rather than callable identity. Changes to
conformance evidence, witness selection, dispatch, and associated-type semantics
were explicitly excluded.

The stable identity repair is IMPLEMENTED / FOCUSED_TESTED. Both projections
preserve all three owner kinds; canonical selectors use `Selector::encode()`.
Schema v11 JSON emits tagged owners and reads legacy v10 declaration-shaped
owners. Generic parameter fingerprints include owner discriminants. Reflection
resolves an enclosing nominal owner when available and returns None for an impl
owner. Synthetic test construction sites were migrated.

Completed evidence:

- `RUSTFLAGS='' cargo check -p phalcom-core`: PASS after repair.
- Semantic lib filter `stable_callable_owners`: PASS, 1 selected.
- Semantic lib filter `callable_parameter_owner_fingerprints`: PASS, 1 selected.
- Semantic integration filter `generic_conformance_stable_callable`: PASS,
  1 selected; two exact Box applications retain one source witness and source-only
  incremental reconstruction agrees with cold stable identity.
- Type metadata `schema_compat`: PASS, 19 selected. The strengthened owner test
  additionally passes whole-bundle v11 roundtrips for all owners and v10 decoding.
- Semantic `integration::metadata`: 31 PASS / 2 FAIL initially. The schema-version
  literal was migrated and its exact test now passes. The remaining structural
  fingerprint test fails with `NonExportableRecordKeyDomain` and is deferred as
  unrelated to callable owner projection (its record export fails before the
  changed callable fingerprint path).
- Core `--test core --no-run` and `--lib --no-run`: PASS (compilation only).

## New R0 blocker: runtime conformance recipe

The unchanged original R0 probe now clears stable identity projection and fails
with a structured compilation error:

```text
generic evidence layout error: missing runtime conformance recipe for semantic
slot 0 at call to Conformance(impl#4).sample(using)
```

Observed command: `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus random -- --nocapture`,
one selected test, FAIL.

The compiler builds the callee's generic evidence environment from its method
constraints (`modules/compile.rs:1266-1277`), then matches instantiated slots
against published specialization/provenance bindings (`1294-1337`). Slot 0
remains without a recipe and fails closed at `1672-1680`. The evidence does not
yet establish whether semantic publication or compiler binding matching owns
the defect. No evidence/witness corrective edit has been attempted.

New decision requested: authorize bounded diagnosis and repair of the missing
method-generic conformance recipe, preserving canonical semantic evidence and
existing witness selection. This is outside the explicitly bounded identity
ruling. Random implementation remains blocked at R0; its design is unchanged.

## Authorized recipe diagnosis and unsuccessful correction

The user authorized bounded diagnosis/repair, conditional on soundness.
Temporary detailed compiler error output established that the canonical exact
proof is present, including all four selected ProbeGenerator requirements.
The published specialization binds TypeParameterId(86) to TypeId(820), while
the callee slot environment contains Canonical(TypeId(835)). Its published exact
proof key correctly uses Canonical(TypeId(820)). The selected source callable is
Conformance(impl#4).sample(using), whereas inference consumes the trait
requirement signature in `checker/context.rs:2996-3025` and
`trait_dispatch.rs:451-498`.

Causal hypothesis: requirement and witness method-local binders are distinct,
and the checked specialization retains the requirement binder while compiler
slots use the selected callable binder.

One serious correction was attempted: at semantic specialization publication,
translate requirement method parameter bindings positionally to selected
callable method parameters, preserving the already-checked exact proof. The
unchanged probe still failed with the same missing slot error. This does not
establish that the proposed publication hook is reached with both signatures
available, nor that this mapping alone is sufficient. The attempted patch and
temporary compiler diagnostics were removed. No recipe production change remains.

Mandatory escalation now applies: the same semantic problem survived the allowed
serious correction. Requested adviser decision: identify the canonical transport
point for requirement-to-witness method binder correspondence and the proof-key
translation contract, or authorize a specific additional discriminating analysis
before another correction. Do not infer conformance in the compiler or alter
witness selection to satisfy the test.

## Identity and required invariant

Requested plan: `stdlib-random-luna-patch-grade-implementation-plan.md`, R0.2.
Execution: `main`, starting revision `c4db36bbab312cd41a589a34dad55dfbfd4ed456`.
User override: work inline on main, no subagents.

Generator conformance witnesses and Distribution associated Output must preserve
canonical callable identity through semantic-to-compiler projection. No random
special cases or Dynamic erosion are allowed.

## Observed and reproducer

`RUSTFLAGS='' cargo check -p phalcom-core` passed before edits.
Added `tests/fixtures/language/random/random_syntax.ph` and its expected output,
with a focused `corpus::random` registration. This minimal syntax probe uses a
trait default generic choose method, associated Output, a private data
constructor, and ordinary conformance implementations.

`RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus random -- --nocapture`
selects one test and panics before runtime:

```text
conformance callable has no declaration owner:
ImplId { module: <root>, local: ImplLocalId(4) }
```

A backtrace rerun with `RUST_BACKTRACE=1` establishes this path:

```text
ProgramCompiler::project_analyzed
metadata::stable_identity::to_stable_callable_with_context
CallableId::declaration_owner
CallableOwnerId::declaration
```

## Causal hypothesis and evidence

`phalcom-semantic/src/identity.rs:292-315` intentionally distinguishes
Conformance(ImplId) from Declaration and rejects requesting its declaration
owner. `metadata/stable_identity.rs:64-66` unconditionally requests that owner
when projecting a stable callable. The probe exposes a general stable-identity
projection gap rather than a PRNG algorithm defect.

No corrective attempt or production implementation was made.

## Escalation and requested decision

Repository RED triggers: stable-ID/cross-layer contract uncertainty and entering
an unexpected semantic metadata subsystem. Stable conformance ownership must
not be silently redesigned by the random implementer.

Decision requested: authorize a bounded prerequisite repair and establish the
canonical stable representation for conformance-owned callables, including the
expected compiler/runtime consumers and regression gate. Then resume R0.

Remaining random work and all final acceptance gates are unexecuted. Existing
unrelated staged/untracked user files were preserved. No commits were made.
