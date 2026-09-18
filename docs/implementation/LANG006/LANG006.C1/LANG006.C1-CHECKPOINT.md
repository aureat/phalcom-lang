---
id: LANG006.C1
category: LANG
program: LANG006
checkpoint: LANG006.C1
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
requires:
  - TDR-0076 transition-1 language surface convergence
  - MODL001 universe/project/module architecture
  - canonical native Universe bootstrap infrastructure
active_plan: LANG006.C1.P2
latest_completed_plan: LANG006.C1.P1
next_plan: null
planning_baseline_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
planning_baseline_commit: "feat(lang005): continue P5 iterator surface"
prepared: 2026-09-17
repository: aureat/phalcom-lang
---

# Checkpoint Record — LANG006.C1 Declaration Modifiers, Access Domains, Native/Foreign Boundaries, and Universe Surface Normalization

## 1. Checkpoint purpose and authority

`LANG006.C1` is the first checkpoint of the new `LANG006` language-change package. It standardizes declaration modifiers and access control so Phalcom no longer encodes ordinary visibility or implementation origin through Universe-specific identifier spellings and compiler privilege conventions.

This checkpoint is the planning authority for the active corrective plan
`LANG006.C1.P2`; the P1 record remains the historical predecessor.

The original planning sequence was intentionally split into three artifacts:

```text
LANG006.C1-CHECKPOINT.md
    ↓
LANG006.C1.P1 header
    ↓
LANG006.C1.P1 task body
```

This checkpoint records the ratified language decisions, the repository takeover state, the required semantic invariants, the migration boundary, the affected implementation surfaces, and the constraints that the later P1 header/body must preserve.

The package's central rule is:

> **Names identify declarations. Language keywords state declaration semantics. Built-in attributes carry metadata. Neither visibility nor implementation origin is encoded in selector or field spelling.**

The canonical target therefore looks like:

```phalcom
native class Buffer {
  private _storage: Bytes
  module _cursor: Int

  internal native rawCapacity -> Int

  package normalizeIndex(_ index: Int) -> Int {
    ...
  }

  at(_ index: Int) -> Byte {
    ...
  }
}
```

rather than:

```phalcom
@native
class Buffer {
  __storage
  _$rawCapacity
  ...
}
```

The checkpoint does **not** attempt to make Universe completely ordinary. Universe remains a builtin Project with genuine bootstrap responsibilities. C1 instead removes unnecessary special language semantics from Universe code.

The governing architectural sentence is:

> **Universe may retain special bootstrap provenance, but ordinary visibility, selector identity, field identity, typing, and native declaration syntax must obey the same language rules as other Phalcom Projects.**

---

## 2. Ratified surface changes

The following decisions are fixed inputs to planning.

### 2.1 New keywords

Phalcom gains these reserved keywords:

```text
native
internal
private
protected
module
package
foreign
```

There is **no `public` keyword**.

`public` remains an internal semantic visibility state only.

### 2.2 New built-in attributes

Phalcom gains these built-in attributes:

```text
@intrinsic
@primitive
```

They are information-carrying metadata only.

They do not, merely by being present:

- change visibility;
- change selector identity;
- change field identity;
- make a declaration `native`;
- make a declaration `foreign`;
- allocate a different runtime representation;
- install a primitive function;
- emit a dedicated bytecode;
- force intrinsic lowering;
- alter typechecking;
- grant access-control privilege.

Their authored use is legal only in the canonical Universe Project. Use outside Universe is a compile-time error.

### 2.3 Default visibility

The absence of a visibility modifier means:

```text
Public
```

There is no alternate spelling.

Thus:

```phalcom
foo() { ... }
```

and:

```phalcom
native foo() -> Int
```

are public declarations.

The language does not accept:

```phalcom
public foo()
```

as visibility syntax.

### 2.4 Visibility keywords

The explicit restrictive visibility forms are:

```text
private
protected
module
package
internal
```

### 2.5 Legacy implementation selectors

The authored `_$name` implementation-selector namespace is retired.

Current internal methods must be classified and migrated to ordinary selector names plus explicit declaration modifiers.

Example:

```phalcom
@internal @native _$byteAt(_ index: Int) -> Option<Int>
```

becomes conceptually:

```phalcom
internal native byteAt(_ index: Int) -> Option<Int>
```

The exact replacement name is an API/implementation naming decision and must not be performed by blind prefix stripping.

### 2.6 Legacy implementation fields

The general `__name` implementation-field convention is retired.

A field such as:

```phalcom
__internalField
```

becomes an ordinary `_field` with explicit visibility:

```phalcom
internal _internalField
```

or another restrictive visibility selected from its actual use.

The migration is semantic, not textual.

### 2.7 Dunder protocols

Standardized dunder protocol identifiers such as `__name__` remain unchanged.

Dunder protocol redesign is explicitly outside C1.

C1 must distinguish:

```text
legacy implementation storage spelling
    from
standardized/reserved dunder protocol spelling
```

and must not delete valid protocol behavior while removing general implementation-field semantics.

### 2.8 Universe detection

Existing Universe detection remains available.

In particular, the implementation must preserve the existing `is_universe` state/check rather than deleting or folding it into new access-control semantics.

The new rule is:

```text
is_universe
    !=
internal visibility authority
```

Universe provenance remains useful independently of member visibility.

---

## 3. C1 semantic model

C1 separates four concepts that are currently partially entangled:

```text
selector / field identity
visibility
implementation kind
metadata classification
```

These become independent axes.

### 3.1 Identity

An ordinary callable selector remains an ordinary selector regardless of visibility or implementation kind.

For example:

```phalcom
internal native byteAt(_ index: Int) -> Option<Int>
```

has selector identity:

```text
byteAt(_)
```

not:

```text
_$byteAt(_)
```

