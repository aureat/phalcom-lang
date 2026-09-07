---
id: TEST001.C1.P1
category: TEST
program: TEST001
checkpoint: TEST001.C1
kind: corrective
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---

# TEST001.C1.P1 — language corpus baseline remediation
## Execution scope

**Scope:** Proposed  
**Scope:** `phalcom-core --test language-corpus` baseline remediation  
**Baseline:** 37 passed, 24 failed, 4 ignored  
**Primary objective:** Restore the language corpus by completing architectural migrations already underway rather than reinstating obsolete global-name fallback behavior.

---

# 1. Purpose

This plan repairs the 24 current `phalcom-core --test language-corpus` failures as an integrated runtime/compiler-lowering/semantic remediation program.

The failures are not 24 independent defects. They cluster around a small number of architectural seams:

1. incomplete canonical prelude/link-binding cutover;
2. incomplete canonical variant identity cutover;
3. stale pre-ADT `Ordering` consumers;
4. missing union-receiver call application;
5. incorrect superclass type formation before the existing proper-type guard;
6. collision between dynamic `Family` capture semantics and static associated lookup semantics;
7. one inline-cache corpus fixture that still tests the obsolete core-global fallback model.

The implementation must preserve the current architecture:

- canonical declaration and binding identity is authoritative;
- prelude references should lower through linked bindings;
- ADT variants are identified semantically, not by spelling;
- `GetGlobal` is not a generic prelude/core fallback mechanism;
- static and runtime semantics must converge on the same canonical identity model;
- tests must be updated where the language/runtime model has intentionally changed.

---

# 2. Non-goals

This remediation does **not**:

- restore the old implicit `GetGlobal -> current module -> core module` fallback as the permanent runtime model;
- add new parser or lexer features;
- weaken superclass kind checking merely to make tests pass;
- reintroduce `Ordering.kind` as a compatibility representation layer;
- make all `::` operations statically eager;
- solve unrelated compiler, LSP, module-lifecycle, or Plan B work;
- mask intended downstream diagnostics with broader exception handling;
- rewrite the entire language corpus.

---

# 3. Baseline failure inventory

| Corpus test | Current primary symptom | Planned owner |
|---|---|---|
| `corpus::arithmetic` | Ordering path reaches `<invalid value>.kind` | C3 |
| `corpus::bindings` | Missing runtime global `List` | C1 |
| `corpus::booleans` | Missing runtime global `True` | C1 |
| `corpus::bytes_negative` | Missing `ArgumentError`; expected diagnostic masked | C1 |
| `corpus::classes` | Superclass rejected as not a proper type | C5 |
| `corpus::collections` | Missing `DuplicateKeyError` | C1 |
| `corpus::collections_literals_negative` | Missing `ArgumentError` | C1 |
| `corpus::compile_errors` | `None` unresolved before sealed-class diagnostic | C2 |
| `corpus::control_flow` | Union receiver rejects `unwrapOr(_)` | C4 |
| `corpus::decorators` | Missing `MessageNotUnderstood` | C1 |
| `corpus::errors` | Variant-global checks return false | C2 |
| `corpus::family` | Missing runtime global `Family` | C1 |
| `corpus::family_negative` | Associated lookup fails too early | C6 |
| `corpus::functions` | Missing runtime global `Closure` | C1 |
| `corpus::ic` | `List` shadowing contract conflicts with linked-prelude model | C7 |
| `corpus::indexing_negative` | Missing `IndexError` | C1 |
| `corpus::iteration` | Missing runtime global `Family` | C1 |
| `corpus::iterator_negative` | Missing `ArgumentError` | C1 |
| `corpus::path_negative` | Missing `ArgumentError` | C1 |
| `corpus::runtime_errors` | Missing `Error` during destructuring and related masking | C1/C2 |
| `corpus::sequence_negative` | Missing `ArgumentError` | C1 |
| `corpus::streams` | Missing runtime global `Future` | C1 |
| `corpus::streams_negative` | Missing `Future`; expected `UnflushedError` masked | C1 |
| `corpus::strings_negative` | Missing `ArgumentError` | C1 |

