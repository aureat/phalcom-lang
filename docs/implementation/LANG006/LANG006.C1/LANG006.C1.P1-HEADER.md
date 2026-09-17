---
id: LANG006.C1.P1
category: LANG
program: LANG006
checkpoint: LANG006.C1
kind: implementation-plan-header
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG006.C1-CHECKPOINT.md
follows: null
supersedes: null
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
planning_commit: "feat(lang005): continue P5 iterator surface"
body_artifact: LANG006.C1.P1-BODY.md
---

# LANG006.C1.P1 — Declaration Modifiers, Access Domains, Native/Foreign Boundaries, and Universe Normalization

> **P1 shared header / executor authority.**  
> This file is the complete shared execution context for `LANG006.C1.P1`. The separate `LANG006.C1.P1-BODY.md` contains the ordered patch-grade implementation tasks. Luna must read this header first, treat every `FIXED` rule and numbered invariant as binding, and use the body only as the ordered execution schedule. If the live repository materially contradicts this header, STOP AND CONSULT rather than silently redesigning the package.

**Goal:** Replace Phalcom's legacy visibility/implementation-name conventions with first-class declaration modifiers and identity-based access domains; introduce `native`, `foreign`, `private`, `protected`, `module`, `package`, and `internal` as real keywords; retain public as the implicit default with no `public` keyword; migrate Universe away from `@native` / `@private` / `@protected` / `@internal`, `_$selector`, and general `__implementationField` conventions; preserve canonical dunder protocols and independent Universe provenance; and establish one cross-stack visibility/implementation source of truth spanning AST, semantic analysis, compiler/runtime dispatch, native metadata, reflection, and tooling.

**Architecture:** Source syntax normalizes once into shared `Visibility` and `ImplementationKind` facts. Semantic declarations carry canonical module/package/project provenance. Runtime methods carry compact declaration-origin authority instead of a blanket `internal: bool`. Native implementations execute with the authority of the Phalcom declaration they implement. Universe remains special only through explicit bootstrap provenance (`ProjectIdentity::Universe` / `is_universe`), not through selector spelling or an all-purpose internal privilege. `@primitive` and `@intrinsic` remain metadata-only built-in attributes, authorable only in canonical Universe source.

**Tech stack:** Rust workspace (`phalcom-common`, `phalcom-ast`, `phalcom-modules`, `phalcom-semantic`, `phalcom-native-meta`, `phalcom-native-surface`, `phalcom-native-macros`, `phalcom-core`, `phalcom-lsp`), Phalcom Universe `.ph` sources, native descriptor/bootstrap verification, compiler/bytecode/VM access control, source indexing/reflection/tooling, focused Cargo tests.

**Checkpoint authority:** `LANG006.C1-CHECKPOINT.md`

**Target landing directory:**

```text
docs/implementation/LANG006/LANG006.C1/
  LANG006.C1-CHECKPOINT.md
  LANG006.C1.P1-HEADER.md
  LANG006.C1.P1-BODY.md
  LANG006.C1.P1-walkthrough.md          # produced by implementation
  LANG006.C1.P1-handoff.md              # produced by implementation
```

---

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High or an equivalently capable implementation model working under constrained architectural authority.

The implementer:

- must read this header in full before reading/executing the body;
- may adapt helper names, local decomposition, private function boundaries, imports, and mechanically equivalent APIs to live repository drift;
- must preserve the language semantics, ownership boundaries, canonical identities, access-domain relations, and migration endpoint defined here;
- must not improvise a different visibility lattice, a different native/foreign ownership model, or a new Universe privilege model;
- must not reintroduce access control through naming conventions;
- must not add a `public` keyword;
- must not redesign standardized dunder protocols;
- must not turn `@primitive` or `@intrinsic` into execution directives;
- must not make `native` Universe-only;
- must not make ordinary `internal` access depend on `is_universe`;
- must not invent a complete FFI or third-party native-extension loader to satisfy this plan;
- must follow BUILD / STABILIZE / CERTIFY testing discipline rather than running broad suites after every edit;
- must classify unrelated failures before spending implementation time on them;
- must update the shared checkpoint/walkthrough/handoff at durable gates required by the task body;
- must STOP AND CONSULT when any global or task-local trigger fires;
- must not commit or push unless separately authorized.

Testing is evidence gathering, not ritual. Run the smallest command that discriminates the invariant currently being changed.

---

## 1. Plan acceptance objective

P1 is intended to implement the **complete C1 language migration vertical**, not merely parser scaffolding.

At P1 completion:

```text
source syntax
    ↓
normalized Visibility + ImplementationKind
    ↓
semantic member surfaces + declaration provenance
    ↓
compiler/native descriptor verification
    ↓
runtime MethodObject access origin
    ↓
direct / cached / reflective / native invocation authorization
    ↓
Universe migrated to ordinary selectors/fields + modifiers
    ↓
LSP/reflection/spec/tooling parity
```

The minimum canonical source shape is:

```phalcom
native class Buffer {
  private _storage: Bytes
  module _cursor: Int
  package _scratch: ScratchState

  @primitive
  @intrinsic
  internal native rawCapacity -> Int

  package normalizeIndex(_ index: Int) -> Int {
    ...
  }

  at(_ index: Int) -> Byte {
    ...
  }
}
```

The minimum semantic interpretation is:

```text
Buffer
    implementation kind = Native

_storage
    field identity = _storage
    visibility = Private

_cursor
    field identity = _cursor
    visibility = Module

_scratch
    field identity = _scratch
    visibility = Package

rawCapacity
    selector identity = rawCapacity
    visibility = Internal
    implementation kind = Native
    primitive metadata = present
    intrinsic metadata = present

normalizeIndex
    selector identity = normalizeIndex(_)
    visibility = Package
    implementation kind = Phalcom

at
    selector identity = at(_)
    visibility = Public
    implementation kind = Phalcom
```

No source spelling may imply hidden access or implementation kind after normalization.

---

