---
id: LANG006.C1.P1
category: LANG
program: LANG006
checkpoint: LANG006.C1
kind: implementation-plan-body
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG006.C1-CHECKPOINT.md
  - LANG006.C1.P1-HEADER.md
follows: null
supersedes: null
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
header_artifact: LANG006.C1.P1-HEADER.md
---

# LANG006.C1.P1 — Patch-Grade Implementation Task Body

> **Execution requirement:** Read `LANG006.C1.P1-HEADER.md` in full before executing this file. This body deliberately does not restate the complete architectural model. All `INV-*`, `CV-*`, `FIXED`, `VERIFY-FIRST`, debugging-budget, non-goal, and STOP/CONSULT rules in the header are binding here.
>
> Execute tasks in order. A task may be split into smaller local commits/working steps, but do not advance past a named gate while its required evidence is missing. Do not preserve two semantic authorities merely to keep later tasks easier.

---

# Task ledger

| Task | Objective | Primary gate |
|---|---|---|
| T0 | Re-ground live repository and build the migration/collision census | G0 — scope/census accepted |
| T1 | Add shared modifier model and lexical/AST foundations | G1 — one structural front-end model |
| T2 | Implement declaration-modifier grammar, target validation, and built-in metadata attributes | G2 — canonical syntax parses and invalid syntax fails |
| T3 | Publish canonical module/package/project access provenance | G3 — exact ownership identities available to semantics/runtime lowering |
| T4 | Converge semantic visibility and member surfaces on the new model | G4 — static access semantics are canonical |
| T5 | Replace runtime internal privilege with structured declaration-derived access authority | G5 — runtime invocation parity |
| T6 | Converge native metadata, descriptor verification, and native installation | G6 — exact source/native parity |
| T7 | Add the bounded `foreign` semantic/lowering scaffold | G7 — foreign is distinct and fails explicitly when unbound |
| T8 | Classify and migrate canonical Universe source and native registrations | G8 — Universe speaks canonical new syntax |
| T9 | Delete legacy prefix/attribute inference and compatibility namespaces | G9 — one authority remains |
| T10 | Converge LSP, reflection, syntax tooling, and normative specs | G10 — presentation/spec parity |
| T11 | Stabilize, certify, record walkthrough/handoff, and close P1 | G11 — P1 acceptance evidence |

---

# T0 — Re-ground the live repository and freeze the migration census

## Objective

Establish the exact live implementation baseline, detect drift since planning, enumerate every legacy surface that P1 must migrate, and classify naming/keyword collisions before any architectural code edit.

## Preconditions / authority

Read:

```text
LANG006.C1-CHECKPOINT.md
LANG006.C1.P1-HEADER.md
AGENTS.md
live workflow documents named in header §4
```

Reference:

```text
INV-01
INV-02
INV-03
INV-09
INV-15
INV-19
header §§2–5, 13–14, 18–20, 24
```

## Required actions

### T0.1 Re-ground revision and working tree

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Record the live revision in the implementation walkthrough/work log.

If the working tree contains unrelated changes:

- do not stage/rewrite them;
- distinguish them from LANG006 edits;
- use the repository workflow's existing dirty-tree discipline.

### T0.2 Verify live authority files and architecture

Re-read the exact live paths listed in header §4.

At minimum verify that these planning facts remain materially true:

```text
shared common crate exists and is low-level
AST has implementation selector/field categories
semantic/core visibility still lacks Module/Package
native context still has old internal privilege or compatible live replacement
native descriptor visibility is narrower/lossy or compatible live replacement
ProjectIdentity + ModuleId remain canonical ownership types
Package remains a canonical package.ph module/object
```

If any is architecturally false, apply the header's VERIFY-FIRST/STOP rule before editing.

### T0.3 Build keyword collision census

Run repository-wide searches for the seven new words in identifier-bearing contexts.

Use broad and then context-specific searches, e.g.:

```sh
rg -n '\b(native|internal|private|protected|module|package|foreign)\b' \
  --glob '*.ph' --glob '*.rs' --glob '*.md'
```

Then inspect actual source/test collisions rather than treating comments/docs as code.

Record any existing Phalcom declaration/local/parameter/selector names using these words.

Special regression:

```sh
rg -n '\bpublic\b' --glob '*.ph'
```

Confirm `public` remains ordinary user spelling and is not being reserved by this work.

### T0.4 Build retired-attribute census

Search at minimum:

```sh
rg -n '@(native|private|protected|internal)\b' \
  phalcom-core phalcom-ast phalcom-semantic phalcom-lsp \
  phalcom-native-* docs/spec docs/specs tools
```

Classify results as:

```text
canonical source
current tests/fixtures
current normative spec
historical implementation record
comments/doc-only
```

Only current semantic authorities and canonical source require migration. Historical records remain historical unless they are incorrectly indexed as normative.

### T0.5 Build `_$` declaration/use census

Search source, compiler, VM, native registration, semantic tests, specs:

```sh
rg -n '_\$[A-Za-z_][A-Za-z0-9_]*' .
```

For every canonical declaration, create a local migration table with:

```text
owner
old selector
declaration location
implementation = Phalcom / native
known call/reference sites
scope of callers
classification = A / B / C / D
provisional new selector
provisional visibility
notes
```

Classification meanings come from header §13.2.

Do not rename anything yet.

### T0.6 Build general `__field` census

Search authored field/declaration positions, not every dunder mention:

```sh
rg -n '__[A-Za-z_][A-Za-z0-9_]*' \
  phalcom-core/core/universe \
  phalcom-ast phalcom-semantic phalcom-core
```

Separate:

```text
legacy general implementation field, e.g. __storage
standardized dunder protocol, e.g. __name__
compiler-generated/internal symbol
comment/string/debug text
```

For each legacy field, record:

```text
owner
reads
writes
cross-class callers
cross-module callers
cross-package callers
new _field spelling
required visibility
```

### T0.7 Census native registration assumptions

Search:

```sh
rg -n 'primitive_internal|primitive_static_internal|NativeVisibility|starts_with\("_\\$"\)|ImplementationSelector|ImplementationField'
```

Also find:

```text
native source census
native descriptor construction
native verifier
primitive installer
compiler internal dispatch bypasses
```

Record every place that independently derives visibility.

### T0.8 Establish baseline focused commands

Discover live test targets:

```sh
cargo test -p phalcom-ast -- --list
cargo test -p phalcom-modules -- --list
cargo test -p phalcom-semantic --test semantic -- --list
cargo test -p phalcom-core --test core -- --list
cargo test -p phalcom-lsp -- --list
```

Do not run all tests yet.

Select the smallest existing tests covering:

```text
lexer keywords
private/protected/internal semantic access
trait visibility
module/project identity
native source verification
runtime method visibility
Universe bootstrap
```

Run only enough to establish whether the starting baseline already has failures in those slices.

## Required evidence

Create/extend the P1 walkthrough with:

