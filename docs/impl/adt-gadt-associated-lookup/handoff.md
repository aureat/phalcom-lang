# Phalcom ADT/GADT Completion — Fresh-Agent Handoff

You are taking over an in-progress repository-grounded compiler remediation task for the programming language **Phalcom**.

Your assignment is to **verify, debug, complete, test, and merge the already-landed ADT/GADT remediation work**.

Most of the planned implementation has already landed. The repository is currently not fully green. You must diagnose the actual failing tests, repair the implementation at the root cause, complete any missing pieces, run the full relevant verification matrix, and merge the work when everything is clean.

Do not merely report failures. Complete the work.

---

# 1. Primary Objective

Start from the existing remediation branch:

```text
fix/adt-remediation-completion
```

All implementation work must remain on this **single branch**.

Do not create additional feature/fix branches.

A draft PR was previously created as:

```text
PR #12
```

It was created primarily to obtain an executable GitHub Actions environment when the previous agent did not have a locally mounted repository.

Use the existing branch and PR if they still exist and are appropriate.

When the implementation is complete and verification is green:

1. review the final diff,
2. ensure the branch contains only intended changes,
3. merge it,
4. verify the merged state.

Do not merge before the verification gates pass.

---

# 2. Start Here

The first required command is:

```bash
cargo test -p phalcom-core --test core adt
```

This was explicitly requested as the initial debugging target.

Do not start by running the entire workspace and drowning in unrelated output.

The preferred debugging sequence is:

```bash
cargo test -p phalcom-core --test core adt
```

Then isolate individual failing ADT tests as necessary:

```bash
cargo test -p phalcom-core --test core <specific_test_name> -- --exact --nocapture
```

or the repository-appropriate equivalent.

Fix failures one root cause at a time.

After the focused ADT suite is green, broaden verification progressively.

---

# 3. Important Previous Execution Finding: Rust Flags / Test Harness

The previous agent initially attempted to reproduce the focused Cargo test through a temporary CI executor.

One apparent Cargo failure was **not an ADT failure**.

The repository/environment supplied:

```text
-Zthreads=2
```

to Rust.

That is an unstable rustc option.

Forcing the wrong stable-toolchain environment caused rustc itself to reject the invocation and Cargo returned status 101.

Therefore:

> Do not interpret every Cargo status 101 as an ADT test failure until you inspect the actual compiler/test output.

If you encounter a failure mentioning `-Zthreads=2`, unstable options, or toolchain incompatibility, first reproduce the repository's intended CI/toolchain environment.

The previous agent experimented with clearing inherited flags:

```bash
RUSTFLAGS=""
```

when constructing the temporary diagnostic executor.

You should instead prefer the repository's native toolchain and scripts/configuration wherever possible.

Inspect:

```text
rust-toolchain*
.cargo/config*
.github/workflows/*
```

before inventing a custom Rust execution environment.

If the repository runs correctly in its normal environment, do not remove project-wide flags merely to make one custom runner work.

---

# 4. Previous Work Already Performed

Before semantic debugging began, the previous agent:

- established the single branch `fix/adt-remediation-completion`,
- opened draft PR #12,
- confirmed that the landed repository could compile far enough for CI to proceed,
- observed Clippy failures that appeared largely mechanical,
- ran repository-native formatting,
- committed formatting changes on the same branch,
- diagnosed the `-Zthreads=2`/toolchain issue described above,
- began building a more reliable focused-test diagnostic path.

The previous agent **did not** complete the semantic/runtime fixes.

Do not assume the ADT implementation was fixed simply because formatting landed.

No final merge was performed.

---

# 5. Architectural Semantics You Must Preserve

This section is authoritative for the remediation.

Do not "fix" failing tests by reverting these decisions.

## 5.1 `::` has two resolution layers

Phalcom already has a general receiver-bound `::` mechanism for behavior.

Examples:

```phalcom
instance::bar
instance::bar::(...)
Foo::bar
Foo::bar::(_)
Foo::bar::()
```

Ordinary behavioral `::` is:

- receiver-bound,
- deferred,
- applicable to ordinary instances, objects, and class objects,
- statically enumerable because Phalcom does not support monkey patching,
- dynamically invoked against the captured receiver.

A class object such as:

```phalcom
Foo
```

is itself the receiver in:

```phalcom
Foo::bar
```

Behavior callable on the class object is declared using Phalcom's class-method attribute:

```phalcom
@class
```

Do **not** use or introduce a `static` keyword.

Correct:

```phalcom
@class
Some(_ a, _ b) { ... }
```

if such a method were otherwise legal.

Incorrect syntax:

```phalcom
static Some(_ a, _ b) { ... }
```

---

## 5.2 Associated members are distinct from behavior

On top of ordinary receiver-bound behavioral `::`, classes can expose **associated members**.

Associated members are **not behavior**.

They form a distinct declaration-owned lookup layer.

For ADTs, enum variants are associated members.

Example:

```phalcom
enum Option<T> {
    @variant Some(value: T)
    @variant None
}
```

Conceptually the class object exposes:

```text
Option associated surface
    Some -> variant-constructor family
    None -> singleton variant
```

Thus:

```phalcom
Option::Some::(_)
```

means:

> Resolve associated base `Some`, then select the exact unary constructor shape.

And:

```phalcom
Option::None
```

means:

> Resolve the singleton associated variant value `None`.

Do not describe `Option::Some` itself as retrieving a singleton. `Some` is a constructor family.

---

## 5.3 Associated lookup has precedence

Resolution should conceptually behave as:

```text
receiver::name / receiver::selector
    |
    +-- associated member exists for selector base?
    |       |
    |       +-- YES -> resolve strictly in associated namespace
    |
    +-- otherwise -> ordinary receiver-bound behavioral :: lookup
```

Associated members take precedence over behavior.

---

## 5.4 Associated names reserve the entire selector base

Reservation is by **base**, not exact shape.

If an associated member named:

```text
Some
```

exists in a declaration, the entire `Some` base is reserved.

Therefore an enum containing:

```phalcom
@variant Some(value: T)
```

must reject competing behavioral declarations such as:

```phalcom
@class
Some(_ a, _ b) { ... }
```

Even though the callable shape differs, `Some` is already owned by the associated namespace.

There must be no behavior/associated overload merging under one base.

This is a critical invariant.

---

# 6. Previously Verified Repository Findings

The following defects were found during the earlier remediation analysis.

Some may already have been fixed by the implementation that landed after the plan was written.

You must **verify each against the current branch** rather than blindly reimplementing them.

Treat this as a checklist of likely root causes.

---

## 6.1 Enum requirements existed but were disconnected from production