Visibility is not part of selector identity.

Native/foreign implementation kind is not part of selector identity.

Similarly, a field:

```phalcom
internal _state
```

has ordinary field identity `_state`; `internal` is access metadata.

### 3.2 Visibility

The C1 member visibility set is:

```text
Public
Private
Protected
Module
Package
Internal
```

There is no surface `Public` modifier.

### 3.3 Implementation kind

The normalized implementation-kind model is conceptually:

```text
Phalcom
Native
Foreign
```

A declaration with no implementation-kind keyword is `Phalcom`.

### 3.4 Metadata classification

`@primitive` and `@intrinsic` are metadata independent of visibility and implementation kind.

A declaration may therefore have orthogonal facts such as:

```text
visibility = Internal
implementation = Native
primitive = true
intrinsic = true
```

without any one fact being inferred from another.

---

## 4. Visibility semantics

The intended access-control relations are identity-based and lexical, not spelling-based.

| Visibility | Accessibility |
|---|---|
| default / `Public` | unrestricted after normal name/import resolution |
| `private` | declaring lexical class |
| `protected` | declaring class and eligible subclasses |
| `module` | same exact defining Module |
| `package` | same exact defining Package |
| `internal` | same internal Project/domain |

`Protected` is inheritance-oriented and therefore is not simply another point in a linear visibility lattice.

### 4.1 Private

`private` retains the existing lexical-class rule.

```phalcom
private helper() { ... }
```

is accessible only from code carrying the declaration owner's lexical class authority.

Nested closures must retain the defining member's lexical authority.

The rule must apply consistently to:

- direct sends;
- class-side sends;
- cached sends;
- bound methods;
- callable references;
- reflection;
- dynamic invocation;
- method-family capture;
- native implementations acting for the declaration.

### 4.2 Protected

`protected` retains the current subclass-based relation.

The relevant question is the lexical calling class, not whether the caller happens to be in the same package or project.

C1 must not make `protected` implicitly `internal`.

### 4.3 Module

`module` means:

> The caller's defining `ModuleId` is exactly the declaration's defining `ModuleId`.

This is primarily a **member/field visibility** rule.

C1 does not introduce a second top-level declaration-visibility system. Top-level declarations remain exposed or hidden through the existing module import/export model.

Thus:

```phalcom
module helper()
```

is useful for cross-class implementation collaboration inside one source module, while ordinary unexported top-level declarations remain module-private through namespace/export semantics.

### 4.4 Package

`package` means:

> The caller and member belong to the same exact Package identity.

This must be identity equality, not string or path-prefix matching.

The rule is conceptually:

```text
caller.package_id == declaration.package_id
```

A nested child package is a distinct Package.

C1 does not introduce "package family", package-descendant, or Java-like recursive package semantics.

### 4.5 Internal

`internal` means:

> The caller and declaration belong to the same internal Project/domain.

For Project-owned source:

```text
caller.project_id == declaration.project_id
```

is the core relation.

This makes `internal` a normal language capability for any Phalcom Project, not a Universe privilege.

---

## 5. Internal domains outside normal Projects

Phalcom already supports source that does not belong to a normal resolved Project.

C1 therefore must not define `internal` as "same ProjectId or unusable".

The implementation requires an internal-domain abstraction equivalent to:

```rust
enum InternalDomainId {
    Project(ProjectId),
    StandalonePackage(PackageId),
    StandaloneModule(ModuleId),
    Synthetic(...),
}
```

The exact type and crate placement are planning details, but semantics must support:

```text
Project-owned source
    internal spans the Project

Standalone Package
    internal spans the standalone Package

Standalone Module
    internal collapses to that Module

Synthetic/REPL source
    internal follows the synthetic ownership domain
```

This keeps access control structural and deterministic without inventing Universe exceptions.

---

## 6. Default public fields are a real migration

C1's default-public rule applies to visibility-bearing members, including fields.

That is not equivalent to the current implementation.

The current semantic checker contains logic that treats fields as private by default. C1 changes the source rule.

Therefore every existing `_field` in the affected source corpus must be audited.

Where current behavior must remain private, migration must make it explicit:

```phalcom
private _state
```

Where collaboration requires a broader implementation boundary, use:

```phalcom
module _state
package _state
internal _state
```

The plan must not assume every current field should become public merely because public is now the default.

The migration must preserve intended encapsulation through explicit modifiers.

---

## 7. New declaration modifier grammar

C1 should normalize modifiers structurally in the AST rather than preserving the current "attributes plus string-prefix inference" architecture.

The target grammar is conceptually:

```text
visibility-modifier :=
    "private"
  | "protected"
  | "module"
  | "package"
  | "internal"

implementation-modifier :=
    "native"
  | "foreign"

member-declaration :=
    attribute*
    visibility-modifier?
    implementation-modifier?
    member-core
```

The canonical source order is:

```text
attributes
visibility
implementation kind
declaration
```

Example:

```phalcom
@primitive
@intrinsic
internal native suspend(_ value: Dynamic) -> Dynamic
```

C1 should not introduce `public`.

### 7.1 Modifier multiplicity

A declaration may contain at most one visibility modifier.

Reject:

```phalcom
private protected foo()
module package foo()
internal private foo()
```

A declaration may contain at most one implementation-kind modifier.

Reject:

```phalcom
native foreign foo()
foreign native foo()
```

### 7.2 Canonical order

The parser/formatter should converge on:

```phalcom
internal native foo()
```

rather than:

```phalcom
native internal foo()
```

The P1 header may decide whether noncanonical order is a parser error or accepted-and-normalized transition syntax, but the final language surface has one canonical order.

### 7.3 Top-level exposure remains export-controlled

C1 does **not** use:

```phalcom
internal class Foo
package class Foo
module class Foo
```

