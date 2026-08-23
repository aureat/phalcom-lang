# Phalcom Canonical Native Universe, Primitive Metadata, and Verified Bootstrap — Revised Repository-Grounded Implementation Plan

> **Status:** Revised implementation plan
> **Repository:** `aureat/phalcom-lang`
> **Grounded revision:** `1b7230c1f9df11097114621a7b26182ba88f5012` (`main`, inspected 2026-08-23)
> **Supersedes:** the implementation-state assumptions and sequencing in `phalcom-canonical-native-universe-bootstrap-implementation-spec.md` grounded at `edbced8914b924f95f4dbc83508c93ca77cd36ce`
> **Normative design inputs:** Spec 03.5 canonical core/native surface; Spec 04 user-facing type syntax/lowering; Spec 04.5 expression typing/inference/flow/explanations; `@native`, `@internal`, `@class`, and Rust `#[primitive(...)]` specifications
> **Primary scope:** finish canonical native declaration generation, source/native semantic conformance, verified bootstrap, runtime typing attachment, universe-source convergence, LSP migration, primitive migration, and deletion of compatibility authorities
> **Verification note:** this revision is based on source inspection at the grounded commit. No fresh local `cargo test`, build, or REPL run is claimed here. GitHub exposed no commit-status checks for the grounded SHA during this archaeology pass. “Implemented” below therefore means “implementation is present in the repository”, not “freshly verified green by this document.”

---

## 0. Revision contract

This document is not a second design of the native-universe architecture. It revises the older execution plan after substantial Spec-03.5, Spec-04, and early Spec-04.5 implementation landed.

The most important correction is that the repository now contains a canonical semantic type/signature substrate strong enough that this project must **reuse it**, not create another native-contract type system.

The older plan proposed several pieces that are now either implemented, partially implemented, or architecturally superseded. This revision makes those changes explicit.

### 0.1 Superseded implementation assumptions

The following assumptions from the older plan are no longer current:

| Older-plan assumption | Current repository state | Revised action |
|---|---|---|
| Create `phalcom-native-decl` | crate exists and is used by the scanner and proc macro | finish convergence; do not recreate |
| Create generated native-surface pipeline | `phalcom-native-surface-gen` and checked-in `generated.rs` exist | complete rich generation and parity; keep current generator architecture |
| Native class hierarchy is duplicated in surface tables | canonical `UNIVERSE_CLASS_RELATIONS` exists in `phalcom-native-meta` and is consumed by runtime/semantic code | make it the primordial hierarchy authority; delete compatibility class tables when consumers migrate |
| Semantic checker needs a native typed-surface implementation | `phalcom-semantic::types::native::register_native_surfaces` imports rich native metadata and produces canonical callable signatures | extend this path; do not create a parallel native type checker |
| Create a new `NormalizedContractType` IR | Spec 04 now has canonical `TypeStore`, `TypeFormResolution`, type lambdas, kinds, generic signatures, and `CallableSemanticSignature` | reject this proposal; source/native conformance must compare canonical semantic signatures/types |
| Inference variables may appear in canonical types | Spec 04.5 has already separated `InferVarId` from `TypeId`; `TypeData::Infer` is absent from current `TypeStore` | native contracts must contain only publishable canonical type terms; no inference-specific native IR |
| Add expression identity as future work | `BodyId`, `LocalExpressionId`, and `ExpressionId` exist | not part of this plan |
| Add 04.5 analysis product models | `ExpressionAnalysis`, `BindingState`, statuses, flow graph, explanation arena, and `InferenceSession` scaffolding exist | do not duplicate them; only depend on the portions relevant to source-signature publication |
| Create `phalcom-core/src/native/source.rs` and `verify.rs` as semantic authority | `phalcom-semantic/src/core_surface/{source,merge,conformance,native,presentation}.rs` already exists | upgrade `phalcom-semantic::core_surface`; keep `phalcom-core` as bootstrap consumer/adaptor |
| Generate surface via `phalcom-native-surface/build.rs` | repository chose a standalone `phalcom-native-surface-gen` executable and checked-in generated artifact | preserve this architecture unless separately re-ratified |
| Add runtime implementation provenance | `MethodImplementationIndex` exists and descriptor installation populates it | treat implementation provenance as landed; remaining runtime work is semantic-callable attachment |
| Add canonical bootstrap relation verification | `UNIVERSE_CLASS_RELATIONS` and `spec03_5_census` runtime relation checks exist | retain and strengthen, not redesign |

### 0.2 Still-valid architectural decisions

The following older-plan decisions remain valid and are preserved:

- canonical `.ph` source and Rust `#[primitive(...)]` are intentionally independent authored declarations that must be mechanically reconciled;
- native type metadata never participates in selector identity or runtime dispatch;
- source `@native` is an assertion/presentation declaration, not a Rust binding mechanism;
- declaration-only native members and reference-bodied native members must emit no source method implementation;
- real `.ph` wrappers over native primitives remain ordinary executable source;
- `@internal` is an assertion over the `_$`/`__` namespaces, not an authorization capability;
- `@native class` completes a primordial runtime identity and must not allocate a second class;
- the VM remains free to keep static semantic metadata in side tables instead of bloating `MethodObject`;
- LSP remains VM-free;
- the legacy runtime installer and legacy LSP/native tables are migration compatibility only and require measurable deletion gates;
- source/native mismatch must be diagnosed, not resolved by silently choosing whichever layer happens to be newer.

### 0.3 New governing rule from Specs 04/04.5

The revised plan adopts one additional invariant:

> **Native/source contract conformance is a canonical semantic-signature comparison, not a second parser/type-normalization subsystem.**

Conceptually:

```text
source MethodDef / GetterDef / SetterDef
        ↓ Spec-04 declaration publication
CallableSemanticSignature
        ↓
        ├──────── compare by CallableId + canonical semantic contract ────────┐
        ↓                                                                     ↓
native PrimitiveSurfaceSpec                                     native semantic importer
                                                                      ↓
                                                        CallableSemanticSignature
```

Any proposed implementation that introduces another semantic type tree solely for native-source comparison should be rejected unless the canonical semantic model demonstrably cannot represent the required information.

---

# Part I — Current Repository State

## 1. Grounded implementation status

The current repository is significantly further along than the older plan assumed, but it is still a migration architecture.

### 1.1 Status vocabulary

This document uses:

- **Implemented** — repository contains the target-shaped implementation for this responsibility.
- **Partial** — target abstraction exists, but production consumers or semantic coverage are incomplete.
- **Compatibility** — deliberately transitional path still active.
- **Not implemented** — no current implementation of the required behavior was found.
- **Blocked** — work should not be completed until a prerequisite semantic gate lands.

### 1.2 Current-state matrix

| Area | State | Repository evidence | Consequence |
|---|---|---|---|
| Canonical `TypeStore` | Implemented foundation | `phalcom-semantic/src/types/store.rs`; `TypeData` has canonical forms and no inference variant | native verifier must use canonical types |
| Solver-local inference | Implemented foundation, not fully integrated | `phalcom-semantic/src/checker/inference.rs`; `InferVarId != TypeId` | native declarations must never publish solver vars |
| Expected-type model | Implemented foundation | `checker/expected.rs::ExpectedType` | not a native-verifier concern except call typing later |
| Expression/body analysis product types | Implemented foundation | `checker/analysis.rs`, `identity.rs` | no need to add another analysis identity layer |
| Formal flow graph/explanation arena | Implemented scaffolding | `checker/flow/graph.rs`, `explain/arena.rs` | orthogonal to native bootstrap |
| Monolithic checker | Compatibility/current production path | `checker/context.rs::CheckingContext` | full 04.5 migration is incomplete but does not block most native work |
| Source type-form lowering | Partial-to-strong | `types/annotation.rs::resolve_type_form`, `resolve_generic_signature` | use it; do not normalize source types independently |
| Source declaration generic publication | Incomplete | `workspace.rs` predeclares source classes as `KindId::TYPE`, `generic_signature: None`; `checker/declaration.rs` ignores method generics/where clauses | strict generic native/source conformance must wait on this gate |
| Type-lambda source lowering | Partial | AST/parser and semantic lambda nodes exist; current lowering does not yet bind lambda parameters capture-safely | native anchors using type lambdas are blocked until Spec-04 S5 semantics are complete |
| Row-tail source lowering | Incomplete | record tail exists in AST; `resolve_type_form` ignores `tail` | open-row native anchors are blocked |
| Canonical callable semantic signature | Implemented | `phalcom-semantic/src/signature.rs` | make this the comparison unit |
| Rich native semantic import | Implemented foundation | `types/native.rs::register_native_surfaces` | extend rather than replace |
| Semantic core-surface merge/presentation | Partial | `phalcom-semantic/src/core_surface/*` | upgrade this into source/native conformance authority |
| Shared Rust primitive declaration crate | Partial | `phalcom-native-decl` exists; macro still duplicates parser/validation | finish convergence |
| Rich generated surface artifact | Partial | `phalcom-native-surface/src/generated.rs`; declaration count marker = 93 at grounded revision | generation is not yet full-authority |
| Surface generator | Partial | `phalcom-native-surface-gen/src/main.rs` scans descriptors but currently validates count/artifact boundary rather than generating all rich records | finish deterministic rich emission |
| Handwritten native compatibility tables | Compatibility | `phalcom-native-surface/src/lib.rs::{NATIVE_CLASSES,NATIVE_MEMBERS,NativeReturnShape}` | delete after consumer parity |
| Descriptor runtime installer | Implemented | `phalcom-core/src/native/install.rs` | target runtime installer |
| Legacy runtime installer | Compatibility, active by default | `VM::new_with_native_install_mode`; `Universe::install_primitives` remains active under Dual/incomplete floor | migrate and delete by parity gate |
| Implementation provenance reflection | Implemented | `MethodImplementationIndex`; descriptor installer populates it | do not re-plan |
| Runtime semantic callable attachment for native methods | Not implemented | `MethodSemanticIndex` exists, installer does not insert `RuntimeCallableRef` | still required |
| Metadata pool allocation safety | Incomplete | registry allocates IDs by `pools.len()` while program materialization loads bundle with `MetadataPoolId(0)` before registration | must be fixed before adding a native metadata pool |
| Canonical runtime class relations | Implemented | `UNIVERSE_CLASS_RELATIONS`; semantic hierarchy and runtime tests consume it | source class anchors verify against this authority |
| `MemberBody::Declaration` | Not implemented | Method/Getter/Setter bodies remain `Vec<Statement>` | still required for declaration-only native members |
| `@internal` builtin | Not implemented | `BuiltinAttr` still lacks `Internal` | still required |
| Attribute argument policy | Not implemented | `AttributeExpander` exposes no central argument-policy method | still required |
| Class-level `@native` | Not implemented | target spec exists; compiler support remains member-oriented | still required |
| Source/native full structural verifier | Partial scaffold only | `core_surface/conformance.rs` resolves native types but does not compare source anchors/signatures | major remaining work |
| Provider-backed universe source | Implemented source authority | `phalcom-modules/src/builtin.rs::BuiltinProjectSourceProvider` | runtime/LSP should converge on it |
| Runtime source loading via provider | Not implemented | `VM::run_universe_modules` still owns `static SOURCES` + `include_str!` | remove duplication |
| LSP actual universe-project model | Not implemented | `semantic/core_source.rs` still embeds `core/core.ph` | migrate |
| LSP rich semantic native merge | Partial | consumes `NATIVE_SURFACE_CATALOG`, but source wins instead of source/native merge; still falls back to `NATIVE_MEMBERS` | migrate to semantic core-surface product |
| LSP implementation-field classification | Bug present | `surface.rs` tests `field.name.starts_with("_$")` | fix to `__` |
| Shared AST selector projection | Not implemented | LSP owns `selectors::selector_from_member`; semantic/compiler construct selectors separately | centralize |
| Bootstrap preflight before installation | Not implemented | bootstrap installs legacy/descriptors before compiling universe source | still required |
| Fallible VM bootstrap seam | Not implemented | `VM::new`/`new_with_native_install_mode` expect/panic through bootstrap | still recommended |
| Strict source-anchor ↔ required-descriptor bijection | Not implemented | census is observational; no anchor policy exists | final migration gate |

