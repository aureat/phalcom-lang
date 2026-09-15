# LANG005.C2.P1 — First-Class Inherent `impl` and Effective Declaration Surfaces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce first-class inherent `impl` blocks for Phalcom nominal types and replace the “primary declaration owns the whole behavioral surface” assumption with a canonical declared-surface → inherent-contribution → effective-surface pipeline, without creating open classes, storage extensions, conditional method availability, or trait semantics.

**Architecture:** `impl` is a source declaration fragment, not a namespace binding and not a runtime reopen. Each block receives stable semantic provenance (`ImplId`), resolves to one same-module nominal target, publishes behavior-only member signatures as an `InherentImplContribution`, and is merged deterministically with the target’s declared members into the one effective `DeclarationSurface` consumed by lookup, body analysis, compiler lowering, source tooling, and later trait work. Callable identity belongs to the target declaration (`CallableId`); `ImplId` records provenance, generic binders, diagnostics, incrementality, and future reflection. P1 accepts only unconditional/covering inherent impls; receiver-specialized heads and `where`-conditioned applicability are reserved for C2.P3.

**Tech Stack:** Rust 2024 workspace; `phalcom-ast`; `phalcom-modules`; `phalcom-semantic` incremental DB, declaration surfaces, callable analysis, associated lookup, source index; `phalcom-core` direct AST-to-bytecode compiler and runtime class/enum/data behavior objects; `phalcom-lsp`; existing generic signature/type-parameter machinery.

**Spec / governing decisions:** TDR-0081; LANG005 program decisions; LANG005.C1 completed product/value architecture; current semantic-authority specifications. This plan is the C2.P1 implementation contract. C2.P2 owns variants-only enum syntax and migration of closed enum root/case behavior into `impl`; C2.P3 owns constrained/specialized inherent impl applicability.

**Repository grounding:** Prepared against `aureat/phalcom-lang` `main` at `58f4828dbd9711432edca034afca1fbac72a1640` on 2026-09-13. Re-run the drift protocol before implementation; mechanical names may move, but the semantic boundaries below do not.

## Global Constraints

- `impl T` is inherent behavior belonging to `T`, not an open-class mutation.
- An inherent impl is legal only in the module that canonically owns the target declaration in C2.P1.
- `impl` never introduces a module namespace binding and is never importable/exportable by its own name.
- `impl` may add behavior only; it may not add fields, data components, enum variants, superclass edges, representation metadata, or other storage/layout state.
- Callable identity from an inherent impl belongs to the target declaration, never to the impl block.
- `ImplId` is provenance/ownership identity, not dispatch identity.
- Existing selector/member identity remains shape-based: selector kind + base + arity/labels/rest shape + dispatch side. Parameter and return types do not overload a selector.
- There is no last-wins behavior. A conflicting concrete member is a source error; invalid duplicates are not installed at runtime.
- Instance and class-side surfaces remain distinct.
- Data components remain `DataComponentId`s, not synthetic fields/getters, even when they reserve a property selector against an impl member.
- Enum variant constructors remain variant/associated-callable identities, not class-side method identities, even when they reserve an associated selector against an impl member.
- C2.P1 accepts only unconditional nominal impls and covering generic impls. `where`-conditioned impls, repeated/concrete/specialized target arguments, exact enum-case targets, and other receiver-conditional heads are rejected for C2.P3.
- `impl Trait for T` is not parsed as a valid inherent impl in C2.P1; trait conformance belongs to LANG005.C4.
- Declaration-only/bodyless impl behavior is rejected in C2.P1. C2.P2 will assign bodyless enum-root impl members their closed-case-requirement meaning.
- Special constructor declarations (`@constructor` / constructor semantic markers) are rejected inside C2.P1 impls. Ordinary behavior selectors are allowed; special constructor lifecycle integration is not silently invented here.
- Primary data product construction remains distinct from behavior added through `impl`.
- Primary enum variant construction remains distinct from behavior added through `impl`.
- Runtime compilation must consume semantically accepted impl contributions. It must not synthesize a `ClassDef` reopen from an impl and call the existing class-reopen path.
- No impl policy is reconstructed from imports or runtime load order.
- No semantic result depends on source ordering between a target declaration and its same-module impl fragments.
- C1 representation/optimization semantics are unchanged by C2.P1.

---

# 1. Requirements Analysis and Ratified C2.P1 Surface

## 1.1 Supported source forms

C2.P1 supports ordinary nominal targets:

```phalcom
class User {
  const _name: String
}

impl User {
  name { _name }

  renamed(_ value: String) -> User {
    User.new(name: value)
  }
}
```

It supports first-class `data` targets:

```phalcom
data Point<T>(
  x: T,
  y: T,
)

impl<T> Point<T> {
  translated(_ dx: T, _ dy: T) -> Point<T> {
    Point(x: x + dx, y: y + dy)
  }
}
```

It supports enum-root behavior without yet migrating enum-declaration behavior syntax:

```phalcom
enum Status {
  Ready
  Failed(_ reason: String)
}

impl Status {
  isTerminal { ... }
}
```

Class-side behavior uses the existing class-side marker:

```phalcom
impl User {
  @class
  fromName(_ name: String) -> User {
    User.new(name: name)
  }
}
```

C2.P1 also supports a covering generic impl whose target arguments are a bijection over the impl’s own generic binders:

```phalcom
impl<A, B> Pair<A, B> {
  swapped -> Pair<B, A> {
    Pair(second, first)
  }
}
```

A permutation remains covering and is legal:

```phalcom
impl<A, B> Pair<B, A> {
  // A and B still range over the entire target declaration.
}
```

The implementation therefore must record the mapping from impl-owned type parameters to target declaration parameters rather than assuming positional identity.

## 1.2 Deliberately rejected C2.P1 forms

The following are semantically rejected, not partially implemented:

```phalcom
// Imported/foreign target: open-class semantics are forbidden.
impl imported.lib.User {
  extra { ... }
}
```

```phalcom
// Specialization: C2.P3.
impl Point<Int> {
  intOnly { ... }
}
```

```phalcom
// Non-covering diagonal head: C2.P3.
impl<T> Pair<T, T> {
  diagonalOnly { ... }
}
```

```phalcom
// Conditional applicability: C2.P3.
impl<T> Point<T> where T <: Number {
  magnitude { ... }
}
```

```phalcom
// Exact enum case behavior: C2.P2.
impl<T> Option<T>::Some(_) {
  ...
}
```

```phalcom
// Trait conformance: LANG005.C4.
impl Printable for User {
  ...
}
```

```phalcom
// Storage is never legal in impl.
impl User {
  const _cache = ...
}
```

```phalcom
// C2.P1 requires executable inherent behavior.
impl Status {
  isTerminal -> Bool
}
```

```phalcom
// Special constructor lifecycle is not introduced by C2.P1.
impl User {
  @constructor
  make(name) { ... }
}
```

## 1.3 Ownership boundary

C2.P1 fixes the inherent-impl ownership rule to:

```text
impl module == target DeclarationId.module
```

That is the complete rule for this checkpoint.

A broader package/coherence ownership regime may be designed later, but it must not be inferred here. Same-package-but-different-module impls are rejected in C2.P1 because admitting them would make a declaration’s canonical behavioral surface depend on which sibling module is analyzed/imported/loaded.

This rule guarantees:

```text
importing module X
    cannot
change the member surface of declaration Y
```

## 1.4 Namespace behavior

`impl` has no declared name.

Therefore:

```text
InterfaceBuilder:
    Statement::Impl -> no namespace declaration
                       no exportable declaration
                       no import target
```

The target name inside an impl is a semantic type/declaration reference and must appear in the source index as such.

## 1.5 Behavioral member set

The reusable behavior-only member category is:

```rust
pub enum BehaviorMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    Index(IndexMethodDef),
}
```

C2.P1 should establish this shared AST category rather than defining a third duplicate member enum.

The current `EnumBehaviorMember` already has this same four-variant shape. Migrate it mechanically to a type alias or compatibility projection over `BehaviorMember` so C2.P2 can later remove enum-local behavior source syntax without another AST representation rewrite:

```rust
pub type EnumBehaviorMember = BehaviorMember;
```

If Rust/existing API constraints make a literal type alias disruptive, retain a compatibility wrapper with `From`/borrow helpers, but `BehaviorMember` becomes the canonical behavior-only syntax category.

`ClassMember` may remain structurally unchanged in P1. Do not rewrite every class-member consumer solely to embed `BehaviorMember`.

