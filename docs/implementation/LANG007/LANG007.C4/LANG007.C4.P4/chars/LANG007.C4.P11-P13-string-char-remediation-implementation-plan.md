# LANG007.C4.P11–P13 — String/Char Remediation and Completion Implementation Plan

**Repository:** `aureat/phalcom-lang`  
**Primary implementation branch:** `codex/lang007-c4-p4-r1-stdlib`  
**Audited implementation:** `800e179325a4fa0d97e3cd1efa636d09d6fab393`  
**Audited lifecycle tip:** `51c4d35a6820b334599a3872185fa64d8fab0651`  
**Common sibling-lane merge base:** `615f5887f262f8adcb666f4cb4cb431e1a828e11`  
**Prepared:** 2026-10-07  
**Plan class:** patch-grade implementation plan  
**Current lifecycle state:** `IN_PROGRESS / PARTIAL / UNVERIFIED`

---

# 0. Objective

Complete and harden the current Char implementation without redesigning its accepted ontology, fix the pattern-lowering correctness defects exposed by audit, establish real P11 verification, and then build the first coherent String↔Char bridge and String typing cleanup required by P12/P13.

The implementation must converge the repository on one semantic pattern authority and one UTF-8 decoding authority.

The two principal architectural goals are:

```text
source Pattern
    ↓
phalcom-semantic resolution
    ↓
PatternResolution + canonical PatternBindingResolution
    ↓
semantic-to-core projection
    ↓
ExecutablePattern + ExecutableBindingSpec
    ↓
compiler staged test/commit
```

and:

```text
valid Rust String
    ↓ byte-coordinate boundary/offset
String-owned native decoding
    ↓
Char
    ↓
Iterable<Char>
```

This plan deliberately avoids reopening the accepted Char design.

---

# 1. Fixed architecture decisions

The implementer must treat the following as settled.

## 1.1 Char ontology

`Char` is:

```text
one valid Unicode scalar
U+0000..U+D7FF ∪ U+E000..U+10FFFF
```

It is not:

- an alias of `Int`;
- a one-scalar `String`;
- a grapheme cluster;
- a heap object per character.

## 1.2 Runtime representation

Preserve:

```rust
ValueTag::Char
Value::char(char)
Value::as_char() -> Option<char>
```

Char remains immediate in the existing 16-byte `Value`.

Do not introduce a heap-backed Char representation.

## 1.3 Semantic identity

Canonical Char type identity remains Universe/declaration-backed.

Static Char values remain:

```rust
StaticValueData::Char(char)
```

The semantic layer must never use runtime VM tags as static type/value authority.

## 1.4 Match semantics

Preserve INC-6:

- Char is a closed Unicode-scalar domain for coverage;
- Char is non-subclassable for matching;
- one literal covers exactly one scalar;
- surrogates are outside the domain;
- duplicate exact literals are redundant;
- wildcard covers all remaining Char values;
- unknown/underconstrained subjects remain blocked;
- runtime literal matching is exact scalar identity and never dispatches user `==`.

## 1.5 Pattern semantic authority

`phalcom-semantic` is the only source-language pattern-resolution authority.

The compiler may consume frozen executable consequences, but it must not independently resolve:

- variant identity;
- or-pattern binding equivalence;
- literal-domain compatibility;
- exhaustiveness;
- field semantic identity;
- declaration-pattern legality.

## 1.6 UTF-8 authority

Runtime `String` is backed by Rust `String`, so valid UTF-8 is already guaranteed.

P12 must use that invariant rather than implementing an independent guest-language UTF-8 validator.

Use host-owned operations equivalent to:

```rust
str::is_char_boundary
str[index..].chars().next()
char::len_utf8
```

## 1.7 Source coordinates

Source/LSP/parser ranges remain UTF-8 byte offsets.

Do not migrate source ranges to Char or grapheme indices.

---

# 2. Hard invariants

Every patch in this plan must preserve these invariants.

| ID | Invariant |
|---|---|
| `CHAR-SCALAR` | Every Char denotes exactly one valid Unicode scalar. |
| `CHAR-DISTINCT` | Char is semantically distinct from Int and String. |
| `CHAR-STATIC` | Static Char identity is owned by semantic static values. |
| `CHAR-IMMEDIATE` | Runtime Char requires no heap allocation. |
| `CHAR-UNIVERSE` | Canonical Char type/class identity originates from Universe source. |
| `CHAR-MATCH` | Char literal matching uses exact scalar identity, never user equality. |
| `CHAR-COVERAGE` | Coverage/exhaustiveness is semantic-owned. |
| `CHAR-UNKNOWN` | Uncertain subjects remain blocked, never falsely impossible/exhaustive. |
| `PATTERN-JOIN` | Compatible or-pattern alternatives share one canonical binding identity. |
| `PATTERN-ATOMIC` | Failed patterns/alternatives commit no visible binding. |
| `PATTERN-PROJECTION` | Compiler consumes semantic executable products rather than re-resolving AST meaning. |
| `MATCH-TOTAL` | `MatchInvariantFailure` is emitted only for semantic matches proven exhaustive. |
| `DECL-PATTERN` | Declaration pattern legality is decided semantically. |
| `STRING-VALID` | Every String remains valid UTF-8. |
| `STRING-COORD` | Byte offsets and scalar ordinals are never silently conflated. |
| `STRING-BOUNDARY` | Public byte slicing validates UTF-8 boundaries. |
| `STRING-CHAR` | Canonical character iteration yields Char. |
| `STRING-INDEX` | `at(i)` and `size` use one coherent coordinate system. |
| `INT-EXACT` | Int-typed natives accept arbitrary-precision Telos Int, not just immediate i64. |
| `NATIVE-TYPE` | Runtime native results inhabit declared semantic result types. |
| `UNICODE-PIN` | Language-visible Unicode classification version is executable/tested. |
| `GENERATED` | Native generated artifacts are regenerated after authored-source integration. |

---

# 3. Scope

This implementation plan contains four connected slices.

## 3.1 P11 remediation and verification

- fix raw-AST or-pattern binding execution;
- migrate `if let`, `while let`, and `for` pattern execution to semantic executable products;
- remove/harden ordinary `match` AST fallback;
- move declaration Char-literal illegality to semantics;
- add complete Char tests;
- pin Unicode property data;
- clarify bounded Char coverage behavior;
- optionally harden raw Char representation invariants.

## 3.2 P12 String↔Char bridge

- `String.isCharBoundary`;
- `String.decodeCharAt`;
- canonical `String.chars`;
- `StringChars` / iterator yielding `Char`;
- coherent byte-slicing boundary semantics;
- repair or retire incoherent code-point indexed view.

## 3.3 P13 String typing/native hygiene

- correct `rawByteAt` to `Option<Int>`;
- use arbitrary-precision Int extraction;
- reject Float for Int-only native parameters;
- tighten deterministic String/list result types;
- remove stale source audit comments.

## 3.4 Integration and verification

- reconcile Char with the sibling math branch at authored-source owners;
- regenerate native surface;
- run focused and combined tests;
- update lifecycle evidence.

---

# 4. Explicit non-goals

Do not expand this patch into:

- grapheme-cluster APIs;
- Unicode normalization;
- locale-aware case conversion;
- regex;
- mutable strings;
- P16 `StringBuilder`;
- a general interval-pattern language;
- range Char patterns;
- a runtime pattern interpreter;
- a new ownership/borrow system;
- a redesign of `Value`;
- a rewrite of all String algorithms for performance.

If an implementation step appears to require one of these, stop and isolate the actual blocker first.

---

# 5. Dependency graph

Implement in this order.

