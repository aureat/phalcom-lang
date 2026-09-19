---
id: STDL002.C2.P1.A1
category: STDL
program: STDL002
checkpoint: STDL002.C2
kind: implementation-plan-amendment
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
amends:
  - STDL002.C2.P1
depends_on:
  - STDL002.C2.P1-plan-00-shared-header.md
  - STDL002.C2.P1-01-body.md
follows: null
supersedes: null
---

# STDL002.C2.P1.A1 — Standalone Universe Dependency Closure, Canonical Period Identity, and T6 Resume

> **Role:** Luna-oriented corrective execution amendment for `STDL002.C2.P1`.
>
> **Important lifecycle rule:** this document is **not a third C2 implementation plan** and does not replace either official C2 plan. `STDL002.C2` still has exactly two plans: `P1` and `P2`. This amendment exists only to remove the live P1 infrastructure/identity blockers, integrate the already-started T6 civil source, and hand execution back to the existing P1 body at the correct resume point.
>
> **Implementation authority after this amendment:** the P1 shared header, installed P1 body, C2 checkpoint, C2 guidance, and any approved consultation amendments remain authoritative. This document narrows and orders the corrective work needed to resume them.

---

## 0. Executor contract

This plan is written for GPT-5.6 Luna executing continuously under constrained architectural authority.

The implementer:

- may adapt **mechanical** details to execution HEAD;
- must preserve the architecture, ownership boundaries, and invariants below;
- must not improvise a new module system, linker policy, Universe identity model, or temporal API;
- must follow the verification budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must keep `STDL002.C2-CHECKPOINT.md` current at durable events;
- must STOP AND CONSULT when any listed escalation trigger fires;
- must not silently turn this amendment into a broader MODL/LANG/SEMA redesign;
- must not create another P1 body, requirements analysis, or competing checkpoint-state document.

Testing is evidence gathering, not ceremony. Run the smallest command that proves the current semantic boundary.

---

# 1. Goal

Remove the standalone compiler's incomplete Universe dependency-closure behavior, independently close the direct `Period` semantic-identity seam if it still reproduces, publish and verify the existing T6 civil modules through normal canonical module semantics, and return `STDL002.C2.P1` to its planned T6/T7 execution sequence without a time-specific workaround.

---

# 2. Checkpoint acceptance objective

This amendment is complete when all of the following are true:

1. a standalone source file may import a canonical Universe module or package whose implementation has transitive imports;
2. standalone source **still cannot gain arbitrary sibling/project import authority**;
3. `ProgramAnalyzer` no longer constructs a one-hop-only Universe graph for standalone compilation;
4. strict `ModuleLinker` behavior remains strict; missing resolution edges are not silently tolerated;
5. no `time`/`Period` special case exists in compiler/module infrastructure;
6. direct canonical import and use of `Period` either:
   - passes with one canonical declaration/type identity, or
   - has received one bounded, generic semantic repair with focused regression evidence;
7. the T6 civil source modules that already exist are registered/published through the canonical Universe graph and can be analyzed/executed through the normal language-corpus path;
8. the current root `time` package import path no longer fails because the package's child imports were omitted from the standalone graph;
9. `STDL002.C2-CHECKPOINT.md` records the corrected root cause and truthful new evidence;
10. the next action is again the existing P1 body:
    - finish any remaining T6 contract, including the separately gated `UtcOffset` range decision;
    - then continue P1's existing later gates/T7 publication work.

This amendment does **not** declare all of P1 complete. It removes the blocker and restores the intended execution path.

---

# 3. Repository grounding

Prepared against remote repository state:

```text
repository: aureat/phalcom-lang
remote branch: main
remote revision: 777d9271906cd9d211a9eb25ac8d54a85be15ad9
revision message: test(stdlib): cover calendar number rejection
parent: e2030124bca94d7e7884a5b3b2bd9812fe8a8d3b
inspection date: 2026-09-19
local working tree: NOT OBSERVABLE during planning
```

The remote `main` revision was inspected directly. The implementer must not assume the local checkout is identical.

### Verified repository facts

1. `phalcom-core/src/modules/compile.rs::ProgramAnalyzer::analyze_standalone_module` still loads only the standalone entry plus each **directly imported** Universe target, then calls strict linking.
2. The standalone path still inserts only:
   ```text
   (entry_id, authored_import_path) -> direct_target
   ```
   and does not traverse imports authored by that target.
3. `ProgramAnalyzer::discover_and_analyze` already owns a worklist-based transitive discovery loop for project/package compilation.
4. `phalcom-modules::ModuleResolver` is the canonical resolver and already emits canonical `ModuleId` targets.
5. `phalcom-modules::ModuleLinker` is a strict consumer of a resolved-edge graph. The current failure does not justify weakening it.
6. `phalcom-core/src/modules/canonical_universe.rs::build_canonical_universe_program` independently demonstrates that the full Universe interface set can be resolved and strictly linked with canonical module identities.
7. `ImportResolutionProduct` and `ImportResolutionTrace` already retain canonical target/prefix/package-resolution facts; do not invent a second import identity model.
8. `phalcom-core/tests/support/mod.rs` compiles language-corpus fixtures through `ProgramCompiler::compile_entry_selection(EntrySelection::Module(...))`, so the standalone path is directly relevant to `language-corpus::time`.
9. `Period`'s source module has no imports; therefore the direct `Period` return-type mismatch cannot be explained solely by the missing transitive-module closure.
10. Existing passing time coverage directly imports non-prelude `Measurement`, so the defect must not be generalized to "all non-prelude data imports".
11. T6 source files already exist for `Date`, `DateOverflow`, `Time`, `DateTime`, `YearMonth`, and `MonthDay`, but the P1 checkpoint still records them as not fully published/runtime-verified.
12. At planning revision `777d9271`, `date.ph` uses `Weekday` but does not import it.
13. At planning revision `777d9271`, the parser's live `parse_import_path` still consumes dotted child separators, while the separate active LANG004 logical-path plan proposes migration to `::`.
14. Therefore import **spelling** is execution-time VERIFY-FIRST. This amendment must operate on the live `ImportPath` AST and canonical `ModuleId`, not hard-coded dotted path strings.
15. `UtcOffset` remains intentionally blocked on the P1 VERIFY-FIRST bound decision. This amendment must not guess that bound.

### Execution-head drift check

Before Task T0:

```bash
git status --short
git rev-parse HEAD
git log -n 5 --oneline
```

Then inspect:

```bash
git diff 777d9271906cd9d211a9eb25ac8d54a85be15ad9..HEAD -- \
  phalcom-core/src/modules/compile.rs \
  phalcom-core/src/modules/canonical_universe.rs \
  phalcom-modules/src/resolver.rs \
  phalcom-modules/src/linker.rs \
  phalcom-ast/src/parser.rs \
  phalcom-core/core/universe/src/time \
  phalcom-core/tests/core/modules \
  phalcom-core/tests/fixtures/language/time \
  phalcom-semantic/src/session.rs \
  docs/implementation/STDL002-standard-library-buildout/C2-time-complete
```

If the primary symbols still own the same responsibilities, adapt mechanically. If responsibility has moved or the one-hop standalone path has already been replaced, re-ground and follow the STOP/CONSULT rule rather than recreating obsolete work.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Read only these before starting; do not broadly re-audit the repository.

```text
AGENTS.md

docs/implementation/STDL002-standard-library-buildout/C2-time-complete/
    STDL002.C2-CHECKPOINT.md
    STDL002.C2-GUIDANCE.md
    STDL002.C2.P1-plan-00-shared-header.md
    STDL002.C2.P1-01-body.md

phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/canonical_universe.rs

phalcom-modules/src/resolver.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/builtin.rs
phalcom-modules/src/interface.rs

phalcom-core/tests/core/modules/universe.rs
phalcom-core/tests/core/modules/linking.rs
phalcom-core/tests/support/mod.rs

phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/time/calendar/package.ph
phalcom-core/core/universe/src/time/calendar/period.ph
phalcom-core/core/universe/src/time/calendar/date.ph
phalcom-core/core/universe/src/time/calendar/time.ph
phalcom-core/core/universe/src/time/calendar/datetime.ph
phalcom-core/core/universe/src/time/calendar/partial.ph

phalcom-semantic/tests/semantic/capabilities/time.rs
```

For import spelling only, inspect:

```text
phalcom-ast/src/parser.rs — parse_import_path
```

If execution HEAD has landed LANG004 C2, also inspect its completed walkthrough/handoff before changing fixtures or package source.

Do **not** read or rehabilitate stale `time.md` specifications; C2 explicitly excludes them.

---

# 5. Normative authority

The authorities for this amendment are ordered by semantic ownership.

## 5.1 Current P1/C2 authority

```text
STDL002.C2-CHECKPOINT.md
    durable checkpoint state and accepted amendments

STDL002.C2-GUIDANCE.md
    durable time implementation rules

STDL002.C2.P1-plan-00-shared-header.md
    P1 architecture/invariants/consultation rules

STDL002.C2.P1-01-body.md
    ordered temporal implementation tasks and acceptance
```

This amendment does not redefine those artifacts.

## 5.2 Module architecture authority

For implementation purposes:

```text
phalcom-modules
    owns module/project/import/export identity and resolution

ModuleResolver
    owns canonical import resolution

ModuleId
    owns canonical module identity

ModuleLinker
    owns binding/linking over resolved module identity

ProgramAnalyzer
    owns compiler-entry discovery/orchestration policy
```

The standalone compiler may restrict which roots the standalone **entry** is allowed to introduce. It may not redefine how an accepted canonical Universe module's dependencies resolve.

## 5.3 Semantic authority