```text
live revision
dirty-tree note
keyword collision findings
legacy attribute counts/locations
_$ migration census
__field migration census
native visibility derivation sites
compiler-internal dispatch sites
baseline focused test status
```

## Gate G0 — census and scope are stable

Pass G0 only if:

```text
[ ] no material planning premise is disproven
[ ] all canonical _$ members are classified A/B/C/D
[ ] all general implementation fields are separated from dunder protocols
[ ] keyword collisions are understood
[ ] native/semantic/runtime visibility inference sites are enumerated
[ ] baseline focused failures, if any, are classified
```

## STOP / CONSULT

Stop if:

- a `_$` member's role cannot be classified between ordinary behavior and protocol/synthetic machinery;
- a `__...` spelling cannot be separated safely from dunder protocol semantics;
- a new keyword collides with a ratified public API in a way requiring language compatibility policy;
- live access/native architecture materially differs from the header.

---

# T1 — Introduce the shared modifier model and front-end lexical/AST foundations

## Objective

Create the single structural representation for visibility and implementation kind, add the seven keyword tokens, and make AST declarations capable of carrying normalized modifier metadata without yet switching all semantic/runtime consumers.

## Preconditions / authority

Gate:

```text
G0 passed
```

Reference:

```text
INV-01
INV-02
INV-03
INV-04
INV-05
INV-12
header §§6, 9, 14
CV-01
CV-02
```

## Expected source areas

```text
phalcom-common/src/
phalcom-common/src/lib.rs
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast parser helpers only as needed to keep compile green
phalcom-ast tests/snapshots
Cargo manifests if phalcom-common re-exports/modules change
```

## Required edits

### T1.1 Add canonical shared modifier enums

In `phalcom-common`, add the canonical definitions equivalent to:

```rust
enum Visibility {
    Public,
    Private,
    Protected,
    Module,
    Package,
    Internal,
}

enum ImplementationKind {
    Phalcom,
    Native,
    Foreign,
}
```

Requirements:

- derive only semantically safe traits;
- `Default` may map `Visibility -> Public`;
- `Default` may map `ImplementationKind -> Phalcom`;
- do **not** rely on `Ord`/numeric rank for accessibility semantics;
- export through a stable common module.

If existing crates have `MemberVisibility`, prepare explicit migration/re-export adapters, but do not create a second canonical enum.

### T1.2 Add seven token variants

Add keyword tokens for:

```text
native
internal
private
protected
module
package
foreign
```

Update:

```text
token display/debug spelling
lexer keyword lookup
semantic-token keyword grouping where AST crate owns it
lexer snapshots
exhaustive matches
```

Do not add `Public`.

### T1.3 Preserve contextual selector/property spelling

Update parser-facing token-name helpers so the new keyword tokens can participate in explicit property/selector-name contexts according to the existing contextual-keyword policy.

Do not add escaping syntax.

Do not loosen ordinary binding/declaration identifier positions merely to preserve a collision.

### T1.4 Add AST modifier storage

Introduce normalized modifier storage on the declaration/member nodes required by header §9.

At minimum behavior declarations and fields need:

```text
visibility
implementation kind where applicable
source range(s) for written modifiers
```

Native class/enum declarations need `ImplementationKind::Native` representation.

Do not infer these fields from attributes or names in AST helper methods.

### T1.5 Preserve temporary compile compatibility

Where downstream code cannot yet consume the new fields, allow a **temporary local compatibility adapter** that maps the new default state onto existing consumers.

Constraints:

- adapters must be marked transitional;
- they must not inspect `_$`/`__` to assign the new AST fields;
- they must be removed by T9.

## Focused tests

Add/adjust AST tests proving:

```text
native/internal/private/protected/module/package/foreign tokenize as keywords
public tokenizes as ordinary identifier
keyword source ranges are exact
new AST modifier enums round-trip/default correctly
explicit selector/property parsing can still consume keyword spelling where intended
```

BUILD commands:

```sh
cargo test -p phalcom-ast <new_keyword_or_modifier_filter>
cargo check -p phalcom-ast
cargo check -p phalcom-common
```

Use exact live filter names.

## Gate G1 — one structural front-end model exists

Pass only if:

```text
[ ] common modifier enums compile
[ ] lexer recognizes exactly seven new keywords
[ ] `public` is not a visibility keyword
[ ] AST nodes can carry structured visibility/implementation facts
[ ] no new prefix-derived AST semantics were added
```

## STOP / CONSULT

Stop if:

- adding common modifier types creates a crate cycle;
- the parser's keyword/property policy cannot support the seven words without a broader lexical redesign;
- a new keyword collision requires a source-compatibility language decision.

---

# T2 — Implement modifier grammar, legality validation, retired syntax diagnostics, and Universe-only metadata attributes

## Objective

Make the new modifier syntax executable at parse/validation level, enforce canonical ordering/target legality, add `@primitive` / `@intrinsic` as built-in metadata, and turn old modifier attributes into errors rather than semantic alternatives.

## Preconditions / authority

Gate:

```text
G1 passed
```

Reference:

```text
INV-01
INV-02
INV-03
INV-11
INV-12
INV-13
INV-14
INV-19
header §§6, 9–12, 14, 23
CV-02
CV-03
CV-13
CV-14
CV-16
```

## Expected source areas

```text
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-ast attribute parsing/registry
phalcom-core/src/compiler/attributes.rs
semantic/compiler validation paths with ModuleId available
diagnostic definitions
AST/compiler focused tests
```

## Required edits

### T2.1 Implement one declaration-prefix parser

Create/refactor one modifier parser equivalent to:

```text
attribute*
visibility?
implementation-kind?
member/declaration core
```

It must:

- default absent visibility to `Public`;
- default absent implementation kind to `Phalcom`;
- preserve written source ranges;
- reject multiple visibility modifiers;
- reject both `native` and `foreign`;
- enforce visibility-before-implementation order;
- issue a targeted noncanonical-order diagnostic for `native internal foo`, etc.

Do not duplicate modifier parsing separately for method/getter/setter/index unless existing grammar forces a shared helper to dispatch into those forms.

### T2.2 Enforce header target legality matrix

Implement the legality matrix from header §9.

Important fixed cases:

```text
native class        legal
native enum         legal
foreign class       rejected
native field        rejected
foreign field       rejected
module/package trait requirements rejected in C1
native/foreign trait abstract requirements rejected
foreign callable    bodyless scaffold only
```

Preserve existing private/protected/internal trait behavior where header requires it.

### T2.3 Enforce body rules

For callable behavior:

```text
ImplementationKind::Phalcom
    follows ordinary body/requirement rules

ImplementationKind::Native
    bodyless declaration required

ImplementationKind::Foreign
    bodyless declaration required
```

Do not retain historical native reference bodies.

If parser representation currently requires a body node, introduce an explicit bodyless declaration state rather than a dummy block.

### T2.4 Add `@primitive` and `@intrinsic`

Add genuine `BuiltinAttr` variants.

Rules:

```text
zero arguments
metadata only
callable behavioral targets only per header
```