```text
T0  Preflight / name map / branch state
 │
 ├── T1 Semantic pattern-site product
 │     ├── T2 IfLet semantic migration
 │     ├── T3 WhileLet semantic migration
 │     └── T4 For semantic migration
 │              │
 │              ▼
 ├────────── T5 Core projection for executable pattern sites
 │              │
 │              ▼
 ├────────── T6 Compiler cutover to staged ExecutablePattern
 │              │
 │              ├── T7 Match fallback hardening
 │              └── T8 Declaration-pattern semantic legality
 │
 ├── T9 P11 regression/verification closure
 │
 ├── T10 String native contract root fixes
 │       │
 │       ▼
 ├── T11 P12 boundary/decode primitives
 │       │
 │       ▼
 ├── T12 P12 Char iteration / sequence repair
 │
 ├── T13 P13 typed String hygiene
 │
 ├── T14 Char/math authored-source integration + generation
 │
 └── T15 Final gates and lifecycle records
```

T10–T13 may be developed after T6 is stable, but P11 must not be marked complete until T1–T9 pass.

---

# 6. Repository responsibility map

## Semantic pattern ownership

Primary files:

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/body.rs        # publication path if needed
phalcom-semantic/src/db/fingerprint.rs
```

Relevant existing symbols:

```text
PatternResolution
PatternBindingResolution
ResolvedOrPattern
resolve_pattern
BindingMode
CallableAnalysis.match_resolutions
CheckingContext.match_resolutions
```

## Semantic→core projection

Primary file:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

Relevant existing symbols:

```text
ExecutablePattern
ExecutableBindingSpec
PatternExecutionFrame       # compiler side
MatchLoweringSpec
project_pattern_resolution
ModuleLoweringSemantics
LoweringSite
LoweringSiteKind
ForIterationLoweringSpec
```

## Compiler pattern execution

Primary files:

```text
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/match_expr.rs
phalcom-core/src/compiler/lib/error.rs
```

Legacy symbols to retire from executable refutable contexts:

```text
declare_pattern_locals
emit_pattern_match_tests
commit_pattern_bindings
assign_pattern_from_slot
```

Do not necessarily delete declaration-destructuring helpers that remain valid for truly irrefutable declaration shapes. The goal is to stop using raw-AST refutable execution where semantic executable products exist.

## Char implementation

Primary files already present:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs

phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/coverage/domain.rs
phalcom-semantic/src/checker/coverage/usefulness.rs
phalcom-semantic/src/match_semantics.rs

phalcom-core/src/value/repr.rs
phalcom-core/src/primitive/char.rs
phalcom-core/core/universe/src/scalar/char.ph
phalcom-core/src/universe/core_classes.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs
```

## String implementation

Primary files:

```text
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/src/primitive/string.rs
phalcom-core/src/primitive/int.rs          # canonical Int extraction helper
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs
```

Tests likely belong under:

```text
phalcom-ast/tests/
phalcom-semantic/tests/semantic/
phalcom-core/tests/core/language/
phalcom-core/tests/fixtures/language/
phalcom-core/tests/core/modules/
```

Use existing repository test organization rather than inventing a parallel test harness.

---

# Task 0 — Re-ground the branch before editing

**Purpose:** prevent implementation against stale line numbers or a branch that has already incorporated sibling work.

## Files

Inspection first; no production edits.

## Steps

- [ ] Record:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log -5 --oneline
```

- [ ] Verify the current branch still contains:

```text
800e1793
51c4d35a
```

or record the new integrated descendants.

- [ ] Verify sibling branch relation:

```bash
git merge-base HEAD origin/stdlib-random-math
git log --oneline --left-right HEAD...origin/stdlib-random-math
```

- [ ] Confirm whether the generated native surface has changed since the audit.

- [ ] Build a current symbol map:

```bash
rg -n \
  'PatternResolution|PatternBindingResolution|resolve_pattern|ExecutablePattern|PatternExecutionFrame|MatchLoweringSpec|declare_pattern_locals|emit_pattern_match_tests|commit_pattern_bindings|rawByteAt|leadByteLen|StringCodePointSequence' \
  phalcom-semantic phalcom-core
```

- [ ] Confirm the pinned toolchain:

```text
nightly-2026-07-10
```

- [ ] Do not reset/stash/discard unrelated work.

## Stop conditions

Stop and amend this plan only if:

- the legacy raw-AST refutable pattern path has already been removed;
- `if let` / `while let` / `for` already carry semantic executable pattern specs;
- String API ownership has moved to a new package/file;
- `Value` representation changed after the audit.

Otherwise proceed.

## Deliverable

A short implementation note in the eventual walkthrough containing:

```text
starting commit
branch
dirty/clean state
actual symbol names
sibling merge-base
```

---

# Task 1 — Introduce a reusable semantic product for non-`match` executable pattern sites

**Purpose:** `if let`, `while let`, and `for` currently resolve/bind patterns through ad-hoc recursive binders rather than the formal `PatternResolution` model. Create one publishable semantic product so code generation can consume the exact same resolved pattern structure used by `match`.

## Architectural ruling

Do **not** overload `MatchResolution` for these contexts.

A `match` has:

- multiple arms;
- residual coverage;
- global exhaustiveness;
- branch result join.

`if let`, `while let`, and `for` need only one resolved executable pattern plus canonical binding identities and context/failure semantics.

Introduce a narrow product.

Recommended semantic model:

```rust
pub enum PatternSiteKind {
    IfLet,
    WhileLet,
    ForLane,
}

pub struct PatternSiteResolution {
    pub kind: PatternSiteKind,
    pub range: SourceRange,
    pub subject: TypeKnowledge,
    pub pattern: PatternResolution,
    pub bindings: Box<[PatternBindingResolution]>,
    pub usefulness: PatternUsefulness,
}
```

The exact naming may follow repository convention. `ExecutablePatternSiteResolution`, `PatternExecutionResolution`, or `RefutablePatternResolution` are acceptable if clearer.

### Why keep `usefulness`

For one pattern against a known subject, semantics can still classify:

```text
Useful
Impossible
```

and or-pattern alternatives already have internal redundancy classification.

Do not add a fake "exhaustiveness" result to this product.

### Site key

Prefer a key that cannot collide between constructs.

Options, in preference order:

1. dedicated semantic source-site identity if already available;
2. `(PatternSiteKind, SourceRange)`;
3. enclosing expression ID + site kind.

Do not key solely by pattern text or name.

## Files

Modify:

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/body.rs or publication owner
phalcom-semantic/src/db/fingerprint.rs
```

Potential tests:

```text
phalcom-semantic/tests/semantic/adts/matching/
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

## Steps

- [ ] Add `PatternSiteKind`.
- [ ] Add `PatternSiteResolution`.
- [ ] Add an index type, conceptually:

```rust
pub type PatternSiteResolutionIndex =
    BTreeMap<PatternSiteKey, PatternSiteResolution>;
```

- [ ] Add mutable storage to `CheckingContext`.
- [ ] Publish immutable storage through `CallableAnalysis`.
- [ ] Update every direct `CallableAnalysis { ... }` fixture construction.
- [ ] Add deterministic fingerprint hashing.
- [ ] Ensure fingerprint order uses deterministic map ordering.
- [ ] Add helper:

```rust
ctx.record_pattern_site_resolution(...)
```

with duplicate-site protection in debug/test mode if repository convention supports it.

## Acceptance criteria

- one semantic product can represent Char, Variant, Or, Tuple, List, Record, and Map patterns;
- canonical `BindingId`s are preserved;
- or-pattern alternatives point at the same remapped shared binding IDs after semantic join;
- no compiler-specific slot information enters semantic data.

## Focused test

Construct:

```telos
if let Left(x) | Right(x) = value { x }
```

and assert semantic product structure equivalent to:

```text
PatternSiteResolution
  kind = IfLet
  pattern = Or[
      Variant Left(Binding id=B),
      Variant Right(Binding id=B)
  ]
  bindings = [B => "x"]
```

The same `BindingId` must appear in both alternatives.

---

# Task 2 — Migrate `if let` semantic analysis to formal pattern resolution

**Purpose:** current `Expr::IfLet` uses a local recursive `bind_pattern` that only meaningfully understands a subset of patterns. Replace it with `checker::pattern::resolve_pattern`.

## Existing defect

Current flow is conceptually:

```text
analyze RHS
then branch:
    recursively bind names/tuple components
