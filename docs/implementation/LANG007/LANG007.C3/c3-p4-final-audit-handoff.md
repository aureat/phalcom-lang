# LANG007.C3.P4 — Audit Remediation, Completion, Regression Repair, Evidence Collection, and Checkpoint Re-Closure

You are continuing implementation of:

```text
Program:    LANG007
Checkpoint: LANG007.C3
Plan:       LANG007.C3.P4
Repository: aureat/phalcom-lang
Branch:     main
```

A full architectural/post-implementation audit is attached to this request. Treat that audit as required implementation guidance. Do not merely respond to its findings in prose: investigate the repository, reproduce the reported failures, repair the implementation, complete unfinished P4 tasks and migrations, add missing regression coverage, collect durable evidence, and reconcile the entire lifecycle record.

The current P4 state must be treated as:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

until every required acceptance obligation has actually been satisfied.

The previous claim that P4 was finished is not authoritative. The repository state, the P4 plan, the checkpoint record, the attached audit, and fresh verification results are authoritative.

Do not weaken tests, hide failures, relabel regressions as baselines without predecessor evidence, introduce `Dynamic` as an escape hatch, add Tuple-specific solver exceptions, ignore source declaration identity, or mark the checkpoint complete merely because the original TupleIterator reproducer is green.

---

# 1. Primary objective

Finish LANG007.C3.P4 correctly.

This work unit must:

1. repair the remaining projection-inference soundness gap;
2. repair complete-pack canonicalization across internal and source-backed intrinsic projection identity;
3. investigate and fix the nine currently failing `phalcom-core` language tests unless clean-predecessor reproduction proves an individual failure is genuinely inherited;
4. investigate and correctly classify/fix the reported semantic integration failures;
5. complete any unfinished P4 T1–T12 implementation/migration obligations revealed by that investigation;
6. complete the repository corpus migration required by T11;
7. complete metadata/incremental/source-index/LSP/specification obligations required by T12;
8. re-audit native re-entry authority under the hostile non-local-return case;
9. add missing law-level and regression-level test coverage;
10. execute all required P4 focused and broad verification gates;
11. create durable evidence and classification records inside the C3 checkpoint directory;
12. create the proper P4 walkthrough and handoff only after implementation and verification genuinely satisfy the plan;
13. reconcile all lifecycle files to one non-contradictory final state.

The end state must be architectural closure, not merely a smaller failing-test count.

---

# 2. Create a dedicated audit-remediation evidence directory first

Within:

```text
docs/implementation/LANG007/LANG007.C3/
```

create:

```text
LANG007.C3.P4-audit-remediation/
```

This directory is part of the deliverable. It must contain durable implementation evidence rather than temporary scratch notes.

Create at least the following files:

```text
docs/implementation/LANG007/LANG007.C3/
└── LANG007.C3.P4-audit-remediation/
    ├── README.md
    ├── 01-projection-solver-soundness.md
    ├── 02-complete-pack-canonicalization.md
    ├── 03-core-runtime-regression-classification.md
    ├── 04-semantic-regression-classification.md
    ├── 05-native-reentry-authority.md
    ├── 06-p4-task-acceptance-ledger.md
    ├── 07-cv-coverage-ledger.md
    ├── 08-verification-command-ledger.md
    └── 09-residual-risk-and-deferred-work.md
```

You may add more files when useful.

Do not use these files as excuses to defer work that belongs to P4. They are evidence records for work performed.

## `README.md`

Record:

- starting revision;
- finishing revision;
- exact P4 state at start;
- attached audit as remediation authority;
- list of findings investigated;
- list of commits produced;
- links to every evidence file;
- final outcome.

## Evidence format

Every investigated failure or architectural issue should record, where applicable:

```text
Issue / test:
Current revision:
Predecessor revision(s):
Reproduction command:
Current result:
Predecessor result:
Classification:
Root cause:
Owning subsystem:
Files changed:
Architectural correction:
Regression tests added:
Post-fix result:
Acceptance impact:
```

Do not write “baseline” without an exact predecessor command/result or already-authoritative historical evidence that is still applicable.

---

# 3. P0 — Fix projection inference proof-state soundness