```text
DeclarationId
    canonical declaration ownership identity

TypeStore / declaration products
    canonical semantic type representation

source-authored Universe declaration
    source declaration authority

native/bootstrap metadata
    implementation/bootstrap association, not a second language declaration
```

## 5.4 Import syntax authority

The live parser owns currently accepted source spelling.

At planning revision:

```phalcom
from universe.time.calendar.period import Period
```

is still accepted by `parse_import_path`.

A separate LANG004 plan intends:

```phalcom
from universe::time::calendar::period import Period
from .pkg1::pkg2::mod1 import ExportedMember
from ..pk2::mod2 import (Member1, Member2)
```

Do not implement that syntax migration here. If it has landed by execution time, use it. If it has not landed, use current dotted spelling. All architecture in this amendment is syntax-independent.

---

# 6. Takeover state

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Standalone entry compilation | `ProgramAnalyzer::analyze_standalone_module` | `phalcom-core` | Standalone entry has restricted import authority |
| Project/package discovery | `ProgramAnalyzer::discover_and_analyze` | `phalcom-core` | Transitive module closure before linking |
| Import resolution | `ModuleResolver` | `phalcom-modules` | Produces canonical `ModuleId` |
| Retained import product | `ImportResolutionProduct` | `phalcom-modules` | Authored site + canonical target + topology evidence |
| Resolution trace | `ImportResolutionTrace` | `phalcom-modules` | Canonical target + package/prefix facts |
| Linking | `ModuleLinker` | `phalcom-modules` | Strictly binds a complete resolved graph |
| Universe source | `UniverseSourceProvider` | `phalcom-modules` | Toolchain-owned source/interface authority |
| Canonical Universe compiler product | `build_canonical_universe_program` | `phalcom-core` | Whole-Universe strict resolver/linker comparator |
| Corpus execution | `run_corpus_case` | `phalcom-core/tests/support` | Production compiler path via `EntrySelection::Module` |
| Period declaration | `time/calendar/period.ph` | Universe source | Canonical non-prelude `data` |
| Civil scaffold | `time/calendar/{date,time,datetime,partial}.ph` | Universe source | T6 source semantics |
| Universe node inventory | `UNIVERSE_NODES` or execution-HEAD equivalent | `phalcom-modules` | Published canonical Universe module set |
| Time package publication | `time/package.ph`, `time/calendar/package.ph` | Universe source | No duplicate wrapper types/aliases |
| Temporal prelude | canonical prelude policy | Universe/semantic layer | Only `Duration` and `Timestamp` |
| UtcOffset bound | P1 T6 VERIFY-FIRST decision | P1 architecture | Must not be guessed |

---

# 7. Architecture

## 7.1 Current failing flow

```text
standalone .ph fixture
    ↓
EntrySelection::Module
    ↓
classify_entry_ownership
    ↓
ProgramAnalyzer::analyze_standalone_module
    ↓
parse standalone entry
    ↓
for each DIRECT entry import:
    validate root == Universe
    synthesize/load direct target
    insert only entry -> target resolution
    ↓
strict ModuleLinker
    ↓
target package/module has its own import
but no resolved(target, child-path) edge exists
    ↓
LinkError::UnresolvedImport
```

The failure:

```text
unresolved import 'universe.time.clock' from module universe:time
```

is therefore a producer-side graph-closure defect.

## 7.2 Target flow

```text
standalone .ph fixture
    ↓
EntrySelection::Module
    ↓
standalone entry admission policy
    |
    | only entry-authored roots allowed by standalone policy
    v
canonical ModuleResolver
    ↓
accepted direct Universe target
    ↓
shared transitive dependency discovery
    ├─ load target parsed source/interface
    ├─ inspect every target import/re-export dependency
    ├─ resolve through ModuleResolver
    ├─ retain canonical target ModuleId
    ├─ enqueue newly discovered modules/package interfaces as required
    └─ repeat to closure
    ↓
complete interfaces + resolved edges
    ↓
strict ModuleLinker
    ↓
semantic analysis
    ↓
compiler/lowering/runtime
```

The distinction is mandatory:

```text
standalone entry admission
    controls which external roots user source may introduce

canonical module resolution
    controls dependency closure after a target is accepted
```

A Universe module does not become "standalone user source" merely because a standalone consumer imported it.

## 7.3 Reuse rule

`discover_and_analyze` already contains a correct transitive worklist.

The implementation should extract/reuse its discovery mechanics rather than copy a second recursive loop into the standalone function.

Preferred ownership:

```text
phalcom-core/src/modules/compile.rs
    private program-discovery orchestration helper
```

unless execution HEAD already has an equivalent reusable abstraction in `phalcom-modules`.

Do not move canonical resolution into `phalcom-core`; the helper must delegate each edge resolution to `ModuleResolver`.

## 7.4 Canonical Universe comparator

`build_canonical_universe_program` loads all Universe interfaces, resolves every import with `ModuleResolver`, then uses strict `ModuleLinker::link_all`.

It is evidence that the canonical Universe source graph itself can be strictly resolved.

Do **not** make every standalone invocation compile/materialize that process-wide whole-Universe product merely to solve this blocker. Standalone discovery should remain reachable-closure based.

## 7.5 Period semantic boundary

After graph closure is fixed:

```text
direct leaf import:
    Period

module has no imports
```

must be tested separately.

If it fails, the failure is semantic declaration/type identity, not dependency discovery.

The required identity relationship is:

```text
import-selected Period declaration
==
impl Period target declaration
==
return annotation Period declaration
==
raw data constructor Period(...) declaration
==
associated factory result declaration
```

and, within one semantic `TypeStore`:

```text
declared result nominal TypeId
==
constructor result nominal TypeId
==
published callable result nominal TypeId
```

No name/path special case is allowed.

## 7.6 T6 resume boundary

Only after module closure and Period identity are green:

```text
fix civil source import completeness
    ↓
register T6 Universe nodes
    ↓
publish through calendar/time packages
    ↓
run T6 civil semantic/runtime coverage
    ↓
hand control back to existing P1
```

---

# 8. Ownership boundaries

## 8.1 Owns

This amendment owns:

- standalone compiler dependency-closure orchestration;
- focused regression coverage for transitive Universe imports;
- verification that strict linker behavior remains unchanged;
- one bounded diagnosis/repair of the direct `Period` semantic identity seam;
- T6 module graph registration/publication for already-authored civil source;
- missing T6 source imports discovered by that integration;
- blocker-related C2 checkpoint bookkeeping;
- exact resume handoff back to P1.

## 8.2 Does not own

This amendment does not own:

- general import-syntax migration from `.` to `::`;
- parser grammar redesign;
- project/module identity redesign;
- persistent workspace/LSP module-session redesign;
- linker tolerant-recovery policy;
- temporal VM specialization;
- new native calendar behavior;
- new time API semantics;
- UtcOffset range selection;
- named timezone/P2 work;
- broad semantic-baseline refactoring unless direct Period evidence proves it necessary;
- stale time specification cleanup.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Accepted import syntax | `phalcom-ast::parse_import_path` at execution HEAD | fixtures, source files | amendment-specific syntax parser |
| Import target | `ModuleResolver` / `ModuleId` | linker, compiler, semantic layer | manual string-to-module reconstruction |
| Module identity | `ModuleId` | all module consumers | path display string |
| Import topology evidence | `ImportResolutionProduct` / `ImportResolutionTrace` | incremental/module consumers | new custom target/prefix record |
| Link validity | `ModuleLinker` | compiler | linker-side resolver fallback |
| Standalone root policy | `ProgramAnalyzer` | standalone compiler | Universe/time source code |
| Universe source/interface | `UniverseSourceProvider` | resolver/compiler/tooling | time-specific embedded source table |
| Declaration identity | `DeclarationId` | type store, dispatch, reflection | leaf name / re-export owner |
| Semantic nominal type | `TypeStore` declaration form | checker/compiler | reconstructed type by name |
| Period source behavior | `period.ph` + canonical semantic products | compiler/runtime | Period-specific semantic hook |
| T6 package surface | canonical Universe package sources | compiler/users | duplicate wrapper declarations |

---

# 9. Global invariants

### INV-01 — Accepted canonical dependencies use one module-resolution semantics

Once an import has resolved to a canonical `ModuleId`, its dependencies resolve through the same `ModuleResolver` rules regardless of whether the consumer is a project, standalone package, or standalone module.

### INV-02 — Standalone entry authority remains restricted

The repair must not grant a package-less standalone source arbitrary sibling/project imports.

### INV-03 — Strict linking receives a complete reachable dependency graph

Before strict link success, every authored import of every loaded reachable interface has a canonical resolved target represented in the discovery products required by the linker.

### INV-04 — Strict linker semantics are not weakened

Do not change the blocker into success by ignoring unresolved imports.

### INV-05 — No time-specific compiler/module workaround

Production module/compiler infrastructure contains no `time`, `calendar`, `Period`, `Timestamp`, or similar standard-library name/path special case for this repair.

### INV-06 — Identity is structural/canonical, not rendered spelling

`ModuleId` and `DeclarationId` determine identity. `ImportPath::to_string()` is presentation/compatibility projection, not an authority.

### INV-07 — Import source spelling follows execution HEAD

This amendment neither requires dotted logical paths nor requires `::`. It requires the AST/canonical products produced by the live parser.

### INV-08 — Period has one canonical declaration/type identity

Direct owner import, package re-export, implementation target, constructor result, and factory return all denote the same canonical `Period`.

### INV-09 — Universe source/bootstrap does not create a second Period declaration

A fix must not duplicate canonical Universe declarations merely to make a consumer typecheck.

### INV-10 — T6 civil values remain ordinary Universe source declarations

