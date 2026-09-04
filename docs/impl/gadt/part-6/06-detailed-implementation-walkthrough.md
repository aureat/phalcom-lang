# Phalcom ADT/GADT + Associated Lookup
## Part 06 — Remaining Core Integration, Native ADT Migration, Reflection & Tooling Completion
### Current-`main`, Execution-Grade Implementation Plan

> **For agentic workers:** use TDD for every behavioral change. Do not recreate already-landed Part-06 scaffolding. Preserve the corrected Part-05.2 match architecture and the subsequent associated-lookup remediation. `phalcom-semantic` remains the sole source-level semantic authority.

**Repository:** `aureat/phalcom-lang`  
**Execution baseline:** current `main`, inspected at `9a8fcdc6537f41f651cbc4667acd1f38d06f4738`  
**Original Part-06 planning baseline:** obsolete `feat/adts` snapshot `26166385f9c1bf35f6e9eb969385fc8a162f2f56`  
**Important landed Part-06 commit:** `347ffedf94c570c18c5589ac1dbf98549f9224cb`  
**Purpose of this plan:** finish the parts that the broad Part-06 landing scaffolded but did not actually converge.

---

# 0. What Is Already Implemented — Do Not Reimplement It

The following architecture exists on current `main` and must be treated as starting infrastructure.

## 0.1 Native enum source extraction exists

Keep:

```text
phalcom-semantic/src/core_surface/source.rs
```

Existing useful products:

```rust
SourceEnumVariantRecord
SourceEnumRecord
SourceDeclarationRecord
extract_source_declarations(...)
```

`SourceEnumRecord` already records declaration-level `@native`, ordinary members, and enum variants.

Do not create a second `NativeEnumSourceRecord`.

The actual problem is that this richer extraction is not yet authoritative in the core/bootstrap pipeline.

## 0.2 Canonical core declaration identities exist

Keep:

```text
phalcom-semantic/src/core_surface/identity.rs
```

with:

```rust
CoreDeclarationIds {
    option,
    result,
    ordering,
    ...
}
```

and:

```rust
is_option(...)
is_result(...)
is_ordering(...)
is_core_adt(...)
```

Do not construct these identities again ad hoc in the VM.

## 0.3 ExactCase already exists and is canonical

Keep:

```text
phalcom-semantic/src/types/store.rs
```

and the existing:

```rust
TypeData::ExactCase {
    variant,
    enum_type,
}
```

plus:

```rust
TypeStore::exact_case_type(...)
```

Part 06 must finish integration/reflection; it does not need another exact-case interner.

## 0.4 Runtime ADT identities/registry exist

Keep:

```text
phalcom-core/src/adt.rs
phalcom-core/src/vm/adt.rs
```

and:

```rust
RuntimeEnumId
RuntimeVariantId
CaseDiscriminant
RuntimeAdtRegistry
RuntimeEnumDescriptor
RuntimeVariantDescriptor
```

The registry is the correct boundary. The native representation binding inside it needs correcting.

## 0.5 Part-05 executable matching is landed

Preserve:

```text
MatchLoweringSpec
ExecutablePattern
ExecutableVariantCandidate
ExecutableFieldProjection
IsVariant
GetVariantPayload
MatchInvariantFailure
```

Do not rewrite match lowering while doing native-core migration.

## 0.6 Reflection DTOs exist

Keep and complete:

```text
phalcom-semantic/src/reflection.rs
```

Existing concepts:

```rust
EnumReflection
VariantReflection
VariantFamilyReflection
VariantFieldReflection
ExactCaseTypeReflection
```

They are currently projections/scaffolds, not yet the finished reflection implementation.

## 0.7 Runtime reflection metadata DTO exists

Keep but substantially revise:

```text
phalcom-core/src/modules/reflection_metadata.rs
```

It already proves the desired projection boundary exists.

The defect is that the current product still stores things such as:

```rust
VariantId
VariantFamilyId
VariantFieldId
DeclarationId
```

directly and decides representation using core-name-derived declaration identity.

That does not yet satisfy the Part-06 persistence/runtime-decoupling contract.

## 0.8 Semantic source target kinds exist

Keep:

```rust
SemanticTargetId::Variant(...)
SemanticTargetId::VariantFamily(...)
SemanticTargetId::VariantField(...)
```

Do not repeat the old Task 16.

The remaining work is occurrence attachment and consumer integration.

## 0.9 Tooling DTOs exist

Keep and replace the shallow implementations in:

```text
phalcom-semantic/src/tooling/patterns.rs
```

Existing product names are good:

```rust
PatternCompletionContext
PatternCompletionCandidate
MissingCaseEditPlan
GeneratedMatchPlan
```

The current `GeneratedMatchPlan::from_enum_info` is not sufficient because it simply enumerates enum variants and renders strings. It is not residual-space-, GADT-, accessibility-, or witness-driven.

---

# 1. Correct the Actual Core-Migration Contract Before Editing Code

There are two important discrepancies between the old Part-06 examples and the repository that must be resolved explicitly.

## 1.1 Result

The repository currently exposes:

```phalcom
class Result<T, E>
class Ok<T, E>
class Err<T, E>
```

Part 06 ratifies canonical variants:

```phalcom
Result::Ok
Result::Error
```

The implementation must therefore intentionally migrate:

```text
Ok      class -> Result::Ok variant
Err     class -> Result::Error variant
```

Do not silently keep `Err` as the canonical variant merely because the old core source uses it.

During this migration, repo-owned call sites must be rewritten.

Run before editing:

```bash
rg -n '\bOk\.new\b|\bErr\.new\b|\bOk\(|\bErr\(' \
    phalcom-core phalcom-semantic phalcom-lsp examples docs \
    --glob '*.ph' --glob '*.rs' --glob '*.md'
```

Record the output in the implementation report.

## 1.2 Ordering

The old Part-06 example says:

```phalcom
Less
Equal
Greater
```

but the actual core language currently has a fourth observable state:

```phalcom
Ordering.unordered
```

and repository code uses it.

Do not delete that state.

The canonical migration is therefore:

```phalcom
@native
enum Ordering {
    @variant Less
    @variant Equal
    @variant Greater
    @variant Unordered
}
```

Preserve the existing lowercase API:

```phalcom
Ordering.less
Ordering.equal
Ordering.greater
Ordering.unordered
```

as ordinary compatibility class-side getters returning the corresponding singleton variants.

This gives a clean distinction:

```text
Ordering::Less
    semantic associated singleton variant

Ordering.less
    compatibility behavior returning Ordering::Less
```

Amend the Part-06 technical specification before calling the implementation complete.

---

# Phase 06.A — Real Core Migration

# Task 1 — Stop Publishing `Some` and `None` as Independent Semantic Core Classes

## Goal

Keep the existing runtime `Some`/`None` classes because immediate Option values need real case behavior classes for `.class`, but stop treating those runtime-support classes as independent language declarations.

This is the cleanest way to preserve the optimized object model while satisfying:

```text
VariantId != runtime case ClassId
```

## Files

Modify:

```text
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/tests/semantic/foundations/declarations.rs
phalcom-core/src/universe/core_classes.rs   # comments/invariants, not allocation removal
```

