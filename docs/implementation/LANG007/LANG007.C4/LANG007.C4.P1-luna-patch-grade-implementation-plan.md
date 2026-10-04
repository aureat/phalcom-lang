---
id: LANG007.C4.P1
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - LANG007.C3
follows: LANG007.C3.P5
supersedes: null
repository: aureat/phalcom-lang
prepared: 2026-10-04
prepared_against_revision: 8c3a98aab41ed8631e9f8746a6b29984cf77581d
title: "Binding, Mutability, Field Lifecycle, and Corpus Cutover"
---

# LANG007.C4.P1 — Binding, Mutability, Field Lifecycle, and Corpus Cutover

## Luna Patch-Grade Implementation Plan

This plan is derived from the C4 requirements analysis prepared against
`main@8c3a98aab41ed8631e9f8746a6b29984cf77581d`.

Its purpose is to make the first C4 semantic cutover executable by GPT-5.6 Luna
High/Extra High without requiring Luna to rediscover the language design or
invent cross-layer architecture while coding.

The plan follows the repository's normative:

```text
docs/workflow/luna-patch-grade-plan-schema.md
```

and uses the same sparse-testing, bounded-debugging, checkpoint, walkthrough,
handoff, and STOP/CONSULT discipline used by the later LANG007.C3 plans.

---

# 0. Executor contract

This is a **Luna patch-grade implementation plan**.

The implementer:

- may adapt private helper names, local Rust decomposition, import locations,
  and other mechanical details to the live repository;
- must preserve every `P1-INV-*` invariant below;
- must treat the semantic layer as the canonical owner of flow/init facts;
- must not make `None` double as uninitialized binding state;
- must not preserve old runtime-`const` semantics under a hidden compatibility
  path;
- must not retain implicit field declaration as a fallback;
- must not reserve `mut` globally merely to implement `let mut`;
- must not add a branch to every ordinary proven local read;
- must not make the compiler independently re-run definite-initialization
  analysis;
- must classify broad failures before editing;
- must STOP AND CONSULT when a global or task-local trigger fires;
- must keep the C4 checkpoint record current at durable gates;
- must create the required P1 walkthrough and handoff;
- must preserve unrelated user/agent worktree changes;
- must not broaden P1 into the P2 static evaluator.

Testing is evidence gathering, not ritual. During BUILD MODE, run only the
smallest command that distinguishes the current implementation hypothesis.

## 0.1 Fixed user-ratified C4 rulings

These are **FIXED** for P1.

### RULING P1-A — runtime binding surface

```telos
let x = expr
let mut y = expr
let mut z
let mut w: T
```

means:

```text
let      = runtime immutable
let mut  = runtime mutable
```

`let` without an initializer is invalid.

### RULING P1-B — `const` is static phase

```telos
const x = expr
```

is a compile-time/static declaration.

It is not a spelling for a runtime immutable binding.

P1 establishes the phase boundary. P2 implements general static evaluation and
materialization.

### RULING P1-C — uninitialized is not `None`

```telos
let mut x: Int
```

creates mutable storage without a value.

Reading it before a value has been established is invalid.

`None` remains an ordinary Telos value and is never implicitly manufactured to
represent missing initialization.

### RULING P1-D — partial definite initialization

The semantic lattice is:

```text
Uninitialized
MaybeInitialized
DefinitelyInitialized
```

Read policy:

```text
DefinitelyInitialized -> statically safe / unchecked lowering
Uninitialized         -> guaranteed invalid read diagnostic
MaybeInitialized      -> legal source / runtime checked read
```

P1 does not impose globally strict definite assignment.

### RULING P1-E — fields are immutable by default

```telos
class Counter {
    _name: String
    mut _count: Int
}
```

means:

```text
_name   = immutable field
_count  = mutable field
```

### RULING P1-F — immutable instance fields use construction lifecycle

An immutable instance field may be established during the canonical
construction lifecycle, exactly once.

A declaration initializer counts as initialization before constructor body
execution.

Every successful normal construction path must leave each required immutable
instance field definitely initialized.

### RULING P1-G — fields are explicit only

Assignment never creates stored field layout.

Every stored field must be declared.

### RULING P1-H — hard cutover

P1 performs a source/corpus cutover.

Do not keep the old public semantics indefinitely behind compatibility aliases.

---

# 1. Goal

Implement the C4.P1 semantic cutover so that Telos has:

1. runtime immutable `let`;
2. runtime mutable `let mut`;
3. true uninitialized mutable binding state;
4. path-sensitive definite-initialization proof with runtime fallback;
5. static-phase `const`;
6. immutable-by-default explicit fields;
7. constructor-lifecycle immutable field initialization;
8. no assignment-created storage;
9. coherent module/source/tooling metadata;
10. a fully migrated repository corpus.

---

# 2. Checkpoint acceptance objective

P1 is the **foundational semantic cutover** of `LANG007.C4`.

It does not close C4.

It establishes the runtime/static declaration distinction and structural
storage invariants that every later C4 plan relies on.

P1 must leave P2 with a clean boundary:

```text
source `const`
    ↓
canonical static declaration identity
    ↓
P2 static evaluator / StaticValue
```

rather than:

```text
source `const`
    ↓
runtime immutable slot
    ↓
P2 somehow retrofits compile-time semantics later
```

P1 also removes the implicit field-layout behavior that would undermine later
structural reflection.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 8c3a98aab41ed8631e9f8746a6b29984cf77581d
relevant predecessor: LANG007.C3.P5 / closed LANG007.C3
requirements analysis:
    LANG007.C4.P1-requirements-analysis.md
```

Verified repository facts at planning time:

1. `BindingKind` currently has `Let` and `Const`.
2. current `Let` means mutable.
3. current `Const` means runtime immutable.
4. `mut` is not a dedicated lexer token.
5. parser infrastructure already recognizes contextual `mut` for fields /
   properties / delegation.
6. `FieldDef` already has `mutable: bool`.
7. current bare class fields are effectively mutable.
8. current `const _field` is the immutable-field spelling.
9. semantic `BindingState` tracks mutability but not binding initialization as
   an independent dimension.
10. semantic field flow already has
    `Uninitialized / MaybeInitialized / DefinitelyInitialized`.
11. `FlowState::join_impl` already merges path state and excludes unreachable
    predecessors.
12. loop widening/fixpoint infrastructure already exists.
13. `FieldSemanticSignature.mutable` already exists and is included in its
    semantic fingerprint.
14. constructor analysis already publishes normal-exit field flow.
15. compiler class layout still derives some fields from assignments.
16. current initializer-less `let` compilation emits `Bytecode::Nil`.
17. the VM's `Bytecode::Nil` pushes guest-visible `None`, not a binding-specific
    uninitialized sentinel.
18. local/global/upvalue runtime reads currently surface internal absence.
19. `DeclarationSurface` currently exposes an ambiguous `is_const: bool`.
20. source index currently distinguishes `TopLevelLet/Const`,
    `LocalLet/Const`, and stores a separate `mutable` bit.
21. source-index mutability for some implicit binders is not aligned with
    semantic binding immutability.
22. current LSP hover text explicitly describes `let` as mutable and
    initializer-less `let` as reading `None`.
23. the `.ph` corpus is large enough that textual replacement is unsafe.
24. code search at the audited revision found essentially no canonical
    `let mut`/`mut _field` usage in the `.ph` corpus.
25. the repository contains a normative Luna plan schema and C3 Luna plans that
    should be followed.

> Re-read the named live paths before editing. Adapt mechanical drift locally.
> Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Read these before touching source:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md

docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-handoff.md

LANG007.C4.P1-requirements-analysis.md
```

Also read the live source files named under each task before editing them.

Read current tests/guides for the directly affected subsystems only:

```text
phalcom-core/tests/README.md
phalcom-semantic/tests/semantic/capabilities/BASELINE_LEDGER.md
```

Read the current accepted binding/absence/class-field specifications as
historical/current takeover material, including the current `let`/`const`
decision and private-`Nil`/`None` model.

Do not treat those older binding spellings as higher authority than the
ratified C4 decisions above.

Avoid broad repository exploration unless a VERIFY-FIRST assumption fails.

---

# 5. Normative authority

Authority order for P1:

## 5.1 Highest authority — ratified C4 behavior

The fixed rulings in §0.1 and the supplied C4.P1 requirements analysis.

These define the target semantics.

## 5.2 Repository workflow authority

```text
AGENTS.md
luna-patch-grade-plan-schema.md
implementation-record-lifecycle-convention.md
shared-consultation-escalation-protocol.md
commit-and-push-discipline.md
```

These define execution discipline and implementation records.

## 5.3 Existing language/runtime decisions where non-conflicting

Existing decisions about:

- `None` as guest-visible absence;
- private internal `Nil`;
- static instance slot layout;
- constructor lowering;
- closures/upvalues;
- module/global storage;
- semantic query ownership;
- incremental fingerprints.

These remain authoritative unless C4 directly supersedes their old
`let`/`const` or field spelling semantics.

## 5.4 Current implementation

Current code is takeover state, not normative language behavior.

When code implements old semantics, P1 intentionally changes it.

---

# 6. Takeover state and stable interface map

| Concept | Current symbol/path | Owner | P1 treatment |
|---|---|---|---|
| Binding syntax | `phalcom-ast/src/ast.rs::BindingKind` | AST | extend to `LetMut`; normalize semantics |
| Binding parser | `phalcom-ast/src/parser.rs` | parser | parse contextual `let mut` |
| Contextual `mut` | parser contextual-word helpers / `starts_mut_member` | parser | reuse; do not reserve globally |
| Binding flow | `checker/flow/state.rs::BindingState` | semantic | add initialization axis |
| Binding mutation | `FlowState::write` / `CheckingContext::write_existing` | semantic | mark initialized on valid write |
| Branch join | `FlowState::join_impl` | semantic | join initialization lattice |
| Loop state | `widen_loop_state_with_hierarchy` | semantic | include init state |
| Binding summary | `checker/analysis.rs::FlowBindingSummary` | semantic | publish init state |
| Field flow | `FieldInitialization` / `FieldState` | semantic | preserve/generalize |
| Field defaults | `field_lifecycle.rs::default_field_seeds` | semantic | already useful |
| Constructor exits | `finalize_instance_field_lifecycle` | semantic | strengthen to obligation |
| Field signature | `signature.rs::FieldSemanticSignature` | semantic | reuse `mutable` |
| Field fingerprint | `db/fingerprint.rs` | semantic | already hashes mutability |
| Module surface | `phalcom-modules/src/interface.rs::DeclarationSurface` | modules | replace `is_const` ambiguity |
| Source binding kind | `source_index/scope.rs::SourceBindingKind` | semantic source index | add LetMut categories |
| Local compiler mutability | `compiler/lib/scope.rs::Local::is_mutable` | compiler | consume normalized meaning |
| Global compiler mutability | compiler global binding map | compiler | consume normalized meaning |
| Old uninit lowering | `compiler/lib/mod.rs` `None => Bytecode::Nil` | compiler | replace |
| Local read | `Bytecode::GetLocal` | VM | unchecked safe path |
| Upvalue read | `Bytecode::GetUpvalue` | VM | checked when proof absent |
| Global read | `Bytecode::GetGlobal` | VM | checked when proof absent |
| Internal value tags | `phalcom-core/src/value/repr.rs::ValueTag` | runtime | likely binding-specific sentinel |
| Class layout | `compiler/lib/class_decl.rs` | compiler | explicit declarations only |
| LSP wording | `phalcom-lsp/src/hover.rs` | tooling | update |
| Semantic tokens | `phalcom-lsp/src/semantic_tokens.rs` | tooling | contextual `mut` |
| VS Code fallback grammar | `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json` | tooling | update |

---

# 7. Architecture

## 7.1 Binding authority flow

```text
source
  │
  ├─ let
  ├─ let mut
  └─ const
  ↓
AST BindingKind
  ↓
normalized semantic declaration meaning
  ├─ BindingPhase
  │    ├─ Runtime
  │    └─ Static
  └─ RuntimeMutability
       ├─ Immutable
       └─ Mutable
  ↓
semantic binding state
  ├─ contract
  ├─ value/type knowledge
  ├─ initialization state
  ├─ denotation
  ├─ consistency
  └─ mutability
  ↓
path-sensitive read classification
  ├─ ProvenInitialized
  ├─ DefinitelyUninitialized
  └─ RuntimeCheckRequired
  ↓
compiler lowering
  ├─ unchecked load
  └─ checked load
  ↓
VM
```

The compiler must not reconstruct initialization from syntax or control flow.

## 7.2 Field authority flow

```text
explicit ClassMember::Field only
  ↓
FieldSemanticSignature
  ├─ field identity
  ├─ declared type
  └─ mutable
  ↓
FieldLifecycleTable
  ├─ default seed
  ├─ constructor flow
  └─ final lifecycle proof
  ↓
compiler class layout
  ├─ declared slots only
  └─ immutable/mutable slot metadata
  ↓
VM instance/class storage
```