Previously observed in:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/enum_requirements.rs
```

The production session path called the enum requirement checker with empty inputs conceptually equivalent to:

```rust
check_enum_requirements(..., &[], &HashMap::new(), ...)
```

and then published empty requirements.

Meanwhile:

```text
phalcom-semantic/src/enum_requirements.rs
```

already contained substantial real checking machinery for:

- requirement completeness,
- missing implementations,
- parameter shape,
- rest parameters,
- parameter type equality under case substitution,
- return subtype compatibility.

The direct unit tests could therefore pass while source-level enum requirements remained a no-op.

### Required verification

Confirm whether normal source enum declarations now produce canonical requirement products and case implementations before invoking the checker.

Regression tests must begin from source declarations / normal semantic sessions, not only hand-construct semantic inputs.

---

# 7. Canonical Callable Signatures

Previously observed:

```text
phalcom-semantic/src/checker/declaration_signature.rs
```

already provides canonical declaration-owned AST-to-semantic callable signature construction.

Enum behavior must reuse or share that machinery.

Do not create a second, subtly different parser for:

- parameter labels,
- positional parameters,
- rest parameters,
- generic parameters,
- return types,
- getter/setter/index shapes,
- selectors.

Avoid ad hoc signature reconstruction in:

```text
session.rs
```

or enum-specific paths.

One semantic declaration model should feed both ordinary declarations and enum behavior.

---

# 8. Enum Declaration Publication

Previously observed:

```text
phalcom-semantic/src/checker/enum_declaration.rs
```

primarily built structural variant metadata and skipped non-variant enum members.

This meant enum-root and/or case behavior products could fail to become canonical semantic declarations.

Verify current implementation covers:

- enum-root declaration-only requirements,
- enum-root bodyful defaults,
- case-local behavior,
- exact callable identities,
- override/requirement checking,
- legal callable kinds.

Do not rebuild this information later in `phalcom-core`.

The semantic layer must remain authoritative.

---

# 9. Exact Variant Identity Must Be Preserved

Previously observed in:

```text
phalcom-core/src/compiler/lib/enum_decl.rs
```

case behavior attachment used a base-only search resembling:

```rust
.find(|vs| vs.id.selector.base == SelectorBase::Named(v.name.clone()))
```

This is incorrect for overloaded variants sharing a base.

Example conceptual family:

```phalcom
@variant Foo(_ x: Int)
@variant Foo(_ x: String, _ y: Int)
```

Case behavior must attach using the exact:

```text
VariantId
```

or exact canonical selector identity.

Never find a case by selector base alone.

Add a regression test with multiple variants sharing the same base but distinct shapes.

---

# 10. Standalone Variant Selector Construction

Previously observed in:

```text
phalcom-core/src/compiler/lib/enum_decl.rs
```

a fallback path synthesized a variant selector by treating every local payload parameter name as an external selector label.

Conceptually:

```rust
for p in payload.parameters {
    slots.push(SelectorSlot::Label(p.name.clone()));
}
```

This is incorrect.

Use the canonical AST selector helper:

```text
phalcom_ast::selector::selector_from_variant(...)
```

or its current canonical equivalent.

Selector identity must be defined once.

---

# 11. Enum Behavior Compilation Was Incomplete

Previously observed in:

```text
phalcom-core/src/compiler/lib/enum_decl.rs
```

The compiler handled only some case behaviors, notably methods/getters, while other legal behavioral forms were omitted.

Verify support for the full currently legal enum behavior surface, including whatever the language supports among:

- method,
- getter,
- setter,
- index.

Also verify enum-root bodyful behavior is compiled.

Do not silently convert declaration-only behavior into executable empty bodies.

If case-local declaration-only behavior is illegal under current language rules, reject it semantically before lowering.

---

# 12. Case-Local Behavior Rules

Previously ratified design:

- case-local behavior is instance-side in the current language version,
- case-local `@class` behavior is not allowed,
- case-local declaration-only requirements are not allowed,
- enum-root declaration-only members are requirements,
- enum-root bodyful members act as shared/default behavior.

Verify these invariants remain enforced.

Do not infer legality merely from what the compiler happens to compile.

The semantic checker should reject illegal declarations.

---

# 13. Associated Surface Conflated Behavior and Variants

Previously observed in:

```text
phalcom-semantic/src/associated.rs
phalcom-semantic/src/checker/associated.rs
```

The associated surface builder grouped variants and behavioral callable members by selector base.

It could diagnose a conflict yet still publish a mixed family.

This is incompatible with the now-authoritative model.

Associated members and behavior are distinct namespaces.

### Required end state

Associated surface:

```text
variants / explicitly associated members only
```

Behavioral surface:

```text
ordinary receiver-bound behavior
```

Resolution:

```text
associated base first
behavior fallback second
```

If the same base is used by both categories in the same declaration, diagnose the declaration and do not publish an incoherent mixed family.

Do not "solve" this by making all `::` class-side.

---

# 14. Current Associated Resolver Was Type-Form-Centric

Previously observed:

```text
phalcom-semantic/src/checker/associated.rs
```

`resolve_associated_owner` required something like:

```rust
SemanticDenotation::TypeForm(...)
```

This reflects the associated layer, not the entire semantics of `::`.

Be careful when modifying this code.

Associated lookup can legitimately require a class/type-associated owner, but ordinary behavioral `::` remains receiver-bound and works on any receiver.

The correct architecture is layered resolution, not one resolver that probes every dispatch side.

---

# 15. Associated Resolution Probed Behavioral Dispatch Sides

Previously observed:

```text
resolve_effective_associated_family
```

walked associated structures while also probing:

```text
DispatchSide::Class
DispatchSide::Instance
```

for behavioral callables.

That conflates associated members and behavior.

Under the correct semantics:

```text
associated lookup
```

should not manufacture a behavioral family.

Once associated lookup misses, the normal receiver-bound behavioral family machinery may run.

Keep the two concepts separate.

---

# 16. Unknown Constructor Parameter Types Were Fabricated as Unit

Previously observed in:

```text
phalcom-semantic/src/checker/associated.rs
```

constructor specialization did something conceptually equivalent to:

```rust
declared_type
    .canonical_type()
    .unwrap_or_else(|| ctx.store.unit())