as an alternative namespace/export model.

Top-level exposure continues through import/export/re-export semantics.

Implementation kind is different: `native class Foo` remains a meaningful class declaration because `native` describes implementation origin, not namespace visibility.

---

## 8. AST normalization requirements

The current AST carries member attributes and several booleans but does not expose one canonical declaration-modifier structure.

C1 must create canonical structured metadata.

The exact Rust shape is a P1 design detail, but it must be equivalent to:

```rust
struct DeclarationModifiers {
    visibility: Visibility,
    implementation: ImplementationKind,
    visibility_range: Option<SourceRange>,
    implementation_range: Option<SourceRange>,
}
```

or normalized fields directly on declaration nodes.

The important invariants are:

1. downstream consumers read visibility from structured AST/semantic data;
2. downstream consumers read implementation kind from structured data;
3. no downstream subsystem scans `@private`, `@protected`, `@internal`, `@native`;
4. no downstream subsystem uses `starts_with("_$")` or general `starts_with("__")` to infer access;
5. source ranges remain available for precise diagnostics.

Where dependency direction permits, the visibility and implementation-kind enums should be shared from a neutral crate.

If crate boundaries require mirrored enums, conversions must be exhaustive and tested. Stringly-typed conversion is not acceptable.

---

## 9. Native declarations

`native` becomes a true language keyword describing implementation kind.

Normative meaning:

> A native declaration is a Phalcom declaration whose implementation or runtime representation is supplied by a Phalcom native implementation provider rather than by the declaration's ordinary Phalcom body.

### 9.1 Native is not visibility

All of these are conceptually distinct:

```phalcom
native foo() -> Int
private native foo() -> Int
protected native foo() -> Int
module native foo() -> Int
package native foo() -> Int
internal native foo() -> Int
```

Native implementation code must not receive blanket internal authority.

### 9.2 Native callables are declaration-backed

For a callable:

```phalcom
native decode(_ bytes: Bytes) -> Image
```

the source signature remains the Phalcom semantic contract.

The declaration participates in:

- typechecking;
- callable identity;
- selector formation;
- reflection;
- generic substitution;
- trait/impl relationships where otherwise legal;
- static member lookup.

The native provider supplies execution.

### 9.3 No fake executable native bodies

C1 should converge native callables on declaration-only source form.

A body that looks executable but is guaranteed not to execute is not a stable language model.

Therefore the target source is:

```phalcom
native decode(_ bytes: Bytes) -> Image
```

not:

```phalcom
native decode(_ bytes: Bytes) -> Image {
  ...
}
```

Existing historical `@native` reference-bodied declarations must be migrated deliberately.

### 9.4 Native classes/enums

C1 must preserve native-backed canonical Universe identities and allow the language to express implementation-backed type declarations such as:

```phalcom
native class String is Object {
  ...
}
```

and native enums where current bootstrap requires them.

A native class does not make every body inside it native.

Example:

```phalcom
native class String is Object {
  internal native byteCount -> Int

  size -> Int {
    byteCount
  }
}
```

`size` is ordinary Phalcom implementation.

### 9.5 Native outside Universe

The keyword `native` is **not Universe-only**.

It is intended as a general Phalcom declaration mechanism for mixed Phalcom/native projects.

C1 must therefore avoid hard-coding:

```text
native declaration => Universe
```

even if the existing native provider infrastructure is currently Universe-centric.

The complete packaging/ABI/loading model for third-party native extensions does not have to be solved by C1, but C1's AST, semantic identity, and descriptor architecture must not prevent it.

---

## 10. Foreign declarations

`foreign` becomes a separate implementation-kind keyword.

Normative distinction:

```text
native
    Phalcom owns the declaration;
    native extension/provider code supplies implementation.

foreign
    the referenced declaration/entity originates outside Phalcom;
    Phalcom describes an interoperability boundary.
```

Native and foreign are mutually exclusive.

### 10.1 C1 foreign scope

C1 owns:

- token/keyword;
- parser grammar;
- AST representation;
- `ImplementationKind::Foreign` or equivalent;
- modifier conflict validation;
- declaration-only behavior where applicable;
- semantic/source-index/reflection representation;
- explicit unsupported/unbound lowering diagnostics where no FFI backend exists.

C1 does **not** own the complete FFI.

Specifically deferred:

```text
C ABI details
Rust ABI details
dynamic loader design
foreign struct layout
ownership transfer rules
lifetime rules
pointer capability model
callback ABI
marshaling policy
foreign exception/error ABI
unsafe boundary design
```

C1 must not invent these merely to make the keyword executable.

### 10.2 Foreign target matrix

The exact set of declaration forms initially permitted with `foreign` is a bounded P1-header decision.

The header must choose explicitly rather than letting parser accident define semantics.

At minimum, callable foreign declarations must have a normalized AST representation.

If foreign classes/types are not implemented in C1, the parser must reject them intentionally with a targeted diagnostic rather than partially constructing unusable semantics.

---

## 11. `@primitive` and `@intrinsic`

These are built-in informational attributes.

### 11.1 Metadata-only invariant

C1 must not make either attribute an execution directive.

In particular:

```text
@primitive
    does not itself install a primitive implementation

@intrinsic
    does not itself force specialized lowering
```

A backend or later compiler stage may consume the metadata, but absence/presence alone does not rewrite ordinary semantics in C1.

### 11.2 Universe-only provenance

Authored use is legal only in canonical Universe source.

The check must use real Universe identity.

It must not rely on:

- project display name;
- source path text;
- package spelling;
- module spelling.

A user project named `universe` must not gain access.

### 11.3 Preserve `is_universe`

C1 explicitly preserves the existing `is_universe` field/check used by compiler/bootstrap code.

The visibility migration must not delete it.

This gives two independent mechanisms:

```text
AccessContext.internal_domain
    answers ordinary language access

is_universe
    answers canonical Universe provenance
```

### 11.4 Attribute targets

C1 does not infer additional semantics from target choice.

The P1 header must lock an explicit target matrix for both attributes based on the live Universe source/metadata needs.

Whatever target set is selected, it must satisfy:

- no arguments unless separately specified;
- canonical Universe provenance required;
- metadata preserved in source/semantic products;
- no implicit native/foreign conversion;
- no implicit visibility conversion.

The plan must not introduce the earlier speculative rule "`@primitive` requires `native`" unless a later ratified design explicitly adds it.

---

## 12. Retired attributes

The following historical forms cease to be the source of declaration semantics:

```text
@native
@private
@protected
@internal
```

They migrate to keywords:

```text
@native      → native
@private     → private
@protected   → protected
@internal    → internal
```

C1 should not permanently support both systems.

A short repository-internal migration window may be used if needed to keep intermediate commits buildable, but the checkpoint endpoint has one source of truth.

Targeted transition diagnostics are preferred:

```text
`@native` is retired; write `native` before the declaration
`@private` is retired; write `private`
`@protected` is retired; write `protected`
`@internal` is retired; write `internal`
```

At checkpoint completion no downstream semantic consumer should derive visibility/implementation kind by scanning these attribute names.

---

## 13. Legacy `_$` selector migration

Current `_$` methods are not one semantic category.

Every occurrence must be classified before migration.

### 13.1 Native runtime floor

Current examples include low-level operations on classes such as:

```text
List
String
Map
Set
Tuple
Record
System
Fiber
Method/reflection objects
```

These typically become:

```phalcom
internal native foo(...)
```

or a narrower visibility based on actual call sites.

### 13.2 Source-defined implementation helper

Some `_$` members are implemented fully in `.ph`.

These become ordinary source methods/getters such as:

```phalcom
private foo(...)
module foo(...)
package foo(...)
internal foo(...)
```

according to actual access.

They must not become native solely because the old selector belonged to the implementation namespace.

### 13.3 Language/compiler protocol

Some current `_$` selectors function as compiler/runtime protocols rather than ordinary implementation helpers.

These require individual classification.

They must not automatically become public or internal ordinary methods if their real role belongs to the reserved protocol system.

Dunder protocol redesign itself remains outside C1.

### 13.4 Synthetic VM/compiler seam

Some implementation selectors may exist only to enable compiler-generated runtime behavior.

C1 may retain a synthetic runtime mechanism if necessary, but authored `_$` syntax must not survive just to preserve that implementation path.

Prefer typed compiler/runtime metadata or direct bytecode/runtime entry points for genuinely synthetic operations.

---

## 14. Legacy `__field` migration

General implementation-field syntax must be audited similarly.

The repository currently has explicit lexical/AST distinctions for implementation fields.

C1's endpoint is one authored field namespace:

```text
_field
```

plus explicit visibility.

Examples:

```phalcom
private _state
module _cursor
package _cache
internal _nativeHandle
```

The migration must **not** interpret every `__name` uniformly because true dunder protocols remain reserved and out of scope.

The lexer/parser plan must therefore preserve the ability to distinguish:

```text
legacy implementation field
standardized dunder protocol
```

until migration is complete.

---

## 15. Current repository takeover state

Planning is grounded against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   e0a77646c56f8b1a7e7692993d378d43bf3fc424
commit:     feat(lang005): continue P5 iterator surface
prepared:   2026-09-17
```

The repository remains under active LANG005 development. The P1 plan must begin with a fresh baseline check and adapt mechanical drift without changing C1 architecture.

Verified current facts include the following.

### 15.1 Lexer/AST namespaces are explicit

`phalcom-ast` currently distinguishes:

```text
Identifier
FieldIdentifier
ImplementationFieldIdentifier
ImplementationSelectorIdentifier
```

and the AST still contains implementation-specific expression/field representation.

Therefore C1 is not merely a semantic rename. It changes lexical and AST structure.

### 15.2 Semantic visibility is incomplete for C1

`phalcom-semantic/src/surface.rs` currently models:

```text
Public
Private
Protected
Internal
```

There are no `Module` or `Package` visibility cases.

The current comment still describes Internal as core/privileged visibility.

### 15.3 Runtime visibility is likewise incomplete

`phalcom-core/src/method/mod.rs` has the same four-case model.

Runtime send authorization currently treats `Internal` through caller privilege rather than a Project/domain identity relation.

### 15.4 Visibility is derived in multiple places

Current code derives access from a mix of:

```text
selector prefix `_$`
field category
@private
@protected
@internal
native metadata visibility
```

This logic appears across:

```text
phalcom-core compiler lowering
phalcom-semantic declaration checking
trait/property handling
delegation handling
editor access checks
native source indexing
native installation
primitive registration
generated core tables
```

C1 must converge these onto one semantic source of truth.

### 15.5 Native contexts currently receive broad internal privilege

`NativeMethodContext` currently carries:

```text
access_owner
internal: bool
```

and native invocation paths may enter with internal authority.

This cannot represent module/package/project visibility correctly.

Native execution must instead receive declaration-derived access origin/domain.

### 15.6 Native metadata visibility is too narrow

Current native metadata distinguishes roughly:

```text
Public
Internal
```

and some conversion code collapses multiple language visibilities into native-internal metadata.

C1 must preserve exact visibility.

### 15.7 Canonical native source indexing is Universe-centric

The current native source index uses canonical Universe identity and `UniverseKey` heavily.

This remains valid for primordial bootstrap identity, but the general language meaning of `native` must move toward project/declaration-qualified identity.

### 15.8 Universe identity checks are already structural

The module/project system already has identity-based Universe checks such as `ProjectIdentity::is_universe()`.

C1 should reuse structural identity rather than adding textual Universe detection.

### 15.9 Public is already a semantic default in some surfaces

Semantic `MemberVisibility` already derives `Default` as Public and member-surface helpers default to Public.

However field-specific source checking currently contains private-default behavior.

The migration therefore must reconcile inconsistent current defaults rather than assuming one global implementation behavior already exists.

---

## 16. Access-origin architecture

C1 needs a declaration/caller origin model capable of evaluating all visibility cases without text parsing.

Conceptually, caller authority needs:

```rust
struct AccessContext {
    lexical_class: Option<ClassId>,
    module: ModuleId,
    package: Option<PackageId>,
    internal_domain: InternalDomainId,
    is_universe: bool,
}
```

Declaration/member access metadata needs enough origin information to evaluate:

```text
private
protected
module
package
internal
```

The exact storage may be normalized handles/IDs rather than the literal structure above.

### 16.1 Authorization relation

The semantic relation is:

```text
Public
    true