The parser/attribute validator can enforce argument count and syntactic target.

Universe provenance must be enforced at the first phase that reliably knows canonical `ModuleId`.

Use:

```text
module_id.project.is_universe()
```

or exact live equivalent.

Do not compare strings/paths.

### T2.5 Retire `@native/@private/@protected/@internal`

At the earliest phase that can produce good diagnostics:

```text
@native      -> error: use `native`
@private     -> error: use `private`
@protected   -> error: use `protected`
@internal    -> error: use `internal`
```

They may remain lexically recognizable solely to issue targeted errors during this plan.

They must not fill normalized AST modifier fields.

### T2.6 Add diagnostic ranges

For:

```text
multiple visibility
native + foreign
wrong order
illegal target
body on native/foreign
Universe-only attribute violation
retired modifier attribute
```

use modifier/attribute ranges rather than whole-declaration fallback where infrastructure permits.

## Focused tests

Parser/AST cases:

```phalcom
foo()
private foo()
protected foo()
module foo()
package foo()
internal foo()

native foo() -> Int
private native foo() -> Int
module native foo() -> Int
package native foo() -> Int
internal native foo() -> Int

foreign foo() -> Int
native class C {}
native enum E {}
```

Negative cases:

```phalcom
public foo()
private protected foo()
module package foo()
native foreign foo()
native internal foo()
native foo() { ... }
foreign foo() { ... }
foreign class C {}
```

Attribute cases:

```text
@primitive in Universe accepted
@intrinsic in Universe accepted
same in user/fake-universe project rejected
arguments rejected
metadata does not mutate modifiers
```

BUILD commands:

```sh
cargo test -p phalcom-ast <modifier_parser_filter>
cargo test -p phalcom-core <attribute_validation_filter>
cargo check -p phalcom-ast
cargo check -p phalcom-core
```

Use a semantic/compiler fixture for canonical Universe identity; do not attempt to prove Universe-only legality with parser-only tests.

## Gate G2 — canonical source syntax is authoritative

Pass only if:

```text
[ ] every new modifier parses into structured AST facts
[ ] invalid combinations/order have targeted diagnostics
[ ] native/foreign callable bodies are rejected
[ ] primitive/intrinsic exist as built-in metadata
[ ] primitive/intrinsic outside canonical Universe are rejected
[ ] retired modifier attributes no longer set semantics
```

## STOP / CONSULT

Stop if:

- correct Universe-only validation requires moving project identity into an unrelated parser layer;
- current trait syntax makes the fixed legality matrix impossible without changing trait semantics;
- bodyless native/foreign declarations require a broad callable AST redesign beyond local declaration-body representation.

---

# T3 — Publish canonical module/package/project access provenance

## Objective

Give semantic and later runtime lowering one canonical, cheap source of defining Module, owning Package, and Project ownership for every visibility-bearing declaration.

## Preconditions / authority

Gate:

```text
G2 passed
```

Reference:

```text
INV-06
INV-07
INV-08
INV-09
INV-17
header §§7–8, 15, 18.3
CV-05
CV-06
CV-07
CV-20
```

## Expected source areas

```text
phalcom-modules/src/identity.rs
phalcom-modules/src/source.rs
phalcom-modules topology/interface/linker/session owner helpers
phalcom-modules tests
phalcom-semantic declaration/source ownership products
```

## Required edits

### T3.1 Verify ProjectIdentity sufficiency

Prove with existing code/tests that:

```text
all modules of one resolved Project share ProjectIdentity::Resolved(id)
Universe modules share ProjectIdentity::Universe
inline/standalone execution receives stable Synthetic identity according to owning execution domain
```

Specifically verify standalone Package behavior.

If modules of one standalone Package do **not** share one synthetic Project identity, trigger the header VERIFY-FIRST stop. Do not patch around this inside semantic access checks.

### T3.2 Add/publish canonical owning-package query

If not already available, add one canonical module-layer query/function:

```text
owning_package(ModuleId) -> Package ModuleId
```

or equivalent.

It must derive from logical module/package topology.

Rules:

```text
package.ph module owns itself
ordinary module resolves to nearest owning package
project root module/package resolves to root package
nested package has its own identity
```

No filesystem path-prefix test may appear in semantic/core access logic.

### T3.3 Add semantic declaration provenance

Ensure declaration/member formation can obtain and retain:

```text
defining ModuleId
owning package ModuleId
ProjectIdentity
```

Use canonical module-layer products.

Do not duplicate package discovery in `phalcom-semantic`.

### T3.4 Fingerprint/incremental provenance where necessary

If declaration/member surfaces are cached/fingerprinted, include provenance only where it semantically affects a product.

Do not cause broad invalidation by adding raw paths or unstable runtime handles to semantic fingerprints.

Use stable semantic IDs.

## Focused tests

Module tests must prove:

```text
two modules in same package -> same owning package
nested package -> different owning package
sibling package -> different owning package
same project -> same ProjectIdentity
different project -> different ProjectIdentity
Universe modules -> ProjectIdentity::Universe
standalone package ownership is stable/canonical
```

BUILD:

```sh
cargo test -p phalcom-modules <ownership_or_package_filter>
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
```

## Gate G3 — exact access provenance is available

Pass only if:

```text
[ ] defining Module identity is canonical
[ ] owning package identity is canonical
[ ] Project ownership is canonical
[ ] semantic declarations can carry/query all three
[ ] no consumer-local path/string package inference was added
```

## STOP / CONSULT

Stop if:

- standalone Package ownership disproves the `ProjectIdentity` model;
- package ownership requires changing the module topology contract;
- only filesystem heuristics can currently recover package identity.

---

# T4 — Converge semantic visibility and static access checking

## Objective

Replace semantic prefix/attribute visibility derivation with the shared `Visibility`, implement exact Module/Package/Internal relations, make fields default public, and align editor/static lookup with one access context.

## Preconditions / authority

Gate:

```text
G3 passed
```

Reference:

```text
INV-02
INV-04
INV-05
INV-06
INV-07
INV-08
INV-18
INV-20
header §§7, 9, 15
CV-04
CV-05
CV-06
CV-07
CV-08
```

## Expected source areas

```text
phalcom-semantic/src/surface.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/editor.rs
semantic access-context helpers
source index/fingerprint metadata
semantic tests
```

## Required edits

### T4.1 Replace semantic `MemberVisibility`

Migrate semantic surfaces to the shared `Visibility`.

Preferred endpoint:

```text
no separate semantic MemberVisibility enum
```

A temporary type alias/re-export is acceptable while migrating code, but no independent semantics remain.

Update all exhaustive matches for `Module` and `Package`.

### T4.2 Delete semantic name/attribute inference

Remove logic equivalent to:

```rust
name.starts_with("_$")
attributes.contains("private")
attributes.contains("protected")
attributes.contains("internal")
is_field => Private
```

Visibility comes from normalized declaration modifiers only.

Retired attributes may still be diagnosed, but never influence surface visibility.

### T4.3 Make field default public