```

This is semantically invalid.

An unknown/noncanonical type does not become `Unit`.

Likewise, do not "fix" this by fabricating `Object`.

Preserve unknownness or block specialization with a proper semantic diagnostic/invariant result.

Never create fake type evidence.

---

# 17. Associated Family Conflict Publication

Previously observed in:

```text
phalcom-semantic/src/associated.rs
```

On a variant/behavior base conflict, the implementation could:

1. emit an `EnumFamilyCategoryConflict`,
2. still publish a family,
3. include mixed variant + behavioral members.

That leaves downstream consumers with contradictory semantic state.

On an illegal category conflict:

- diagnose,
- omit the invalid family, or
- publish an explicit poisoned/error semantic product if the architecture supports one.

Do not publish a valid-looking mixed family.

---

# 18. Pattern Semantics: Record/Map Fell Through to Wildcard

Previously observed:

```text
phalcom-semantic/src/checker/pattern.rs
```

The semantic pattern resolver explicitly handled several patterns, then ended with a catch-all equivalent to:

```rust
_ => (PatternResolution::Wildcard, expected_space.clone())
```

AST patterns included Record/Map forms.

Meanwhile downstream/runtime code already understood Record/Map matching.

That caused legal refutable patterns to behave semantically as wildcards.

Consequences include incorrect:

- exhaustiveness proofs,
- redundancy/usefulness results,
- bindings,
- diagnostics.

Verify whether Record/Map semantic pattern handling has now landed.

If not, implement proper semantic resolution.

Do not merely special-case tests.

---

# 19. Pattern-Space Algebra Lacked Record/Map Representation

Previously observed:

```text
phalcom-semantic/src/checker/pattern_space.rs
```

Pattern space included concepts such as:

```text
Empty
Opaque
Union
Variant
Tuple
List
```

but not Record/Map.

If Record/Map semantics are now implemented elsewhere, verify the space algebra is still sound.

Treating a refutable Record/Map pattern as the entire expected space is also unsound.

Use a sound explicit representation or a conservative predicate/opaque model that cannot falsely prove exhaustiveness.

---

# 20. Runtime Already Implemented Record/Map Pattern Matching

Previously observed:

```text
phalcom-core/src/compiler/lib/patterns.rs
```

Runtime/compiler matching already had actual Record and Map tests.

Therefore the semantic layer should not reject or wildcard these patterns merely because the runtime path is ahead.

Semantic analysis and executable behavior must agree.

---

# 21. Pattern-Space Exact Specialization / GADT Compatibility

Previously observed in:

```text
phalcom-semantic/src/checker/pattern_space.rs
```

Variant intersection/subtraction primarily checked:

- same VariantId,
- same field count,
- field spaces.

This can be insufficient for exact generic case specialization.

Example conceptual issue:

```text
Some<Int>
Some<String>
```

must not necessarily intersect merely because they share the same nominal VariantId.

Pattern-space algebra must respect exact enum specialization and relevant proof bindings.

Also inspect proof-binding merge behavior: previously bindings could overwrite one another through simple map insertion instead of detecting incompatible equalities.

Conflicting proof constraints must not silently overwrite.

---

# 22. Exhaustiveness Must Fail Closed on Missing Metadata

Previously observed:

```text
phalcom-semantic/src/checker/exhaustiveness.rs
```

Closed-enum expansion used a `filter_map`-style operation that silently dropped variants whose metadata was unavailable.

That is unsound.

If an enum declaration says there are N variants but semantic metadata for one variant is missing, the exhaustiveness prover must not reason as though that case does not exist.

Preferred behavior:

```text
missing metadata -> proof blocked / Opaque / internal semantic error
```

Never:

```text
missing metadata -> silently omit possible values
```

ExactCase missing metadata was already handled more conservatively; preserve that behavior.

---

# 23. GADT Equality Solver Lacked Record Equality

Previously observed:

```text
phalcom-semantic/src/checker/gadt_proof.rs
```

The equality-based GADT solver was structurally correct in direction: GADT case constraints should produce equality proofs rather than simply subtype-filtering cases.

However `unify_equality` previously handled:

- parameter,
- applied,
- exact case,
- union,
- tuple,
- callable,

but not Record.

Verify whether this is fixed.

Phalcom has row-aware record types, so record equality must respect the row model rather than naïvely zipping field arrays.

Relevant row structures were found in:

```text
phalcom-semantic/src/types/row.rs
```

including:

```text
RecordRowData
RecordRowTail::Closed
RecordRowTail::Parameter(...)
```

Use the canonical row/type machinery.

---

# 24. GADT Substitution Used an Arbitrary 64-Pass Ceiling

Previously observed in:

```text
phalcom-semantic/src/checker/gadt_proof.rs
```

substitution normalization repeatedly applied substitutions up to 64 iterations, then returned the current result.

This is not a principled normalization algorithm.

Replace/verify replacement with graph-aware or recursively memoized normalization using states such as:

```text
unvisited
visiting
resolved
```

A cycle should be handled explicitly rather than hidden behind "64 was probably enough."

Do not let a large but valid substitution chain fail because of an arbitrary constant.

---

# 25. Executable Lowering Lost Metadata / Failed Open

Previously observed in:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

Several bad fallback patterns existed.

Examples included conceptually:

```rust
missing variant metadata -> VariantShape::Singleton
```

and:

```text
missing binding -> index 0
```

and unchecked narrowing conversions.

These are dangerous because missing semantic facts become plausible executable values.

Preferred rule:

> Lowering should be mechanical projection of canonical semantic products and should fail closed when required information is missing.

Do not invent:

- singleton shape,
- slot zero,
- default arity,
- default selector,
- default rest mode.

Return a lowering error/invariant failure instead.

---

# 26. Rest Metadata Was Erased

Previously observed in:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

associated/executable projection repeatedly hardcoded:

```rust
rest_mode: ExecutableRestMode::None
```

even when semantic callable shapes contained rest information.

Verify whether this landed fix is complete.

Preserve:

```text
none
positional
labeled
complete
```

or the current equivalent.

The runtime should not rediscover source-level signature information later.

---

# 27. Unchecked Narrowing Casts

Previously observed across enum lowering/runtime code:

```text
usize -> u16/u32/etc
```

using unchecked `as` conversions.

Examples may include:

- variant discriminants,
- field indices,
- parameter counts,
- slots,
- arities.

Replace relevant casts with checked conversion:

```rust
try_from(...)
```

and emit the repository-appropriate compilation/lowering/invariant diagnostic when the value cannot be represented.

Do not silently truncate.

---

# 28. VM Declaration-Class Lookup Used Global Leaf-Name Fallback

Previously observed in:

```text
phalcom-core/src/vm/associated.rs
```

`resolve_declaration_class` checked proper declaration/registry information and eventually performed a global scan by leaf class name.

That is unsound for module-qualified declarations.

A declaration such as:

```text
module_a::Foo
```

must not resolve to:

```text
module_b::Foo
```

merely because both have leaf name `Foo`.

Remove or verify removal of this fallback.

Resolution should use canonical declaration identity.

---

# 29. Runtime Associated Binding Reflected the Old Conflated Model

Previously observed in:

```text
phalcom-core/src/vm/associated.rs
```

behavioral associated binding attempted class/metaclass lookup and then fallback logic.

Reassess this code under the corrected semantics.

There should not be a runtime concept where "associated behavioral target" ambiguously probes class-side/instance-side members.

Conceptually:

```text
associated member resolution
    -> associated capability/value/constructor