else branch
```

It does not publish formal pattern resolution for:

- Char;
- Variant;
- Or;
- Record;
- Map;
- rich List behavior.

## Files

Modify:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/pattern.rs        # helper exposure only if needed
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/matching/
phalcom-semantic/tests/semantic/control/       # use existing location
```

## Steps

- [ ] Analyze RHS once as today.
- [ ] Derive a `CoverageSubject` from:

```text
RHS TypeKnowledge canonical type
RHS LocalType when present
```

matching `synthesize_match_expr` behavior.

- [ ] In the then-branch lexical scope, call:

```rust
resolve_pattern(ctx, &if_let.pattern, &subject, &mut bindings)
```

instead of the local `bind_pattern`.

- [ ] Preserve causal invalidity / flow explanation metadata on the canonical bindings.

If `resolve_pattern` does not currently accept causal/explanation information, do **not** fork it. Add a narrow post-resolution binding-fact attachment helper or extend the binding declaration hook in one place.

- [ ] Compute single-pattern usefulness against the subject using the existing coverage engine.

- [ ] For an impossible pattern, emit the existing `MatchPatternImpossible` or introduce a context-appropriate equivalent only if diagnostic policy requires it.

Do not classify uncertainty as impossible.

- [ ] Record a `PatternSiteResolution` only from the real analysis pass.

- [ ] Analyze then body with those canonical bindings live in scope.

- [ ] Analyze else body with none of those bindings visible.

- [ ] Preserve existing branch result/flow join semantics.

## Required regressions

1. `if let 'a' = c` with `c: Char`.
2. `if let Left(x) | Right(x) = value`.
3. nested `Some(Left(x) | Right(x))`.
4. impossible Char pattern against known `Int`.
5. underconstrained/generic subject remains non-falsely-impossible.
6. else branch cannot resolve `x`.
7. then branch resolves exactly one `x`.

## Acceptance criteria

`Expr::IfLet` contains no parallel AST-only pattern binder for executable semantics.

---

# Task 3 — Migrate `while let` semantic analysis without breaking loop fixed-point analysis

**Purpose:** use formal pattern resolution while preserving the existing loop fixed-point machinery.

## Critical implementation concern

`Expr::WhileLet` performs probe analyses during loop-header fixed-point solving.

Do not publish probe-only pattern products or allow probe allocations to become final semantic identity.

The final stable pass is authoritative.

## Files

Modify:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/pattern.rs        # helper exposure if needed
```

Tests:

```text
phalcom-semantic/tests/semantic/control/
phalcom-semantic/tests/semantic/adts/matching/
```

## Steps

- [ ] Keep RHS evaluation once per semantic loop step.
- [ ] Build the pattern `CoverageSubject` from the per-step RHS fact.
- [ ] During fixed-point probes:
  - resolve the pattern in the probe context;
  - allow probe-local binding IDs;
  - do not publish a `PatternSiteResolution` into the final analysis.
- [ ] During the final stable-header real pass:
  - resolve pattern in the real context;
  - record the final canonical `PatternSiteResolution`;
  - use those bindings for body analysis.
- [ ] Preserve:
  - normal backedge;
  - continue edges;
  - break exits;
  - condition/pattern failure exit flow.
- [ ] Ensure pattern failure does not apply pattern-bound flow facts to the loop exit.
- [ ] Ensure successful pattern bindings are scoped to the loop body/iteration context.

## Required regressions

```telos
while let Left(x) | Right(x) = next() {
    use(x)
}
```

Verify:

- RHS semantic model is one evaluation per iteration step;
- `x` has one final canonical binding identity;
- loop exit does not expose `x`;
- `continue` and normal backedge use the same binding shape;
- failed pattern exit remains reachable.

Add nested Char and variant patterns.

## Stop condition

If the current fixed-point context clone shares mutable pattern-resolution publication state unexpectedly, first isolate publication from probe contexts. Do not work around it with source-range overwrite races.

---

# Task 4 — Migrate `for` lane semantic patterns to formal resolution

**Purpose:** current `Statement::For` obtains canonical iterator evidence but still binds lane patterns with another local AST recursive binder. Replace the pattern half while preserving the canonical iteration lane semantics.

## Semantics to preserve

A `for` pattern is **not a filter**.

Failure semantics remain:

```text
yielded value fails lane pattern
    -> ordinary runtime pattern-mismatch error
```

The semantic product describes how to execute the pattern; it does not change this failure policy.

## Files

Modify:

```text
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/analysis.rs/context.rs
```

Potentially extend existing iteration resolution types only if doing so avoids duplicate source keys.

## Recommended ownership model

Keep iteration and pattern products separate but keyed by the same lane range/source.

```text
ForIterationLoweringSpec      = how to obtain items
PatternSiteResolution         = how to bind/test each item
```

Do not embed pattern semantics into the iterator trait selection product.

## Steps

- [ ] Keep `resolve_for_iteration_lane` unchanged.
- [ ] For each lane element fact, form a `CoverageSubject`.
- [ ] During loop probe body analysis, resolve lane patterns in probe context.
- [ ] During final real pass, record one `PatternSiteResolution(kind=ForLane)` per lane.
- [ ] Use canonical pattern bindings to analyze the body.
- [ ] Keep `at index` lane binding semantics separate from pattern bindings.
- [ ] Preserve multi-lane strict zip behavior.
- [ ] Preserve mismatch-as-error semantics rather than silent skip.
- [ ] Ensure or-pattern shared names join once.

## Required regressions

Single-lane:

```telos
for Left(x) | Right(x) in values {
    use(x)
}
```

Multi-lane:

```telos
for Left(x) | Right(x) in xs,
    Some(y) in ys {
    ...
}
```

Verify:

- one shared `x`;
- no duplicate declaration;
- each lane has distinct binding table;
- pattern failure raises;
- iteration lane trait evidence is unaffected.

---

# Task 5 — Project non-match semantic pattern sites into `ModuleLoweringSemantics`

**Purpose:** freeze semantic pattern products into backend-only executable specs.

## Architectural design

Reuse:

```rust
ExecutablePattern
ExecutableBindingSpec
project_pattern_resolution(...)
```

Do not invent a second backend pattern IR.

Introduce a small wrapper:

```rust
pub struct PatternSiteLoweringSpec {
    pub pattern: ExecutablePattern,
    pub bindings: Box<[ExecutableBindingSpec]>,
}
```

Optionally include:

```rust
pub kind: ExecutablePatternSiteKind
```

only if one map contains all site categories.

## Keying

Recommended:

```rust
pub enum LoweringSiteKind {
    ...
    Match,
    IfLetPattern,
    WhileLetPattern,
    ForLanePattern,
}
```

and:

```rust
pub pattern_sites:
    BTreeMap<LoweringSite, PatternSiteLoweringSpec>
```

Alternative acceptable design:

```text
if_let_patterns: BTreeMap<SourceRange, ...>
while_let_patterns: ...
for_lane_patterns: ...
```

Prefer the unified `LoweringSite` approach because the repository already uses it to prevent source-range meaning collisions.

## Files

Modify:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/mod.rs
```

Tests:

```text
phalcom-core/tests/core/language/compiler/lowering.rs
phalcom-core/tests/core/language/compiler/lowering_scenarios.rs
```

## Steps

- [ ] Add lowering site kinds.
- [ ] Add `PatternSiteLoweringSpec`.
- [ ] Generalize `project_pattern_resolution` so both match arms and standalone pattern sites reuse it.
- [ ] Project binding tables through the same `BindingId -> binding_index` mapping logic.
- [ ] Iterate callable analyses and project `PatternSiteResolution`s for the compiling module.
- [ ] Reject duplicate attachments.
- [ ] Project cross-module `VariantFieldId` through canonical enum semantics exactly as match lowering does.
- [ ] Do not copy:
  - coverage witnesses;
  - GADT proof environments;
  - usefulness diagnostics;
  - source selector patterns;
  - type terms not needed for runtime execution.