Private
    caller.lexical_class == member.lexical_class

Protected
    caller lexical class is owner or eligible subclass

Module
    caller.module == member.module

Package
    caller.package == member.package

Internal
    caller.internal_domain == member.internal_domain
```

### 16.2 Hot-path requirement

Runtime authorization participates in dispatch.

Do not implement access checks by:

- reparsing module names;
- comparing path strings;
- resolving package ancestry on every send;
- consulting source text;
- rescanning attributes.

Use canonical IDs or interned ownership handles.

---

## 17. Native execution authority

Native code has no bytecode frame, so existing native execution context remains the correct architectural seam.

C1 should evolve it rather than bypass it.

A native implementation must execute with authority derived from the Phalcom declaration it implements.

It must **not**:

- inherit arbitrary caller private authority;
- receive blanket internal privilege;
- receive Universe privilege merely because the implementation is native;
- bypass module/package/project restrictions.

Required cases include:

```text
native declaration in Project A
    may use A-internal implementation members

native declaration in Project A
    may not use Project B internal members

package native declaration
    receives its declaring package authority

private native declaration
    receives its declaring lexical-class authority
```

`is_universe` may also be carried where needed for genuine Universe-only metadata/protocol behavior, but it is not an access-control shortcut.

---

## 18. Native descriptor and provider normalization

C1 should preserve the strong parts of the existing native architecture:

```text
canonical source declaration
    ↔
native descriptor
    ↔
verified structural binding
```

The source declaration remains authoritative for the Phalcom-facing type/signature.

The provider metadata remains authoritative for implementation binding facts.

### 18.1 Required exact metadata

Native verification should be able to compare, where applicable:

```text
owner identity
dispatch side
selector
member kind
parameter shape
parameter types
return type
visibility
implementation kind
```

Visibility must be exact, not collapsed to Public/Internal.

### 18.2 General owner identity

The long-term general native key should be project/declaration qualified, conceptually:

```rust
NativeMemberKey {
    owner: DeclarationId,
    side: DispatchSide,
    selector: Selector,
}
```

`UniverseKey` remains valid as a primordial bootstrap adapter where required.

C1 need not delete `UniverseKey`, but it must not make new general native semantics depend on it.

### 18.3 Third-party native loading is not required

C1 does not have to implement the full extension loader/manifest/native ABI for arbitrary third-party projects.

It does have to avoid introducing syntax or semantic identity that makes such an extension system impossible.

---

## 19. Reflection and tooling model

The language/tooling representation should expose semantic facts, not legacy spelling.

### 19.1 Reflection

Where reflection exposes declaration metadata, C1 should represent:

```text
visibility:
    public
    private
    protected
    module
    package
    internal

implementationKind:
    phalcom
    native
    foreign
```

`@primitive` and `@intrinsic` may appear through normal built-in attribute metadata if that system exposes declaration attributes.

There is no "implicit public" versus "explicit public" distinction because explicit public syntax does not exist.

### 19.2 LSP

The LSP must understand the seven keywords and new visibility semantics.

Affected behavior includes:

```text
semantic tokens
completion filtering
hover
go-to-definition
references
rename
source index
diagnostics
code actions
native declaration navigation
```

Completion and navigation must use the same access-domain semantics as semantic typechecking.

### 19.3 Syntax tooling

Update repository syntax grammars and any generated tables that enumerate keywords/attributes/visibility.

Do not leave `_$` and implementation fields highlighted as active canonical syntax after C1 completion.

---

## 20. Keyword compatibility and lexical collision audit

Introducing seven true keywords is a breaking lexical change.

P1 must perform a repository-wide collision census for:

```text
native
internal
private
protected
module
package
foreign
```

Potential collision positions include:

```text
locals
parameters
labels
method names
getter names
imports
aliases
fields
pattern bindings
type parameters
```

The header must explicitly establish Phalcom's keyword-as-selector/property-name rule for these tokens.

The parser already has precedent for some keywords remaining usable in property-name contexts. C1 must make that intentional rather than accidental.

Regression requirement:

```text
public
```

must remain an ordinary identifier token, not a keyword.

---

## 21. Built-in attribute implementation requirements

`@primitive` and `@intrinsic` should be represented as genuine `BuiltinAttr` variants rather than magic user-attribute strings.

The attribute registry/parser must:

1. recognize them as built-ins;
2. preserve source ranges;
3. reject arguments unless the P1 header ratifies arguments;
4. enforce canonical Universe provenance;
5. preserve them long enough for semantic/source/tooling consumers;
6. avoid current subtractive behavior used for legacy `@native`;
7. not synthesize visibility or implementation kind.

A non-Universe usage should produce a dedicated compile-time diagnostic.

The check must operate on actual project identity.

---

## 22. Trait and impl interaction boundary

C1 must not casually reinterpret trait conformance semantics while expanding visibility.

Ordinary inherent `impl` members may use the new visibility model where current member semantics permit.

Abstract trait requirements require more care because:

```text
module
package
internal
```

are provenance-dependent access sets, not a universal total-order visibility ladder.

C1 should avoid defining cross-package conformance rules accidentally.

The P1 header must explicitly state which modifiers are legal on:

```text
trait requirements
trait defaults
inherent impl members
trait impl members
delegated accessors
associated declarations
```

The conservative default is:

- preserve current supported trait visibility semantics;
- do not invent package/project-scoped abstract requirements without an explicit semantic rule;
- allow normal visibility on concrete inherent implementation behavior.

This is a plan-header decision, not something Luna should improvise while coding.

---

## 23. Compiler-generated/internal dispatch audit

The VM currently contains compiler-internal dispatch machinery used for privileged synthetic sends.

Removal of authored `_$` selectors does not automatically prove this machinery obsolete.

P1 must audit each use and classify it:

```text
ordinary member access
    → use declaration access context