## Step 1.1 — Extend universe binding kind

Find:

```rust
pub enum UniverseBindingKind {
    Class,
}
```

Replace with:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum UniverseBindingKind {
    /// A runtime class that is also a canonical language declaration.
    Class,

    /// A runtime class required by the VM/object model but which must not
    /// create an independent source-semantic declaration.
    RuntimeSupportClass,
}
```

## Step 1.2 — Mark `Some` and `None` as runtime support

In `UNIVERSE_BINDINGS`, change only the entries for `Some` and `None`:

```rust
UniverseBindingSpec {
    key: UniverseKey::Some,
    name: "Some",
    kind: UniverseBindingKind::RuntimeSupportClass,
    exported: false,
    prelude: false,
},
UniverseBindingSpec {
    key: UniverseKey::None,
    name: "None",
    kind: UniverseBindingKind::RuntimeSupportClass,
    exported: false,
    prelude: false,
},
```

Leave:

```rust
UniverseKey::Option
```

as an ordinary semantic declaration identity because the canonical enum itself owns that declaration.

Runtime existence and source-semantic declaration existence are now explicitly separate.

## Step 1.3 — Filter support classes from declaration bootstrapping

In:

```text
phalcom-semantic/src/declarations.rs
```

change:

```rust
for binding in UNIVERSE_BINDINGS {
    let key = binding.key;
    let decl = universe_resolver(key);
```

to:

```rust
for binding in UNIVERSE_BINDINGS {
    if binding.kind == phalcom_native_meta::universe::UniverseBindingKind::RuntimeSupportClass {
        continue;
    }

    let key = binding.key;
    let decl = universe_resolver(key);
```

Import `UniverseBindingKind` directly if preferred.

## Step 1.4 — Add regression test

Add:

```rust
#[test]
fn option_case_behavior_classes_are_not_semantic_declarations() {
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(
        &mut store,
        &|key| DeclarationId::new(ModuleId::core(), key.name().into()),
    );

    let option = DeclarationId::new(ModuleId::core(), "Option".into());
    let some = DeclarationId::new(ModuleId::core(), "Some".into());
    let none = DeclarationId::new(ModuleId::core(), "None".into());

    assert!(declarations.get(&option).is_some());
    assert!(declarations.get(&some).is_none());
    assert!(declarations.get(&none).is_none());
}
```

## Step 1.5 — Preserve runtime classes

Do **not** delete this conceptual runtime allocation from:

```text
phalcom-core/src/universe/core_classes.rs
```

```rust
let option_class = ...;
let some_class = ...;
let none_class = ...;
```

Update its comment to say:

```text
Option is the language declaration/root runtime class.
Some and None are runtime case behavior classes for the canonical
Option variants. They are not independent semantic declarations.
```

## Verification

Run:

```bash
cargo fmt --all
cargo test -p phalcom-semantic declarations -- --nocapture
cargo test -p phalcom-core object_model -- --nocapture
cargo check -p phalcom-native-meta -p phalcom-semantic -p phalcom-core
```

Commit:

```text
refactor(core): separate Option case classes from semantic declarations
```

---

# Task 2 — Replace the Actual Core `Option` Class Hierarchy with `@native enum Option<T>`

## Files

Modify:

```text
phalcom-core/core/universe/src/option/option.ph
```

Do not create another Option source file.

## Step 2.1 — Remove declarations

Delete:

```phalcom
@native
class Option<T> is Object { ... }

@native
class Some<T> is Option<T> { ... }

@native
class None is Option {}
```

Do not delete the `Unit` declaration in this file.

## Step 2.2 — Replace them with the canonical enum

Use this declaration header:

```phalcom
@native
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
```

Then move the existing Option behavior into the enum root.

## Step 2.3 — Preserve the old functional `match` API as ordinary source behavior

The existing native:

```phalcom
@native
match(some: Dynamic, none: Dynamic) -> Dynamic
```

must stop being the implementation of case recognition.

Replace it with:

```phalcom
match(some: Dynamic, none: Dynamic) -> Dynamic {
    match self {
        Some(value) => some.call(value)
        None => none.call()
    }
}
```

This is valuable because almost all existing Option methods can continue using:

```phalcom
self.match(...)
```

while `match` itself is now implemented through canonical ADT pattern semantics.

It also provides a compatibility bridge for user code that deliberately uses the combinator.

## Step 2.4 — Preserve existing root methods

Do not redesign `Option`'s library API in this task.

Keep:

```text
ifNone
orElse
isSome
isNone
map
flatMap
filter
ifSome
unwrapOr
toString
okOr
==
hash
```

where currently present.

Only rewrite construction expressions that still depend on the old `Some` class.

Prefer canonical associated construction in core source:

```phalcom
Option::Some(value)
```

and canonical singleton lookup:

```phalcom
Option::None
```

For example, replace:

```phalcom
return self.match(
    some: |v| { Some(f.call(v)) },
    none: | | { None }
)
```

with:

```phalcom
return self.match(
    some: |v| { Option::Some(f.call(v)) },
    none: | | { Option::None }
)
```

This removes ambiguity during the bootstrap migration.

## Step 2.5 — Do not preserve `Some.new`

Delete the old class-side compatibility:

```phalcom
@class
@native
call(_ value: Dynamic) -> Some

@class
@native
new(_ value: Dynamic) -> Some
```

The canonical constructor is the variant constructor.

Search and migrate all repository-owned uses:

```bash
rg -n '\bSome\.new\(' .
```

Replacement:

```text
Some.new(x)
    -> Option::Some(x)
```

Do not implement `Some.new` by inventing a pseudo-class semantic identity.

## Step 2.6 — Add actual-core semantic test

The existing `native_core.rs` synthetic test is insufficient.

Add a test using the repository's actual core source-loading/session fixture.

Name:

```rust
#[test]
fn bootstrapped_core_option_is_canonical_enum()
```

It must prove all of:

```rust
let option = CoreDeclarationIds::default().option;

let enum_info = snapshot
    .enum_semantics
    .enum_info(&option)
    .expect("core Option must be an enum");

assert_eq!(enum_info.variants.len(), 2);

let some = /* exact Some(_) VariantId from enum_info */;
let none = /* exact None VariantId from enum_info */;

assert_eq!(
    snapshot.enum_semantics.variant_info(&some).unwrap().shape,
    VariantShape::Constructor,
);

assert_eq!(
    snapshot.enum_semantics.variant_info(&none).unwrap().shape,
    VariantShape::Singleton,
);
```

Also assert:

```rust
assert!(
    snapshot
        .enum_semantics
        .enum_info(&DeclarationId::new(ModuleId::core(), "Some".into()))
        .is_none()
);
```

and equivalent for `None`.

## Verification

Run:

```bash
cargo test -p phalcom-semantic --test semantic native_core -- --nocapture
cargo test -p phalcom-core option -- --nocapture
cargo check -p phalcom-semantic -p phalcom-core
```

Do not continue to runtime migration until the actual loaded core source produces canonical `EnumInfo`.

Commit:

```text
feat(core): migrate Option source to canonical native enum
```

---

# Task 3 — Replace `Result` / `Ok` / `Err` Classes with Canonical `Result<T,E>`

## Files

Modify:

```text
phalcom-core/core/universe/src/errors/error.ph
```

## Step 3.1 — Replace these declarations

Delete:

```phalcom
class Result<T, E> { ... }

class Ok<T, E> is Result<T, E> {
    @constructor
    new(_ v) { _value = v }

    match(ok, err) { ok.call(_value) }
}

class Err<T, E> is Result<T, E> {
    @constructor
    new(_ e) { _error = e }

    match(ok, err) { err.call(_error) }
}
```

## Step 3.2 — Insert this enum skeleton

```phalcom
@native
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Error(_ error: E)

    match(ok, err) {
        match self {
            Ok(value) => ok.call(value)
            Error(error) => err.call(error)
        }
    }

    isOk {
        self.match(
            ok: |v| { true },
            err: |e| { false }
        )
    }

    isErr {
        self.match(
            ok: |v| { false },
            err: |e| { true }
        )
    }

    map(_ f) {
        self.match(
            ok: |v| { Result::Ok(f.call(v)) },
            err: |e| { self }
        )
    }

    mapErr(_ f) {
        self.match(
            ok: |v| { self },
            err: |e| { Result::Error(f.call(e)) }
        )
    }

    andThen(_ f) {
        self.match(
            ok: |v| { f.call(v) },
            err: |e| { self }
        )
    }

    unwrap {
        self.match(
            ok: |v| { v },
            err: |e| { e.raise() }
        )
    }

    unwrapOr(_ default) {
        self.match(
            ok: |v| { v },
            err: |e| { default }
        )
    }

    unwrapErr {
        self.match(
            ok: |v| { v.raise() },
            err: |e| { e }
        )
    }

    ok() {
        self.match(
            ok: |v| { Option::Some(v) },
            err: |e| { Option::None }
        )
    }

    toString {
        self.match(
            ok: |v| { "Ok(" + v.toString + ")" },
            err: |e| { "Error(" + e.toString + ")" }
        )
    }
}
```

The exact declared result types may be restored from surrounding type annotations after this mechanical migration. Do not introduce new generic semantics here.

## Step 3.3 — Migrate all old constructors

Run:

```bash
rg -n '\bOk\.new\(|\bErr\.new\(' \
    phalcom-core examples phalcom-semantic phalcom-lsp \
    --glob '*.ph' --glob '*.rs'
```

Rewrite:

```text
Ok.new(value)
    -> Result::Ok(value)

Err.new(error)
    -> Result::Error(error)
```

Where expected-type contextual resolution is demonstrably available, shorthand may later be used, but use qualification during the migration.

## Step 3.4 — Migrate Result patterns

Search:

```bash
rg -n '\bErr\(' phalcom-core examples --glob '*.ph'
```

Where the expression is a Result pattern, rewrite:

```phalcom
Err(error)
```

to:

```phalcom
Error(error)
```

Do not rewrite unrelated identifiers containing `Err`.

## Step 3.5 — Result uses general runtime representation

Part 06 does **not** require Result to have an immediate representation.

Therefore:

```text
Result is native semantically
Result's initial physical representation = General
```

Do not create a tagged immediate Result representation in this part.

## Tests

Add actual-core test:

```rust
#[test]
fn bootstrapped_core_result_is_canonical_enum()
```

Assert:

```text
owner = CoreDeclarationIds::result
variants = Ok(_), Error(_)
both constructor-shaped
both have exactly one VariantFieldId
no semantic Result child declarations named Ok or Err
```

Add typing test:

```phalcom
const r: Result<Int, String> = Result::Ok(42)

match r {
    Ok(value) => value
    Error(error) => 0
}
```

Assert no diagnostics and canonical exact cases.

## Verification

```bash
cargo test -p phalcom-semantic --test semantic native_core -- --nocapture
cargo test -p phalcom-core result -- --nocapture
cargo test -p phalcom-core errors -- --nocapture
cargo check -p phalcom-core
```

Commit:

```text
feat(core): migrate Result to canonical native enum
```

---

# Task 4 — Migrate the Real Four-State `Ordering`

## Files

Replace:

```text
phalcom-core/core/universe/src/object/ordering.ph
```

## Use this source shape

```phalcom
@native
enum Ordering {
    @variant Less
    @variant Equal
    @variant Greater
    @variant Unordered

    @class
    less { Ordering::Less }

    @class
    equal { Ordering::Equal }

    @class
    greater { Ordering::Greater }

    @class
    unordered { Ordering::Unordered }

    reverse {
        match self {
            Less => Ordering::Greater
            Equal => Ordering::Equal
            Greater => Ordering::Less
            Unordered => Ordering::Unordered
        }
    }

    toString { toRepr }

    toRepr {
        match self {
            Less => "Ordering.less"
            Equal => "Ordering.equal"
            Greater => "Ordering.greater"
            Unordered => "Ordering.unordered"
        }
    }
}

export Ordering
```

Delete:

```text
_kind
_less
_equal
_greater
_unordered
private create constructor
new() guard
lazy singleton allocation
```

The enum singleton representation itself now owns singleton identity.

## Tests

Add:

```rust
#[test]
fn bootstrapped_core_ordering_has_four_canonical_cases()
```

Prove:

```text
Less
Equal
Greater
Unordered
```

all exist as singleton `VariantId`s.

Add runtime compatibility test:

```phalcom
Ordering.less === Ordering::Less
Ordering.equal === Ordering::Equal
Ordering.greater === Ordering::Greater
Ordering.unordered === Ordering::Unordered
```

Add reverse tests for all four cases.

## Verification

```bash
cargo test -p phalcom-core ordering -- --nocapture
cargo test -p phalcom-semantic --test semantic native_core -- --nocapture
```

Commit:

```text
feat(core): migrate Ordering to canonical native enum
```

---

# Task 5 — Wire Native Enum Extraction into Core-Surface Validation

The Part-06 commit added `SourceEnumRecord`, but the existing `merge_surfaces` path remains class-oriented.

Do not leave enum extraction as a test-only feature.

## Files

Modify:

```text
phalcom-semantic/src/core_surface/merge.rs
phalcom-semantic/src/core_surface/conformance.rs
phalcom-semantic/src/core_surface/presentation.rs
phalcom-semantic/tests/semantic/integration/native_conformance.rs
```

## Step 5.1 — Stop accepting only `SourceClassRecord`

Introduce:

```rust
#[derive(Clone, Debug)]
pub enum MergedDeclarationSource<'a> {
    Class(&'a SourceClassRecord),
    Enum(&'a SourceEnumRecord),
}
```

Replace:

```rust
pub struct MergedClassSurface<'a>
```

with the more accurate:

```rust
pub struct MergedDeclarationSurface<'a> {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub source: Option<MergedDeclarationSource<'a>>,
    pub members: BTreeMap<(DispatchSide, String), SurfaceMergeOutcome<'a>>,
}
```

Keep a compatibility wrapper temporarily only if other tests depend on `MergedClassSurface`.

## Step 5.2 — Change merge input

Replace:

```rust
pub fn merge_surfaces(
    source_classes: &[SourceClassRecord],
    native_records: &[NativeSurfaceRecord],
)
```

with:

```rust
pub fn merge_surfaces<'a>(
    source_declarations: &'a [SourceDeclarationRecord],
    native_records: &'a [NativeSurfaceRecord],
) -> Vec<MergedDeclarationSurface<'a>>
```

Dispatch:

```rust
match declaration {
    SourceDeclarationRecord::Class(class) => { ... }
    SourceDeclarationRecord::Enum(enum_) => { ... }
}
```

Enum root behavior enters the same native-member conformance machinery.

Variant declaration members do **not** become behavioral native methods.

## Step 5.3 — Validate native enum authorization

For:

```phalcom
@native
enum Option<T>
```

require:

```rust
CoreDeclarationIds::default().is_core_adt(&enum_.declaration_id)
```

or equivalent native-core authorization data.

A user module spelling:

```phalcom
@native enum Evil { ... }
```

must produce the existing unauthorized-native diagnostic rather than receive VM privilege.

## Step 5.4 — Validate native shapes

Add explicit semantic/core integration validation:

```text
Option:
    Some(_) constructor arity 1
    None singleton