ordinary :: behavior
    -> bound receiver + statically determined behavioral family
```

The runtime should execute semantic/lowering decisions, not re-run namespace policy heuristically.

---

# 30. Part 05.2 and Part 06 Already Landed

The repository changed materially after the original remediation plan.

Parts **05.2 + 06** have landed.

Therefore:

> Re-audit current `main` / current remediation branch before assuming any old finding still exists.

Part 05.2 apparently added more match/executable lowering infrastructure.

Part 06 added broader native ADT/reflection/integration work.

Do not revert those features while fixing old issues.

Treat them as preservation constraints.

Add integration tests where necessary to make sure remediation fixes do not break:

- reflection,
- native ADTs,
- exact case types,
- matching,
- GADT proofs,
- executable lowering,
- runtime enum representation.

---

# 31. ADT Representation / Identity Invariants

Preserve the existing identity architecture where possible.

Previously observed semantic identities include concepts equivalent to:

```text
VariantId { owner, selector }
VariantFamilyId
AssociatedFamilyId
VariantFieldId
VariantConstructorId

CallableOwnerId
    Declaration
    Variant

CallableId
    owner
    selector
    side

InvocationTargetId
    Behavioral
    VariantConstructor
```

This is a good basis.

Avoid collapsing distinct variant constructors into a single callable identity.

Variant identity is exact-selector-based.

Exact case types should remain canonical conceptually as:

```text
ExactCase(VariantId, EnumSpecialization)
```

or the repository's current equivalent.

---

# 32. Singleton vs Nullary Constructor Must Remain Distinct

Phalcom distinguishes:

```phalcom
@variant None
```

from:

```phalcom
@variant None()
```

The former is a singleton variant.

The latter is a zero-argument constructor that can create distinct values according to constructor semantics.

Do not collapse them during associated lookup, lowering, reflection, or runtime representation.

In particular:

```phalcom
Option::None
```

may be the singleton variant.

A constructor-family member is not automatically a singleton just because it has zero payload fields.

---

# 33. Reflection Must Preserve Exact Types

Reflection should be capable of observing exact specialized case types where the architecture supports it.

Do not erase:

```text
Some<Int>
```

to merely:

```text
Some
```

if the semantic type is an exact specialized case.

This matters for language features such as typed dispatch and exact case reasoning.

---

# 34. Implementation Architecture

The preferred architecture remains:

```text
parse
  ->