language/compiler protocol
    → retain narrowly scoped protocol authority

obsolete legacy implementation-selector bypass
    → delete

direct runtime operation
    → prefer typed/direct mechanism
```

The checkpoint requirement is:

> No user-authored `_$` selector namespace survives merely because existing compiler-generated sends rely on it.

Any retained compiler-only privilege must remain separate from ordinary `internal` visibility.

---

## 24. Diagnostics contract

C1 should add or migrate stable diagnostics around the new semantic concepts.

The exact repository codes may be refined by the P1 header, but coverage must include:

```text
retired @native
retired @private
retired @protected
retired @internal
retired _$ selector
retired implementation field

multiple visibility modifiers
conflicting implementation kinds
illegal modifier target
invalid modifier order

private access violation
protected access violation
module access violation
package access violation
internal access violation

native body not allowed
native binding missing
native visibility mismatch
native signature mismatch

foreign body/binding unsupported
native + foreign conflict

@primitive outside Universe
@intrinsic outside Universe
illegal @primitive target
illegal @intrinsic target
```

Diagnostics must report new semantics rather than preserving names such as `internal.selector_access` when no implementation-selector namespace remains.

---

## 25. Testing requirements

Testing must be selective but architectural.

The P1 plan is intended for Luna and should therefore favor narrow owner-focused test slices plus a final cross-stack certification pass, rather than broad workspace testing after every edit.

### 25.1 Lexer tests

Prove:

```text
native      => keyword
internal    => keyword
private     => keyword
protected   => keyword
module      => keyword
package     => keyword
foreign     => keyword

public      => ordinary identifier
```

Also cover:

```text
_field
legacy __field
dunder __name__
legacy _$selector
```

with precise migration behavior.

### 25.2 Parser/AST tests

Cover the canonical combinations:

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
```

Reject conflicting forms.

Verify normalized AST visibility/implementation metadata rather than testing downstream string inference.

### 25.3 Visibility tests

For each access scope, test relevant boundaries.

#### Module

```text
same class / same module
different class / same module
different module / same package
different package / same project
different project
```

#### Package

```text
same module
different module / same exact package
child package
parent package
sibling package
different project
```

#### Internal

```text
same module
same package
different package / same project
different project
Universe → Universe
ordinary project → Universe
Universe → ordinary project
```

Also cover standalone/synthetic ownership where existing harnesses support it.

### 25.4 Invocation-path parity

At least one focused matrix must verify access enforcement through:

```text
direct send
cached send
method reference
bound method
dynamic/reflection invocation
native method body
nested closure
class-side dispatch
```

The plan may use representative visibility cases rather than a Cartesian explosion, but every authorization path must have evidence.

### 25.5 Native authority tests

Required regression cases:

1. native code in Project A can call A-internal behavior;
2. it cannot call Project B-internal behavior;
3. package-native code is restricted to its package for package members;
4. private native methods preserve declaring-class authority;
5. native execution does not inherit unrelated caller authority;
6. native code does not receive blanket internal privilege.

### 25.6 Attribute tests

For both `@primitive` and `@intrinsic`:

```text
recognized as BuiltinAttr
accepted in canonical Universe
rejected outside Universe
rejected in fake project named "universe"
metadata retained
no visibility change
no implementation-kind change
no selector change
no automatic lowering change
```

### 25.7 Migration tests

Explicitly prove old spellings no longer determine semantics.

Examples:

```text
@internal
    does not remain the semantic visibility mechanism

_$foo
    does not automatically imply Internal

__storage
    does not silently retain implementation-field authority
```

---

## 26. Repository migration requirements

C1 includes a source migration, not only compiler infrastructure.

The migration must census at least:

```text
@native
@private
@protected
@internal
_$...
__... implementation fields
primitive registration macros
native visibility descriptors
compiler privilege paths
tests/spec fixtures using retired syntax
documentation/spec examples
syntax grammar files
generated tables
```

For every `_$` and implementation field, the migration record/task should identify:

```text
current owner
current spelling
implementation kind
actual callers
required visibility
new spelling
protocol/synthetic exception if any
```

Luna should not be asked to infer these ad hoc during search-and-replace.

---

## 27. Specification/decision work

C1 supersedes part of the accepted transition language model.

TDR-0076 currently treats:

```text
__name
_$name
```

as structural implementation namespaces and ties internal selector privilege to bootstrap-core identity.

C1 therefore requires a new accepted decision record or equivalent authoritative amendment.

Normative specifications affected include at least:

```text
lexical structure
classes/object model
fields
selectors
attributes
access control
native declarations
reflection
VM execution/access authority
Universe bootstrap
module/project ownership
```

Historical implementation plans should remain historical. The current specifications and decision indexes must identify the new authority clearly.