Change semantic field publication so:

```text
no modifier -> Public
```

Then update repository-owned fixtures/source that intentionally relied on private-by-default behavior, initially only where required to keep focused tests/source correct.

Full Universe migration occurs in T8.

### T4.4 Build one semantic AccessContext

Extend/refactor semantic access context to carry canonical:

```text
lexical declaring/access class
caller ModuleId
caller owning package ModuleId
caller ProjectIdentity
```

Implement one helper:

```text
is_member_accessible(visibility, declaration_origin, access_context, hierarchy)
```

or equivalent.

Use it from:

```text
checker member lookup
editor/completion lookup
trait/member surface lookup where applicable
callable reference formation
```

Do not repeat match logic independently in editor and checker.

### T4.5 Implement exact new access relations

Implement:

```text
Module:
    caller.module == declaration.module

Package:
    caller.package == declaration.package

Internal:
    caller.project == declaration.project
```

Private/protected preserve existing lexical/subclass semantics.

### T4.6 Preserve trait visibility boundary

Migrate existing trait public/private/protected/internal semantics to shared `Visibility`.

Reject new `module`/`package` on trait requirements/defaults per header.

Do not invent enum-order-based visibility compatibility.

If conformance witness compatibility currently compares visibilities, write an explicit semantic relation for supported cases.

### T4.7 Preserve export orthogonality

Do not filter top-level declarations by member `Visibility`.

Exports remain the namespace authority.

## Focused tests

Add semantic fixtures for:

### Module

```text
same class/same module allowed
other class/same module allowed
sibling module/same package denied
```

### Package

```text
same module allowed
sibling module/same package allowed
nested child package denied
parent package denied
sibling package denied
```

### Internal

```text
different package/same project allowed
other project denied
Universe member accessible from another Universe package/module
ordinary project cannot access Universe internal
Universe cannot access ordinary project internal merely because it is Universe
```

### Private/protected regression

```text
private same lexical class allowed
private subclass denied where current semantics say so
protected eligible subclass allowed
unrelated class denied
nested closure retains lexical authority
```

### Field default

```text
unmodified field publishes Public
explicit private remains private
module/package/internal field scopes work
```

BUILD:

```sh
cargo test -p phalcom-semantic --test semantic <new_visibility_filter>
cargo test -p phalcom-semantic --test semantic <existing_trait_visibility_filter>
cargo check -p phalcom-semantic
```

## Gate G4 — static access semantics are canonical

Pass only if:

```text
[ ] semantic visibility comes only from normalized modifiers
[ ] module/package/internal use canonical identities
[ ] field default is Public
[ ] private/protected regressions pass
[ ] editor/checker share equivalent access authority
[ ] trait boundary follows header
```

Update walkthrough with the final semantic access relation and any migrated fixtures.

## STOP / CONSULT

Stop if:

- editor/checker require materially different access semantics;
- trait witness visibility cannot be preserved without a new conformance design;
- package/internal semantics need textual/path inference.

---

# T5 — Replace runtime internal privilege with structured declaration-derived access authority

## Objective

Make runtime dispatch enforce the same visibility relation as semantics on every invocation path, replacing `caller_internal`/blanket native privilege with canonical declaration origin.

## Preconditions / authority

Gate:

```text
G4 passed
```

Reference:

```text
INV-06
INV-07
INV-08
INV-09
INV-10
INV-17
INV-20
header §8
CV-05–CV-10
CV-18
```

## Expected source areas

```text
phalcom-core/src/method/mod.rs
phalcom-core/src/method/object.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/modules/
compiler method-install/lowering code
reflection/dynamic invocation helpers
bound/method-reference invocation paths
core runtime tests
```

## Required edits

### T5.1 Migrate runtime visibility type

Use the shared `Visibility` in `MethodObject` and runtime dispatch.

Remove the independent core `MemberVisibility` enum or reduce it to a temporary re-export that disappears by T9.

### T5.2 Introduce compact RuntimeAccessOrigin

Implement the header §8 direction.

It must provide O(1)-comparable:

```text
defining runtime module identity
owning runtime package identity
ProjectIdentity/internal owner
```

Retain existing `access_owner: Option<ClassId>` for private/protected lexical authority unless a smaller equivalent live structure already exists.

Do not allocate one origin per send.

### T5.3 Populate origin when methods are installed

For bytecode/source methods, origin comes from the source declaration/module being compiled/materialized.

For native methods, origin comes from the canonical source declaration/native descriptor owner.

Do not infer it from caller frames.

### T5.4 Replace caller authority transport

Refactor:

```text
ArgumentView.caller_internal
caller_authority() -> (Option<ClassId>, bool)
NativeMethodContext.internal
```

into structured authority.

Ensure caller authority is captured before crossing the native gateway where necessary.

### T5.5 Centralize runtime authorization

Create/use one authorization function equivalent to:

```text
authorize_member(
    visibility,
    declaration access_owner/origin,
    caller access_owner/origin,
    hierarchy
)
```

All runtime invocation paths must converge on it.

### T5.6 Cover direct and cached dispatch

Ensure inline/monomorphic cache hits do not bypass the access decision.

If cache entries currently assume visibility invariance, either:

- cache an already-authorized result only where caller authority is part of the key; or
- keep authorization outside the cached lookup.

Do not add caller-specific global cache invalidation as a shortcut.

### T5.7 Cover method references / bound calls / reflection / dynamic sends

Audit:

```text
&object.method(...)
bound Method invocation
perform/dynamic send
reflection invocation
family/method-object invocation
super/class-side dispatch where applicable
```

They must enforce the same relation.

### T5.8 Narrow native context

When entering a native method:

```text
push declaration-derived NativeMethodContext
```

not:

```text
internal = true
```

A native method in Project A must not acquire authority over Project B internals.

### T5.9 Keep compiler synthetic authority separate

Audit `compiler_internal_dispatch_depth` and related bypasses.

Classify each use from T0:

```text
ordinary member access -> remove bypass/use access origin
real compiler protocol -> keep narrowly scoped
obsolete legacy _$ support -> delete
```

Do not rename that bypass to "internal" and reuse it for ordinary project visibility.

## Focused tests

Core/runtime tests must include:

```text
module direct send allow/deny
package direct send allow/deny
internal same-project allow / cross-project deny
private/protected regression
cache hit preserves denial
method reference/bound call preserves denial
dynamic/reflection invocation preserves denial
nested closure caller authority remains correct
class-side method authority remains correct
```

Native authority cases:

```text
native Project A -> A internal allowed
native Project A -> B internal denied
package-native -> same package allowed
package-native -> sibling package denied
private native -> owner lexical authority
native does not inherit unrelated caller private/internal authority
```

BUILD:

```sh
cargo test -p phalcom-core --test core <member_visibility_filter>
cargo test -p phalcom-core --test core <native_access_filter>
cargo check -p phalcom-core
```

## Gate G5 — runtime invocation parity

Pass only if:

```text
[ ] no runtime ordinary Internal authorization uses a boolean privilege
[ ] MethodObject/native declarations carry canonical access origin
[ ] direct/cached/bound/dynamic/reflection paths agree
[ ] native methods use declaration-derived authority
[ ] compiler synthetic authority is separately classified
```

Update walkthrough with the final runtime access-origin representation and invocation path map.

## STOP / CONSULT

Stop if:

- correct access enforcement requires a per-send module/package lookup by source path;
- caches require a material redesign not anticipated by header;
- native ABI must change materially rather than only authority metadata;
- compiler protocol bypass classification remains ambiguous.

---

# T6 — Converge native metadata, source indexing, descriptor verification, and installation

## Objective

Make native source declarations consume the new keyword/visibility model exactly, eliminate lossy visibility mappings and `_$` assumptions, and preserve canonical structural source/native binding.

## Preconditions / authority

Gate:

```text
G5 passed
```

Reference:

```text
INV-02
INV-03
INV-04
INV-10
INV-11
INV-16
INV-17
header §§10, 15–16
CV-11
CV-12
```

## Expected source areas

```text
phalcom-native-meta/
phalcom-native-surface/
phalcom-native-macros/
phalcom-core/src/native/source.rs
phalcom-core/src/native/descriptor.rs
phalcom-core/src/native/verify.rs
phalcom-core/src/native/install.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/primitive/
native/core-table generators
native tests
```

## Required edits

### T6.1 Reuse shared Visibility in native metadata

Make native metadata carry exact shared `Visibility`.

Delete/collapse `NativeVisibility` if possible.

If public API compatibility temporarily requires a wrapper, its conversion must be total and one-to-one for all six visibility cases.

No conversion may map:

```text
Private | Protected | Module | Package | Internal
    -> Internal
```

as a final state.

### T6.2 Carry implementation kind structurally

Native source census/index must read:

```text
ImplementationKind::Native
```

from AST.

Delete `has_native_attr` as semantic authority.

Native class/enum/member discovery must not inspect `@native`.

### T6.3 Delete `_$` visibility inference in native source census

Remove logic equivalent to:

```text
name.starts_with("_$")
@internal || prefix
```

Visibility comes from AST.

Selector identity is the ordinary selector.

### T6.4 Generalize descriptor owner identity where practical

Retain existing `UniverseKey` bootstrap mapping for primordial declarations.

Add/use declaration-qualified identity in the general native descriptor/source verifier path:

```text
DeclarationId
DispatchSide
Selector
```

or live equivalent.

Do not rewrite the entire Universe bootstrap registry if a small adapter can map `UniverseKey -> canonical declaration`.

### T6.5 Verify exact source/native parity

Native verifier must compare:

```text
owner
side
selector
member kind
signature shape
parameter types
return type
visibility
native implementation kind
```

where these facts already exist.

A visibility mismatch is a bootstrap/link error.

### T6.6 Remove prefix assertions/macros as semantic authority

Refactor macros such as:

```text
primitive_internal!
primitive_static_internal!
```

so visibility is descriptor data, not encoded by macro choice/prefix assertion.

Possible endpoint:

```text
one registration macro/function + explicit visibility
```

or thin convenience wrappers that merely pass an explicit `Visibility` and never inspect selector spelling.

By P1 completion no registration path may require selector prefix `_$`.

### T6.7 Populate runtime access origin from descriptor source identity

Native-installed `MethodObject`s must receive the origin established in T5.

Do not set origin from whichever module called the installer at runtime.

## Focused tests

Native tests must prove:

```text
Public native descriptor matches Public source
Private/Protected/Module/Package/Internal exact values survive metadata
mismatch fails
ordinary selector binds without _$
native class/enum keyword is recognized by source census
@native alone is rejected/not treated as native
source signature mismatch remains caught
```

BUILD:

```sh
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-surface
cargo test -p phalcom-core --test core <native_source_or_descriptor_filter>
cargo check -p phalcom-core
```

Add macro-crate tests if exact crate has tests.

## Gate G6 — source/native parity is exact

Pass only if:

```text
[ ] native metadata represents all six visibility cases
[ ] source census reads ImplementationKind, not @native
[ ] selector prefixes do not determine native visibility
[ ] structural native verification remains authoritative
[ ] runtime native methods receive source declaration origin
[ ] UniverseKey remains only where bootstrap identity needs it
```

## STOP / CONSULT

Stop if:

- shared common visibility creates a native-meta dependency cycle;
- current descriptor identity cannot be bridged to declaration identity without replacing bootstrap architecture;
- exact visibility transport requires a new stable ABI/serialization design outside this plan.

---

# T7 — Add the bounded `foreign` semantic and lowering scaffold

## Objective

Make `foreign` a real, distinct implementation kind through AST/semantic/tooling/lowering without pretending that C1 has implemented an FFI.

## Preconditions / authority

Gate:

```text
G6 passed
```

Reference:

```text
INV-03
INV-12
header §§11, 23
CV-14
```

## Expected source areas

```text
AST semantic conversion paths
phalcom-semantic member/source metadata
phalcom-core compiler/lowering declaration handling
source index / reflection metadata if applicable
focused tests
```

## Required edits

### T7.1 Preserve Foreign through semantic products

Ensure a bodyless foreign callable carries:

```text
ImplementationKind::Foreign
```

through semantic declaration/member/source index products.

Do not treat it as:

```text
Native
abstract trait requirement
missing body
```

### T7.2 Reject unsupported targets

Confirm target matrix enforcement for:

```text
foreign class/enum/field/trait requirement -> targeted error
```

### T7.3 Add explicit lowering/link failure

When compilation reaches a legal foreign callable without a supported binding backend, produce:

```text
foreign.binding_unimplemented
```

or exact approved diagnostic.

Do not:

- install a stub returning nil;
- defer to runtime method missing;
- route through native primitive registry;
- select a C/Rust ABI.

### T7.4 Keep reflection/source presentation distinct

Where implementation kind is surfaced, show `foreign`.

## Focused tests

```text
foreign callable parses
foreign callable is semantically distinguishable from native
foreign callable body rejected
native foreign conflict rejected
foreign class rejected
lowering/unbound compile emits explicit foreign diagnostic
```

BUILD:

```sh
cargo test -p phalcom-ast <foreign_filter>
cargo test -p phalcom-semantic --test semantic <foreign_filter>
cargo test -p phalcom-core --test core <foreign_filter>
```

## Gate G7 — foreign is a real but bounded category

Pass only if:

```text
[ ] Foreign survives front-end/semantic representation
[ ] Foreign is never silently Native
[ ] unsupported binding fails explicitly before runtime
[ ] no ABI/ownership design was invented
```

## STOP / CONSULT

Stop immediately if implementation pressure requires choosing:

```text
C ABI
Rust ABI
pointer ownership
marshaling
dynamic library naming/loading
foreign type representation
```

Those are not P1 work.

---

# T8 — Classify and migrate canonical Universe source, fields, selectors, and native registration

## Objective