## 2. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   e0a77646c56f8b1a7e7692993d378d43bf3fc424
commit:     feat(lang005): continue P5 iterator surface
prepared:   2026-09-17
```

Recent mainline work is still active in LANG005. The implementation must begin with:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then re-read all live paths named in §4 and use `rg` to rediscover mechanically moved symbols/tests.

**Freshness rule:**

- mechanical drift: adapt locally;
- file relocation/name drift: rediscover and continue;
- changed helper signatures with unchanged architecture: adapt locally;
- changed visibility/native/module architecture: STOP AND CONSULT;
- a new implementation of any LANG006 feature already landed: reconcile with it; do not create a competing subsystem.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 3. Verified takeover facts

The planning pass verified these live architectural facts.

### 3.1 Lexical/AST legacy namespaces are explicit

`phalcom-ast` currently distinguishes ordinary/source/implementation namespaces including:

```text
Identifier
FieldIdentifier
ImplementationFieldIdentifier
ImplementationSelectorIdentifier
```

and retains implementation-specific AST expression/field forms.

Therefore `_$` / general `__field` retirement requires lexer/parser/AST changes, not only semantic renaming.

### 3.2 Visibility currently has only four cases

Both semantic/runtime layers currently model roughly:

```text
Public
Private
Protected
Internal
```

with no `Module` or `Package`.

`Internal` is still tied to privilege-oriented behavior in several consumers.

### 3.3 Visibility is re-derived in multiple consumers

Live code still infers access from combinations of:

```text
name.starts_with("_$")
field category
@private
@protected
@internal
native metadata
```

in compiler, semantic, trait, editor, native-source, installer, primitive registration, and generated-table paths.

P1 must converge these onto one canonical representation.

### 3.4 Native execution currently carries broad internal authority

`NativeMethodContext` currently carries an `internal: bool`, and native invocation paths can push a context with broad internal privilege.

This is incompatible with exact `module` / `package` / project-internal access.

### 3.5 Native metadata visibility is lossy

Native metadata currently distinguishes a narrower visibility set and some conversions collapse private/protected/internal into a single native-internal category.

P1 must preserve exact visibility.

### 3.6 Universe native indexing remains Universe-shaped

The canonical native source/descriptor system legitimately uses `UniverseKey` and canonical Universe bootstrap identities.

P1 must preserve those where primordial identity requires them while ensuring the general `native` language concept is represented by project/declaration identity, not `UniverseKey`.

### 3.7 Module/project identity is structural

`phalcom-modules` already has canonical:

```text
ProjectIdentity::{Universe, Resolved, Synthetic}
ModuleId { project, path }
ModuleKind::{Module, Package}
```

and entry ownership distinguishes Project, standalone Package, standalone Module, and inline/synthetic contexts.

`ProjectIdentity::is_universe()` already provides identity-based Universe provenance.

### 3.8 Package objects are package source units

The module architecture already treats a Package as the canonical `package.ph` module/object for that package.

P1 therefore does **not** need a second textual package namespace. Exact package access can use the canonical owning package module identity.

### 3.9 Public is already a semantic default in some surfaces

Current semantic member-surface helpers default to `Public`.

However existing field analysis contains private-default behavior.

P1 must deliberately migrate field defaults rather than assuming the repository is already uniform.

---

## 4. Required reads before implementation

### 4.1 Workflow authority

Read live equivalents of:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
docs/workflow/sol-principal-architect-project-doctrine.md   # if present
```

### 4.2 LANG006 authority

```text
docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P1-HEADER.md
```

If these files have not yet been copied into the repository, consume the supplied planning artifacts rather than creating divergent authority.

### 4.3 Accepted/spec authority that C1 changes

Re-read live authoritative equivalents for:

```text
docs/decisions/accepted/0076-transition-1-language-surface-convergence.md
docs/spec/lexical-structure.md
docs/spec/current/object-model.md
docs/spec/current/classes.md
docs/spec/attributes/...
docs/specs/vm/vm-execution-model.md
native declaration / Universe bootstrap specifications
module/project/package ownership specifications
```

C1 supersedes the TDR-0076 rules that make general `__name` an implementation-field namespace, `_$name` an implementation-selector namespace, and internal selector authority a bootstrap-core privilege.

Standardized dunder protocol rules are not superseded.

### 4.4 Exact implementation seams

At minimum re-read:

```text
phalcom-common/
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-modules/src/identity.rs
phalcom-modules/src/source.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/session.rs
module topology/package-owner helpers

phalcom-semantic/src/surface.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/source_index/builder.rs
semantic access-context / hierarchy helpers
semantic fingerprints/metadata where member visibility is serialized

phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/method/object.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/native/source.rs
phalcom-core/src/native/descriptor.rs
phalcom-core/src/native/verify.rs
phalcom-core/src/native/install.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/primitive/
phalcom-core/src/modules/

phalcom-native-meta/
phalcom-native-surface/
phalcom-native-macros/

phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/editor/navigation/completion paths

phalcom-core/core/universe/src/
tools/phalcom*.json
```

### 4.5 Focused test conventions

Rediscover live tests with `cargo test ... -- --list` and `rg`.

Expected owners include:

```text
phalcom-ast/tests/
phalcom-modules/tests/
phalcom-semantic/tests/semantic/
phalcom-core/tests/core/
phalcom-lsp/tests/
native metadata/macro crate tests
Universe source/bootstrap tests
```

Do not restart a broad repository audit unless a STOP/CONSULT trigger requires it.

---

## 5. Normative language authority

### 5.1 The new keywords are fixed

Exactly these new keywords are introduced:

```text
native
internal
private
protected
module
package
foreign
```

### 5.2 There is no `public` keyword

Public is the semantic default.

```phalcom
foo() { ... }             // Public
native foo() -> Int       // Public + Native
```

`public` remains an ordinary identifier under the lexer unless another independent language feature already reserves it. P1 must not reserve it for visibility.

### 5.3 The new built-in attributes are fixed

```text
@primitive
@intrinsic
```

They are metadata-only and authorable only in canonical Universe source.

### 5.4 Names do not carry access semantics

After migration:

```text
_$foo
__legacyField
```

carry no sanctioned authored visibility semantics.

Ordinary selectors and `_field` identity are independent of visibility.

### 5.5 Dunder protocols remain stable

Standardized/reserved:

```text
__name__
```

protocol spelling stays outside C1.

A migration of general implementation fields must not reinterpret or remove those protocols.

### 5.6 Universe provenance remains independently observable

`is_universe` / `ProjectIdentity::Universe` remains legitimate state.