Body assignment is a consumer of declared storage, never a creator of storage.

## 7.3 Runtime uninitialized binding model

Preferred architecture:

```text
ValueTag::UninitializedBinding
```

or equivalent private immediate state.

It must be:

- distinct from guest-visible `None`;
- distinct from ordinary initialized values;
- private to compiler/VM;
- zero-allocation;
- trapped at checked read boundaries;
- impossible to wrap in `Some`;
- impossible to print/hash/compare/send as a normal value.

Do **not** globally redefine generic private `Nil` semantics merely to avoid
adding a binding-specific representation.

## 7.4 Checked-read strategy

```text
semantic proven safe:
    GetLocal / ordinary fast load

semantic uncertain:
    checked load -> trap if UninitializedBinding
```

Equivalent strategy is permitted if the hot safe path remains branch-free.

For upvalues/globals, where local flow may not prove state at the read site,
checked reads remain the fallback.

## 7.5 Static `const` boundary

P1 must end with:

```text
Const -> Static phase
```

not:

```text
Const -> Runtime { mutable: false }
```

P1 may reject/gate runtime consumption of static constants until P2, but may not
silently execute them as ordinary runtime initializers.

---

# 8. Ownership boundaries

## 8.1 P1 owns

- binding phase/mutability syntax and normalization;
- binding initialization state;
- read safety classification;
- runtime uninitialized-binding representation;
- checked versus unchecked binding read lowering;
- static-phase classification of `const`;
- explicit field declaration authority;
- field default mutability inversion;
- immutable-field initialization lifecycle enforcement;
- corpus migration;
- source/module/tooling corrections directly required by the cutover.

## 8.2 P1 does not own

- general static expression evaluation;
- `StaticValue`;
- static recursion/residualization;
- Product/Sum reflection;
- generalized FieldKey;
- first-class Pack;
- static matching/loops;
- cross-module static-value serialization;
- new ownership/borrow system;
- comprehensive constructor escape analysis.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| surface binding spelling | AST/parser | semantic/source index/compiler | compiler string inspection |
| runtime/static phase | semantic normalized binding semantics | modules/compiler/P2 | `is_const` boolean reinterpretation |
| runtime mutability | semantic normalized meaning | flow/compiler/tooling | repeated `kind == Let` logic |
| binding initialization | semantic `FlowState` | lowering/tooling | compiler-side CFG analysis |
| binding read safety | semantic callable/expression analysis | compiler | syntax-derived guess |
| field mutability | `FieldSemanticSignature` derived from AST | checker/compiler/metadata | layout-default reconstruction |
| field existence | explicit class declaration | checker/compiler/reflection | assignment scan |
| field initialization | semantic lifecycle table | checker/lowering | constructor-name heuristic |
| class layout | compiler projection of explicit semantic/source declaration | VM | assignment discovery |
| uninitialized runtime state | runtime private representation | checked loads | `None` |
| static constant identity | module/semantic static declaration category | P2/P8 | runtime global slot identity |

---

# 9. Global invariants

`P1-INV-*` IDs are durable and should be referenced by tests, gates, checkpoint
updates, walkthrough, and handoff.

### P1-INV-01 — `mut` remains contextual

P1 does not reserve `mut` globally unless a consultation/amendment explicitly
changes that design.

### P1-INV-02 — `let` is runtime immutable

A `let` binding may not be reassigned.

### P1-INV-03 — `let mut` is runtime mutable

Only the explicit `mut` form permits runtime reassignment.

### P1-INV-04 — immutable `let` requires initializer

There is no `let x` uninitialized immutable state.

### P1-INV-05 — `const` is static phase

`const` is not represented as ordinary runtime immutable storage.

### P1-INV-06 — uninitialized is not `None`

No path converts missing binding initialization into guest-visible `None`.

### P1-INV-07 — initialization is independent of type knowledge

Initialization state is not inferred from `Unknown(MissingInitializer)` alone.

### P1-INV-08 — guaranteed uninitialized read is diagnosed

A read proven uninitialized is a compile-time semantic error.

### P1-INV-09 — maybe-initialized read is legal

Uncertain initialization does not become a false compile-time rejection.

### P1-INV-10 — uncertain read is runtime checked

If initialization cannot be proven at the read, the runtime can trap safely.

### P1-INV-11 — proven initialized local read stays fast

No runtime check is added to an ordinary local read that semantic analysis has
proved initialized.

### P1-INV-12 — compiler consumes semantic proof

Lowering does not independently solve definite initialization.

### P1-INV-13 — captured mutable storage preserves initialization

An upvalue cell can remain uninitialized until a later assignment and traps if
read before that assignment.

### P1-INV-14 — module binding category is explicit

Runtime immutable, runtime mutable, static constant, and declaration are not
collapsed into one `is_const` boolean.

### P1-INV-15 — fields are explicit

No assignment allocates a field slot.

### P1-INV-16 — bare field is immutable

`_x: T` is immutable by default.

### P1-INV-17 — `mut _x` is mutable

Mutable field storage is always explicit.

### P1-INV-18 — immutable field initialization is lifecycle-bound

Ordinary methods cannot initialize or mutate immutable instance fields.

### P1-INV-19 — immutable field is write-once

A legal initialization write is permitted only from definitely-uninitialized
state.

### P1-INV-20 — declaration default initializes the field

A declaration initializer makes an immutable field initialized before
constructor body execution.

### P1-INV-21 — successful construction proves required immutable fields

Every normal successful constructor exit must leave required immutable instance
fields definitely initialized.

### P1-INV-22 — partial constructor state cannot violate field contracts

Calling behavior during construction must not surface raw uninitialized field
storage as if it satisfied the final field type.

### P1-INV-23 — class-side immutable storage has a valid initialization rule

Without a distinct class-init lifecycle, immutable class-side fields require a
declaration initializer.

### P1-INV-24 — field mutability fingerprint remains canonical

Changing field mutability invalidates semantic dependents.

### P1-INV-25 — binding initialization participates in semantic fingerprints

Changing read safety from definite to maybe/uninitialized cannot reuse a stale
callable semantic product.

### P1-INV-26 — corpus migration preserves old runtime intent

Old mutable `let` becomes `let mut`; old runtime immutable `const` becomes
runtime immutable `let`.

### P1-INV-27 — field migration preserves layout

Existing source programs retain old field identities, sides, and slot order
after explicit declarations are inserted, unless an intentional previously
invalid behavior is rejected.

### P1-INV-28 — no hidden legacy runtime `const`

After cutover, no compatibility lowering silently executes `const` as ordinary
runtime storage.

### P1-INV-29 — tooling describes canonical semantics

Hover/highlighting/completion/docs do not teach the retired mutability model.

### P1-INV-30 — no P2 architecture in P1

P1 establishes the static declaration boundary but does not invent a partial
static evaluator.

---

# 10. Non-goals

Do not add:

- general compile-time function/method execution;
- `StaticValue`;
- static block evaluation;
- static pattern matching;
- residual IR;
- compile-time memoization;
- new record/tuple/Pack semantics;
- generalized field keys;
- structural reflection;
- ownership/borrow checking;
- full interprocedural closure definite assignment;
- whole-program global initialization proof;
- transactional constructor semantics;
- a new class-init feature solely to solve P1;
- a broad new IR solely for binding reads.

---

# 11. Expected impact map

## 11.1 Expected source areas

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-modules/src/interface.rs
phalcom-modules/src/fingerprint.rs   (if interface fingerprint lives here)

phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/binding.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/field_lifecycle.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/signature.rs

phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs

phalcom-lsp/src/hover.rs
phalcom-lsp/src/semantic_tokens.rs

phalcom-repl/src/highlighter.rs
phalcom-repl/src/completer.rs   (only if keyword/contextual completion exists)

tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
```

## 11.2 Expected tests

```text
phalcom-ast/tests/parser.rs

phalcom-semantic/tests/semantic/capabilities/bindings.rs
phalcom-semantic/tests/semantic/capabilities/fields.rs
flow-state unit tests near checker/flow/state.rs

phalcom-core unit tests for Value/bytecode/dispatch
phalcom-core language-corpus fixtures for bindings and fields

phalcom-lsp tests for hover / semantic tokens / compiler parity
phalcom-repl focused highlighter/parser-completeness tests
```

Create focused new test files if existing files would become unreasonably mixed.

## 11.3 Expected documentation/state

If `LANG007.C4` directory/checkpoint does not yet exist, create:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
```

Place/copy the requirements analysis and this plan under the same C4 directory
when integrating into the repository.

At completion create:

```text
LANG007.C4.P1-walkthrough.md
LANG007.C4.P1-handoff.md
```

## 11.4 Unexpected-touch rule

Mechanical adjacent touches are allowed.

STOP AND CONSULT before entering a materially different subsystem when that
entry would change:

- semantic authority;
- runtime representation beyond binding/field initialization;
- module linking architecture;
- constructor object lifetime;
- optimizer correctness model;
- static-evaluation architecture.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. `let` runtime immutability.
2. `let mut` runtime mutability.
3. initializer requirement for immutable `let`.
4. initializer-less `let mut`.
5. `const` static phase.
6. uninitialized distinct from `None`.
7. three-state initialization lattice.
8. semantic ownership of initialization proof.
9. runtime fallback for maybe/unprovable reads.
10. branch-free ordinary proven local read.
11. contextual `mut` preference.
12. immutable-by-default field surface.
13. explicit fields only.
14. constructor-lifecycle immutable instance-field initialization.
15. write-once immutable field semantics.
16. no P2 static evaluator in P1.
17. hard corpus cutover.

## 12.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- names of private enums/helpers;
- whether binding and field initialization use one shared enum immediately or
  a short-lived mechanically duplicated enum that is unified in the same plan;
- exact location of normalized binding-semantic helpers;
- exact semantic read-safety product carrier;
- checked-load bytecode naming;
- whether checked local invocation gets a dedicated fused opcode or fusion is
  disabled for checked reads;
- exact one-shot codemod implementation vehicle;
- private test fixture names;
- exact diagnostic wording while preserving meaning;
- exact runtime error Rust variant name;
- layout metadata container type.

## 12.3 VERIFY-FIRST

Before relying on these assumptions, inspect the live repository.

| Assumption | Verify | If false |
|---|---|---|
| parser still has contextual-word helper | `parser.rs` | adapt mechanically; do not reserve `mut` without consultation |
| `BindingState` still has `mutable` | flow state | adapt field names |
| field init lattice unchanged | flow/field lifecycle | map to live equivalent |
| `FlowBindingSummary` participates result fingerprint | analysis/fingerprint | if no canonical summary exists, add state to actual canonical product |
| compiler has a stable way to consume semantic per-expression facts | compiler/semantic lowering boundary | if absent and adding one changes architecture, STOP AND CONSULT |
| `ValueTag` has available internal representation capacity | `value/repr.rs` | if representation change has nonlocal ABI/metadata consequences, consult |
| `Bytecode::Nil` still pushes surface `None` | bytecode + dispatch | adapt to current implementation |
| local read fusion still includes `InvokeLocal` | bytecode optimizer/dispatch | audit live fusions |
| compiler still infers fields from assignment | `class_decl.rs` | if already removed, skip deletion but keep explicit-field tests |
| module interface still uses `is_const` | `phalcom-modules/interface.rs` | replace live equivalent |
| constructor analysis still publishes normal exits | field lifecycle/body analysis | if not, consultation before inventing new lifecycle |
| early-construction ordinary method calls remain possible | constructor lowering/dispatch | if statically prohibited already, verify and use proof instead of redundant runtime guard |
| no class-init lifecycle exists | source/semantic/compiler | if one exists, use canonical lifecycle rather than declaration-default-only rule |
| C4 checkpoint record absent | docs tree | create only if absent |

---

# 13. Global STOP / CONSULT triggers

Stop editing and build an implementation incident packet if any occurs:

1. `let mut` cannot be parsed contextually without materially changing lexical
   semantics.
2. implementing static-phase `const` requires designing P2 evaluator behavior.
3. current module/link architecture cannot represent a non-runtime binding
   category without changing import/export identity semantics.
4. no stable semantic-to-lowering path exists for read-safety facts and adding
   one requires a new cross-layer ownership model.
5. a binding-specific uninitialized runtime state cannot be represented without
   materially changing all `Value` ABI/serialization/native interfaces.
6. a checked read cannot be kept off the proven local hot path.
7. current optimizer/fusion architecture cannot preserve checked-read semantics
   without a material optimizer redesign.
8. assignment-derived field layout is required by a still-normative feature.
9. immutable-field constructor lifecycle conflicts with actual constructor
   semantics in a way not covered by the plan.
10. early-construction `self` use cannot be made sound by the planned minimal
    runtime backstop without changing object lifetime semantics.
11. a class-side initialization lifecycle exists but conflicts with the planned
    default-initializer rule.
12. source migration cannot preserve field slot order using current compiler
    layout authority.
13. accepted spec/tests assert old semantics as current normative behavior and
    no clear supersession path exists.