This is the highest-priority semantic issue.

The current solver correctly stopped reverse-inferencing subjects of non-injective projections, but it still conflates:

```text
constraint produced no substitutions
```

with:

```text
constraint has been proven
```

That is not acceptable.

Current patterns such as:

```rust
AssociatedProjection != AssociatedProjection
    => Ok(SolveEffect::Unchanged)

TypeMemberProjection vs anything unresolved
    => Ok(SolveEffect::Unchanged)

Positionals(P) == Positionals(Q)
    => Ok(SolveEffect::Unchanged)
```

must not allow an unresolved required equality to disappear as though satisfied.

## Required semantic model

The solver must distinguish at least:

```text
PROVEN
CHANGED/PROGRESS
DEFERRED/UNRESOLVED
REFUTED
```

You may implement this by evolving `SolveEffect`, by introducing a residual-obligation collection, or by another design that integrates naturally with the existing inference architecture.

Do not add a second solver.

Do not make type-member or trait-associated projections injective.

Do not solve:

```text
F<X> == F<Y>
```

by inferring:

```text
X == Y
```

unless the specific operation is independently defined and represented as injective.

### Required behavior

For a non-injective projection equality:

```text
Tuple<X>::Element == Tuple<Y>::Element
<X as Trait>::Item == <Y as Trait>::Item
Positionals(P) == Positionals(Q)
```

the solver must:

1. preserve child inference variables;
2. allow those variables to be solved by independent constraints;
3. periodically substitute current solutions;
4. forward-normalize/materialize the projection when possible;
5. prove equality if normalized results are equal;
6. refute if normalized results are provably unequal;
7. retain an unresolved obligation if normalization remains symbolic;
8. prevent `finish_frame` / `finish_root` from returning `Solved` while mandatory obligations remain unresolved.

A completely substituted variable graph is not sufficient for `Solved` if an unresolved projection relation remains.

## Required direct solver tests

Add law-level tests that exercise `InferenceSession` itself, not only `TypeStore`.

At minimum prove:

```text
1. TypeMemberProjection non-injectivity
   F<X> == F<Y>
   does not solve X == Y.

2. AssociatedProjection non-injectivity
   <X as Trait>::Item == <Y as Trait>::Item
   does not solve X == Y.

3. Pack lane non-injectivity
   Positionals(P) == Positionals(Q)
   does not solve P == Q.

4. Residual obligation blocks false success
   An unresolved projection equality cannot end as InferenceOutcome::Solved.

5. Delayed positive proof
   Independent constraints solve subjects;
   projections normalize to the same canonical result;
   the residual obligation is discharged.

6. Delayed negative proof
   Independent constraints solve subjects;
   projections normalize to unequal canonical results;
   inference becomes conflicting/refuted.

7. Projection against concrete type
   F<X> == Int cannot silently succeed merely because no reverse inference is possible.

8. No-variable residual case
   Two distinct symbolic projections with zero remaining InferVarIds still prevent false Solved.

9. Identical projection
   P == P remains immediately proven without creating a useless residual obligation.

10. Complete-pack identity lane, if represented as a projection:
    normalize forward rather than relying on a generic projection-injectivity exemption.
```

Also retain and rerun the TupleIterator regression proving:

```text
Tuple<***Es>
→ TupleIterator<***Es>
→ Iterator::Item
→ Tuple<***Es>::Element
```

without `Dynamic`.

Document the solver change in:

```text
LANG007.C3.P4-audit-remediation/01-projection-solver-soundness.md
```

---

# 4. P0 — Fix complete-pack canonicalization across source-backed identity

The new law:

```text
Complete(
    Positionals(P),
    Labeled(P)
) ≡ P
```

is correct.

Its implementation must not work only for projections with:

```rust
source_declaration: None
```

while failing for the canonical source-authored intrinsic members carrying:

```rust
source_declaration: Some(...)
```

P4 requires source-visible intrinsic member identity to remain source-owned.

## Required architectural correction

Recognize the canonical intrinsic operations by semantic member identity/evaluator ownership, not by the absence of source provenance.

The normalization rule should establish that the two lane projections are the canonical:

```text
CompleteTypePack::Positionals
CompleteTypePack::Labeled
```

operations over the same complete-pack subject.

It must not accidentally trigger for unrelated user members merely named:

```text
Positionals
Labeled
```

Preserve `source_declaration`; do not erase it globally or remove it from projection identity as a shortcut.

## Required tests

Add:

```text
internal Positionals(P) + internal Labeled(P)
    → P

source-backed Positionals(P) + source-backed Labeled(P)
    → P

canonical internal/source mixed formation
    → canonical semantic result where appropriate

Positionals(P) + Labeled(Q)
    → does not collapse

unrelated user-owned Positionals/Labeled names
    → do not trigger intrinsic law

source-visible projection retains canonical TypeMemberDeclarationId
```

Document formation paths and identity invariants in:

```text
02-complete-pack-canonicalization.md
```

---

# 5. Reproduce and classify all nine `phalcom-core` failures

The following tests currently fail:

```text
language::associated_traits::direct_field_delegation_executes_as_ordinary_accessors

language::conditional_conformance::cv50_slot_isolation_multiple_evidence_values
language::conditional_conformance::cv51_environment_reuse_repeated_invocations

language::derived_conformance::cv44_repeated_guarded_derived_calls_reuse_runtime_environments

language::generic_conformance::bounded_generic_function_dispatches_exact_witness_and_default
language::generic_conformance::multiple_type_parameters_with_distinct_traits_execute
language::generic_conformance::owner_and_method_local_evidence_execute_together
language::generic_conformance::runtime_conformance_environment_reuses_identical_environments
language::generic_conformance::same_trait_on_distinct_type_parameters_selects_distinct_witnesses
```

Do not assume these are baseline/downstream.

Repository history contains evidence that multiple tests were previously green.

## Required predecessor matrix

For every failed test, run the exact test at, at minimum:

```text
current main
0d6a6e9b parent: c3862536
earlier relevant revision: cce11d22
```

Use an earlier historically green conformance revision if necessary.

Record exact commands and results.

Classify every test as exactly one of:

```text
A. CURRENT REGRESSION
B. INTENTIONAL ACCEPTED MIGRATION REQUIRING FIXTURE/SPEC UPDATE
C. TRUE CLEAN-PREDECESSOR BASELINE
```

No vague “downstream” category.

## Runtime conformance environment audit

Audit all environment-bearing forwarding paths.

The VM already has explicit APIs such as:

```text
call_method_with_selector_as_and_environments
activate_captured_method_as_with_environment
```

but convenience wrappers frequently inject:

```text
RuntimeTypeEnvironmentId::EMPTY
RuntimeConformanceEnvironmentId::EMPTY
```

Audit at least:

```text
ordinary specialized call
InvokeSpecialized
InvokeTraitSelected
InvokeTraitRequirement
InvokeConditional
conditional family forwarding
bound method forwarding
bound family forwarding
derived conformance forwarding
module export forwarding
rest-method dispatch
captured method activation
dynamic/reflective gateway paths where semantic evidence must survive
```

A semantic forwarding seam must not discard an already-active evidence environment.

Pay particular attention to:

```rust
dispatch_conditional_family_entry
```

which currently routes through `activate_captured_method_as(...)`, and therefore defaults the conformance environment to `EMPTY`.

Also inspect:

```rust
Bytecode::InvokeConditional
```

and any path through the simple `call_method(...)` compatibility wrapper that should retain the current conformance/type environment.

## Environment invariants

Prove:

```text
one evidence slot maps to the intended requirement;
multiple same-trait type parameters remain slot-isolated;
owner-level and method-local evidence coexist;
nested selected/derived evidence survives forwarding;
repeated calls reuse interned identical environments;
different evidence sets do not alias;
default trait method calls retain required evidence;
bound/reified callables preserve captured evidence.
```

Add missing regression tests at the lowest useful layer plus language integration tests.

Document everything in:

```text
03-core-runtime-regression-classification.md
```

---

# 6. Investigate direct field delegation separately

Do not force:

```text
direct_field_delegation_executes_as_ordinary_accessors
```

into the same root cause if it is not.