The new ordinary access rule is never:

```text
Internal => is_universe
```

---

## 6. Shared canonical model

### 6.1 FIXED: canonical enums live in `phalcom-common`

P1 will introduce the shared low-level semantic categories in `phalcom-common`:

```rust
pub enum Visibility {
    Public,
    Private,
    Protected,
    Module,
    Package,
    Internal,
}

pub enum ImplementationKind {
    Phalcom,
    Native,
    Foreign,
}
```

Exact naming/module placement inside `phalcom-common` is mechanically flexible, but the ownership decision is fixed.

Rationale:

- `phalcom-ast` already depends on `phalcom-common`;
- semantic/core consumers can depend on the same neutral definitions;
- `phalcom-native-meta` can add a low-level dependency on `phalcom-common` rather than maintaining a lossy parallel visibility enum;
- this removes cross-consumer drift and conversion-by-string.

Do **not** create independent canonical semantic visibility enums in AST, semantic, core, and native-meta.

Compatibility wrapper/re-export types may exist transiently during migration but must disappear or become transparent aliases by P1 completion.

### 6.2 AST representation

Declarations that support modifiers must store normalized structured facts equivalent to:

```rust
visibility: Visibility,
implementation: ImplementationKind,
visibility_range: Option<SourceRange>,
implementation_range: Option<SourceRange>,
```

The exact helper struct versus direct fields is mechanically flexible.

Absence of source visibility syntax lowers to:

```rust
Visibility::Public
```

Absence of source implementation syntax lowers to:

```rust
ImplementationKind::Phalcom
```

### 6.3 Source ordering

The canonical declaration order is:

```text
attributes*
visibility?
implementation-kind?
declaration-core
```

Example:

```phalcom
@primitive
@intrinsic
internal native suspend(_ value: Dynamic) -> Dynamic
```

**FIXED P1 parser rule:** new modifier order is strict. `visibility` precedes `native`/`foreign`.

Reject noncanonical permutations such as:

```phalcom
native internal foo()
```

with a targeted modifier-order diagnostic rather than silently normalizing them.

This keeps the grammar deterministic and gives the language one written form.

---

## 7. Visibility semantics

### 7.1 Canonical set

```text
Public
Private
Protected
Module
Package
Internal
```

### 7.2 Exact relations

```text
Public
    accessible after ordinary declaration/name/import resolution

Private
    caller lexical class == declaration lexical class

Protected
    caller lexical class == declaration lexical class
    OR caller lexical class is an eligible subclass

Module
    caller defining ModuleId == declaration defining ModuleId

Package
    caller owning package identity == declaration owning package identity

Internal
    caller ProjectIdentity/internal owner == declaration ProjectIdentity/internal owner
```

### 7.3 Protected is not part of a simple total order

Do not implement visibility with numeric comparison.

`Protected` is class-hierarchy oriented, while `Module` / `Package` / `Internal` are provenance-oriented.

Any compatibility calculation that needs "at least as accessible as" must compare actual access domains/semantics, not enum ordinals.

### 7.4 Package identity

**FIXED:** package identity is the canonical `ModuleId` of the owning `package.ph` package source unit.

For a Project root package, the root package module is its package identity.

For a nested package, that nested `package.ph` module is the identity.

For an ordinary module, its owning package is the nearest logical package ancestor already established by the module topology/source ownership model.

No path-string or filesystem-prefix comparison is permitted in access checks.

If the live module layer does not currently publish an owning-package query, P1 may add one to `phalcom-modules`; it must derive from canonical module topology/source ownership, not ad-hoc filesystem probing in semantic/core consumers.

### 7.5 Internal-domain identity

**FIXED default representation:** ordinary `internal` authority uses the module's canonical `ProjectIdentity`.

The existing variants already distinguish:

```text
Universe
Resolved(project)
Synthetic(session/entry)
```

Standalone package/module execution must continue to allocate/retain synthetic ownership so the intended source group shares one `ProjectIdentity` where appropriate.

Do not create a second parallel internal-domain identity if the live source/module ownership model already makes `ProjectIdentity` sufficient.

**VERIFY-FIRST trigger:** if live standalone Package members do not share one canonical `ProjectIdentity`, STOP AND CONSULT before inventing a new internal-domain scheme. The correct fix may belong in module ownership.

### 7.6 Member versus export visibility

C1 member visibility does not replace module export semantics.

Top-level declarations remain exposed by existing:

```text
export / import / re-export / expose
```

rules.

C1 does not introduce:

```phalcom
internal class Foo
package class Foo
module class Foo
```

as general top-level namespace visibility.

`native class Foo` is different because `native` is implementation kind.

---

## 8. Runtime access-origin architecture

### 8.1 Semantic declaration provenance

Semantic/member products must retain enough canonical provenance to evaluate:

```text
lexical class
defining ModuleId
owning package ModuleId
ProjectIdentity
```

Do not reconstruct provenance from source paths at query sites.

### 8.2 FIXED runtime representation direction

Retain the existing `access_owner: Option<ClassId>` role for private/protected lexical-class authority.

Add compact runtime provenance equivalent to:

```rust
struct RuntimeAccessOrigin {
    module: ObjRef,
    package: Option<ObjRef>,
    project: ProjectIdentity,
}
```

The exact struct name/location and whether `project` is stored directly or via an equally cheap interned owner handle are mechanically flexible.

The required properties are fixed:

- module comparison is O(1) canonical identity comparison;
- package comparison is O(1) canonical package-object/handle comparison;
- internal comparison is O(1) canonical project-owner comparison;
- no source/path parsing occurs on a send;
- no attribute scanning occurs on a send.

A method may carry this origin directly or a compact handle to an interned origin record. Do not create per-call heap allocation.

### 8.3 Caller authority

Replace the current effective:

```text
(lexical_class, caller_internal_bool)
```

transport with a structured caller authority carrying the relevant access origin.

This affects, where live code requires it:

```text
ArgumentView
send/invoke helpers
native gateway
method references/bound calls
reflection/dynamic invocation
compiler synthetic invocation
```

No invocation path may remain a loophole that authorizes `Internal` through a boolean after P1 completion.

### 8.4 Native method context

`NativeMethodContext` must derive authority from the Phalcom declaration implemented by the native function.