## 1.6 Member attributes

Impl behavior uses existing member semantics:

- `@class` selects class-side dispatch.
- visibility attributes retain existing semantics;
- contract attributes retain existing semantics;
- privileged `@internal`/`@native` rules remain the existing rules.

Impl-specific code must not bypass current attribute legality.

Attributes that imply storage derivation or constructor lifecycle are rejected where they do not make sense for an impl.

## 1.7 Identity model

Required identity split:

```text
DeclarationId(User)
    canonical nominal target

ImplId(module, local)
    one source impl fragment / provenance

CallableId {
    owner: User,
    selector: name-shape,
    side: Instance
}
    canonical behavioral member identity

CallableDefinitionProvenance {
    declared directly
    or
    inherent impl ImplId
}
    source/body origin
```

Two declarations:

```phalcom
class User {
  name { ... }
}

impl User {
  name { ... }
}
```

produce the same would-be `CallableId`.

That is a conflict, not two overloads and not two callable identities.

## 1.8 Stable impl identity

Introduce:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImplLocalId(pub u32);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImplId {
    pub module: ModuleId,
    pub local: ImplLocalId,
}
```

Assign `ImplLocalId` deterministically from impl-statement order within one parsed module.

`ImplId` is snapshot/project semantic provenance. Callable runtime dispatch never keys on it.

If the repository already has a canonical top-level source-fragment local-ID mechanism that is explicitly suitable for semantic declaration fragments, reuse it instead of adding a parallel counter. Record that mechanical substitution in implementation state.

## 1.9 Generic ownership

Current `TypeParameterOwner` supports declaration- and callable-owned binders. C2.P1 adds:

```rust
TypeParameterOwner::Impl(ImplId)
```

and carries this variant through the existing stable/export metadata owner representations that exhaustively encode type-parameter ownership.

Do not encode impl-owned parameters as declaration-owned parameters, callable-owned parameters, anonymous type IDs, or name-only substitutions.

## 1.10 Covering generic target rule

C2.P1 target resolution computes an explicit map:

```text
impl type parameters
        ↕
target declaration parameters
```

For a generic target to be unconditional/covering:

1. target declaration arity equals impl binder arity;
2. every target argument is exactly one impl-owned type parameter;
3. every impl-owned type parameter appears exactly once in target arguments;
4. there are no concrete target arguments;
5. there are no repeated parameters;
6. there are no omitted/extra impl parameters;
7. there is no non-empty `where` clause or impl-level generic constraint.

This is a bijection, not necessarily positional identity.

```text
impl<A,B> Pair<A,B>       covering
impl<A,B> Pair<B,A>       covering
impl<T>   Pair<T,T>       specialized / reject until P3
impl<T>   Pair<T,Int>     specialized / reject until P3
impl<T,U> Box<T>          unused U / reject
impl      Box<Int>        specialized / reject until P3
```

## 1.11 Canonicalization of covering generic signatures

Impl members are checked in the impl-owned generic environment because source names belong to the impl binder.

Before publishing a member into the unconditional target effective surface, normalize all impl-owned target-head parameters into the target declaration’s canonical parameter space using the covering substitution.

Conceptually:

```text
impl<A,B> Pair<B,A>

target declaration:
    Pair<X,Y>

covering substitution:
    B -> X
    A -> Y
```

A method declared:

```phalcom
firstImplValue -> A
```

publishes to the effective `Pair<X,Y>` surface as return type `Y`.

This prevents the final declaration surface from retaining free `ImplId`-owned parameters after the impl fragment has been merged.

Callable-owned method generics remain callable-owned and are not rewritten as declaration parameters.

## 1.12 Declared surface versus effective surface

The current repository has one `DeclarationSurface` projection containing instance/class maps. C2.P1 makes its role explicit:

```text
primary syntax
    |
    v
DeclaredSurfaceProduct
    |
    +----------------------+
                           |
impl syntax                |
    |                      |
    v                      |
InherentImplContribution --+
                           |
                           v
EffectiveSurfaceProduct
                           |
                           v
DeclarationSurface
(final lookup/dispatch projection)
```

The existing `DeclarationSurface` struct can remain the final projection type.

Do not mutate one `DeclarationSurface` incrementally while walking source statements. Build semantic products, then merge.

## 1.13 Query architecture

Add query/product identities conceptually equivalent to:

```rust
QueryKey::DeclaredSurface(DeclarationId)
QueryKey::InherentImplContribution(ImplId)
QueryKey::InherentImplSet(DeclarationId)
QueryKey::DeclarationSurface(DeclarationId) // final effective surface
```

Keep `QueryKey::DeclarationSurface` as the existing consumer-facing final surface where practical so current lookup consumers do not all need a semantic rename.

The final effective surface depends on:

```text
DeclaredSurface(target)
+
ordered InherentImplContribution ids for target
```

not on impl member bodies.

Callable bodies remain separately queried by `CallableId`.

## 1.14 Contribution product

Target shape:

```rust
pub struct InherentImplContribution {
    pub id: ImplId,
    pub target: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub covering: CoveringImplSubstitution,
    pub members: Box<[InherentMemberContribution]>,
    pub source: SemanticSourceSpan,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}
```

Member contribution:

```rust
pub struct InherentMemberContribution {
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source_member: ImplMemberSource,
}
```

Use repository-native source/provenance types instead of inventing duplicate span wrappers when possible.

## 1.15 Effective member definition and provenance

An effective surface map alone is insufficient: later `CallableSignature(CallableId)` and `CallableBody(CallableId)` queries must know which syntax/body defines an impl-origin callable.

Introduce an effective definition index conceptually equivalent to:

```rust
pub enum CallableDefinitionOrigin {
    PrimaryDeclaration,
    InherentImpl(ImplId),
}

pub struct EffectiveCallableDefinition {
    pub callable: CallableId,
    pub origin: CallableDefinitionOrigin,
    pub source_site: SourceSiteId,
}
```

Store this in, or alongside, the effective-surface product.

Do not make body lookup scan all module impls by selector every time.

## 1.16 Conflict model: no last wins

The current `MemberSurface` maps can overwrite by selector. C2.P1 must perform checked namespace insertion before projecting into those maps.

Conflicts include:

- primary callable vs impl callable with same `CallableId`;
- impl vs impl with same `CallableId`;
- an impl property getter colliding with a data component’s property selector;
- a class-side impl callable colliding with an enum variant constructor/associated callable of the same selector shape where current associated lookup would become ambiguous;
- any other member category current associated/member resolution treats as the same selector namespace.

Conflict recovery:

```text
valid language semantics:
    duplicate is rejected

diagnostic recovery:
    keep the already-canonical accepted definition
    mark later contribution rejected
    do not publish or runtime-install rejected definition
```

For a primary-declaration conflict, the primary definition is retained for recovery.

For impl-vs-impl conflicts, deterministic `ImplId` order selects the recovery survivor only so downstream analysis can continue. This does not make source order a valid override mechanism.

## 1.17 Data component selector reservation

Do not represent a data component as a synthetic getter `CallableId`.

Instead, when building the effective member namespace, reserve its property selector with origin:

```rust
EffectiveMemberOrigin::DataComponent(DataComponentId)
```

An impl member that would make `p.x` ambiguous is diagnosed.

The final data component remains a `DataComponentId` and continues to lower through data projection semantics.

## 1.18 Enum associated selector reservation

Enum variant constructors remain `VariantId`/family identities.

Class-side inherent behavior must participate in the current associated lookup namespace. If an impl adds an associated callable whose exact selector shape collides with a variant constructor, diagnose rather than changing precedence.

Do not turn the variant constructor into a method and do not hide it.

## 1.19 Body checking

Impl member bodies are checked as though behavior were declared on the target:

```text
current declaration owner = target DeclarationId
current dispatch side = member side
Self = target receiver role
private/protected access owner = target
field/component lookup = target
```

but source generic names first resolve through the impl’s own generic environment.

For a covering generic impl, body checking has both impl-owned binders and target application substitution available.

The effective published signature is then canonicalized into declaration parameter space as described above.

## 1.20 Inheritance

A valid inherited lookup must behave exactly as if the accepted member had been written in the target’s primary declaration.

```phalcom
class Base {}
impl Base {
  f { 1 }
}
class Child extends Base {}