canonical semantic declaration products
  ->
canonical enum behavior / requirement products
  ->
canonical match + exact-case + GADT proofs
  ->
lossless executable lowering
  ->
mechanical runtime execution
```

Key principle:

> `phalcom-semantic` is the semantic authority.

Avoid semantic reconstruction in:

```text
phalcom-core
```

The core/compiler/runtime should consume already-decided semantic identities and executable metadata.

Do not duplicate language rules in multiple layers.

---

# 35. Debugging Discipline

Use systematic debugging.

For each failing test:

1. reproduce it,
2. read the full failure/panic/diagnostic,
3. identify the earliest incorrect semantic/executable state,
4. inspect the producer of that state,
5. write or strengthen a regression test,
6. make the smallest architectural fix that restores the invariant,
7. rerun that test,
8. rerun the focused ADT group.

Avoid editing multiple unrelated subsystems before rerunning.

Do not modify expected output merely to make a regression disappear unless the language semantics themselves intentionally changed.

---

# 36. Prefer Source-Level Regression Tests

A recurring weakness in the previous implementation was testing internal helpers directly while leaving the production pipeline unwired.

Therefore, for semantic features, prefer tests that begin with actual Phalcom source.

For example, do not only test:

```rust
check_enum_requirements(hand_built_data)
```

Also test:

```text
Phalcom source
 -> parse
 -> semantic session
 -> published semantic product / diagnostic
```

Likewise for runtime tests:

```text
Phalcom source
 -> semantic lowering
 -> compile
 -> execute
 -> observed result
