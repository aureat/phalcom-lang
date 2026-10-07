# Random R0 implementation incident

Status: BLOCKED / NOT_STARTED / UNVERIFIED

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