14. the same nontrivial semantic failure survives one serious correction.
15. an invariant must be weakened to make tests green.
16. a workaround would create duplicate flow, layout, or static-phase truth.
17. scope expands materially beyond the areas in §11.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Up to three coherent correction cycles while each cycle has a concrete local
cause and shows progress.

```text
inspect
identify cause
make one coherent correction
run smallest discriminator
```

## 14.2 Semantic failures

Before editing, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same semantic cause.

If it persists: STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architectural fixes.

Consult immediately.

Never rerun an unchanged failing command unless relevant source/state changed.

---

# 15. Testing surface analysis

| Coverage ID | Dimension | Invariants | Required evidence |
|---|---|---|---|
| P1-CV-01 | parser success | 01–05 | `let`, `let mut`, `const` parse distinctly |
| P1-CV-02 | parser negative | 04–05 | missing initializer errors |
| P1-CV-03 | contextual word | 01 | identifier `mut` remains legal where previously legal |
| P1-CV-04 | local immutability | 02–03 | `let` assignment rejected; `let mut` accepted |
| P1-CV-05 | init lattice | 06–10 | U/M/D joins and writes |
| P1-CV-06 | unreachable paths | 08–10 | dead predecessor does not force Maybe |
| P1-CV-07 | loop flow | 05–10 | maybe/definite loop outcomes correct |
| P1-CV-08 | direct uninit read | 06–08 | compile-time diagnostic |
| P1-CV-09 | maybe read | 09–10 | compiles to checked read |
| P1-CV-10 | proven read | 11–12 | unchecked local bytecode |
| P1-CV-11 | None distinction | 06 | initialized `None` never traps |
| P1-CV-12 | upvalue lifecycle | 13 | closure traps before assignment, succeeds after |
| P1-CV-13 | global lifecycle | 10,14 | uninitialized global checked |
| P1-CV-14 | runtime sentinel privacy | 06 | no print/hash/Some/user escape |
| P1-CV-15 | const phase | 05,28 | no ordinary runtime slot/lowering |
| P1-CV-16 | module category | 14 | interface distinguishes binding categories |
| P1-CV-17 | incremental binding fact | 25 | init change invalidates result |
| P1-CV-18 | field syntax | 16–17 | bare immutable / mut mutable |
| P1-CV-19 | explicit layout | 15 | assignment cannot allocate field |
| P1-CV-20 | field migration parity | 27 | old/new slot identity/order |
| P1-CV-21 | immutable first write | 18–20 | constructor first write accepted |
| P1-CV-22 | immutable second write | 19 | rejected |
| P1-CV-23 | declaration default | 20 | constructor rewrite rejected |
| P1-CV-24 | constructor branch | 21 | partial init exit rejected |
| P1-CV-25 | throwing exit | 21 | non-success path not counted |
| P1-CV-26 | mutable field | 17 | repeated writes accepted |
| P1-CV-27 | early self use | 22 | cannot surface uninitialized field |
| P1-CV-28 | class-side immutable | 23 | no-init rejected unless lifecycle exists |
| P1-CV-29 | field fingerprint | 24 | mutability edit invalidates semantic product |
| P1-CV-30 | corpus rewrite | 26–27 | semantic/layout parity report |
| P1-CV-31 | tooling | 29 | hover/token/highlighter canonical |
| P1-CV-32 | hot-path performance | 11 | proven local load unchanged/no check |
| P1-CV-33 | P2 boundary | 30 | no StaticValue/evaluator introduced |

---

# 16. Verification execution budget

## 16.1 BUILD MODE

Use only:

- exact new unit test;
- exact parser test;
- exact semantic capability filter;
- exact bytecode/VM unit test;
- `cargo check` for the directly edited crate when exhaustive enum changes make
  local compilation the useful discriminator.

Do **not** run workspace tests after every task.

## 16.2 STABILIZE MODE

After coherent semantic/runtime gates:

```text
phalcom-ast test crate
phalcom-semantic focused affected capability modules
phalcom-modules crate tests
phalcom-core affected language/runtime tests
phalcom-lsp focused tests
phalcom-repl focused tests
```

## 16.3 CERTIFY MODE

Only after migration and all focused P1 gates are green:

```text
cargo test -p phalcom-ast
cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

Run workspace-wide verification only if C4 checkpoint policy or repository
guidance requires it for P1 completion.

Do not automatically run workspace clippy/format/test merely because source
changed.

## 16.4 T0–T5 ladder

```text
T0 exact reproducer / new regression
T1 directly affected feature tests
T2 owning subsystem suite
T3 adjacent cross-layer integration
T4 broad affected crates
T5 workspace/release certification
```

The task contracts below say where to stop.

---

# 17. Baseline / unrelated failure policy

Classify failures:

```text
A — definitely caused by P1
B — probably caused by P1
C — unclear
D — clearly baseline/unrelated
```

A/B: fix.

C: one bounded classification pass; record if nonblocking.

D: record and continue.

Never weaken P1 invariants to make broad unrelated tests green.

The durable home is:

```text
LANG007.C4-CHECKPOINT.md
```

under a deferred/baseline failures section.

---

# 18. Implementation sequence summary

```text
T0  establish C4 takeover record and pre-cutover oracle
 ↓
T1  add canonical syntax/AST representation additively
 ↓
T2  build and run AST-aware corpus migration while legacy layout oracle exists
 ↓
T3  flip canonical binding/module/source semantics
 ↓
T4  add binding initialization lattice and semantic read safety
 ↓
T5  add real runtime uninitialized state and checked-read lowering
 ↓
T6  enforce static-phase const boundary
 ↓
T7  delete implicit field layout authority and finalize explicit field surface
 ↓
T8-spec  publish the accepted class-field initialization contract in docs/specs
 ↓
T8a  publish canonical semantic constructor/initializer products
 ↓
T8  enforce immutable-field lifecycle + early-construction soundness
 ↓
T9  align tooling/docs/diagnostics and remove compatibility residue
 ↓
G1  semantic/runtime behavior gate
 ↓
T10 performance + incremental stabilization
 ↓
G2  affected-crate stabilization
 ↓
T11 completion records / checkpoint / walkthrough / handoff
```

Important ordering rule:

> T2 must capture/migrate implicit field layout **before** T7 deletes assignment
> inference.

Do not invert that order.

---

# 19. T0 — Establish C4 takeover state and migration oracle

## Purpose

Create the durable C4 state record and freeze enough pre-cutover evidence to
detect migration mistakes.

## Preconditions

- clean understanding of current worktree;
- C3 closed;
- requirements analysis available.

## Consumes

- C3 checkpoint/handoff;
- P1 requirements analysis;
- live repository status.

## Produces

- C4 checkpoint record;
- pre-cutover corpus census;
- pre-cutover field-layout oracle strategy;
- baseline focused test record.

## Files and symbols

**Read:**

```text
AGENTS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-handoff.md
phalcom-core/tests/README.md
phalcom-semantic/tests/semantic/capabilities/BASELINE_LEDGER.md
```

**Create if absent:**

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
```

**Do not modify implementation yet.**

## T8a — Publish canonical constructor/initializer products

This semantic product is a required precondition for T8 semantic super-call
validation and compiler default lowering. Source-declared constructor
signatures alone are insufficient: ordinary classes with defaults and
`@construct`/`@data` constructors also have canonical initialization paths,
but their identities are currently synthesized only during compiler work.

**Goal:** publish one semantic-owned product for every supported constructor
path so semantic checking and compiler lowering use the same factory,
initializer, field-default, parameter-binding, and parent-initializer facts.

**Depends on:** T8-spec and T7's explicit field identities/layout. Do not begin
T8 super validation or default lowering until this product is published.

**Write-set:**

```text
phalcom-semantic/src/signature.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/tests/semantic/capabilities/fields.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-core/src/modules/compile.rs::compile_analyzed
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/tests/core/language/compiler/lowering_scenarios.rs
```

Extend the canonical callable/snapshot product where it can represent this
contract without duplicating signature authority. Otherwise publish a
`ConstructorInitializationProduct` in the same semantic snapshot that
references canonical callable/signature identities; it must not become a
second source of callable signatures. It must cover source-declared
`@constructor`, compiler-derived `@construct` and `@data`, and the
default-only zero-argument path required by the current class specification.
Do not add synthetic constructors to the source index or create synthetic
source AST declarations merely to make them look source-declared.

T8a also wires this same published snapshot product through the compiler
entrypoint `compile_analyzed` into semantic lowering. The compiler consumer
must receive constructor identities, signatures, field/default mappings, and
origin from the semantic snapshot; it may not reconstruct these by scanning
source AST or expanding attributes. T8's later lowering work emits the
initializer behavior from that canonical input.

Each constructor product must publish, directly or by canonical references:

- the declaring owner and stable constructor identity;
- the class-side factory callable identity and its canonical selector,
  parameter labels/types, and result signature;
- the paired same-instance initializer identity and its parameter signature;
- the ordered `FieldId`s whose declaration defaults this owner initializes;
- generated constructor parameter-to-`FieldId` mappings, preserving field and
  parameter order;
- the constructor origin (source, `@construct`, `@data`, or default-only), so
  lowering can reuse existing generated assignment behavior and never emit a
  second default prologue;
- canonical callable identities/signatures for the direct parent's constructor
  candidates, which the T8 semantic checker uses to resolve an explicit
  `super.new(...)` call to exactly one paired parent initializer identity.

The checked direct-parent initializer identity is part of the canonical
semantic construction result too: T8's semantic super-call product records
the explicit call-to-parent-initializer link using these published
constructor products. The compiler consumes that resolved link; it does not
perform parent lookup again.

Use the repository's canonical declaration/callable identity policy for
stable identities. A generated identity must remain stable for an unchanged
owner and constructor shape across incremental analysis; do not derive it
from source spans, compiler-local counters, selector spelling alone, or
allocation order.

Constructor products depend on the declaring class's field identities,
defaults and their expressions, constructor/derive declarations and
attributes, and relevant semantic type products. The resolved parent link
depends on the subclass's superclass declaration, explicit super-call syntax
and argument types, and the direct parent's constructor products.
Fingerprints/dependency edges must invalidate affected constructor products
when their inputs change, and invalidate dependent subclass super-call
results when the direct parent's products change. Removal or loss of a valid
generated/default-only path must remove the published product and invalidate
its consumers; no stale constructor identity may survive.

**Verification:** add semantic product assertions for source-declared,
`@construct`, `@data`, and default-only paths; verify factory/initializer
pairing, ordered default field identities, and generated parameter-to-field
mapping. Add incremental cases changing and removing parent
defaults/fields/constructor attributes and verify that the parent product is
updated/removed and dependent semantic consumers are invalidated. T8's
semantic super-call tests must verify explicit direct-parent identity and the
existing three-level explicit-super fixture remains a required regression.
Add a compiler projection regression proving `compile_analyzed` passes the
published product to semantic lowering intact, with no source-side identity or
field-mapping reconstruction.

**Acceptance condition:** compiler class construction consumes this published
semantic product without independently deriving a constructor identity,
signature, parent target, or field/parameter mapping. Do not use source-only
validation or a compiler-local synthetic fallback.

### T8a amendment — semantic-product-driven class construction

Adopted after the 2026-10-04 constructor-projection consultation recorded in
`LANG007.C4-CHECKPOINT.md`. Product publication and `compile_analyzed` /
semantic-lowering transport are prerequisites, not completion: generated
constructor materialization must consume those products too.

At the class compiler boundary, obtain the declaring owner's constructor
products before constructor materialization. Replace field scanning in
`derive_construct` with projection from canonical `default_fields` and
`parameter_fields` order. Source syntax may supply only the default expression
for a canonical `FieldId`. `@data` may continue deriving non-constructor
methods, but its constructor must come from `ConstructorOrigin::Data`.
Materialize default-only factory and initializer paths from their products.
`lower_constructors` must consume product factory/initializer identities and
signatures; it must not derive a selector from an AST name or parameters.

Defaults execute once. Generated `@construct`/`@data` assignment behavior
remains their only default prologue; source-declared/default-only construction
uses the canonical initializer. The compiler consumes the semantic resolved
parent initializer link and the canonical `CallableSignatureTable`; transport
the existing table if needed instead of copying or rebuilding it. If the
existing `MethodDef` path cannot preserve product-owned identities, use a
compiler-only lowering projection, never a synthesized source declaration or
a fallback identity/mapping derivation.

Add consumer-level codegen regressions for generated and default-only paths,
including product-owned identity/signature, field/default order, parameter
mapping, explicit-parent selection, and once-only defaults. T8a remains open
until class construction uses the products end to end. If implementation
requires changing semantic ownership, product identity policy, accepted
construction semantics, or the compiler/semantic contract above, stop and
consult rather than redesigning locally.

## T8 execution order after T8-spec

```text
T8a publish canonical constructor/initializer products
     and wire the snapshot product through compile_analyzed/semantic_lowering
  -> T8 semantic checking consumes them for explicit-super resolution,
     constructor lifecycle and successful-exit obligations
  -> T8 compiler codegen/runtime consumes the same products and resolved
     parent link, preserving checked-read and raw-allocation behavior
```