Its reported symptom:

```text
<Counter instance> does not understand 'doubled'
```

suggests a possible regression in:

```text
trait default member installation
associated-trait witness publication
impl lowering
method table installation
default-callable inheritance
```

Trace the semantic callable identity through compilation and VM method-table installation.

Prove whether an empty:

```phalcom
impl Trait for Counter {}
```

correctly receives/dispatches a valid default requirement implementation when the trait contract permits it.

Add a focused regression that distinguishes:

```text
missing default method installation
wrong selector
wrong holder
wrong associated witness plan
lost environment
```

Do not “fix” this by adding generic DNU fallback behavior.

---

# 7. Reproduce, classify, and repair the semantic integration failures

The previous report identified approximately 40 failures. Re-run the current semantic integration suite and create the exact live failure list.

For every failure, compare with known-good historical records and/or clean predecessor revisions.

Populate:

```text
04-semantic-regression-classification.md
```

with a row for every failure.

Required columns:

```text
test
current result
known/predecessor result
classification
P4 task ownership
root cause
fix/migration
post-fix result
```

## 7.1 Pack syntax migration failures

P4 ratified explicit contribution syntax for pack parameters.

If a positive fixture is still using legacy syntax, migrate it to the final syntax.

Examples should use the accepted forms such as:

```phalcom
Foo<*Ts>
Foo<**Fs>
Foo<***Es>

Tuple<***(Int, String, name: Bool)>
Record<**{ name: String }>
```

Do not leave positive fixtures failing and label them baseline.

For obsolete syntax that should now fail, retain/add explicit negative tests asserting migration-quality diagnostics.

Be careful not to apply pack-expansion requirements to ordinary fixed generic parameters.

A test such as generic identity over:

```text
T: Type
```

must not regress because the pack parser/lowering path was applied too broadly.

## 7.2 Flow-analysis failures

Do not mechanically convert all `let` to `const`.

First establish whether the observed behavior is:

```text
regression
```

or an explicitly accepted semantic change.

Existing flow capability tests are law-level coverage and were previously green.

If opaque-call invalidation was broadened unintentionally, repair the implementation.

If a semantics change is genuinely accepted and belongs to current architecture:

1. document the accepted rule;
2. update the relevant specification;
3. preserve meaningful positive/negative flow coverage;
4. migrate fixtures intentionally;
5. test mutable locals separately rather than hiding the path with `const`.

Restore mutable iterator/collection tests where needed so iteration coverage does not only pass because locals were rewritten as immutable.

## 7.3 Workspace/declaration/editor failures

Investigate failures such as:

```text
invalid_generic_kind_does_not_publish_ready_declaration_header
editor_visible_symbols_use_prelude_policy_and_preserve_local_shadowing
```

These were historically green.

P4 directly owns publication/source-index/LSP behavior.

Repair regressions or explicitly migrate the semantics if an accepted design requires it.

Do not change expected symbol counts blindly.

## 7.4 Incremental failures

Tests such as:

```text
p3_mode_only_pack_binder_edit_invalidates_dependents_to_cold_parity
test_reexport_retargeting_invalidates_exact_consumer
```

must receive a real investigation.

For each:

1. compare cold vs incremental final semantic products;
2. distinguish correctness divergence from recomputation-count drift;
3. repair dependency/fingerprint invalidation when final states differ;
4. only update internal-count assertions if canonical semantic parity is preserved and the old count is no longer contractually relevant.

P4 requires cold/incremental convergence.

---

# 8. Re-open the P4 task ledger and complete every unfinished task

Audit all P4 tasks T1–T13 against the live repository, not against previous completion claims.

Create:

```text
06-p4-task-acceptance-ledger.md
```

with:

```text
Task
Plan acceptance requirement
Current implementation
Missing work
Tests/evidence
Final state
```

At minimum explicitly revisit:

```text
T1  lifecycle normalization
T2  structural kind syntax
T3  typed product-shape expansion syntax/AST
T4  semantic product-shape templates/domain specialization
T5  runtime specialization/shared behavior class/exact layouts
T6  declaration type-family/runtime projections
T7  source structural domains and Tuple/Record APIs
T8  Record/Map/Set syntax and row-tail/remainder migration
T9  Unit vs EmptyRecord runtime laws
T10 runtime ownership and aliases
T11 Universe/core/repository corpus migration
T12 metadata/incremental/source-index/LSP/specification persistence
T13 stabilization/certification/lifecycle closure
```

The previous checkpoint explicitly left T4/T5 acceptance incomplete. Do not skip them.

## T4/T5

Prove product-shape specialization and runtime layout behavior with the exact plan-required semantics.

Prove:

```text
specializations have exact payload layouts;
specializations do not manufacture one behavior class per specialization;
canonical nominal identity remains semantic;
runtime class ownership remains shared as designed.
```

Add missing positive and hostile tests.

## T8/T11 migration census

Run a repository census for stale forms.

Search for, classify, and migrate as appropriate:

```text
old Record opener/spelling
old Map literal spelling
old Set spelling
RecordRow public spelling where RecordFields is required
legacy Tuple/Record declaration surfaces
bare proper-type uses of higher-kinded Tuple/Record
legacy pack contribution syntax
known-shape Dynamic erasure
TupleEntries pseudo-domain if prohibited by final model
stale Unit runtime-class assumptions
stale empty-record-as-unit assumptions
```

Keep intentionally negative migration tests explicitly labeled.

Record the census and residual count.

## T12

Re-run and repair:

```text
metadata
schema compatibility
incremental
source index
editor/LSP
definition/navigation
rename where applicable
cold parity
```

Do not consider T12 complete while these surfaces are failing.

---

# 9. Re-audit native re-entry authority

The current activation-ID design is better than raw frame-count comparison, but the exact hostile case from the architectural ruling must be proven.

Create a test equivalent to:

```text
bytecode frame A
    ↓
native callback N
    ↓
N re-enters bytecode frame B
    ↓
B performs a non-local return/removal
while A remains live
    ↓
execution resumes in native N
```

At the point native N resumes:

```text
current_access_class/current_access_origin
```

must reflect N's native authority, not merely the last remaining bytecode frame.

Also test:

```text
nested native → bytecode → native → bytecode
private access restoration
protected/module/package authority where relevant
lexical closure authority
unwind/error path
frame slot reuse
fiber/switch path if the architecture permits re-entry there
```

If the activation-ID/re-entry owner stack is sufficient, document the proof and tests.

If not, retain a specific re-entered frame generation/token or equivalent stable activation identity.

Do not revert to raw `frames.len()` identity.

Record this in:

```text
05-native-reentry-authority.md
```

---

# 10. Pattern destructuring coverage

The new shared positional read performs:

```text
at(index)
→ unwrapOr(nil)
```

Add focused tests demonstrating that the new Option-returning collection contract integrates safely with destructuring.

At minimum:

```text
valid tuple element binding
valid list element binding

wrong exact Tuple arity
    → shape failure, not nil binding

too-short List fixed prefix
    → shape failure

valid actual nil payload
    → binds nil normally

nested Tuple/List destructuring
    → unwrap behavior remains correct
```

Do not remove the existing arity checks.

---

# 11. Preserve the restored Tuple iterator contract

Retain:

```phalcom
class TupleIterator<***Es: CompleteTypePack> {
    _source: Tuple<***Es>

    next -> Option<Tuple<***Es>::Element>
}

impl<***Es: CompleteTypePack> Iterator
    for TupleIterator<***Es>
{
    type Item = Tuple<***Es>::Element
}

impl<***Es: CompleteTypePack> Iterable
    for Tuple<***Es>
{
    type Item = Tuple<***Es>::Element
    type Iterator = TupleIterator<***Es>
}
```

Do not reintroduce:

```text
Dynamic
Object
bare Tuple
```

as convenience erasure for this path.

Add/retain direct proofs that for a concrete example:

```text
Tuple<Int, String>
```

the iterable/iterator `Item` normalizes to:

```text
Int | String
```

and that:

```text
<TupleIterator<Es> as Iterator>::Item
==
<Tuple<Es> as Iterable>::Item
==
Tuple<Es>::Element
```

---