Transform Universe into canonical new Phalcom syntax while preserving bootstrap behavior and using actual caller scope to choose visibility. This is the main source migration and must consume the T0 census rather than perform blind prefix removal.

## Preconditions / authority

Gates:

```text
G6 passed
G7 passed
```

Reference:

```text
INV-01–INV-19
header §§13, 24–25
CV-12
CV-13
CV-15
CV-16
CV-19
```

## Expected source areas

```text
phalcom-core/core/universe/src/**/*.ph
corresponding native primitive/descriptor registrations
Universe source/bootstrap fixtures
current normative source-facing examples
```

## Required edits

### T8.1 Migrate declaration modifiers

Canonical Universe source replacements:

```text
@native      -> native
@private     -> private
@protected   -> protected
@internal    -> internal
```

Do this only after T6 ensures descriptors consume the new model.

Examples:

```phalcom
@native
class String ...
```

becomes:

```phalcom
native class String ...
```

and:

```phalcom
@internal
@native
_$foo(...)
```

becomes a census-selected ordinary selector form such as:

```phalcom
internal native foo(...)
```

### T8.2 Migrate `_$` class A — native runtime floor

For each class-A census row:

1. inspect all source/compiler/runtime callers;
2. choose the narrowest correct visibility:
   - `private`
   - `module`
   - `package`
   - `internal`
3. choose a semantic ordinary selector name;
4. rename declaration and every canonical call/reference;
5. rename/update native descriptor registration selector identity;
6. preserve signature and behavior.

Do not automatically choose `internal`.

Prefer names that describe operation semantics rather than implementation status.

### T8.3 Migrate `_$` class B — Phalcom source helpers

For each class-B row:

```text
remove implementation-selector spelling
do not add native
choose actual visibility from caller set
update callers/references
```

Verify no Rust/native registration exists for it.

### T8.4 Resolve class C protocol rows

For every class-C row from T0:

- compare against current standardized protocol rules;
- if it already belongs to an existing reserved dunder/protocol mechanism, migrate through that existing mechanism;
- if it is compiler-recognized but not yet standardized and the correct replacement is obvious/direct metadata, use the existing typed compiler/runtime path;
- otherwise STOP AND CONSULT.

Do not invent new dunder protocol semantics in this task.

### T8.5 Resolve class D synthetic rows

Remove authored selector dependence where the operation is solely a compiler/VM seam.

Prefer:

```text
direct bytecode
typed compiler metadata
canonical callable ID not exposed as user helper
existing VM internal function
```

Use the narrowest mechanical change consistent with existing architecture.

### T8.6 Migrate general implementation fields

For each T0 field row:

```text
__foo -> _foo
```

plus explicit visibility from actual use.

Examples:

```phalcom
private _foo
module _foo
package _foo
internal _foo
```

Update every source/compiler reflection reference.

Do not touch standardized `__foo__` dunders.

### T8.7 Make all intended field privacy explicit

Audit ordinary Universe `_field`s that relied on the old implicit-private field rule.

Add explicit:

```phalcom
private
```

or other correct visibility.

The source migration must not accidentally widen storage API.

### T8.8 Keep `is_universe` usages that remain legitimate

Do not delete structural checks used for:

```text
bootstrap
kernel identities
Universe-only @primitive/@intrinsic validation
primordial mappings
prelude construction
other legitimate canonical-Universe decisions
```

Only remove Universe checks that existed solely to authorize old ordinary `Internal` members.

### T8.9 Introduce `@primitive` / `@intrinsic` only from evidence

Do not blanket annotate every native member.

Use the attributes only where one of these holds:

```text
existing authoritative metadata already identifies the operation as primitive/intrinsic
a current normative spec explicitly classifies it
the implementation architecture already has a stable primitive/intrinsic identity
```

If there is no authoritative classification, leave the attributes absent.

Remember: the attributes are metadata-only; runtime behavior must not depend on adding them.

### T8.10 Update native registration names in lockstep

When a native `_$foo` selector becomes `foo` or a semantic replacement, update:

```text
native descriptor selector
registration macro/function
source/native verifier expectations
tests
reflection expected selector strings
```

Do not leave old selector aliases unless required by a separately ratified compatibility rule.

## Recommended migration staging

Keep this task reviewable by migrating subsystem clusters, for example:

```text
Object/Class/reflection protocols
scalar/String/Number
collections List/Map/Set
Option/Result/ADTs
concurrency Fiber/Future/System
time/Instant/Duration
remaining Universe utility classes
```

After each cluster, run only its source/native/bootstrap discriminator plus the shared native verifier tests.

## Focused tests

At minimum certify representative migrations from:

```text
String
List
Fiber/System
one ADT native enum
one source-only helper cluster
one migrated field owner
one preserved dunder protocol
```

Test:

```text
selector reflection uses new ordinary name
source/native descriptor matches
internal/package/module access still succeeds where intended
external user access fails where intended
Universe bootstrap completes
```

BUILD per cluster:

```sh
cargo test -p phalcom-core --test core <cluster_filter>
```

Then at end of T8:

```sh
cargo test -p phalcom-core --test core <universe_bootstrap_filter>
cargo test -p phalcom-semantic --test semantic <universe_surface_filter>
```

## Gate G8 — Universe speaks canonical syntax

Pass only if:

```text
[ ] canonical Universe @native/@private/@protected/@internal uses are migrated
[ ] every canonical _$ declaration is resolved from census
[ ] every general implementation field is migrated
[ ] intended field privacy is explicit
[ ] dunder protocols remain intact
[ ] native registrations use new selector identities
[ ] is_universe remains for legitimate provenance uses
[ ] no primitive/intrinsic annotation was guessed
```

Update walkthrough with final migration tables and any protocol/synthetic exceptions.

## STOP / CONSULT

Stop if:

- any class-C/class-D selector lacks an architecturally clear replacement;
- a native selector rename breaks external ABI/metadata compatibility requiring policy;
- field migration exposes a representation rule that cannot be expressed by the new visibility model;
- dunder semantics would need changing.

---

# T9 — Remove legacy inference, token/AST namespaces, and compatibility paths

## Objective

Delete the old semantic authorities only after the new vertical and Universe source are functional, leaving no hidden alternate visibility/native model.

## Preconditions / authority

Gate:

```text
G8 passed
```

Reference:

```text
INV-02
INV-03
INV-15
INV-19
header §§13, 19, 29
CV-16
```

## Expected source areas

```text
phalcom-ast token/lexer/AST/parser legacy categories
semantic starts_with / attribute scans
core compiler visibility scans
native source scans
primitive registration prefix assertions
retired BuiltinAttr semantic handling
compiler_internal legacy bypasses classified in T5/T8
tests/snapshots
```

## Required edits

### T9.1 Remove authored `ImplementationSelectorIdentifier`

Delete the dedicated `_$` authored selector token/AST path once canonical source/tests no longer need it.

Desired behavior for new source using `_$foo`:

- targeted retired-syntax diagnostic if lexer/parser can provide it cheaply;
- otherwise an ordinary syntax error with no Internal semantics.