The existing §§27.1–27.7 define the invariants for T8. They do not
authorize either consumer to reconstruct constructor identity or signature.
T8 semantic lifecycle validation and field/default codegen must not start
before T8a's semantic product and compiler input wiring are established.

## Required implementation shape

Start the C4 checkpoint with:

```text
Objective
Plans ledger
Invariant ledger
Verification ledger
Consultation/amendment ledger
Deferred/baseline failures
Next action
```

Record P1 as active only when implementation actually begins.

Capture repository state:

```sh
git status --short
git rev-parse HEAD
```

Capture source census with `rg` or an equivalent script:

```text
let declarations
const declarations
field const declarations
bare field declarations
contextual mut fields
initializer-less current let
```

Do not use raw counts as a semantic migration algorithm.

### Field-layout oracle

Before deleting implicit field inference, provide a way to compare old layout
against migrated explicit declarations.

Preferred:

- a focused Rust test/helper calling the existing class-layout collection path;
- or a temporary migration-side probe that prints deterministic:
  - class;
  - side;
  - field names;
  - slot order;
  - count.

Do not snapshot heap object IDs or process-local intern IDs.

## Forbidden approaches

- no implementation edits before worktree inspection;
- no broad source rewrite;
- no regex-only migration;
- no checkpoint marked complete merely because C3 is complete.

## Test changes required

None required yet.

## Tests to run now

Only baseline discriminators already known to be cheap, e.g. existing binding
and field semantic test filters if available.

Record failures without fixing unrelated baseline issues.

## Acceptance

- starting revision recorded;
- C4 checkpoint exists;
- migration oracle approach is concrete;
- no semantic source changed.

## Local STOP / CONSULT triggers

- current class layout cannot be deterministically observed without materially
  redesigning compiler state;
- repository has an active C4 checkpoint with conflicting architecture.

## Checkpoint update

Yes. This is the C4 takeover event.

---

# 20. T1 — Add canonical binding and field surface representation

## Purpose

Make the parser/AST capable of representing the target syntax before rewriting
the corpus.

This task is intentionally **additive**. It does not yet complete the semantic
cutover.

## Preconditions

T0 complete.

## Consumes

- current contextual `mut` parser machinery;
- current `BindingKind`;
- current `FieldDef.mutable`.

## Produces

- `BindingKind::LetMut`;
- parsed `let mut`;
- parser-level canonical field forms;
- normalized binding helper API;
- parser RED/GREEN coverage.

## Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/tests/parser.rs
```

**Modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/parser.rs
```

Touch `token.rs` / `lexer.rs` only if compilation mechanically requires it.

## Required implementation shape

### 20.1 BindingKind

Target:

```rust
pub enum BindingKind {
    Let,
    LetMut,
    Const,
}
```

Do not encode `LetMut` as a boolean bolted onto `LetBinding`.

The syntax category should be explicit for source tools and migration.

### 20.2 Normalized helpers

Add one canonical projection, location mechanically flexible:

```rust
enum BindingPhase {
    Runtime,
    Static,
}

enum RuntimeMutability {
    Immutable,
    Mutable,
}
```

or equivalent methods:

```rust
BindingKind::phase()
BindingKind::runtime_mutability()
BindingKind::requires_initializer()
```

Required mapping:

```text
Let     -> Runtime / Immutable
LetMut  -> Runtime / Mutable
Const   -> Static
```

Do not let every downstream consumer reimplement matches.

### 20.3 Parser

After consuming `let`, inspect contextual word `mut`.

Reuse the same contextual-word primitive already used for member `mut`.

Do not add `Token::Mut` in this task.

### 20.4 Initializer grammar

Parser must distinguish:

```telos
let x = e           valid
let mut x = e       valid
let mut x           valid simple-name form
let mut x: T        valid simple-name form
let x               invalid
const x             invalid
```

If semantic diagnostics are the repository convention for missing initializer,
the parser may preserve AST and defer the diagnostic, but the final source
behavior must match the invariant.

Destructuring without initializer remains invalid.

### 20.5 Fields

Make parser tests accept:

```telos
_x: T
mut _x: T
```

Do not yet remove every legacy `const _x` parse path until T2 has migrated the
corpus if doing so would prevent migration tooling from parsing the old tree.

The final removal occurs by T7/T9.

This is an intentional short transitional parser compatibility inside the
implementation sequence, not a final language compatibility promise.

## Forbidden approaches

- no reserved `mut` token;
- no semantic meaning derived from token text in compiler;
- no two-booleans `is_const/is_mut` model;
- no runtime changes here.

## Test changes required

Create focused tests named approximately:

```text
parse_let_is_runtime_immutable_kind
parse_let_mut_is_distinct_binding_kind
parse_initializerless_let_mut_name
reject_initializerless_immutable_let
reject_initializerless_const
reject_initializerless_mut_destructure
parse_bare_field_as_immutable_target_form
parse_contextual_mut_field
contextual_mut_does_not_become_global_keyword
```

Names may follow local conventions.

## Tests to run now

```sh
cargo test -p phalcom-ast --test parser let_mut -- --nocapture
cargo test -p phalcom-ast --test parser initializer -- --nocapture
cargo test -p phalcom-ast --test parser field -- --nocapture
cargo check -p phalcom-ast
```

If enum exhaustiveness breaks downstream crates, use `cargo check` on direct
dependents only as needed to make the additive AST compile.

## Tests explicitly deferred

- semantic binding tests;
- runtime tests;
- full corpus;
- workspace tests.

## Acceptance

- target syntax has an explicit AST;
- `mut` remains contextual;
- parser-level tests distinguish all three binding kinds;
- the old corpus can still be parsed long enough for T2 migration.

## Local STOP / CONSULT triggers

- contextual `mut` cannot distinguish `let mut` without reserving the word;
- parser architecture forces user-visible ambiguity not covered here.

## Checkpoint update

Record `BindingKind::LetMut` as a durable source-interface change after focused
parser green.

Do not commit a semantically misleading intermediate cutover unless T2/T3 are
completed in the same coherent commit sequence.

---

# 21. T2 — Perform AST-aware binding/field corpus migration

## Purpose

Rewrite existing source to the new spelling while the old compiler/layout
behavior can still serve as an oracle.

## Preconditions

T1 parser can parse the target forms.

Implicit field layout inference still exists.

## Consumes

- legacy source meaning;
- old class layout oracle;
- new AST syntax.

## Produces

- repository source expressed in target binding/field syntax;
- explicit declarations for legacy inferred fields;
- layout parity report;
- idempotent migration evidence.

## Migration mapping

### Bindings

```text
old runtime mutable:
    let x = e
 -> let mut x = e

old initializer-less mutable:
    let x
 -> let mut x

old runtime immutable:
    const x = e
 -> let x = e
```

Do not translate old runtime immutable `const` to new static `const`.

### Declared fields

```text
old bare mutable field:
    _x: T
 -> mut _x: T

old immutable field:
    const _x: T
 -> _x: T

already-contextual mut field:
    mut _x: T
 -> mut _x: T
```

### Implicit fields

For every field old layout would infer solely from assignment:

```text
insert explicit `mut` declaration
```

Preserve side and old slot order.

## Files

Potentially every current `.ph` source/fixture/doc-example that is compiled or
treated as canonical source.

Likely major roots:

```text
phalcom-core/core/
examples/
phalcom-core/tests/
phalcom-semantic/tests/fixtures or inline source strings
phalcom-lsp/tests/fixtures
phalcom-repl tests/fixtures
docs current code examples where canonical
```

Historical docs should not be bulk-rewritten unless repository policy marks
them current.

## Required implementation shape

### 21.1 Migration tool

Use an AST-aware or syntax-aware transformation.

Acceptable implementation vehicles:

- small temporary Rust migration tool using `phalcom-ast`;
- a retained repository migration utility if repository convention supports it;
- structured script that uses parser ranges and applies non-overlapping edits
  from end-to-start.

A regex-only source swap is forbidden.

### 21.2 Context classification

The migrator must distinguish:

- top-level/local binding;
- field declaration;
- unrelated `const`;
- comments/strings/docs;
- contextual `mut` identifier occurrences.

### 21.3 Implicit field insertion

For each class:

1. ask the old layout collector for effective own fields/side/order;
2. collect explicit declarations already present;
3. compute old inferred-only remainder;
4. insert explicit mutable declarations in an order that reproduces old slots;
5. do not synthesize declaration initializers;
6. preserve superclass prefix untouched.

If exact source placement is ambiguous, choose a deterministic class-body
location consistent with formatter/style and verify resulting slot order.

### 21.4 Idempotence

Run the migration twice.

Second run must produce no diff.

### 21.5 Parity report

Before semantics are flipped, compare old/new normalized intent:

```text
binding:
    phase under old semantics
    mutability
    initializer presence
    pattern shape

field:
    owner
    side
    field name
    slot order
    mutability
```

For old `const` bindings, parity means:

```text
old runtime immutable == new `let` runtime immutable target intent
```

not textual identity.

## Forbidden approaches

- no `sed` two-pass swap;
- no global textual `const -> let`;
- no inferred field order based on hash-map iteration;
- no changing field declaration initializer semantics;
- no leaving old source spellings “for compatibility” in active core/test code.

## Test changes required

Add migration utility tests if utility is committed.

Add one fixture containing all ambiguous forms:

```text
comment/string containing "let"/"const"
local binding
top-level binding
field declaration
method assignment to implicit field
class-side field assignment
identifier named mut where legal
```

## Tests to run now

During migration:

```sh
cargo check -p phalcom-ast
```

plus the migration tool's own exact test.

Run the field layout parity harness.

Do **not** run the full language corpus while the semantic flip is incomplete.

## Acceptance

- active `.ph` corpus uses new runtime binding spellings;
- active fields use new mutability spellings;
- legacy inferred fields are explicitly declared;
- layout parity passes;
- migration is idempotent;
- no old runtime immutable `const` remains in active runtime source except cases
  intentionally intended to become actual static constants later.

## Local STOP / CONSULT triggers

- field slot parity cannot be preserved deterministically;
- a current implicit-field case depends on assignment-dependent dynamic layout;
- migration encounters `const` uses whose runtime/static intended meaning cannot
  be mechanically determined.

## Checkpoint update

Record corpus migration census and layout-parity evidence.

Do not mark semantic cutover complete yet.

---

# 22. T3 — Flip canonical binding/module/source semantics

## Purpose

Make the compiler and semantic/source/module model interpret the migrated
syntax according to C4.

## Preconditions

T2 source migration is complete enough that the active corpus no longer relies
on old runtime spelling.

## Consumes

- normalized BindingKind helper;
- migrated source.

## Produces

- runtime immutable `let`;
- runtime mutable `let mut`;
- static-phase `const` category;
- corrected source-index mutability;
- explicit module binding semantics.

## Files and symbols

**Modify likely:**

```text
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/builder.rs

phalcom-modules/src/interface.rs
phalcom-modules/src/fingerprint.rs or actual interface fingerprint owner

phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/compiler/attributes.rs
```

## Required implementation shape

### 22.1 Remove direct old matches

Replace logic equivalent to:

```rust
binding.kind == BindingKind::Let
matches!(binding.kind, BindingKind::Let)
```

when used as mutability with the normalized helper.

Final mapping:

```text
Let     runtime immutable
LetMut  runtime mutable
Const   static
```

### 22.2 SourceBindingKind

Prefer explicit source taxonomy:

```text
TopLevelLet
TopLevelLetMut
TopLevelConst
LocalLet
LocalLetMut
LocalConst
```

or an equivalent representation that preserves source form and phase.

`SourceBindingInfo.mutable` must match semantic meaning.

### 22.3 Implicit binder parity

Audit source index versus semantic binding creation for:

```text
method parameters
setter/index parameters
closure parameters
for bindings
for index bindings
pattern binders
```

Do not silently carry historical `mutable: true` if semantic binders are
immutable.

If a language construct is intentionally mutable, preserve it explicitly and
document why.

### 22.4 Module surface

Replace:

```rust
is_const: bool
```

as semantic category.

Preferred:

```rust
enum ModuleBindingSemantics {
    Runtime { mutable: bool },
    StaticConstant,
    Declaration,
}
```

Exact naming may adapt.

Classes/data/traits/type aliases must not be mislabeled “const”.

### 22.5 Compiler local/global tables

Only `LetMut` produces mutable local/global binding state.

`Let` produces immutable runtime storage.

`Const` must not be treated as runtime storage here; full lowering barrier lands
in T6.

### 22.6 Generated bindings

Audit AST generated by compiler transforms.

Classify each synthetic binding:

```text
single assignment -> Let
written later     -> LetMut
```

Do not mechanically convert generated `Let` to `LetMut`.

## Forbidden approaches

- no new `is_const` boolean under another name;
- no source-index/compiler semantics divergent from checker;
- no “all patterns mutable” shortcut unless language semantics explicitly
  require it;