## Acceptance criteria

For:

```telos
if let Left(x) | Right(x) = value { ... }
```

the lowering product must contain:

```text
ExecutablePattern::Or([
    ExecutablePattern::Variant(... Binding index 0 ...),
    ExecutablePattern::Variant(... Binding index 0 ...)
])
bindings = [x]
```

---

# Task 6 — Cut `if let`, `while let`, and `for` over to the staged executable pattern engine

**Purpose:** remove the critical bug where raw AST declaration/commit traverses every or-pattern alternative.

## Existing correct machinery to reuse

`phalcom-core/src/compiler/lib/patterns.rs` already contains:

```text
PatternExecutionFrame
setup_pattern_frame
emit_executable_pattern
commit_pattern_frame
```

This is the correct executor.

## Existing legacy machinery to stop using in these contexts

```text
declare_pattern_locals
emit_pattern_match_tests
commit_pattern_bindings
assign_pattern_from_slot
```

The most important rule:

> No refutable executable pattern context may re-traverse `Pattern::Or` to commit bindings.

## Files

Modify:

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/error.rs
```

Potential tests:

```text
phalcom-core/tests/core/language/compiler/
phalcom-core/tests/fixtures/language/patterns/
```

## Step 6.1 — Add compiler lookup helper

Implement a helper conceptually:

```rust
fn pattern_site_spec(
    &self,
    kind: LoweringSiteKind,
    range: SourceRange,
) -> Result<PatternSiteLoweringSpec, CompilerError>
```

A missing semantic spec in an analyzed production module is an internal lowering-contract error.

Do not silently use raw AST.

## Step 6.2 — `if let`

Retain control skeleton:

```text
evaluate RHS once
store $if_let_value
setup PatternExecutionFrame
emit_executable_pattern
success:
    commit_pattern_frame
    execute then body
failure:
    cleanup
    execute else / None path
```

Important:

- binding locals exist only in then branch scope;
- else branch must not see them;
- failed alternatives may leave staging scratch values but must never commit them;
- success commits once.

## Step 6.3 — `while let`

Retain:

```text
reserve value slot once
loop:
    evaluate RHS once
    store
    setup/reuse visible binding frame appropriately
    test/stage
    failure -> loop exit
    success -> commit
    body
    close captured visible bindings if required
    backedge
```

Be careful with runtime local lifetime:

- visible pattern binding slots should remain stable across loop iterations if current closure capture behavior expects that;
- staging slots may be reused each iteration;
- no stale staging value is semantically visible.

## Step 6.4 — `for`

For each lane item:

```text
extract Some payload into lane value slot
emit lane ExecutablePattern
if failure -> shared for-pattern mismatch raise
on success -> commit lane bindings
```

Only after **all lane patterns** succeed should body execution proceed.

### Atomicity across multiple lanes

Current code commits each lane immediately after its test.

That can expose lane 1 bindings before lane 2 fails, even though the body never executes.

Although failure raises immediately and may make the partial write unobservable in normal source execution, the architecturally cleaner rule is:

```text
stage all lanes
all lane patterns succeed
commit all visible lane bindings
```

Implement this if practical in the same patch.

At minimum, never commit an individual or-pattern alternative incorrectly.

Recommended design:

```text
one PatternExecutionFrame per lane
test/stage all lanes
shared success label
commit all frames
body
```

## Step 6.5 — Retire legacy usage

After migration:

```bash
rg -n 'declare_pattern_locals|emit_pattern_match_tests|commit_pattern_bindings' phalcom-core/src/compiler
```

Expected remaining uses should be zero for:

- `if let`;
- `while let`;
- `for`.

If helpers remain only for old declaration destructuring, rename/comment them so they cannot be mistaken for the canonical refutable pattern engine.

## Step 6.6 — Critical regressions

Use a real two-case data/enum:

```telos
Left(value)
Right(value)
```

Test:

```telos
if let Left(x) | Right(x) = Left(42) { System.print(x) }
if let Left(x) | Right(x) = Right(42) { System.print(x) }
```

Then `while let` and `for`.

Assert:

- no duplicate-local error;
- one canonical visible `x`;
- no non-selected payload projection;
- `x == 42`;
- failed first alternative does not poison second;
- closures capturing `x` capture the visible committed binding.

## Acceptance criteria

The exact audited defect must become impossible by construction:

```rust
Pattern::Or { alternatives } => {
    for alternative in alternatives {
        commit(...)
    }
}
```

must no longer participate in executable refutable pattern lowering.

---

# Task 7 — Remove the unchecked ordinary `match` fallback

**Purpose:** enforce the checker→compiler exhaustiveness authority boundary.

## Current defect

`compile_match_expr` falls back to:

```rust
synthesize_fallback_pattern(...)
```

when no semantic `MatchLoweringSpec` exists and still emits:

```rust
Bytecode::MatchInvariantFailure
```

This allows unproven syntax to be compiled under a proven-total runtime invariant.

## Required invariant

```text
MatchInvariantFailure may be emitted
IFF
semantic MatchResolution.exhaustiveness == Proven
and project_match_resolution succeeded.
```

## Files

Modify:

```text
phalcom-core/src/compiler/lib/match_expr.rs
phalcom-core/src/compiler/lib/error.rs
```

Tests:

```text
phalcom-core/tests/core/language/compiler/lowering_scenarios.rs
```

## Steps

- [ ] Add/confirm:

```text
CompilerError::MissingMatchLoweringSemantics(SourceRange)
```

- [ ] In production `compile_match_expr`, require attached semantic lowering.
- [ ] Delete `synthesize_fallback_pattern` from the production path.
- [ ] Remove fallback-only imports/types if no longer required.
- [ ] If compiler unit tests require AST-only compilation, create an explicitly named test helper, e.g.:

```text
compile_unchecked_match_for_test
```

but do not route normal module compilation through it.

- [ ] Preserve `project_match_resolution` check:

```rust
ExhaustivenessResult::Proven
```

- [ ] Add a regression proving a match with no semantic lowering returns `MissingMatchLoweringSemantics` rather than compiling.

## Acceptance criteria

There is no source-to-`ExecutablePattern` path for ordinary match outside semantic projection in production compilation.

---

# Task 8 — Move declaration pattern legality to the semantic layer

**Purpose:** remove compiler ownership of source-language validity for Char literal patterns in declaration destructuring.

## Current implementation

Compiler contains a recursive:

```text
contains_char_literal(pattern)
```

and returns:

```text
InvalidExecutablePattern
```

for declaration binding.

This is defense-in-depth at the wrong authority layer.

## Desired semantic model

Introduce explicit declaration-pattern policy.

Conceptual API:

```rust
enum PatternContext {
    MatchArm,
    IfLet,
    WhileLet,
    ForBinding,
    DeclarationBinding,
    ParameterBinding,
}
```

This does not need to become public if a smaller internal helper is sufficient.

For `DeclarationBinding` in the accepted current language:

Allowed irrefutable declaration forms include the existing declaration destructuring forms.

Char literal pattern is refutable and must be rejected.

Likewise reject nested Char literals.

Do not accidentally make all future variant patterns legal in declaration binding unless the language separately chooses refutable declaration syntax.

## Files

Modify:

```text
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/pattern.rs or a validation helper
phalcom-semantic/src/diagnostic.rs / diagnostic code owner if a new code is needed
```

Compiler:

```text
phalcom-core/src/compiler/lib/patterns.rs
```

may retain an internal assertion/guard.

## Steps

- [ ] Before `bind_declaration_pattern`, validate the AST pattern against declaration policy.
- [ ] Emit a semantic user diagnostic at the literal/pattern range.
- [ ] Suggested message:

```text
character literal patterns are refutable and cannot be used in declaration destructuring
```

- [ ] Recursively detect nested invalid forms:
  - tuple;
  - list;
  - record;
  - rest;
  - future pattern forms.
- [ ] Do not make this Char-specific internally if a generic `is_irrefutable_declaration_pattern` helper is cleaner.
- [ ] Keep compiler guard only as unreachable/defensive internal protection.

## Tests

```telos
let 'a' = c
let ('a', x) = pair
let ['a', *rest] = values
let { key: 'a' } = record
```

must fail semantically.

Tooling/LSP semantic diagnostics should see the same error before codegen.

---

# Task 9 — Close the P11 Char verification contract

**Purpose:** convert P11 from code-by-inspection to evidence-backed implementation.

This task is mandatory even if all production fixes appear obvious.

---

## 9.1 Lexer tests

### Valid cases

Add exact-token tests for:

```text
'a'
'é'
'中'
'🙂'
'\n'
'\r'
'\t'
'\\'
'\''
'\u{0}'
'\u{41}'
'\u{D7FF}'
'\u{E000}'
'\u{10FFFF}'
```

Assert:

```rust
Token::Char(expected)
```

not merely `is_ok()`.

Also assert exact byte ranges.

### Invalid cases

```text
''
'ab'
'é'                 // e + U+0301
raw LF
raw CR
raw CRLF
'\q'
'\u'
'\u{}'
'\u{XYZ}'
'\u{1234567}'
'\u{D800}'
'\u{DFFF}'
'\u{110000}'
unterminated quote
unterminated \u{...}
```

Assert exact `LexicalError` variant and range.

### Files

```text
phalcom-ast/tests/lexer.rs
```

---

## 9.2 Parser tests

Assert actual AST nodes.

Expression:

```rust
Statement::Expr {
    expr: Expr::Char { value: 'é', range },
    ..
}
```

Pattern:

```text
match c { 'a' => ..., _ => ... }
if let 'a' = c { ... }
```

Or-pattern:

```text
'a' | 'b'
```

### Files

```text
phalcom-ast/tests/parser.rs
```

---

## 9.3 Semantic literal/static tests

Add assertions for:

```text
'a' : Char
'a' is not Int
'a' is not String
```

Static values:

```text
intern('a') == intern('a')
intern('a') != intern('b')
Char('a') != Int(97)
equal Char values hash equally
fingerprints deterministic
materialization returns Value::char('a')
```

Incremental:

- edit static Char constant `'a' -> 'b'`;
- verify product fingerprint changes;
- dependent static/materialized products invalidate.

### Files

Use existing semantic static-value/incremental test locations.

---

## 9.4 Coverage tests

Required source/semantic cases:

### Exact + wildcard total

```telos
match c {
    'a' => 1
    _ => 2
}
```

### Duplicate redundant

```telos
match c {
    'a' => 1
    'a' => 2
    _ => 3
}
```

Second arm must be redundant.

### Wrong subject impossible

Known `Int` subject with Char literal must be impossible, not equivalent to 97.

### Unknown subject

Generic/local-rigid/projection uncertainty must remain blocked where relation proof is unavailable.

### Direct utility boundary tests

Test `first_missing_scalar` around:

```text
U+D7FF
U+D800..DFFF excluded
U+E000
U+10FFFF
```

Do not create a source fixture with 1,112,064 arms.

---

## 9.5 Runtime tests

Required:

```text
'a'.class == Char
'a'.codePoint == 97
'a'.utf8Length == 1
'é'.utf8Length == 2
'中'.utf8Length == 3
'🙂'.utf8Length == 4
'é'.toString == "é"
```

`Char.fromCodePoint`:

```text
0
97
0xD7FF
0xE000
0x10FFFF
-1
0xD800
0xDFFF
0x110000
huge LargeInt
```

Identity:

```text
'a' == 'a'
'a' !== 'b'
'a' != 97
'a' != "a"
```

Classification representatives:

```text
ASCII
digit
alphabetic
upper/lower
alphanumeric
hex digit
Unicode whitespace
Unicode control
```

---

## 9.6 Exact pattern execution

Verify Char literal pattern compilation emits exact constant + `Same`.

Assert it does **not** invoke:

```text
==
toString
codePoint
```

for matching.

---

## 9.7 Unicode version guard

Add a Rust regression for the pinned toolchain Unicode tables.

Use the exact standard-library constant/type exposed by `nightly-2026-07-10`.

Conceptual assertion:

```rust
assert_eq!(std::char::UNICODE_VERSION, expected_17_0_0);
```

If the toolchain exposes a structured version instead of tuple, use its native representation.

---

## 9.8 Bounded coverage contract

Update the Char spec/lifecycle note:

```text
Char is formally closed.
A wildcard proves remaining coverage immediately.
A mathematically complete literal singleton set is total.
Coverage queries remain resource-bounded and may return Blocked/BudgetExceeded
for pathological proofs.
```

Do not raise `QueryBudget::default().max_steps` merely to support a million-arm pathological match.

---

## 9.9 Raw representation debug invariant

Optional but recommended.

At trusted raw-word boundaries, add a debug validation helper:

```text
if tag == Char:
    payload <= 0x10FFFF
    payload not in 0xD800..0xDFFF