```

The goal is to test the complete production path.

---

# 37. Important Regression Tests to Ensure Exist

At minimum, verify or add coverage for the following.

## Enum requirements

```text
root declaration-only requirement
case provides compatible implementation
case misses required implementation
wrong parameter labels/shape
wrong rest shape
wrong exact parameter type under specialization
invalid return type
```

---

## Root/default behavior

A bodyful root behavior should be shared/inherited appropriately by variant instances.

---

## Exact case behavior identity

Two variants with the same selector base but different exact shapes must attach and dispatch the correct case-local behavior.

---

## Illegal case behavior

Reject:

```text
case-local declaration-only behavior
case-local @class behavior
```

if those remain prohibited by the language design.

---

## Behavior kinds

Exercise legal:

```text
method
getter
setter
index
```

where supported.

---

## Associated-member reservation

Given:

```phalcom
enum Option<T> {
    @variant Some(value: T)
}
```

a competing:

```phalcom
@class
Some(_ a, _ b) { ... }
```

must fail because the **base `Some` is reserved**.

Do not allow it merely because the shapes differ.

---

## Associated precedence

If an associated base exists, associated lookup wins over ordinary behavior.

Behavior fallback occurs only when no associated member owns the base.

---

## Exact constructor-family selection

Exercise:

```phalcom
Option::Some::(_)
```

and multiple same-base constructor shapes if the language supports them.

The selected result must preserve exact selector identity.

---

## Singleton variant

Exercise:

```phalcom
Option::None
```

as an associated singleton value.

---

## Zero-arg constructor distinction

Ensure:

```phalcom
@variant Marker()
```

does not become the same thing as:

```phalcom
@variant Marker
```

---

## Bound behavioral `::`

Do not regress ordinary semantics:

```phalcom
instance::bar
instance::bar::(...)
Foo::bar
Foo::bar::(_)
Foo::bar::()
```

The captured receiver should be evaluated once.

---

## Class-method syntax

Use:

```phalcom
@class
```

in all class-method regression fixtures.

Do not introduce `static`.

---

## Record/Map patterns

Test:

```text
refutability
bindings
usefulness/redundancy
non-exhaustiveness
exhaustiveness where provable
runtime agreement
```

---

## Exact specialization

Ensure pattern spaces distinguish incompatible exact cases such as:

```text
Some<Int>
Some<String>
```

when appropriate.

---

## Proof conflicts

Conflicting GADT equalities must not be silently overwritten.

---

## Missing metadata fail-closed

Exhaustiveness/lowering must never prove/execute successfully by silently omitting required enum metadata.

---

## Record equality in GADTs

Exercise row-aware record equality and relevant open/closed row behavior.

---

## Deep substitution chains

Add a test beyond the previous arbitrary 64-step threshold if the old algorithm still exists.

---

## Rest preservation

Exercise positional/labeled/complete rest metadata through:

```text
semantic signature
 -> lowering
 -> runtime invocation