It must not use:

```rust
internal: true
```

as ambient authorization.

Conceptually it becomes equivalent to:

```rust
struct NativeMethodContext {
    access_owner: Option<ClassId>,
    origin: RuntimeAccessOrigin,
    is_universe: bool, // only if live bootstrap/compiler consumers require it
}
```

`is_universe` remains provenance metadata, not a substitute for `origin.project`.

---

## 9. Declaration target legality matrix

The following P1 legality matrix is fixed unless a live pre-existing normative spec directly contradicts it; such contradiction is a STOP/CONSULT event.

Legend:

```text
✓  legal in C1
—  not applicable
R  rejected with targeted diagnostic in C1
C  preserve current behavior; do not broaden during C1
```

| Declaration form | Visibility modifier | `native` | `foreign` | `@primitive` | `@intrinsic` |
|---|---:|---:|---:|---:|---:|
| class declaration | — | ✓ | R | R | R |
| enum declaration | — | ✓ | R | R | R |
| data declaration | — | R | R | R | R |
| trait declaration | — | R | R | R | R |
| type alias | — | R | R | R | R |
| class method/getter/setter/index/constructor | ✓ | ✓ | ✓* | ✓** | ✓** |
| class field | ✓ | R | R | R | R |
| inherent `impl` behavior member | ✓ | ✓ | ✓* | ✓** | ✓** |
| trait requirement/default | C | R | R | ✓** | ✓** |
| trait-conformance `impl` behavior member | C | ✓ | ✓* | ✓** | ✓** |
| delegation member / `via` | C | R | R | R | R |
| associated type declaration/binding | C | R | R | R | R |
| enum variant | C | inherited from native enum where current model requires | R | R | R |

`*` `foreign` callable support in P1 is a semantic scaffold only; see §11.

`**` `@primitive` / `@intrinsic` are legal only when the declaration is in canonical Universe source. They remain metadata-only and do not require `native`, do not require a body, and do not forbid a body by themselves.

### 9.1 Trait visibility compatibility

Current Phalcom already has trait-member visibility behavior/tests for `private`, `protected`, and legacy `internal`.

P1 must migrate that existing capability to keyword-backed normalized visibility rather than deleting it.

**C1 does not broaden abstract trait visibility to new `module` or `package` requirements.**

Therefore:

```text
trait behavior:
    Public       legal
    Private      preserve current legality
    Protected    preserve current legality
    Internal     preserve current legality, redefined to Project identity
    Module       reject in C1
    Package      reject in C1
```

For trait-conformance implementation members, preserve current visibility legality/compatibility behavior; do not introduce new `module`/`package` witness rules unless the requirement is already expressible and a focused semantic proof is available.

Inherent `impl` behavior may use the full C1 visibility set because it is ordinary concrete behavior.

This boundary prevents C1 from becoming a redesign of trait witness visibility.

---

## 10. Native semantics

### 10.1 Definition

> A `native` declaration is a Phalcom-owned declaration whose implementation or runtime representation is supplied by a Phalcom native implementation provider rather than by an ordinary executable Phalcom body.

### 10.2 Native is general, not Universe-only

`native` is a language keyword usable by ordinary projects.

The current runtime may only have a complete provider/binder for Universe during P1.

That implementation limitation must surface as an explicit native-provider/binding diagnostic, not as a parser restriction that says only Universe may declare `native`.

### 10.3 Native callable bodies

Canonical native callable declarations are bodyless.

```phalcom
native decode(_ bytes: Bytes) -> Image
```

A source body on a native callable is rejected.

Historical `@native` reference bodies are migrated; they do not justify preserving a body-that-never-runs model.

### 10.4 Native classes/enums

`native class` and `native enum` are legal and replace current class/enum `@native` presentation declarations.

A native type declaration may contain ordinary Phalcom behavior.

```phalcom
native class String is Object {
  internal native byteCount -> Int

  size -> Int {
    byteCount
  }
}
```

The native type modifier does not implicitly mark every enclosed member native.

### 10.5 Native binding identity

General native member identity is declaration-qualified:

```text
DeclarationId + DispatchSide + Selector
```

or an exactly equivalent canonical identity.

`UniverseKey` remains a bootstrap adapter for primordial Universe declarations where needed.

Do not use Rust function names or selector spelling prefixes as the source/native binding key.

### 10.6 Native visibility metadata

Native metadata must represent the shared `Visibility` exactly.

Do not collapse:

```text
Private
Protected
Module
Package
Internal
```

into one native-internal bucket.

`phalcom-native-meta` should depend on/reuse the shared common visibility type unless live crate architecture proves a cycle. If a dependency cycle unexpectedly appears, STOP AND CONSULT before creating another canonical enum.

---

## 11. Foreign semantics

### 11.1 Definition

> A `foreign` declaration describes a callable whose implementation identity originates outside the Phalcom runtime/native-extension model and is reached through an interoperability boundary.

It is distinct from `native`.

### 11.2 P1 bounded support

P1 introduces:

```text
Token::Foreign
AST ImplementationKind::Foreign
modifier legality/conflict diagnostics
semantic/source-index/reflection representation
bodyless foreign callable declaration syntax
explicit unsupported/unbound lowering result
```

P1 does **not** implement a general FFI.

### 11.3 FIXED P1 foreign target

In P1, `foreign` is legal only on callable behavior declarations in contexts where a callable declaration can be represented:

```text
class behavior
inherent impl behavior
trait-conformance impl behavior
```

It is rejected on:

```text
class/enum/data/trait/type declarations
fields
trait abstract requirements/defaults
associated declarations
delegations
variants
```

The final FFI may later introduce dedicated foreign type/function forms. C1 intentionally does not pre-commit those forms.

### 11.4 Foreign body rule

A `foreign` callable is bodyless.

Any compile/lower attempt without a supported foreign binding backend must fail with an explicit diagnostic such as:

```text
foreign.binding_unimplemented
```

rather than installing a stub, silently treating it as native, or producing method-missing at runtime.

---

## 12. `@primitive` / `@intrinsic` contract

### 12.1 Built-in, no arguments

Add genuine built-in attribute variants:

```text
Primitive
Intrinsic
```

Both accept zero arguments only in C1.

### 12.2 Universe-only authoring