Child.new().f
```

uses ordinary inherited dispatch.

No extra “search impl blocks” stage appears in lookup.

## 1.21 Runtime/compiler model

Do not compile:

```phalcom
impl User { ... }
```

by synthesizing:

```phalcom
class User { ... }
```

and feeding it to the existing class reopen implementation.

Instead:

1. semantic lowering determines accepted impl members and their target;
2. compiler pre-indexes accepted impl fragments by target declaration;
3. when compiling the target declaration’s behavior object, compile/install accepted impl members through a behavior-member helper;
4. the later `Statement::Impl` emits no runtime target lookup/mutation;
5. rejected/foreign impls are never installed.

The compiler should extract a reusable method/getter/setter/index compilation/install helper from `class_decl.rs` rather than duplicating the entire class declaration compiler.

## 1.22 Source-order independence

These must have the same member surface and runtime behavior:

```phalcom
class User {}
impl User { f { 1 } }
```

```phalcom
impl User { f { 1 } }
class User {}
```

provided the module interface/semantic pass accepts both.

The compiler therefore needs a prepass/index over impl statements before emitting target declarations.

An impl is declarative contribution, not a runtime statement whose effects occur at its textual position.

## 1.23 Compiler fail-closed rule

Unlike ordinary legacy standalone class compilation, impl compilation cannot safely invent semantic authorization.

If executable lowering does not contain an accepted impl contribution for an `ImplId`, the compiler must not mutate a runtime target.

Use a compiler internal/missing-lowering error.

Do not resolve a target by runtime global name and attach methods opportunistically.

## 1.24 Incremental requirements

| Edit | Must invalidate | Must not unnecessarily invalidate |
|---|---|---|
| impl body only | callable body/analysis/artifact | target effective surface if signature unchanged |
| impl return/parameter signature | contribution + effective surface + dependent calls | unrelated target surfaces |
| add/remove impl member | contribution + target effective surface | other declarations |
| add/remove impl block | target impl set + effective surface | unrelated targets |
| change target | old target impl set + new target impl set | unrelated targets |
| change visibility/dispatch side | contribution + effective surface | unrelated targets |
| whitespace/range-only edit | source maps as needed | semantic signature/effective fingerprint |
| unrelated method body | its body query | impl set/effective surfaces |

Member bodies must not be hashed into the effective-surface fingerprint.

## 1.25 Tooling requirements

Source indexing must expose:

- the `impl` block;
- the target type occurrence → target `DeclarationId`;
- generic parameter declarations/uses;
- each member declaration → canonical target-owned `CallableId`;
- member body occurrences through ordinary callable attachment;
- provenance that allows hover/navigation to indicate an impl-origin definition if the editor surface chooses to render it.

`impl` itself does not appear as a module symbol.

Go-to-definition on an impl target goes to the nominal declaration.

Go-to-definition on a call to an impl-defined method goes to that method declaration inside the impl block.

## 1.26 Runtime reflection boundary

C2.P1 does not implement the final reflection API for impl provenance.

However, runtime method lookup must see accepted inherent behavior.

Do not expose `ImplId` as a runtime class identity.

C7 will add full reflection/provenance surfaces.

---

# 2. Repository Facts Verified for This Plan

1. `Statement` currently has `Class`, `Enum`, and `Data`, but no `Impl`.
2. `InterfaceBuilder` explicitly collects Class/Enum/Data/TypeAlias/Let declarations into the module namespace. `Impl` must be an ignored contribution statement there.
3. The semantic `DeclarationSurface` currently contains instance/class `MemberSurface`s. Each member map is keyed by selector/ID and its insertion helpers do not themselves enforce conflict diagnostics; C2.P1 needs a checked merger.
4. `QueryKey` already includes `DeclarationSurface(DeclarationId)`, `CallableSignature(CallableId)`, and other declaration-granular products. Keep `DeclarationSurface` as the final consumer query where possible.
5. `CallableId` already contains owner, selector, and dispatch side. That is exactly the identity required for impl-defined behavior.
6. `TypeParameterOwner` currently has only `Declaration` and `Callable`.
7. `CompiledTypeParameterOwner` and stable metadata ownership representations also exhaustively encode only declaration/callable ownership and must migrate with `TypeParameterOwner::Impl`.
8. The current `EnumBehaviorMember` is exactly Method/Getter/Setter/Index and should become compatibility over a shared `BehaviorMember`.
9. Existing `register_class_surface` directly builds one `DeclarationSurface` from primary `ClassDef.members`; this is the architectural seam that must become declared-surface publication rather than final truth.
10. Current class compilation performs class allocation/finalization and method installation together and also supports runtime class-reopen mechanics. `impl` must not be mapped to that reopen entry point.
11. First-class data now has explicit `DeclarationKind::Data`, canonical data identities, behavior class, and ProductStorage representation.
12. Existing enum semantics retain `EnumBehaviorProduct { root_defaults, root_requirements, case_implementations }`; C2.P1 must coexist with it. C2.P2 will re-home those source semantics.
13. Current source-index architecture already tracks formal declarations/expressions and canonical semantic targets. Impl members must plug into this architecture, not create an LSP-local resolver.
14. Class-side behavior uses existing dispatch-side machinery and the existing `@class` source convention.

---

# 3. Non-Negotiable C2.P1 Invariants

1. `ImplId` never replaces `CallableId`.
2. `CallableId.owner` for an inherent impl member is the target declaration.
3. An impl cannot add storage.
4. An impl cannot change inheritance.
5. An impl cannot change data/enum representation.
6. An impl cannot be contributed by an imported/foreign module in P1.
7. An impl block creates no global/module binding.
8. Selector conflicts diagnose; no later fragment silently replaces an earlier member.
9. Invalid duplicate impl members are not runtime-installed.
10. A data component remains a `DataComponentId`.
11. A variant constructor remains a `VariantId`.
12. Instance and class-side namespaces remain separate except where existing associated lookup defines a common associated selector namespace.
13. Effective surface publication is source-order independent.
14. Body edits do not change effective-surface identity when the signature is unchanged.
15. Conditional availability is not smuggled into P1.
16. Generic covering normalization leaves no free impl-owned parameter in the published unconditional target surface.
17. Lookup does not scan source impl blocks at call sites.
18. Runtime dispatch does not scan `ImplId`s.
19. Compiler does not attach methods to a runtime class by unresolved global-name lookup.
20. Current enum behavior remains valid until C2.P2 migrates it.
21. C1 data/enum physical representation remains untouched.
22. Source/LSP and compiler use the same canonical `CallableId` for an impl-defined member.

---

# 4. Checkpoint Map

| Checkpoint | Tasks | Contract proved |
|---|---:|---|
| C0 | 1–3 | syntax, behavior-only AST, `ImplId`, and module-source indexing exist without changing lookup/runtime |
| C1 | 4–6 | same-module nominal targets and covering generic impls produce canonical semantic contributions |
| C2 | 7–10 | declared + impl contributions merge into one conflict-checked effective surface with canonical callable definition provenance |
| C3 | 11–13 | impl bodies type-check under the target receiver/generic environment and ordinary dispatch/inheritance sees effective behavior |
| C4 | 14–17 | compiler/runtime installs accepted inherent members into class/data/enum behavior objects without runtime reopen semantics |
| C5 | 18–21 | source tooling, incrementality, hostile conflict/open-class gates, broad tests, docs, and fingerprint evidence close P1 |


# Checkpoint C0 — Syntax, shared behavior AST, and impl provenance identity

## Task 1 — Freeze the pre-impl baseline and record architectural takeover

**Purpose:** Establish the actual repository revision, current declaration-surface behavior, and baseline tests before adding `impl`.

**Files:**
- Modify: `docs/implementation/LANG005/LANG005.C2-/implementation-state.md` if a C2 state file already exists; otherwise create the canonical C2 state file at that path.
- Inspect only: current AST/parser/module/semantic/compiler/runtime files listed in §2.

**Interfaces:**
- Consumes: current C1-complete repository.
- Produces: C2.P1 baseline revision, file/symbol takeover map, test evidence.

- [ ] Record `git rev-parse HEAD`, `git status --short`, and recent relevant commits.
- [ ] Run the current focused class member/dispatch tests.
- [ ] Run current data behavior/runtime tests.
- [ ] Run current enum behavior/ADT tests.
- [ ] Run current semantic `DeclarationSurface`/dispatch tests.
- [ ] Record final current symbols for:
  - declaration-surface query;
  - callable signature/body query;
  - source attachment/index;
  - class/data/enum compiler member installation;
  - associated class-side/variant lookup.
- [ ] Add a state section:

```markdown
## C2.P1 takeover
- base revision:
- DeclarationSurface query:
- callable definition/signature query:
- class member compiler:
- data behavior class compiler:
- enum root/case behavior compiler:
- source index attachment:
```

- [ ] Commit only if the repository’s implementation workflow expects plan-execution state commits:

```bash
git add docs/implementation/LANG005/LANG005.C2-/implementation-state.md
git commit -m "docs(lang005): record c2 p1 implementation baseline"
```

**Hard gate:** No production semantics changed.

---

## Task 2 — Add `impl` syntax and canonical `BehaviorMember`

**Purpose:** Parse inherent impl fragments as first-class source syntax without introducing a namespace binding.

**Files:**
- Modify: `phalcom-ast/src/token.rs`
- Modify: `phalcom-ast/src/lexer.rs`
- Modify: `phalcom-ast/src/ast.rs`
- Modify: `phalcom-ast/src/parser.rs`
- Modify: AST parser tests, create/extend `phalcom-ast/tests/impl_syntax.rs`
- Modify exhaustive AST walkers/fingerprint code as compiler errors identify them.

**Interfaces:**
- Produces:

```rust
pub enum BehaviorMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    Index(IndexMethodDef),
}