Result:
    Ok(_) constructor arity 1
    Error(_) constructor arity 1

Ordering:
    Less singleton
    Equal singleton
    Greater singleton
    Unordered singleton
```

This validation belongs at the core/native binding boundary.

Do not encode these shapes in the general enum checker.

## Step 5.5 — Replace empty source merge test

Current test conceptually does:

```rust
let empty_sources = Vec::new();
let merged = merge_surfaces(&empty_sources, NATIVE_SURFACES);
```

Add a real core-source extraction fixture and pass:

```rust
extract_source_declarations(...)
```

into merge.

The test must prove that an actual native enum survives extraction and merge as an enum declaration rather than disappearing from the class-only pipeline.

## Verification

```bash
cargo test -p phalcom-semantic native_conformance -- --nocapture
cargo test -p phalcom-semantic native_enum_extraction -- --nocapture
```

Commit:

```text
feat(core-surface): integrate native enums into core conformance
```

---

# Task 6 — Move Runtime Representation Choice out of `VM::register_enum_from_spec`

Current code in:

```text
phalcom-core/src/vm/adt.rs
```

constructs:

```rust
DeclarationId::new(ModuleId::core(), "Option".into())
DeclarationId::new(ModuleId::core(), "Result".into())
```

inside runtime registration.

Delete that logic.

## Files

Modify:

```text
phalcom-core/src/adt.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/vm/adt.rs
```

## Step 6.1 — Fix the representation enum

Change:

```rust
pub enum RuntimeAdtRepresentation {
    General,
    NativeOption,
    NativeResult,
}
```

to:

```rust
pub enum RuntimeAdtRepresentation {
    /// Normal ADT singleton/AdtCaseObject representation.
    General,