---

## 2. What Specs 04 and 04.5 changed for this project

The previous plan was correct to anticipate a source/native type-normalization problem, but the solution has changed because the canonical semantic platform is now substantially implemented.

### 2.1 No `NormalizedContractType`

Do **not** add the previously proposed:

```rust
pub enum NormalizedContractType { ... }
```

That would create a third type language:

```text
AST type syntax
canonical TypeStore
native-contract normalized types   <-- duplicate semantic authority
```

The repository now already provides the correct convergence point:

```text
source TypeAnnotation
    ↓ resolve_type_form / declaration publication
TypeId / TypeTerm

native TypeExprSpec
    ↓ resolve_native_type_form
TypeId / TypeTerm
```

The comparison must therefore happen after both sides have entered one `TypeStore` and one callable-signature model.

### 2.2 `InferVarId != TypeId` is already real

`TypeData` no longer contains an inference variant. `InferenceSession` owns solver-local `InferVarId` and `InferenceTerm` values.

That gives the native verifier a simple invariant:

> Every source/native declaration contract admitted to conformance must be materialized as canonical publishable `TypeTerm`/`TypeId` data. Unsolved inference terms are not legal contract metadata.

The native verifier therefore does not need its own “unknown metavariable” representation.

### 2.3 Full 04.5 is not a prerequisite

The following 04.5 work is **not** required before native universe verification can progress:

- flow-state narrowing;
- mutable-binding joins;
- loop fixed points;
- expected-result inference in ordinary call sites;
- explanation graph rendering;
- causal diagnostic suppression;
- LSP formal flow migration.

Native/source contract verification deals with declarations, not inferred local program state.

The real prerequisite is narrower:

> Source declarations must publish canonical declaration and callable signatures, including generic binders/constraints, into the same semantic tables consumed by native metadata.

### 2.4 Spec-04 publication gates relevant to native verification

The native plan depends on these exact semantic gates:

| Spec-04 capability | Current state | Native-plan dependency |
|---|---|---|
| S1 core type-form parsing | substantially implemented | use directly |
| S2 explicit lowering outcomes | implemented in `TypeFormResolution`, though some unsupported forms still collapse conservatively | conformance must preserve mismatch vs blocked/unknown distinctions |
| S3 generic binders/kinds | AST + resolver helpers exist; workspace declaration publication incomplete | required for generic native class/method anchors |
| S4 `where` constraints | AST + `resolve_generic_signature` exist; callable publication incomplete; native metadata cannot yet express constraints | required before constrained generic native anchors |
| S5 type lambdas | AST/canonical lambda infrastructure exists; source lowering is not yet capture-safe | block such anchors until fixed |
| S6 generic superclass / `Self` | canonical support exists in pieces; workspace publication incomplete | required for native class declaration conformance where used |
| S7 aliases / rows | aliases/row AST exist; row tails are not semantically lowered | do not accept open-row conformance yet |
| S8 type-form values | unrelated to declaration contract matching | not a native-bootstrap blocker |
| S9 native/source convergence | partially implemented via `types::native` + `core_surface` | this revised plan completes it |

### 2.5 Current source-publication defect to fix in Spec 04, not duplicate here

`analyze_workspace` currently creates every source class as if it were monomorphic:

```rust
DeclarationTypeInfo {
    kind: KindId::TYPE,
    generic_signature: None,
    supertype_template: None,
    ...
}
```

and `checker/declaration.rs::register_class_surface` lowers method parameter/return annotations into the older `dispatch::CallableSignature` without method generic binders or `where` constraints.

Meanwhile native import already publishes richer `CallableSemanticSignature` records into `CallableSignatureTable`.

The revised native plan therefore introduces a hard dependency gate:

```text
SOURCE-SIGNATURE-PUBLICATION

For every source callable that can participate in native conformance:
    CallableSignatureTable.get(CallableId)
        returns one canonical source CallableSemanticSignature
        with generics, parameter TypeTerms, return TypeTerm,
        labels/rest shape, source span, and implementation provenance.
```

This gate should be implemented as part of Spec 04 declaration publication, not as a native-only duplicate lowering path.

---

# Part II — Target End State

## 3. Canonical authority graph

The target architecture after revising for Specs 04/04.5 is:

```text
                    ┌───────────────────────────────┐
                    │ canonical Phalcom universe .ph │
                    │ @native / @internal / @class │
                    │ types / generics / where     │
                    │ Phaldoc / reference body     │
                    └──────────────┬────────────────┘
                                   │
                                   │ parser + Spec-04 semantic publication
                                   ▼
                         CallableSemanticSignature
                                   │
                                   │
Rust #[primitive(...)]             │
        │                          │
        ├── proc macro ────────────┼──────────► PRIMITIVES
        │                          │
        └── surface generator ─────┼──────────► NATIVE_SURFACES
                                   │
                                   ▼
                         native semantic importer
                                   │
                                   ▼
                         CallableSemanticSignature
                                   │
                         canonical conformance
                                   │
                                   ▼
                         VerifiedCoreSurface
                         /        |          \
                        /         |           \
                       ▼          ▼            ▼
                 VM bootstrap   metadata      LSP/presentation
                     │           export            │
                     ▼              │              ▼
               descriptor install  │       real universe source navigation
                     │              │
                     ├──────────────┘
                     ▼
             MethodSemanticIndex
             MethodImplementationIndex
```

### 3.1 Three categories remain separate

The target must preserve:

```text
semantic truth
    CallableSemanticSignature / declaration type metadata

runtime projection
    MethodObject + runtime typing/provenance side tables

implementation provenance
    NativePrimitive / Source / Generated / Intrinsic / Rust source location
```

The runtime implementation does not define semantic truth merely because it executes the call. Conversely, source presentation does not install a runtime method merely because it describes one.

### 3.2 Static typing still does not select runtime methods

Even after source/native signatures fully converge:

```text
selector identity = ordinary selector shape
runtime dispatch   = ordinary Phalcom message dispatch
```

The following never become dispatch keys:

```text
parameter types
generic arguments
where constraints
effects
raises
proof results
expected types
inference solutions
```

---

# Part III — Remaining Workstreams

## 4. Workstream A — Finish the shared Rust primitive declaration authority

### Goal

Make `phalcom-native-decl` the one parser/validator for authored `#[primitive(...)]` metadata and make both proc-macro expansion and generated surface emission consume it without semantic duplication.

### Current state

`phalcom-native-decl` exists and `phalcom-native-surface-gen` calls `parse_primitive_attribute`. The proc macro also calls that parser as a validation pass, but then reparses the attribute using its own `PrimitiveAttrArgs` and repeats selector, visibility, lifecycle, ABI, flow, effect, and type handling.

This is still two interpretations of one attribute grammar.

### Required changes

#### A1. Evolve `NormalizedPrimitiveDecl` into the complete validated declaration

Files:

```text
phalcom-native-decl/src/lib.rs
phalcom-native-decl/src/normalized.rs
phalcom-native-decl/src/validate.rs
phalcom-native-decl/src/parser.rs   (or current equivalent)
```

The normalized form must carry every field required to emit `PrimitiveSurfaceSpec` and `PrimitiveDescriptor` metadata without reparsing strings in the proc macro.

The proc macro may still own **Rust function ABI validation**, because that depends on `syn::ItemFn`, but it should not own a second primitive-metadata parser.

Target split:

```text
phalcom-native-decl
    parse syntax
    normalize selector
    parse symbolic type/callable syntax
    validate metadata cross-field invariants
    return validated owned declaration

phalcom-native-macros
    validate Rust ItemFn ABI against declaration
    emit static metadata
    emit PrimitiveDescriptor

phalcom-native-surface-gen
    scan ItemFn attributes/docs
    call same declaration parser
    emit VM-free records
```

#### A2. Strengthen implementation visibility

The current shared validator and macro still allow:

```text
_$selector + visibility = public
```

when visibility is explicitly supplied.

Final invariant:

```text
selector starts with "_$"
    ⇔ NativeVisibility::Internal
```

More precisely:

```text
_$ + omitted visibility     => error
_$ + public                 => error
_$ + internal               => valid
ordinary + internal         => error
ordinary + public/omitted   => valid
```

Compile-time failure is preferable to bootstrap discovery for a fact fully known to the proc macro.

#### A3. Add explicit anchor policy

`PrimitiveSurfaceSpec` still has no source-anchor completeness policy.

Add a small explicit policy in `phalcom-native-meta`:

```rust
pub enum NativeAnchorPolicy {
    Required,
    Hidden,
}
```

and:

```rust
pub anchor: NativeAnchorPolicy,
```

Default authored primitive policy should be `Required`.

`Hidden` is only for a runtime method intentionally absent from canonical language-facing source. It must not become a convenient escape from migration work.

Do **not** create a separate checked-in exemption list. The exception belongs to the primitive declaration itself.

#### A4. Audit the native symbolic type grammar against Spec 04

Current `phalcom-native-meta::types` can represent:

- `Never`;
- `SelfType` symbolically;
- universe nominals;
- generic parameters;
- applications;
- unions;
- tuples;
- callable-level generic parameters with kinds.

It cannot yet represent all current Spec-04 forms. In particular, `KindSpec` currently has only `Type` and arrow kinds (no `RecordRow`), `TypeExprSpec` has no open-record/row node, type-lambda node, or nested callable node, and primitive callable metadata has no `where`-constraint representation. `UniverseTypeFormSpec` also does not currently encode declaration-site variance.

Do not preemptively clone the entire source grammar into native metadata. Instead:

1. census the actual native primitive signatures;
2. identify forms genuinely required by a native contract;
3. extend `phalcom-native-meta` only for those semantic forms;
4. lower both source and native forms to the same canonical semantic model;
5. report `Blocked(NativeMetadataFormUnavailable)` instead of degrading an unsupported written source contract to `Unknown` during conformance.

### Tests first

Add/strengthen tests in `phalcom-native-decl` and proc-macro compile-fail coverage:

```text
shared parser round-trips all supported fields
proc macro and generator consume identical normalized declaration
_$ + public                    rejected
_$ + internal                  accepted
ordinary + internal            rejected
anchor omitted                 Required
anchor = hidden                accepted
invalid anchor                 rejected
duplicate metadata field       rejected
selector/parameter lane drift  rejected
callable-vs-params drift        rejected
```

### Completion gate

Workstream A is complete when the proc macro no longer contains an independent metadata grammar/semantic validator and generated records can be emitted solely from the shared normalized declaration.

---

## 5. Workstream B — Complete deterministic native-surface generation

### Goal

Turn `phalcom-native-surface/src/generated.rs` into actual deterministic generated output from every authored `#[primitive]`, and reduce `phalcom-native-surface` to a stable API/index over that output.

### Current state

The repository chose a standalone generator:

```text
phalcom-native-surface-gen/src/main.rs
```

rather than the older plan's proposed `build.rs`.

That is a good architecture to keep. It avoids making `phalcom-native-surface`'s build script reach upward into `phalcom-core/src/primitive`, and it allows checked-in generated output with an explicit drift check.

However, the current generator only:

- scans primitive Rust files;
- parses authored declarations;
- checks duplicate keys;
- checks that `generated.rs` looks generated;
- checks the authored declaration-count marker.

It explicitly does **not** yet generate the complete rich record file.

### Required changes

#### B1. Make the generator emit all rich records

Command target:

```bash
cargo run -p phalcom-native-surface-gen -- --root .
```

should deterministically replace:

```text
phalcom-native-surface/src/generated.rs
```

with all required:

- static type expression nodes;
- parameter tuples;
- callable specs;
- lifecycle/effect/raise/flow data;
- `NativeSurfaceRecord`s;
- declaration count;
- a deterministic generator/schema fingerprint if useful.

Sort by:

```text
(owner UniverseKey, side, canonical selector)
```

not filesystem enumeration order.

#### B2. Make `--check` compare the full artifact

`--check` must regenerate in memory and byte/structure-compare with checked-in `generated.rs`.

A count marker alone cannot detect:

- changed return type;
- changed visibility;
- changed effect;
- changed lifecycle;
- changed selector with equal total count;
- changed trust/intrinsic metadata.

#### B3. Keep current rich catalog API

Retain the useful landed APIs:

```text
NativeSurfaceRecord
NativeSurfaceId
NativeSurfaceCatalog
NATIVE_SURFACE_CATALOG
catalog_fingerprint
validate_native_surface_catalog
```

They are the correct VM-free projection boundary.

#### B4. Do not preserve `NativeReturnShape` as semantic authority

`NativeReturnShape`, `NativeMember`, and `NATIVE_MEMBERS` are now compatibility structures.

The semantic layer already consumes actual `TypeExprSpec`, `ReturnFlowSpec`, and canonical `TypeId`s. New code must not add dependencies on `NativeReturnShape`.

### Completion gate

```text
set(generator-authored primitive keys)
    == set(NATIVE_SURFACES keys)
```

and full generated record contents are reproducible by `--check`.

Only after that should the project begin deleting handwritten compatibility rows.

---

## 6. Workstream C — Add declaration-only member bodies and `@internal` without reopening Spec-04 grammar design

### Goal

Implement the source declaration mechanics already normatively specified for canonical native source, while leaving type grammar/generic syntax ownership with Spec 04.

### Current state

- method/getter/setter bodies are still `Vec<Statement>`;
- there is no declaration-only body state;
- `BuiltinAttr::Internal` does not exist;
- `AttributeExpander` has no central argument policy;
- `@native` target spec permits class-level use, but compiler support remains limited;
- current source-native presentation scaffolding already recognizes member `@native` conceptually.

### Required changes

#### C1. Introduce explicit callable body presence