# 12. Keep `Message.args` explicitly classified

The attached audit identifies an unresolved source/native contract mismatch:

```phalcom
Message.args -> Dynamic
```

versus native metadata describing:

```text
() -> Object
```

and stale comments saying `List`.

Investigate whether this is within the final accepted P4 boundary.

If it is required by P4's native-surface consistency work, fix it correctly.

If native metadata cannot currently express `Dynamic` without a broader schema change and the plan does not require this surface, do not opportunistically redesign the metadata language. Instead:

1. create a clear residual issue entry;
2. record exact source/native mismatch;
3. explain why it is outside P4;
4. ensure P4 closeout does not falsely claim it was fixed.

No silent omission.

---

# 13. Build a complete CV-01..CV-47 coverage ledger

Create:

```text
07-cv-coverage-ledger.md
```

It must enumerate every P4 coverage ID:

```text
CV-01
CV-02
...
CV-47
```

For every CV provide:

```text
requirement
test(s)/inspection artifact
command
result
notes
```

The plan requires real automated tests except where it explicitly permits representation-only inspection.

Do not use:

```text
"covered by code review"
```

for a missing required test.

If a CV currently lacks coverage, add the test before closure.

---

# 14. Verification protocol

Use the BUILD / STABILIZE / CERTIFY discipline from the P4 plan.

Do focused tests while repairing each subsystem.

Once implementation converges, execute the complete focused gate matrix required by the plan.

Then execute the broad closure exactly once on the final candidate revision.

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check --workspace --all-targets

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test --workspace

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo clippy --workspace --all-targets -- -D warnings

cargo fmt --all -- --check

git diff --check
```

Also run all P4-required focused semantic/core/parser/metadata/incremental/LSP/corpus commands from the plan.

Create:

```text
08-verification-command-ledger.md
```

For every command record:

```text
command
revision
date
pass/fail
test counts
failure names
classification
follow-up commit if repaired
final rerun result
```

Do not summarize a failed command as PASS because its failures were called “baseline.”

If an accepted clean-predecessor failure genuinely remains, record:

```text
overall command: FAILED
accepted inherited failures: [...]
clean-predecessor evidence: [...]
```

and apply the repository's actual lifecycle convention. Do not falsify the command result.

---

# 15. Lifecycle files must be reconciled definitively

After implementation and verification, audit and update all relevant lifecycle authorities.

At minimum:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-intrinsic-structural-data-semantic-completion-plan.md
```

Also create, only when justified:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-handoff.md
```

## Required consistency

There must be one truth.

It is forbidden to leave combinations such as:

```text
C3 = COMPLETE
P4 = IN_PROGRESS
```

or:

```text
checkpoint = UNVERIFIED
STATUS prose = complete
```

or:

```text
latest_completed_plan = P3.R1
while P4 is simultaneously declared complete elsewhere
```

If P4 fully satisfies acceptance:

```yaml
P4:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED   # or stronger only if lifecycle convention supports it