```

Avoid adding runtime cost to ordinary `Value::char`.

---

# Task 10 — Fix String native contracts at the root

**Purpose:** establish truthful typed primitives before layering P12 Char APIs over them.

## 10.1 `rawByteAt` result type

### Current

```telos
internal native rawByteAt(_ index: Int) -> Int
```

runtime may return `None`.

### Target

```telos
internal native rawByteAt(_ index: Int) -> Option<Int>
```

Update:

```text
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/src/primitive/string.rs
phalcom-native-meta/src/universe.rs        # generated/metadata owner as applicable
phalcom-native-surface/src/generated.rs    # regenerate, do not hand-edit final form
```

Native macro:

```text
returns = Option
types = "(Int) -> Option<Int>"
```

Use the repository's existing native metadata syntax for generic Option result.

## 10.2 Exact Int semantics

Replace:

```rust
Value::as_int()
Value::as_float()
```

index parsing with canonical:

```rust
crate::primitive::int::expect_int_big(...)
```

### `rawByteAt`

Policy:

```text
negative       -> None
too large usize -> None
>= byte length -> None
valid index    -> Some(byte Int)
```

Use existing `Option` wrapping helper.

Do not return bare byte Int under an `Option<Int>` signature if native Option representation requires `Some(...)`.

Audit current call sites because they presently compare the raw return to `None`. After the type correction, runtime representation must still match language semantics.

### `rawSlice`

Both endpoints:

```text
must be Int
negative -> defined range error
too large for usize -> out-of-range error
Float -> Type error
```

Preserve boundary validation.

## 10.3 No Float coercion

Remove integral Float acceptance.

Native runtime must accept exactly the values its semantic signature permits.

## Tests

Direct native/core tests for:

```text
0
last valid byte
byte_count
-1
LargeInt within usize if constructible
huge LargeInt
1.0 Float
NaN
```

For `rawSlice`:

```text
valid boundaries
negative
start > end
end > len
continuation start/end
Float endpoint
LargeInt overflow
```

---

# Task 11 — Implement P12 String boundary and Char decode primitives

**Purpose:** make String itself the canonical UTF-8→Char bridge.

## Target surface

Recommended internal/public API:

```telos
String.isCharBoundary(_ byteOffset: Int) -> Bool
String.decodeCharAt(_ byteOffset: Int) -> Option<Char>
```

Whether these begin `internal` or public should follow the accepted P12 surface plan, but parser/library consumers must have a typed path.

## Runtime implementation

Prefer native Rust implementation.

### `isCharBoundary`

Algorithm:

1. extract Telos Int exactly;
2. negative -> `false`;
3. conversion overflow -> `false`;
4. if `offset > s.len()` -> `false`;
5. return `s.is_char_boundary(offset)`.

EOF is a valid boundary.

### `decodeCharAt`

Algorithm:

1. exact Int extraction;
2. negative/overflow -> `None`;
3. `offset >= s.len()` -> `None`;
4. if `!s.is_char_boundary(offset)` -> `None`;
5. obtain:

```rust
let ch = s[offset..].chars().next().expect(...valid non-EOF boundary...);
```

6. return `Some(Value::char(ch))`.

No manual byte-classifier is required.

## Optional helper product

If callers frequently need the next offset, a compact data type can be added:

```telos
data DecodedChar(
    char: Char,
    nextByteOffset: Int,
)
```

and:

```telos
decodeCharStep(_ byteOffset: Int) -> Option<DecodedChar>
```

However do not add this if `decodeCharAt + Char.utf8Length` is sufficient for v1.

Keep v1 small.

## `leadByteLen`

Choose one:

### Preferred

Demote/remove it from canonical iteration internals after migration.

### If retained

Make continuation offsets return `None`.

It must not be the canonical source of truth once `isCharBoundary` / `decodeCharAt` exist.

## Required tests

For:

```text
"Aé中🙂"
```

byte layout:

```text
A  : 0..1
é  : 1..3
中 : 3..6
🙂 : 6..10
EOF: 10
```

Assert:

```text
isCharBoundary(0)  == true
isCharBoundary(1)  == true
isCharBoundary(3)  == true
isCharBoundary(6)  == true
isCharBoundary(10) == true