```

---

## Qualified class lookup

Two modules containing classes with the same leaf name must not cross-resolve.

---

## Large arity / checked conversions

Where practical, directly test conversion failure/invariant handling instead of requiring impossible source-size fixtures.

---

# 38. Clippy / Formatting

The previous run exposed Clippy issues in newly landed code.

Do not ignore them.

After semantic correctness is established, clean all warnings required by the repository's CI policy.

Use repository-native formatting.

Expected final gates include:

```bash
cargo fmt --all -- --check
```

and likely:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Adjust only to the repository's actual CI command if it differs.

Do not globally silence warnings to obtain a green run.

---

# 39. Progressive Verification Matrix

Run progressively.

## Gate A — focused ADT core tests

```bash
cargo test -p phalcom-core --test core adt
```

Must be green before broadening.

---

## Gate B — relevant semantic ADT/match/GADT tests

Run the focused semantic modules covering:

```text
enum declarations
requirements
associated members
patterns
exhaustiveness
GADT proofs
exact cases
```

Use repository test names/modules rather than guessing if organization changed.

---

## Gate C — phalcom-core package

```bash
cargo test -p phalcom-core
```

---

## Gate D — phalcom-semantic package

```bash
cargo test -p phalcom-semantic
```

or the repository's appropriate command.

---

## Gate E — workspace

```bash
cargo test --workspace
```

If the repository has known intentionally excluded/feature-gated packages, follow CI configuration rather than arbitrarily modifying the command.

---

## Gate F — formatting

```bash
cargo fmt --all -- --check
```

---

## Gate G — Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

or the exact CI equivalent.

---

## Gate H — Miri / special runtime lanes

Inspect CI and run applicable Miri or runtime-safety tests if the repository defines them.

Do not invent a Miri lane that the project does not support.

If a lane cannot run locally for environmental reasons, ensure the remote CI equivalent passes.

---

# 40. Final Diff Review

Before merge, inspect:

```bash
git diff main...HEAD
git status
git log --oneline main..HEAD
```

Verify:

- only intended files changed,
- no temporary diagnostic workflow remains unless genuinely useful and approved,
- no captured test-output artifacts remain,
- no debugging prints remain,
- no accidental generated files remain,
- no unrelated formatting avalanche remains beyond the formatting commit already made,
- no semantics were weakened merely to satisfy tests.

If the temporary draft-PR CI harness introduced files solely for debugging, remove them before merge unless they improve repository CI in a deliberate way.

---

# 41. Merge Requirement

The user explicitly requested:

> Work only on a single branch, merge at the end.

Therefore the job is not complete at "PR is green."

Once all required verification passes:

1. ensure the branch is current with the intended base,
2. perform the merge using the repository's normal workflow,
3. verify the merged commit/branch state,
4. report the resulting commit SHA,
5. report the test evidence from the final code.

Do not create another branch for cleanup or merge preparation.

---

# 42. What Not to Do

Do not:

- replace `@class` with a nonexistent `static` syntax,
- redefine all `::` as static/class-side lookup,
- merge associated members and behavior into one overload family,
- allow an associated base to coexist with behavioral shapes under that same base,
- describe a constructor family such as `Some` as a singleton,
- collapse `VariantId` identity to selector base,
- fabricate `Unit` or `Object` for unknown types,
- default missing executable metadata to plausible values,
- silently omit variants from exhaustiveness proofs,
- use leaf class names as a substitute for canonical declaration identity,
- rebuild semantic signatures in the compiler/runtime,
- blindly update tests to match broken output,
- merge before the full relevant verification matrix passes.

---

# 43. Definition of Done

The task is complete only when all of the following are true:

- [ ] Work remained on `fix/adt-remediation-completion`.
- [ ] `cargo test -p phalcom-core --test core adt` passes.
- [ ] Every focused ADT/GADT semantic failure discovered during debugging has a root-cause fix.
- [ ] Regression tests cover each repaired semantic/runtime invariant.
- [ ] Enum requirements work through the production source/session pipeline.
- [ ] Exact variant identity is preserved.
- [ ] Enum root/default/case behavior is correctly published and compiled.
- [ ] Associated members are distinct from behavior.
- [ ] Associated lookup takes precedence.
- [ ] Associated bases reserve the entire selector base.
- [ ] `@class` behavior remains ordinary class-object behavior.
- [ ] Ordinary receiver-bound `::` semantics remain intact.
- [ ] Record/Map pattern semantics agree with runtime behavior.
- [ ] Exhaustiveness fails closed.
- [ ] Exact specialization/GADT proof behavior is sound.
- [ ] Lowering does not fabricate missing metadata.
- [ ] Rest/selector/identity information survives lowering.
- [ ] Module-qualified runtime identities resolve correctly.
- [ ] Checked conversions replace dangerous narrowing in affected ADT paths.
- [ ] Part 05.2 and 06 functionality remains green.
- [ ] `cargo test -p phalcom-core` passes.
- [ ] `cargo test -p phalcom-semantic` passes where applicable.
- [ ] `cargo test --workspace` passes.
- [ ] formatting passes.
- [ ] Clippy passes under CI's warning policy.
- [ ] applicable Miri/special lanes pass.
- [ ] final diff contains no temporary diagnostics.
- [ ] branch is merged.
- [ ] merged state is verified.

---

# 44. Expected Final Report

When finished, report concisely but concretely:

## Root causes

For each failure cluster:

```text
failing test(s)
 -> incorrect state
 -> root cause
 -> files changed
 -> invariant restored
```

## Implementation

List the substantive changes by subsystem.

## Regression coverage

List the new/strengthened tests.

## Verification

Include the exact commands and final results.

Do not report only "all tests pass"; give the relevant commands.

## Git result

Report:

```text
branch
final branch commit
merge commit / resulting main SHA
PR state
```

## Remaining issues

If anything genuinely remains blocked, describe it explicitly.

Do not label the task complete if a required gate is still red.

---

# 45. Guiding Principle

The goal is not merely to make the current assertions green.

The goal is to finish the ADT/GADT implementation so that the semantic architecture is coherent:

```text
source declarations
    -> canonical semantic identities and proofs
    -> lossless executable representation
    -> mechanical runtime behavior
```

Every fix should move the repository toward that invariant.

Start with:

```bash
cargo test -p phalcom-core --test core adt
```

and continue until the branch is verified and merged.