---

# 4. Architectural invariants

These invariants are mandatory acceptance criteria, not implementation suggestions.

## INV-1 — Canonical prelude bindings are linked

If semantic analysis resolves an unqualified name to a canonical prelude declaration, compiler lowering must emit a linked binding access, not a plain name-based `GetGlobal`.

Conceptually:

```text
source name
  -> semantic declaration identity
  -> linked binding identity / slot
  -> GetLinked
```

No successful prelude resolution path should require runtime string/name fallback.

## INV-2 — Compiler-generated canonical references use the same identity path

Compiler-synthesized references such as:

- `ArgumentError`
- `IndexError`
- `DuplicateKeyError`
- `MessageNotUnderstood`
- `Error`
- `List`
- `Closure`
- `Family`
- `Future`

must not bypass semantic/canonical binding resolution by constructing a symbol and emitting `GetGlobal`.

## INV-3 — Variant identity is semantic

`None`, `Some`, `Ok`, `Err`, `Error`, `Less`, `Equal`, `Greater`, and `Unordered` must resolve to canonical variant identity through semantic/prelude binding infrastructure.

Ordinary successful analysis must not infer the owner from the spelling of the variant name.

## INV-4 — `Ordering` is an ADT

Relational operators consume canonical `Ordering` variants. No runtime/core-library behavior may depend on an obsolete `kind` field on ordering results.

## INV-5 — Union-call validity is all-arms validity

For a receiver `A | B | ...`, a call is valid only when the selected member is applicable on every reachable arm. The result is the canonical join/union of the arm results after specialization.

## INV-6 — Superclass checking remains strict

A superclass must ultimately form a proper type of kind `Type`. Repair upstream type formation; do not relax the guard.

## INV-7 — Dynamic Family capture and static associated lookup are distinct

For runtime-value family capture, `receiver::member` may capture a route without proving the member exists yet when the language contract requires late-bound failure.

For static class/type/enum associated lookup, associated-member resolution remains eager and semantic.

## INV-8 — Inline caches follow the new binding model

Tests must validate caching and invalidation of the current binding mechanisms. They must not force restoration of obsolete core-global fallback behavior.

---

# 5. Checkpoint sequence

Implementation order is intentional:

```text
C0 Baseline + instrumentation
  -> C1 canonical prelude/link binding cutover
  -> C2 canonical variant identity
  -> C3 Ordering ADT consumer repair
  -> C4 union receiver application
  -> C5 superclass type formation
  -> C6 Family vs associated lookup split
  -> C7 IC corpus migration
  -> C8 full verification and cleanup
```

C1 and C2 are foundational. Do not begin by patching individual corpus fixtures.

---

# 6. C0 — Reproduce, classify, and pin the baseline

## Goal

Create a stable before-state and enough focused regression coverage that later checkpoints can prove which failures they repair.

## Repository areas

Expected primary targets:

- `phalcom-core/tests/language_corpus.rs` or equivalent corpus harness
- `phalcom-core/tests/fixtures/language/**`
- focused tests under `phalcom-core/tests/core/**`
- relevant semantic tests under `phalcom-semantic/tests/**`

## Tasks