isCharBoundary(2) == false
isCharBoundary(4) == false
isCharBoundary(5) == false
isCharBoundary(7) == false
```

Decode:

```text
decodeCharAt(0) -> Some('A')
decodeCharAt(1) -> Some('é')
decodeCharAt(3) -> Some('中')
decodeCharAt(6) -> Some('🙂')
decodeCharAt(2) -> None
decodeCharAt(10) -> None
```

Also test negative and huge Int.

---

# Task 12 — Implement canonical `String.chars` and repair code-point sequence semantics

**Purpose:** stop exposing Int code points as the primary character iteration model.

## Target API

```telos
String.chars -> StringChars
```

where:

```text
StringChars does CanonicalIterable
type Item = Char
```

and its iterator:

```text
next -> Option<Char>
```

## Recommended structure

```telos
class StringChars {
  _string: String

  iterator -> StringCharIterator
}

class StringCharIterator {
  _source: String
  mut _byteOffset: Int
  mut _done: Bool

  next -> Option<Char> { ... }
}
```

Internal iterator state may use byte offsets.

The API must expose Char values, not offsets or code points.

## Implementation strategy

On `next`:

```text
if offset >= byteCount -> None
decodeCharAt(offset) -> Some(ch)
offset += ch.utf8Length
return Some(ch)
```

Because String is valid UTF-8 and offset is produced internally, `decodeCharAt` failure at a non-EOF iterator offset is an internal invariant error, not normal source behavior.

## `StringCodePointSequence`

Current contract is incoherent:

```text
size = number of scalars
at(i) = codePointAt(byteOffset=i)
```

Choose one of these.

### Recommended migration

Keep `codePoints` for compatibility but make:

```text
codePoints.at(i)
```

mean the **i-th scalar code point**, O(n).

That restores normal sequence semantics.

Implementation can iterate through `chars` until ordinal `i` and return `char.codePoint`.

### Alternative

If repository compatibility allows, remove indexed-sequence claims from `codePoints` and leave only iteration.

Do not leave current mixed coordinate system.

## `each`

Character-oriented iteration should pass `Char` for `chars.each`.

`codePoints.each`, if retained, may continue to pass Int code points.

## Required tests

For `"Aé中🙂"`:

```text
chars -> ['A','é','中','🙂']
chars.iterator.next types are Char