- no static evaluator.

## Test changes required

Focused semantic/source/module tests:

```text
let_binding_is_immutable
let_mut_binding_is_mutable
source_index_reports_let_mut_only_as_mutable
module_surface_distinguishes_runtime_immutable_mutable_static
field_declaration_not_mislabeled_const
```

## Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic bindings -- --nocapture
cargo test -p phalcom-modules -- --nocapture
cargo check -p phalcom-core
```

If test filtering differs in live harness, run the smallest equivalent.

## Tests explicitly deferred

- uninitialized runtime tests;
- field lifecycle;
- full corpus.

## Acceptance

- assignment to `let` is rejected through canonical mutability;
- assignment to `let mut` reaches mutable transfer;
- module/source index expose correct category;
- old mutability meaning is no longer encoded by `BindingKind::Let`.

## Local STOP / CONSULT triggers

- module linker requires runtime slot identity for every declaration;
- static constant category cannot be preserved without cross-module static
  architecture from P8.

In that case, consult; do not fake static constants as runtime immutable.

## Checkpoint update

Record normalized binding/module semantics as a durable interface.

---

# 23. T4 — Add binding initialization state and semantic read safety

## Purpose

Implement O1 definite initialization in the existing flow engine.

## Preconditions

T3 mutability semantics are canonical.

## Consumes

- `FlowState`;
- binding declaration seeds;
- branch join;
- loop widening;
- callable analysis/fingerprints.

## Produces

- independent binding initialization lattice;
- assignment transition;
- branch/loop propagation;
- guaranteed-uninitialized diagnostic;
- maybe-read safety classification;
- stable lowering fact.

## Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/binding.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/db/fingerprint.rs
```

## Required implementation shape

### 23.1 Initialization type

Preferred end state:

```rust
pub enum InitializationState {
    Uninitialized,
    MaybeInitialized,
    DefinitelyInitialized,
}
```

Generalize/rename existing field initialization if mechanical churn is
reasonable.

If using existing `FieldInitialization` temporarily, do not allow binding and
field semantics to diverge.

### 23.2 BindingState

Add:

```rust
initialization: InitializationState
```

Do not encode initialization solely through `TypeKnowledge`.

### 23.3 Declaration seeds

```text
let x = e:
    mutable = false
    init = DefinitelyInitialized

let mut x = e:
    mutable = true
    init = DefinitelyInitialized

let mut x:
    mutable = true
    init = Uninitialized

let mut x: T:
    mutable = true
    init = Uninitialized
    contract = T
```

For uninitialized state:

```text
current knowledge = Unknown(MissingInitializer) or live equivalent
```

but the initialization enum is authoritative.

### 23.4 Write transfer

A semantically valid mutable assignment transitions:

```text
initialization -> DefinitelyInitialized
current -> RHS fact
version += 1
facts invalidated
```

Do not mark initialized after a causally invalid assignment.

### 23.5 Join

Use the same lattice as field initialization:

```text
D + D -> D
U + U -> U
otherwise -> M
```

Exclude unreachable predecessors using the current flow rule.

### 23.6 Loop widening/fixpoint

Initialization participates in header/backedge comparison and convergence.

Do not add a separate loop solver.

### 23.7 Read classification

At each binding read, publish one canonical semantic outcome:

```rust
enum BindingReadSafety {
    ProvenInitialized,
    DefinitelyUninitialized,
    RuntimeCheckRequired,
}
```

Naming/location flexible.

Rules:

```text
D -> ProvenInitialized
U -> DefinitelyUninitialized
M -> RuntimeCheckRequired
missing/cross-boundary proof -> RuntimeCheckRequired where source is legal
```

Definitely-uninitialized read emits a semantic diagnostic.

### 23.8 Lowering product

The read-safety fact must be published in a semantic product the compiler can
consume by stable source/expression/binding identity.

Do not let lowering inspect mutable worker state.

### 23.9 Fingerprints

Include initialization/read-safety facts wherever callable semantic result
fingerprints are computed.

Changing:

```text
D -> M
```

must invalidate lowering/tooling consumers.

## Forbidden approaches

- no initialization inferred from `current.ty().is_some()`;
- no CFG implementation in compiler;
- no strict rejection of `MaybeInitialized`;
- no treating Maybe as `Option<T>`;
- no widening Maybe to Dynamic merely to avoid runtime check.

## Test changes required

Low-level flow tests:

```text
binding_init_join_uninitialized
binding_init_join_definite
binding_init_join_mixed_is_maybe
binding_init_join_ignores_unreachable
assignment_marks_binding_definitely_initialized
loop_may_initialize_binding_is_maybe
loop_all_exits_initialize_binding_is_definite
```

Semantic source tests:

```text
read_definitely_uninitialized_is_diagnostic
read_after_assignment_is_proven
conditional_assignment_requires_runtime_check
explicit_none_is_definitely_initialized
immutable_let_reassignment_is_diagnostic
```

Incremental test:

```text
changing_only_initialization_safety_changes_callable_product_fingerprint
```

## Tests to run now

```sh
cargo test -p phalcom-semantic binding_init -- --nocapture
cargo test -p phalcom-semantic --test semantic bindings -- --nocapture
```

Use exact live filters.

Do not run runtime tests yet.

## Acceptance

- lattice laws green;
- source read classification correct;
- semantic result contains a stable lowering fact;
- fingerprint changes with read safety.

## Local STOP / CONSULT triggers

- semantic callable analysis has no stable per-expression/result carrier and
  publishing one requires material query architecture changes;
- loop fixed point cannot include initialization without altering relation
  semantics.

## Checkpoint update

Record binding initialization/read safety as durable semantic facts.

---

# 24. T5 — Implement real runtime uninitialized binding state and checked reads

## Purpose

Make runtime fallback sound without penalizing proven reads.

## Preconditions

T4 publishes read safety.

## Consumes

- semantic read classification;
- current Value representation;
- local/global/upvalue read bytecodes.

## Produces

- private uninitialized-binding runtime state;
- checked local/global/upvalue reads;
- branch-free safe local reads;
- runtime error;
- compiler lowering that consumes semantic safety.

## Files and symbols

**Read/modify likely:**

```text
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/nil.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/error.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/heap/module.rs
phalcom-core/src/heap/upvalue.rs
```

Audit optimizer/fusion code that transforms:

```text
GetLocal + Invoke
```

or equivalent.

## Required implementation shape

### 24.1 Runtime representation

Preferred:

```rust
ValueTag::UninitializedBinding
```

with private constructor:

```rust
Value::uninitialized_binding()
```

Alternative internal representation is allowed only if:

- zero allocation;
- not `None`;
- does not collide with generic allocator `Nil`;
- traps reliably at checked binding reads.

### 24.2 Privacy

The sentinel must not be semantically usable.

Add defensive assertions/rejections in any path where it could otherwise reach:

- `Some` wrapping;
- user display/string conversion;
- equality/hash;
- native send;
- return boundary;
- collection insertion if reachable.

The intended design should make most such paths unreachable, with assertions
rather than branches on every normal value operation.

### 24.3 Seed lowering

Initializer-less:

```telos
let mut x
```

must seed storage with the binding-specific uninitialized state.

Do not emit guest `None`.

For locals, use a bytecode or compiler stack-seeding path that places the
private sentinel.

For globals, module slot initialization must distinguish declared-uninitialized
binding from missing/undefined name.

### 24.4 Checked loads

Conceptually:

```text
GetLocalChecked
GetUpvalueChecked
GetGlobalChecked
```

Exact names flexible.

Checked load:

```text
if value == UninitializedBinding:
    raise UninitializedBindingError
else:
    push value unchanged
```

It must not surface `None`.

### 24.5 Unchecked loads

Existing ordinary:

```text
GetLocal
```

remains unchecked for `ProvenInitialized`.

Do not insert a sentinel branch there.

Global/upvalue policy may conservatively use checked forms where proof is not
available.

### 24.6 Compiler handoff

The compiler must use the semantic read-safety product:

```text
ProvenInitialized -> unchecked
RuntimeCheckRequired -> checked
DefinitelyUninitialized -> semantic phase should already reject; lowering
                            treats arrival as invariant violation
```

Do not re-run flow.

### 24.7 Fusion audit

If optimizer has:

```text
GetLocal + Invoke -> InvokeLocal
```

then:

- safe read may fuse;
- checked read may not fuse into unchecked `InvokeLocal`.

Options:

1. `InvokeLocalChecked`;
2. leave checked sequence unfused.

Prefer the simplest correct approach unless benchmark evidence justifies a new
fused opcode.

### 24.8 Upvalue scenario

Preserve one shared cell:

```telos
let mut x: Int
let f = { x }
f()    // trap
x = 1
f()    // 1
```

Do not copy “uninitialized” as a permanent closure state if the upvalue remains
open/shared.

### 24.9 Runtime error

Add a distinct runtime error, e.g.:

```text
UninitializedBindingError
```

Do not reuse `UndefinedVariable`.

## Forbidden approaches

- no `None` sentinel;
- no allocation per uninitialized binding;
- no check in every `GetLocal`;
- no compiler-side flow reanalysis;
- no converting uninitialized to Dynamic.

## Test changes required

Value/runtime unit tests:

```text
uninitialized_binding_is_not_none
uninitialized_binding_is_not_nil_if_separate_tag
uninitialized_binding_cannot_wrap_some
checked_local_traps
checked_local_after_assignment_succeeds
checked_upvalue_traps_then_succeeds
checked_global_traps
explicit_none_checked_read_succeeds
```

Bytecode/lowering tests:

```text
proven_local_uses_unchecked_getlocal
maybe_local_uses_checked_getlocal
checked_getlocal_is_not_fused_into_unchecked_invoke_local
```

## Tests to run now

```sh
cargo test -p phalcom-core uninitialized_binding -- --nocapture
cargo test -p phalcom-core checked_local -- --nocapture
cargo test -p phalcom-core checked_upvalue -- --nocapture
cargo test -p phalcom-core checked_global -- --nocapture
cargo test -p phalcom-core invoke_local -- --nocapture
```

Then one semantic-to-runtime fixture for Maybe.

## Acceptance

- uninitialized is not `None`;
- direct proven invalid read rejected before runtime;
- uncertain read traps only when actually uninitialized;
- assignment then read succeeds;
- safe local read remains unchecked;
- optimizer preserves checks.

## Local STOP / CONSULT triggers

- Value representation change affects public/native ABI nonlocally;
- checked read cannot be expressed without branch on every normal local load;
- compiler cannot consume semantic read safety through existing lowering
  architecture.

## Checkpoint update

Record runtime sentinel/check model and hot-path invariant after green.

---

# 25. T6 — Enforce the static-phase `const` barrier

## Purpose

Complete the runtime/static declaration split without implementing P2.

## Preconditions

- migrated runtime immutable bindings use `let`;
- module binding category can represent static constant.

## Consumes

- `BindingKind::Const`;
- module/static declaration category.

## Produces

- no ordinary runtime `const` local/global lowering;
- explicit P2 handoff seam;
- targeted diagnostics for unsupported static consumption if needed.

## Files and symbols

Likely:

```text
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/source_index/*
phalcom-modules/src/interface.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/compiler/attributes.rs
```

## Required implementation shape

### 25.1 Declaration

`const`:

- requires initializer;
- is indexed as static declaration;
- is not assigned runtime mutability.

### 25.2 Lowering barrier

No ordinary path:

```text
compile_expr(initializer)
add_local / DefineGlobal
```

may represent `const` as its semantic implementation.

Until P2 supplies static evaluation/materialization:

- declarations may be accepted/indexed where appropriate;
- runtime use requiring value materialization must produce a targeted
  “static constant materialization not yet available in C4.P1” internal/feature
  gate diagnostic, or the live repository's standard unsupported-feature
  outcome.

Do not execute initializer at runtime.

### 25.3 Imports/exports

Preserve static category through interfaces.

If importer cannot consume value until P2, keep the identity/category and fail
at value-materialization use, not by lying that it is runtime storage.

### 25.4 Optimizer

`Const` is not an ordinary immutable product-optimization candidate.

`let` is now the immutable runtime candidate.

## Forbidden approaches

- no old const runtime slot;
- no evaluator stub that runs arbitrary Telos;
- no compile-time VM;
- no “temporary runtime implementation” after this gate.

## Test changes required

```text
const_is_static_module_surface
const_initializer_not_emitted_as_runtime_define
const_does_not_allocate_runtime_local
const_without_initializer_rejected
runtime_use_of_unmaterialized_static_const_fails_closed
```

If P2-ready static constants already have a narrower current mechanism, adapt
mechanically but do not expand it here.

## Tests to run now

Focused AST/semantic/module/compiler tests only.

Inspect generated bytecode for a minimal module containing `const`.

## Acceptance

- P1-INV-05 and P1-INV-28 proven;
- no runtime immutable const fallback remains;
- P2 has a clear static declaration seam.

## Local STOP / CONSULT triggers

- current bootstrap/core requires runtime `const` in a way migration cannot
  replace with `let`;