1. Run:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus
```

2. Record the exact 24 failing corpus groups and preserve stderr/stdout for representative first failures.

3. Add or identify narrow regression tests for:
   - prelude-linked `List`;
   - compiler-synthesized `ArgumentError`;
   - `True`;
   - `Family`;
   - `Future`;
   - `Ordering` relational comparison;
   - union `Option`/control-flow receiver `unwrapOr(_)`;
   - ordinary class inheritance;
   - late-bound `Family` missing-member failure.

4. Add a test-only inspection hook if necessary to distinguish:
   - `GetGlobal`;
   - `GetLinked`;
   - associated-target lowering;
   - variant-target lowering.

Do not expose a production reflection API merely for these tests.

## Exit gate

- baseline reproduced;
- representative failures independently testable;
- no production behavior changed.

---

# 7. C1 — Complete canonical prelude/link-binding lowering

## Goal

Remove the largest root cause: canonical runtime/prelude names still lowering through name-based `GetGlobal`.

## Primary repository targets

Investigate and modify as applicable:

- `phalcom-core/src/interpret.rs`
  - `attach_prelude_bindings`
  - compile-entry binding preparation
- `phalcom-core/src/compiler/lib/expr.rs`
  - ordinary unqualified-name lowering
- `phalcom-core/src/compiler/lib/patterns.rs`
  - compiler-generated collection/list helpers
- `phalcom-core/src/compiler/lib/loops.rs`
  - synthesized error paths
- `phalcom-core/src/compiler/lib/class_decl.rs`
  - synthesized canonical class/error references if present
- other compiler lowering sites found by:
  ```bash
  rg -n 'GetGlobal|intern\("(ArgumentError|IndexError|DuplicateKeyError|MessageNotUnderstood|Error|List|Closure|Family|Future|True)"' phalcom-core/src/compiler
  ```
- executable semantic/binding structures used by `GetLinked`
- module linked-read construction/materialization
- tests for prelude binding lowering

## Required implementation

### 7.1 Establish one canonical compiler API for runtime/prelude bindings

Add or centralize an operation equivalent to:

```rust
fn emit_canonical_binding_read(
    &mut self,
    binding: CanonicalBindingRef,
    range: SourceRange,
) -> Result<(), CompilerError>
```

The exact type should reuse existing semantic/linker structures. Do not introduce another name-to-runtime registry if an existing declaration/binding identity can represent the target.

Responsibilities:

- accept canonical semantic/link identity;
- allocate/reuse the linked-read slot;
- emit `GetLinked` or the existing equivalent;
- never fall back to symbol-name lookup.

### 7.2 Route ordinary resolved prelude names through linked reads

In ordinary expression lowering:

```text
local
-> upvalue
-> module-local binding
-> linked/prelude binding
-> unresolved error
```

Do not model prelude as a magic global fallback.

Confirm lexical shadowing remains correct:

```phalcom
let List = 42
// later local references resolve to this lexical declaration
```

The existence of a local declaration must be resolved before bytecode emission, not via runtime fallback invalidation.

### 7.3 Migrate compiler-synthesized canonical reads

Audit every compiler-generated `GetGlobal` that names a canonical runtime facility.

Representative categories:

- argument validation;
- unsupported iteration/range paths;
- index failures;
- duplicate collection keys;
- message-not-understood synthesis;
- destructuring/runtime `Error`;
- list construction for synthesized pattern/lowering paths.

Replace symbol-string emission with the canonical binding emitter.

### 7.4 Preserve true module globals

`GetGlobal` remains valid for actual module-owned global slots where the compiler has intentionally selected the current module's binding.

Do not mechanically replace every `GetGlobal`.

### 7.5 Add an invariant test

Add a structural/bytecode test that compiles representative source plus compiler-generated error paths and asserts:

- canonical prelude reads use `GetLinked`;
- no canonical prelude name is emitted as `GetGlobal`.

Prefer checking executable lowering metadata/bytecode rather than matching disassembly strings where possible.

## Expected corpus recovery

This checkpoint should repair or materially advance:

- `bindings`
- `booleans`
- `bytes_negative`
- `collections`
- `collections_literals_negative`
- `decorators`
- `family`
- `functions`
- `indexing_negative`
- `iteration`
- `iterator_negative`
- `path_negative`
- `runtime_errors`
- `sequence_negative`
- `streams`
- `streams_negative`
- `strings_negative`

## C1 verification

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus bindings
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus booleans
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus bytes_negative
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus functions
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus streams
```

Run the full corpus before closing the checkpoint and record which failures remain.

## C1 rejection criteria

Reject the patch if it:

- restores VM core fallback in `GetGlobal`;
- injects canonical classes into every module's globals as aliases merely to preserve old bytecode;
- adds string matching in the VM for error/prelude names;
- fixes only the listed fixtures without addressing synthesized compiler references.