`Date`, `Time`, `DateTime`, `YearMonth`, `MonthDay`, and `DateOverflow` are published through the normal Universe module/package graph.

### INV-11 — Prelude policy is unchanged

Only `Duration` and `Timestamp` are temporal prelude promotions.

### INV-12 — UtcOffset bound remains VERIFY-FIRST

No arbitrary numeric bound may be introduced by this amendment.

### INV-13 — Runtime ownership remains correct

For standalone execution, imported already-bootstrapped Universe support modules must not be accidentally treated as runtime-owned standalone modules merely because they are present in semantic/linking support.

### INV-14 — Project/package compilation cannot regress

The shared discovery extraction must preserve current project/package transitive discovery semantics.

---

# 10. Non-goals

Do not:

- implement LANG004 logical-path syntax migration;
- change `ModuleId`, `ModulePath`, or project identity;
- make `ModuleLinker` resolve imports itself;
- use `link_with_unresolved_imports` in the strict compiler path;
- precompile the complete Universe for every standalone file as a shortcut;
- add `Period` to the prelude;
- add a time-root duplicate `Period` declaration;
- special-case `Period` return types by spelling;
- solve a semantic mismatch by widening `Dynamic`;
- move civil semantics to Rust/native code;
- redesign semantic incrementality;
- redesign REPL import semantics;
- begin P2 timezone work;
- select an unapproved `UtcOffset` bound;
- revive stale `time.md` material.

---

# 11. Expected impact map

## 11.1 Expected source areas

Primary:

```text
phalcom-core/src/modules/compile.rs
    shared/reused dependency closure
    standalone orchestration

phalcom-core/tests/core/modules/universe.rs
    standalone transitive import regressions

phalcom-core/core/universe/src/time/calendar/date.ph
    missing canonical dependency import if still present

phalcom-modules/src/builtin.rs
    T6 Universe node registration if still absent

phalcom-core/core/universe/src/time/calendar/package.ph
    T6 calendar publication

phalcom-core/core/universe/src/time/package.ph
    root publication where current P1 stage requires it

docs/implementation/STDL002-standard-library-buildout/C2-time-complete/
    checkpoint/update records
```