pub struct ImplDef {
    pub type_params: Vec<TypeParameterDef>,
    pub target: TypeAnnotation,          // use actual canonical type-syntax AST type
    pub where_clause: Option<WhereClause>, // use existing generic-where AST type
    pub members: Vec<BehaviorMember>,
    pub range: SourceRange,
    pub target_range: SourceRange,
}

Statement::Impl(ImplDef)
```

Mechanical field types/names must use the repository’s existing generic/type AST nodes rather than adding parallel type syntax.

- [ ] Write lexer test proving `impl` is a keyword rather than an identifier.
- [ ] Write parser test for:

```phalcom
impl User {
  name { "x" }
}
```

- [ ] Write parser test for a covering generic impl using syntax already accepted for generic headers.
- [ ] Write parser test accepting a syntactic `where` clause so C2.P3 does not need a grammar redesign; semantic rejection belongs to Task 5.
- [ ] Introduce `BehaviorMember`.
- [ ] Migrate `EnumBehaviorMember` to a compatibility alias/wrapper over `BehaviorMember`.
- [ ] Extract/reuse the behavior-member parser shared by enum behavior and impl.
- [ ] Ensure impl parser accepts only behavior forms. A field declaration inside `impl` must not become an impl member.
- [ ] Ensure `Statement::Impl` participates in statement range and AST fingerprint walking.
- [ ] Run:

```bash
cargo +stable test -p phalcom-ast --test impl_syntax
cargo +stable test -p phalcom-ast
```

- [ ] Commit:

```bash
git add phalcom-ast
git commit -m "feat(ast): add inherent impl syntax and shared behavior members"
```

---

## Task 3 — Add `ImplId` and module/source structural indexing without namespace binding

**Purpose:** Give each impl stable semantic provenance while proving it does not create a module binding.

**Files:**
- Modify: canonical semantic/module identity crate where source-owned semantic IDs live.
- Modify: `phalcom-modules/src/interface.rs`
- Modify: `phalcom-semantic/src/semantic_shard.rs`
- Modify: `phalcom-semantic/src/source_index/*`
- Modify: semantic/module fingerprint code.
- Test: modules interface tests; semantic source-structure tests.

**Interfaces:**
- Produces:

```rust
pub struct ImplLocalId(pub u32);

pub struct ImplId {
    pub module: ModuleId,
    pub local: ImplLocalId,
}
```

and a module-local index equivalent to:

```rust
impls: BTreeMap<ImplId, ImplSourceInfo>
```

- [ ] Add `ImplId` using an existing canonical top-level local-ID pattern if available.
- [ ] Assign impl IDs deterministically during module source-structure extraction.
- [ ] Add `Statement::Impl` to semantic shard/source fingerprinting.
- [ ] Confirm `InterfaceBuilder` does **not** call `collect_declaration` for `Statement::Impl`.
- [ ] Add module test where `User` is exported and its impl does not add another namespace declaration.
- [ ] Add negative module test proving there is no exportable impl binding.
- [ ] Add source-structure test proving two impl blocks receive distinct `ImplId`s.
- [ ] Add source-index structural site for the impl block and target range, but do not yet resolve target semantics.
- [ ] Run:

```bash
cargo +stable test -p phalcom-modules
cargo +stable test -p phalcom-semantic source_index
```

- [ ] Commit:

```bash
git add phalcom-modules phalcom-semantic
git commit -m "feat(semantic): assign inherent impl provenance identities"
```

Checkpoint C0 is COMPLETE only when `impl` parses, has `ImplId`, affects semantic fingerprints, and creates no namespace binding or runtime behavior yet.

---

# Checkpoint C1 — Target resolution and unconditional/covering impl contributions

## Task 4 — Add impl-owned generic parameter identity across semantic/stable metadata

**Purpose:** Establish first-class generic binder ownership for impl fragments.

**Files:**
- Modify: `phalcom-semantic/src/types/parameter.rs`
- Modify: `phalcom-semantic/src/export.rs` and/or current metadata export owner representation.
- Modify: `phalcom-semantic/src/metadata/stable_identity.rs`
- Modify: `phalcom-type-meta/src/generic.rs`
- Modify all exhaustive matches over type-parameter owner.
- Test owner encode/decode/roundtrip/fingerprint tests.

**Interfaces:**
- Produces:

```rust
TypeParameterOwner::Impl(ImplId)
CompiledTypeParameterOwner::Impl(...)
StableTypeParameterOwnerRef::Impl(...)
```

using actual metadata identity forms.

- [ ] Write failing semantic test creating two impls with generic parameter `T` and prove their parameter IDs are distinct because their owners differ.
- [ ] Add `Impl` to canonical owner enums.
- [ ] Add stable/export conversion for `ImplId`.
- [ ] Update type parameter display/debug formatting to preserve useful provenance.
- [ ] Update metadata serialization schema/version only if the repository requires an explicit version change for enum expansion.
- [ ] Add roundtrip test proving impl-owned type parameters do not decode as declaration/callable owners.
- [ ] Run:

```bash
cargo +stable test -p phalcom-type-meta
cargo +stable test -p phalcom-semantic type_parameter
```

- [ ] Commit:

```bash
git add phalcom-semantic phalcom-type-meta
git commit -m "feat(types): add impl-owned generic parameter identity"
```

---

## Task 5 — Resolve inherent impl targets and enforce C2.P1 ownership/applicability

**Purpose:** Turn impl target syntax into a canonical same-module nominal target or a diagnostic.

**Files:**
- Create: `phalcom-semantic/src/impls.rs`
- Modify: semantic checker/session module enumeration.
- Modify: diagnostic codes.
- Test: create `phalcom-semantic/tests/semantic/impls/targets.rs` and register with existing test harness.

**Interfaces:**
- Produces:

```rust
pub enum InherentImplApplicability {
    Unconditional,
    Covering(CoveringImplSubstitution),
}

pub struct ResolvedInherentImplTarget {
    pub id: ImplId,
    pub declaration: DeclarationId,
    pub target_type: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub applicability: InherentImplApplicability,
}
```

`CoveringImplSubstitution` must explicitly represent impl-param ↔ declaration-param mapping.

- [ ] Write failing test for non-generic same-module class target.
- [ ] Write failing tests for data and enum-root targets.
- [ ] Resolve target using the existing type resolver; do not look up a bare string in module AST manually.
- [ ] Require the resolved target to be a canonical nominal declaration, not structural Tuple/Record/callable/union/Dynamic.
- [ ] Reject type aliases as impl owners even if they resolve to a nominal target.
- [ ] Enforce `target.module == impl.module`.
- [ ] Add a stable foreign-target diagnostic.
- [ ] Add a stable non-nominal-target diagnostic.
- [ ] Build impl-owned generic signature.
- [ ] Parse `where` syntax but reject a non-empty clause with a dedicated C2.P3-deferred diagnostic.
- [ ] For generic nominal targets, compute the covering bijection.
- [ ] Accept `impl<A,B> Pair<A,B>`.
- [ ] Accept `impl<A,B> Pair<B,A>` and assert the mapping.
- [ ] Reject `impl<T> Pair<T,T>`.
- [ ] Reject `impl<T> Pair<T,Int>`.
- [ ] Reject unused/extra impl parameters.
- [ ] Reject exact enum-case target.
- [ ] Reject trait-style `impl Trait for T` at parser/semantic boundary as outside P1.
- [ ] Run:

```bash
cargo +stable test -p phalcom-semantic impl_target
```

- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): resolve same-module covering inherent impl targets"
```

---

## Task 6 — Build `InherentImplContribution` and canonicalize signatures into target parameter space

**Purpose:** Publish behavior-only member signatures with target-owned callable identity.

**Files:**
- Modify: `phalcom-semantic/src/impls.rs`
- Refactor: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: semantic session/query product plumbing.
- Test: `phalcom-semantic/tests/semantic/impls/signatures.rs`

**Interfaces:**
- Produces the contribution/member products from §1.14.

- [ ] Extract a behavior-member signature helper reusable by `ClassMember`, current enum behavior, and `BehaviorMember`.
- [ ] Derive `DispatchSide` using existing `@class`/static semantics.
- [ ] Construct every inherent member’s `CallableId` with target `DeclarationId`, selector, and dispatch side; never use `ImplId` as callable owner.
- [ ] Preserve impl source provenance separately.
- [ ] Reject any behavior node carrying special constructor semantics.
- [ ] Reject bodyless members in P1.
- [ ] Apply existing member visibility/attribute rules.
- [ ] Form source signatures inside an impl-owned generic resolver.
- [ ] Apply the covering substitution to canonicalize target-head impl parameters to declaration-owned parameters before contribution publication.
- [ ] Assert published unconditional surface signatures contain no free target-head `TypeParameterOwner::Impl(current_impl)` parameter.
- [ ] Preserve callable-owned method generic parameters unchanged.
- [ ] Add permutation test proving `impl<A,B> Pair<B,A>` publishes the correct declaration-parameter signature.
- [ ] Add class-side contribution test.
- [ ] Add data and enum-root contribution tests.
- [ ] Run:

```bash
cargo +stable test -p phalcom-semantic impl_signature
```

- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): publish inherent impl member contributions"
```

Checkpoint C1 is COMPLETE only when every accepted P1 impl has canonical target identity, covering generic mapping, target-owned callable IDs, and contribution signatures, but final dispatch lookup is not yet switched.

---

# Checkpoint C2 — Declared/effective surfaces, conflict checking, and callable definition provenance

## Task 7 — Split primary `DeclaredSurface` from final `DeclarationSurface`

**Purpose:** Stop treating the primary declaration as the complete behavioral truth.

**Files:**
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/product.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify enum/data declared-surface producers.
- Test declaration-surface query tests.

**Interfaces:**
- Produces:

```rust
QueryKey::DeclaredSurface(DeclarationId)
QueryKey::InherentImplContribution(ImplId)
QueryKey::InherentImplSet(DeclarationId)
QueryKey::DeclarationSurface(DeclarationId) // effective
```

and repository-specific product wrappers.

- [ ] Write a query regression proving a class with no impl has effective surface identical to its declared surface.
- [ ] Refactor current class `register_class_surface` responsibility so the primary member list becomes the declared surface product.
- [ ] Apply the same declared-surface concept to data/enum roots without changing their existing special semantic products.
- [ ] Ensure callable body checking does not require final effective surface to build the primary declared surface, preventing a dependency cycle.
- [ ] Add dependency tracing test asserting `DeclarationSurface(User)` depends on `DeclaredSurface(User)` and `InherentImplSet(User)` rather than the inverse.
- [ ] Run semantic DB query/cycle tests.
- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "refactor(semantic): separate declared and effective member surfaces"
```

---

## Task 8 — Build target-indexed `InherentImplSet` with fine-grained fingerprints

**Purpose:** Let one declaration depend only on the impl fragments that canonically target it.

**Files:**
- Modify: `phalcom-semantic/src/impls.rs`
- Modify: semantic module/source product.
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Test incremental/query tests.

**Interfaces:**
- Produces:

```rust
pub struct InherentImplSet {
    pub target: DeclarationId,
    pub impls: Box<[ImplId]>,
}
```

- [ ] Build the index during semantic module product formation, not during call lookup.
- [ ] Include only same-module fragments for the target; invalid fragments retain diagnostics but cannot become accepted contributions.
- [ ] Sort deterministically by `ImplId`.
- [ ] Fingerprint the set by impl identities and contribution-signature fingerprints, not body syntax.
- [ ] Add test with impls for `A` and `B`; editing `A` signature invalidates `A` effective surface but not `B`.
- [ ] Add body-only edit test proving effective surface fingerprint remains unchanged.
- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): index inherent impl fragments by target"
```

---

## Task 9 — Merge declared and impl members with an explicit conflict namespace

**Purpose:** Produce one valid effective surface without silent hash-map overwrite.

**Files:**
- Create or extend: `phalcom-semantic/src/effective_surface.rs`
- Modify: surface query builder.
- Modify: data/associated semantic accessors needed for selector reservations.
- Test: `phalcom-semantic/tests/semantic/impls/conflicts.rs`

**Interfaces:**
- Produces:

```rust
pub struct EffectiveSurfaceProduct {
    pub owner: DeclarationId,
    pub surface: DeclarationSurface,
    pub definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}
```

and an internal checked member namespace with origins such as:

```rust
enum EffectiveMemberOrigin {
    PrimaryCallable(CallableId),
    InherentCallable { callable: CallableId, impl_id: ImplId },
    DataComponent(DataComponentId),
    VariantConstructor(VariantId),
}
```

- [ ] Write failing primary-vs-impl duplicate selector test.
- [ ] Write failing impl-vs-impl duplicate selector test.
- [ ] Write success test for same base with distinct selector shapes/labels.
- [ ] Write success test proving getter and setter remain distinct selector identities.
- [ ] Reserve data component property selectors without creating synthetic `CallableId`s.
- [ ] Add data component collision test.
- [ ] Reserve enum associated variant constructor selectors against class-side impl callables using current associated lookup selector rules.
- [ ] Add enum class-side collision test.
- [ ] Preserve first accepted member only for diagnostic recovery; record rejected contribution.
- [ ] Never call final `MemberSurface::add_callable*` insertion until conflict admission succeeds.
- [ ] Ensure rejected duplicate is absent from `definitions`.
- [ ] Run:

```bash
cargo +stable test -p phalcom-semantic impl_conflict
```

- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): merge inherent impls into checked effective surfaces"
```

---

## Task 10 — Make callable signature/body queries consume effective definition provenance

**Purpose:** Ensure `CallableId` lookup finds the one accepted body whether declared inline or in an impl.

**Files:**
- Modify: callable signature/body query implementations.
- Modify: source attachment lookup.
- Modify: semantic session indexing.
- Test callable query tests.

**Interfaces:**
- Consumes: `EffectiveSurfaceProduct.definitions`.
- Produces: canonical definition-source lookup for `CallableId`.

- [ ] Write failing test querying signature/body for an impl-defined callable by target-owned `CallableId`.
- [ ] Add a canonical definition locator keyed by `CallableId` and target declaration.
- [ ] Primary-origin definition resolves to current class/enum source member.
- [ ] Impl-origin definition resolves directly through `ImplId + member source identity`.
- [ ] Remove any fallback that scans every impl block by selector.
- [ ] Ensure rejected duplicate body is never selected by query.
- [ ] Ensure `CallableSignature(CallableId)` and `CallableBody(CallableId)` do not depend on source order.
- [ ] Add query cache test proving a body-only edit re-runs body analysis while preserving surface signature fingerprint.
- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): resolve callable definitions through effective surfaces"
```

Checkpoint C2 is COMPLETE only when lookup authority is effective-surface based, duplicate conflicts cannot overwrite maps, and every accepted `CallableId` has one canonical source/body provenance.


# Checkpoint C3 — Body analysis, dispatch, inheritance, and nominal-type integration

## Task 11 — Check impl bodies in target receiver and impl generic environments

**Purpose:** Give impl bodies exactly the lexical/type capabilities of target-owned behavior without making the impl a separate receiver type.

**Files:**
- Modify: semantic callable-body query/context.
- Modify: `phalcom-semantic/src/impls.rs`
- Refactor declaration body-check helpers.
- Test: `phalcom-semantic/tests/semantic/impls/bodies.rs`

**Interfaces:**
- Consumes: resolved target + covering substitution + canonical callable signature.
- Produces: ordinary `CallableAnalysis` under target `current_class/current_side`.

- [ ] Test instance `self` type in class impl.
- [ ] Test access to target private field from same-target impl.
- [ ] Test class-side `Self`/class-object semantics using existing rules.
- [ ] Test data component access from a data impl without converting it to a field.
- [ ] Test enum-root method body.
- [ ] Build scoped type resolver containing impl-owned binders while checking source body.
- [ ] Set semantic receiver owner to target `DeclarationId`.
- [ ] Reuse current expected-return and parameter analysis.
- [ ] Verify return diagnostics refer to the impl member source range.
- [ ] Verify access checks regard target ownership, not `ImplId`, as privacy owner.
- [ ] Add generic permutation body test ensuring source `A/B` resolve under impl binder while published signature stays declaration-canonical.
- [ ] Run focused body tests.
- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): analyze inherent impl bodies as target-owned behavior"
```

---

## Task 12 — Switch ordinary lookup/dispatch to final effective surfaces and prove inheritance

**Purpose:** Make inherent members ordinary members after semantic aggregation.

**Files:**
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify any associated/member lookup consumers that bypass the declaration-surface query.
- Test dispatch/inheritance/associated lookup.

**Interfaces:**
- Source of truth: final `DeclarationSurface(DeclarationId)` query.

- [ ] Add direct instance method lookup test for impl-defined callable.
- [ ] Add subclass inheritance test.
- [ ] Add override test where subclass primary member overrides inherited base impl member according to existing class inheritance rules.
- [ ] Add class-side lookup test.
- [ ] Add visibility tests.
- [ ] Ensure lookup never contains a secondary `for impl in ...` search.
- [ ] Ensure current declared-only classes behave identically.
- [ ] Ensure method family lookup continues grouping by selector base/shape according to existing rules.
- [ ] Run dispatch/family tests.
- [ ] Commit:

```bash
git add phalcom-semantic
git commit -m "feat(semantic): dispatch through effective inherent member surfaces"
```

---

## Task 13 — Integrate data and enum-root inherent behavior without disturbing representation/case semantics

**Purpose:** Prove nominal value/sum types can own inherent behavior using the same semantic surface model.

**Files:**
- Modify only semantic dispatch/associated integration required by tests.
- Do not modify C1 ProductStorage/DataObject/ADT layout code.
- Test data/enum behavior.

**Interfaces:**
- `impl DataType` targets its declaration-level behavior class/surface.
- `impl EnumRoot` targets root enum behavior only.

- [ ] Test data instance method dispatch through impl.
- [ ] Test generic covering data impl.
- [ ] Test class-side data impl member if current class-side behavior class supports it.
- [ ] Test enum-root inherent method on multiple cases.
- [ ] Confirm exact-case-only availability is not implemented.
- [ ] Confirm current enum declaration root defaults/requirements continue to work unchanged.
- [ ] Confirm C1 `DataObject`/`ProductStorage`/`AdtCaseObject` code has no impl-specific storage fields.
- [ ] Run data + ADT/GADT suites.
- [ ] Commit:

```bash
git add phalcom-semantic phalcom-core/tests
git commit -m "test(lang005): qualify inherent behavior on data and enum roots"
```

Checkpoint C3 is COMPLETE only when semantic body checking and lookup make impl members indistinguishable from primary inherent behavior for valid calls, while preserving separate provenance and existing C1/enum semantics.

---

# Checkpoint C4 — Compiler/runtime installation without open-class mutation

## Task 14 — Publish accepted impl lowering to the compiler

**Purpose:** Give codegen an explicit semantic authorization list instead of making it resolve impl targets itself.

**Files:**
- Modify: `phalcom-core/src/modules/semantic_lowering.rs`
- Modify semantic→core lowering builder.
- Test lowering projection.

**Interfaces:**
- Produces conceptually:

```rust
pub struct InherentImplLoweringSpec {
    pub id: ImplId,
    pub target: DeclarationId,
    pub members: Box<[InherentImplMemberLowering]>,
}

pub struct InherentImplMemberLowering {
    pub callable: CallableId,
    pub source_member: ImplMemberSource,
}
```

Rejected members should be omitted from executable lowering rather than carried with an `accepted` boolean unless the current lowering architecture has a strong reason to retain rejected entries for diagnostics.

- [ ] Write lowering test for one class impl.
- [ ] Assert target is canonical `DeclarationId`, not source string.
- [ ] Assert callable IDs are target-owned.
- [ ] Assert a duplicate rejected by semantic merge is absent from executable lowering.
- [ ] Assert foreign impl is absent.
- [ ] Add target→impl lowering index for compiler prepass.
- [ ] Do not include representation/layout mutation fields.
- [ ] Commit:

```bash
git add phalcom-core/src/modules/semantic_lowering.rs phalcom-semantic
git commit -m "feat(lowering): project accepted inherent impl members to codegen"
```

---

## Task 15 — Extract a reusable behavior-member compiler instead of compiling impl as class reopen

**Purpose:** Reuse current method/getter/setter/index code generation without re-entering class declaration/reopen logic.

**Files:**
- Refactor: `phalcom-core/src/compiler/lib/class_decl.rs`
- Create: `phalcom-core/src/compiler/lib/impl_decl.rs`
- Modify compiler module registration/tests.

**Interfaces:**
- Produces a helper conceptually:

```rust
fn compile_behavior_member_for_target(
    &mut self,
    target: CompiledBehaviorTarget,
    member: &BehaviorMember,
    callable: &CallableId,
) -> Result<(), CompilerError>
```

Use actual runtime class/stack conventions rather than inventing a parallel installation protocol.

- [ ] Identify the smallest existing class-member compilation block that compiles closure/body, creates selector/signature metadata, and installs method/getter/setter/index behavior.
- [ ] Extract it without including class allocation, superclass resolution, field collection/layout, class reopen validation, declaration-global definition, or class-level attributes.
- [ ] Keep existing class compilation using the helper and prove bytecode regression unchanged for a representative class.
- [ ] Add compile helper for `BehaviorMember`.
- [ ] Reject compiler entry for impl without semantic lowering (`MissingImplLoweringSemantics` or repository-equivalent fail-closed error).
- [ ] Add negative test proving compiler does not perform runtime-global target-name lookup.
- [ ] Run class compiler/runtime suites.
- [ ] Commit:

```bash
git add phalcom-core/src/compiler
git commit -m "refactor(compiler): extract target behavior member installation"
```

---

## Task 16 — Pre-index impl fragments and install them during target declaration compilation

**Purpose:** Make impl contribution declarative and source-order independent.

**Files:**
- Modify: top-level compiler/program compilation driver.
- Modify: `impl_decl.rs`
- Modify: class/data/enum declaration compilers with one post-primary-behavior hook.
- Test source-order and runtime method availability.

**Interfaces:**
- Compiler transient index:

```rust
BTreeMap<DeclarationId, Vec<AcceptedImplFragment>>
```

built from AST `ImplId` + semantic lowering.

- [ ] Build impl AST/lowering index before ordinary statement emission.
- [ ] For each class target, after primary behavior object exists and before declaration compilation releases/finalizes the installation context, install accepted impl members.
- [ ] Add equivalent hook for data behavior class.
- [ ] Add equivalent root enum behavior hook.
- [ ] `Statement::Impl` itself emits no runtime mutation bytecode when visited in normal top-level order.
- [ ] Test impl-before-target and target-before-impl programs produce equivalent runtime behavior.
- [ ] Test multiple fragments.
- [ ] Test top-level call after target declaration but textually before later impl and require impl member is already present under the declarative aggregation model.
- [ ] Test foreign imported target impl fails semantically and never installs.
- [ ] Test duplicate rejected impl member is not installed last-wins.
- [ ] Commit:

```bash
git add phalcom-core/src/compiler
git commit -m "feat(compiler): install inherent impl fragments with target declarations"
```

---

## Task 17 — Prove runtime class/data/enum dispatch integration and absence of layout mutation

**Purpose:** Verify generated methods land on existing behavior objects with no new runtime representation concept.

**Files:**
- Test-focused changes in `phalcom-core/tests`.
- Narrow runtime fixes only if existing behavior installation helper exposes a real incompatibility.

**Interfaces:**
- Runtime has no `ImplObject`, `ImplId` dispatch table, or per-instance impl metadata.

- [ ] Class instance method end-to-end test.
- [ ] Class-side method end-to-end test.
- [ ] Inherited class impl method test.
- [ ] Data impl method end-to-end test.
- [ ] Enum-root impl method end-to-end test.
- [ ] Assert DataObject fields/representation unchanged.
- [ ] Assert AdtCaseObject fields/representation unchanged.
- [ ] Assert InstanceObject field count/layout unaffected by impl.
- [ ] Assert no new heap object variant represents impl blocks.
- [ ] Search:

```bash
rg 'ImplObject|impl_id.*Object|runtime_impl' phalcom-core/src
```

and explain intentional test/docs hits; expected production representation hits: zero.
- [ ] Run core data/ADT/class tests.
- [ ] Commit:

```bash
git add phalcom-core/tests
git commit -m "test(lang005): verify runtime inherent impl dispatch without layout mutation"
```

Checkpoint C4 is COMPLETE only when runtime behavior is installed through canonical target behavior objects, source ordering is irrelevant, and compiler/runtime contain no open-class/impl-object mechanism.

---

# Checkpoint C5 — Source tooling, incrementality, hostile verification, and delivery

## Task 18 — Complete source index and LSP navigation for impl provenance

**Purpose:** Make editor semantics consume canonical IDs without inventing a second impl resolver.

**Files:**
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Modify: `phalcom-semantic/src/source_index/*`
- Modify: `phalcom-lsp/src/hover.rs`, definition/reference handlers only as required.
- Modify semantic tokens keyword list for `impl`.
- Test semantic index + LSP.

**Interfaces:**
- target occurrence → `DeclarationId`;
- impl member declaration → target-owned `CallableId`;
- optional provenance → `ImplId`.

- [ ] Add semantic token test for `impl` keyword.
- [ ] Add source-index target occurrence test.
- [ ] Add source-index member declaration test.
- [ ] Add go-to-definition from `impl User` target to primary `User`.
- [ ] Add go-to-definition from a call to an impl-defined method to the member declaration inside the impl block.
- [ ] Add references test if current reference subsystem indexes callable declarations.
- [ ] Ensure impl block itself is not listed as module/class symbol.
- [ ] Ensure same-name methods on different target declarations retain distinct `CallableId`s.
- [ ] Run LSP tests.
- [ ] Commit:

```bash
git add phalcom-semantic phalcom-lsp
git commit -m "feat(lsp): index inherent impl targets and members canonically"
```

---

## Task 19 — Prove incremental invalidation and body/surface fingerprint separation

**Purpose:** Prevent impl support from collapsing the semantic DB back into module-wide recomputation.

**Files:**
- Modify: semantic incremental tests.
- Modify fingerprint helpers only where evidence proves a missing boundary.

**Interfaces:**
- Uses products from C2.

- [ ] Add cold/edit equivalence fixture with two targets and multiple impls.
- [ ] Edit impl body only; assert callable body changes, effective surface fingerprint does not.
- [ ] Edit return annotation; assert target contribution/effective surface changes.
- [ ] Add member; assert only target effective surface/dependent caller invalidates.
- [ ] Move impl target from `A` to `B`; assert old/new target sets invalidate.
- [ ] Edit unrelated class body; assert impl contribution/surface cache hit.
- [ ] Perform range/whitespace-only edit under existing location-insensitive fingerprint policy and assert semantic fingerprint stability.
- [ ] Verify no impl member body bytes are hashed into surface fingerprint.
- [ ] Run semantic incremental test suite.
- [ ] Commit:

```bash
git add phalcom-semantic/tests phalcom-semantic/src/db
git commit -m "test(semantic): pin impl incremental invalidation boundaries"
```

---

## Task 20 — Run hostile semantic/runtime matrix and negative architectural gates

**Purpose:** Prove the architecture, not merely happy-path syntax.

**Files:**
- Test-only unless a failure reveals a narrow implementation defect.

**Interfaces:**
- Consumes: all C0–C4 syntax, semantic, lowering, compiler, and runtime contracts.
- Produces: release-blocking cross-layer evidence that invalid impls cannot mutate effective/runtime behavior and valid impls are source-order independent.

**Required matrix:**

| Case | Expected |
|---|---|
| impl before target | works; same surface as after-target |
| impl after target | works |
| two non-conflicting fragments | both members present |
| primary vs impl same selector | diagnostic; primary retained for recovery |
| impl vs impl same selector | diagnostic; one deterministic recovery definition, no runtime last-wins |
| same base/different labels | both valid if current selector model distinguishes them |
| getter vs setter | both valid |
| data component vs getter | conflict |
| enum variant associated selector vs class-side impl callable | conflict |
| imported foreign target | reject |
| type alias target | reject |
| structural tuple/record target | reject |
| `Point<Int>` specialization | reject until P3 |
| `Pair<T,T>` | reject until P3 |
| `where` constrained impl | reject until P3 |
| exact enum case target | reject until P2 |
| trait-style impl | reject until C4 |
| bodyless member | reject until P2 enum-root requirement semantics |
| constructor-marked member | reject |
| field/storage member | reject |
| private target field access from own impl | allowed according to existing target ownership rules |
| inherited impl method | ordinary inheritance |
| class-side impl method | ordinary class-side lookup |
| generic covering impl | works |
| generic permutation covering impl | works with correct type substitution |
| source edit body only | surface unchanged |
| REPL / repeated module compilation | no accumulated duplicate runtime installation beyond ordinary module semantics |

- [ ] Implement one test per row where not already covered.
- [ ] Run negative searches:

```bash
rg 'Statement::Impl' phalcom-modules/src/interface.rs
```

Expected: branch explicitly ignores/non-binds impl, not collects declaration.

```bash
rg 'ImplObject|impl_methods|runtime_impl' phalcom-core/src/heap phalcom-core/src/vm
```

Expected production representation hits: zero.

```bash
rg 'compile_class.*Impl|ClassDef.*ImplDef|impl.*reopen' phalcom-core/src/compiler
```

Investigate every hit. There must be no “synthesize class reopen from impl” path.

```bash
rg 'callables_by_selector\.insert|callable_signatures\.insert' phalcom-semantic/src
```

Review effective merge paths and prove checked admission precedes final insertion.

```bash
rg 'TypeParameterOwner::' phalcom-semantic phalcom-type-meta
```

Every exhaustive owner representation must consciously handle `Impl`.

- [ ] Run core/semantic/module/LSP focused suites.
- [ ] Record results in implementation state.
- [ ] Commit:

```bash
git add phalcom-ast phalcom-modules phalcom-semantic phalcom-core phalcom-lsp docs/implementation/LANG005
git commit -m "test(lang005): close inherent impl semantic and runtime matrix"
```

---

## Task 21 — Final broad gates, documentation, and C2.P2 handoff

**Purpose:** Deliver C2.P1 as a stable semantic foundation for enum behavior migration.

**Files:**
- Modify: C2 implementation state.
- Modify: language/internal docs describing declaration surfaces and impl architecture.
- Do not implement C2.P2 syntax migration.

**Interfaces:**
- Produces the final C2.P1 implementation-state record and C2.P2 handoff invariant.

- [ ] Document:

```text
DeclaredSurface
InherentImplContribution
InherentImplSet
Effective DeclarationSurface
CallableId vs ImplId
covering generic impl
same-module ownership
```

- [ ] Record C2.P1 established invariants.
- [ ] Record all diagnostics introduced.
- [ ] Record deferred items explicitly owned by C2.P2/P3/C4.
- [ ] Run:

```bash
cargo +stable fmt --all -- --check
cargo +stable check --workspace --all-targets

cargo +stable test -p phalcom-ast
cargo +stable test -p phalcom-modules
cargo +stable test -p phalcom-semantic
cargo +stable test -p phalcom-core
cargo +stable test -p phalcom-lsp

cargo +stable test --workspace --all-targets
cargo +stable clippy --workspace --all-targets -- -D warnings
```

Use the repository’s current pinned toolchain if implementation-time CI differs.

- [ ] Run all current C1 data/product/ADT regression filters unchanged.
- [ ] Run existing enum behavior/GADT suites unchanged.
- [ ] Run current source-index/LSP broad gates.
- [ ] Perform a deferred-evidence audit: every C0–C5 evidence item is PASS, explicitly outside P1 scope, or an active release-blocking incident.
- [ ] Mark C2.P1 COMPLETE only with no active incident.
- [ ] Set next resume action:

```text
LANG005.C2.P2 — variants-only enum declarations and migration of root/default/case behavior into impl
```

- [ ] Final commit:

```bash
git add docs phalcom-ast phalcom-modules phalcom-semantic phalcom-core phalcom-lsp phalcom-type-meta
git commit -m "feat(lang005): complete first-class inherent impl surfaces"
```

---

# 5. Compiler/Runtime Implementation Guidance

## 5.1 Why existing class reopen is not the impl architecture

The current compiler already has code that can reopen an existing class and install methods. Reusing that *mechanism* at the lowest member-installation level is useful; reusing the *semantic operation* is wrong.

A reopen is runtime mutation:

```text
find existing class
validate reopen
attach members now
```

An inherent impl is declaration aggregation:

```text
parse all fragments
resolve semantic ownership
build effective surface
compile accepted fragments with target
```

The user-visible difference is critical. With impl, whether another module is imported, whether a fragment executes earlier/later, and runtime global lookup order must not change the type’s canonical behavioral surface.

## 5.2 Correct compiler staging

Preferred shape:

```text
Program AST
  |
  +-- semantic lowering:
  |      ImplId -> accepted target/member IDs
  |
  +-- compiler prepass:
         target DeclarationId -> [Impl AST fragments]

Then target declaration:

compile class/data/enum behavior object
compile primary members
compile accepted impl members for this target
finalize/define target

Later Statement::Impl:
    no runtime effect
```

If a particular target compiler must finalize before methods can be attached, adapt the local hook while preserving the same declarative ordering and semantic authorization.

## 5.3 Invalid programs must not demonstrate last-wins runtime behavior

Semantic diagnostics may permit compilation to continue for editor/recovery use. Therefore rejected duplicate impl members must be excluded from executable lowering.

Do not rely on “we emitted a diagnostic so runtime result does not matter.” The compiler must not install the rejected member and accidentally create source-order-dependent behavior in recovered execution.

---

# 6. Semantic Implementation Guidance

## 6.1 Avoid a giant mutable declaration table

Do not implement:

```rust
for statement in module {
    if impl_stmt {
        declaration_surfaces[target].add_callable(...)
    }
}
```

That destroys fine-grained query dependencies and creates statement-order semantics.

Implement immutable/query-owned products.

## 6.2 Effective surface is a derived projection

The semantic truth is:

```text
declared source product
impl contribution products
```

The effective surface is computed.

Later traits will add `TraitSurface` and `TraitConformance` without requiring inherent impls to be reinvented.

## 6.3 Callable identity and definition identity are different

One callable:

```text
CallableId(User, name, Instance)
```

can be defined in the primary `User` declaration or one accepted impl fragment.

The `CallableId` is what calls/dispatch/reference identity use.

Definition provenance is what body analysis/navigation/reflection uses.

Never add `ImplId` to `CallableId`.

---

# 7. Interaction with C2.P2

C2.P2 will rely on P1 to express:

```phalcom
enum Expression {
  Literal(_ value: Int)
  Add(_ left: Expression, _ right: Expression)
}

impl Expression {
  evaluate -> Int
  precedence -> Int { 100 }
}

impl Expression::Literal(_) {
  evaluate -> Int { value }
}
```

C2.P1 does **not** implement that full example.

P1 lays only the reusable pieces:

- `impl` syntax;
- behavior-only members;
- `ImplId`;
- contribution/effective-surface architecture;
- target-owned callable identity;
- generic impl provenance;
- compiler behavior-member installation.

P2 then adds:

- exact-case impl targets;
- enum-root bodyless requirement meaning;
- variant-specific witnesses/overrides;
- removal of source `EnumBehaviorMember` bodies from `enum`.

Do not preempt P2 by assigning bodyless semantics in P1.

---

# 8. Interaction with C2.P3

P3 extends the P1 target model:

```text
P1:
  nominal target
  unconditional
  covering generic head

P3:
  constrained target head
  receiver applicability
  where constraints
  specialized applied heads
  overlap/conflict policy
```

Therefore C2.P1 must preserve `ImplId`, impl-owned generic parameters, resolved target application, and covering substitution as first-class products.

Do not reduce P1 to:

```rust
struct Impl {
    target: DeclarationId
}
```

with no generic/head representation. That would force P3 to replace the entire impl model.

---

# 9. Diagnostics

Use repository diagnostic-code conventions, but provide stable distinct categories equivalent to:

```text
impl.target.unresolved
impl.target.not_nominal
impl.target.foreign
impl.target.alias
impl.target.specialized_deferred
impl.target.not_covering
impl.constraints.deferred
impl.member.storage_forbidden
impl.member.constructor_forbidden
impl.member.body_required
impl.member.conflict
impl.member.associated_conflict
```

Every diagnostic must point primarily at the impl/member source that introduced the invalid contribution and, for conflicts, retain the first/conflicting definition span when repository diagnostics support secondary causes.

---

# 10. Performance Requirements

C2.P1 should not make every member lookup scan impl fragments.

Required complexity direction:

```text
impl aggregation:
    query/build time

ordinary lookup:
    same effective DeclarationSurface maps as today
```

Runtime method send cost should remain unchanged after class construction.

Incremental edits should rebuild only the affected contribution/effective target.

Add no per-instance impl metadata.

Add no per-send provenance lookup.

---

# 11. Final Negative Gates

Before release, verify all:

```text
no runtime ImplObject
no per-instance ImplId
no lookup-time scan of source impl blocks
no imported foreign inherent impl
no impl namespace binding
no field/storage declaration in impl
no bodyless inherent behavior semantics in P1
no exact-case impl semantics in P1
no constrained/specialized impl applicability in P1
no trait conformance semantics in P1
no last-wins member insertion
no rejected duplicate runtime installation
no data component converted to getter identity
no variant constructor converted to method identity
no class reopen synthesis used as impl semantics
no target-owned callable identity replaced by impl-owned identity
no body hash in effective-surface fingerprint
```

---

# 12. Failure Protocol

On a failure, classify before broadening the patch:

```text
PARSER
    impl/behavior syntax or source ranges wrong

TARGET
    target resolution/ownership/applicability wrong

GENERIC
    Impl-owned binders or covering substitution wrong

CONTRIBUTION
    member signature/provenance product wrong

MERGE
    conflict/effective-surface construction wrong

CALLABLE SOURCE
    CallableId cannot find canonical definition/body

BODY
    receiver/Self/private/generic body context wrong

DISPATCH
    effective surface not used by lookup/inheritance

LOWERING
    accepted semantic impl not projected correctly

COMPILER
    member install/runtime staging wrong

RUNTIME
    existing behavior object cannot host accepted member as expected

INCREMENTAL
    invalidation/fingerprint boundary wrong

TOOLING
    source/LSP identity projection wrong

BASELINE
    pre-existing behavior fails without C2.P1

PLAN DRIFT
    repository architecture invalidates a mechanical assumption
```

Fix the narrow owner.

Do not weaken ownership, conflict, or source-order invariants to make a test pass.

---

# 13. Release-Complete Criteria

LANG005.C2.P1 is COMPLETE only when:

- `impl` is a first-class parsed statement.
- `BehaviorMember` is the canonical behavior-only AST category.
- impl blocks have `ImplId`.
- impl blocks create no module binding.
- `TypeParameterOwner::Impl` and stable metadata ownership are complete.
- same-module nominal target ownership is enforced.
- class, data, and enum-root targets work.
- covering generic impls work.
- non-covering/constrained/specialized heads are rejected for P3.
- exact enum-case impl targets are rejected for P2.
- trait-style impls remain outside P1.
- member signatures have target-owned `CallableId`s.
- impl-owned target parameters are canonicalized out of unconditional effective signatures.
- declared surfaces and effective surfaces are distinct semantic products.
- final `DeclarationSurface` merges accepted impl contributions.
- data-component/variant-constructor selector conflicts are checked without identity collapse.
- duplicate members diagnose and do not last-win.
- every accepted callable has one canonical definition provenance.
- body analysis uses target receiver/private/Self semantics.
- ordinary dispatch and inheritance see impl members without scanning impls.
- compiler consumes accepted semantic lowering.
- compiler does not implement impl through runtime class reopen.
- source ordering of impl vs target does not change semantics.
- invalid/rejected impl members are not runtime-installed.
- class/data/enum runtime representation is unchanged.
- source index/LSP uses canonical target/callable identities.
- body-only incremental edits do not invalidate effective surfaces.
- signature/target edits invalidate only appropriate targets/dependents.
- all focused and broad tests pass.
- all negative gates pass.
- implementation state has complete C0–C5 evidence and no active incident.

At that point C2.P2 may rely on this invariant:

> **A Phalcom nominal declaration has one canonical effective behavioral surface assembled from its primary declaration and authorized same-owner inherent impl fragments. Callable identity belongs to the nominal target; impl identity records provenance. The resulting behavior is independent of source/import/runtime load order and can be installed into the existing target behavior object without changing representation.**
