# Random R0 implementation incident

Current state: stable-owner and method-generic recipe prerequisites are
IMPLEMENTED / FOCUSED_TESTED. The original R0 syntax probe passes. The API slice
and canonical Universe fixes are present; the plan remains IN_PROGRESS / PARTIAL /
FOCUSED_TESTED. Native source identity/effect metadata, provider registration,
numeric-policy sampling, the random corpus, and unrepresentable byte-count
rejection have focused PASS evidence. The user deferred source mutation-effect
presentation; representable byte-allocation safety still needs an owner-layer
decision. Release
performance, full corpus, broad core, and cross-platform gates remain incomplete.
The completion handoff owns the next steps; earlier entries below record the
investigation history.

## Architect delivery checkpoint (2026-10-08)

Status remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED. R11 source-effect deferral
stands. R21 is resolved through a general configurable 64 MiB Bytes.new limit
and fallible backing reservation, independent of numeric budgets. Owner and
public secure-caller regressions pass. Release expression-owner stack repair
passed optimized syntax startup. Reactor idle-poll repair passed all 16 reactor
tests. Final core-lib selection: 20 PASS / 1 FAIL / 147 filtered, 28.74 seconds.
Retained publication repair remains PARTIAL: expanded actual-fixture replacement
fails at `phalcom-semantic/src/session.rs:6169`, where an accepted inherent
callable lacks a canonical signature. Preserve the failing regression and the
invariant. The earlier simple four-step retained sequence passed.

Completed incremental lane: 208 PASS / 36 FAIL / 4 ignored. All 36 failures
reproduce on predecessor behavior: 33 smaller exact tests plus three large reuse
cases (0 PASS / 3 FAIL, 205.16 seconds). These are Category D baseline failures.
Exact identities are retained in the walkthrough. No assertions were weakened.

Remaining tasks: finish canonical retained inherent-signature publication, rerun
exact regression and affected acceptance gates, rebuild final release CLI,
complete R20 throughput/allocation/GC evidence, full corpus classification, broad
core and cross-platform gates. The million-word benchmark was interrupted at
225.51 seconds before reactor repair; no workload throughput is claimed.

The user requested lifecycle updates, commit/push of everything, and an inline
handoff at this checkpoint. Existing COLL006, slotmap, and math-document
relocations are included under that instruction without additional behavioral
verification. Delivery does not promote completion or verification status.

## Architect blocker repair (2026-10-08)

The user renewed general fix authority with the architect role, inline on main,
without subagents. R21 is authorized at its owning Bytes/VM boundary: a
configurable per-construction limit, default 64 MiB, independent of numeric
budgets; fallible backing reservation; controlled limit/allocation errors before
secure host fill. Fixed-length zero-filled storage and canonical native source
identity remain unchanged. The plan and VM specification record this amendment.
The small configured boundary and public secure caller have focused PASS
results; no giant allocation was attempted. R11's prior deferral stands.

Release-bootstrap diagnosis found `pop_expression_owner` putting the stack
mutation inside `debug_assert_eq!`. Optimized builds omitted the mutation, so
later checked operations could attach to a nested expression instead of their
actual owner. Move the pop outside the assertion; retain the LIFO check and add
a nested-owner regression. Prediction: canonical associated invocation products
will again attach to the correct expression and release Universe bootstrap will
succeed. Release build PASS and original syntax fixture PASS (`1`), 13.17 seconds
whole-process time, 619,413,504 bytes peak RSS. No compiler fallback is introduced.

Retained-session diagnosis reproduced stale source on a seed-only replacement.
Using the existing fallible semantic mutation API exposed unvalidated effective
callable definitions; subsequent exact diagnostics exposed constructor-body vs
factory-signature identity, retained witness body/access facts, and omitted
primary member prerequisites when reopening a declaration surface. The session
repairs use canonical definition tables, accepted source coordinates, existing
constructor correspondence, and exact owner signature worklists. The first
four-step replacement/cold-parity regression passes; the expanded actual-fixture
regression and owning incremental gates are pending.