Conditional after evidence:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/checker/data_declaration.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/resolver.rs
phalcom-semantic/src/types/store.rs
```

Only enter these for the direct `Period` incident if it still reproduces after the module closure repair.

## 11.2 Expected tests

```text
phalcom-core/tests/core/modules/universe.rs
phalcom-core/tests/fixtures/language/time/**
phalcom-core/tests/fixtures/language/time/negative/**
phalcom-semantic/tests/semantic/capabilities/time.rs
```

Possibly:

```text
phalcom-modules/tests/**
```

only if the shared discovery implementation is moved into `phalcom-modules` or changes a modules-crate public contract.

## 11.3 Expected documentation/state

- update `STDL002.C2-CHECKPOINT.md`;
- create amendment walkthrough:
  `STDL002.C2.P1.A1-blocker-remediation-walkthrough.md`;
- create amendment handoff:
  `STDL002.C2.P1.A1-blocker-remediation-handoff.md`;
- eventual official `STDL002.C2.P1-walkthrough.md` and handoff must summarize this amendment when P1 itself finishes.

No competing `implementation-state.md`.

## 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically required.

STOP AND CONSULT before entering a materially different subsystem such as:

```text
parser grammar
LSP resolution
VM execution
native metadata model
generic type system architecture
incremental database architecture
P2 timezone backend
```

unless a task below explicitly authorizes that entry.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. `ModuleResolver` is canonical import-resolution authority.
2. `ModuleId` is canonical module identity.
3. `ModuleLinker` remains strict in compiler success paths.
4. Standalone user source remains prohibited from arbitrary sibling/project imports.
5. A canonical dependency's own imports use normal module semantics after that dependency is admitted.
6. No time-specific module/compiler workaround.
7. No duplicate `Period` declaration/type identity.
8. Imported Universe support modules do not become standalone-owned runtime modules by accident.
9. P1 temporal semantics remain source-first.
10. `UtcOffset` range remains a separate VERIFY-FIRST decision.
11. This amendment does not perform the `.` → `::` language migration.

## 12.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- helper names;
- whether the shared discovery helper remains private in `compile.rs` or moves to an existing execution-HEAD module;
- private result-struct decomposition;
- worklist collection type;
- exact regression test names;
- whether one or two focused core tests best prove closure;
- import source spelling based on execution HEAD;
- grouped import/export spelling based on execution HEAD;
- mechanical signatures/callers changed by repository drift.

## 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Local HEAD still contains one-hop `analyze_standalone_module` | `phalcom-core/src/modules/compile.rs` | If already fixed, run blocker regressions first and skip redundant refactor |
| `discover_and_analyze` still owns worklist closure | same file | Reuse the new execution-HEAD owner instead |
| Live import paths still use `.` | `phalcom-ast/src/parser.rs::parse_import_path` | If `::` landed, write all new source/fixtures with `::` |
| Grouped import/export syntax is available | parser + current fixtures | Use current canonical syntax; do not modify parser here |
| `ModuleResolver` can resolve/load Universe dependencies from the compiler's normal resolver context | `resolver.rs`, existing project path | If not, investigate existing Universe provider integration before inventing a composite provider |
| Existing T6 nodes are still absent | `UNIVERSE_NODES` / provider node inventory | Register only missing nodes |
| `date.ph` still lacks `Weekday` import | live file | If already corrected, do not churn it |
| Direct Period leaf import still fails | focused T3 reproducer | If green, record seam as already resolved and skip semantic edits |
| Period failure is declaration/type identity | diagnostic + semantic product comparison | If different, classify before editing |
| UtcOffset bound remains unapproved | checkpoint/P1 records | STOP at existing P1 bound gate; do not guess |

---

# 13. Global STOP / CONSULT triggers

STOP editing and prepare a consultation packet if any of the following occurs:

1. the standalone closure fix appears to require changing import grammar;
2. the fix appears to require weakening strict linker behavior;
3. the only apparent solution is a `time`/`Period` compiler special case;
4. canonical dependency resolution cannot be reused without introducing a second resolver;
5. two plausible closure designs would materially change module ownership or runtime ownership;
6. the module fix unexpectedly requires LSP or VM architecture changes;
7. direct Period semantic failure persists after one serious generic correction;
8. fixing Period appears to require changing canonical `DeclarationId` ownership, prelude policy, data-object semantics, or bootstrap/source authority;
9. tests pass only by converting a precise type/error to `Dynamic`;
10. a duplicate canonical Universe declaration/product would be introduced;
11. execution HEAD contradicts the assumption that standalone source may import Universe at all;
12. project/package behavior regresses and the cause is not a mechanical extraction mistake;
13. `UtcOffset` implementation is reached while the bound remains undecided;
14. T6 source integration exposes a new general language defect that cannot be fixed locally without changing another checkpoint's architecture;
15. the patch expands materially beyond the source areas listed in this amendment.

A triggered condition cannot be self-waived.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to three coherent correction cycles for:

- Rust imports/signature mismatches;
- helper extraction borrow/lifetime cleanup;
- wrong test selectors;
- syntax fixture drift after `::` migration;
- node-table ordering/constructor arguments.

A cycle is:

```text
inspect
→ identify concrete cause
→ make one coherent correction
→ rerun the smallest discriminating command
```

## 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow **one serious corrective attempt** for the same underlying semantic failure.

If the same Period identity mismatch persists: STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

Never rerun an unchanged failing test unless code or relevant preconditions changed.

---

# 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | canonical success | INV-01, INV-03 | standalone imports a non-leaf Universe module, e.g. `Timestamp` module that imports `Duration` |
| CV-02 | package closure | INV-01, INV-03 | standalone imports from the `time` package and package child dependencies link |
| CV-03 | hostile authority | INV-02 | package-less standalone sibling import remains rejected |
| CV-04 | strict failure | INV-04 | existing linker test still rejects genuinely missing resolved edge/import |
| CV-05 | no special case | INV-05 | production search finds no new time-name branching in module/compiler repair |
| CV-06 | spelling independence | INV-06, INV-07 | regressions use execution-HEAD syntax and assert canonical identities, not rendered strings |
| CV-07 | Period leaf identity | INV-08, INV-09 | direct owner-module Period import + factory/constructor behavior typechecks |
| CV-08 | comparator | INV-08 | passing direct non-prelude data import such as `Measurement` remains green |
| CV-09 | T6 closure | INV-10 | standalone/corpus import of `Date` traverses Month/Period/Weekday dependencies |
| CV-10 | civil validation | INV-10 | existing P1 T6 Date/Time/DateTime/partial positive/negative cases pass |
| CV-11 | package publication | INV-10, INV-11 | calendar/time package exports resolve to canonical owner declarations |
| CV-12 | runtime ownership | INV-13 | standalone runtime module set does not rematerialize canonical Universe support modules |
| CV-13 | project parity | INV-14 | project/package focused module tests remain green after shared discovery extraction |
| CV-14 | UtcOffset boundary | INV-12 | no offset source/bound is added unless P1 decision is already approved |

Not required in this amendment unless implementation unexpectedly crosses those boundaries:

- LSP tests;
- incremental cold/warm equivalence;
- GC/lifetime tests;
- VM bytecode-specific tests;
- workspace-wide tests.

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Sparse, discriminating feedback. Most implementation time belongs here.

### STABILIZE MODE

Run blocker regressions plus directly affected time/module suites.

### CERTIFY MODE

Run only amendment delivery gates. Full workspace certification remains P1/C2 final-gate work unless a repository rule explicitly requires it.

## 16.2 Verification ladder

```text
T0 — exact standalone transitive-import reproducer
T1 — focused core module regressions
T2 — affected core/module or time test module
T3 — semantic time + language-corpus integration
T4 — affected crate checks
T5 — workspace/release gates (DEFERRED to P1/C2 final certification)
```

## 16.3 Mandatory during BUILD

- exact new standalone transitive-import regression;
- exact standalone package-import regression or equivalent closure probe;
- `cargo check -p phalcom-core` after structural discovery refactor if Rust fanout is nontrivial;
- direct Period reproducer after G1;
- exact T6 source fixture when registering civil modules.

## 16.4 Mandatory during STABILIZE

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_universe -- --nocapture
```

or the narrower current module path if `modules_universe` naming drifted.

Then:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

Use exact execution-HEAD test selectors.

## 16.5 Conditional tests

If `phalcom-modules` production code changes:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test workspace_session
```

Run the full `phalcom-modules` crate only if the patch actually changes shared modules-crate semantics.

## 16.6 Explicitly deferred / DO NOT RUN during BUILD

Do not run:

```text
cargo test --workspace
workspace clippy
all language-corpus labels
LSP integration suite
P2 timezone suites
```

Reason: they do not discriminate this blocker and consume implementation time. Broader release evidence belongs to P1/C2 final certification.

---

# 17. Baseline / unrelated failure policy

Classify failures:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not weaken assertions or use tolerant linking to make broad gates green.
- Do not consume adviser compute on D unless it blocks amendment acceptance.

---

# 18. Semantic gate map

Repository convention uses `T<n>` for tasks and `G<n>` for plan-local gates. These gates serve the semantic-checkpoint role required by this amendment.

| Gate | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| G0 | T0 | Live takeover and red blocker shape are current | exact transitive/package repro + syntax verification | broad module/time suites |
| G1 | T1–T2 | Standalone compilation discovers complete canonical Universe dependency closure while preserving standalone authority and strict linking | CV-01–CV-05, CV-12–CV-13 | Period semantics, T6 civil runtime |
| G2 | T3–T4 | Direct Period import/factory/constructor uses one canonical declaration/type identity | CV-07–CV-08 | T6 package publication |
| G3 | T5–T6 | Existing T6 civil source is registered, published, and verified through normal compiler/module semantics | CV-09–CV-11, existing P1 T6 cases | UtcOffset if still decision-blocked; later T7 ordinary-package closure |
| G4 | T7 | Current blocker is closed and P1 can resume from its existing body without workaround | all blocker evidence, checkpoint update, negative gates | full P1/C2 release certification |

---

# 19. Tasks

## T0 — Re-ground execution HEAD and lock exact blocker regressions

### Purpose

Prove the live blocker before editing and lock syntax/ownership assumptions so the implementation does not solve a stale failure.

### Risk

- Semantic: MEDIUM
- Implementation fanout: local tests/read-only source inspection

### Preconditions

- local checkout available;
- user changes preserved;
- C2 checkpoint and current P1 body read.

### Consumes

- `ProgramAnalyzer::analyze_standalone_module`;
- current parser import syntax;
- existing module test conventions.

### Produces

- exact red reproducer for transitive standalone Universe import;
- exact red reproducer for root-package import;
- passing hostile comparator for forbidden standalone sibling import;
- execution-head syntax decision recorded.

### Files and symbols

**Read:**

```text
phalcom-core/src/modules/compile.rs
    ProgramAnalyzer::analyze_entry_selection
    ProgramAnalyzer::analyze_standalone_module
    ProgramAnalyzer::discover_and_analyze

phalcom-ast/src/parser.rs
    parse_import_path

phalcom-core/tests/core/modules/universe.rs
phalcom-core/tests/support/mod.rs
```

**Modify/tests:**

```text
phalcom-core/tests/core/modules/universe.rs
```

Prefer existing test helpers. Add a new fixture file only if that test module convention already uses fixtures.

### Required implementation shape

1. Confirm the one-hop standalone loop still exists.
2. Confirm the live source spelling:
   - dotted paths if current parser still consumes `Token::Dot`;
   - `::` paths if LANG004 has landed.
3. Add a minimal transitive import regression using a known non-leaf Universe target. Preferred discriminator:
   - the `Timestamp` module, because it imports `Duration`.
4. Add or adapt a root package regression importing a stable member from `time` such that failure occurs at package child linking, not because the imported member itself is unfinished.
5. Reuse the existing standalone sibling-import rejection test as hostile evidence; add one only if no equivalent exists.
6. Run only these tests and capture exact errors before production edits.

### Suggested source shape

Use execution-HEAD syntax.

Conceptually:

```phalcom
from <canonical timestamp owner module> import Timestamp

const epoch = Timestamp.epoch
System.print(epoch.unixNanoseconds)
```

and:

```phalcom
from <time package> import Clock

const value: Clock = SystemClock.instance
System.print(value.is(Clock))
```

Do not hard-code these exact bodies if the current public surface has mechanically drifted; choose an already-established T2–T5 member that isolates package closure.

### Forbidden approaches

- no production edit before reproducing;
- no Period-based package probe as the only package regression, because Period has an independent semantic incident;
- no parser change;
- no expected-output weakening.

### Tests to run now

Run the exact new tests only.

Example selector form:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core <new_exact_test_name> -- --exact --nocapture
```

Expected pre-fix result for transitive/root-package tests:

```text
FAIL
LinkError::UnresolvedImport
or corresponding ProgramCompileError
for an import owned by the direct Universe target
```

Expected hostile comparator:

```text
PASS
standalone sibling/project import remains rejected
```

### Tests explicitly deferred

- full `modules_universe`;
- semantic time;
- language corpus;
- workspace tests.

### Acceptance

- blocker reproduced at execution HEAD;
- exact missing edge/import recorded;
- import syntax recorded;
- hostile standalone authority comparator remains green.

### Local STOP / CONSULT triggers

- transitive/root package imports already pass before any patch and checkpoint diagnosis is stale;
- standalone Universe imports are no longer intended/allowed;
- parser syntax state conflicts with completed LANG004 records.

### Checkpoint update

Record execution revision and exact reproducer only if it materially changes the durable checkpoint diagnosis.

---

## G0 — Live blocker confirmation

### Purpose

Establish that implementation is solving the current repository, not the earlier `e2030124` snapshot.

### Required evidence

1. execution HEAD/status recorded;
2. transitive standalone import exact failure captured;
3. time-package import exact failure captured;
4. sibling standalone import restriction still passes;
5. live import spelling recorded.

### Do not broaden after G0

Do not run broad tests.

### G0 completion

- [ ] current blocker reproduced
- [ ] current syntax verified
- [ ] hostile standalone authority comparator green
- [ ] no production edit yet

---

## T1 — Extract/reuse one transitive program dependency-closure traversal

### Purpose

Remove the architectural divergence between standalone one-hop discovery and project/package transitive discovery without moving import-resolution authority out of `phalcom-modules`.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file only if an existing reusable abstraction requires it; preferred local core refactor

### Preconditions

- G0 COMPLETE;
- canonical `ModuleResolver` behavior understood;
- current `discover_and_analyze` worklist inspected.

### Consumes

- `ModuleResolver`;
- `ModuleResolver::resolve_import` or `resolve_import_with_trace`;
- `InterfaceBuilder`/loaded interfaces;
- `ModuleLinker`;
- current project/package discovery worklist.

### Produces

A single reusable reachable-dependency discovery mechanism consumed by:

```text
project/package path
standalone path after root admission
```

or, if execution HEAD already has an equivalent helper, one canonical helper used by both.

### Files and symbols

**Primary:**

```text
phalcom-core/src/modules/compile.rs
    ProgramAnalyzer::discover_and_analyze
    ProgramAnalyzer::analyze_standalone_module
```

**Inspect before editing:**

```text
phalcom-modules/src/resolver.rs
    ModuleResolver
    resolve_import
    resolve_import_with_trace
    ImportResolutionTrace

phalcom-modules/src/linker.rs
    strict link entry points
```

**Secondary only if needed:**

```text
phalcom-core/src/modules/canonical_universe.rs
    comparator pattern
```

### Current implementation

Project/package path:

```text
pending worklist
→ load source/interface
→ resolve imports
→ enqueue targets
→ strict link
```

Standalone path:

```text
parse entry
→ validate direct root
→ load direct target only
→ insert entry->target edge only
→ strict link
```

### Target implementation

The standalone entry remains special only at its **admission boundary**.

After each accepted direct target is canonicalized to a `ModuleId`, dependency traversal must use the same recursive/worklist mechanism as project/package discovery.

### Edit operations

1. OPEN `phalcom-core/src/modules/compile.rs`.
2. FIND `ProgramAnalyzer::discover_and_analyze`.
3. Identify the smallest coherent reusable unit that owns:
   - worklist/visited set;
   - source/interface insertion;
   - edge resolution;
   - target enqueueing;
   - package-interface/ancestor enqueueing required by canonical resolution.
4. EXTRACT that unit without changing project/package semantics.
5. FIND `ProgramAnalyzer::analyze_standalone_module`.
6. RETAIN:
   - standalone source parse/interface creation;
   - standalone root admission check;
   - synthetic entry identity;
   - standalone runtime ownership policy.
7. REPLACE direct-target-only loading with:
   - canonical direct resolution;
   - enqueue of accepted target(s);
   - shared transitive closure traversal.
8. Ensure every newly loaded dependency's authored imports are resolved through `ModuleResolver`.
9. If `resolve_import_with_trace` is already the cleanest way to obtain package-interface dependencies, use its canonical facts rather than reconstructing parent paths from strings.
10. SEARCH for another standalone-only recursive/discovery implementation before introducing a new helper.
11. Keep the helper private unless multiple crate consumers demonstrably need a public API.
12. CLEAN comments that still describe one-hop behavior.

### Code instructions

**STRUCTURAL — do not copy literally.**

A suitable private shape is conceptually:

```rust
struct DiscoveredProgramClosure {
    interfaces: BTreeMap<ModuleId, UnlinkedModuleInterface>,
    sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    resolved: BTreeMap<(ModuleId, String), ModuleId>,
}

fn discover_reachable_dependencies(
    resolver: &mut ModuleResolver<'_, impl SourceProvider>,
    seeds: impl IntoIterator<Item = ModuleId>,
    preseeded: ...,
) -> Result<DiscoveredProgramClosure, ProgramCompileError> {
    // one visited/pending worklist
    // resolver is the only import-target authority
}
```

However, execution HEAD may make this generic signature inappropriate because the standalone synthetic entry is not loadable through the same provider. In that case, use one of these equivalent mechanics:

```text
A. pre-seed standalone entry source/interface, then enqueue only accepted canonical dependencies;

or

B. parameterize private load operations while retaining ModuleResolver for every dependency edge.
```

The semantic design is fixed; the private helper signature is flexible.

### Provider constraint

Do **not** introduce a new general `CompositeSourceProvider` merely to make this extraction aesthetically uniform unless execution evidence proves it is necessary and no equivalent abstraction exists.

The standalone entry can remain pre-seeded while canonical Universe dependencies use existing resolver/provider machinery.

### Runtime ownership requirement

Preserve the current standalone distinction:

```text
semantic/link graph may contain Universe support modules
runtime_modules for standalone execution must not automatically become all linked Universe support modules
```

Do not change runtime ownership merely because discovery becomes transitive.

### Forbidden approaches

- duplicate the project worklist into a second standalone worklist;
- manually split/render logical path strings to derive module identity;
- special-case `universe.time`;
- use whole-Universe canonical compiled singleton as mandatory standalone input;
- make the linker call the resolver;
- change project/package semantics while extracting.

### Test changes required

No additional test beyond T0 unless extraction exposes a distinct hostile case.

### Tests to run now

Optional compile checkpoint after structural edit:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Proves:

- Rust API/caller migration compiles;
- exhaustive helper refactor is mechanically coherent.

Does not prove:

- transitive import semantics.

Then rerun only T0 exact blockers.

### Tests explicitly deferred

- full `modules_universe`;
- time corpus;
- semantic time.

### Acceptance

- both standalone regressions advance past the former missing child edge;
- existing project/package path still compiles;
- no duplicate traversal remains for the same semantics;
- runtime ownership remains entry-only for standalone if that is current contract.

### Local STOP / CONSULT triggers

- helper reuse requires changing module identity;
- the only viable implementation makes the linker perform resolution;
- Universe dependencies cannot be loaded through any existing canonical provider/resolver path and two materially different new provider designs emerge.

### Checkpoint update

Record the final helper/owner symbol only after G1 passes.

---

## T2 — Lock strict-linker and standalone-authority preservation

### Purpose

Prove the closure repair did not achieve success by weakening failure semantics or broadening standalone import authority.

### Risk

- Semantic: HIGH
- Implementation fanout: tests/local assertions

### Preconditions

- T1 implemented.

### Consumes

- new shared closure;
- existing strict linker tests;
- standalone restriction test.

### Produces

Evidence for INV-02, INV-04, INV-05, INV-13, INV-14.

### Files and symbols

Primary:

```text
phalcom-core/tests/core/modules/universe.rs
phalcom-modules/tests/linker.rs   # reuse, modify only if needed
```

### Required implementation shape

1. Reuse existing strict-linker regression proving an actually unresolved import is an error.
2. Reuse/extend standalone sibling-import rejection.
3. Add an assertion to the new successful standalone test if available that `AnalyzedProgram::runtime_modules` does not claim imported Universe support modules as standalone-owned runtime modules.
4. Search production compiler/module source for time-specific branching.

### Negative searches

```bash
rg -n 'universe\.time|time\.calendar|Period|Timestamp' \
  phalcom-core/src/modules \
  phalcom-modules/src
```

Expected:

- no new blocker-repair branches keyed to those names/paths;
- existing legitimate unrelated references must be enumerated if any.

Search the former one-hop signature:

```bash
rg -n 'resolved\.insert\(\(entry_id\.clone\(\),.*path\.to_string' \
  phalcom-core/src/modules/compile.rs
```

Expected:

- the old direct-only standalone mechanism is gone or no longer represents a one-hop-only traversal.

### Tests to run now

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core modules_universe -- --nocapture
```

If this aggregate is expensive at execution HEAD, run:
1. new exact success cases;
2. exact sibling rejection;
3. exact runtime-ownership assertion;
then leave the aggregate for G1.

If `phalcom-modules` changed, run exact linker test that asserts unresolved imports fail.

### Acceptance

- transitive/package imports succeed;
- sibling import remains rejected;
- strict missing-import behavior remains;
- no time-specific compiler/module fix exists;
- project/package focused module tests do not regress.

---

## G1 — Canonical standalone dependency closure

### Why this is a semantic checkpoint

T1 and T2 are meaningful only together: a reusable traversal is not correct unless it both completes canonical dependencies and preserves the standalone/root/linker boundaries.

### Entry conditions

- G0 COMPLETE.
- T1/T2 implemented.

### Working set

Primary:

```text
phalcom-core/src/modules/compile.rs
phalcom-core/tests/core/modules/universe.rs
```

Secondary only if changed:

```text
phalcom-modules/src/resolver.rs
phalcom-modules/src/linker.rs
phalcom-modules/tests/linker.rs
```

Out of scope:

```text
Period semantic code
T6 civil algorithms
parser
LSP
VM
```

### Semantic contract established

- standalone admitted Universe dependencies receive complete canonical transitive discovery;
- standalone source authority is not broadened;
- strict linker remains strict;
- no time-specific branch exists;
- runtime ownership remains correct.

### Hostile cases

- sibling standalone file is still rejected;
- genuinely unresolved dependency still fails;
- no `time` name/path branch was added.

### Required evidence

1. CV-01 transitive import — PASS.
2. CV-02 time-package closure — PASS.
3. CV-03 sibling authority hostile case — PASS.
4. CV-04 strict missing import — PASS/reused unchanged test.
5. CV-05 negative search — acceptable.
6. CV-12 runtime ownership — PASS.
7. CV-13 affected project/package comparator — PASS.

### Do not run yet

- semantic time;
- full language corpus;
- workspace tests.

### Escalate immediately if

- passing CV-01 requires tolerant linking;
- package import still fails with a missing edge owned by a loaded dependency after traversal;
- project/package discovery semantics diverge after helper extraction.

### G1 completion

- [ ] T1/T2 implemented
- [ ] closure regressions pass
- [ ] hostile authority case passes
- [ ] strict linker evidence passes
- [ ] negative special-case search passes
- [ ] checkpoint records new shared discovery owner
- [ ] no active incident

### Suggested commit grouping

```text
test(core/modules): reproduce standalone transitive Universe closure
fix(core/modules): unify standalone transitive dependency discovery
test(core/modules): preserve strict linking and standalone authority
```

---

## T3 — Reproduce and classify the direct Period semantic seam

### Purpose

Separate the remaining direct Period failure from module discovery and identify the exact canonical identity/type disagreement before editing semantic code.

### Risk

- Semantic: HIGH
- Implementation fanout: read-heavy; code edit conditional

### Preconditions

- G1 COMPLETE.

### Consumes

- canonical standalone closure;
- canonical Period source;
- semantic snapshot/type products;
- passing `Measurement` and canonical `Result` comparators.

### Produces

One of:

```text
A. Period direct leaf import is already green:
   no semantic repair needed;

or

B. exact mismatch:
   importer target
   declaration identity
   constructor result
   callable return
   TypeId/form
   narrowed to one owning semantic seam.
```

### Files and symbols

**Start with:**

```text
phalcom-core/core/universe/src/time/calendar/period.ph
phalcom-core/tests/fixtures/language/time/**
phalcom-semantic/tests/semantic/capabilities/time.rs
```

**Inspect only if failure reproduces:**

```text
phalcom-semantic/src/session.rs
    Universe data predeclaration/base-data setup
    workspace data rebuild path

phalcom-semantic/src/checker/data_declaration.rs
    build_data_semantics

phalcom-semantic/src/checker/expression.rs
    data-construction/call target logic

phalcom-semantic/src/resolver.rs
    LinkedTypeResolver / imported type resolution

phalcom-semantic/src/types/store.rs
    nominal declaration interning/form creation
```

Use the live symbol names; do not invent missing ones.

### Minimal reproducer

Use execution-HEAD import spelling:

```phalcom
from <canonical Period owner module> import Period

const p = Period.months(1)
System.print(p.months)
```

Also isolate a factory body if the diagnostic points inside Universe source.

### Identity ledger to capture

Record:

```text
import-selected DeclarationId:
impl target DeclarationId:
Period return-annotation DeclarationId:
raw Period(...) constructor DeclarationId:
published factory result TypeId/form:
constructor result TypeId/form:
```

Expected declaration owner:

```text
canonical Universe time/calendar/period module
```

The package/root re-export may refer to that declaration but must not own another declaration.

### Passing comparators

At least one:

```text
direct non-prelude Measurement import
canonical Result identity test
```

The purpose is to disprove broad "all non-prelude data" hypotheses.

### Forbidden approaches

- no `if name == "Period"`;
- no prelude promotion;
- no duplicate alias declaration;
- no package-owned wrapper type;
- no weakening return mismatch to Dynamic;
- no broad semantic baseline rewrite before identity evidence.

### Tests to run now

Only:

1. minimal Period exact reproducer;
2. one passing comparator.

### Acceptance

- failure either disappears after G1 and is recorded as stale/coupled evidence, or
- one exact semantic seam is identified before code edits.

### Local STOP / CONSULT triggers

- evidence shows multiple canonical Period declarations exist by design;
- fixing the mismatch requires changing data-object semantics or source/native authority;
- root cause cannot be narrowed after one focused identity trace.

### Checkpoint update

Record only if the earlier checkpoint statement conflating Period with package-linking must be corrected—which is expected.

---

## T4 — Repair Period canonical identity generically, only if T3 proves it necessary

### Purpose

Eliminate the direct Period mismatch at the owning generic semantic seam without introducing a temporal special case.

### Risk

- Semantic: HIGH
- Implementation fanout: potentially cross-semantic; bounded by T3 evidence

### Preconditions

- T3 produced a concrete identity/type mismatch.
- If T3 is green, SKIP this task and record `NOT_NEEDED`.

### Consumes

- exact T3 identity ledger;
- one passing comparator;
- canonical semantic owner identified by T3.

### Produces

- one generic semantic correction;
- Period direct import/factory/constructor canonicality;
- regression defeating the specific duplicate/reconstruction mechanism.

### Required implementation shape

**INVESTIGATE-BEFORE-EDIT**

Do not prescribe a semantic file change until T3 identifies the mismatch.

Permitted repair categories include, only if evidence selects one:

```text
imported canonical declaration target publication
data-constructor result type projection
impl target declaration reuse
Universe baseline/workspace product reuse
nominal type interning with the canonical DeclarationId
```

The correction must improve the general identity invariant, not `Period`.

### Likely bootstrap/source hostility to test

If T3 points to canonical baseline versus workspace reanalysis:

```text
canonical Universe baseline declaration
    must be enriched/reused

imported Universe source in consumer analysis
    must not create/rebuild a competing declaration identity
```

Do not suppress Universe source entirely if compiler lowering/provenance legitimately consumes it. Repair ownership at the exact product boundary.

### Test changes required

Add the smallest regression at the owner layer that asserts the actual canonical product:

```text
DeclarationId equality
or
TypeId/form equality
or
constructor/factory published result identity
```

Do not rely only on output text if the defect is identity.

### Tests to run now

1. exact owner-layer semantic regression;
2. direct Period source reproducer;
3. passing comparator.

### Debugging budget

One serious semantic correction.

If the same mismatch persists after that correction: STOP AND CONSULT.

### Acceptance

- direct Period import/factory compiles;
- identity regression passes;
- comparator remains green;
- no time-specific semantic branch exists.

### Checkpoint update

Record:
- actual root cause;
- corrected symbol;
- canonical identity invariant;
- consultation if any.

---

## G2 — Canonical Period identity

### Why this is a semantic checkpoint

The module graph can be correct while semantic declaration/type identity is still wrong. G2 proves these concerns are independently closed.

### Entry conditions

- G1 COMPLETE.
- T3 classification complete.
- T4 complete or `NOT_NEEDED`.

### Semantic contract established

Direct `Period` owner import, implementation target, constructor result, and factory return use one canonical declaration/type identity.

### Hostile cases

- passing non-prelude `Measurement` path remains unchanged;
- no prelude promotion;
- no package/root duplicate Period declaration;
- no name-based special case.

### Required evidence

1. CV-07 direct Period identity — PASS.
2. CV-08 comparator — PASS.
3. semantic negative search for Period special casing — zero new production hits.
4. if semantic code changed, relevant focused semantic test module — PASS.

### Do not run yet

Full time corpus remains G3 unless the exact Period fixture lives there and is the cheapest discriminator.

### G2 completion

- [ ] Period direct import/factory is canonical
- [ ] identity evidence passes
- [ ] hostile comparator passes
- [ ] no special case exists
- [ ] no active incident

### Suggested commit grouping

If a repair is needed:

```text
test(semantic): reproduce canonical Period identity mismatch
fix(semantic): preserve canonical imported data identity
```

If no repair is needed, no semantic commit is required.

---

## T5 — Make the existing T6 civil source graph complete and publishable

### Purpose

Remove source/module-registration omissions that were intentionally deferred while the compiler blocker was active.

### Risk

- Semantic: MEDIUM-HIGH
- Implementation fanout: Universe source + node inventory + package publication

### Preconditions

- G2 COMPLETE.
- live P1 T6 source scaffold inspected.
- import/export syntax verified against execution HEAD.

### Consumes

- `Date`, `DateOverflow`, `Time`, `DateTime`, `YearMonth`, `MonthDay` source already present;
- existing `Month`, `Weekday`, `Period`, `Gregorian`;
- canonical Universe node/provider model.

### Produces

Canonical registered modules and package exports for the T6 civil types that are already implemented.

### Files and symbols

**Read/modify as required:**

```text
phalcom-core/core/universe/src/time/calendar/date.ph
phalcom-core/core/universe/src/time/calendar/time.ph
phalcom-core/core/universe/src/time/calendar/datetime.ph
phalcom-core/core/universe/src/time/calendar/partial.ph

phalcom-core/core/universe/src/time/calendar/package.ph
phalcom-core/core/universe/src/time/package.ph

phalcom-modules/src/builtin.rs
    UNIVERSE_NODES or execution-HEAD equivalent
```

### Required implementation shape

#### A. Source dependency completeness

Inspect every T6 source file for referenced non-prelude declarations.

At planning revision, `date.ph` uses:

```phalcom
Weekday.fromNumber(...)
```

without importing `Weekday`.

If still present, add the canonical import using execution-HEAD spelling.

Do not rely on accidental broad Universe name fallback for non-prelude temporal declarations.

#### B. Node registration

Register each missing physical module under its canonical logical path:

```text
time / calendar / date
time / calendar / time
time / calendar / datetime
time / calendar / partial
```

Use the existing `UniverseNodeSpec` pattern and correct `ModuleKind`.

Do not create one node per declaration when multiple declarations intentionally share one source module (`partial.ph`).

#### C. Calendar package publication

Expose/import/export the canonical owner declarations required by P1 T6:

```text
Date
DateOverflow
Time
DateTime
YearMonth
MonthDay
```

alongside existing:

```text
Month
Weekday
Period
```

Use execution-HEAD canonical grouped syntax if it has landed; otherwise use the current parser-supported form. Do not modify grammar.

#### D. Time root publication

Publish the T6 declarations at the time root only to the extent the existing P1 task requires at this stage.

Do not publish `UtcOffset` until it actually exists with an approved bound.

No wrappers. Re-exports must preserve canonical owner identity.

### Forbidden approaches

- no duplicate types in `time/package.ph`;
- no prelude promotion for civil types;
- no moving Gregorian algorithms into Rust;
- no import syntax migration;
- no UtcOffset placeholder with guessed bound.

### Tests to add now

Add the smallest positive fixture proving one transitive T6 module now compiles through the formerly broken standalone path.

Preferred target: `Date`, because it exercises dependencies on Month/Gregorian, Period, and Weekday.

Use current source syntax.

### Tests to run now

Run only that exact fixture/test after registration.

Then optional:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

if node registration changes Rust code.

### Tests explicitly deferred

- full T6 behavior matrix;
- root grouped package fixture;
- ordinary project/package T7 fixture;
- UtcOffset.

### Acceptance

- provider loads every new module;
- package interfaces link;
- `Date` canonical import succeeds through standalone production compiler;
- no unresolved child import remains.

### Local STOP / CONSULT triggers

- provider registration requires a new module kind/topology rule;
- civil source requires hidden broad prelude visibility to compile;
- import cycle appears that contradicts P1 dependency direction.

### Checkpoint update

Record published T6 module paths once the exact fixture passes.

---

## T6 — Complete focused T6 civil verification already required by P1

### Purpose

Resume—not redesign—the existing P1 T6 task far enough to prove the civil modules are usable after blocker removal.

### Risk

- Semantic: MEDIUM-HIGH
- Implementation fanout: time source + time corpus

### Preconditions

- T5 source graph registered.
- G2 canonical Period identity established.

### Consumes

Existing P1 T6 semantics:

```text
Date
DateOverflow
Time
DateTime
YearMonth
MonthDay
Period application
Gregorian epoch-day conversion
```

### Produces

Focused evidence for the existing T6 civil layer excluding unresolved `UtcOffset` work.

### Required implementation shape

Use the P1 body as the semantic specification. This amendment must not restate or alter algorithms.

Ensure focused cases cover, as already required by P1:

```text
Date valid month edges
Date day 0 rejects
April 31 rejects
Feb 29 2024 succeeds
Feb 29 1900 rejects
Feb 29 2000 succeeds
known weekday anchors
1970-01-01 epoch-day 0
1969-12-31 epoch-day -1
civil <-> epoch-day representative round trips
Time lower/upper valid boundaries
hour 24 rejects
minute 60 rejects
second 60 rejects
nanosecond 1_000_000_000 rejects
DateTime projections
Period month -> overflow -> day order
Jan 31 + month Constrain
Jan 31 + month Reject
YearMonth positive/negative movement
MonthDay April 31 rejection
MonthDay Feb 29 behavior
Period ordering unavailable
```

Reuse existing fixtures before adding new ones.

### Forbidden approaches

- no new property-test dependency;
- no broad rewrite of already passing Gregorian helpers;
- no DateTime + Duration addition;
- no Time.now;
- no timezone state;
- no UtcOffset guess.

### Tests to run now

First exact newly enabled civil fixtures.

Then at G3 run the focused suites.

### UtcOffset boundary

At the point the existing P1 T6 task reaches `UtcOffset`:

1. read the current checkpoint for an approved bound decision;
2. if approved, resume the existing P1 T6 instructions;
3. if still unresolved, STOP AND CONSULT under the existing P1 mandatory VERIFY-FIRST rule.

The module/Period blocker is considered removed even if this separate public-API decision remains awaiting approval. Do not conflate them.

### Acceptance

All non-UtcOffset T6 civil obligations made reachable by this amendment are focused-tested through normal module/compiler semantics.

### Checkpoint update

Record only coherent T6 evidence, not every fixture.

---

## G3 — T6 civil graph and runtime integration

### Why this is a semantic checkpoint

Registration alone proves nothing; civil source behavior alone is insufficient if it cannot be imported through normal module semantics. G3 proves both together.

### Entry conditions

- G2 COMPLETE.
- T5/T6 implemented.

### Working set

Primary:

```text
time/calendar source
Universe node inventory
time/calendar package
time package
time semantic tests
time language corpus
```

Out of scope:

```text
UtcOffset if decision-blocked
T7 ordinary-package finalization
P2 timezone work
```

### Required evidence

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic capabilities::time
```

Proves:
- public declaration/type contracts observed by the semantic capability suite remain coherent.

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test language-corpus time
```

Proves:
- positive temporal source compiles/runs through the production standalone compiler path.

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test language-corpus time_negative
```

Proves:
- intended temporal invalid states/operations continue to reject.

Also run the exact standalone root-package regression from G1 again after publication changes.

### Hostile cases

- Date cannot obtain Weekday through accidental prelude leakage;
- Period ordering remains unavailable;
- invalid civil constructors reject;
- no duplicate time-root declaration identity.

### Do not run yet

- full workspace;
- P2 tests;
- final ordinary-package T7 suite unless T7 preconditions are now fully met.

### G3 completion

- [ ] civil modules registered
- [ ] package publication canonical
- [ ] semantic time suite passes
- [ ] positive time corpus passes
- [ ] negative time corpus passes
- [ ] root package closure still passes
- [ ] no active blocker incident

### Suggested commit grouping

```text
fix(stdlib/time): complete T6 civil module dependencies
feat(universe): publish T6 civil modules
test(stdlib/time): verify civil values through canonical module graph
```

---

## T7 — Close blocker bookkeeping and hand execution back to P1

### Purpose

Remove stale blocker wording, preserve the actual root cause/evidence, and leave an unambiguous next action for Luna's next P1 session.

### Risk

- Semantic: LOW
- Implementation fanout: documentation/state only

### Preconditions

- G3 COMPLETE.
- Any Period repair recorded.
- current UtcOffset decision status known.

### Files

Modify:

```text
docs/implementation/STDL002-standard-library-buildout/C2-time-complete/
    STDL002.C2-CHECKPOINT.md
```

Create:

```text
STDL002.C2.P1.A1-blocker-remediation-walkthrough.md
STDL002.C2.P1.A1-blocker-remediation-handoff.md
```

### Required checkpoint correction

Replace/qualify wording that attributes the package failure to generic root publication/linker behavior.

Record the actual mechanism:

```text
standalone compiler discovery published only direct entry -> Universe target edges;
strict linker then correctly rejected target-owned imports absent from the graph.
```

Record separately:

```text
direct Period semantic seam:
    root cause/result/evidence
```

Do not continue claiming the same cause unless T3 evidence unexpectedly proved that statement.

### Required next action

The checkpoint next action must identify the exact current P1 position.

Typical result:

```text
Current blocker removed.

Resume STDL002.C2.P1 T6:
- finish any remaining civil acceptance;
- perform mandatory UtcOffset bound VERIFY-FIRST decision if still pending;
- implement/verify UtcOffset only after approval;
- continue existing P1 later gate/T7 package and ordinary-package closure;
- do not reopen module discovery.
```

If T6 is fully complete including UtcOffset by execution time, point to the next actual P1 task instead.

### Acceptance

A fresh Luna session reading only the checkpoint + P1 body + amendment handoff knows exactly where to resume without conversation context.

---

## G4 — Blocker closure and P1 resume

### Purpose

Certify this amendment, not the entire P1/C2 program.

### Required evidence

All of:

```text
G1 COMPLETE
G2 COMPLETE
G3 COMPLETE
```

plus final negative/deletion gates below.

### Completion claim

After G4:

> Standalone source consumes transitive canonical Universe dependencies through normal strict module semantics; the direct Period identity seam is closed or proven already resolved; T6 civil modules are reachable and focused-tested; STDL002.C2.P1 may resume from its existing body without a time-specific compiler/module workaround.

### G4 completion

- [ ] all amendment tasks complete
- [ ] G1/G2/G3 evidence passes
- [ ] blocker wording corrected
- [ ] walkthrough created
- [ ] handoff created
- [ ] deferred gates named
- [ ] UtcOffset decision status truthful
- [ ] no active INCIDENT
- [ ] next P1 resume action exact

---

# 20. Verification gates summary

## Gate G0 — Reproducer

Run exact new standalone regressions.

Expected pre-patch:

```text
transitive Universe import: FAIL at child edge
time package import: FAIL at package child edge
sibling standalone import: PASS as rejected
```

## Gate G1 — Module closure

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core modules_universe -- --nocapture
```

Use narrower exact tests first if the aggregate is slow.

If modules crate changed:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test workspace_session
```

## Gate G2 — Period identity

Run:

```text
exact Period direct-import regression
one owner-layer semantic identity regression if a fix was needed
Measurement/Result comparator
```

Do not broaden if green.

## Gate G3 — T6 integration

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

## Gate G4 — Amendment delivery

Run:

```bash
cargo fmt --all -- --check
git diff --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

If semantic production code changed:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

If modules production code changed:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
```

Do not automatically run workspace tests.

---

# 21. Final focused acceptance ledger

The implementer must fill actual results; do not pre-mark PASS.

| Coverage ID | Test/evidence | Result |
|---|---|---|
| CV-01 | standalone transitive Universe import | UNVERIFIED |
| CV-02 | standalone `time` package import | UNVERIFIED |
| CV-03 | standalone sibling import rejection | UNVERIFIED |
| CV-04 | strict unresolved import regression | UNVERIFIED |
| CV-05 | production special-case negative search | UNVERIFIED |
| CV-06 | execution-HEAD import spelling + canonical target assertions | UNVERIFIED |
| CV-07 | direct Period canonical identity | UNVERIFIED |
| CV-08 | Measurement/Result comparator | UNVERIFIED |
| CV-09 | Date transitive T6 import | UNVERIFIED |
| CV-10 | focused civil validation | UNVERIFIED |
| CV-11 | package publication canonicality | UNVERIFIED |
| CV-12 | standalone runtime ownership | UNVERIFIED |
| CV-13 | project/package discovery comparator | UNVERIFIED |
| CV-14 | no guessed UtcOffset bound | UNVERIFIED |

---

# 22. Final negative/deletion gates

## 22.1 No time-specific module fix

```bash
rg -n 'universe\.time|time\.calendar|Period|Timestamp' \
  phalcom-core/src/modules \
  phalcom-modules/src
```

Expected:

- no blocker-repair code branches on time names;
- enumerate any pre-existing legitimate references.

If `::` has landed, also search equivalent textual spelling only as a hygiene check; do not treat string rendering as identity.

## 22.2 One-hop standalone mechanism retired

Search the old direct-only pattern:

```bash
rg -n 'resolved\.insert\(\(entry_id\.clone\(\),.*path\.to_string' \
  phalcom-core/src/modules/compile.rs
```

Expected:

- zero old direct-only standalone occurrences;
- if a structurally similar insertion remains inside the shared traversal, document why it is now per-importer and transitive.

## 22.3 Strict linker not bypassed

```bash
rg -n 'link_with_unresolved_imports' \
  phalcom-core/src/modules/compile.rs \
  phalcom-modules/src
```

Expected:

- no new strict compiler-path use introduced by this amendment;
- existing tolerant workspace uses may remain and must not be deleted merely for this gate.

## 22.4 Standalone authority remains

```bash
rg -n 'StandaloneImportRequiresPackageContext' phalcom-core
```

Expected:

- intentional production/test occurrences remain;
- the repair did not retire the standalone restriction.

## 22.5 No Period semantic name hack

If semantic files changed:

```bash
rg -n '"Period"|==\s*"Period"|name.*Period|Period.*name' \
  phalcom-semantic/src
```

Expected:

- zero new semantic special cases for Period identity.

---

# 23. Deferred-evidence audit

At amendment completion:

```text
No deferred test/check may disappear silently.
```

Each deferred item must be one of:

1. executed successfully;
2. still explicitly assigned to the existing P1 later gate;
3. excluded as unrelated with justification;
4. recorded as a known release blocker.

Expected legitimate deferrals after this amendment may include:

```text
UtcOffset tests
    → P1 T6 after approved bound decision, if still unresolved

ordinary-package final T7 fixture
    → existing P1 T7 when all T6 public declarations exist

full workspace test/clippy
    → P1/C2 certification

P2 timezone suites
    → STDL002.C2.P2
```

---

# 24. Checkpoint bookkeeping

## 24.1 At amendment start

Update `STDL002.C2-CHECKPOINT.md` only if repository convention expects active amendment recording.

Record:

```text
execution revision
active P1 blocker amendment
confirmed live import syntax
```

Do not rewrite historical P1 completion claims.

## 24.2 During amendment

Update only on durable events:

- shared discovery owner established;
- Period root cause confirmed/corrected;
- T6 module publication established;
- consultation/amendment;
- meaningful deferred failure;
- G1/G2/G3 completion.

## 24.3 At amendment completion

Record:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only if evidence supports those terms.

Also record:

- exact files/symbols changed;
- tests actually run;
- tests deferred;
- Period result;
- UtcOffset decision status;
- next P1 action.

Do not claim `STDL002.C2.P1` itself COMPLETE unless the original P1 acceptance gate has also passed.

---

# 25. Amendment walkthrough deliverable

Create:

```text
docs/implementation/STDL002-standard-library-buildout/C2-time-complete/
STDL002.C2.P1.A1-blocker-remediation-walkthrough.md
```

It must contain:

- final amendment result;
- execution revision;
- actual shared discovery architecture;
- exact standalone policy retained;
- exact root cause of former package-link failure;
- Period outcome/root cause;
- T6 node/publication changes;
- import syntax observed at execution;
- tests added;
- tests actually run;
- tests deferred;
- verification classification;
- deviations from this plan;
- consultations;
- residual risks;
- exact next P1 task.

---

# 26. Amendment handoff deliverable

Create:

```text
docs/implementation/STDL002-standard-library-buildout/C2-time-complete/
STDL002.C2.P1.A1-blocker-remediation-handoff.md
```

It must contain:

```text
current revision
G1/G2/G3/G4 status
shared discovery helper/owner
standalone import/root invariant
Period canonical identity invariant
T6 modules now published
UtcOffset decision status
next P1 objective
must-read files
deferred failures
first recommended commands
things not to redesign/re-explore
```

The handoff must explicitly say:

```text
Do not re-investigate the former one-hop package-link blocker unless a regression fails.
Resume the existing P1 body from the recorded T6/T7 point.
```

---

# 27. Completion truth table

Do not conflate:

```text
source edited
tests added
exact regressions passed
focused suites passed
amendment accepted
P1 accepted
C2 accepted
release certified
```

The expected amendment terminal state is normally:

```text
A1:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

P1:
    remains active until its original final acceptance passes
```

---

# 28. Suggested staged commit groups

Use coherent commits, not one commit per micro-edit.

### Group 1 — Reproducer + module closure

```text
test(core/modules): lock standalone transitive Universe imports
fix(core/modules): unify reachable dependency discovery
```

### Group 2 — Period identity, only if needed

```text
test(semantic): pin canonical Period data identity
fix(semantic): preserve canonical imported data identity
```

Skip if T3 proves no production repair is needed.

### Group 3 — T6 resume

```text
fix(stdlib/time): complete civil module dependencies
feat(universe): publish T6 civil modules
test(stdlib/time): verify civil values through canonical imports
```

### Group 4 — Records

```text
docs(stdlib): record P1 blocker closure and resume state
```

Do not force commits if the execution environment/user workflow says otherwise.

---

# 29. Known scope exclusions

Explicitly excluded:

```text
LANG004 logical-path `::` migration itself
general parser work
LSP import completion migration
module-system redesign
whole semantic baseline refactor without Period evidence
P2 timezone database/backend
named time zones
ZonedDateTime
canonical time parsing/formatting
reactor/timer redesign
new temporal bytecodes/value variants
stale time specification work
unapproved UtcOffset range choice
full repository baseline cleanup
```

---

# 30. Failure protocol

If any required gate fails unexpectedly:

## 30.1 Record exact reproduction

```text
command:
test:
error:
```

## 30.2 Trace direct path

Examples:

```text
standalone fixture
→ EntrySelection::Module
→ ProgramAnalyzer
→ dependency discovery
→ ModuleResolver
→ ModuleLinker
→ failure
```

or:

```text
Period fixture
→ import binding
→ DeclarationId
→ impl/factory
→ constructor result TypeId
→ return check
→ failure
```

## 30.3 Find a passing comparator

Examples:

```text
leaf Month import passes, Timestamp transitive import fails
Measurement direct data import passes, Period direct import fails
project import passes, standalone import fails
```

## 30.4 Classify

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## 30.5 State narrow repair boundary before editing

Do not patch outward first.

## 30.6 Rejected broad repairs

Do not:

- restore name fallbacks;
- weaken linker assertions;
- accept unresolved imports;
- turn semantic errors into Dynamic;
- special-case time;
- modify parser syntax;
- add duplicate declarations.

A failed required gate makes the active gate `INCIDENT`, not "mostly complete".

---

# 31. Supervisor-facing completion report template

At each semantic gate:

```text
Gate G<N> COMPLETE

Established:
    <one dominant semantic claim>

Changed:
    <path> — <symbol/responsibility>

Evidence:
    <command> — PASS
    <command> — PASS

Hostile cases:
    <case> — PASS

Negative gates:
    <search/assertion> — expected result

Deferred:
    <gate> → <destination>

Unexpected findings:
    none | <fact>

Next:
    G<N+1> / T<N>
```

For a failed gate:

```text
Gate G<N> INCIDENT

Observed:
...
Classification:
...
Narrow repair boundary:
...
Consultation trigger:
...
```

---

# 32. Final broad gates for this amendment

At G4, after focused semantic evidence is green:

```bash
cargo fmt --all -- --check
git diff --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Conditional:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

only if semantic production code changed.

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
```

only if modules production code changed.

The amendment does **not** require:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Those remain broader P1/C2 delivery gates unless repository policy at execution HEAD explicitly requires them.

---

# 33. State-file completion requirements

`STDL002.C2-CHECKPOINT.md` must finish this amendment with:

## Established invariants

At minimum:

```text
A1-INV-01 standalone canonical dependencies receive transitive discovery
A1-INV-02 standalone sibling authority remains prohibited
A1-INV-03 strict linker remains strict
A1-INV-04 Period direct import uses canonical identity
A1-INV-05 T6 civil modules use normal Universe publication
```

## Decisions

Record:

- final shared discovery owner;
- whether import spelling at execution was `.` or `::`;
- Period root cause or `NOT_REPRODUCED_AFTER_G1`;
- UtcOffset decision status.

## Evidence ledger

| Gate | Command | Result | Proves |
|---|---|---|---|
| G1 | ... | ... | canonical standalone dependency closure |
| G2 | ... | ... | canonical Period identity |
| G3 | ... | ... | T6 civil integration |
| G4 | ... | ... | amendment delivery readiness |

## Deferred gates

Each with destination.

## Active incident

Must be:

```text
None.
```

before G4 COMPLETE.

## Next resume action

Must point to an exact existing P1 task/gate.

---

# 34. Release-complete criteria for this amendment

This amendment is complete only when:

- [ ] execution HEAD was re-grounded;
- [ ] live import syntax was verified rather than assumed;
- [ ] transitive standalone Universe import regression passes;
- [ ] root `time` package dependency closure passes;
- [ ] standalone sibling authority remains rejected;
- [ ] strict linker semantics remain unchanged;
- [ ] no time-specific compiler/module workaround exists;
- [ ] direct Period canonical identity passes;
- [ ] any Period semantic repair is generic and owner-layer tested;
- [ ] T6 civil modules are registered/published through canonical Universe topology;
- [ ] T6 focused semantic/runtime tests pass for implemented civil scope;
- [ ] no guessed UtcOffset bound was introduced;
- [ ] checkpoint diagnosis is corrected;
- [ ] amendment walkthrough exists;
- [ ] amendment handoff exists;
- [ ] no unresolved INCIDENT remains;
- [ ] every deferred gate has a destination;
- [ ] P1's exact next resume action is recorded.

The amendment does **not** imply `STDL002.C2.P1` or `STDL002.C2` is complete.

---

# 35. Plan self-review

## Repository grounding

- [x] exact inspected remote HEAD recorded;
- [x] current standalone compiler path inspected;
- [x] current project/package discovery path inspected;
- [x] canonical Universe build path inspected;
- [x] resolver/import product architecture inspected;
- [x] current import parser syntax inspected;
- [x] current T6/checkpoint state inspected;
- [x] relevant test path identified;
- [x] local worktree uncertainty stated explicitly.

## Architecture

- [x] `ModuleResolver` remains import authority;
- [x] `ModuleId` remains identity authority;
- [x] `ModuleLinker` remains strict consumer;
- [x] standalone admission and canonical dependency resolution are separated;
- [x] no second resolver/identity planned;
- [x] Period semantic incident is separate from module closure;
- [x] bootstrap/source hostility is bounded and evidence-driven;
- [x] runtime ownership distinction is explicit.

## Luna executability

- [x] fixed/flexible/verify-first areas explicit;
- [x] STOP/CONSULT triggers objective;
- [x] debugging loops bounded;
- [x] repository wandering bounded;
- [x] import syntax drift handled mechanically;
- [x] no unverified semantic fix is presented as EXACT.

## Testing

- [x] tests tied to invariants;
- [x] hostile standalone authority case included;
- [x] strict linker hostile case included;
- [x] Period comparator included;
- [x] broad tests deferred intentionally;
- [x] every broad command states its purpose.

## Lifecycle

- [x] amendment does not create a third official C2 plan;
- [x] C2 checkpoint remains durable state authority;
- [x] amendment walkthrough/handoff required;
- [x] handback into original P1 body is explicit;
- [x] UtcOffset's separate decision gate is preserved.

---

# 36. First commands for Luna

Start with:

```bash
git status --short
git rev-parse HEAD
git log -n 5 --oneline

rg -n 'fn analyze_standalone_module|fn discover_and_analyze' \
  phalcom-core/src/modules/compile.rs

rg -n 'fn parse_import_path|Token::ColonColon|while self\.eat\(&Token::Dot\)' \
  phalcom-ast/src/parser.rs

rg -n 'universe\.time|universe::time|Weekday|Period' \
  phalcom-core/core/universe/src/time \
  docs/implementation/STDL002-standard-library-buildout/C2-time-complete/STDL002.C2-CHECKPOINT.md
```

Then read the exact existing standalone module tests and begin T0.

Do not begin by editing `time/package.ph`. The dependency-closure defect must be proven and repaired first.