codePoints.size == 4
codePoints.at(0) == 65
codePoints.at(1) == 233
codePoints.at(2) == 20013
codePoints.at(3) == 128578
```

No continuation-byte offset is observable through ordinal `at`.

---

# Task 13 — Complete P13 String typing hygiene

**Purpose:** remove Dynamic-era signatures from deterministic String APIs.

Do this after T10–T12 establish the correct primitive types.

## Primary file

```text
phalcom-core/core/universe/src/scalar/string.ph
```

Potential supporting collection typings as required.

## Targets

At minimum tighten:

```telos
split(_ delimiter: String) -> List<String>
contains(_ needle: String) -> Bool
includes(_ needle: String) -> Bool
indexOf(_ needle: String) -> Int
replace(_ needle: String, _ to: String) -> String
trim(_ chars: String) -> String
trimStart(_ chars: String) -> String
trimEnd(_ chars: String) -> String
```

Where zero-argument overloads exist, keep them separately typed.

Change:

```text
List<Dynamic>
```

to:

```text
List<String>
```

where construction and pushes are statically String.

## Runtime `is(String)` checks

Once the source signature is type checked, decide whether runtime checks are still required at dynamic call boundaries.

Do not blindly remove them if Telos permits dynamic invocation that can bypass static checking.

The correct rule is:

- static source contract is precise;
- runtime still protects truly dynamic/untyped boundaries if the runtime dispatch model requires it.

## Multiplication/repeat

Do not expand this task into StringBuilder.

If `String * count` remains Number-typed historically, document it and defer broader numeric API correction unless P13 explicitly owns it.

## Stale comments

Remove the obsolete `REVIEW: BLOCKING ISSUES` header in:

```text
phalcom-core/src/primitive/string.rs
```

and stale fixture comments.

Only live invariants/limitations should remain.

---

# Task 14 — Reconcile Char with sibling math/native-surface work

**Purpose:** avoid certifying generated artifacts from one divergent lane and losing the other lane's authored registrations.

## Current topology

Audit found:

```text
merge base: 615f5887
Char branch: codex/lang007-c4-p4-r1-stdlib
math branch: stdlib-random-math
```

Both touch native/Universe integration surfaces.

## Rule

Do not manually merge final generated native-surface records.

## Integration sequence

- [ ] Merge/rebase/cherry-pick authored changes according to current project integration policy.
- [ ] Resolve canonical source owners first:
  - Universe source declarations/packages;
  - `UniverseKey`;
  - `CoreClasses`;
  - primitive modules;
  - bootstrap/native registration;
  - module builtin/package registration;
  - semantic core declaration identities.
- [ ] Resolve lifecycle/checkpoint docs manually.
- [ ] Delete conflict-marker/manual patch assumptions from generated surface.
- [ ] Run the repository native-surface generator.
- [ ] Accept the regenerated combined artifact.
- [ ] Run parity/consistency tests.

## Files likely overlapping

```text
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/primitive/mod.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/...follow-on-plan.md
```

Current math branch may also have evolved further; re-check before integration.

## Acceptance criterion

A fresh generation from merged authored sources produces no unexpected diff.

---

# Task 15 — Focused verification, classification, and lifecycle closure

**Purpose:** produce evidence sufficient to advance P11 and accurately record P12/P13 state.

## 15.1 Test-selector sanity

Before claiming any command as evidence, confirm it selects actual tests.

Do not report:

```text
0 tests passed
```

as verification.

## 15.2 Suggested focused gate order

Exact package/test names should follow current workspace structure.

### AST

```bash
cargo test -p phalcom-ast character -- --nocapture
cargo test -p phalcom-ast char -- --nocapture
```

Then full crate if affordable:

```bash
cargo test -p phalcom-ast
```

### Semantic

Run focused:

```text
Char literal type/static tests
Char coverage tests
PatternSiteResolution tests
if-let/while-let/for or-pattern tests
incremental fingerprint tests
```

Then:

```bash
cargo check -p phalcom-semantic
```

and the directly affected semantic test target(s).

### Core

Run focused:

```text
Char runtime
pattern lowering
if let
while let
for
match lowering
String raw natives
String Char boundaries
String chars
native surface parity
```

Then:

```bash
cargo check -p phalcom-core
```

### Generated/native metadata

Run the repository-native generation/check command documented by the current build scripts.

Do not guess the command if it has changed; locate it first.

## 15.3 Failure classification

For every failure classify:

```text
REGRESSION CAUSED BY THIS PATCH
PRE-EXISTING KNOWN FAILURE
UNRELATED CURRENT-BRANCH FAILURE
INFRASTRUCTURE/TOOLCHAIN FAILURE
```

Never weaken tests or language contracts to make unrelated failures disappear.

## 15.4 P11 completion gate

P11 may move to a tested/completed state only when all of the following are true:

- [ ] exact Char lexer valid/invalid matrix exists;
- [ ] parser asserts real Char AST nodes;
- [ ] semantic Char identity/static tests pass;
- [ ] Char coverage tests pass;
- [ ] exact runtime match tests pass;
- [ ] `if let`/`while let`/`for` use semantic executable patterns;
- [ ] or-pattern binding tests pass in all three contexts;
- [ ] ordinary `match` no longer has production semantic bypass;
- [ ] declaration Char literal illegality is semantic;
- [ ] core Char native/runtime tests pass;
- [ ] Unicode version guard passes;
- [ ] directly affected crates compile.

If P12/P13 work remains incomplete, P11 may still complete as long as its lifecycle scope clearly excludes those later slices.

## 15.5 P12 completion gate

P12 may complete when:

- [ ] `isCharBoundary` exists and is typed;
- [ ] `decodeCharAt` returns `Option<Char>`;
- [ ] continuation offsets return `None`;
- [ ] EOF boundary/decode distinction is tested;
- [ ] `String.chars` yields Char;
- [ ] source ranges remain byte offsets;
- [ ] no String operation can create malformed UTF-8.

## 15.6 P13 completion gate

P13 may complete when:

- [ ] `rawByteAt` type is truthful;
- [ ] String Int indices use arbitrary-precision Int semantics;
- [ ] Int-only natives no longer accept Float;
- [ ] deterministic String/List results are precisely typed;
- [ ] `StringCodePointSequence` has coherent indexing semantics;
- [ ] stale blocker comments are removed.

---

# 7. Detailed compiler execution design

This section is normative for T5–T7.

## 7.1 One shared staged executor

Do not build separate code generators for:

```text
match
if let
while let
for
```

All must share:

```rust
emit_executable_pattern(
    pattern,
    value_slot,
    frame,
    failures,
    range,
)
```

and:

```rust
commit_pattern_frame(...)
```

Only the failure continuation changes by context.

## 7.2 Failure continuations

| Context | Failure behavior |
|---|---|
| match arm | jump to next arm |
| if let | else/None path |
| while let | exit loop |
| for lane | raise pattern mismatch |
| declaration | not a refutable executor; semantic legality + irrefutable destructuring only |

## 7.3 Or-pattern staging law

For:

```text
A(x) | B(x)
```

the lowering table has one binding slot `x`.

Alternative A stages into `stage[0]`.

If A fails after staging, that scratch value is ignored.

Alternative B may overwrite `stage[0]`.

Only after complete Or success:

```text
stage[0] -> visible x
```

No alternative-specific visible local exists.

## 7.4 Variant candidate staging law

For a selector/family pattern with several candidate variants, each candidate may source `x` from a different physical payload slot.

The semantic projection decides each slot.

Every candidate stages into the same logical binding index.

The compiler must never infer a field slot from:

```text
source name
label spelling
argument position
selector text
```

## 7.5 Scratch lifetime

Scratch slots must remain valid across all bytecode operations that may suspend/yield if the VM permits suspension in those paths.

Use existing compiler scratch-local conventions.

Do not use raw operand-stack temporaries across arbitrary sends when a rooted local is required.

---

# 8. Semantic pattern-site design details

## 8.1 Do not publish probe artifacts

Loop fixed-point analysis may execute semantic bodies multiple times.

Only the final real pass should populate the final callable `PatternSiteResolutionIndex`.

Probe contexts may create temporary resolutions/bindings internally.

## 8.2 Binding identity

`PatternBindingResolution.binding` is the semantic identity used for:

- body name resolution;
- executable binding-table projection;
- or-pattern alternative join.

The final semantic pattern product and source-index/body binding occurrence attachments must agree.

Add a regression checking body `x` resolves to the same `BindingId` exported in the pattern-site product.

## 8.3 Causal invalidity

Pattern-bound knowledge should inherit relevant invalidity from the subject expression.

Do not lose existing `CausalInvalidity` behavior when replacing `bind_pattern`.

If `resolve_pattern` currently declares bindings with clean causal state, introduce one centralized way to attach the subject cause to the declared canonical bindings.

Do not duplicate recursive pattern traversal merely to propagate the cause.

## 8.4 Flow proof refinement

For exact variant patterns, successful branches may know a narrowed case type.

Reuse the existing match helpers where applicable:

```text
pattern_exact_case_type
pattern_common_proof
```

Do not introduce a second GADT/refinement solver.

For v1 non-match site migration, preserve current flow behavior first; add exact-case refinement only through already-established semantic helpers.

---

# 9. Char coverage performance policy

The implementation must preserve a bounded checker.

## Do

- use exact singleton sets for current literal-only Char coverage;
- return `Blocked(BudgetExceeded)` when the global checker budget is exceeded;
- use direct `first_missing_scalar` for one-dimensional Char residual witness generation;
- test surrogate-gap behavior directly.

## Do not

- enumerate the entire Unicode scalar domain for ordinary wildcard checks;
- set default semantic budget above one million;
- generate one constructor per Char;
- pretend Char is an enum declaration with 1.1M cases.

## Future-compatible extension point

If later adding Char ranges/properties, use an interval set:

```text
[0x0000,0xD7FF]
[0xE000,0x10FFFF]
```

with union/difference.

This is not part of the current patch.

---

# 10. String byte/scalar model

The implementer should use these definitions consistently.

## Byte count

```text
String.size
```

currently means UTF-8 byte count.

Do not silently redefine it to scalar count.

## Byte boundary

An offset is a boundary if it is:

- 0;
- the start of a UTF-8 scalar;
- EOF.

## Decode position

A decodable position is a boundary strictly before EOF.

Thus:

```text
EOF isCharBoundary == true
EOF decodeCharAt   == None
```

## Scalar ordinal

The n-th Char in a String is not generally reachable by O(1) indexing.

If exposed, ordinal lookup is O(n) unless a future indexing structure is added.

Document that honestly.

---

# 11. Required test scenario catalog

The implementation should cover at least these named scenarios.

## CHAR-LEX-01 ASCII literal

```text
'a'
```

exact token/AST/range.

## CHAR-LEX-02 multibyte literals

```text
'é' '中' '🙂'
```

one scalar each.

## CHAR-LEX-03 escape matrix

Every supported escape.

## CHAR-LEX-04 invalid scalar

Surrogate and > U+10FFFF.

## CHAR-LEX-05 multiple scalar

`'é'` rejected.

## CHAR-SEMA-01 literal identity

`'a' : Char`.

## CHAR-STATIC-01 fingerprint identity

Same/different Char static values.

## CHAR-COV-01 wildcard totality

Literal + wildcard.

## CHAR-COV-02 redundancy

Duplicate Char literal.

## CHAR-COV-03 impossible domain

Char pattern against known Int.

## CHAR-COV-04 blocked unknown

Generic/projection subject.

## CHAR-COV-05 surrogate gap witness

D7FF→E000.

## CHAR-RUN-01 immediate class/value

Class, equality, hash.

## CHAR-RUN-02 UTF-8 length

1/2/3/4.

## CHAR-RUN-03 fromCodePoint boundaries

All accepted/rejected boundaries.

## PAT-OR-IF-01 shared binding

`if let A(x) | B(x)`.

## PAT-OR-WHILE-01 shared binding

`while let A(x) | B(x)`.

## PAT-OR-FOR-01 shared binding

`for A(x) | B(x)`.

## PAT-OR-NEST-01 partial first alternative

First alternative stages then fails; second succeeds; visible binding comes only from second.

## MATCH-AUTH-01 no lowering spec

Must fail compilation with semantic-lowering error.

## DECL-PAT-01 direct Char literal

Semantic diagnostic.

## DECL-PAT-02 nested Char literal

Semantic diagnostic.

## STR-NATIVE-01 rawByteAt Option

Type + runtime.

## STR-NATIVE-02 LargeInt index

Correct out-of-range behavior.

## STR-NATIVE-03 Float rejection

No accidental coercion.

## STR-BOUND-01 UTF-8 boundaries

ASCII/2/3/4 byte string.

## STR-DECODE-01 decode Char

Correct Char values.

## STR-DECODE-02 continuation rejection

`None`.

## STR-DECODE-03 EOF distinction

Boundary true, decode none.

## STR-CHARS-01 iteration

Yields Char.

## STR-CP-01 ordinal codePoints.at

If retained, coherent O(n) ordinal semantics.

## GEN-01 fresh native-surface generation

No unexpected diff.

---

# 12. Suggested commit boundaries

Keep commits narrow enough to audit.

Recommended sequence:

```text
1. sema: publish executable pattern site resolutions
2. sema: migrate if-let pattern analysis
3. sema: migrate while-let and for pattern analysis
4. core: project executable pattern site lowering
5. core: migrate refutable pattern contexts to staged executable patterns
6. core: require semantic lowering for match
7. sema: validate declaration pattern refutability
8. test: close Char P11 regression matrix
9. core: fix String raw native type and Int contracts
10. stdlib: add String Char-boundary decode primitives
11. stdlib: add Char iteration and repair code-point sequence
12. stdlib: tighten String types and remove stale review comments
13. build: reconcile native surfaces and regenerate
14. docs: record P11/P12/P13 lifecycle evidence
```

Do not combine the semantic pattern migration and String API rewrite into one giant commit.

---

# 13. Stop-and-consult conditions

Stop implementation and request architecture review if any of the following occurs.

## Pattern system

- `PatternResolution` lacks enough information to execute one of the currently legal refutable pattern shapes without reading AST semantic meaning.
- a non-match pattern site cannot obtain stable canonical `BindingId`s during final analysis.
- fixed-point probe contexts share final publication maps in a way that makes pattern-site products nondeterministic.
- migrating `for` requires changing its mismatch semantics from raise to filter.
- semantic projection cannot map `VariantFieldId` to a physical slot without guessing.

## Runtime Char

- Char no longer fits existing `Value` representation.
- any valid Char requires heap allocation.
- native Char class identity cannot remain canonical Universe-backed.

## String

- String backing storage is no longer guaranteed valid UTF-8.
- `decodeCharAt` would require parsing arbitrary invalid byte arrays stored inside String.
- changing `rawByteAt` to `Option<Int>` exposes a wider native-metadata inconsistency that requires a separate global native-type migration.
- collection typing prevents `List<String>` without unrelated generic-system changes.

## Integration

- math branch has already been merged through a different generated-surface version and the generator cannot deterministically recreate the combined surface.
- generated artifacts contain hand-authored semantic information not reproducible from canonical sources.

---

# 14. Files expected to change

This is a planning map, not a mandate to touch every file.

## Strongly expected

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/db/fingerprint.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/match_expr.rs
phalcom-core/src/compiler/lib/error.rs

phalcom-core/src/primitive/string.rs
phalcom-core/core/universe/src/scalar/string.ph

phalcom-ast/tests/lexer.rs
phalcom-ast/tests/parser.rs
```