Dunder protocol rules remain untouched.

---

## 28. Non-goals

C1 does not own:

```text
a `public` keyword
dunder protocol redesign
friend declarations
package-family visibility
recursive package visibility
new top-level declaration export semantics
complete native extension manifest/loading ABI
complete FFI ABI
foreign struct layout
pointer ownership model
callback ABI
unsafe FFI model
runtime trait/conformance redesign
making visibility part of selector identity
making implementation kind part of selector identity
deleting `is_universe`
deleting `UniverseKey`
eliminating primordial bootstrap special cases
making @primitive install a primitive
making @intrinsic force intrinsic lowering
```

These boundaries are architectural.

---

## 29. Critical correctness invariants

The P1 header/body must repeat and enforce these.

### C1-I1 — No public keyword

```text
Public is semantic default.
`public` is not declaration syntax.
```

### C1-I2 — One visibility truth

A declaration's visibility is normalized once.

No consumer re-derives it from:

```text
_$ prefix
__ prefix
legacy visibility attributes
native descriptor heuristics
```

### C1-I3 — Selector identity is ordinary

```text
foo(_)
```

is the selector regardless of visibility/implementation kind.

### C1-I4 — Field identity is ordinary

```text
_field
```

is the authored field namespace regardless of visibility.

### C1-I5 — Access domains are identity-based

```text
module   => exact Module identity
package  => exact Package identity
internal => exact internal-domain identity
```

### C1-I6 — Native is not privileged

Native execution receives declaration-derived access authority only.

### C1-I7 — Universe provenance is independent

`is_universe` remains available and meaningful.

It is not the implementation of ordinary `internal`.

### C1-I8 — Primitive/intrinsic attributes are metadata

They do not independently alter execution semantics.

### C1-I9 — Primitive/intrinsic authored use is Universe-only

The legality check uses canonical project identity.

### C1-I10 — Dunder protocols remain stable

C1 must not accidentally remove or reinterpret standardized `__name__` protocols.

### C1-I11 — Foreign is distinct from native

The semantic model must not collapse foreign declarations into native declarations merely because both lack ordinary Phalcom bodies.

### C1-I12 — Export semantics stay orthogonal

Member visibility does not replace top-level module exports.

---

## 30. Principal risks

### 30.1 Cross-consumer visibility drift

The repository currently computes visibility in several subsystems.

A partial migration could produce:

```text
typechecks but fails at runtime
hidden in completion but callable dynamically
works directly but fails through method reference
works from Phalcom but fails in native implementation
source says one visibility while descriptor installs another
```

P1 must therefore contain explicit convergence gates.

### 30.2 Native blanket privilege dependency

Existing native methods may rely on current broad internal privilege without declaring that dependency.

Changing native authority can reveal real hidden coupling.

The Universe migration must identify and correctly re-scope these calls.

### 30.3 Field default compatibility

Existing fields may rely on implicit private behavior.

Changing the language default to Public requires explicit migration of intended encapsulation.

### 30.4 Dunder overlap

Legacy implementation-field syntax and standardized dunder protocols share double-underscore territory.

A lexical simplification that removes both would be incorrect.

### 30.5 Keyword collisions

Seven new reserved words may currently be used as identifiers/selectors/labels in tests, Universe source, examples, or user-facing specs.

The plan must audit before final parser tightening.

### 30.6 Over-expanding FFI

`foreign` is a required keyword, but C1 must not become an accidental full FFI redesign.

The plan must implement the semantic category cleanly while stopping at explicit unsupported/unbound boundaries.

### 30.7 Over-generalizing native loading

The native keyword is intended to be general, but the current runtime provider is Universe-centric.

C1 should normalize identities and semantics without inventing an entire extension distribution system.

---

## 31. P1 planning contract

`LANG006.C1.P1` will be produced as two coordinated files:

```text
P1 header
    complete shared authority, repository facts, invariants,
    file map, test strategy, decision matrix, execution rules

P1 body
    ordered patch-grade tasks with concrete files,
    edits, tests, gates, and stop/consult conditions
```

The P1 body must be executable by Luna without needing to rediscover the architectural model.

The header must therefore include:

```text
checkpoint authority
baseline commit and freshness procedure
ratified syntax/semantics
modifier/target legality matrix
visibility access matrix
native/foreign semantics
Universe-only attribute rules
exact repository seams
legacy migration classification rules
diagnostic policy
test command strategy
non-goals
adviser/consultation triggers
```

The body must then reference the header rather than duplicating the entire design.

---

## 32. Bounded decisions still to lock in the P1 header

The requirements are sufficiently stable to plan, but the next analysis pass must lock a few implementation-level choices before Luna tasks are written.

These are not open language-direction questions; they are bounded design decisions.

### 32.1 `@primitive` / `@intrinsic` target matrix

Need explicit legality across:

```text
class
enum
method
getter
setter
constructor/factory
index member
trait requirement/default
impl member
field
```

Requirements already fixed:

```text
Universe-only
metadata-only
no arguments unless explicitly ratified
no implicit native/foreign/visibility semantics
```

### 32.2 Initial `foreign` target matrix

Need decide whether C1 enables only callable foreign declarations or also class/enum/type forms.

Whatever is not implemented must fail intentionally.

### 32.3 Modifier order enforcement

Need decide:

```text
strict parser order
    or
transitional parser accepts permutations but canonicalizes
```

Final canonical order remains visibility before implementation kind.

### 32.4 Shared enum placement

Need resolve whether `Visibility` and `ImplementationKind` live in `phalcom-common` or use explicit mirrored AST/semantic/runtime enums due crate dependency constraints.

No string-based model is permitted.

### 32.5 Runtime access-origin representation

Need choose minimal hot-path representation:

```text
direct IDs on MethodObject
shared access-origin handle
module object handle + derived ownership
other canonical compact representation
```

Semantics are already fixed.

### 32.6 Keyword-as-selector behavior

Need explicitly define which new keyword tokens are accepted in property/selector positions, following existing parser policy.

This prevents accidental source breakage beyond intentional reservation.

---

## 33. Expected completion state

At C1 completion, canonical Phalcom should support source such as:

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

with these semantics:

```text
Buffer
    native-backed class identity

_storage
    ordinary field
    private

_cursor
    ordinary field
    exact-module visibility

_scratch
    ordinary field
    exact-package visibility

rawCapacity
    ordinary selector
    internal Project/domain visibility
    native implementation
    primitive metadata
    intrinsic metadata
    @primitive/@intrinsic legal because this declaration is in canonical Universe

normalizeIndex
    ordinary Phalcom method
    package visibility

at
    ordinary public Phalcom method
    public because no visibility modifier exists
```

And these historical mechanisms are no longer semantic authorities:

```text
@native
@private
@protected
@internal
_$selector
general __implementationField
blanket native internal privilege
Universe-only Internal visibility
```

while these remain available:

```text
standardized dunder protocols
canonical is_universe detection
Universe bootstrap identity
UniverseKey where primordial identity still requires it
module/package/project ownership model
existing import/export semantics
```

That is the checkpoint boundary the P1 header and task body must implement.

---

## 34. Implementation state

`LANG006.C1.P1` is implemented in the shared working tree at base revision
`bf4e12ba` (the implementation was intentionally not committed or pushed in
this turn). The canonical source, AST/semantic/runtime access path, native
metadata, LSP presentation, syntax grammar, current specification pointer, and
accepted decision record now agree on the C1 model.

The plan is classified `IMPLEMENTED` with `FOCUSED_TESTED` verification:
focused owner checks and workspace compilation pass, while one unrelated
semantic incremental identity test and one core VM walk test remain failing in
the current shared checkout. The repository-wide formatter check also
reports pre-existing formatting drift outside this work unit; no broad
formatting rewrite was applied.

### Plan ledger

| Plan | Status | Completion | Verification | Durable state |
| --- | --- | --- | --- | --- |
| `LANG006.C1.P1` | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Canonical modifiers/access domains, native authority, Universe migration, AST retirement, tooling/spec/decision parity |
| `LANG006.C1.P2` | IN_PROGRESS | PARTIAL | BASELINE_BLOCKED | Exact access provenance transport, selected-call authorization, source/native runtime origins, canonical native conformance, kind/provenance split, Foreign boundary, and focused parity evidence |

### Durable residuals

- `phalcom-semantic` incremental caller identity assertion remains a baseline
  failure outside the changed modifier/native surfaces.
- `phalcom-core` VM walk ordering test remains a baseline runtime/iterator
  failure (`fold(initial,using)` does not understand the canonical runtime
  selector in that test).
- Full package/workspace test certification and repository-wide formatting are
  deferred; focused checks and `cargo check --workspace --all-targets` pass.

The next action is checkpoint-level certification or a separately scoped
follow-up for either baseline failure; no additional C1 implementation plan is
required for the delivered vertical.

It should freeze all shared execution information Luna will need so the final turn can consist almost entirely of ordered, patch-grade implementation tasks.

## 35. Corrective closure state — LANG006.C1.P2

The post-P1 audit reopened the checkpoint at revision
`9f1d2c9fa98e94edc6088aee1834520361e08d3e`. P2 now has a working corrective
vertical for exact access provenance, selected-call authorization, runtime
declaration origin, native conformance, implementation-kind separation,
bounded `Foreign`, and tooling/specification parity.

Durable interfaces established in the current worktree:

- `phalcom-semantic::access` publishes declaration and caller module/package/
  project provenance from canonical module topology; editor and checker paths
  share the same access relation and no longer reconstruct package identity
  from paths.
- Formal dispatch authorizes the selected callable after lookup, preserving the
  distinction between inaccessible and missing members.
- Source compiler method constructors stamp exact module-derived runtime origin;
  native anchors carry their declaring `ModuleId`, and bootstrap refreshes
  installed methods after source modules materialize.
- Native/source contracts compare canonical owner/side/selector/generics,
  parameter shape/types, and return types through semantic products. The
  comparison treats erased/dynamic metadata as non-conflicting and preserves
  receiver-flow and setter semantics explicitly.
- `phalcom-common::ImplementationKind` is the language category; the old
  native-meta axis is now `ImplementationProvenance`. `Foreign` is retained
  through semantic/presentation products and fails at an explicit unsupported
  binding boundary.

Focused evidence currently passes: semantic access-control (8), native
conformance (3), native signature mismatch unit coverage, semantic presentation
(6), core native unit tests (10), core reflection conformance (3), explicit
Foreign binding failure, AST modifier regressions (2), LSP hover-filtered
coverage, native-meta/surface/decl tests, both owning-package compile checks,
and the exact Universe bootstrap measurement test. The source declaration
specification and keyword hover/token surfaces were updated with the accepted
visibility-before-implementation rule.

P2 remains `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`: focused P2 evidence
passes, but the full semantic suite ended `1420 passed, 22 failed, 50 ignored`
and the full core suite ended `537 passed, 74 failed, 28 ignored`. The exact
compiler declaration-dispatch failures for iterator syntax, retired private
syntax, and bootstrap `Class.allocate()` access reproduce on the pushed P1
revision. A clean-P1 semantic comparison also reproduces the source-visibility
surface failure. The remaining broad failures are carried ADT, iterator,
incremental, conformance, module, and runtime-baseline failures; no P2-specific
access/native conformance regression was observed. No checkpoint completion
claim is made until the remaining P2 coverage is certified against those
baselines.