The first million-word release benchmark was interrupted after 225.51 seconds
without completion (15.55 seconds user CPU, 19.04 seconds system CPU). A one-second
sample put 859/871 main-thread samples in reactor polling, with no active source
registrations. This is incomplete throughput evidence. The general reactor now
skips the OS poll only when its registration table has no active source, while
always draining completion ingress. Active polling and executor waits retain
their existing behavior. Reactor gates and renewed measurements are pending.

## R11 and R21 acceptance blockers (2026-10-08)

Continuing the completion plan exposed two repository-wide premises that the
random implementation cannot safely repair with a local workaround.

### Source mutation effects

**Observed:** the accepted plan requires source-authored `Xoshiro256PlusPlus`
state mutation to appear in ordinary source effect inference. The owning
`phalcom-semantic/src/effects/infer.rs::infer_intraprocedural_effects` initializes
`current_effects` to `EffectSet::EMPTY`, checks only expression status/opacity,
and returns that empty set for every ready callable. It does not inspect
`ExpressionAnalysis.binding_write`, field writes, or call dependencies. The
snapshot retains callable analyses, but `infer_interprocedural_effects_scc` has
no production caller that publishes a source effect product. Therefore this
repository cannot currently provide the planned deterministic mutation-effect
assertion. Native effect metadata is separately verifiable and is covered by a
new focused source/native identity test.

**Causal assessment:** source effect inference/publication is a general semantic
gap, not a random-specific declaration omission. A random-only effect table or
manual annotation would create a second source of truth and misrepresent the
semantic authority.

**Requested decision:** identify/authorize the owning general source-effect
inference and publication seam, or amend R11 to defer deterministic source
effect presentation until that general capability exists. No random-specific
effect workaround is permitted.

**User ruling (2026-10-08):** defer deterministic source effect presentation
until general source effect inference/publication exists. The random plan's
R11.1 acceptance criterion is amended; native effect metadata and canonical
source-identity checks remain required and are covered.

### Large `SecureRandom.bytes` allocation

**Observed:** a focused probe of
`SecureRandom.system.bytes(10 ** 100)` returns the controlled
`Bytes.new` index error `Expected a non-negative integer index, got object`
before host fill. This covers counts not representable as immediate host
indices. However, `phalcom-core/src/primitive/bytes.rs::bytes_class_new` calls
`expect_index` and then `BytesObject::new_zeroed`; the latter uses
`vec![0u8; len]` without consulting a byte-allocation limit or using a fallible
reservation path. `NumericPolicy.max_numeric_allocation_bytes` applies to
numeric objects, not byte buffers. A very large count that still fits the
immediate integer range can therefore reach an allocation that may abort on
memory pressure.

**Causal assessment:** the R21 plan premise that the existing `Bytes.new`
boundary enforces allocation feasibility is false in the live tree. Adding a
random-specific cap or allocating an intermediate vector would violate the
plan. A safe repair needs an owning general `Bytes` allocation/resource
contract and host-failure behavior.

**Requested decision:** authorize/amend a bounded general `Bytes.new` resource
limit/fallible-allocation contract for the random caller to consume, or accept
that only unrepresentable host-index counts are currently guaranteed to fail.
No large representable allocation was attempted.

### Release benchmark bootstrap

**Observed:** `RUSTFLAGS='' cargo build --release -p phalcom-core --bin phalcom`
completed. Running either the new `next_u64.ph` benchmark or the existing small
random syntax fixture with `target/release/phalcom` fails before entry-source
analysis, twice, while compiling `universe:callable.function`:

```text
associated family invocation lowering is not implemented until associated
semantic resolution lands
```

The failed release startup took 13.62–14.71 seconds and peaked at roughly
608–628 MiB RSS. The existing debug CLI successfully ran the same small syntax
fixture and printed its expected `1`, but debug timing is not performance
evidence. No random workload timing or allocation measurement was acquired.

**Classification:** unrelated to the random source changes; a broad optimized
Universe-bootstrap failure. Do not repair it as part of random acceptance.
The R20 performance checkpoint remains unverified until a release CLI can
bootstrap this source closure.

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

Delivery hygiene: scoped Rust formatting and `git diff --check` PASS. The nested
expression-owner unit regression PASS 1/1. No broad test PASS is claimed.