- module initialization architecture requires evaluating static initializer at
  runtime.

## Checkpoint update

Record static-phase const boundary as durable.

---

# 26. T7 — Delete implicit field layout authority and finalize field syntax

## Purpose

Make explicit field declarations the sole stored-field authority.

## Preconditions

- T2 inserted declarations for old inferred fields;
- layout parity evidence exists.

## Consumes

- migrated explicit declarations;
- existing `FieldSemanticSignature`;
- class layout builder.

## Produces

- assignment cannot create fields;
- canonical immutable/mutable field syntax;
- undeclared-field diagnostics;
- explicit layout only.

## Files and symbols

Likely:

```text
phalcom-ast/src/parser.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/diagnostic.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/expr.rs
```

## Required implementation shape

### 26.1 Parser final state

```text
bare field -> mutable=false
mut field  -> mutable=true
```

Remove final legacy `const _field` acceptance from canonical grammar, or issue
the repository's targeted migration diagnostic if parser recovery policy prefers
a diagnostic node.

### 26.2 Layout source

Delete/disable field discovery based on:

```text
assignment scan
collect_assigned_fields...
```

or live equivalent.

The layout builder should walk explicit `ClassMember::Field` only.

### 26.3 Unknown field write

```telos
class C {
    f() { _missing = 1 }
}
```

must not create layout.

Emit semantic undeclared-field diagnostic.

Compiler should fail closed if invalid AST reaches lowering.

### 26.4 Field signature

Continue to use:

```rust
FieldSemanticSignature.mutable
```

No new mutability store.

### 26.5 Field ordering

Own-class slot order follows explicit declaration order.

Superclass prefix behavior remains unchanged.

## Forbidden approaches

- no “explicit first, infer missing as fallback”;
- no assignment-based hidden slot creation;
- no second field registry;
- no changing field identity from current `FieldId` unless separately required.

## Test changes required

```text
bare_field_signature_is_immutable
mut_field_signature_is_mutable
assignment_to_undeclared_field_is_diagnostic
assignment_does_not_change_layout
explicit_field_layout_preserves_declaration_order
migrated_layout_matches_pre_cutover_oracle
legacy_const_field_spelling_rejected_or_migration_diagnostic
```

## Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-core field_layout -- --nocapture
```

Run layout parity harness again after inference deletion.

## Acceptance

- no implicit field storage path remains;
- old/new migrated layout parity still holds;
- canonical field mutability is explicit and fingerprinted.

## Local STOP / CONSULT triggers

- compiler uses assignment discovery for a non-field-layout feature that cannot
  be separated mechanically;
- explicit field order cannot reproduce legacy slots for migrated corpus.

## Checkpoint update

Record explicit-field authority and parity evidence.

---

# 27. T8 — Enforce immutable-field construction lifecycle and early-use soundness

## Purpose

Upgrade existing field lifecycle analysis from descriptive facts to the frozen
immutable-field invariant.

## Preconditions

- explicit fields only;
- field mutability canonical;
- existing field lifecycle engine green;
- complete T8-spec before resuming T8 source implementation; C4-I-05 is
  accepted, but ordinary instance-default execution is not yet stated in the
  current normative specification corpus.

## Consumes

- `FieldLifecycleTable`;
- `FieldInitialization`;
- constructor normal exits;
- field assignment semantic resolution;
- runtime field storage sentinel.

## Produces

- constructor-only initialization writes;
- exactly-once immutable writes;
- constructor-exit obligations;
- early-construction runtime soundness;
- class-side immutable rule.

## Files and symbols

Likely:

```text
phalcom-semantic/src/checker/field_lifecycle.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/semantic_shard.rs

phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/bytecode.rs              (if checked field read opcode added)
phalcom-core/src/vm/dispatch.rs
```

## T8-spec — Publish the accepted field/construction contract first

This is a required documentation subtask and a precondition for T8 source
implementation. The current `docs/specs/objects/` corpus has no class/field
specification; `objects/values.md` leaves class semantics to a specialized
specification. Add `docs/specs/objects/classes.md` as that owner and register it
in `docs/specs/README.md`. Align the VM realization in:

```text
docs/specs/vm/values-and-objects.md
docs/specs/vm/bytecodes.md
```

**Goal:** make accepted C4 class-field initialization behavior part of the
current normative specification before source implementation resumes.

**Depends on:** T7's explicit field surface; C4-I-05; accepted ADR-0051
constructor lowering; the human-resolved explicit-super decision recorded in
the C4 checkpoint.

**Write-set:** exactly the four `docs/specs/` paths listed above. The class
specification is a new file; do not broaden the task into unrelated class
semantics.

The new language specification must state only the accepted C4 behavior needed
for this lifecycle:

- Every declared instance-field default is evaluated exactly once per object.
- Defaults run in the declaring owner's canonical instance initializer, in
  field declaration order, before that owner's remaining user-authored
  constructor body. If parent initialization is required, the first explicit
  `super.new(...)` chain call completes before the child's defaults; a root
  initializer with no parent chain runs its defaults at entry.
- A declaration default is prior initialization; the constructor body cannot
  write that immutable field again.
- A successful subclass construction explicitly invokes the required parent
  initializer exactly once with `super.new(...)`; omission is an invalid
  constructor path. P1 does not add implicit superclass chaining.
- Parent initialization completes before derived-owner defaults and body.
  Inherited fields/defaults are initialized only by their declaring owner's
  initializer; the child never replays them.
- Raw allocation does not evaluate defaults. A raw/incomplete instance remains
  under construction, and checked immutable reads trap until initialization.
- `@construct` and `@data` retain their generated default assignments as the
  canonical prologue; lowering must not add a duplicate prologue.

The class language specification owns the source-visible rule. The VM value
spec must distinguish an unread immutable field from guest-visible `None`, and
the bytecode spec must define checked immutable-field reads and state that
`NewInstance` allocates only. The implementation plan is not the permanent
normative source.

### T8-spec acceptance

- `docs/specs/objects/classes.md` is linked from the current specs README and
  claims only the accepted lifecycle rules above.
- VM wording no longer implies that a checked immutable-field read may surface
  `None` or raw `Nil` during incomplete construction.
- The docs are checked against C4-I-05, ADR-0051's constructor lowering
  contract, and live `@construct`/`@data` expansion.
- No T8 source implementation resumes before this acceptance point.

**Risk:** wording in the foundational values or VM specifications could
accidentally permit raw `Nil` or `None` to escape while an immutable field is
incomplete. Keep the source rule in the new class specification and update VM
realization text consistently.

**Review strategy:** manually check all four links/authority entries and
cross-check the source-visible rule against C4-I-05, ADR-0051, and the existing
same-instance `super.new()` behavior. This is a prose-only subtask; no Cargo
validation is applicable.

**Forward-looking constraint:** specify owner-local initialization and explicit
super chaining without requiring defaults for mutable fields or defining a
general class-initialization VM. Preserve raw allocation as a distinct,
incomplete stage.

## Required implementation shape

### 27.0 Canonical constructor/initializer identity prerequisite

Complete T8a before T8 semantic super-call validation or compiler default
lowering. T8a publishes canonical factory/initializer products for source,
`@construct`, `@data`, and default-only paths. The T8 semantic checker
resolves each required explicit direct-parent `super.new(...)` call against
those products and records its paired parent initializer identity. Compiler
lowering consumes that product and resolved link without deriving either
again.

### 27.1 Legal immutable write

Allow an immutable instance-field write only when:

```text
current callable/lifecycle = canonical instance initialization lifecycle
AND
field initialization == Uninitialized
```

If:

```text
DefinitelyInitialized -> reject
MaybeInitialized      -> reject
```

`MaybeInitialized` cannot be written because some predecessor may already hold
the immutable value.

### 27.2 Default initializer

The canonical owner-local construction path is identified by T8a's semantic
constructor/initializer product. Semantic lifecycle checks and compiler
lowering consume that product; they do not independently infer whether an
initializer exists or which parent initializer a call selects.

The accepted C4 rule applies to every declared instance-field default, not only
to fields on `@construct`/`@data` classes. The semantic seed

```text
default -> DefinitelyInitialized
```

means constructor body may not assign it again. Runtime must make this fact
true on every successful normal construction path:

1. Evaluate each own instance-field default exactly once per object, in
   declaration order, in that declaring class owner's canonical initializer.
2. For a root initializer, run owner defaults at initializer entry before all
   user-authored statements. For a subclass, the required first explicit
   `super.new(...)` call is the constructor-chain prologue: complete the parent
   initializer (parent defaults, then parent body), then run child defaults
   before the remainder of the child body. This is base-to-derived
   initialization; the explicit chain call itself is not moved or duplicated.
3. Never evaluate instance defaults in `VM::allocate_class_instance`, the
   internal `Class::allocate()` implementation (`class_new_`/`new_`), or the
   low-level `NewInstance` opcode. Those paths allocate storage only.
4. The source `@constructor new(...)` marker lowers to a class-side factory
   that calls `allocate()` and then invokes the hidden instance-side
   `init new(...)`. Execute defaults in that initializer, not in the factory
   or allocator.
5. A class with defaults and no user constructor gets a synthesized
   zero-argument canonical `new`/`init new` path only when no required parent
   initialization edge is omitted, so ordinary `Class.new()` executes its
   defaults. A subclass that needs parent initialization must provide a source
   constructor with the explicit super call. Do not synthesize a successful
   path for required no-default immutable fields it cannot initialize.
6. `@construct` and `@data` already emit generated default assignments in
   declaration order. Preserve those statements as their single default
   prologue; do not prepend or lower those defaults a second time.
7. A subclass initializer whose parent has an initialization lifecycle must
   begin with exactly one explicit `super.new(...)` chain call. The call runs
   the parent's canonical initializer; parent defaults and parent body finish
   before derived defaults and the remainder of the child body. Omission is a
   diagnostic and invalid construction path; P1 does not auto-chain. Parent
   defaults and inherited immutable fields are initialized by the parent
   initializer only.
8. A derived `@construct`/`@data` constructor that cannot express the required
   explicit parent call fails closed with a targeted diagnostic. It must not
   create a successful child-construction path. A user-written `@constructor`
   with the explicit super call is the supported P1 path.

Raw allocation may temporarily produce an instance whose immutable slots hold
the private sentinel. It is not successfully constructed; checked immutable
reads trap until its canonical initializer establishes the field. The raw
sentinel must never surface as `None`, a Phalcom value, or an untyped value.

### 27.3 Constructor exit obligation

For every successful normal construction exit of a concrete constructible
class, every required immutable instance field without a prior valid default
must be:

```text
DefinitelyInitialized
```

`Uninitialized` / `MaybeInitialized` is a diagnostic.

Throwing/non-returning exits are not successful constructed-object exits.

### 27.4 Constructor identity

Use the canonical constructor/initializer product published by T8a. It must
cover source, `@construct`, `@data`, and default-only paths, and pair each
factory callable with its declaring-owner instance initializer. Resolve an
explicit `super.new(...)` against exactly one direct-parent factory product,
then record and lower the paired parent initializer identity.

Do not key legality on selector spelling `new`.

### 27.5 Early-construction self use

The plan's minimal fixed backstop is:

> Uninitialized immutable-field storage must trap rather than surface `None` or
> a value that violates the field contract.

If the live compiler already statically prohibits all relevant pre-initialized
`self` use, prove it with a focused regression and avoid redundant runtime
checks.

Otherwise implement checked field reads for slots that can still be
uninitialized.

Preferred low-risk path:

- keep private uninitialized field slot sentinel;
- checked immutable-field read when post-construction proof is unavailable;
- trap with a field initialization error.

Do not attempt full escape/effect analysis in P1.

### 27.6 Ordinary post-construction reads

If the runtime cannot prove construction phase cheaply, a field read check may
remain as the soundness backstop.

Measure it in T10.

Do not silently accept type unsoundness to avoid a field check.

### 27.7 Class-side immutable fields

VERIFY-FIRST whether a canonical class initialization lifecycle exists.

If none:

```text
immutable class-side field requires declaration initializer
```

Mutable class-side field may remain declaration-uninitialized subject to
existing runtime rules if the language permits.

Do not use instance constructors to initialize class-side storage.

## Forbidden approaches

- no `is_constructor` syntactic/name check as sole authority;
- no second immutable write allowed merely because still inside constructor;
- no first-write-anywhere freeze semantics;
- no treating Maybe as writable;
- no final field invariant that can be violated by constructor-time method call.

## Test changes required

Semantic tests:

```text
immutable_constructor_first_write_accepted
immutable_constructor_second_write_rejected
immutable_default_then_constructor_write_rejected
immutable_nonconstructor_write_rejected
immutable_partial_branch_exit_rejected
immutable_all_normal_exits_initialized
throwing_exit_does_not_require_value_after_throw
mutable_field_reassignment_allowed
ordinary_field_default_initializes_without_user_constructor
explicit_constructor_default_runs_before_user_body
default_only_constructor_is_canonical_and_runs_once
required_explicit_super_initializer_is_first_and_runs_once
missing_required_super_initializer_rejects_construction
derived_construct_without_explicit_super_rejects_construction
inherited_defaults_run_once_in_declaring_owner_order
super_resolves_source_construct_data_and_default_only_parent_initializers
incremental_parent_constructor_product_invalidates_child_super_link
```

Core/runtime regressions must additionally cover:

```text
construct_and_data_generated_default_assignments_are_not_duplicated
raw_new_allocator_does_not_evaluate_instance_defaults
checked_immutable_read_traps_on_raw_or_incomplete_instance
parent_default_and_child_default_run_base_to_derived
```

T8a must verify that the semantic product exists and is canonical for
source-declared, `@construct`, `@data`, and default-only construction paths;
that incremental parent shape changes invalidate dependent child resolution.
T8 semantic tests must prove explicit `super.new(...)` resolves to the direct
parent's paired initializer for source-declared, `@construct`, `@data`, and
default-only parent paths. T8 compiler-lowering verification must prove it
consumes these products without re-deriving constructor identity or
field/parameter mappings.

Use a plain class with `_value: Int = 7`, no derive attribute, and no
user-authored constructor as the smallest runtime discriminator: normal
`Cell.new().read()` must return `7`. A matching semantic fixture must prove
that the defaulted immutable field is already initialized and cannot be
assigned again in its constructor body.

Critical regression:

```telos
class C {
    _x: Int