---

# 8. C2 — Canonicalize variant-prelude identity end-to-end

## Goal

Remove spelling-based fallback and make variant globals resolve through canonical semantic identity.

## Primary repository targets

Likely areas:

- semantic prelude/core-surface binding construction
- `phalcom-semantic` name resolution
- enum/variant declaration identity structures
- compiler pattern lowering
- executable variant lowering metadata
- any fallback helper such as `synthesize_fallback_pattern`
- ADT registration/lowering in:
  - `phalcom-core/src/vm/adt.rs`
  - executable semantic target tables
- tests for `Option`, `Result`, and `Ordering`

## Tasks

### 8.1 Represent prelude-visible variants as first-class semantic bindings

Ensure canonical prelude construction can bind variant names directly to `VariantId` or the existing equivalent.

Minimum required canonical mappings include:

```text
None       -> Option::None
Some       -> Option::Some(_)
Ok         -> Result::Ok(_)
Err        -> Result::Err(_)
Less       -> Ordering::Less
Equal      -> Ordering::Equal
Greater    -> Ordering::Greater
Unordered  -> Ordering::Unordered
```

If `Error` remains a compatibility spelling for a Result variant, decide and encode that alias once in the canonical prelude, not in downstream fallback matching.

### 8.2 Remove owner inference by spelling from successful paths

Locate spelling tables that manufacture variant owners and restrict/remove them.

Recovery-only code may remain temporarily if the semantic architecture explicitly requires error recovery, but successful canonical resolution must never depend on it.

### 8.3 Make pattern lowering consume resolved `VariantId`

Pattern lowering should receive exact semantic variant identity and emit the existing exact variant target/discriminant representation.

### 8.4 Make ordinary value references consume canonical variant bindings

Bare `None` and other prelude-visible variants must work in value position without becoming unresolved globals.

### 8.5 Preserve intended diagnostics

`compile_errors` must reach its intended sealed-class diagnostic rather than failing earlier because `None` cannot be resolved.

Do not special-case the corpus fixture; fix resolution ordering.

## Expected corpus recovery

- `compile_errors`
- `errors`
- remaining `runtime_errors` cases involving variant identity
- secondary improvements in pattern/ADT corpus tests

## C2 verification