The source declaration's canonical module project must satisfy:

```rust
module_id.project.is_universe()
```

or the live equivalent.

A user project named `universe` is not privileged.

### 12.3 Metadata-only

Neither attribute changes:

```text
Visibility
ImplementationKind
selector
field identity
typing
dispatch
body legality
native binding
foreign binding
bytecode selection
optimization
```

No rule in P1 says:

```text
@primitive => native
@intrinsic => native
@primitive => dedicated VM primitive
@intrinsic => special lowering
```

A later backend may consume these facts.

### 12.4 Initial target set

C1 allows both attributes on callable behavioral declarations only, including methods/getters/setters/index/constructors and callable trait/default/impl behavior where attributes are otherwise supported.

They are rejected on class/enum/data/trait/type declarations, fields, associated declarations, delegations, and variants.

If existing canonical Universe source contains a required use outside this target set, STOP AND CONSULT with a concrete census rather than broadening ad hoc.

### 12.5 Metadata retention

The attributes must survive parsed AST/source indexing/semantic metadata far enough to be observable by compiler/tooling consumers that inspect built-in declaration metadata.

They must not be stripped by the legacy `@native` subtraction path.

---

## 13. Legacy migration rules

### 13.1 Retired attributes

Canonical replacements:

```text
@native      → native
@private     → private
@protected   → protected
@internal    → internal
```

At P1 completion, these legacy attributes are no longer semantic authorities.

Use targeted diagnostics for authored old syntax.

Do not retain permanent dual syntax.

### 13.2 `_$` selector census

Every existing `_$` declaration/use must be classified before replacement:

| Class | Meaning | Migration |
|---|---|---|
| A | native runtime floor | ordinary selector + correct visibility + `native` |
| B | source-defined helper | ordinary selector + correct restrictive visibility |
| C | real compiler/language protocol | preserve/move through protocol mechanism; do not blindly normalize |
| D | compiler/VM synthetic seam | prefer typed/direct internal mechanism; no authored `_$` requirement |

For every migrated `_$` member, determine actual callers before selecting `private` / `module` / `package` / `internal`.

Do not implement:

```text
strip "_$" and add "internal"
```

as a repository-wide rewrite.

### 13.3 General `__field` census

Every legacy general implementation field is classified by actual access:

```text
private _field
module _field
package _field
internal _field
```

Do not assume all become `internal`.

### 13.4 Dunder exclusion

A spelling matching the standardized reserved protocol family, e.g.:

```text
__name__
```

is not migrated merely because it begins with `__`.

P1 must preserve protocol parsing/semantics.

If lexer tokenization currently conflates legacy implementation fields with standardized dunders, introduce the minimum lexical distinction needed to remove the legacy field namespace without changing dunder protocols.

### 13.5 Default-public field migration

Because C1 makes absent visibility `Public`, all existing ordinary `_field` declarations must be reviewed.

Any field that relied on old implicit-private behavior must gain:

```phalcom
private _field
```

or the correct broader restrictive scope.

No accidental field exposure is acceptable as migration fallout.

---

## 14. Keyword reservation policy

### 14.1 True keyword behavior

The seven new words are real lexer tokens in declaration/binding grammar.

They are not ordinary local/parameter/type/member declaration identifiers in ambiguous declaration positions.

### 14.2 Selector/property contexts

Follow Phalcom's established contextual keyword-as-property policy where syntax is already unambiguous.

Specifically, it is acceptable for the parser's property/selector production to consume these keyword tokens after an explicit selector introducer such as `.` / `#` / callable-reference syntax if the live parser already uses this model for other keywords.

Do **not** add bespoke lexical escape syntax in C1.

### 14.3 Collision handling

Before tightening parser rules, census current repository uses of all seven spellings as identifiers.

Repository-owned collisions should be renamed intentionally.

A material public-language collision that cannot be resolved without changing an established API is a STOP/CONSULT condition.

### 14.4 `public`

Regression invariant:

```text
public
```

does not become a visibility keyword.

---

## 15. Source-of-truth table

| Fact | Canonical owner after P1 | Consumers | Forbidden duplicate |
|---|---|---|---|
| visibility category | `phalcom-common::Visibility` | AST, semantic, core, native-meta, LSP | local independent enums / strings |
| implementation kind | `phalcom-common::ImplementationKind` | AST, semantic, compiler, native/foreign paths, LSP | `@native` scan / booleans |
| source modifier spans | AST declaration modifier metadata | diagnostics/tooling | rescanning source text |
| lexical class authority | semantic declaration → runtime `access_owner` | semantic/runtime access checks | caller-name inference |
| defining module | canonical `ModuleId`; runtime module handle | semantic/runtime/module visibility | source path identity |
| owning package | canonical owning `package.ph` `ModuleId`; runtime package object handle | package visibility | path-prefix ancestry |
| internal owner | canonical `ProjectIdentity` | internal visibility | `is_universe` / privilege bool |
| Universe provenance | `ProjectIdentity::Universe` / `is_universe()` | bootstrap, Universe-only attrs/protocols | project name checks |
| native binding identity | declaration identity + dispatch side + selector | source/native verifier/installer | Rust symbol name / `_$` prefix |
| primitive metadata | built-in `@primitive` attribute | source index/tooling/backend consumers | implementation-kind inference |
| intrinsic metadata | built-in `@intrinsic` attribute | source index/tooling/backend consumers | automatic special lowering |
| export visibility | module export/linker system | import/name resolution | member Visibility |
| dunder protocol identity | existing protocol authority | compiler/runtime/protocol users | legacy implementation-field logic |

---

## 16. Global invariants

### INV-01 — No public keyword

`Visibility::Public` is the default semantic state; `public` is not declaration visibility syntax.

### INV-02 — One visibility truth

Visibility is normalized once from source modifiers and consumed structurally. No subsystem derives it from naming prefixes or retired attributes.

### INV-03 — One implementation-kind truth

Implementation kind is normalized once. `@native` and selector spelling are not implementation-kind authorities.

### INV-04 — Selector identity is visibility-independent

Changing a member from public to internal/package/module/private/protected does not change its canonical selector.

### INV-05 — Field identity is visibility-independent

`_field` remains the field identity regardless of visibility.

### INV-06 — Module access is exact identity