It must never parse as an Internal selector.

### T9.2 Remove general implementation-field token/AST semantics

Delete general `ImplementationFieldIdentifier` / `FieldKind::Implementation` behavior if no standardized dunder protocol depends on that exact representation.

Preserve dedicated dunder protocol lexical support separately.

### T9.3 Delete prefix inference everywhere

Repository search must no longer find semantic logic like:

```rust
starts_with("_$")
starts_with("__") // when used to infer ordinary implementation field visibility
```

except targeted retired-syntax diagnostics/tests.

### T9.4 Delete retired attribute semantics

`@native/@private/@protected/@internal` may remain recognized solely to produce migration diagnostics.

They must not:

```text
populate Visibility
populate ImplementationKind
change lowering
change native source census
```

If the architecture can issue the targeted error without retaining BuiltinAttr variants, delete them.

### T9.5 Remove old internal registration assumptions

Delete/normalize:

```text
primitive_internal! prefix assertions
special public exceptions for _$ selectors
native visibility conversions based on internal selector namespace
```

### T9.6 Remove ordinary Internal privilege boolean transport

Search for:

```text
caller_internal
internal: bool
current_has_internal_privilege
is_privileged_universe_module
```

Classify remaining hits.

Allowed residuals:

```text
legitimate Universe/bootstrap provenance
narrow compiler protocol privilege explicitly justified in walkthrough
historical docs/tests
```

Not allowed:

```text
ordinary member Internal authorization
native blanket privilege
```

### T9.7 Clean compatibility adapters

Remove T1/T2 transitional aliases/adapters that allowed old AST/semantic representations to coexist.

## Focused tests

Add negative regression fixtures:

```text
_$foo does not grant access
@internal foo does not create Internal semantics
@native foo does not create Native implementation kind
legacy __storage is rejected/not treated as privileged field
standardized __name__ protocol still parses/executes
```

Run:

```sh
cargo test -p phalcom-ast <legacy_syntax_filter>
cargo test -p phalcom-semantic --test semantic <legacy_syntax_filter>
cargo test -p phalcom-core --test core <legacy_syntax_filter>
```

Run repository code census:

```sh
rg -n 'starts_with\("_\\$"\)|ImplementationSelectorIdentifier|ImplementationFieldIdentifier'
rg -n '@(native|private|protected|internal)\b' phalcom-core/core/universe/src
rg -n '_\$[A-Za-z_]' phalcom-core/core/universe/src
```

Expected results must be zero or explicitly documented targeted-diagnostic/test references.

## Gate G9 — one authority remains

Pass only if:

```text
[ ] no prefix derives visibility/implementation
[ ] no retired attribute derives visibility/implementation
[ ] canonical Universe has no authored _$ implementation selector
[ ] canonical Universe has no general __implementation field
[ ] ordinary runtime Internal does not use privilege boolean
[ ] dunder protocol support remains
```

## STOP / CONSULT

Stop if deleting a legacy path breaks a still-authoritative compiler protocol that was missed by the census.

---

# T10 — Converge LSP, reflection, syntax tooling, and normative documentation

## Objective

Make every user-facing/tooling surface present the same modifier and access semantics, and record the new language authority without rewriting historical records.

## Preconditions / authority

Gate:

```text
G9 passed
```

Reference:

```text
INV-01–INV-20
header §§17, 19, 23, 25
CV-17
```

## Expected source areas

```text
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp hover/completion/navigation/source-index consumers
reflection descriptor/string presentation
tools/phalcom.tmLanguage*.json
current docs/spec + docs/specs
decision index / new decision record
LANG006 docs
```

## Required edits

### T10.1 Semantic token support

Classify:

```text
native
internal
private
protected
module
package
foreign
```

as language keywords.

Do not classify `public` as visibility keyword.

`@primitive/@intrinsic` use ordinary built-in attribute presentation.

### T10.2 Completion/access filtering

Completion must call/consume the canonical semantic access relation.

Do not copy a local LSP match over visibility cases.

Verify:

```text
module hidden across module
package hidden across package
internal hidden across project
private/protected regressions
```

### T10.3 Hover/source presentation

Where member metadata is shown, render semantic facts from structured data.

Examples:

```text
visibility: package
implementation: native
```

Do not expose `_$` or old attribute spelling.

### T10.4 Reflection

Where reflection exposes member/declaration metadata, support:

```text
Public
Private
Protected
Module
Package
Internal
Phalcom
Native
Foreign
```

If reflection currently does not expose implementation kind publicly, do not invent a new public API solely for C1; ensure any existing internal/debug descriptor remains correct.

### T10.5 Syntax grammar/tooling

Update TextMate or generated syntax descriptions for the seven keywords and two built-in attributes.

Remove canonical highlighting rules that promote `_$`/general `__field` as active implementation namespaces.

Preserve dunder highlighting/protocol support.

### T10.6 Superseding decision record

Create an accepted decision/amendment that explicitly supersedes the relevant TDR-0076 clauses.

It must state:

```text
no public keyword; Public default
new visibility keywords
native/foreign implementation kinds
primitive/intrinsic metadata-only Universe-only attributes
_$ authored implementation selectors retired
general __implementationField retired
dunder protocols unchanged
ordinary internal uses Project identity
is_universe retained independently
```

Update decision indexes as required.

### T10.7 Update normative specs

Update current authoritative specs for:

```text
lexical keywords
member declarations/modifier order
fields/default visibility
access control
native declarations
foreign scaffold status
attributes
VM access authority
Universe bootstrap relationship
reflection/tooling where normative
```

Do not mass-edit historical plans/walkthroughs merely because they mention old syntax.

If a historical document is still linked as current authority, update the index/authority pointer rather than rewriting history.

## Focused tests

LSP:

```sh
cargo test -p phalcom-lsp <semantic_token_filter>
cargo test -p phalcom-lsp <completion_or_navigation_filter>
```

Reflection/core if applicable:

```sh
cargo test -p phalcom-core --test core <reflection_visibility_filter>
```

Docs/tooling validation:

```text
run repository's existing spec/decision/syntax validation commands if present
```

## Gate G10 — presentation/spec parity

Pass only if:

```text
[ ] LSP understands all seven keywords
[ ] completion uses canonical access relation
[ ] `public` remains non-keyword
[ ] reflection/debug metadata is structurally correct
[ ] syntax tooling no longer treats _$ as canonical implementation namespace
[ ] dunder tooling remains
[ ] new accepted authority supersedes old implementation-namespace rules
[ ] current normative specs describe C1 model
```

## STOP / CONSULT

Stop if current authoritative specs conflict materially with the checkpoint/header on a fixed rule.

---

# T11 — Stabilize, certify, record implementation state, and close P1

## Objective

Verify the complete vertical with sparse but sufficient cross-stack evidence, ensure no legacy semantic authority remains, classify any unrelated baseline failures, and produce durable implementation records.

## Preconditions / authority

Gate:

```text
G10 passed
```