Files:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-core/src/compiler/**
phalcom-semantic/src/checker/declaration.rs
phalcom-lsp/src/semantic/** compatibility consumers
```

Recommended shape remains:

```rust
pub enum MemberBody {
    Declaration,
    Statements(Vec<Statement>),
}
```

or a naming-equivalent enum.

Do not use `Vec::is_empty()` as declaration semantics.

These remain distinct:

```phalcom
@native
foo() -> Unit
```

```phalcom
foo() -> Unit {}
```

The parser should understand declaration bodies structurally; semantic/compiler legality decides whether they are allowed.

#### C2. Add `BuiltinAttr::Internal`

Implement the already-written `@internal` spec exactly:

```text
_$selector  implementation selector
__field     implementation field
@internal   explicit assertion
```

No privilege is granted by the attribute.

#### C3. Add central attribute argument policy

The current `AttributeExpander` API does not centrally reject arguments for no-argument attributes.

Add a reusable policy such as:

```rust
pub enum AttributeArgPolicy {
    Any,
    None,
    Exact(usize),
    AtLeast(usize),
}
```

At minimum enforce `None` for:

```text
@native
@internal
@class
@private
@protected
@constructor
```

Do not alter contract attributes that legitimately carry expressions.

#### C4. Run declaration-integrity checks before subtractive `@native` lowering

This is mandatory because `@native` removes a source member from executable lowering.

Order:

```text
parse declaration
    ↓
validate attribute target/arguments
    ↓
validate namespace + privilege + native-source authority
    ↓
record semantic/source anchor
    ↓
only then remove native member from executable lowering
```

Never allow an invalid `@native` declaration to disappear before validation.

#### C5. Restrict `@native` to trusted native-source authority

The current architecture must not let arbitrary user code spell `@native` and silently lose a method body.

Until a separate extension/FFI authority is designed:

```text
ordinary source + @native => error
trusted canonical universe/native source + @native => legal
```

Trust must be based on resolved module/project identity, not spelling.

#### C6. Add class-level `@native`

Class-level `@native` means:

```text
source declaration = completion/presentation of an existing primordial class
not fresh allocation
```

The compiler/runtime must verify `UniverseKey` and existing `ClassId` rather than create a second identity.

### Spec-04 interaction

This workstream must not add or alter:

- generic binder syntax;
- kind syntax;
- `where` syntax;
- type-lambda syntax;
- row syntax;
- value-space type-form grammar.

It only changes whether a class member has an executable source body and adds declaration metadata attributes.

### Completion gate

Declaration-only native members parse, remain present for semantic/source indexing, and cannot reach executable lowering. Reference-bodied native members are likewise retained for tooling but removed from runtime source installation.

---

## 7. Workstream D — Finish canonical source declaration/signature publication (Spec-04 dependency gate)

### Goal

Ensure source declarations and native declarations enter the same compiler-owned semantic tables before conformance.

### Ownership

This is principally a **Spec-04 completion task**, not native-specific semantics. It is included because strict native/source verification depends on it.

### Current defects

`workspace.rs` currently predeclares source classes as monomorphic even when AST generic binders exist. `checker/declaration.rs` publishes legacy dispatch signatures but not source `CallableSemanticSignature`s.

### Required source publication pipeline

#### D1. Publish declaration generic form before member signatures

For every `ClassDef`:

1. predeclare `DeclarationId` shell;
2. resolve generic parameter kinds and variance;
3. intern `TypeParameterId`s by owner/index;
4. compute declaration kind;
5. publish canonical nominal form;
6. resolve class `where` constraints;
7. resolve generic superclass template;
8. replace provisional monomorphic declaration info.

The final `DeclarationTypeInfo` must reflect source syntax instead of hardcoding:

```text
kind = Type
generic_signature = None
supertype_template = None
```

#### D2. Publish `CallableSemanticSignature` for every source callable

Extract one shared source-signature publisher rather than teaching each consumer to lower AST annotations separately.

Conceptual API:

```rust
pub fn publish_source_callable_signature(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    owner: &DeclarationId,
    member: &ClassMember,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> SourceCallablePublication;
```

For methods it must include:

```text
CallableId
selector
side
method-local GenericSignature
external labels
rest modes
parameter TypeTerms
return TypeTerm
source spans
implementation = Source   // later merge may project NativePrimitive
```

#### D3. Receiver and method generic scopes remain separate

Do not flatten:

```text
class generic environment
method generic environment
```

into one anonymous substitution map.

That is required both for ordinary 04.5 generic-call inference and for native conformance.

#### D4. Preserve explicit unsupported/blocked states

If source annotation lowering reaches a form whose Spec-04 semantics are not complete—currently notable examples are open row tails and capture-sensitive type lambdas—the publisher should not pretend the declaration is a canonical `Unknown` contract and allow native verification to pass.

It should produce a structured blocked/unavailable publication state that the native verifier can surface as:

```text
native.source_contract_blocked
```

with the underlying Spec-04 reason.

### Tests

Add source semantic publication tests for:

```text
class Box<T>
class Box<T :: Type>
method <T>
method where T <: U
source parameter/return applications
Self in instance/class context
generic superclass templates
alpha-stable owner/index parameter identity
invalid type syntax remains invalid/blocked, not an accepted Unknown contract
```

### Completion gate

For every source callable eligible to be a native anchor:

```rust
snapshot.callable_signatures.get(&callable_id).is_some()
```

with all declared generics and constraints faithfully represented.

---

## 8. Workstream E — Upgrade `phalcom-semantic::core_surface` into the conformance authority

### Goal

Reuse the landed `core_surface` modules and turn them from presentation scaffolding into the authoritative source/native semantic reconciliation layer.

### Do not create

Do not create a second primary semantic verifier under:

```text
phalcom-core/src/native/source.rs
phalcom-core/src/native/verify.rs
```

Thin bootstrap adapters in `phalcom-core` are acceptable, but semantic truth belongs in `phalcom-semantic`.

### Current state

`phalcom-semantic/src/core_surface` already contains:

```text
source.rs        source extraction
merge.rs         source/native collision model
conformance.rs   native type resolvability checks
native.rs        native import adapter
presentation.rs  merged presentation IR
```

This is the correct ownership boundary, but it is incomplete:

- source extraction manually rebuilds selectors;
- method side detection is not robust against unexpanded `@class` in all cases;
- source records omit canonical parameter/return/generic contracts;
- docs are placeholders;
- conformance validates native rows internally but does not compare source to native;
- merge identity is partly string-based;
- presentation can describe native/source provenance but is not backed by a verified semantic contract.

### Required changes

#### E1. Centralize AST member selector projection

Create a VM-free shared selector projection, preferably in `phalcom-ast` because the input is AST and output is `phalcom-common::selector::Selector`.

Conceptual API:

```rust
pub fn selector_from_member(member: &ClassMember)
    -> Result<Selector, SelectorProjectionError>;
```

Cover:

- methods;
- getters;
- setters;
- subscript get/set;
- labels;
- rest shapes.

Then migrate:

```text
phalcom-lsp/src/selectors.rs
phalcom-semantic/src/core_surface/source.rs
checker/declaration.rs selector construction where practical
compiler duplicate-selector validation where practical
```

The native verifier must not compare selectors built by a fourth implementation.

#### E2. Replace source presentation records as the type authority

`SourceMemberRecord` may remain useful for docs/body/reference information, but contract equality should use canonical signatures.

Recommended shape:

```rust
pub struct SourceCoreMember {
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub body_kind: SourceBodyKind,
    pub source: SemanticSourceSpan,
    pub documentation: Option<...>,
    pub native_marker: bool,
    pub internal_marker: bool,
}
```

Do not duplicate parameter type trees inside this record.

#### E3. Import native rows as canonical callable signatures

Reuse/extend:

```text
phalcom-semantic/src/types/native.rs::register_native_surfaces
NativeSurfaceImportReport.callable_signatures
```

Enhance generic native import when needed:

- create method-owned parameters by `CallableId` + index;
- respect `CallableTypeSpec.type_params` kinds;
- lower `SelfType` with owner/side/role semantics instead of falling back to opaque when enough context exists;
- preserve unsupported metadata as a structured failure, not silent loss.

#### E4. Compare semantic contracts structurally

Key identity:

```text
CallableId = (owner declaration, side, Selector)
```

For a source `@native` anchor and native row compare:

```text
selector kind/slots                exact
side                               exact
fixed parameter count              exact
labels                             exact
rest mode                          exact
parameter semantic TypeTerm        equivalent under canonical parameter identity
return semantic TypeTerm           equivalent
method generic arity/kinds         exact
method generic constraints         exact when native metadata can represent them
internal visibility assertion      exact
```

Do **not** require source local parameter names to equal synthetic/native names. Source local names are presentation/source metadata.

Do **not** compare types by display string.

#### E5. Make class conformance semantic too

For `@native class`, verify against canonical bootstrap metadata:

```text
UniverseKey                         exact
primordial runtime identity exists exact
superclass relation                 UNIVERSE_CLASS_RELATIONS
class generic arity/kinds           UNIVERSE_TYPE_FORMS / DeclarationTypeTable
class variance                      canonical generic metadata once representable
source class is completion           not fresh allocation
```

This is a new strengthening made possible by Spec 04's generic declaration model.

If canonical native metadata cannot yet represent a source generic property such as variance, that is a metadata-model gap; do not silently ignore the source declaration.

#### E6. Introduce explicit conformance outcome states

Recommended:

```rust
pub enum CoreConformanceOutcome {
    Verified,
    Mismatch(CoreContractMismatch),
    Blocked(CoreConformanceBlockReason),
}
```

Examples of `Blocked`:

```text
SourceSignatureNotPublished
SourceTypeLambdaLoweringIncomplete
OpenRowLoweringIncomplete
NativeMetadataCannotRepresentConstraint
NativeMetadataCannotRepresentTypeForm
```

Bootstrap strict mode eventually rejects both `Mismatch` and `Blocked` for required anchors. Migration tooling may report blocked entries without pretending they are verified.

#### E7. Merge source/native into one semantic callable

A verified source `@native` anchor should project as one callable:

```text
semantic contract        canonical verified signature
source span/docs/names   source
implementation kind      NativePrimitive
native id                generated native row
Rust provenance          PrimitiveDescriptor/NativeSourceSpec
reference body           source tooling only
```

The source signature should not “win” and cause native metadata to be skipped, as the current LSP compatibility path does.

### Diagnostics

Use structured mismatch categories:

```text
native.orphan_anchor
native.missing_anchor
native.duplicate_anchor
native.class_identity_mismatch
native.superclass_mismatch
native.generic_signature_mismatch
native.selector_mismatch
native.side_mismatch
native.visibility_mismatch
native.parameter_shape_mismatch
native.parameter_type_mismatch
native.return_type_mismatch
native.constraint_mismatch
native.source_contract_blocked
```

A useful argument mismatch should show both source and native evidence, while normal output remains concise.

### Completion gate

Every matched source/native callable produces one `VerifiedCoreSurface` record backed by canonical semantic signatures. No conformance decision depends on `NativeReturnShape` or source/native type strings.

---

## 9. Workstream F — Converge universe declaration identity

### Goal

Eliminate the semantic split between the temporary legacy `ModuleId::core()` identity and real builtin-universe module declarations before strict source navigation/metadata identity is considered complete.

### Why this is now explicit

Current semantic bootstrap still does:

```text
UniverseKey -> DeclarationId::new(ModuleId::core(), key.name())
```

while the actual universe source corpus is organized as builtin modules such as:

```text
builtin:universe:scalar.number
builtin:universe:scalar.string
builtin:universe:option.option
...
```

`ModuleId::core()` is explicitly documented as a temporary compatibility identity.

If this is left unresolved, source `String` and native semantic `String` can be structurally “the same class” in human terms while carrying different compiler identities. That undermines strict `CallableId` conformance and durable metadata source identity.

### Required changes

#### F1. Build a canonical `UniverseDeclarationIndex`

Compiler-owned semantic product:

```rust
pub struct UniverseDeclarationIndex {
    by_key: BTreeMap<UniverseKey, DeclarationId>,
}
```

Source of mapping:

- canonical linked builtin universe source declarations;
- `@native class` anchors for primordial classes;
- canonical `UniverseKey` names;
- package/module linkage.

The mapping is semantic/static. It does not change runtime `ClassId` identity.

#### F2. Parameterize universe bootstrap declarations with the index

`bootstrap_universe_declarations` already accepts a `UniverseKey -> DeclarationId` resolver. Stop universally passing `ModuleId::core()` once actual universe declarations are available.

Transitional unit tests may still use the core compatibility resolver when no universe source graph is present.

#### F3. Preserve one runtime class object

This identity convergence must not allocate a new runtime class.

The relation is:

```text
UniverseKey::String
    ├─ semantic DeclarationId = builtin universe source declaration
    └─ runtime ClassId        = existing primordial String class
```

#### F4. Use source identity for metadata/navigation

After convergence:

- `CallableId.owner` uses real universe source declaration identity;
- source spans point into the actual `phalcom://universe/...` module;
- durable metadata uses stable builtin project/module identity;
- LSP navigation no longer needs a fake single core module to make owner identity line up.

### Completion gate

There is one compiler declaration identity per canonical universe class, and source/native signatures use it consistently.

---

## 10. Workstream G — Verified bootstrap preflight

### Goal

Make runtime bootstrap consume a verified semantic/native universe before native installation and source execution.

### Current state

Current ordering remains effectively:

```text
construct VM/kernel
install core globals
stamp fixed layouts
legacy primitive install (Dual/incomplete floor)
descriptor primitive install
finalize base names
compile+run hard-coded universe source list
post-bootstrap roots/invariants
```

There is no source/native preflight before installation.

### Required target phases

#### G0. Fallible bootstrap API

Introduce a testable fallible seam:

```rust
pub fn try_new() -> Result<Self, BootstrapError>
```

or equivalent, preserving:

```rust
pub fn new() -> Self {
    Self::try_new().expect("VM bootstrap must succeed")
}
```

Do not force native/source contract failures through generic runtime `PhError` if a dedicated `BootstrapError` gives better structure.

#### G1. Primordial runtime construction

Build:

- heap;
- object/class/metaclass tower;
- primordial class identities;
- canonical runtime class relations;
- fixed representation layouts that source cannot own;
- core/universe package roots required for source resolution.

No source methods execute here.

#### G2. Validate descriptor registry

Validate:

- unique `PrimitiveKey`;
- selector canonicality;
- visibility invariant;
- lifecycle/intrinsic invariants;
- generated-surface fingerprint/parity in development/test paths as appropriate.

#### G3. Load canonical universe source through `BuiltinProjectSourceProvider`

Do not use a second `include_str!` list.

The provider already owns:

```text
source_id
source_text
load_parsed
load_interface
UNIVERSE_NODES
```

Add small ergonomic enumeration APIs only if needed; do not create another source-catalog crate.

#### G4. Build source semantic declarations/signatures

Use the same Spec-04 source publication code used by normal workspace analysis.

Do not create a bootstrap-specific source type checker.

#### G5. Run canonical core/native conformance

Produce:

```rust
VerifiedCoreSurface
```

before installing any descriptor-backed native method.

Migration mode may initially require only:

```text
every authored source @native anchor resolves and verifies
```

Final strict mode requires:

```text
every Required descriptor has exactly one verified source anchor
```

#### G6. Export/register native semantic metadata

Use the verified canonical signature product and existing semantic metadata exporter/loader infrastructure.

Do not invent a second native runtime signature record format.

#### G7. Install descriptor primitives

Install `PRIMITIVES` deterministically.

For each installed method, populate both:

```text
MethodImplementationIndex   // already implemented
MethodSemanticIndex         // remaining work
```

#### G8. Compile/execute real source methods

Compile canonical universe source after native primitives exist.

The source compiler must:

```text
skip executable emission for @native declarations/reference bodies
compile ordinary .ph wrappers/algorithms normally
```

#### G9. Verify post-bootstrap invariants

Retain existing runtime invariants and add source/native completion invariants.

### Parse-once requirement

The previous plan correctly called out duplicate parsing. Preserve that goal, but align it with current `ParsedModuleUnit` architecture.

Bootstrap should retain the exact parsed unit used by source/native preflight and feed its AST into compilation through an AST-taking compiler entry point.

Do not verify one parse and execute a separately reparsed source string if avoidable.

### Completion gate

A deliberately mismatched source/native fixture fails before any native method installation or universe-source execution.

---

## 11. Workstream H — Native class completion and special binding preservation

### Goal

Formalize primordial source class completion without conflating class identity with global binding.

### Current runtime fact

Bootstrap already treats `None` specially: the class row exists, but the public global `None` is the immediate absence value rather than the class object. The runtime class is inserted into internal class lookup so a source reopen can target it.

### Required class-level `@native` semantics

For a trusted `@native class`:

1. resolve source class to `UniverseKey`;
2. resolve `UniverseKey` to existing primordial `ClassId`;
3. verify source superclass against `UNIVERSE_CLASS_RELATIONS`;
4. verify generic form/kind metadata against canonical universe type metadata;
5. verify representation-field compatibility where Rust owns fixed layout;
6. attach real source methods to the existing class;
7. do not allocate a new class;
8. do not blindly rebind the source name global.

### Global-binding rule

For native class completion:

```text
runtime class identity
    != necessarily
public global binding value
```

If the canonical preexisting global intentionally denotes another value (`None`), completion preserves that global.

This must be generic behavior for native completion, not a parser special case for the identifier `None`.

### Layout rule

Do not let source-authored fields silently redefine native representation slots.

Classify separately:

```text
source-owned fields           _name
implementation storage       __name
Rust-fixed representation     bootstrap/native layout metadata
```

Only add source declarations for `__` fields after the runtime/source layout contract for that field is explicitly defined.

---

## 12. Workstream I — Attach verified native callable semantics to runtime methods

### Goal

Make installed native `MethodObject`s participate in the same runtime typing reflection model as source methods, while preserving the already-landed implementation provenance side table.

### Current state

Landed:

```text
MethodImplementationIndex
    MethodObject -> ImplementationKind / PrimitiveKey / intrinsic / ABI / Rust source
```

Existing but not populated by native installer:

```text
MethodSemanticIndex
    MethodObject -> RuntimeCallableRef { pool, record }
```

### Required changes

#### I1. Fix metadata-pool identity allocation first

Current program materialization calls `load_metadata_bundle(MetadataPoolId(0), ...)` and then calls `RuntimeTypingRegistry::register_pool`, whose returned ID is based on `pools.len()`.

That assumption becomes invalid as soon as bootstrap loads a native metadata pool before program metadata.

Introduce one authoritative allocation/load path, for example:

```rust
impl RuntimeTypingRegistry {
    pub fn load_and_register_bundle(
        &mut self,
        bundle: Arc<SemanticMetadataBundle>,
        limits: &ValidationLimits,
    ) -> Result<MetadataPoolId, ...>;
}
```

The registry allocates the pool ID, passes that exact ID to the loader, then stores the pool.

Migrate ordinary program materialization to the same helper.

#### I2. Export verified native signatures through existing metadata exporter

Prefer:

```text
DeclarationTypeTable
CallableSignatureTable
TypeStore
    ↓ MetadataExporter
SemanticMetadataBundle
    ↓ runtime loader
Runtime metadata pool
```

rather than hand-building runtime callable records in `phalcom-core`.

#### I3. Return installed method refs from descriptor installation

Change the installer to expose installed method identity, e.g.:

```rust
pub struct InstalledPrimitive {
    pub key: PrimitiveKey,
    pub method: ObjRef,
}
```

or make `install_one` return `ObjRef`.

Then map verified `PrimitiveKey`/`NativeSurfaceId` to the corresponding metadata callable record and insert:

```rust
vm.typing_registry.method_semantics.insert(
    method,
    RuntimeCallableRef { pool, record },
);
```

#### I4. Remove both side-table entries when replacing methods

During compatibility dual-install, descriptor installation may replace a legacy method object.

The installer already removes stale implementation provenance. It should also remove any stale semantic callable mapping for the replaced method.

### Completion gate

For a representative native method, reflection reaches:

```text
live MethodObject
    ↓ MethodSemanticIndex
TrustedNative callable metadata
    ↓
exact canonical parameters / return / generics available at runtime
```

without adding full semantic metadata to `MethodObject`.

---

## 13. Workstream J — Converge runtime and tooling on the builtin universe source provider

### Goal

Remove duplicate source corpora and make physical/bundled source selection a presentation concern over one logical builtin universe identity.

### Current state

`phalcom-modules::BuiltinProjectSourceProvider` is already a real source authority with stable:

```text
ModuleId::builtin(Universe, path)
phalcom://universe/... SourceId
source_text
load_parsed
load_interface
UNIVERSE_NODES
```

But:

- `VM::run_universe_modules` still owns `static SOURCES` and direct `include_str!` calls;
- LSP `semantic/core_source.rs` still embeds `phalcom-core/core/core.ph` as one fallback document.

### Required changes

#### J1. Runtime source corpus

Replace `VM::run_universe_modules::SOURCES` with provider module IDs/parsed units.

Preserve runtime initialization order explicitly during migration. Do not casually replace the current bootstrap execution order with a new topological order until module initialization semantics prove parity.

A transitional provider API may expose the bootstrap sequence as logical module IDs if needed:

```rust
pub fn universe_bootstrap_sequence() -> &'static [ModulePath];
```

The sequence must reference provider modules, not source strings.

Longer term, ordinary linked builtin-project initialization should own this order.

#### J2. Corpus coverage test

Add a repository test that ensures the provider covers every intended `phalcom-core/core/universe/src/**/*.ph` module and that every declared provider node loads/parses.

Do not create another hand-maintained list outside `phalcom-modules`.

#### J3. Physical workspace overrides

For developer navigation, logical builtin identity should remain:

```text
builtin:universe:<module>
```

while source location may point to an editable physical checkout when available.

This lets LSP open real files without changing semantic declaration identity.

---

## 14. Workstream K — Migrate LSP from compatibility core surfaces to formal semantic core truth

### Goal

Stop making the LSP independently reconcile source/native core members and make it consume compiler-owned semantic products plus source presentation metadata.

### Current state

The LSP has advanced beyond the older plan:

- it uses `NATIVE_SURFACE_CATALOG` for rich generated rows;
- it uses `UNIVERSE_CLASS_RELATIONS` before compatibility class rows;
- it has explicit `MemberOrigin::{Source,Native,Generated}`.

But it still:

- models one `core.ph` rather than the universe project;
- skips native enrichment if a source member already exists;
- falls back to handwritten `NATIVE_MEMBERS`;
- stores only `native_return: Option<NativeReturnShape>` on `MemberSurface`;
- owns a separate selector projection;
- misclassifies implementation fields using `_$` instead of `__`.

### Required changes

#### K1. Fix implementation-field classification immediately

Change:

```rust
field.name.starts_with("_$")
```

to:

```rust
field.name.starts_with("__")
```

Add tests differentiating:

```text
_sourceField       source storage
__runtimeField     implementation storage
_$selector         implementation selector, not field namespace
```

This fix does not depend on the rest of the migration.

#### K2. Stop adding new semantics to `NativeReturnShape`

The LSP can temporarily keep `native_return` for old consumers, but new hover/completion/type behavior must use formal semantic signatures.

#### K3. Consume the actual universe project

Replace single-core source modeling with module-aware builtin universe source.

Use the same logical `ModuleId`s and semantic declaration identities as compiler analysis.

#### K4. Merge source anchor + native metadata, do not skip

For verified `@native` source:

```text
source location/docs/names
+ canonical verified type signature
+ implementation/native metadata
= one LSP member
```

Current “if source member exists, skip native row” behavior is a compatibility shortcut that must be removed.

#### K5. Prefer formal semantic snapshot products

The LSP already has a formal static semantic integration path. Migrate core hover/completion/definition consumers toward:

```text
SemanticSnapshot / callable signatures / core-surface presentation
```

rather than extending the advisory `phalcom-lsp/src/semantic/*` model with another formal native typing implementation.

#### K6. Preserve editor-only heuristics separately

Editor convenience heuristics may remain in the LSP, but they cannot define native contract truth.

### Completion gate

The LSP can navigate a native method to its real universe `.ph` declaration, display source Phaldoc and canonical type/effect/raise metadata, identify native implementation provenance, and do so without `NATIVE_MEMBERS` or `NativeReturnShape` as semantic authority.

---

## 15. Workstream L — Migrate primitive coverage and canonical source anchors

### Goal

Move every ordinary language primitive to one descriptor-backed installation path and one verified source anchor unless explicitly hidden.

### Current state

At the grounded revision, generated metadata records an authored `#[primitive]` declaration count of **93**. That number is a repository observation, not proof of total primitive coverage.

The runtime still defaults to `NativeInstallMode::Dual` and preserves the legacy installer when descriptor coverage does not equal the compatibility native-member floor.

`spec03_5_census.rs` currently reports:

```text
generated set
descriptor set
legacy set
generated-only
descriptor-only
legacy-only
```

but does not yet assert full equality.

### Revised migration rule

Migrate by coherent primitive owner/module and in one commit or review unit:

1. add/fix complete `#[primitive(...)]` declaration;
2. regenerate `NATIVE_SURFACES`;
3. add/update canonical source `@native` anchor;
4. add `@internal` for implementation selectors;
5. validate source/native canonical signature conformance;
6. remove the corresponding legacy `Universe::install_primitives` entry;
7. assert descriptor installation is the only live installer for that key;
8. run runtime + semantic + LSP parity tests for that owner.

Do not leave one primitive installed by both paths merely because descriptor installation overwrites the legacy one deterministically.

### Source classification

Every current core member must be explicitly classified:

```text
A. declaration-only @native primitive
B. reference-bodied @native primitive
C. real .ph wrapper over native floor
D. real .ph high-level algorithm
E. intentionally hidden native primitive (anchor = Hidden)
```

“Rust-only but language-visible by accident” is not an acceptable final category.

### Recommended migration waves

The older wave ordering remains broadly good, but use current descriptor coverage to choose exact atoms:

1. scalars and Option/Some/None-facing operations;
2. object/class/behavior/reflection kernel;
3. collections/storage primitives;
4. callable/family/method gateways;
5. errors/system/resources/fibers;
6. typing/reflection runtime primitives.

Do not migrate a module merely to satisfy the order if its source semantic signature cannot yet be published correctly; dependency correctness beats aesthetic sequencing.

---

# Part IV — Compatibility and Deletion Ledger

## 16. Explicit deletion ledger

Nothing below is deleted merely because the final architecture dislikes it. Each removal has a measurable gate.

| Compatibility item | Current role | Delete only when |
|---|---|---|
| `phalcom-native-surface::NATIVE_MEMBERS` | legacy coverage for LSP/runtime census | generated `NATIVE_SURFACES` covers every remaining consumer key; LSP and registry no longer import it; census equality is strict |
| `phalcom-native-surface::NATIVE_CLASSES` | transitional LSP class fallback | all required primordial/source classes resolve through `UNIVERSE_CLASS_RELATIONS` + real universe source declarations; no consumer references it |
| `NativeReturnShape` | lossy LSP compatibility return information | all semantic consumers use canonical callable signatures/flow; no public compatibility promise requires it |
| `Universe::install_primitives` ordinary entries | legacy runtime installer | descriptor set covers each key, descriptor-only runtime parity tests pass, no legacy-only key remains |
| `NativeInstallMode::Dual` default | migration safety | descriptor floor complete + descriptor-only bootstrap/runtime suite passes |
| `descriptor_floor_is_complete()` compatibility comparison | prevents premature descriptor-only startup | strict descriptor/source/generated parity makes the fallback unnecessary |
| proc-macro-local `PrimitiveAttrArgs` semantic parser | duplicate grammar | proc macro can emit complete descriptor from `phalcom-native-decl` normalized declaration plus ItemFn ABI validation |
| manually authored rich rows in `generated.rs` | checked-in partial projection | generator emits full file deterministically and `--check` verifies full content |
| `VM::run_universe_modules::SOURCES` | bootstrap source list/text authority | runtime consumes provider-backed logical module sequence/parsed units |
| LSP `BUNDLED_CORE_SOURCE` as native-core authority | single-core compatibility surface | universe project source provider + formal semantic snapshot cover core navigation/semantics |
| LSP fallback merge from `NATIVE_MEMBERS` | compatibility coverage | generated rich surface complete and semantic merge authoritative |
| LSP `native_return` | lossy native type hint | consumers use formal callable signatures |
| `ModuleId::core()` as canonical semantic owner of universe classes | compatibility declaration identity | `UniverseDeclarationIndex` and real builtin universe DeclarationIds are used by semantic/native publication |
| `checker/declaration.rs` duplicate source signature construction | legacy checker path | source callable publication is centralized and legacy dispatch adapter derives from it |

### 16.1 What is not deleted by this plan

Do not delete:

- `PrimitiveDescriptor`;
- `PRIMITIVES` distributed registry;
- `PrimitiveSurfaceSpec`;
- `NativeSurfaceRecord` / generated VM-free catalog;
- `UNIVERSE_CLASS_RELATIONS`;
- `UNIVERSE_BINDINGS`;
- `RuntimeTypingRegistry`;
- `MethodSemanticIndex`;
- `MethodImplementationIndex`;
- canonical source `.ph` declarations;
- real source wrappers;
- runtime fixed-layout/bootstrap code where representation genuinely requires it.

---

# Part V — Revised Implementation Sequence

## 17. Dependency graph

The revised implementation order should be driven by actual dependencies rather than the older twelve-phase sequence.

```text
A. finish shared primitive declaration parser
           │
           ├────────► B. full deterministic generated surface
           │                    │
           │                    └────────────┐
           │                                 │
C. MemberBody + @internal + @native class    │
           │                                 │
           ▼                                 │
D. Spec-04 source signature publication      │
           │                                 │
           ├────────► F. universe declaration identity
           │                    │            │
           └────────► E. semantic source/native conformance ◄────┘
                                │
                                ▼
                      G. bootstrap preflight
                         │             │
                         ▼             ▼
                  I. runtime typing   J. provider convergence
                         │             │
                         └──────┬──────┘
                                ▼
                         K. LSP migration
                                │
                                ▼
                         L. full primitive/source migration
                                │
                                ▼
                         strict bijection + deletions
```

### 17.1 Parallel-safe work

Can proceed without waiting for full Spec 04/04.5:

```text
A1/A2 shared parser convergence + visibility hardening
A3 anchor policy
B generator completion
K1 LSP __ field bug fix
census/test strengthening
runtime metadata pool-ID helper
provider corpus coverage tests
```

### 17.2 Work that requires Spec-04 source publication

Do not declare final completion of:

```text
strict typed source/native conformance for generic methods
@native class generic conformance
constraint conformance
capture-sensitive type-lambda anchor verification
open-row anchor verification
```

until their canonical source semantics are published.

### 17.3 Work that does not need to wait for full 04.5

Do not block this project on:

```text
flow narrowing
loop fixed points
ordinary generic-call expected-result inference
explanation rendering
causal suppression
LSP flow migration
```

Those are important, but orthogonal.

---

## 18. Recommended implementation phases

### Phase 0 — Freeze the current census and add stronger observability

Tasks:

- enhance `spec03_5_census` output to include exact missing sets in stable order;
- add generator declaration-to-artifact full comparison scaffolding;
- add source-anchor census tooling even before strict source syntax exists;
- record descriptor/legacy/generated counts in test diagnostics, not hard-coded acceptance criteria.

Gate:

```text
every installed legacy/native key is visible to the census
```

No deletion yet.

### Phase 1 — Finish `phalcom-native-decl` and rich generation

Tasks:

- remove proc-macro semantic parser duplication;
- enforce strict internal visibility;
- add anchor policy;
- make generator emit full `generated.rs`;
- make `--check` compare full generated output.

Gate:

```text
proc-macro declaration interpretation == generator declaration interpretation
```

by construction.

### Phase 2 — Land source declaration mechanics

Tasks:

- `MemberBody`;
- declaration-only parser behavior;
- `BuiltinAttr::Internal`;
- attribute argument policy;
- privilege/namespace integrity;
- class-level `@native` legality;
- preserve reference body AST.

Gate:

```text
native declaration can exist in source without emitting executable code
```

while invalid user `@native` cannot disappear silently.

### Phase 3 — Complete Spec-04 declaration/signature publication

Tasks owned primarily by Spec 04:

- publish generic class declaration forms;
- publish generic superclass templates;
- publish source callable semantic signatures;
- include method generics and `where` constraints;
- make unsupported forms structured rather than silently accepted.

Gate:

```text
source CallableSignatureTable is canonical enough for conformance
```

### Phase 4 — Upgrade semantic `core_surface`

Tasks:

- shared AST selector projection;
- source class/member anchor extraction with real source spans/docs/body kind;
- source/native canonical signature merge;
- class generic/superclass conformance;
- structured mismatch/blocked diagnostics;
- `VerifiedCoreSurface` product.

Gate:

```text
one source @native anchor + one native row -> one verified semantic callable
```

### Phase 5 — Converge universe declaration identity

Tasks:

- build `UniverseDeclarationIndex`;
- migrate semantic bootstrap owner resolution from `ModuleId::core()` when real universe source is present;
- verify stable metadata identity/source navigation.

Gate:

```text
native and source CallableId.owner are the same semantic DeclarationId
```

### Phase 6 — Add bootstrap preflight and provider-backed parse-once corpus

Tasks:

- fallible bootstrap seam;
- provider source loading;
- preflight before installation;
- retain parsed AST;
- compile from retained AST;
- initially run “anchors must resolve” mode.

Gate:

intentional mismatch fails before native installation/source execution.

### Phase 7 — Attach native semantic metadata at runtime

Tasks:

- fix pool ID allocation;
- export/load verified native metadata;
- make native installer return method IDs;
- populate `MethodSemanticIndex`;
- retain implementation provenance insert.

Gate:

runtime reflection retrieves exact trusted native signature for installed primitive.

### Phase 8 — LSP universe-project/formal-semantic migration

Tasks:

- fix `__` field classification;
- replace core single-file authority with logical builtin universe modules;
- consume verified semantic signatures/core surface;
- merge source + native instead of skipping;
- retire `native_return` from formal paths;
- preserve physical workspace source override for navigation.

Gate:

hover/completion/definition agree with compiler semantic truth for core callables.

### Phase 9 — Complete descriptor/source-anchor migration

Tasks:

- module-by-module primitive migration;
- source declaration/Phaldoc cleanup;
- remove corresponding legacy installer rows immediately;
- verify descriptor-only behavior for migrated owners.

Gate:

no ordinary language primitive key remains legacy-only.

### Phase 10 — Strict bijection and compatibility deletion

Enable:

```text
required descriptors == verified source @native anchors
```

Then delete compatibility authorities according to the ledger.

### Phase 11 — Documentation/source cleanup

After semantics are stable:

- replace task-history comments with durable Phaldoc/implementation explanation;
- retain conceptual bodies only when truthful;
- do not duplicate machine-readable types/effects in prose unnecessarily.

---

# Part VI — Exact Algorithms and Invariants

## 19. Canonical callable conformance algorithm

Given a source `@native` member `S` and native surface row `N`:

```text
1. Resolve source owner to canonical Universe DeclarationId.
2. Project source member to canonical Selector.
3. Construct CallableId(owner, selector, side).
4. Fetch source CallableSemanticSignature from source publication.
5. Import native PrimitiveSurfaceSpec into a CallableSemanticSignature
   using the same TypeStore and owner/callable parameter identity rules.
6. Assert CallableId equality.
7. Compare parameter lane shape.
8. Compare generic signature.
9. Compare canonical parameter TypeTerms.
10. Compare canonical return TypeTerm.
11. Compare source @internal assertion against native visibility.
12. Record implementation provenance and source presentation metadata.
13. Publish one VerifiedNativeCallable.
```

### 19.1 Equality versus assignability

Conformance is declaration agreement, not call-site assignability.

Therefore source/native contract comparison normally requires semantic **equivalence** of declared types, not merely:

```text
source type <: native type
```

A looser relation would permit the two authorities to describe different APIs.

### 19.2 Generic alpha-equivalence

Generic parameter names are descriptive. Compare by canonical owner/index identity and structure.

For:

```text
source: <T> foo(T) -> T
native: <U> foo(U) -> U
```

these are equivalent if both lower to the same callable-owner/index parameter structure.

### 19.3 Local source parameter names

These are not part of native contract equality:

```text
source local name = value
native generator synthetic name = arg0
```

Source local names should enrich docs/reflection after successful structural verification.

### 19.4 `Self`

Native `SelfType` must lower with the same owner/dispatch-role semantics as source `Self`.

Do not leave native `SelfType` permanently as `OpaqueNative` once owner context is known.

### 19.5 Constraints

If source has a `where` clause and native metadata cannot express it:

```text
result = Blocked(NativeMetadataCannotRepresentConstraint)
```

not Verified and not Unknown.

Either extend native metadata or prohibit that native declaration form until representation exists.

---

## 20. Native class conformance algorithm

For source `@native class C`:

```text
1. UniverseKey::from_name(C.name) must resolve.
2. Runtime universe must already own a primordial ClassId for that key.
3. Source semantic DeclarationId must enter UniverseDeclarationIndex.
4. Source superclass must match UNIVERSE_CLASS_RELATIONS.
5. Source generic declaration must match canonical Universe type-form metadata:
       arity
       parameter kinds
       variance when represented
       declaration constraints when represented
6. Source class must be marked native in trusted universe source.
7. Compiler class lowering must use completion path, not allocation path.
8. Existing public global binding must be preserved when not equal to the class object.
9. Fixed runtime layout compatibility must hold.
```

This produces a verified mapping:

```text
UniverseKey
    ↔ semantic DeclarationId
    ↔ runtime ClassId
    ↔ source declaration span
```

---

## 21. Bootstrap state machine

Recommended high-level bootstrap state:

```text
Uninitialized
    ↓
KernelConstructed
    ↓
DescriptorCatalogValidated
    ↓
UniverseSourceLoaded
    ↓
UniverseSemanticsPublished
    ↓
NativeSourceConformanceVerified
    ↓
NativeMetadataRegistered
    ↓
PrimitivesInstalled
    ↓
UniverseSourceExecuted
    ↓
RuntimeInvariantsVerified
    ↓
Ready
```

Forbidden transitions:

```text
UniverseSourceExecuted before NativeSourceConformanceVerified
PrimitivesInstalled before required preflight succeeds
Ready with blocked/mismatched required native anchors
```

During migration only, legacy installation may coexist between catalog validation and source execution; its use must be explicit in bootstrap state/reporting.

---

# Part VII — Test Strategy

## 22. Tests by ownership layer

### 22.1 `phalcom-native-decl`

Unit tests:

- all attribute fields parse once through shared grammar;
- strict internal visibility;
- lifecycle consistency;
- canonical selector encoding;
- anchor policy;
- generic callable kinds;
- invalid/duplicate metadata.

### 22.2 `phalcom-native-surface-gen`

Golden/determinism tests:

- scan every primitive source recursively;
- stable sort;
- full generated output equality;
- docs extraction;
- generated keys unique;
- generated rich metadata equals normalized declaration.

### 22.3 `phalcom-ast`

Parser/AST tests:

```text
native declaration method       MemberBody::Declaration
native declaration getter       MemberBody::Declaration
empty executable body           Statements([])
reference body                  Statements([...])
@internal builtin recognition
class-level @native preserved
annotations/generics/where preserved on declaration-only member
```

### 22.4 Compiler attribute/integrity tests

```text
user @native                                fail
trusted native source @native               pass
_$ + @internal                              pass
_$ without @internal in authored universe   fail
@internal ordinary selector                 fail
__field + @internal                         pass
@internal _field                            fail
@internal + @private/@protected             fail
@native(args)                               fail
@internal(args)                             fail
native declaration reaches executable lowering  impossible/asserted
reference-bodied native emits no live source method
```

### 22.5 Spec-04 source signature publication tests

```text
generic class kind publication
generic method signature publication
where constraint publication
Self owner/side publication
generic superclass template
invalid type syntax not accepted as verified Unknown
source CallableSignatureTable parity with declaration AST
```

### 22.6 Semantic core conformance tests

Fixtures for:

```text
exact source/native match                 Verified
missing descriptor                        orphan anchor
missing required source                   missing anchor
hidden descriptor no source               allowed
duplicate source anchor                   duplicate error
selector mismatch                         mismatch
side mismatch                             mismatch
rest shape mismatch                       mismatch
parameter label mismatch                  mismatch
parameter type mismatch                   mismatch
return type mismatch                      mismatch
generic kind mismatch                     mismatch
constraint mismatch                       mismatch/blocked until representable
Self contract equivalence                 Verified
unsupported open row                      Blocked, not Verified
unsupported type lambda                   Blocked, not Verified
```

### 22.7 Declaration identity tests

```text
UniverseKey::String maps to real builtin universe DeclarationId
native/source String CallableId owner equal
same source name in user project does not collide
ModuleId::core compatibility path does not become durable universe identity after migration
```

### 22.8 Bootstrap tests

```text
mismatch fails before primitive installation
mismatch fails before universe body executes
descriptor install succeeds after verified preflight
@native declaration does not overwrite primitive
reference body does not overwrite primitive
source wrapper executes normally
@native class reuses exact primordial ClassId
special None global remains immediate None
fixed native layout mismatch rejected
post-bootstrap UNIVERSE_CLASS_RELATIONS invariant retained
```

### 22.9 Runtime typing tests

```text
native MethodObject -> MethodSemanticIndex
native MethodObject -> MethodImplementationIndex
callable authority TrustedNative
parameter/return type exact
native + program metadata pools coexist without ID collision
replaced compatibility method leaves no stale side-table entry
```

### 22.10 LSP/compiler convergence tests

For the same universe source and generated catalog:

```text
compiler callable identity == LSP callable identity
hover type == semantic callable signature
go-to-definition source == real universe source anchor
completion hides Internal by default
source @native + native row produces one member, not two
source wrapper remains Source implementation
```

### 22.11 Differential migration tests

For migrated owners, run VM boot twice where supported:

```text
Dual
DescriptorOnly
```

Compare:

- installed selector sets;
- visibility;
- method kind/provenance;
- core behavior tests;
- reflection-visible semantics.

Only then remove legacy rows for that owner.

---

## 23. Strict final invariant tests

At final migration state, CI must assert:

```text
1. generator --check is clean
2. generated NATIVE_SURFACES keys == descriptor PRIMITIVES keys
3. every Required descriptor has one verified source anchor
4. every source @native anchor has one descriptor
5. every authored _$ universe member carries @internal
6. every authored __ universe field carries @internal
7. no _$ descriptor is Public
8. no ordinary language primitive remains legacy-only
9. DescriptorOnly VM bootstrap passes full runtime suite
10. LSP no longer imports NATIVE_MEMBERS for formal semantics
11. runtime no longer needs duplicate source-text SOURCES list
12. canonical universe semantic owners are real builtin-universe DeclarationIds
```

---

# Part VIII — Verification Commands

## 24. Repository verification matrix

The implementation should be verified incrementally. These commands are execution gates, not claims that this document has run them successfully.

### Shared native declaration and generation

```bash
cargo test -p phalcom-native-decl
cargo test -p phalcom-native-macros
cargo run -p phalcom-native-surface-gen -- --root . --check
cargo test -p phalcom-native-surface
```

If proc-macro compile-fail tests use a dedicated harness, run that target explicitly as well.

### AST / source declaration mechanics

```bash
cargo test -p phalcom-ast
cargo test -p phalcom-core --lib compiler
```

Use the repository's actual focused test names once the new declaration-only and attribute tests are added; do not rely on a broad command as proof that the negative cases were exercised.

### Semantic publication and conformance

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-type-meta
```

Required focused test groups should include source signature publication, native import, canonical core-surface merge/conformance, declaration identity, metadata export, and blocked-vs-mismatch behavior.

### Runtime/bootstrap

```bash
cargo test -p phalcom-core --test spec03_5_census
cargo test -p phalcom-core --test spec03_5_conformance
cargo test -p phalcom-core --test spec03_reflection
cargo test -p phalcom-core
```

After descriptor-only parity infrastructure exists, add a dedicated test target that constructs the VM in `DescriptorOnly` mode and exercises the complete kernel/native conformance suite. Do not infer descriptor-only parity from normal `VM::new()`, because normal startup currently uses `Dual`.

### LSP

```bash
cargo test -p phalcom-lsp
```

Focused tests must exercise real builtin-universe module identity, source/native merge, definition location, hover signature, internal completion filtering, and the `__` implementation-field rule.

### Final workspace gate

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If repository CI uses a different lint policy, follow the repository's CI configuration instead of introducing unrelated lint churn.

### Evidence rule

A phase is not complete merely because the code paths exist. Record the command, exit status, and relevant focused test counts/results in the implementation PR or completion note.

---

# Part IX — Performance and Incremental Requirements

## 25. Do not optimize before measuring

No performance claim is made by this plan. Add measurements before considering shortcuts such as precomputed trusted hashes that skip full development preflight.

### 24.1 Bootstrap metrics

Measure separately:

```text
native descriptor scan/validation
builtin universe source loading
parse time
source signature publication
source/native conformance
metadata export/load
native descriptor installation
universe source compile
universe source execution
```

Record cold and repeated-process behavior where meaningful.

### 24.2 Avoid duplicate canonical work

Required efficiencies:

- parse each builtin universe module once per bootstrap analysis;
- intern source/native types in one `TypeStore`;
- index signatures by `CallableId`;
- index native surfaces by `PrimitiveKey`/`NativeSurfaceId`;
- compare contracts without repeated string parsing;
- reuse verified results for metadata export and installation;
- use catalog fingerprints as invalidation/debug evidence, not as a substitute for correctness before measurement justifies it.

### 24.3 LSP incremental behavior

The LSP should treat generated native catalog data as immutable for a toolchain build and source universe modules as normal semantic inputs.

A user-file body edit must not rebuild the native surface catalog.

A universe-source edit in a development checkout should invalidate only relevant builtin source/interface/semantic products, subject to current workspace architecture.

Do not add native-specific local flow/inference caches to the LSP. Formal typing stays compiler-owned.

---

# Part X — File Ownership Map

## 26. Files to modify, not recreate

### `phalcom-native-decl/`

Modify existing parser/normalized/validator so it becomes the complete primitive metadata authority.

### `phalcom-native-macros/src/lib.rs`

Delete the duplicate primitive metadata parser/semantic validation once shared declaration output is sufficient; keep proc-macro ItemFn ABI checking/emission.

### `phalcom-native-surface-gen/src/main.rs`

Upgrade from census/drift-count checker to full deterministic generator.

### `phalcom-native-surface/src/generated.rs`

Become fully generated output only.

### `phalcom-native-surface/src/lib.rs`

Retain catalog/index API. Stage-delete `NATIVE_MEMBERS`, `NATIVE_CLASSES`, `NativeReturnShape`, and duplicate compatibility enums when their consumers are gone.

### `phalcom-native-meta/src/primitive.rs`

Add anchor policy and any metadata needed for strict source/native declaration agreement.

### `phalcom-native-meta/src/types.rs`

Extend only for semantic forms actually required by native contracts; account for generic kinds/constraints/variance gaps exposed by Spec 04.

### `phalcom-native-meta/src/universe.rs`

Keep `UNIVERSE_CLASS_RELATIONS` as primordial hierarchy authority. Extend type-form metadata only if source generic class conformance requires missing facts.

### `phalcom-ast/src/ast.rs`

Add `MemberBody`, `BuiltinAttr::Internal`, helpers. Do not redesign Spec-04 type syntax here.

### `phalcom-ast/src/parser.rs`

Parse declaration-only member bodies without coupling grammar recognition exclusively to `@native`.

### optional `phalcom-ast/src/selector.rs`

Add shared AST member selector projection.

### `phalcom-semantic/src/types/annotation.rs`

Finish Spec-04 source lowering gaps under their owning spec; do not add native-only normalization.

### `phalcom-semantic/src/signature.rs` or a new focused signature-publication submodule

Add/centralize source `CallableSemanticSignature` publication.

### `phalcom-semantic/src/workspace.rs`

Replace monomorphic source declaration predeclaration with full declaration publication phases and populate source callable signatures.

### `phalcom-semantic/src/checker/declaration.rs`

Migrate to centralized source signature publication; adapt callable body handling to `MemberBody`; retain body checking only for executable/reference analysis as intended.

### `phalcom-semantic/src/types/native.rs`

Extend native import for generic params/`Self`/supported canonical forms; keep it as native → canonical semantic bridge.

### `phalcom-semantic/src/core_surface/source.rs`

Upgrade source extraction to real source identity/docs/body markers and canonical callable signatures.

### `phalcom-semantic/src/core_surface/merge.rs`

Merge by canonical owner/side/Selector/CallableId and verified semantic contract.

### `phalcom-semantic/src/core_surface/conformance.rs`

Replace “native types resolve” only validation with source/native canonical semantic conformance and structured outcomes.

### `phalcom-semantic/src/core_surface/presentation.rs`

Consume verified merged semantics; preserve source docs/conceptual body/native provenance without becoming semantic authority.

### `phalcom-semantic/src/declarations.rs`

Use real universe declaration resolver/index when available; preserve test compatibility resolver only where necessary.

### `phalcom-modules/src/builtin.rs`

Remain builtin source corpus authority; add only the enumeration/bootstrap-sequence API actually required by runtime/LSP.

### `phalcom-core/src/compiler/attributes.rs`

Add internal/native argument/target policy and ensure subtractive native processing occurs after integrity validation.

### `phalcom-core/src/compiler/lib/class_decl.rs`

Implement `@native class` completion, source-integrity checks, special global preservation, and MemberBody executable lowering guards.

### `phalcom-core/src/vm/bootstrap.rs`

Introduce fallible phases, provider-backed source input, semantic preflight before installation, parse-once execution, and eventual DescriptorOnly default.

### `phalcom-core/src/native/install.rs`

Return installed method IDs; populate semantic index in addition to already-landed implementation provenance; clean stale side-table entries when replacing compatibility methods.

### `phalcom-core/src/native/registry.rs`

Keep uniqueness/census during migration; remove compatibility-floor comparison once strict parity replaces it.

### `phalcom-core/src/typing/registry.rs`

Add pool-ID-safe load/register API.

### `phalcom-core/src/modules/materialize.rs`

Stop hardcoding `MetadataPoolId(0)` and use registry-owned pool allocation.

### `phalcom-core/src/universe/primitives.rs`

Shrink atomically with descriptor migration; delete ordinary primitive authority at final gate.

### `phalcom-lsp/src/semantic/core_source.rs`

Replace single-core compatibility authority with real universe source modules/formal semantic data.

### `phalcom-lsp/src/semantic/surface.rs`

Fix `__` field classification; stop using `native_return` as formal semantic truth; project verified implementation origin/type metadata.

### `phalcom-lsp/src/selectors.rs`

Delegate to shared AST selector projection and eventually delete duplicated implementation.

---

# Part XI — Open Implementation Decisions

## 27. Decision register

### D-NATIVE-1 — Generator architecture

**Status:** Ratified by current repository direction.

**Decision:** Keep standalone `phalcom-native-surface-gen` + checked-in generated artifact rather than adding a sibling-source-scanning `build.rs`.

**Reason:** explicit reproducibility/drift checks; no upward build dependency from surface crate to core source tree.

### D-NATIVE-2 — Contract comparison representation

**Status:** Ratified by Specs 04/04.5 architecture.

**Decision:** Compare canonical `CallableSemanticSignature`/`TypeTerm` data.

**Rejected:** separate `NormalizedContractType` semantic tree.

### D-NATIVE-3 — Source verifier ownership

**Status:** Ratified by current `phalcom-semantic::core_surface` implementation.

**Decision:** semantic conformance belongs in `phalcom-semantic`; runtime bootstrap consumes it.

**Rejected:** `phalcom-core` becoming a second semantic verifier.

### D-NATIVE-4 — Anchor exception representation

**Status:** Proposed, still recommended.

**Decision:** explicit per-descriptor `Required | Hidden` policy.

**Rejected:** external exemption list or implicit “internal means hidden”. Internal methods may still be important canonical source declarations.

### D-NATIVE-5 — Native metadata support for Spec-04 advanced forms

**Status:** Open implementation choice, gate-driven.

Options:

A. immediately mirror all Spec-04 type forms/constraints in `phalcom-native-meta`;
B. extend only when a real native signature requires a form, and block strict verification otherwise.

**Recommendation:** B. Keep the Rust metadata language as small as the actual native ABI contract requires, while never coercing unsupported written source semantics to `Unknown` and calling it verified.

### D-NATIVE-6 — Universe semantic owner identity

**Status:** Proposed requirement, high priority.

**Decision:** converge canonical universe classes onto actual builtin universe source `DeclarationId`s; retain `ModuleId::core()` only as transition/test compatibility.

**Reason:** strict `CallableId`, metadata, and go-to-definition coherence cannot be permanently built on two owners for one class.

### D-NATIVE-7 — Bootstrap full conformance in release builds

**Status:** Deferred performance choice.

Start with full structural verification in development/test. Measure startup cost before considering prevalidated artifact fingerprints or reduced release checks.

Correctness comes first.

### D-NATIVE-8 — Reference-body semantic checking

**Status:** Deferred/orthogonal.

A reference body may be parsed/indexed before it is required to type-check as an executable implementation. If later checked, its result is explanatory/advisory and must not prove equivalence to Rust behavior.

### D-NATIVE-9 — Generic native `where` constraints

**Status:** Open until a real native contract requires them.

Current source semantics can represent them; current native metadata cannot fully represent them.

**Recommendation:** either extend native metadata before allowing constrained `@native` anchors or explicitly reject/block such anchors. Never ignore the source `where` clause.

---

# Part XII — Risks

## 28. Risk register

### R1. Accidentally creating a third type system

**Risk:** native-source verifier introduces its own normalized type tree/string comparison.

**Mitigation:** require `TypeStore` + `CallableSemanticSignature` as conformance inputs.

### R2. Treating partially implemented Spec-04 forms as verified

**Risk:** row tail/type lambda/generic constraints lower incompletely and native conformance silently accepts `Unknown`.

**Mitigation:** structured blocked outcomes; strict bootstrap rejects blocked required anchors.

### R3. Two semantic owners for one universe class

**Risk:** native types use `ModuleId::core`, source uses builtin universe module IDs.

**Mitigation:** explicit `UniverseDeclarationIndex` convergence gate.

### R4. Generator and proc macro continue to drift

**Risk:** shared parser is called only as a shallow validation pass while macro uses its own interpretation.

**Mitigation:** proc macro must consume the normalized declaration directly.

### R5. Descriptor/source migration leaves double installation

**Risk:** deterministic overwrite hides the fact both systems still own a method.

**Mitigation:** per-owner migration deletes legacy row in same change; descriptor-only differential tests.

### R6. Native metadata pool breaks program metadata

**Risk:** current hard-coded pool zero collides once bootstrap preloads native metadata.

**Mitigation:** registry-owned pool allocation before native pool is introduced.

### R7. LSP becomes a second formal semantics engine

**Risk:** richer `MemberSurface` logic duplicates compiler source/native conformance.

**Mitigation:** LSP consumes compiler semantic snapshot/core-surface products; keep editor heuristics advisory.

### R8. `@internal` becomes mistaken for authority

**Risk:** user spelling grants `_$`/`__` access.

**Mitigation:** trusted module/project identity remains the privilege check.

### R9. `@native class` clobbers special globals

**Risk:** `None` class completion emits ordinary class global binding.

**Mitigation:** native completion separates class identity from public global value.

### R10. Source provider convergence changes bootstrap order accidentally

**Risk:** replacing manual `SOURCES` with ordinary module ordering changes behavior while native migration is in flight.

**Mitigation:** first preserve current logical execution sequence using provider-backed module IDs; topological runtime convergence is a separate measured change.

---

# Part XIII — Final Acceptance Criteria

## 29. Architecture completion criteria for this plan

This native-universe/bootstrap project is complete when all of the following are true.

### Rust/native declaration authority

- `phalcom-native-decl` is the one primitive metadata parser/validator.
- proc macro no longer implements a second metadata grammar.
- `_$` native selectors cannot be public.
- anchor policy is explicit.
- `phalcom-native-surface-gen --check` verifies the complete generated artifact.
- `NATIVE_SURFACES` is fully generated from authored primitive declarations.

### Source declaration model

- declaration-only callable bodies are first-class AST state.
- `@internal` is implemented according to its specification.
- `@native` is restricted to trusted native-source authority.
- class-level `@native` completes primordial identities.
- reference-bodied native members remain non-executable.
- real source wrappers remain executable source.

### Spec-04 semantic publication

- canonical source declaration generic forms are published.
- source callable signatures are in `CallableSignatureTable`.
- method generics and `where` constraints are preserved.
- unsupported/incomplete forms cannot masquerade as verified contracts.

### Semantic conformance

- source and native use one canonical declaration/callable identity.
- source/native type agreement is evaluated using canonical semantic types/signatures.
- no separate native-contract semantic type IR exists.
- class hierarchy/generic declaration agreement is verified.
- every required native source anchor is either Verified or bootstrap-failing; no silent Blocked success.

### Bootstrap

- source/native preflight occurs before native installation and universe execution.
- source corpus is provider-backed rather than VM-owned text duplication.
- preflight and execution reuse the same parsed source where practical.
- native methods install exactly once in final mode.
- `DescriptorOnly` becomes the normal path after parity.
- special globals such as `None` retain correct runtime values.
- post-bootstrap runtime relation/layout invariants still hold.

### Runtime typing/reflection

- native methods populate `MethodSemanticIndex` and `MethodImplementationIndex`.
- native callable metadata uses trusted canonical semantic records.
- metadata pool IDs are allocation-safe with multiple pools.
- implementation provenance remains separate from callable type truth.

### LSP/tooling

- LSP remains VM-free.
- formal core semantics come from compiler-owned semantic products.
- native source + native metadata becomes one callable.
- real universe source is the primary definition/navigation target.
- internal members are hidden from ordinary completion unless appropriate.
- implementation fields use `__`, not `_$`, classification.

### Deletion

- handwritten `NATIVE_MEMBERS` is gone from formal/runtime/tooling authority.
- compatibility `NATIVE_CLASSES` is gone where canonical relations/source cover it.
- `NativeReturnShape` no longer drives type semantics.
- ordinary language primitives are gone from `Universe::install_primitives`.
- Dual install is no longer the default/required path.
- VM bootstrap no longer owns an independent universe `SOURCES` text list.
- LSP no longer models one synthetic `core.ph` as the canonical native universe.
- `ModuleId::core()` is no longer the durable semantic identity of canonical universe classes.

---

## 30. Final conceptual lifecycle

The revised target lifecycle is:

```text
Rust implementation function
        │
        ▼
#[primitive(...)]
        │
        ├──────────────► PrimitiveDescriptor / PRIMITIVES
        │
        └──────────────► generated NativeSurfaceRecord
                                 │
canonical universe .ph           │
@native declaration              │
        │                        │
        ▼                        ▼
Spec-04 source signature    native semantic signature
        │                        │
        └──────── canonical conformance ────────┐
                                                ▼
                                      VerifiedCoreSurface
                                       │       │       │
                                       │       │       └──► LSP/presentation
                                       │       └──────────► metadata export
                                       └──────────────────► bootstrap install
                                                                │
                                                                ▼
                                                         live MethodObject
                                                          │           │
                                                          ▼           ▼
                                                MethodSemanticIndex  MethodImplementationIndex
```

The critical improvement over the older plan is not merely that more pieces are implemented. It is that the implementation now has a canonical semantic layer mature enough to be the **single comparison language** for source and native contracts.

That should be exploited aggressively.

The remaining work is therefore no longer “build a native semantic model beside the type system.” It is:

> **finish publishing source semantics, feed native declarations into the same semantics, verify the two, and then delete every compatibility registry or tooling path that still pretends there are multiple semantic authorities.**