## Likely test/generated/lifecycle changes

```text
phalcom-semantic/tests/semantic/...
phalcom-core/tests/core/language/...
phalcom-core/tests/fixtures/language/...
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs

docs/specs/objects/characters.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P11-char-and-literals-plan.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P11-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P11-handoff.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P11-pattern-coverage-incident.md
```

P12/P13 lifecycle files should be created/updated according to the repository's lifecycle convention if they now move from `PROPOSED / NOT_STARTED`.

---

# 15. Lifecycle updates after implementation

Do not pre-mark tasks complete.

## P11 walkthrough must record

- actual implementation commits;
- exact tests added;
- exact commands run;
- number of selected tests;
- pass/fail results;
- unrelated failures and classification;
- semantic pattern-site migration;
- removal of ordinary match fallback;
- declaration-pattern semantic diagnostic;
- Unicode version evidence.

## P11 handoff must state

Whether P11 is now:

```text
COMPLETE / FOCUSED_TESTED
```

or remains partial.

Do not claim workspace/release verification unless those gates were actually run.

## P12/P13

Record independently.

P11 completion must not imply P12/P13 completion.

---

# 16. Final implementation checklist

## Semantic pattern authority

- [ ] `PatternSiteResolution` exists.
- [ ] `if let` uses `resolve_pattern`.
- [ ] `while let` uses `resolve_pattern`.
- [ ] `for` lane pattern uses `resolve_pattern`.
- [ ] final products are published only from real/final analysis.
- [ ] or-pattern alternatives share canonical binding IDs.
- [ ] pattern-site products are fingerprinted.

## Core projection

- [ ] non-match pattern sites project to `ExecutablePattern`.
- [ ] binding tables use semantic `BindingId`.
- [ ] no AST name/variant resolution occurs during projection.
- [ ] duplicate lowering attachments are rejected.

## Compiler

- [ ] `if let` uses `PatternExecutionFrame`.
- [ ] `while let` uses `PatternExecutionFrame`.
- [ ] `for` uses `PatternExecutionFrame`.
- [ ] or-pattern commit is single/canonical.
- [ ] failed alternatives do not commit.
- [ ] ordinary `match` requires semantic lowering.
- [ ] `MatchInvariantFailure` is proof-backed only.

## Declaration semantics

- [ ] direct Char literal declarations rejected semantically.
- [ ] nested Char literal declarations rejected semantically.
- [ ] compiler guard is defense-in-depth only.

## Char verification

- [ ] valid lexer matrix.
- [ ] invalid lexer matrix.
- [ ] exact parser AST assertions.
- [ ] semantic type identity.
- [ ] static value/fingerprint/materialization.
- [ ] coverage/redundancy/impossibility/blocking.
- [ ] runtime class/equality/hash.
- [ ] utf8Length 1/2/3/4.
- [ ] `fromCodePoint` boundaries + LargeInt.
- [ ] Unicode classification.
- [ ] Unicode version pin.
- [ ] exact pattern runtime matching.

## String native root fixes

- [ ] `rawByteAt -> Option<Int>`.
- [ ] native metadata agrees.
- [ ] BigInt extraction used.
- [ ] Float no longer accepted for Int.
- [ ] rawSlice uses exact Int semantics.
- [ ] UTF-8 boundary validation retained.

## P12

- [ ] `isCharBoundary`.
- [ ] `decodeCharAt -> Option<Char>`.
- [ ] continuation offsets rejected.
- [ ] EOF distinction tested.
- [ ] `String.chars`.
- [ ] iterator yields Char.
- [ ] code-point sequence indexing coherent.

## P13

- [ ] `List<String>` where deterministic.
- [ ] String argument/result signatures tightened.
- [ ] stale source blocker comments removed.

## Integration

- [ ] sibling authored native sources reconciled.
- [ ] generated surface regenerated.
- [ ] generator/parity tests pass.

## Lifecycle

- [ ] P11 status updated only from executed evidence.
- [ ] P12/P13 tracked independently.
- [ ] checkpoint updated with exact commit/test evidence.

---

# 17. Completion definition

This implementation effort is successful when the following architectural statement is true in the repository:

> Every executable pattern with semantic meaning is resolved once in `phalcom-semantic`, projected into one backend `ExecutablePattern` representation, and executed through staged atomic binding; the compiler never reconstructs or-pattern binding identity or match exhaustiveness from syntax. Char remains a distinct immediate Unicode scalar with complete evidence-backed tests. String remains valid UTF-8 and exposes a typed byte-boundary-to-Char bridge whose character iteration yields Char rather than Int. Native String contracts agree with Telos's arbitrary-precision Int and static type system, and all generated native surfaces are recreated from the reconciled authored sources.

That is the target state the implementation plan should deliver.