    /// Core Option uses Value's nested Some-depth immediate representation.
    NativeOption,
}
```

Do not call Result's ordinary ADT representation `NativeResult`.

Native declaration provenance and physical representation are independent facts.

## Step 6.2 — Add representation to lowering spec

Change:

```rust
pub struct EnumLoweringSpec {
    pub owner: DeclarationId,
    pub variants: Box<[VariantLoweringSpec]>,
}
```

to:

```rust
pub struct EnumLoweringSpec {
    pub owner: DeclarationId,
    pub representation: RuntimeAdtRepresentation,
    pub variants: Box<[VariantLoweringSpec]>,
}
```

Because this file is in `phalcom-core`, it is an acceptable place to project semantic identity into a runtime representation policy.

## Step 6.3 — Select representation once during core projection

Inside `build_module_lowering_semantics`, create once:

```rust
let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
```

Then construct:

```rust
let representation = if core_ids.is_option(owner) {
    RuntimeAdtRepresentation::NativeOption
} else {
    RuntimeAdtRepresentation::General
};
```

and:

```rust
enums.push(EnumLoweringSpec {
    owner: owner.clone(),
    representation,
    variants: variants.into_boxed_slice(),
});
```

`Result` and `Ordering` therefore naturally use `General`.

## Step 6.4 — Delete runtime name reconstruction

Delete from `register_enum_from_spec`:

```rust
let option_decl = DeclarationId::new(ModuleId::core(), "Option".into());
let result_decl = DeclarationId::new(ModuleId::core(), "Result".into());

let representation = ...
```

Use:

```rust
spec.representation
```

directly.

## Tests

Add lowering test:

```rust
assert_eq!(option_spec.representation, RuntimeAdtRepresentation::NativeOption);
assert_eq!(result_spec.representation, RuntimeAdtRepresentation::General);
assert_eq!(ordering_spec.representation, RuntimeAdtRepresentation::General);
```

## Verification

```bash
cargo test -p phalcom-core lowering -- --nocapture
cargo test -p phalcom-core associated_lowering -- --nocapture
```

Commit:

```text
refactor(runtime): project ADT representation before VM registration
```

---

# Task 7 — Reuse the Existing Option Runtime Classes Instead of Allocating Duplicates

This is a critical migration task.

Current generic `register_enum_from_spec` allocates:

```text
new Option root class
new Option::Some case class
new Option::None case class
```

But the universe already owns:

```text
option_class
some_class
none_class
```

For native Option, reuse them.

## Files

Modify:

```text
phalcom-core/src/vm/adt.rs
phalcom-core/src/adt.rs
```

## Step 7.1 — Split class allocation from semantic registration

Add in `vm/adt.rs`:

```rust
struct RuntimeEnumClassBinding {
    root: ClassId,
    variants: std::collections::BTreeMap<VariantId, ClassId>,
}
```

Add:

```rust
impl VM {
    fn class_binding_for_enum(
        &mut self,
        spec: &EnumLoweringSpec,
    ) -> Result<RuntimeEnumClassBinding, RuntimeError> {
        match spec.representation {
            RuntimeAdtRepresentation::NativeOption => {
                self.bind_native_option_classes(spec)
            }
            RuntimeAdtRepresentation::General => {
                self.allocate_general_enum_classes(spec)
            }
        }
    }
}
```

## Step 7.2 — Add exact Option shape binding

`bind_native_option_classes` must validate exact semantic selector shape once.

Use:

```rust
Selector::method(
    "Some",
    vec![SelectorSlot::Positional],
)?
```

and:

```rust
Selector::getter("None")?
```

to construct the expected canonical `VariantId`s under:

```rust
spec.owner
```

This is allowed native representation binding code; it is not generic semantic resolution.

Then:

```rust
let expected_some = VariantId::new(
    spec.owner.clone(),
    Selector::method(
        "Some",
        vec![SelectorSlot::Positional],
    ).map_err(|error| RuntimeError::Internal(error.to_string()))?,
);

let expected_none = VariantId::new(
    spec.owner.clone(),
    Selector::getter("None")
        .map_err(|error| RuntimeError::Internal(error.to_string()))?,
);
```

Validate:

```text
expected_some exists
shape == Constructor
payload_fields.len() == 1

expected_none exists
shape == Singleton
payload_fields.len() == 0