Reference:

```text
all INV-*
all CV-*
header §§21–30
```

## Required actions

### T11.1 Compile affected crates first

Run:

```sh
cargo check -p phalcom-common
cargo check -p phalcom-ast
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-native-meta
cargo check -p phalcom-native-surface
cargo check -p phalcom-core
cargo check -p phalcom-lsp
```

Adapt exact package names if a crate was renamed.

Do not proceed to broader tests while an affected crate does not compile.

### T11.2 Run focused owner suites

Run the focused tests created/identified through T1–T10.

Minimum evidence categories:

```text
CV-01 lexer keywords/default
CV-02 AST normalization
CV-03 modifier conflict/order
CV-04 field default/migration
CV-05 module access
CV-06 package access
CV-07 internal access
CV-08 private/protected regression
CV-09 dynamic invocation parity
CV-10 native authority
CV-11 native descriptor parity
CV-12 Universe migration
CV-13 primitive/intrinsic legality
CV-14 foreign scaffold
CV-15 dunder preservation
CV-16 legacy rejection
CV-17 tooling parity
CV-18 cache parity
CV-19 source identity
CV-20 standalone ownership
```

The walkthrough should map each CV ID to a concrete command/test name and result.

### T11.3 Run package-level stabilization suites

If focused evidence is green, run the affected package suites once:

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-modules
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-surface
cargo test -p phalcom-core --test core
cargo test -p phalcom-lsp
```

If any suite is known to contain unrelated baseline failures:

- capture exact failure;
- reproduce against the pre-P1 baseline when feasible;
- classify;
- do not alter unrelated architecture to obtain green.

### T11.4 Perform canonical old-syntax census

Run targeted searches.

Canonical source should have no old modifier syntax:

```sh
rg -n '@(native|private|protected|internal)\b' phalcom-core/core/universe/src
rg -n '_\$[A-Za-z_][A-Za-z0-9_]*' phalcom-core/core/universe/src
```

Search code for semantic inference:

```sh
rg -n 'starts_with\("_\\$"\)|ImplementationSelectorIdentifier|ImplementationFieldIdentifier' \
  phalcom-ast phalcom-semantic phalcom-core phalcom-native-*
```

Search privilege transport:

```sh
rg -n 'caller_internal|internal:\s*bool|current_has_internal_privilege' \
  phalcom-core phalcom-semantic
```

Every remaining result must be one of:

```text
targeted retired-syntax diagnostic
historical compatibility test
legitimate non-access use explicitly documented
historical docs
```

No ordinary access authority may remain.

### T11.5 Verify new syntax census

Confirm canonical source actually exercises:

```sh
rg -n '\b(private|protected|module|package|internal)\b' phalcom-core/core/universe/src
rg -n '\bnative\b' phalcom-core/core/universe/src
```

Confirm `@primitive/@intrinsic` only appear where deliberately added and Universe-only.

### T11.6 Run final workspace-level compile

Run:

```sh
cargo check --workspace
```

This is the preferred broad structural certification.

Run:

```sh
cargo test --workspace
```

only if required by the repository's active certification workflow or if the implementation changed shared surfaces broadly enough that the focused/package suites cannot discriminate downstream breakage. If run, classify unrelated failures; do not chase them reflexively.

### T11.7 Review diff for architectural regressions

Inspect:

```sh
git diff --stat
git diff
```

Specifically audit for forbidden patterns:

```text
visibility enum duplicated in multiple canonical crates
string/path-based package access
is_universe used as ordinary Internal authorization
new prefix-derived visibility
native => blanket privilege
primitive/intrinsic changing implementation kind
foreign routed through native
dunder protocol edits outside necessary lexical separation
public keyword introduction
```

### T11.8 Update checkpoint

Update `LANG006.C1-CHECKPOINT.md` with actual completion state.

If all acceptance criteria are met:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
latest_completed_plan: LANG006.C1.P1
```

Use the repository's actual lifecycle vocabulary if it differs.

Record any classified baseline blockers exactly.

### T11.9 Produce walkthrough

Create:

```text
LANG006.C1.P1-walkthrough.md
```

It must include:

```text
live start/end revisions
implementation summary
actual shared types
actual AST grammar
actual access-origin representation
module/package/internal relation
native descriptor changes
Universe migration tables
protocol/synthetic exceptions
primitive/intrinsic use sites
foreign scaffold behavior
diagnostic changes
CV-01..CV-20 evidence matrix
test commands/results
baseline failure classifications
residual technical debt
```

### T11.10 Produce handoff

Create:

```text
LANG006.C1.P1-handoff.md
```

If C1 is fully closed, state that no additional C1 implementation plan is required.

If bounded residual work remains because of an approved consultation decision, identify the exact next plan rather than hiding it under "cleanup".

## Gate G11 — P1 complete

P1 is complete only when every header §29 success criterion is satisfied or a formally approved amendment changes it.

Final checklist:

```text
[ ] seven new keywords canonical
[ ] no public keyword
[ ] shared Visibility/ImplementationKind canonical
[ ] field default Public
[ ] private/protected correct
[ ] module exact
[ ] package exact
[ ] internal exact
[ ] native declaration-derived access
[ ] native metadata exact
[ ] foreign distinct/bounded
[ ] primitive/intrinsic Universe-only metadata
[ ] is_universe retained independently
[ ] canonical _$ gone
[ ] general implementation fields gone
[ ] dunders preserved
[ ] old modifier attrs non-semantic
[ ] direct/cached/dynamic/native invocation parity
[ ] LSP/tooling/spec parity
[ ] old semantic inference census clean
[ ] walkthrough produced
[ ] handoff produced
[ ] checkpoint updated
```

## Final STOP / CONSULT

Do not declare completion if:

- an invariant is knowingly violated;
- old and new visibility authorities coexist;
- a native access path still has blanket internal privilege;
- package/internal semantics differ between semantic analysis and runtime;
- dunder protocols were changed without explicit authority;
- tests pass only because an access check was weakened;
- broad failures are unclassified.

---

# Implementation sequence summary

The executor should experience the work as this dependency chain:

```text
T0 census
    ↓
T1 shared data model
    ↓
T2 grammar + metadata attributes
    ↓
T3 canonical provenance
    ↓
T4 semantic access
    ↓
T5 runtime access
    ↓
T6 native parity
    ↓
T7 foreign scaffold
    ↓
T8 Universe migration
    ↓
T9 legacy deletion
    ↓
T10 tooling/specs
    ↓
T11 certification
```

Do not move the Universe source migration ahead of semantic/runtime/native convergence merely to make the source look finished earlier.

Do not delete old parser/token paths before canonical source and descriptors have migrated.

Do not implement `foreign` as an FFI.

Do not use `is_universe` to recover old internal privilege.

Do not infer behavior from `_$/__` spelling.

At every point, the target is one language:

```text
ordinary names
+ explicit restrictive visibility
+ explicit native/foreign implementation kind
+ metadata-only built-in attributes
+ canonical module/package/project ownership
```