    @constructor
    new() {
        observe()
        _x = 1
    }

    observe() => _x
}
```

Expected:

```text
must not surface None/raw sentinel as Int
```

Either static rejection or runtime initialization trap is acceptable only if it
matches the implemented fixed strategy.

Class-side test:

```text
immutable class-side without initializer rejected when no class-init lifecycle
```

## Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-core immutable_field -- --nocapture
cargo test -p phalcom-core constructor -- --nocapture
```

Run the exact early-self regression separately.

## Acceptance

- immutable instance field is initialized exactly once;
- every instance declaration default is executed exactly once on all successful
  ordinary construction paths, including a default-only class with no source
  constructor when no parent-init obligation is omitted;
- `@construct`/`@data` defaults are reused, not duplicated;
- raw allocation executes no defaults and checked reads trap on incomplete
  immutable slots;
- subclass initialization requires one explicit parent initializer call;
- parent defaults run in the parent owner path before child defaults, and
  inherited defaults are not replayed by subclasses;
- a subclass path missing its required explicit parent call cannot be accepted
  or return a successfully constructed instance;
- every successful constructor exit proves it;
- ordinary method cannot mutate it;
- early construction cannot violate type contract;
- class-side rule is explicit.

## Local STOP / CONSULT triggers

- making early construction sound requires object-construction state shared
  across arbitrary runtime subsystems;
- constructor lowering has multiple uncoordinated construction lifecycles;
- `super.new(...)` cannot be resolved to exactly one canonical parent
  initializer from the published semantic product or omission cannot be
  diagnosed reliably;
- compiler lowering cannot consume the published constructor/initializer
  product without an independent identity/signature/field-mapping derivation;
- a raw construction path can return an instance as successfully constructed
  without running the required owner initializers.

## Checkpoint update

Record the normative-spec sync, canonical constructor-product publication,
default execution path, explicit-super rule, immutable field lifecycle, and
early-use resolution. Keep P1 partial until T8-spec, T8a, T8 implementation,
focused regressions, and the remaining plan gates complete.

---

# 28. T9 — Tooling, diagnostics, docs, and compatibility residue removal

## Purpose

Make the repository consistently teach and expose the new semantics.

## Preconditions

Core semantics green.

## Consumes

- canonical binding kinds;
- initialization state;
- field mutability;
- module/source index facts.

## Produces

- canonical diagnostics;
- LSP hover/tokens;
- REPL highlighting/completion;
- TextMate fallback;
- current docs/spec alignment;
- no old compatibility branches.

## Files and symbols

Likely:

```text
phalcom-semantic/src/diagnostic.rs
phalcom-core/src/compiler/lib/error.rs

phalcom-lsp/src/hover.rs
phalcom-lsp/src/semantic_tokens.rs

phalcom-repl/src/highlighter.rs
phalcom-repl/src/completer.rs

tools/vsphalcom/syntaxes/phalcom.tmLanguage.json

current binding/class/value specs and C4 implementation records
```

## Required implementation shape

### 28.1 Diagnostics

Canonical messages should teach:

```text
immutable `let` -> use `let mut` for reassignment
immutable `let` requires initializer
static `const` requires initializer
uninitialized binding read
immutable field already initialized
immutable field write outside initialization lifecycle
required immutable field not initialized on every successful constructor exit
undeclared field
```

Remove user-facing advice saying:

```text
use `let` for mutation
let x reads None
const is runtime immutable
```

### 28.2 LSP hover

Update keyword/contextual descriptions.

When semantic fact exists, optionally present:

```text
mutable; possibly uninitialized here
```

Do not display `T | None` unless that is the actual type.

### 28.3 Semantic tokens

Contextually classify `mut` in:

```text
let mut
mut field
mut trait property
mut delegation
```

Do not classify arbitrary identifier `mut` globally.

### 28.4 REPL

Ensure:

```telos
let mut x
```

is treated as a complete declaration.

Update highlighting/help/completion text.

### 28.5 TextMate

Update fallback grammar for canonical forms.

Semantic tokens remain authoritative for contextual classification.

### 28.6 Remove compatibility residue

Search for old semantics:

```text
"let is mutable"
"uninitialized let reads None"
"const binding immutable runtime"
BindingKind::Let used as mutability
const field terminology
collect assigned fields as layout authority
```

Every live current-source occurrence must be classified.

Do not rewrite historical archived implementation documents unless current-doc
policy requires explicit supersession markers.

## Forbidden approaches

- no tooling-only mutability inference that disagrees with source index;
- no global regex keyword coloring for `mut` that breaks contextual use;
- no stale docs left in current canonical spec tree.

## Test changes required

LSP tests:

```text
hover_let_describes_immutable_runtime_binding
hover_let_mut_describes_mutable_runtime_binding
hover_const_describes_static_binding
semantic_tokens_contextual_mut
semantic_tokens_identifier_named_mut_not_keyword
```

REPL:

```text
let_mut_uninitialized_is_complete_input
let_mut_highlighting
```

## Tests to run now

Focused LSP and REPL tests only.

Then grep/audit old wording.

## Acceptance

- tooling and current docs match C4 semantics;
- no active compatibility path retains old behavior;
- P1-INV-29 green.

## Local STOP / CONSULT triggers

- LSP needs a second source of semantic mutability/init truth because source
  index cannot represent the facts.

## Checkpoint update

Record tooling/docs alignment.

---

# 29. G1 — Semantic/runtime coherent behavior gate

## Purpose

Prove the full vertical works before performance and broad stabilization.

## Required cases

### Binding

```telos
let x = 1
x = 2
```

-> immutable diagnostic.

```telos
let mut x = 1
x = 2
```

-> succeeds.

```telos
let mut x: Int
x
```

-> guaranteed-uninitialized semantic diagnostic.

```telos
let mut x: Int
if condition { x = 1 }
use(x)
```

-> accepted; checked read.

```telos
let mut x: Int | None = None
use(x)
```

-> accepted; no uninitialized trap.

Closure:

```telos
let mut x: Int
let f = { x }
f()
```

-> runtime trap.

After assignment:

```telos
x = 1
f()
```

-> succeeds.

### Fields

```telos
class C {
    _id: Int
    mut _count: Int
}
```

-> immutable/mutable signatures correct.

Undeclared assignment -> diagnostic, no slot.

Constructor first write -> accepted.

Second immutable write -> rejected.

Partial branch constructor -> rejected.

Early self read -> static rejection or runtime trap; never value-contract
violation.

### Const

Minimal static `const` declaration must not emit an ordinary runtime storage
binding.

## Run

Use exact focused semantic and core filters created by T4–T8.

Inspect bytecode for:

- proven local read;
- maybe local read;
- static `const`.

## Expected

All P1-CV-01 through P1-CV-29 directly relevant to the core vertical pass or
have explicit evidence.

Do not broaden after PASS until T10.

On failure, classify before editing.

---

# 30. T10 — Incremental and performance stabilization

## Purpose

Prove P1 does not corrupt incremental reuse or ordinary hot-path performance.

## Preconditions

G1 green.

## Consumes

- final semantic/runtime architecture.

## Produces

- incremental invalidation evidence;
- bytecode hot-path evidence;
- performance comparison;
- no accidental allocation.

## 30.1 Incremental requirements

Test:

1. edit `let mut` control flow so read changes:
   ```text
   definite -> maybe
   ```
   while type remains same;
2. semantic callable product must invalidate/recompute;
3. lowering must change:
   ```text
   unchecked -> checked
   ```
4. unrelated callable products should remain reusable under existing
   incremental policy.

Field:

```text
bare field <-> mut field
```

must invalidate field signature product and dependents.

## 30.2 Performance requirements

### Local read

Hard gate:

```text
proven GetLocal has no uninitialized branch
```

Use bytecode/unit code-path inspection and, where practical, a microbenchmark.

### Checked read

Measure only to ensure it is bounded and allocation-free.

### Uninitialized representation

Must allocate zero heap objects per declaration.

### Field layout

Removing assignment inference should not add new body scans.

## 30.3 Benchmark surface

Use existing benchmark harness if available.

At minimum compare:

```text
tight local read loop
tight mutable initialized local read loop
maybe-initialized checked read path
ordinary field read
```

Do not invent a broad benchmark suite merely for this plan.

## Test changes required

Add microbench only if existing harness has a natural place.

Otherwise record bytecode/assembly-independent structural evidence:

```text
safe opcode has no check
checked opcode has check
```

plus existing VM benchmark before/after if cheap.

## Tests to run now

Focused incremental tests.

Existing VM benchmark subset if established.

No release-wide performance ceremony.

## Acceptance

- P1-INV-11 proven;
- P1-INV-24/25 proven;
- no per-binding allocation;
- no unexpected material regression in ordinary local access.

## Local STOP / CONSULT triggers

- preserving branch-free local reads requires deleting semantic runtime
  fallback;
- performance regression is architectural rather than mechanical.

## Checkpoint update

Record performance/incremental evidence.

---

# 31. G2 — Affected-crate stabilization gate

## Purpose

Prove P1 integration across the directly affected crates.

## Run

Use repository-specific flags from current guidance, approximately:

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

If a crate has established narrower feature flags/environment requirements,
use them.

Run formatter/check policy required by repository only at this stabilization
boundary.

## Failure policy

Classify every failure A/B/C/D.

Do not fix D.

Do not weaken semantics to satisfy stale old-syntax tests. Migrate/update those
tests if they are P1-owned.

## Expected

- all P1-owned affected-crate failures green;
- baseline failures recorded truthfully;
- active source corpus parses under target syntax.

---

# 32. T11 — Completion records and durable handoff

## Purpose

Close P1 truthfully.

## Preconditions

G1 and G2 complete to the level required for P1.

## Produces

```text
LANG007.C4-CHECKPOINT.md updated
LANG007.C4.P1-walkthrough.md
LANG007.C4.P1-handoff.md
```

## Checkpoint completion fields

Record:

- plan status;
- completion state;
- verification classification;
- final P1 invariants;
- any consultations/amendments;
- corpus migration counts;
- layout parity result;
- focused/broad tests actually run;
- deferred baseline failures;
- performance evidence;
- next action: C4.P2.

Do not mark C4 complete.

## Walkthrough required content

```text
1. Plan identity/final state
2. Objective/outcome
3. Architecture implemented
4. Important code changes
5. Binding initialization design
6. Runtime sentinel/check design
7. const phase boundary
8. field explicit-layout/lifecycle design
9. corpus migration details
10. deviations
11. consultations
12. tests added
13. tests executed
14. tests deferred
15. performance evidence
16. residual/baseline failures
17. verification classification
18. follow-up for P2
```

## Handoff required content

```text
current revision/state
stable P1 interfaces
P1 invariants
C4.P2 objective
must-read files
known baseline failures
first recommended commands
do-not-redesign guidance
```

Explicitly tell P2:

```text
do not turn const back into runtime storage
do not replace binding uninitialized state with None
do not bypass semantic phase ownership
do not restore implicit field layout
```

---

# 33. Detailed compiler/lowering checklist

Use this as an implementation audit, not as separate microtasks.

## BindingKind consumers

Search:

```sh
rg -n 'BindingKind::(Let|Const)|kind == BindingKind|matches!\([^)]*BindingKind' \
  phalcom-ast phalcom-modules phalcom-semantic phalcom-core phalcom-lsp phalcom-repl
```

Classify each occurrence:

```text
syntax
phase
mutability
source presentation
runtime lowering
test expectation
historical comment
```

No mutability consumer should remain as:

```text
BindingKind::Let => mutable
```

## Old wording

Search:

```sh
rg -n 'let.*mutable|mutable.*let|const.*immutable|uninitialized.*None|reads.*None' \
  phalcom-* docs tools examples
```