`module` uses exact canonical Module identity, not path/string equality invented by consumers.

### INV-07 — Package access is exact package identity

`package` uses the owning canonical `package.ph` module/package object identity. Child/parent/sibling packages are not equal.

### INV-08 — Internal access is project-owner identity

Ordinary `internal` uses canonical Project ownership, including Universe as one ordinary project identity for this relation.

### INV-09 — Universe provenance is independent

`is_universe` remains separately available and does not implement ordinary internal access.

### INV-10 — Native code is not globally privileged

Native execution receives the declaration's access authority and nothing broader merely because it is native.

### INV-11 — Native is not Universe-only

The semantic language category is general even if P1's fully operational provider remains Universe-centric.

### INV-12 — Foreign is not native

Foreign and native are distinct implementation kinds and cannot be combined.

### INV-13 — Primitive/intrinsic are metadata-only

Presence of `@primitive` / `@intrinsic` does not itself alter execution semantics.

### INV-14 — Primitive/intrinsic source authoring is Universe-only

Legality is checked by canonical Universe project identity.

### INV-15 — Dunder protocols are outside migration scope

General `__field` retirement must not mutate standardized `__name__` protocol semantics.

### INV-16 — Native source signatures remain Phalcom contracts

Native ABI representation never replaces the Phalcom signature/type contract.

### INV-17 — Runtime access is hot-path structural

No source/path parsing, attribute scanning, or string-prefix analysis occurs to authorize a dispatch.

### INV-18 — Export and member visibility remain orthogonal

Module export semantics are not rewritten into member visibility modifiers.

### INV-19 — No silent compatibility aliases at completion

Retired `@native` / `@private` / `@protected` / `@internal`, `_$`, and general implementation-field semantics do not remain hidden alternate authorities.

### INV-20 — Semantic/runtime parity

A statically inaccessible member cannot become accessible through dynamic, cached, bound, reflected, or native invocation paths.

---

## 17. Expected impact map

### 17.1 Shared model

```text
phalcom-common/
    shared Visibility / ImplementationKind
```

### 17.2 Front end

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/
```

### 17.3 Module/package ownership

```text
phalcom-modules/src/identity.rs
phalcom-modules/src/source.rs
topology/interface/session helpers publishing owning-package identity
phalcom-modules/tests/
```

### 17.4 Semantic model

```text
phalcom-semantic/src/surface.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/source_index/builder.rs
metadata/fingerprint products that retain visibility/implementation kind
semantic access context
phalcom-semantic/tests/semantic/
```

### 17.5 Compiler/runtime

```text
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/method/object.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/modules/
phalcom-core/tests/core/
```

### 17.6 Native stack

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
```

### 17.7 Universe source

```text
phalcom-core/core/universe/src/**/*.ph
```

including a classified migration of:

```text
@native
@private
@protected
@internal
_$...
legacy general __fields
```

### 17.8 Tooling/specification

```text
phalcom-lsp/
tools/phalcom*.json
current normative language specs
new/superseding decision record
LANG006 checkpoint/walkthrough/handoff
```

### 17.9 Unexpected-touch rule

Mechanical adjacent changes are allowed when required by the shared type migration.

STOP AND CONSULT before entering a materially different subsystem if that entry changes architecture, semantic ownership, runtime representation strategy, FFI scope, module topology, or dunder protocol semantics.

---

## 18. Implementer decision authority

### 18.1 FIXED

The implementer must not change:

1. the seven-keyword set;
2. absence of a `public` keyword;
3. public as default visibility;
4. the six-case semantic visibility set;
5. exact Module/package/Project identity semantics;
6. package identity as the owning canonical `package.ph` package;
7. `ProjectIdentity` as the intended internal-domain owner;
8. the `phalcom-common` shared enum ownership direction;
9. `Phalcom` / `Native` / `Foreign` implementation kinds;
10. bodyless native/foreign callable target syntax;
11. general `native` availability outside Universe;
12. metadata-only `@primitive` / `@intrinsic`;
13. Universe-only authoring of those two attributes;
14. preservation of `is_universe`;
15. retirement of semantic `_$` and general implementation fields;
16. dunder protocols being out of scope;
17. no replacement of module exports with member visibility;
18. no complete FFI/native-extension loader expansion;
19. native declaration-derived access authority;
20. no permanent legacy syntax aliases.

### 18.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact Rust type/field names;
- helper decomposition;
- module path of the common modifier enums;
- whether AST stores one `DeclarationModifiers` struct or equivalent fields;
- whether runtime origin is embedded directly or referenced by a compact interned handle;
- exact diagnostic enum names while preserving categories/meaning;
- exact focused test filenames following repository naming conventions;
- exact migration names for individual old `_$` helpers after caller analysis;
- local transition adapters needed to keep intermediate patches compiling;
- command filters based on live test binary layout.

### 18.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| `phalcom-common` can be depended on by `phalcom-native-meta` without cycle | Cargo manifests/dependency graph | STOP AND CONSULT before duplicating canonical enums |
| standalone Package modules share a stable `ProjectIdentity` suitable for `internal` | `phalcom-modules` ownership/session tests | STOP AND CONSULT; do not invent consumer-local domain IDs |
| owning package module can be derived canonically from existing topology | module topology/interface/session | add one canonical owner query if mechanical; consult if topology semantics must change |
| current trait visibility includes private/protected/internal | semantic trait tests/live code | preserve live supported subset; consult if checkpoint assumption is materially false |
| native class/enum presentation remains required by Universe | native source census | adapt mechanically if some category disappeared |
| current native descriptor binding uses canonical source identity | native descriptor/verify/install | consult if binding architecture has materially changed |
| native invocation still uses broad `internal` authority | VM native context/send path | if already fixed compatibly, consume live architecture |
| `public` is currently available as ordinary identifier | lexer/parser census | consult only if independently reserved by newer language work |
| dunder protocols are lexically distinguishable from legacy general implementation fields or can be separated locally | lexer/parser/protocol code | STOP AND CONSULT if protocol redesign would be required |

---

## 19. Global STOP / CONSULT triggers

The implementer must stop editing and prepare a consultation packet if any of these occurs:

1. any FIXED language rule appears impossible or inconsistent with newer normative authority;
2. `ProjectIdentity` cannot correctly model standalone-package internal ownership without module-architecture changes;
3. exact package identity cannot be derived from canonical package/module topology without redesign;
4. the shared common enums would introduce a crate dependency cycle requiring architectural duplication;
5. native access cannot be made declaration-derived without changing the native ABI/activation model materially;
6. correct `module`/`package`/`internal` enforcement would require per-send path parsing or other noncanonical lookup;
7. removing `_$` exposes a current compiler/runtime protocol whose correct replacement is ambiguous;
8. legacy `__field` migration collides with standardized dunder protocol semantics;
9. a public repository/API uses one of the seven new keywords in a way that requires a language compatibility decision;
10. trait visibility semantics require a new conformance theory rather than migration of existing behavior;
11. implementing `foreign` appears to require choosing ABI/ownership/marshaling policy;
12. third-party native support appears to require a manifest/loader ABI beyond the semantic normalization in this plan;
13. source/native descriptor verification cannot retain exact visibility without a substantial metadata redesign not anticipated here;
14. a semantic access test and runtime access test imply different correct results;
15. the same nontrivial semantic failure persists after one serious correction;
16. two plausible fixes differ materially in ownership, identity, or runtime representation;
17. a workaround based on selector/field spelling begins to emerge;
18. an invariant must be weakened merely to pass existing tests;
19. live authoritative spec/code/test rules materially conflict;
20. task scope expands into dunder protocol redesign, export-system redesign, or full FFI.

A triggered consultation cannot be self-waived.

---

## 20. Consultation packet format

When escalation is required, report:

```text
Trigger:
Exact live revision:
Task/body step:
Observed:
Expected by header/invariant:
Minimal reproducer/test:
Relevant files/symbols:
Evidence:
Why mechanical adaptation is insufficient:
Option A:
Option B:
Tradeoff:
Recommended question for principal architect:
```

Do not continue speculative architecture work while waiting for a decision.

---

## 21. Debugging budget

### 21.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

One cycle:

```text
inspect
identify concrete mechanical cause
make one coherent correction
rerun smallest discriminating command
```

Examples:

```text
renamed AST field
moved test module
missing import
exhaustive match update
metadata conversion compile failure
```

### 21.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### 21.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

### 21.4 Test rerun rule

Never rerun an unchanged failing test unless code/configuration relevant to that failure changed.

---

## 22. Testing surface analysis

The task body must reference these coverage IDs rather than inventing a new testing model per task.

| Coverage ID | Dimension | Invariants | Required evidence |
|---|---|---|---|
| CV-01 | lexer keywords/default | INV-01 | seven new keyword tokens; `public` remains identifier |
| CV-02 | AST normalization | INV-02, INV-03 | visibility/implementation stored structurally |
| CV-03 | modifier conflicts/order | INV-01–03 | duplicate/conflicting/order errors |
| CV-04 | field migration/default | INV-05, INV-19 | default-public + explicit private/module/package/internal |
| CV-05 | module access | INV-06, INV-20 | same module allowed; sibling module denied |
| CV-06 | package access | INV-07, INV-20 | same exact package allowed; child/parent/sibling denied |
| CV-07 | internal access | INV-08, INV-09 | same Project allowed; other Project denied; Universe behaves by Project identity |
| CV-08 | private/protected regression | INV-20 | lexical class/subclass behavior unchanged |
| CV-09 | dynamic invocation parity | INV-20 | method ref/bound/reflection/dynamic cannot bypass visibility |
| CV-10 | native authority | INV-10 | native method receives declaration origin, not blanket internal |
| CV-11 | native metadata parity | INV-02, INV-11, INV-16 | exact visibility/signature binding |
| CV-12 | native Universe migration | INV-02–05, INV-19 | representative String/List/Fiber/System surfaces use new syntax |
| CV-13 | primitive/intrinsic legality | INV-09, INV-13, INV-14 | Universe accepted; fake/user Universe rejected; metadata inert |
| CV-14 | foreign scaffold | INV-12 | foreign parsed/represented; body rejected; unsupported binding explicit |
| CV-15 | dunder preservation | INV-15 | standardized dunder protocol remains unchanged |
| CV-16 | legacy rejection | INV-19 | retired attrs / `_$` / general implementation fields no longer act as semantic aliases |
| CV-17 | tooling parity | INV-02, INV-03, INV-20 | LSP token/hover/completion access parity |
| CV-18 | cache parity | INV-17, INV-20 | cached dispatch applies same access result |
| CV-19 | source identity | INV-04, INV-05 | migration changes visibility/implementation, not ordinary selector/field identity |
| CV-20 | standalone ownership | INV-08 | standalone Package/Module internal domain behavior where harness supports it |

### 22.1 BUILD / STABILIZE / CERTIFY budget

**BUILD**

Run the smallest owner test after each durable interface is introduced:

```text
common/AST compile
lexer/parser focused test
semantic focused visibility test
core focused dispatch/native test
native-meta focused descriptor test
```

Do not run the whole workspace between body tasks.

**STABILIZE**

After one cross-layer vertical is connected, run the focused owning integration slices:

```text
phalcom-ast
phalcom-modules ownership/package slice
phalcom-semantic visibility/access slice
phalcom-core language/runtime access slice
native metadata/source verification slice
```

**CERTIFY**

At P1 completion:

- affected crates compile;
- focused LANG006 tests pass;
- canonical Universe bootstrap/source tests pass;
- LSP focused modifier/access tests pass;
- repository-wide old-syntax census is clean except historical/non-authoritative docs or explicitly preserved test fixtures;
- broader workspace tests are run only according to the repository workflow and failure classification rules.

---

## 23. Diagnostic policy

Use centralized typed diagnostics where the repository architecture supports them.

Required semantic categories include equivalents of:

```text
syntax.retired_native_attribute
syntax.retired_visibility_attribute
syntax.retired_implementation_selector
syntax.retired_implementation_field

declaration.multiple_visibility_modifiers
declaration.conflicting_implementation_kinds
declaration.modifier_order
declaration.illegal_modifier_target

member.private_access
member.protected_access
member.module_access
member.package_access
member.internal_access

native.body_not_allowed
native.missing_implementation
native.visibility_mismatch
native.signature_mismatch

foreign.body_not_allowed
foreign.binding_unimplemented
foreign.conflicting_native

attribute.primitive_requires_universe
attribute.intrinsic_requires_universe
attribute.primitive_illegal_target
attribute.intrinsic_illegal_target
```