no unexpected extra variant
```

Then return:

```rust
RuntimeEnumClassBinding {
    root: self.universe.classes.option_class,
    variants: BTreeMap::from([
        (expected_some, self.universe.classes.some_class),
        (expected_none, self.universe.classes.none_class),
    ]),
}
```

## Step 7.3 — Generic enums still allocate behavior classes

Move existing class allocation code into:

```rust
fn allocate_general_enum_classes(...)
```

Result and Ordering use this path.

## Step 7.4 — Register using selected ClassIds

Inside `register_enum_from_spec`, obtain:

```rust
let class_binding = self.class_binding_for_enum(spec)?;
```

then:

```rust
let enum_id = self.adt_registry.register_enum_with_representation(
    spec.owner.clone(),
    class_binding.root,
    spec.representation,
);
```

For each variant:

```rust
let behavior_class = *class_binding
    .variants
    .get(&var_spec.id)
    .ok_or_else(|| RuntimeError::Internal(
        format!("missing runtime behavior class for variant `{}`", var_spec.id.selector)
    ))?;
```

Pass that to `register_variant`.

Do not derive runtime identity from class name.

## Verification

Add test:

```rust
assert_eq!(
    vm.adt_registry.enum_descriptor(option_enum).unwrap().root_class,
    vm.universe.classes.option_class,
);

assert_eq!(
    vm.adt_registry.variant_descriptor(some_runtime).unwrap().behavior_class,
    vm.universe.classes.some_class,
);

assert_eq!(
    vm.adt_registry.variant_descriptor(none_runtime).unwrap().behavior_class,
    vm.universe.classes.none_class,
);
```

Also assert no duplicate Option root class was allocated.

Commit:

```text
refactor(runtime): bind Option variants to existing case classes
```

---

# Task 8 — Give Native Option Exact Runtime Variant Bindings

Current `runtime_variant_of` does this:

```text
if None -> find a singleton variant
if Some -> find a constructor variant
```

That is unacceptable.

The runtime must know:

```text
this RuntimeVariantId = Option::Some(_)
this RuntimeVariantId = Option::None
```

## Files

Modify:

```text
phalcom-core/src/adt.rs
phalcom-core/src/vm/adt.rs
```

## Step 8.1 — Add bound Option runtime IDs

Add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeOptionVariantIds {
    pub some: RuntimeVariantId,
    pub none: RuntimeVariantId,
}
```

Add to `RuntimeAdtRegistry`:

```rust
native_option: Option<NativeOptionVariantIds>,
```

Initialize through `Default`.

Add:

```rust
pub fn bind_native_option_variants(
    &mut self,
    some: RuntimeVariantId,
    none: RuntimeVariantId,
) -> Result<(), &'static str> {
    if self.native_option.is_some() {
        return Err("native Option variant identities already bound");
    }

    self.native_option = Some(NativeOptionVariantIds { some, none });
    Ok(())
}

pub fn native_option_variants(&self) -> Option<NativeOptionVariantIds> {
    self.native_option
}
```

## Step 8.2 — Bind after Option registration

During `register_enum_from_spec`, retain the newly registered runtime IDs for exact `Some` and `None`.

After both are registered:

```rust
self.adt_registry
    .bind_native_option_variants(some_runtime, none_runtime)
    .map_err(|message| RuntimeError::Internal(message.into()))?;
```

## Step 8.3 — Replace Option scanning

Delete this behavior:

```rust
for &v_id in &enum_desc.variants {
    ...
    if value.is_none() && v_desc.shape == Singleton { ... }
    if value.is_some() && v_desc.shape == Constructor { ... }
}
```

Replace with:

```rust
if value.is_option() {
    let variants = self.adt_registry.native_option_variants()?;
    return Some(if value.is_none() {
        variants.none
    } else {
        variants.some
    });
}
```

No selector comparison happens per value.

No variant scanning happens per value.

## Step 8.4 — Route payload operations through representation

Introduce:

```rust
fn native_option_payload_len(&self, value: Value) -> Option<usize>
fn native_option_payload_at(
    &self,
    value: Value,
    index: usize,
) -> Result<Value, RuntimeError>
```

Then make general public helpers dispatch by representation/case rather than accumulating unrelated special conditions.

Preserve exact current one-layer extraction:

```rust
value.with_some_depth(value.some_depth_raw() - 1)
```

or use:

```rust
value.option_case()
```

if the visibility boundary is adjusted.

## Tests

Must cover:

```text
None -> exact registered None RuntimeVariantId
Some(1) -> exact registered Some RuntimeVariantId
Some(None) -> Some RuntimeVariantId
Some(Some(1)) -> Some RuntimeVariantId
payload slot 0 peels exactly one layer
payload slot 1 fails
```

Commit:

```text
fix(runtime): bind immediate Option to exact variant identities
```

---

# Task 9 — Make `ConstructVariant` Representation-Aware

Constructing:

```phalcom
Option::Some(value)
```

must produce the existing immediate representation, not `AdtCaseObject`.

Constructing:

```phalcom
Result::Ok(value)
```

must produce the ordinary ADT representation.

## Files

Modify:

```text
phalcom-core/src/vm/adt.rs
phalcom-core/src/vm/dispatch.rs
```

## Step 9.1 — Add one central constructor

Add:

```rust
pub fn construct_variant_value(
    &mut self,
    variant: RuntimeVariantId,
    payload: Vec<Value>,
) -> Result<Value, RuntimeError>
```

Implementation:

```rust
let variant_desc = self
    .adt_registry
    .variant_descriptor(variant)
    .cloned()
    .ok_or_else(|| RuntimeError::Internal(
        format!("unknown runtime variant {}", variant.raw())
    ))?;

let enum_desc = self
    .adt_registry
    .enum_descriptor(variant_desc.enum_id)
    .cloned()
    .ok_or_else(|| RuntimeError::Internal(
        format!("unknown runtime enum {}", variant_desc.enum_id.raw())
    ))?;
```

Then:

```rust
match enum_desc.representation {
    RuntimeAdtRepresentation::NativeOption => {
        let ids = self
            .adt_registry
            .native_option_variants()
            .ok_or_else(|| RuntimeError::Internal(
                "native Option variants are not bound".into()
            ))?;

        if variant == ids.none {
            if !payload.is_empty() {
                return Err(/* existing arity error */);
            }
            return Ok(Value::none());
        }

        if variant == ids.some {
            let [value] = payload.as_slice() else {
                return Err(/* existing arity error */);
            };
            return value.wrap_some();
        }

        Err(RuntimeError::Internal(
            "non-Option variant registered under NativeOption representation".into(),
        ))
    }

    RuntimeAdtRepresentation::General => {
        // existing AdtCaseObject/singleton construction path
    }
}
```

Use the repository's existing arity error instead of inventing a new public runtime diagnostic if one already exists.

## Step 9.2 — Route bytecode through it

Find the `Bytecode::ConstructVariant` executor.

Replace inline `AdtCaseObject` construction with:

```rust
let value = self.construct_variant_value(runtime_variant, payload)?;
self.stack.push(value);
```

Likewise ensure `LoadVariantSingleton` returns:

```text
Option::None -> Value::none()
general singleton -> descriptor.singleton
```

Do not encode `Option` directly in compiler bytecode.

## Runtime tests

```phalcom
const x = Option::Some(42)
const y = Option::None

x.class
y.class
```

Assert:

```text
x has no Option wrapper object allocation
x.class == runtime Some case behavior class
y.class == runtime None case behavior class
```

Then:

```phalcom
const r = Result::Ok(42)
```

assert it uses ordinary ADT storage.

Commit:

```text
feat(runtime): construct native ADTs through representation strategy
```

---

# Task 10 — Delete the Old Option Semantic Compatibility Layer

Only do this after Tasks 2 and 6–9 pass.

## Search

```bash
rg -n \
'Option.*compat|core.*Option.*VariantId|constructor == "Some"|constructor == "None"|isSome.*variant|isNone.*variant' \
phalcom-semantic phalcom-core
```

## Required final state

Allowed:

```text
option.ph source names
NativeOption binding code
tests
documentation
Value::is_some / is_none physical representation helpers
```

Forbidden:

```text
semantic pattern resolution based on "Some"
semantic exhaustiveness based on "None"
compiler construction based on class Some
VM selecting semantic variant by shape
separate manually synthesized Option VariantId table
```

Delete any Part-05.2 compatibility semantic table that is now redundant.

Keep the representation adapter.

Commit:

```text
refactor(core): remove legacy Option semantic bridge
```

---

# Phase 06.B — Complete Exact-Case Reflection and Runtime Reflection

# Task 11 — Fix `ExactCaseTypeReflection`

The existing `ExactCaseTypeReflection::from_exact_case` currently does not perform true candidate specialization of field types.

## Files

Modify:

```text
phalcom-semantic/src/reflection.rs
phalcom-semantic/src/types/substitution.rs   # only if canonical helper is missing
```

## Required API

Change construction to accept all required semantic inputs explicitly:

```rust
pub fn from_exact_case(
    ty: TypeId,
    store: &mut TypeStore,
    enums: &EnumSemanticTable,
) -> Option<Self>
```

or the equivalent current immutable/mutable store split.

## Required algorithm

For:

```rust
TypeData::ExactCase {
    variant,
    enum_type,
}
```

perform:

1. recover canonical `VariantId`;
2. fetch `VariantInfo`;
3. derive the substitution from the exact enum specialization;
4. apply the same substitution machinery used by constructor invocation/pattern specialization;
5. specialize every `VariantFieldInfo.declared_type`;
6. use the case result specialization from `CaseTypeEnvironment`;
7. return canonical `TypeId`s.

Do not use:

```rust
declared_type.canonical_type()
```

as a substitute for specialization.

## Tests

Required:

```text
ExactCase(Some, Option<Int>)
    field value: Int

ExactCase(Some, Option<String>)
    field value: String

same VariantId
different ExactCase TypeId
different specialized field type
```

GADT:

```text
Expr::Int
    enum_type = Expr<Int>
    field = Int
    result = Expr<Int>
```

Commit:

```text
feat(reflection): specialize exact-case metadata canonically
```

---

# Task 12 — Replace Runtime Reflection's Local IDs with Stable Keys

Current:

```text
phalcom-core/src/modules/reflection_metadata.rs
```

stores compiler semantic identity structs directly.

That is acceptable as an in-memory prototype, not as the Part-06 persistent projection.

## Create semantic stable keys

Recommended file:

```text
phalcom-semantic/src/stable_identity.rs
```