Classify current versus historical docs.

## Field inference

Search:

```sh
rg -n 'collect_assigned_fields|assigned_fields|implicit field|const_fields' \
  phalcom-core phalcom-semantic
```

No assignment-derived layout remains after T7.

## Read fusions

Search:

```sh
rg -n 'GetLocal|InvokeLocal|GetUpvalue|GetGlobal' phalcom-core/src
```

Every transformation must preserve checked/unchecked semantics.

---

# 34. Diagnostic contract

Final user-facing semantics should be coherent even if exact text differs.

## Binding

### Immutable reassignment

```text
cannot assign to immutable binding `x`; declare it with `let mut` if reassignment is required
```

### Missing immutable initializer

```text
immutable `let` binding `x` requires an initializer
```

### Missing static initializer

```text
static `const` binding `x` requires an initializer
```

### Guaranteed uninitialized read

```text
binding `x` is read before it is initialized
```

### Runtime maybe-read failure

```text
binding `x` was read before initialization
```

or equivalent runtime error.

## Fields

### Undeclared

```text
field `_x` is not declared on `C`
```

### Immutable outside init lifecycle

```text
field `_x` is immutable and may only be initialized during construction
```

### Already initialized

```text
immutable field `_x` is already initialized
```

### Incomplete constructor

```text
immutable field `_x` is not initialized on every successful constructor path
```

Do not mention retired `const field` syntax in primary guidance except as a
migration note.

---

# 35. Corpus migration patch discipline

The corpus rewrite will likely be large.

Treat it as a coherent migration unit, not hand-edited noise.

Recommended commit grouping if commits are authorized:

```text
1. lang007: add C4 binding syntax and migration support
2. lang007: migrate binding and explicit field source corpus
3. lang007: enforce runtime binding initialization semantics
4. lang007: enforce explicit immutable field lifecycle
5. lang007: align C4 tooling and complete P1
```

Exact grouping may differ, but do not commit a known semantically broken
intermediate state merely because the textual migration is large.

Before each commit:

```sh
git status --short
git diff --stat
git diff -- <scoped paths>
```

Stage only P1 files.

Do not reset/clean unrelated work.

---

# 36. Field migration parity algorithm

This algorithm is fixed enough for Luna to implement without guessing.

For each class under the old layout collector:

```text
old_instance = ordered own instance fields
old_class    = ordered own class-side fields
```

After migration:

```text
new_instance = explicit own instance declarations in effective slot order
new_class    = explicit own class-side declarations in effective slot order
```

Compare:

```text
names
side
relative own-class slot indices
field count
super prefix offset
```

Mutability compare under semantic translation:

```text
old bare/inferred mutable -> new mut
old const immutable       -> new bare
```

Do not compare source range.

Do not compare process-local Symbol IDs.

Fail migration on any unreviewed difference.

---

# 37. Binding flow laws

Implement/tests should encode these exact laws.

Let:

```text
U = Uninitialized
M = MaybeInitialized
D = DefinitelyInitialized
```

Join:

```text
U ⊔ U = U
D ⊔ D = D
U ⊔ D = M
D ⊔ U = M
U ⊔ M = M
M ⊔ U = M
D ⊔ M = M
M ⊔ D = M
M ⊔ M = M
```

Transfer:

```text
valid write(U) -> D
valid write(M) -> D for mutable bindings
valid write(D) -> D for mutable bindings
```

For immutable runtime bindings, ordinary write is rejected independently of
initialization.

Read:

```text
read(U) -> static error
read(M) -> checked runtime read
read(D) -> unchecked runtime read
```

The mutable-binding transfer differs from immutable-field write-once semantics.
Do not accidentally reuse the field “only from U” write rule for `let mut`.

---

# 38. Immutable-field lifecycle laws

For immutable fields:

```text
declaration default:
    entry state = D

no default:
    constructor entry state = U
```

Legal construction write:

```text
U -> D
```

Illegal:

```text
M -> write
D -> write
ordinary method -> write
```

Normal constructor merge:

```text
all D -> D / success
any U/M -> constructor diagnostic
```

Exceptional/non-returning path:

```text
does not create a successful constructed instance
```

Mutable fields do not use the write-once restriction.

---

# 39. Runtime sentinel laws

The runtime representation must satisfy:

```text
UninitializedBinding != None
UninitializedBinding != Unit
UninitializedBinding is not user-constructible
UninitializedBinding cannot become Some(...)
UninitializedBinding cannot be a successful checked read result
UninitializedBinding can live in a local/global/upvalue cell
assignment overwrites it normally
```

If a dedicated `ValueTag` is used, update exhaustive matches so internal
invariants fail loudly rather than silently treating the tag like ordinary
value data.

Do not expose it in reflection/type classification as a Telos type/value.

---

# 40. Static `const` P2 handoff contract

P1's final static binding representation must allow P2 to attach:

```text
definition identity
initializer semantic body
declared result domain/type
StaticValue or residual result
dependency fingerprint
materialization descriptor
```

P1 does not need to implement those fields now.

But P1 must not choose a representation that makes `const` inseparable from a
runtime slot.

The P1 handoff should identify the exact module/semantic structure P2 should
extend.

---

# 41. Incremental invalidation checklist

Verify that these edits invalidate the right semantic products:

```text
let -> let mut
let mut -> let
initializer added/removed
control-flow assignment added/removed
field bare -> mut
field mut -> bare
field declaration added/removed
constructor path initialization changed
```

The key property is semantic dependency sensitivity, not source-range churn.

No fingerprint should contain unstable process-local IDs solely because P1 adds
initialization state.

---

# 42. Optimizer soundness checklist

Audit all optimizations that assume binding immutability/mutability:

```text
product virtualization
constant facts/scopes
local invoke fusion
allocation sinking if keyed to immutable binding
closure capture specialization
dead store/elision if present
```

Rules:

- `let` is the runtime immutable candidate now;
- `let mut` is mutation-sensitive;
- `const` is static, not runtime optimizer input;
- checked-read requirement is a semantic effect that cannot be erased.

If an optimization has no proof path for read safety, it must fall back rather
than speculate.

---

# 43. Tooling source-of-truth checklist

LSP/REPL should consume canonical data where available.

Do not introduce:

```text
hover mutability inference
semantic-token mutability inference
completion mutability inference
```

as separate semantic engines.

Token classification may be syntax-aware, but binding meaning and
initialization come from semantic/source-index products.

---

# 44. Required negative tests

At least the following should exist somewhere in the focused acceptance surface.

```telos
let x
```

error.

```telos
const x
```

error.

```telos
let x = 1
x = 2
```

error.

```telos
let mut (a, b)
```

error.

```telos
let mut x: Int
x
```

error.

```telos
class C {
    _x: Int
    f() { _x = 1 }
}
```

error outside construction lifecycle.

```telos
class C {
    _x: Int
    @constructor new() {
        _x = 1
        _x = 2
    }
}
```

error.

```telos
class C {
    @constructor new() {
        _x = 1
    }
}
```

error undeclared field.

```telos
class C {
    _x: Int
    @constructor new(flag: Bool) {
        if flag { _x = 1 }
    }
}
```

error on successful path with no assignment.

---

# 45. Required positive tests

```telos
let x = 1
use(x)
```

valid, unchecked read.

```telos
let mut x = 1
x = 2
use(x)
```

valid.

```telos
let mut x: Int
x = 1
use(x)
```

valid.

```telos
let mut x: Int | None = None
use(x)
```

valid, `None` is value.

```telos
let mut x: Int
if cond { x = 1 } else { x = 2 }
use(x)
```

definitely initialized.

```telos
class C {
    _id: Int
    mut _count: Int

    @constructor
    new(id: Int) {
        _id = id
        _count = 0
    }

    bump() {
        _count = _count + 1
    }
}
```

valid.

```telos
class C {
    _id: Int = 1
}
```

valid immutable default.

---

# 46. Verification gates mapping

| Gate | Coverage |
|---|---|
| T1 parser green | CV-01..03, 18 syntax portion |
| T2 migration parity | CV-20, 30 |
| T3 semantic taxonomy | CV-04, 15, 16 |
| T4 flow | CV-05..09, 17 |
| T5 runtime | CV-10..14, 32 |
| T6 const barrier | CV-15, 33 |
| T7 explicit fields | CV-18..20, 29 |
| T8 lifecycle | CV-21..28 |
| T9 tooling | CV-31 |
| G1 | core vertical CV-01..29 |
| T10 | CV-17,29,32 |
| G2 | affected-crate integration |

---

# 47. Completion truth table

Do not conflate these states:

```text
source syntax added
corpus migrated
semantic cutover implemented
runtime checks implemented
field lifecycle implemented
focused tests passed
affected crate suites passed
P1 complete
C4 complete
release certified
```

P1 may be marked `COMPLETE` only when its own bounded acceptance is met.

`LANG007.C4` remains active for P2+.

---

# 48. Performance/resource evidence

P1 has explicit performance obligations.

## 48.1 Required evidence

### Proven local read

Show that:

```text
semantic ProvenInitialized
    -> ordinary unchecked local load
```

with no sentinel comparison in the opcode path.

### Checked local read

Show:

```text
RuntimeCheckRequired
    -> one bounded check
    -> no allocation
```

### Sentinel

Show zero heap allocation.

### Field layout

Show explicit layout does not add body-wide field discovery work.

## 48.2 Acceptance

No material ordinary-local regression beyond benchmark noise.

If benchmark variance is high, structural opcode evidence is mandatory and
benchmark evidence may be classified inconclusive rather than forcing
micro-optimization.

---

# 49. Checkpoint bookkeeping

## At plan start

- create/activate `LANG007.C4-CHECKPOINT.md`;
- record starting revision;
- add P1 to plans ledger;
- copy/register P1 invariant ledger.

## During plan

Update only on durable events:

- T1 syntax interface;
- T2 migration/layout parity;
- T4 semantic initialization fact;
- T5 runtime checked-read architecture;
- T6 static const boundary;
- T7 explicit field authority;
- T8 field lifecycle;
- consultations/amendments;
- G1/G2 evidence;
- deferred baseline failure.

Do not turn checkpoint into a per-command diary.

## At completion

Record:

```text
P1 COMPLETE or partial
verification state
final invariants
consultations
migration summary
baseline failures
performance evidence
next action = C4.P2
```

---

# 50. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P1-walkthrough.md
```

Must contain:

- final result;
- architecture implemented;
- files/components changed;
- migration approach and counts;
- binding init flow;
- checked read runtime;
- const phase boundary;
- explicit-field/layout change;
- immutable-field lifecycle;
- early-self resolution;
- deviations;
- consultations;
- tests added;
- tests actually run;
- tests deferred;
- performance evidence;
- baseline failures;
- verification classification;
- P2 follow-up.

Never claim broad green without evidence.

---

# 51. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P1-handoff.md
```

Must contain:

```text
current revision/state
completed prerequisites
stable P1 interface map
P1-INV list or compact reference
binding phase/mutability API
binding initialization/read-safety API
runtime sentinel/checked-load API
module binding category
field lifecycle API
known repository drift
deferred failures
must-read files
next objective: C4.P2
recommended first commands
do-not-redesign guidance
```

The handoff should be compact enough to prevent rediscovery.

---

# 52. Plan self-review

Before considering this plan ready for execution, verify:

- every invariant has an owner;
- every soundness risk has a test obligation;
- `None` is never overloaded;
- static `const` is not accidentally runtime;
- field layout has one source of truth;
- field lifecycle has one source of truth;
- compiler does not gain a duplicate flow engine;
- checked-read optimizer interaction is explicit;
- constructor early-use risk is explicitly covered;
- migration precedes inference deletion;
- migration is AST-aware and idempotent;
- BUILD tests are narrow;
- broad tests occur only at named gates;
- STOP/CONSULT triggers are objective;
- C4 checkpoint bookkeeping exists;
- walkthrough and handoff are required;
- no task depends on a later task's interface;
- no P2 evaluator work is hidden in P1.

---

# 53. Final Luna execution directive

Implement in task order.

The most important sequencing constraints are:

```text
1. add target syntax
2. migrate source while old layout oracle exists
3. flip semantic meaning
4. add initialization proof
5. add runtime fallback
6. cut const away from runtime
7. delete implicit fields
8. enforce immutable field lifecycle
9. align tooling/docs
10. stabilize/perf
```

Do not “simplify” the plan by:

- mapping uninitialized to `None`;
- rejecting all Maybe reads;
- checking every local at runtime;
- treating `const` as old runtime immutable until P2;
- keeping implicit field inference as a fallback;
- making immutable fields first-write-anywhere;
- reserving `mut` globally;
- re-running flow analysis in the compiler.

The target architecture is:

```text
ordinary Telos source semantics
        ↓
canonical semantic declaration + flow facts
        ↓
stable lowering facts
        ↓
minimal compiler/runtime representation
```

P1 is complete when that architecture is true end-to-end and the repository
corpus has been cut over to it.