Exact names may follow live diagnostic taxonomy, but diagnostics must describe the new semantic model.

Do not retain `internal.selector_access` as the canonical user-facing concept after the implementation-selector namespace is removed.

For modifier conflicts, retain both relevant source spans where the diagnostics infrastructure permits.

---

## 24. Migration census contract

Before source rewrites, the task body will require census tables for:

```text
@native
@private
@protected
@internal
_$ declarations
_$ call sites/references
legacy general __fields
compiler-generated internal sends
primitive/native registration macros
native visibility mappings
keyword collisions
```

For each `_$` declaration that will be migrated, record at least:

```text
owner declaration
old selector
implementation kind
all meaningful caller scopes
classification A/B/C/D
chosen new selector
chosen visibility
protocol/synthetic note
```

For each general `__field`, record:

```text
owner declaration
old field
read/write callers
chosen new _field spelling
chosen visibility
dunder exclusion check
```

This census is implementation evidence, not optional documentation.

---

## 25. Documentation and state contract

P1 completion requires:

1. a new accepted decision or authoritative amendment superseding the relevant TDR-0076 implementation-namespace rules;
2. normative lexical/member/access/native documentation updated to the new modifiers;
3. dunder protocol documents unchanged except cross-reference clarification if necessary;
4. `LANG006.C1-CHECKPOINT.md` updated with completed P1 state;
5. `LANG006.C1.P1-walkthrough.md` recording actual implementation and verification;
6. `LANG006.C1.P1-handoff.md` recording residual work or checkpoint closure;
7. historical plans left historical rather than mass-rewritten.

---

## 26. Non-goals

P1 does not own:

```text
a `public` keyword
dunder protocol redesign
friend declarations
package-family / descendant-package visibility
a second top-level declaration visibility system
replacement of import/export semantics
complete FFI ABI design
C/Rust ABI spelling
foreign struct layout
foreign pointer/lifetime/ownership policy
callback ABI
unsafe FFI model
full third-party native manifest/loader/distribution ABI
native field syntax
trait/conformance architecture redesign
new module topology architecture
removal of is_universe
removal of UniverseKey where primordial bootstrap uses it
elimination of legitimate primordial bootstrap special cases
automatic primitive installation from @primitive
automatic intrinsic lowering from @intrinsic
making visibility part of selector identity
making implementation kind part of selector identity
```

---

## 27. Body interface: how to read `LANG006.C1.P1-BODY.md`

The body is the ordered patch schedule only.

Every body task must contain:

```text
Task ID
Objective
Preconditions / referenced invariants
Exact expected source areas
Required edits
Required focused tests
Gate / expected evidence
STOP/CONSULT triggers specific to task
Checkpoint/walkthrough update if durable
```

The body must not silently override this header.

When the body says:

```text
"per header"
```

it refers to the semantic/architectural rules in this file.

If a task can be completed in more than one mechanically equivalent way, Luna may choose among them only within §18.2 flexibility.

---

## 28. Planned body decomposition

The final task body should follow this architectural sequence; this section is **not** the implementation task body and contains no patch instructions.

```text
T0  Re-ground live checkout, read authority, census legacy syntax/keywords
T1  Introduce shared common modifier types and front-end tokens/AST normalization
T2  Parse/validate modifier grammar + built-in primitive/intrinsic metadata
T3  Publish canonical module/package/project access provenance in semantic model
T4  Converge semantic visibility consumers and trait/inherent behavior
T5  Replace runtime privilege boolean with structured access origin
T6  Converge native metadata/source verifier/installer on exact shared visibility
T7  Establish bounded foreign semantic scaffold
T8  Classify and migrate Universe selectors/fields/attributes
T9  Remove legacy prefix/attribute inference and compatibility namespaces
T10 LSP/reflection/tooling/specification convergence
T11 Stabilize, audit old-syntax absence, certify cross-path parity
```

The task body may split these into smaller task/gate units for Luna execution, but must not reorder semantic dependencies in a way that creates two simultaneous authorities.

---

## 29. P1 success criteria

P1 is complete only when all of the following are true:

```text
[ ] seven new keywords are canonical source syntax
[ ] no public keyword exists
[ ] absent visibility is Public for visibility-bearing members/fields
[ ] visibility uses shared structural metadata
[ ] implementation kind uses shared structural metadata
[ ] Module visibility uses exact Module identity
[ ] Package visibility uses exact owning package identity
[ ] Internal visibility uses exact Project owner identity
[ ] private/protected behavior remains correct
[ ] native execution has declaration-derived authority
[ ] native metadata preserves exact visibility
[ ] native class/enum/callable source uses keyword syntax
[ ] foreign has a bounded bodyless callable semantic representation
[ ] @primitive / @intrinsic are builtin, metadata-only, Universe-only
[ ] is_universe remains available
[ ] authored _$ implementation selectors are gone from canonical source
[ ] general authored __implementation fields are gone from canonical source
[ ] standardized dunder protocols still work unchanged
[ ] @native/@private/@protected/@internal are no longer semantic authorities
[ ] Universe sources are migrated according to caller-aware census
[ ] semantic/direct/cached/dynamic/reflection/native paths agree on access
[ ] LSP/tooling presents the same visibility/implementation facts
[ ] authoritative specs/decision record describe the new model
[ ] focused certification passes or any baseline failures are explicitly classified
```

---

## 30. Final executor reminder

The difficult part of LANG006.C1.P1 is **not** adding seven tokens.

The difficult part is removing multiple competing visibility authorities without breaking bootstrap, native invocation, dynamic dispatch, trait surfaces, module/package ownership, or dunder protocols.

The required authority flow is:

```text
source modifiers
    ↓ normalize once
shared Visibility / ImplementationKind
    ↓
canonical semantic declaration + module/package/project provenance
    ↓
compiler/native binding metadata
    ↓
runtime compact access origin
    ↓
one authorization relation on every invocation path
```

If implementation work starts recreating visibility from spelling, source paths, attribute names, or Universe privilege bits, it has departed from the plan and must stop.