Run focused semantic and core ADT tests plus:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus compile_errors
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus errors
RUSTFLAGS='' cargo test -p phalcom-core --test core option
RUSTFLAGS='' cargo test -p phalcom-semantic
```

## C2 rejection criteria

Reject if:

- `None` is fixed by adding an ad-hoc parser/compiler keyword case;
- owner identity is still guessed from raw source spelling on the success path;
- runtime selector/name matching is introduced for exact variant recognition.

---

# 9. C3 — Finish the `Ordering` ADT migration

## Goal

Repair relational behavior and diagnostic rendering after the migration from an ordering object/field model to canonical ADT singleton variants.

## Primary repository targets

- `phalcom-core/core/universe/src/object/object.ph`
- `phalcom-core/core/universe/src/object/ordering.ph`
- `phalcom-core/src/primitive/number.rs`
  - `ordering_value`
  - `number_compare`
- `phalcom-core/src/value/render.rs`
- ADT singleton rendering/class tests

## Tasks

### 9.1 Replace `.kind`-based relational derivation

Current stale shape:

```phalcom
<(_ other) { (self <=> other).kind === #less }
```

Replace with direct ADT case semantics.

Preferred form:

```phalcom
<(_ other) {
  match (self <=> other) {
    Less => true
    _ => false
  }
}
```

Use the canonical syntax currently accepted by the language. Do the equivalent for:

- `<`
- `<=`
- `>`
- `>=`

Ensure `Unordered` yields the intended behavior for all four relations.

### 9.2 Keep `Number.compare(_)` returning canonical Ordering variants

Do not change `number_compare` back to an object with a `kind` field.

### 9.3 Render `AdtSingleton` as a valid surface/debug value

`ValueTag::AdtSingleton` is a legitimate value representation and must not fall through to `"<invalid value>"`.

Implement rendering through the ADT registry/descriptor where a VM is available.

Requirements:

- valid singleton never renders as invalid;
- debug rendering identifies owner/variant or canonical `toRepr` equivalent;
- no panic if registry metadata is unexpectedly absent; use a stable diagnostic fallback.

### 9.4 Add regression tests

Test:

- `1 < 2`
- `2 <= 2`
- `3 > 2`
- `3 >= 3`
- float comparisons
- unordered comparison behavior if NaN is supported
- direct rendering of `Ordering::Less`

## Expected corpus recovery

- `arithmetic`

## C3 verification

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus arithmetic
RUSTFLAGS='' cargo test -p phalcom-core --test core
```

---

# 10. C4 — Implement union-receiver callable application

## Goal

Close the known semantic gap where `TypeData::Union` cannot participate in ordinary call resolution.

## Primary repository targets

- `phalcom-semantic/src/checker/call.rs`
- receiver decomposition utilities
- dispatch surface / declaration lookup helpers
- type join/union canonicalization
- explanation/diagnostic nodes for per-arm resolution
- tests referenced by SC-2 union-receiver work

## Required semantics

For receiver:

```text
R = A | B | ... | N
```

and call selector `S`:

1. canonicalize/decompose the receiver union;
2. resolve `S` independently on every reachable arm;
3. receiver member is callable only if every arm admits the call;
4. apply receiver specialization independently per arm;
5. bind/check arguments using the canonical application funnel;
6. collect specialized result types;
7. join/canonicalize results;
8. preserve explanations indicating which arm failed if the call is rejected.

Do not synthesize one fake declaration owner for the union.

## Edge cases

Add tests for:

- same selector, same result on all arms;
- same selector, different results -> union result;
- one arm missing selector -> error;
- generic receiver arms;
- inherited selector on one or more arms;
- `Never` arm handling;
- `Unknown`/blocked arm behavior according to existing analysis policy;
- Option/control-flow case that currently rejects `unwrapOr(_)`.

## Expected corpus recovery

- `control_flow`

## C4 verification

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus control_flow
```

## C4 rejection criteria

Reject if:

- union calls are accepted when only one arm has the method;
- result typing picks an arbitrary arm;
- union handling bypasses the existing canonical callable-application funnel.

---

# 11. C5 — Repair superclass proper-type formation

## Goal

Fix the upstream type-resolution bug that causes valid superclass syntax to reach the proper-type guard with a non-`Type` form.

## Primary repository targets

Investigate:

- `phalcom-semantic/src/session.rs`
- superclass annotation resolution
- declaration type information construction
- `TypeStore` nominal form vs nominal type helpers
- class-object type formation
- generic superclass template resolution
- SC-1 superclass/kinding tests

## Diagnosis discipline

The existing check:

```text
superclass kind must be Type
```

is correct.

Do **not** start by changing it.

Instrument the failing class corpus source and inspect the actual resolved `TypeId`/kind at the point before rejection.

Determine whether the resolver is incorrectly producing one of:

- class-object type;
- constructor-kinded nominal form;
- declaration form before zero-parameter saturation;
- blocked/recovery type incorrectly treated as formed;
- wrong Universe declaration identity.

## Required fix

For a nongeneric superclass:

```phalcom
class Child is Parent { ... }
```

`Parent` in superclass position must form the nominal instance type `Parent : Type`.

For generic parents:

```phalcom
class Child<T> is Parent<T> { ... }
```

the applied superclass must produce a proper type after applying the declaration constructor to valid arguments.

Unsaturated constructors must remain invalid.

## Tests

Add/confirm:

- nongeneric parent;
- generic applied parent;
- unsaturated generic parent rejected;
- wrong-kinded type constructor rejected;
- inherited generic specialization remains correct;
- Universe/native parent remains canonical;
- no regression in hierarchy cycle detection.

## Expected corpus recovery

- `classes`

## C5 verification

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus classes
```

---

# 12. C6 — Separate dynamic Family capture from static associated lookup

## Goal

Restore late-bound Family semantics without weakening static associated resolution for classes/types/enums.

## Primary repository targets

Likely:

- AST associated lookup representation
- `phalcom-semantic` associated resolution
- `phalcom-core/src/compiler/lib/expr.rs`
  - family application lowering
  - associated lookup lowering
- executable associated target metadata
- runtime Family objects/dispatch
- family corpus fixtures

## Required semantic split

### Mode A — static associated lookup

Examples:

```phalcom
Option::Some
Ordering::Less
SomeClass::someStaticAssociatedMember
```

Requirements:

- receiver denotes a static declaration/type/class/enum surface;
- associated member is resolved eagerly;
- lowering captures exact canonical associated target;
- missing target is a semantic error.

### Mode B — dynamic Family capture

Example:

```phalcom
const f = Foo.new()
const g = f::typo
System.print(g.get())
```

Requirements:

- receiver is a runtime value;
- `f::typo` creates/captures the exact Family route;
- capture itself does not probe `f` for the member if the ratified Family contract says failure is call-time;
- `g.get()` performs ordinary runtime resolution;
- missing member reaches ordinary `doesNotUnderstand`/expected runtime error at invocation time.

## Implementation guidance

Do not encode both meanings as one “resolve associated target now” operation.

Prefer an explicit semantic/lowering discriminator derived from the receiver denotation, for example:

```text
AssociatedLookupTarget::Static(...)
AssociatedLookupTarget::DynamicFamily(...)
```

Reuse existing AST if it already stores enough information; avoid parser churn unless absolutely required.

## Tests

Preserve/add:

- dynamic exact getter capture succeeds even when absent;
- failure occurs when Family is called;
- existing static ADT associated lookup remains eager;
- static typo remains compile/semantic error;
- dynamic family exact method and getter shapes remain distinct;
- no accidental probing during Family construction.

## Expected corpus recovery

- `family_negative`
- possibly residual `family`/`iteration` issues after C1

## C6 verification

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus family
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus family_negative
RUSTFLAGS='' cargo test -p phalcom-semantic
```

---

# 13. C7 — Migrate the stale IC fixture to linked-binding semantics

## Goal

Stop the corpus from requiring obsolete dynamic core-global fallback behavior.

## Primary repository targets

- `phalcom-core/tests/fixtures/language/ic/ic_global_cache_shadow_invalidates.ph`
- corresponding `.expected`
- global/linked-read cache tests
- possibly `Chunk` cache tests if coverage is missing

## Current obsolete contract

The fixture currently assumes:

```text
GetGlobal("List")
  -> core fallback
  -> cache core slot
  -> later local declaration "List"
  -> globals_version invalidates cache
  -> same callsite changes binding identity
```

That conflicts with canonical linked prelude identity.

## Replacement contract

Test two separate things.

### 13.1 Linked prelude read stability

A reference semantically bound to prelude `List` remains bound to that canonical declaration.

A later lexical/module declaration with the same spelling must not retroactively change the earlier binding.

### 13.2 Module-global cache invalidation

Keep a separate fixture for real module-global caching:

```phalcom
let x = 1

class C {
  @class
  get { x }
}

System.print(C.get)
x = 2
System.print(C.get)
```

Verify value updates remain visible through the cached module slot.

If declarations can alter slot topology, test the appropriate version/invalidation rule with an actual module-local binding rather than a prelude fallback.

## Exit criteria

- no fixture requires dynamic rebinding from prelude declaration to later same-spelled module declaration;
- GetGlobal cache behavior is still covered for real globals;
- linked-read behavior is directly covered.

## Expected corpus recovery

- `ic`

---

# 14. C8 — Full verification, cleanup, and regression hardening

## Goal

Prove that the remediation restores the corpus without architectural backsliding.

## 14.1 Full test gates

Run:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' cargo test -p phalcom-core
RUSTFLAGS='' cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-modules
```

Then, if practical for the branch:

```bash
RUSTFLAGS='' cargo test --workspace --all-targets
```

## 14.2 Required final baseline

Minimum target:

```text
phalcom-core --test core
  0 failed

phalcom-core --test language-corpus
  0 failed
```

Ignored tests may remain only if they were already intentionally ignored and are unrelated.

## 14.3 Static architectural searches

Run and review:

```bash
rg -n 'Bytecode::GetGlobal' phalcom-core/src/compiler
rg -n 'synthesize_fallback_pattern|Some.*None.*Ok.*Err|Less.*Equal.*Greater.*Unordered' phalcom-core phalcom-semantic
rg -n '\.kind === #less|\.kind === #equal|\.kind === #greater' phalcom-core/core
```

Expected outcome:

- every remaining compiler `GetGlobal` site is justified as a true module-global read;
- successful variant resolution does not depend on spelling tables;
- no stale `Ordering.kind` consumers remain.

## 14.4 Add architectural invariant tests

At minimum:

1. canonical prelude reads lower through linked binding access;
2. compiler-generated canonical errors use linked binding identity;
3. canonical variant globals retain exact `VariantId`;
4. `Ordering` singleton rendering is valid;
5. union receiver call checks all arms;
6. ordinary superclass names form proper nominal types;
7. dynamic Family capture does not probe;
8. static associated lookup still probes;
9. linked prelude identity is not changed by later same-spelled declarations.

---

# 15. Failure-to-checkpoint matrix

| Test | C1 | C2 | C3 | C4 | C5 | C6 | C7 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| arithmetic |  |  | X |  |  |  |  |
| bindings | X |  |  |  |  |  |  |
| booleans | X |  |  |  |  |  |  |
| bytes_negative | X |  |  |  |  |  |  |
| classes |  |  |  |  | X |  |  |
| collections | X |  |  |  |  |  |  |
| collections_literals_negative | X |  |  |  |  |  |  |
| compile_errors |  | X |  |  |  |  |  |
| control_flow |  |  |  | X |  |  |  |
| decorators | X |  |  |  |  |  |  |
| errors |  | X |  |  |  |  |  |
| family | X |  |  |  |  |  |  |
| family_negative |  |  |  |  |  | X |  |
| functions | X |  |  |  |  |  |  |
| ic |  |  |  |  |  |  | X |
| indexing_negative | X |  |  |  |  |  |  |
| iteration | X |  |  |  |  |  |  |
| iterator_negative | X |  |  |  |  |  |  |
| path_negative | X |  |  |  |  |  |  |
| runtime_errors | X | X |  |  |  |  |  |
| sequence_negative | X |  |  |  |  |  |  |
| streams | X |  |  |  |  |  |  |
| streams_negative | X |  |  |  |  |  |  |
| strings_negative | X |  |  |  |  |  |  |

---

# 16. Commit strategy

Use small, bisectable commits. Recommended sequence:

1. `test(core): pin language-corpus baseline root-cause regressions`
2. `fix(core): lower canonical prelude reads through linked bindings`
3. `fix(core): migrate compiler-synthesized runtime bindings off GetGlobal`
4. `fix(semantic): canonicalize prelude variant identities`
5. `fix(core): finish Ordering ADT relational semantics`
6. `fix(core): render ADT singleton values`
7. `fix(semantic): support callable application over union receivers`
8. `fix(semantic): form valid superclass annotations as proper nominal types`
9. `fix(core): separate dynamic Family capture from static associated lookup`
10. `test(core): migrate IC corpus to linked-prelude semantics`
11. `test(core): harden canonical binding and variant invariants`

Do not combine all seven architectural repairs in one commit.

---

# 17. Risk register

## R1 — Hidden `GetGlobal` compatibility dependencies

Changing compiler-generated reads may expose tests that unknowingly depended on core fallback.

Mitigation:

- classify every remaining failing test after C1;
- distinguish stale test semantics from missing linked bindings;
- do not restore fallback globally.

## R2 — Prelude shadowing semantics

Linked binding identity changes when shadowing is decided: compile/semantic time rather than runtime cache invalidation time.

Mitigation:

- add explicit lexical-shadowing tests;
- document that already-resolved references do not retroactively rebind.

## R3 — Variant alias ambiguity

`Error` may refer both to the runtime error class and historical `Result` variant spelling.

Mitigation:

- make the language decision explicit in canonical prelude construction;
- never resolve ambiguity by downstream spelling heuristics.

## R4 — Union-call combinatorial cost

Resolving every union arm can increase semantic work.

Mitigation:

- canonicalize/deduplicate arms first;
- reuse dispatch/application caches;
- enforce existing analysis budgets;
- memoize arm-resolution where current architecture already supports it.

## R5 — Family/associated lookup ambiguity

Receiver classification must be semantic, not syntactic guesswork.

Mitigation:

- base the mode on resolved receiver denotation;
- test class objects, enum roots, ordinary instances, and dynamic/unknown receivers.

## R6 — Superclass fix may affect generic hierarchy metadata

Repairing nominal formation could change stored superclass templates.

Mitigation:

- verify nongeneric and generic hierarchy projection;
- run semantic generic-supertype and inheritance suites before closing C5.

---

# 18. Review checklist

A reviewer should reject the implementation unless all answers below are "yes".

## Canonical bindings

- Are prelude reads semantically linked?
- Do compiler-synthesized canonical references use the same path?
- Is `GetGlobal` reserved for genuine module-global reads?
- Is there a regression test preventing canonical names from returning to synthesized `GetGlobal`?

## Variants

- Are bare variants backed by exact canonical semantic identity?
- Is spelling-based owner inference absent from successful analysis?
- Do pattern and value lowering share variant identity?

## Ordering

- Does `Number.compare(_)` still return canonical `Ordering`?
- Do relational methods consume variants directly?
- Can ADT singletons render without `<invalid value>`?

## Union calls

- Are all arms checked?
- Are argument mappings preserved per arm?
- Is result typing joined canonically?
- Are diagnostics able to identify a failing arm?

## Superclasses

- Is the existing proper-type invariant intact?
- Does a normal parent form a nominal `Type`?
- Are unsaturated generic parents still rejected?

## Family semantics

- Does dynamic capture remain late-bound where required?
- Does static associated lookup remain eager?
- Is the distinction made from semantic receiver identity?

## IC tests

- Do cache tests match the current linked-binding architecture?
- Is obsolete core-fallback rebinding no longer required?

---

# 19. Definition of done

This remediation is complete only when all of the following are true:

1. `phalcom-core --test core` has zero failures.
2. `phalcom-core --test language-corpus` has zero failures.
3. Canonical prelude references no longer rely on runtime core-global fallback.
4. Compiler-synthesized errors/runtime facilities lower through canonical linked binding identity.
5. Prelude-visible ADT variants carry canonical `VariantId` from resolution through execution.
6. `Ordering` relational behavior uses ADT cases directly.
7. Valid ADT singleton values never render as `<invalid value>`.
8. Union receivers support ordinary callable application under all-arms safety.
9. Valid superclass syntax forms a proper type without weakening kind checks.
10. Dynamic Family capture and static associated lookup obey distinct, tested semantics.
11. The IC corpus validates current linked/global cache contracts rather than obsolete fallback semantics.
12. No individual corpus test was "fixed" by adding an ad-hoc spelling, fallback, or fixture-only bypass.

---

# 20. Final implementation principle

The repair should make the architecture **more singular**, not more permissive.

The current failures are valuable because they expose places where old runtime name lookup, new canonical semantic identity, ADT migration, and static analysis still overlap.

The correct patch removes those overlaps:

```text
name spelling
    ↓
semantic identity
    ↓
linked/runtime identity
    ↓
execution
```

The implementation should converge on that pipeline everywhere rather than preserving multiple resolution models for compatibility.