Add:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableVariantKey {
    pub owner: DeclarationId,
    pub selector: Selector,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableVariantFamilyKey {
    pub owner: DeclarationId,
    pub base: SelectorBase,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableVariantFieldKey {
    pub variant: StableVariantKey,
    pub index: u32,
}
```

If an existing stable declaration key exists, substitute it for raw `DeclarationId` here.

Do not introduce a second module/project identity scheme.

## Change runtime metadata

Replace:

```rust
pub id: VariantId
```

with:

```rust
pub key: StableVariantKey
```

Replace:

```rust
pub family: Option<VariantFamilyId>
```

with:

```rust
pub family: Option<StableVariantFamilyKey>
```

Replace:

```rust
pub id: VariantFieldId
```

with:

```rust
pub key: StableVariantFieldKey
```

## Remove representation inference from reflection metadata

Delete:

```rust
if core_ids.is_option(owner) { ... }
else if core_ids.is_result(owner) { ... }
```

`ModuleReflectionMetadata` should project semantic reflection information.

Runtime representation belongs to lowering/materialization, not reflection ontology.

If runtime reflection needs to describe implementation provenance, store:

```rust
pub native: bool
```

not:

```rust
NativeOption / NativeResult
```

as semantic reflection identity.

## Tests

Build metadata twice with semantically equivalent stores and assert equality of stable metadata despite different local TypeIds/VariantTypeIds.

Commit:

```text
feat(metadata): stabilize ADT reflection projection
```

---

# Task 13 — Materialize Runtime Reflection Objects

The DTO modules currently exist, but Part 06 requires an actual public language reflection surface.

## Files

Inspect existing reflection object machinery, then implement in:

```text
phalcom-core/src/reflection/adt.rs
```

or the existing reflection module if one already owns metaobjects.

Do not put reflection semantics in `primitive/class.rs`.

## Required runtime objects

At minimum:

```text
EnumDescriptor
VariantDescriptor
VariantFamilyDescriptor
VariantFieldDescriptor
ExactCaseTypeDescriptor
```

The internal Rust names may reuse the existing `*Reflection` terminology.

## Required public relationships

Enum descriptor:

```text
variants
variantCount
variant(selector:)
variantFamily(named:)
```

Variant:

```text
enum
selector
family
fields
singleton?
constructor?
resultType
caseClass
```

Field:

```text
localName
externalLabel
type
```

Exact case:

```text
variant
enumType
fields
resultType
```

## Critical `.class` invariant

Keep:

```text
value.class
    -> runtime case behavior class
```

Separate:

```text
semantic type reflection
    -> ExactCaseTypeDescriptor
```

Tests must assert those are different objects/concepts.

Commit:

```text
feat(reflection): materialize ADT semantic metaobjects
```

---

# Phase 06.C — Finish Source Identity and Tooling Rather Than Leaving DTO Scaffolds

# Task 14 — Verify and Complete Pattern Source Attachments

Do not modify `SemanticTargetId`; the required cases already exist.

## Files

Modify as needed:

```text
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/editor.rs
```

## Required tests

For:

```phalcom
match option {
    Option::Some(value) => value
    None => 0
}
```

assert:

```text
Option token -> DeclarationId(Option)
Some token -> VariantId(Option::Some(_))
None token -> VariantId(Option::None)
value -> BindingId
```

For family:

```phalcom
Animal::Dog*
```

assert:

```text
Dog token -> VariantFamilyId
candidate set -> all resolved exact Dog VariantIds
```

For:

```phalcom
Dog(_, named: value)
```

attach:

```text
named -> exact VariantFieldId
```

or a deterministic set of VariantFieldIds for multi-candidate family patterns.

Never choose the first candidate.

Commit:

```text
feat(source-index): complete ADT pattern occurrence attachment
```

---

# Task 15 — Replace Shallow Pattern Tooling with Residual-Space Tooling

Current:

```rust
GeneratedMatchPlan::from_enum_info(...)
```

simply iterates variants.

Delete that implementation.

## Files

Modify:

```text
phalcom-semantic/src/tooling/patterns.rs
```

Use:

```text
MatchResolution
PatternSpace
CoverageWitness
ResolvedVariantPattern
```

as inputs.

## Change `PatternCompletionContext`

Current:

```rust
pub struct PatternCompletionContext {
    pub expected: TypeKnowledge,
    pub candidates: Box<[PatternCompletionCandidate]>,
    pub wildcard_recommended: bool,
}
```

Add an explicit residual summary:

```rust
pub struct PatternCompletionContext {
    pub expected: TypeKnowledge,
    pub residual: PatternSpaceSummary,
    pub candidates: Box<[PatternCompletionCandidate]>,
    pub wildcard_recommended: bool,
}
```

Define `PatternSpaceSummary` as a bounded editor product rather than exposing internal recursive proof representation directly.

## Required candidate policy

Order:

```text
1 exact uncovered reachable cases
2 useful families covering residual cases
3 wildcard if opaque/unspellable residual remains
```

Filter:

```text
impossible GADT cases
illegal explicit spellings
already-covered exact cases
```

Do not remove inaccessible cases from formal exhaustiveness.

## Missing-case plan

`MissingCaseEditPlan` must be created from residual/witness products, not all `EnumInfo.variants`.

## Generate-match plan

Generate from the initial closed pattern space for the expression's actual type.

Handle:

```text
closed enum
GADT
closed union of exact/root cases
```

Use field `local_name` to suggest payload bindings.

Check lexical collisions before emitting binding names.

Commit:

```text
feat(tooling): derive match edits from semantic pattern space
```

---

# Task 16 — Wire the Semantic Tooling Products into LSP

The current repository does not consume `PatternCompletionContext` from `phalcom-lsp`; the product exists only in semantic code/documentation.

## Files

Modify:

```text
phalcom-lsp/src/completion.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/diagnostics.rs
```

and the current definition/rename/semantic-token/code-action handlers under `phalcom-lsp/src`.

## Completion

LSP must ask the semantic/editor service for:

```rust
PatternCompletionContext
```

and only translate it to protocol items.

Delete LSP-side variant-name scanning if present.

## Hover

Use:

```text
TypePresenter
VariantReflection
ExactCaseTypeReflection
MatchResolution branch proof
```

Do not build exact-case strings in LSP.

## Definition

Resolve from:

```rust
SemanticTargetId
```

Rules:

```text
Variant -> exact @variant source declaration
VariantFamily -> every declaration site in family
VariantField -> payload field declaration
```

## Rename

Base variant rename operates on:

```rust
VariantFamilyId
```

not source text.

External payload labels use:

```rust
VariantFieldId
```

and run selector collision validation.

## Code actions

Add:

```text
Add Missing Match Cases
Generate Match
```

using the semantic plans from Task 15.

LSP owns:

```text
indentation
text edits
snippet placeholders
protocol ranges
```

Semantic layer owns:

```text
which cases
which fields
which qualification
which cases are reachable
which cases are legal to spell
```

Commit:

```text
feat(lsp): consume canonical ADT tooling products
```

---

# Phase 06.D — Conformance, Incremental, Persistence, Deletion

# Task 17 — Replace Synthetic Native-Core Tests with Actual Core Integration Tests

The existing synthetic tests are useful unit tests but must not be accepted as the Part-06 proof.

Keep them as declaration-language tests.

Add a new integration group that loads the actual core universe.

Recommended:

```text
phalcom-semantic/tests/semantic/adts/core_integration.rs
phalcom-core/tests/core/language/algebraic_data/native_core.rs
```

Required vertical cases:

## Option

```phalcom
const x: Option<Int> = Option::Some(42)

match x {
    Some(value) => value
    None => 0
}
```

Verify:

```text
actual core EnumInfo
canonical VariantIds
ExactCase(Some, Option<Int>)
formal binding = Option<Int>
observed evidence = exact Some where justified
immediate runtime value
correct case class
reflection
```

## Result

```phalcom
const x: Result<Int, Error> = Result::Ok(42)

match x {
    Ok(value) => value
    Error(error) => 0
}
```

Verify general runtime ADT representation.

## Ordering

Check all four cases.

## Bool

Verify semantic enum table still has no Bool enum.

Do not infer from the existence of runtime `True`/`False` classes that Bool became an ADT.

Commit:

```text
test: verify bootstrapped native ADTs end to end
```

---

# Task 18 — Cross-Module and Incremental Tests

Add:

```text
phalcom-semantic/tests/semantic/adts/modules.rs
phalcom-semantic/tests/semantic/adts/incremental.rs
```

## Cross-module fixture

```text
A:
    enum Response<T> ...

B:
    constructs Response

C:
    matches Response
```

Assert all imports/re-exports retain the defining `VariantId`.

## Incremental fixture 1

Start:

```phalcom
enum E {
    @variant A
    @variant B
}
```

match A/B exhaustively.

Edit source to add:

```phalcom
@variant C
```

Assert recomputation of:

```text
EnumInfo fingerprint
MatchResolution
CoverageWitness
PatternCompletionContext
MissingCaseEditPlan
reflection metadata
```

## Incremental fixture 2

Change a GADT result specialization and assert:

```text
ExactCase
branch proof
hover
reflection
```

invalidate.

## Incremental fixture 3

Change construction visibility and prove:

```text
completion changes
reflection acquisition changes
exhaustiveness universe does not
```

Commit:

```text
test: cover ADT cross-module and incremental convergence
```

---

# Task 19 — Final Architecture Deletion Search

Run all of these on final HEAD.

```bash
rg -n 'constructor == "Some"|constructor == "None"' \
    phalcom-core phalcom-semantic phalcom-lsp
```

Expected: none.

```bash
rg -n 'DeclarationId::new\(ModuleId::core\(\), "Option"' \
    phalcom-core/src
```

Allowed only in explicit native Option binding/bootstrap code.

Not allowed in generic matching/construction/reflection.

```bash
rg -n 'DeclarationId::new\(ModuleId::core\(\), "Result"' \
    phalcom-core/src
```

Expected: no runtime representation special case.

```bash
rg -n '\bclass Option\b|\bclass Some\b|\bclass None\b' \
    phalcom-core/core/universe
```

Expected: no source-semantic class declarations.

Comments describing runtime classes must be clearly labelled runtime-only.

```bash
rg -n '\bclass Result\b|\bclass Ok\b|\bclass Err\b' \
    phalcom-core/core/universe
```

Expected: none.

```bash
rg -n '\bclass Ordering\b' \
    phalcom-core/core/universe
```

Expected: none.

```bash
rg -n '\bOk\.new\(|\bErr\.new\(' \
    phalcom-core examples
```

Expected: none.

```bash
rg -n 'PatternCompletionContext|MissingCaseEditPlan|GeneratedMatchPlan' \
    phalcom-lsp/src
```

Expected: actual consumers, not zero hits.

```bash
rg -n 'EnumReflection|VariantReflection|ExactCaseTypeReflection' \
    phalcom-core/src phalcom-lsp/src
```

Expected: real reflection/editor consumers.

```bash
rg -n 'RuntimeVariantId|CaseDiscriminant|VariantTypeId|TypeId' \
    phalcom-core/src/modules \
    | grep -E 'reflection|serial|persist|artifact|metadata'
```

Review every hit.

No local runtime/store ID may act as persisted semantic identity.

---

# Task 20 — Documentation Corrections

Modify:

```text
docs/spec/adts.md
docs/spec/current/core/*
docs/impl/adt-gadt-associated-lookup/part-6/*
```

Required corrections:

```text
Option = @native enum
Result = @native enum
Ordering = @native enum
Ordering has Unordered in current core semantics
Bool remains primitive
Some/None runtime classes are case behavior classes, not variant declarations
Err class no longer exists
Result::Error is the canonical failure variant
Result physical representation is initially General
Option physical representation is NativeOption
.class returns runtime case behavior class
ExactCase reflection is semantic type reflection
:: is not reflection syntax
```

Update old comments such as:

```text
hidden case behavior class
```

to:

```text
runtime case behavior class
```

when the old wording implies non-reflectability.

Commit:

```text
docs: synchronize native ADT core semantics
```

---

# Task 21 — Full Final Verification

Run exactly:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log -1 --oneline
```

Then:

```bash
cargo fmt --all -- --check
```

Then:

```bash
cargo check -p phalcom-native-meta
cargo test -p phalcom-native-meta -- --nocapture
```

Then:

```bash
cargo check -p phalcom-ast
cargo test -p phalcom-ast -- --nocapture
```

Then:

```bash
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic --lib -- --nocapture
cargo test -p phalcom-semantic --test semantic -- --nocapture
```

Then:

```bash
cargo check -p phalcom-core
cargo test -p phalcom-core --lib -- --nocapture
cargo test -p phalcom-core --tests -- --nocapture
```

Then:

```bash
cargo check -p phalcom-lsp
cargo test -p phalcom-lsp -- --nocapture
```

Then rerun the focused ADT groups:

```bash
cargo test -p phalcom-semantic --test semantic adts -- --nocapture
cargo test -p phalcom-semantic native_core -- --nocapture
cargo test -p phalcom-core algebraic_data -- --nocapture
cargo test -p phalcom-core option -- --nocapture
cargo test -p phalcom-core result -- --nocapture
cargo test -p phalcom-core ordering -- --nocapture
cargo test -p phalcom-core match -- --nocapture
```

Adjust only test-target spelling if the current Cargo test hierarchy requires it; do not silently omit a suite.

Finally rerun every Task-19 architecture search and paste the output into the implementation report.

---

# Recommended Commit Sequence

Use small convergence commits rather than one enormous Part-06 commit:

```text
1.  refactor(core): separate Option case classes from semantic declarations
2.  feat(core): migrate Option source to canonical native enum
3.  feat(core): migrate Result to canonical native enum
4.  feat(core): migrate Ordering to canonical native enum
5.  feat(core-surface): integrate native enums into core conformance
6.  refactor(runtime): project ADT representation before VM registration
7.  refactor(runtime): bind Option variants to existing case classes
8.  fix(runtime): bind immediate Option to exact variant identities
9.  feat(runtime): construct native ADTs through representation strategy
10. refactor(core): remove legacy Option semantic bridge
11. feat(reflection): specialize exact-case metadata canonically
12. feat(metadata): stabilize ADT reflection projection
13. feat(reflection): materialize ADT semantic metaobjects
14. feat(source-index): complete ADT pattern occurrence attachment
15. feat(tooling): derive match edits from semantic pattern space
16. feat(lsp): consume canonical ADT tooling products
17. test: verify bootstrapped native ADTs end to end
18. test: cover ADT cross-module and incremental convergence
19. docs: synchronize native ADT core semantics
```

---

# Final Acceptance Criteria

Do not call Part 06 complete unless all of these statements are true.

```text
The actual core Option source is an @native enum.

The actual core Result source is an @native enum.

The actual core Ordering source is an @native enum.

Ordering preserves the existing Unordered state.

Bool remains a primitive semantic type.

Some/None still have runtime ClassIds, but those ClassIds are runtime case
behavior classes rather than semantic declarations.

Option::Some and Option::None own ordinary VariantIds.

Result::Ok and Result::Error own ordinary VariantIds.

Ordering's four cases own ordinary VariantIds.

Option construction through ConstructVariant creates the immediate Value
representation.

Result construction through ConstructVariant creates ordinary ADT values.

The VM never discovers Some/None semantic identity by scanning variant shape.

The VM does not construct "Option" or "Result" DeclarationIds inside generic
runtime execution to decide language semantics.

Option's pre-existing root/Some/None runtime classes are reused; duplicate
case classes are not allocated.

.class on an Option value returns the corresponding Some/None runtime case
behavior class.

.class does not replace ExactCase or VariantId anywhere in static semantics.

ExactCase(Some, Option<Int>) and ExactCase(Some, Option<String>) remain
different canonical TypeIds sharing one VariantId.

Exact-case reflection actually specializes payload/result types.

Runtime reflection metadata uses reconstruction-safe semantic keys.

SemanticTargetId VariantFamily/VariantField are attached to real pattern
source occurrences.

Pattern completion is driven by residual PatternSpace.

Missing-case generation consumes CoverageWitness/residual semantics.

Generate Match does not merely iterate EnumInfo.variants.

The LSP consumes semantic tooling products instead of recreating pattern
reasoning.

No source class Result/Ok/Err hierarchy remains.

No source class Option/Some/None hierarchy remains.

No source class Ordering remains.

No Part-05 temporary Option semantic identity bridge remains.

No Result-specific immediate representation is introduced under Part 06.

Part 05.2 match lowering continues passing unchanged.

The only work handed to Part 07 is performance/representation optimization.
```

# Architectural End State

After these tasks, the core path is:

```text
core source
    @native enum Option<T>
    @native enum Result<T,E>
    @native enum Ordering
        ↓
ordinary enum declaration analysis
        ↓
EnumInfo / VariantId / VariantFieldId / ExactCase
        ↓
ordinary associated + pattern semantics
        ↓
semantic lowering
        ↓
EnumLoweringSpec
        ├── Option -> NativeOption representation
        └── Result/Ordering -> General representation
        ↓
RuntimeAdtRegistry
        ├── Option
        │     root class      = existing Option ClassId
        │     Some case class = existing Some ClassId
        │     None case class = existing None ClassId
        │     values          = immediate Value encoding
        │
        ├── Result
        │     runtime case behavior classes
        │     ordinary ADT values
        │
        └── Ordering
              runtime case behavior classes
              ordinary singleton ADT values
        ↓
runtime reflection metadata
        ↓
semantic reflection / LSP projections
```

That is the Part-06 convergence condition. The important difference from the previous plan is that the migration is now about replacing the *real bootstrapped core model*, rather than adding more semantic scaffolding around a class-based Option/Result/Ordering implementation.