C3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED   # or exact justified state
```

Set:

```text
latest_completed_plan: LANG007.C3.P4
active_plan: none
```

only when that is factually true.

If any acceptance blocker remains, leave P4/C3 open and state the exact blocker.

Do not mark `RELEASE_COMPLETE` unless all required release certification gates meet the repository's definition.

---

# 16. P4 walkthrough requirements

`LANG007.C3.P4-walkthrough.md` must be a real execution record.

Include:

1. starting revision;
2. final revision;
3. commit list;
4. architecture changes;
5. each audit finding and disposition;
6. T1–T13 acceptance table;
7. CV-01..CV-47 table or link to the canonical evidence ledger;
8. complete failing-test predecessor classification;
9. migration census;
10. runtime conformance environment correction;
11. solver proof-state correction;
12. complete-pack canonicalization correction;
13. native re-entry hostile-case result;
14. metadata/incremental/LSP result;
15. broad verification command results;
16. known residual issues;
17. explicit statement of what remains outside P4.

Do not write a celebratory completion narrative that contradicts test output.

---

# 17. Handoff requirements

`LANG007.C3.P4-handoff.md` should be concise but exact.

If P4 closes, state:

```text
what was completed
what architectural invariants now hold
what was deliberately left out of scope
what known baselines remain, with evidence
what future work may proceed
```

If P4 does not close, the handoff must instead state:

```text
PARTIAL
remaining blocker(s)
exact reproduction
next action
```

Do not manufacture a successful handoff.

---

# 18. Do not weaken tests to obtain green status

Explicit prohibitions:

Do not:

```text
change an expected exact type to Dynamic;
change an expected evidence dispatch to ordinary dynamic dispatch;
change semantic equality to printed-name equality;
ignore source_declaration identity;
delete failing law-level tests;
replace mutable-path tests entirely with const-path tests;
skip incremental tests;
mark failed historical-green tests baseline without reproduction;
add Tuple-specific exceptions to generic conformance or inference;
make type-member projections injective;
split CompleteTypePack into fake independently inferred lane binders merely to satisfy one constructor call;
introduce a parallel projection evaluator;
introduce a second row/pack solver;
introduce AnyTuple/AnyRecord;
relax runtime type safety to make P4 tests pass.
```

A failing test that reveals an architectural bug must cause an implementation repair.

---

# 19. Required architectural invariants at completion

Before claiming P4 complete, explicitly prove:

```text
A. Tuple iterator specialization
Tuple<Es>
→ TupleIterator<Es>
→ Iterator::Item
→ Tuple<Es>::Element

B. Non-injective projection soundness
Projection result equality does not reverse-solve subjects.

C. Projection obligations
Unresolved projection equality cannot falsely produce Solved.

D. Complete-pack identity
Complete(Positionals(P), Labeled(P)) ≡ P
for canonical intrinsic source/internal formation.

E. Source member authority
Public intrinsic type members retain canonical Universe declaration identity.

F. Runtime evidence transport
Required trait/conformance environments survive every semantic forwarding seam.

G. Evidence isolation
Distinct evidence slots remain distinct; identical environments may be interned/reused safely.

H. Structural generic soundness
Whole CompleteTypePack variables remain one semantic variable, not independently fabricated lane variables.

I. No Dynamic erasure
Known Tuple/Record shapes retain computable source types.

J. Runtime specialization
Exact layouts may specialize, but behavior-class identity follows the P4 shared-class design.

K. Unit vs EmptyRecord
Semantic and runtime laws match the ratified P4 model.

L. Corpus migration
No accidental legacy syntax remains in positive first-party source.

M. Incremental parity
Cold and incremental final semantic products converge.

N. Tooling
Source index/LSP points to canonical authored intrinsic members and final source names.

O. Lifecycle
PROGRAM, STATUS, checkpoint, plan, walkthrough, and handoff contain one consistent state.
```

---

# 20. Commit discipline

Use coherent commits.

Suggested grouping:

```text
1. projection solver proof-state + tests
2. complete-pack/source-member canonicalization + tests
3. runtime conformance environment propagation + tests
4. direct-field/default associated trait regression
5. semantic migration/regression repairs
6. T4/T5/T8–T12 remaining completion work
7. native re-entry hostile-case correction/tests if needed
8. certification/lifecycle/evidence records
```

Adapt if dependencies require different grouping, but avoid one giant opaque commit.

Keep the working tree clean at final handoff.

---

# 21. Final report format

At the end, report:

```yaml
Status:
Completion:
Verification:
Final revision:
Working tree:
```

Then provide:

## Implemented

Exact architectural changes.

## Regression classification

Counts of:

```text
fixed current regressions
intentional migrations completed
true predecessor baselines
remaining unresolved failures
```

## P4 task ledger

T1–T13 final states.

## Coverage

CV-01..CV-47 status.

## Verification

Exact commands and results.

## Lifecycle

Files updated/created and final C3/P4 state.

## Residual issues

Only genuinely out-of-scope or clean-predecessor issues, each with evidence.

If any mandatory P4 acceptance item is still incomplete, report `PARTIAL`. Do not call the work complete.

The objective of this implementation pass is not to defend the previous completion claim. The objective is to make that claim true—or, if repository evidence proves an unavoidable blocker, to leave the lifecycle state accurately open with the blocker completely documented.