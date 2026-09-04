# Phalcom `print` Intrinsic Family Object
## Requirements Analysis → Technical Specification → Patch-Grade Implementation Plan

This design is repository-grounded against the current Phalcom architecture.

The most important finding from the repository analysis is that `print` fits the existing architecture better than it initially appeared:

- `System.print(_)` already exists as the canonical native operation, currently typed `(Object) -> Unit`; it calls `Value::to_display_string`, emits a newline, and writes directly to stdout.
- `FamilyObject` already has precisely the representation we need: an immutable receiver plus an exact or pattern selector. There is no need for a new `Value` or `Object` kind.
- the canonical Universe source already declares `System.print(_)`, so the language-level semantic authority can remain source-owned rather than moving into compiler magic.
- the native surface already has a general `NativeIntrinsicId` channel and supports `intrinsic = ...` declarations; `BoolAnd`, `BoolOr`, and `BoolNot` already use it.
- native callable metadata already supports generic callable type parameters, including the exact syntax `<T>(T) -> ...`.
- the semantic-to-codegen pipeline already projects resolved family applications into immutable `ModuleLoweringSemantics`, and the compiler already has a dedicated static-family application lowering path.

There is, however, one real architectural prerequisite that this feature exposes:

> Phalcom currently has an explicit **type prelude**, but not an equally principled **value prelude**.

`PreludeTypeMap` is deliberately type-only and maps prelude type names to canonical source `DeclarationId`s.  At runtime, the VM's current prelude synchronization is likewise explicitly centered on Universe class bindings and even comments that it records only class declarations.

Therefore `print` should not be hacked into the existing type-prelude machinery. This feature should introduce the minimal general concept of a **canonical Universe value-prelude binding**.

That is the only substantial architectural extension required.

---

# Part I — Requirements Analysis

## 1. Objective

The objective is to make:

```phalcom
print("hello")
print(42)
print(value)
```

the canonical Phalcom printing surface while satisfying all of the following simultaneously:

1. `print` is a genuine first-class Phalcom value.
2. Its runtime value is a genuine `Family`.
3. Its callable behavior is the same behavior obtained through ordinary Family invocation.
4. Its typing uses ordinary generic callable semantics.
5. Its intrinsic implementation is identified by canonical semantic identity, never source spelling.
6. direct calls may bypass family dispatch and native-call overhead.
7. statically proven argument representations may receive further specialized lowering.
8. dynamic or indirect calls continue to work correctly without any optimization.
9. `System.print` remains the underlying canonical operation and compatibility surface.
10. no parser-level or AST-level special case is introduced.

The architectural rule is:

> **Intrinsicness is an execution property of a canonically resolved callable, not a different kind of language value or call.**

---

# 2. Current State Analysis

## 2.1 Current language surface

Today:

```phalcom
System.print("hello")
```

ultimately reaches:

```rust
system_class_print(...)
```

whose native declaration currently states:

```text
System.class::print(_)
(Object) -> Unit
```

and whose implementation performs approximately:

```text
Value
  ↓
to_display_string(vm)
  ↓
String
  ↓
stdout
  ↓
newline
```

This works but has three problems.

### Ergonomics

`System.print(...)` is unnecessarily verbose for one of the most common operations in a language.

### Invocation overhead

The ordinary surface can require normal receiver lookup, message dispatch/native dispatch, argument machinery, and a native boundary for an operation whose exact implementation is statically knowable in common cases.

### Representation overhead

The current implementation constructs a display string before output. Even after eliminating family dispatch, primitive printing would still have avoidable formatting/allocation overhead.

---

# 3. Existing Family Semantics Fit the Design

A significant correction from the conceptual discussion is important.

In current Phalcom, a `Family` is not an abstract global overload set. It is a bound callable object carrying:

```rust
FamilyObject {
    receiver: Value,
    spec: FamilySpec,
}
```

where the spec is either an exact selector or selector pattern.

Therefore the correct conceptual definition of `print` is:

```phalcom
const print = System::#print(_)
```

not:

```phalcom
family print {
    ...
}
```

and not a new compiler-created callable category.

Conceptually:

```text
print
  │
  ▼
FamilyObject
  receiver = canonical System class object
  spec     = Exact("print(_)")
```

This is excellent because it means the language does not need any new callable semantics.

---

# 4. Required Surface Semantics

## REQ-PRINT-01 — `print` is an ordinary value

The following must be legal:

```phalcom
let p = print
p("hello")
```

and:

```phalcom
fn use(_ f) {
    f("hello")
}

use(print)
```

and any other operation that accepts a compatible callable/family value.

No "builtin function value" exception may be required.

---

## REQ-PRINT-02 — Runtime representation is ordinary `Family`

At runtime:

```phalcom
print
```

must evaluate to the same kind of object produced by:

```phalcom
System::#print(_)
```

That means:

```text
Object::Family(FamilyObject)
```

using the existing representation.

There must be:

- no `Value::Intrinsic`;
- no `Object::IntrinsicFunction`;
- no `BuiltinPrintObject`;
- no separate print-callable hierarchy.

---

# 5. Canonical Binding Requirement

`print` must be a canonical Universe-owned immutable value binding.

The preferred source definition is conceptually:

```phalcom
const print = System::#print(_)
```

located alongside the canonical `System` definition.

Because `const` initialization runs once for the canonical Universe module, the value becomes one canonical runtime Family instance.

All implicit prelude reads of `print` should point to that source binding rather than constructing a fresh Family.

Thus:

```phalcom
let a = print
let b = print
```

retrieve the same canonical family value.

---

# 6. Value Prelude Requirement

This is the first prerequisite Phalcom does not currently fully possess.

The existing canonical semantic prelude is explicitly a `PreludeTypeMap`.

The feature therefore requires a parallel concept:

```rust
PreludeValueMap
```

or, preferably, a generalized prelude abstraction that distinguishes:

```text
Prelude
├── types
└── values
```

I recommend the former initially to avoid destabilizing the working type-prelude implementation:

```rust
pub struct PreludeValueMap {
    entries: BTreeMap<Box<str>, SymbolId>,
}
```

where `SymbolId` already represents a canonical module-owned global declaration as:

```rust
SymbolId {
    module: ModuleId,
    name: Box<str>,
}
```

and linked reads already support arbitrary binding symbols.

This is a much better fit than attempting to represent `print` as a `DeclarationId`, because `print` is a value binding rather than a nominal type declaration.

---

# 7. Explicit Prelude Policy Requirement

Prelude membership must remain declarative.

Do not make the VM scan Universe globals for a global named `"print"`.

Do not make semantic analysis contain:

```rust
if name == "print" { ... }
```

Instead introduce explicit policy analogous to the existing class-prelude policy.

Conceptually:

```rust
pub struct UniversePreludeValueSpec {
    pub module_path: &'static [&'static str],
    pub name: &'static str,
}
```

with:

```rust
pub const UNIVERSE_PRELUDE_VALUES: &[UniversePreludeValueSpec] = &[
    UniversePreludeValueSpec {
        module_path: &["concurrency", "fiber"],
        name: "print",
    },
];
```

The table specifies only:

> this source-owned value is implicitly visible.

It must not duplicate its type, target selector, intrinsic ID, or implementation.

Those already have better authorities:

```text
source initializer      → System::#print(_)
source method           → generic callable type
native surface          → NativeIntrinsicId::Print
semantic analysis       → actual resolved target
```

This avoids metadata drift.

---

# 8. Shadowing Requirement

Prelude visibility does not make `print` reserved syntax.

Local shadowing must work exactly as normal lexical shadowing works.

For example:

```phalcom
fn run(print) {
    print("hello")
}
```

must invoke the parameter.

Likewise:

```phalcom
let print = logger
print("hello")
```

must invoke `logger`.

The compiler must never recover builtin semantics merely from the text `"print"`.

---

# 9. Generic Callable Requirement

The underlying callable should become:

```phalcom
@classmethod-like
print<T>(_ value: T) -> Unit
```

using actual Phalcom syntax:

```phalcom
@class @native
print<T>(_ value: T) -> Unit
```

The native contract should correspond to:

```text
<T>(T) -> Unit
```

The native callable metadata infrastructure already supports generic type parameters and parameter references.

This is preferable to:

```phalcom
print(_ value: Object) -> Unit
```

because semantic application then preserves:

```text
print(42)
    T := Int

print("hello")
    T := String

print(x: Dynamic)
    T := Dynamic
```

That gives lowering an already-solved specialization.

---

# 10. No Semantic Primitive Overloads

The following should explicitly not be introduced:

```phalcom
print(_ value: Int)
print(_ value: Float)
print(_ value: String)
print(_ value: Bool)
```

Those would alter family candidate selection and make optimization part of language overload semantics.

The desired architecture is instead:

```text
semantic callable:
    print<T>(T) -> Unit

implementation specializations:
    Print<Int>
    Print<Float>
    Print<String>
    Print<Bool>
    Print<Generic>
```

One semantic callable. Multiple executable strategies.

---

# 11. Callable Identity Requirement

Static specialization must never create distinct semantic callable identities.

These calls:

```phalcom
print(1)
print(2.5)
print("hello")
```

all resolve to the same:

```text
CallableId(System.class::print(_))
```

with different substitutions.

Conceptually:

```text
call #1
    callable = C
    { T → Int }

call #2
    callable = C
    { T → Float }

call #3
    callable = C
    { T → String }
```

This agrees with Phalcom's existing generic-call architecture: specialization is not supposed to mutate selector or callable identity.

---

# 12. Intrinsic Identity Requirement

The repository already has:

```rust
enum NativeIntrinsicId {
    BoolAnd,
    BoolOr,
    BoolNot,
}
```

so extend it:

```rust
enum NativeIntrinsicId {
    BoolAnd,
    BoolOr,
    BoolNot,
    Print,
}
```

Do not invent another `IntrinsicId` enum.

The native primitive declaration becomes conceptually:

```rust
#[primitive(
    System,
    "print(_)",
    ...
    intrinsic = Print
)]
```

The current primitive metadata specification explicitly defines intrinsic metadata as permission for compiler-maintained specialized implementation paths.

---

# 13. Intrinsic Recognition Requirement

Intrinsic recognition must happen from resolved identity:

```text
resolved Family application
        ↓
resolved CallableId
        ↓
implementation provenance
        ↓
NativeIntrinsicId::Print
```

Never:

```rust
if variable_name == "print"
```

Never:

```rust
if selector.encode() == "print(_)"
```

Never merely:

```rust
if owner_name == "System"
```

String matching may occur when parsing canonical metadata, but never as the compiler's proof that an arbitrary call is intrinsic.

---

# 14. Ordinary Semantic Resolution Requirement

Intrinsic calls must first undergo ordinary semantics.

For:

```phalcom
print(x)
```

the semantic pipeline remains:

```text
resolve identifier `print`
        ↓
resolve canonical prelude value binding
        ↓
obtain Family type/denotation
        ↓
resolve Family application
        ↓
resolve System.class::print(_)
        ↓
generic application
        ↓
solve T from x
        ↓
produce ordinary resolved call
        ↓
only now consider intrinsic execution
```

There must not be a parallel `typecheck_print()` path.

This is one of the central correctness constraints.

---

# 15. Static Specialization Requirement

After ordinary application is complete, lowering may derive an executable specialization.

Recommended initial set:

```rust
pub enum PrintSpecialization {
    Generic,
    Int,
    Float,
    Bool,
    String,
}
```

Do not add every language type immediately.

The value of these initial cases is that they cover the dominant primitive paths while leaving everything else semantically complete through `Generic`.

---

# 16. Representation-Proof Requirement

This point needs to be stricter than our earlier conceptual discussion.

A nominal static type is not automatically sufficient to bypass user-visible display behavior.

For example, if Phalcom permits a value statically typed as some superclass but dynamically holding a subclass with custom display semantics, direct primitive formatting would be incorrect.

Therefore:

> A specialized print lane may be selected only when semantic/backend knowledge proves the runtime representation strongly enough that bypassing generic display dispatch is observationally equivalent.

So:

```text
proven exact Int representation
    → PrintSpecialization::Int

merely Number
    → Generic

Dynamic
    → Generic

unknown/open object hierarchy
    → Generic
```

The optimization proof must be conservative.

---

# 17. Dynamic Completeness Requirement

This must work regardless of static knowledge:

```phalcom
let value: Dynamic = something()
print(value)
```

It should still receive an intrinsic call if the **callee identity** is known to be canonical `print`.

The lowering can be:

```text
Intrinsic Print(Generic)
```

rather than falling all the way back to Family dispatch.

Thus there are two independent proof axes:

```text
callee identity known?       argument representation known?
```

producing:

| Callee | Argument | Execution |
|---|---|---|
| unknown | any | ordinary callable/family path |
| canonical print | unknown/dynamic | intrinsic generic print |
| canonical print | exact supported representation | specialized intrinsic print |

This is the correct optimization lattice.

---

# 18. Indirect Family Invocation Requirement

For:

```phalcom
let p = print
p(42)
```

correctness must not depend on the optimizer proving `p == print`.

Baseline behavior:

```text
p
 ↓
ordinary FamilyObject
 ↓
Family invocation
 ↓
System.print(_)
 ↓
native fallback
```

Later constant propagation may prove:

```text
p == canonical print
```

and recover intrinsic lowering.

That is explicitly a future optimization, not required by version one.

---

# 19. Closed-Kernel Requirement

Direct intrinsic lowering bypasses live family dispatch, so it is only semantically valid if the underlying canonical target cannot be replaced.

Phalcom's accepted closed-class model already provides the relevant foundation: classes are defined once and kernel classes are protected from user mutation/reopening.

Therefore direct intrinsic execution of canonical `System.print(_)` is valid provided future reflection remains consistent with the existing rule that kernel behavior cannot be mutated.

This should be stated explicitly in the spec.

---

# 20. Output Semantics Requirement

Version one must preserve current `System.print` behavior:

```text
display representation(value)
+
newline
+
return Unit
```

So:

```phalcom
print(x)
```

and:

```phalcom
System.print(x)
```

must produce byte-equivalent output and the same return value.

No formatting redesign should be mixed into this patch.

Specifically defer:

- `end:`;
- `sep:`;
- variadic values;
- stderr destination;
- stream/file destination;
- formatting templates;
- pretty printing versus debug printing.

---

# 21. Display-Semantics Requirement

For objects and any case where representation equivalence is not proven, the generic intrinsic must retain current display semantics.

Conceptually:

```text
Print(Generic, value)
    ↓
existing display path
    ↓
output
```

It must not silently change from user-visible display semantics to debug formatting.

---

# 22. Primitive Fast-Path Requirement

Once equivalence has been verified, primitive-specialized implementations should avoid intermediate heap `String`s wherever possible.

For example:

```text
Int
 ↓
writer formatting
 ↓
stdout
```

rather than:

```text
Int
 ↓
heap String
 ↓
stdout
```

Likewise for `Bool`:

```text
true → static bytes "true"
```

and String:

```text
String backing bytes → stdout
```

The performance requirement is:

> Primitive-specialized `print` must not allocate merely to convert the primitive into an intermediate display string when the output can be emitted directly with equivalent semantics.

---

# 23. Shared Runtime Implementation Requirement

Do not maintain two separate definitions of printing.

The architecture should be:

```text
                        ┌─ intrinsic specialized call
                        │
                        ▼
                 shared runtime print engine
                        ▲
                        │
System.print primitive ─┘
```

The native `System.print` primitive remains the ordinary fallback.

This prevents semantic drift.

---

# 24. I/O Abstraction Scope Decision

The current primitive writes directly via process stdout.

A VM-owned output sink would eventually be preferable for:

- embedding;
- capture;
- tests;
- REPLs;
- independent VM instances;
- redirected output.

However, that is an orthogonal I/O architecture project.

For this patch:

> Centralize printing behind one VM/runtime helper, but do not require the introduction of a complete pluggable output-sink subsystem.

Design the helper so an output sink can replace its internals later.

This keeps the `print` work focused.

---

# 25. Side-Effect Requirement

`print` must be explicitly marked as I/O-bearing native functionality.

The native metadata system already has `NativeEffect::Io`.

The primitive should therefore carry:

```rust
effects = [io]
```

This gives optimizers an explicit reason not to:

- dead-code eliminate printing;
- reorder print operations across observable effects;
- hoist prints;
- merge prints without proof.

---

# 26. Evaluation-Order Requirement

Intrinsic lowering must preserve normal call evaluation semantics.

For:

```phalcom
print(foo())
```

`foo()` is evaluated exactly once before printing.

Future extensions such as:

```phalcom
print(foo(), end: bar())
```

must follow normal argument evaluation order if they are ever added.

Intrinsic lowering changes only invocation implementation.

---

# 27. Error Requirement

Generic display may fail or user display code may raise.

Intrinsic printing must preserve those failures.

The intrinsic may not turn:

```text
display operation raises E
```

into silent output or some separate builtin error category.

Likewise incorrect call shapes must be diagnosed by ordinary Family/call semantics, not by bespoke `print` diagnostics.

---

# 28. Reflection Requirement

Reflection over:

```phalcom
print
```

must see an ordinary Family.

The runtime Family object should not carry `NativeIntrinsicId::Print`.

The intrinsic identity belongs to implementation metadata of the resolved underlying callable, not the first-class object representation.

This distinction is essential:

```text
FamilyObject
    language/runtime value

NativeIntrinsicId::Print
    compiler/runtime implementation provenance
```

---

# 29. Tooling Requirement

Because `print` becomes a genuine prelude value, LSP/source-index tooling should understand it as such:

```text
hover(print)
    → Family callable surface

go-to-definition(print)
    → canonical source `const print = System::#print(_)`

go-to-definition(System.print)
    → canonical System source declaration
```

Do not navigate users to Rust primitive implementation as the language definition.

---

# 30. Compatibility Requirement

`System.print` remains valid.

This is important both for backwards compatibility and as the ordinary fallback path.

No automatic source migration is required.

Recommended documentation direction:

```text
print(x)          idiomatic
System.print(x)   explicit/compatibility surface
```

---

# 31. Scope Boundaries

### In scope

- canonical `print` value;
- value prelude;
- generic `print<T>`;
- `NativeIntrinsicId::Print`;
- semantic intrinsic recognition;
- intrinsic lowering;
- primitive representation specialization;
- shared runtime execution;
- correctness, semantic, lowering and benchmark tests.

### Out of scope

- aliases optimized through arbitrary control flow;
- general monomorphization;
- formatting language;
- variadic print;
- `println`/`eprint`;
- output streams;
- pluggable VM stdout;
- user-defined intrinsics;
- user-extensible intrinsic families;
- intrinsic reflection API;
- broad core-source reorganization.

---

# 32. Requirements Conclusion

The feature is feasible with the current Phalcom architecture and does not require a new callable model.

The implementation decomposes cleanly into four architectural additions:

```text
1. canonical value-prelude binding
2. existing Family object reused unchanged
3. existing NativeIntrinsicId extended with Print
4. existing family semantic-lowering path gains intrinsic execution metadata
```

The largest new concept is not "intrinsic print."

It is:

> **Prelude values should have the same canonical, source-owned identity discipline Phalcom already applies to prelude types.**

That is a useful addition beyond printing itself.

---

# Part II — Detailed Technical Specification

## 33. Status

**Proposed for ratification**

Feature:

> `print` as a canonical intrinsic Family value.

---

# 34. Normative Surface

The canonical source surface shall be equivalent to:

```phalcom
@native
class System is Object {

    @class
    @native
    print<T>(_ value: T) -> Unit

    // ...
}

const print = System::#print(_)
```

`print` shall be implicitly available through the value prelude.

User code:

```phalcom
print("hello")
```

shall be semantically equivalent to invoking the canonical bound family.

---

# 35. Canonical Source Ownership

The canonical `print` binding shall live in the same Universe module that owns the current `System` declaration unless/until a separate source-reorganization change moves `System`.

Currently that is:

```text
phalcom-core/core/universe/src/concurrency/fiber.ph
```

The `print` feature must not opportunistically reorganize `System` into another module.

This keeps language design work separate from source-layout cleanup.

---

# 36. Family Construction

The canonical initializer shall be equivalent to:

```phalcom
System::#print(_)
```

The resulting runtime value is:

```rust
Object::Family(FamilyObject {
    receiver: system_class_value,
    spec: FamilySpec::Exact(print_selector),
})
```

No additional fields are required on `FamilyObject`.

---

# 37. Canonicality

The Universe source module initializes `print` once.

Prelude linkage points directly at that global binding.

Therefore the runtime graph is:

```text
Universe source global slot
          │
          │ contains
          ▼
 canonical FamilyObject
          ▲
          │
prelude BindingRef
```

A consuming module must not create a new Family object merely because it reads `print`.

---

# 38. Value Prelude Model

Introduce:

```rust
#[derive(Clone, Debug, Default)]
pub struct PreludeValueMap {
    entries: BTreeMap<Box<str>, SymbolId>,
}
```

with operations parallel to `PreludeTypeMap`:

```rust
impl PreludeValueMap {
    pub fn canonical_universe() -> Self;
    pub fn shared_canonical_universe() -> Arc<Self>;

    pub fn get(&self, name: &str) -> Option<&SymbolId>;
    pub fn contains_name(&self, name: &str) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = (&str, &SymbolId)>;
}
```

Do not merge this into `PreludeTypeMap`; the latter has a valid invariant that its targets are nominal source `DeclarationId`s.

---

# 39. Prelude Policy Metadata

Add a value-prelude policy structure separate from `UNIVERSE_BINDINGS`, because `UNIVERSE_BINDINGS` currently describes builtin/runtime classes keyed by `UniverseKey`.

Proposed:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UniversePreludeValueSpec {
    pub module_path: &'static [&'static str],
    pub name: &'static str,
}

pub const UNIVERSE_PRELUDE_VALUES: &[UniversePreludeValueSpec] = &[
    UniversePreludeValueSpec {
        module_path: &["concurrency", "fiber"],
        name: "print",
    },
];
```

The constructor for `PreludeValueMap` must validate that every listed target actually exists as a source-owned top-level binding.

Failure of canonical Universe construction due to a stale policy entry is an internal bootstrap/configuration error, not silently ignored policy.

This should be stricter than historical native-prelude fallback behavior.

---

# 40. Runtime Prelude Linkage

`VM::prelude_bindings` already stores generic `BindingRef`s rather than class IDs.

Therefore its representation does not need to change.

The current class-centric synchronization logic should be generalized from:

```text
sync_universe_class_aliases
```

toward something conceptually like:

```text
sync_universe_prelude_bindings
```

with two projections:

```text
class prelude policy
value prelude policy
```

The value entry for `print` must point directly to the owner module's global slot.

No copied runtime value should become the canonical prelude binding.

---

# 41. Native Callable Signature

Update canonical source:

```phalcom
@class @native print<T>(_ value: T) -> Unit
```

Update native metadata:

```rust
#[phalcom_native_macros::primitive(
    System,
    "print(_)",
    params = [T],
    returns = Unit,
    types = "<T>(T) -> Unit",
    side = class,
    intrinsic = Print,
    effects = [io]
)]
```

The native macro's current callable parser already supports generic binder syntax and resolves parameter references from the parsed callable's binders.

Thus no new generic syntax should be invented specifically for this primitive.

---

# 42. Native Intrinsic Identity

Extend:

```rust
pub enum NativeIntrinsicId {
    BoolAnd,
    BoolOr,
    BoolNot,
    Print,
}
```

Every exhaustive mapping in:

```text
phalcom-native-macros
phalcom-native-surface-gen
phalcom-native-surface validation
```

must add `Print`.

Canonical expectation:

```rust
(
    NativeIntrinsicId::Print,
    UniverseKey::System,
    NativeDispatch::Class,
    "print(_)",
    1,
)
```

This prevents attaching the ID accidentally to an unrelated primitive.

---

# 43. Semantic Call Product

Ordinary generic application must produce the usual resolved callable and substitutions.

For:

```phalcom
print(42)
```

conceptually:

```rust
ResolvedApplication {
    callable: system_print_callable,
    substitutions: {
        T => Int
    },
    ...
}
```

No print-specific call-analysis structure is required.

---

# 44. Intrinsic Implementation Provenance

The semantic/compiler pipeline must have a canonical mapping:

```text
CallableId
    →
implementation metadata
    →
Option<NativeIntrinsicId>
```

The repository already carries intrinsic implementation provenance in native-surface and runtime typing metadata, including `RuntimeImplementationRef.intrinsic`.

The implementation must reuse that authority.

If the full semantic snapshot currently does not retain the relevant implementation provenance keyed by `CallableId`, add that projection rather than reconstructing it from source names during code generation.

---

# 45. Executable Lowering Product

Extend the executable lowering model with:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrintSpecialization {
    Generic,
    Int,
    Float,
    Bool,
    String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutableIntrinsic {
    Print(PrintSpecialization),
}
```

Then extend static Family application lowering:

```rust
pub enum FamilyApplicationLoweringSpec {
    Static {
        operation: FamilyOperationShape,
        target: Option<ExecutableInvocationTarget>,
        arity: u8,

        intrinsic: Option<ExecutableIntrinsic>,
    },

    DynamicPack {
        candidates: Box<[ExecutableFamilyCandidate]>,
    },
}
```

This is preferable to a second intrinsic-site map because intrinsic execution is a property of an already-resolved application.

---

# 46. Projection Algorithm

For each statically resolved Family application:

```text
family application
      ↓
resolved ExecutableInvocationTarget
      ↓
CallableId
      ↓
implementation provenance
      ↓
NativeIntrinsicId?
```

If:

```text
NativeIntrinsicId::Print
```

then:

```text
derive safe PrintSpecialization
```

and attach:

```rust
intrinsic: Some(
    ExecutableIntrinsic::Print(...)
)
```

Otherwise:

```rust
intrinsic: None
```

The ordinary target must remain present.

That is useful for:

- debugging;
- lowering assertions;
- fallbacks;
- semantic-equivalence tests.

---

# 47. Specialization Selection

The specialization function should be explicit and conservative:

```rust
fn print_specialization(
    argument_knowledge: ...
) -> PrintSpecialization
```

Its rules are approximately:

```text
proven exact canonical Int representation
    → Int

proven exact canonical Float representation
    → Float

proven exact canonical Bool representation
    → Bool

proven exact canonical String representation
    → String

everything else
    → Generic
```

Do not encode this as a source-name comparison.

Use canonical type/declaration/representation identity.

---

# 48. Generic Type Parameter Case

Inside:

```phalcom
fn show<T>(_ value: T) {
    print(value)
}
```

if `T` is not proven to have a specialized runtime representation:

```text
Print(Generic)
```

is emitted.

There is no requirement to monomorphize `show<Int>` merely to optimize printing.

If future specialization can prove the instantiated body has `T = Int`, it may later rewrite the intrinsic.

---

# 49. Dynamic Case

For:

```phalcom
fn show(_ value: Dynamic) {
    print(value)
}
```

the compiler still knows the callee is canonical `print`, so emit:

```text
Print(Generic)
```

not ordinary Family dispatch.

This already eliminates the majority of call-routing overhead while retaining dynamic display semantics.

---

# 50. Bytecode

For the first implementation, use a print-specific executable bytecode rather than prematurely designing a universal intrinsic ABI.

Recommended:

```rust
Bytecode::Print(PrintSpecialization)
```

or, if the bytecode encoding convention requires compact payloads:

```rust
Bytecode::Print(u8)
```

with a validated enum conversion.

Why not:

```rust
Bytecode::InvokeIntrinsic(...)
```

now?

Because `BoolAnd`/`BoolOr` have very different control-flow semantics, and there is not yet evidence that all intrinsic operations share a useful runtime ABI.

The **identity system** should be generic.

The **bytecode representation** may remain purpose-specific.

That is the smaller and safer architecture.

---

# 51. Compiler Lowering

Current static Family application lowering already has the ideal interception point.

Modify conceptually:

```rust
match spec {
    FamilyApplicationLoweringSpec::Static {
        intrinsic: Some(ExecutableIntrinsic::Print(mode)),
        ..
    } => {
        compile args;
        emit Bytecode::Print(mode);
    }

    FamilyApplicationLoweringSpec::Static { ... } => {
        // existing family path
    }

    ...
}
```

The compiler must not inspect:

```text
callee variable name
source spelling
selector source spelling
```

at this point.

---

# 52. Evaluation Stack Contract

For:

```phalcom
print(expr)
```

the compiler should evaluate:

```text
expr
```

and leave its result in the stack position expected by `Bytecode::Print`.

The intrinsic opcode:

1. pops or consumes one argument according to normal VM stack conventions;
2. emits its display representation plus newline;
3. pushes canonical `Unit`.

Thus the observable result matches:

```phalcom
System.print(expr)
```

---

# 53. Runtime Print Engine

Introduce a focused helper, for example:

```rust
impl VM {
    pub(crate) fn print_value(
        &mut self,
        value: Value,
        mode: PrintSpecialization,
    ) -> PhResult<Value>;
}
```

Better internal factoring:

```rust
fn write_print_value(
    &mut self,
    value: Value,
    mode: PrintSpecialization,
) -> PhResult<()>;

fn print_value(...) -> PhResult<Value> {
    self.write_print_value(...)?;
    write newline;
    Ok(self.unit_value())
}
```

Both native fallback and bytecode execution use this helper.

---

# 54. Generic Runtime Path

Version-one generic path:

```rust
PrintSpecialization::Generic => {
    let text = value.to_display_string(self)?;
    write text;
}
```

This intentionally preserves the existing behavior.

Optimization work should start from a correct common helper.

---

# 55. Int Fast Path

For an exact proven Int representation:

```text
small immediate Int
    → directly format integer to writer

LargeInt backing representation
    → direct bigint formatting to writer
```

Both must produce exactly the same characters as generic display.

Remember that Phalcom's surface `Int` includes arbitrary-precision integer behavior; a large-integer representation must not become a different printed language value.

---

# 56. Float Fast Path

`Float` direct formatting must use the same textual rules as current display.

Do not change:

- exponent notation policy;
- NaN rendering;
- infinity rendering;
- signed zero rendering;

under cover of print optimization.

Golden tests must precede bypassing `to_display_string`.

---

# 57. Bool Fast Path

Use static representations:

```text
true
false
```

provided these exactly match current display behavior.

No heap allocation is needed.

---

# 58. String Fast Path

If current String display semantics are proven to be the underlying string contents unchanged:

```text
String backing storage → output writer
```

may bypass `to_display_string`.

If any escaping/quoting distinction exists between display contexts, keep String on Generic until that behavior is formally settled.

Correctness takes precedence.

---

# 59. Native Fallback

Refactor:

```rust
system_class_print(...)
```

to become essentially:

```rust
pub fn system_class_print(
    vm: &mut VM,
    _receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    vm.print_value(args[0], PrintSpecialization::Generic)
}
```

The generic mode is important.

A call made through an arbitrary ordinary Family should not silently assume a compile-time specialization.

---

# 60. No Intrinsic Metadata in Runtime Family

`FamilyObject` remains:

```rust
pub struct FamilyObject {
    pub receiver: Value,
    pub spec: FamilySpec,
}
```

unchanged.

This guarantees:

```text
print as value
```

remains completely ordinary.

---

# 61. Formal Laws

I recommend recording these as the normative laws for the feature.

### `INTR-01 — Ordinary-Semantics Law`

An intrinsic callable is semantically defined by its ordinary language callable surface. Intrinsic execution does not constitute a second invocation model.

### `INTR-02 — Identity Law`

Intrinsic lowering requires canonical resolved callable identity and cannot be inferred from source spelling alone.

### `INTR-03 — Equivalence Law`

An intrinsic execution path and the ordinary fallback invocation of the same callable must be observationally equivalent.

### `INTR-04 — Fallback Completeness Law`

Failure to prove intrinsic eligibility changes optimization only; it never makes a valid ordinary invocation invalid.

### `INTR-05 — Specialization Non-Identity Law`

Implementation specialization does not create a new selector, `CallableId`, Family identity, or language-level overload.

### `INTR-06 — First-Class Law`

Taking an intrinsically optimizable operation as a first-class callable value does not require the runtime value to encode or preserve its intrinsic optimization status.

### `PRINT-01 — Family Law`

The canonical `print` binding denotes an ordinary exact Family bound to the canonical `System` class receiver and the `print(_)` selector.

### `PRINT-02 — Prelude Law`

`print` is a canonical Universe value-prelude binding and is not a reserved keyword.

### `PRINT-03 — Generic Law`

The canonical callable signature is:

```text
<T>(T) -> Unit
```

### `PRINT-04 — Shadowing Law`

Lexical resolution may shadow the prelude `print` binding exactly as other prelude bindings may be shadowed.

### `PRINT-05 — Display Law`

`print(value)` emits the same value display representation as canonical `System.print(value)`, followed by one newline.

### `PRINT-06 — Return Law`

Successful printing returns canonical `Unit`.

### `PRINT-07 — Evaluation Law`

The argument expression is evaluated exactly once according to ordinary call evaluation order.

### `PRINT-08 — I/O Effect Law`

Printing is externally observable I/O and cannot be removed or reordered unless ordinary effect equivalence has been proven.

### `PRINT-09 — Generic-Fallback Law`

Any printable value for which no specialized representation is proven executes through the generic display path.

### `PRINT-10 — Specialized-Representation Law`

A primitive fast path is legal only where its byte output and failure behavior are equivalent to the generic display path for every value admitted by the proof.

### `PRINT-11 — Compatibility Law`

`System.print(value)` remains a valid ordinary invocation of the canonical underlying callable.

### `PRINT-12 — Canonical-Value Law`

All unshadowed prelude reads of `print` in one VM refer to the canonical Universe-owned Family value.

---

# 62. Acceptance Criteria

The feature is complete only when all of these hold:

```phalcom
print(42)
```

works without explicit import.

```phalcom
let p = print
p(42)
```

works.

```phalcom
fn call(_ f) {
    f(42)
}
call(print)
```

works.

```phalcom
fn run(print) {
    print(42)
}
```

uses the parameter, not builtin print.

Direct canonical:

```phalcom
print(42)
```

lowers to the print intrinsic.

A dynamic argument:

```phalcom
let x: Dynamic = 42
print(x)
```

lowers to generic intrinsic print.

An ordinary indirect Family whose identity is not proven does not receive unsafe intrinsic lowering.

`print(42)` and `System.print(42)` produce byte-identical output.

Reflection observes a `Family`.

No parser changes are necessary.

No new heap object kind is necessary.

---

# Part III — Patch-Grade Implementation Plan

The implementation should be staged so semantic correctness lands before the optimization.

---

## Task 1 — Lock Existing `System.print` Semantics

### Files

**Test / inspect:**

```text
phalcom-core/src/primitive/system.rs
phalcom-core/tests/...
```

Add a focused test package if no dedicated System/printing test file exists.

### Tests first

Establish golden output for:

```phalcom
System.print(0)
System.print(-1)
System.print(42)
System.print(3.5)
System.print(true)
System.print(false)
System.print("")
System.print("hello")
System.print(())
System.print([1, 2])
```

Also test:

```phalcom
let result = System.print("x")
result == ()
```

where appropriate.

### Purpose

These tests become the semantic oracle for all later intrinsic paths.

### Verification

```bash
cargo test -p phalcom-core print
```

Expected:

```text
all existing/fresh System.print behavior tests pass
```

### Commit

```text
test(runtime): lock System.print display semantics
```

---

# Task 2 — Make `System.print` Generically Typed

### Modify

```text
phalcom-core/core/universe/src/concurrency/fiber.ph
phalcom-core/src/primitive/system.rs
```

Change source declaration:

```phalcom
@class @native print(_ value: Object) -> Unit
```

to:

```phalcom
@class @native print<T>(_ value: T) -> Unit
```

Change primitive metadata to:

```rust
params = [T],
returns = Unit,
types = "<T>(T) -> Unit",
side = class,
effects = [io]
```

Do not yet add intrinsic execution.

### Tests

Add semantic tests proving:

```phalcom
System.print(42)
```

solves:

```text
T = Int
```

and:

```phalcom
System.print("hello")
```

solves:

```text
T = String
```

and:

```phalcom
let x: Dynamic = ...
System.print(x)
```

solves:

```text
T = Dynamic
```

Verify return type:

```text
Unit
```

### Native metadata verification

Run generator validation:

```bash
cargo run -p phalcom-native-surface-gen -- --check
```

or the repository's canonical generator/check invocation.

If generated output is intentionally changed:

```bash
cargo run -p phalcom-native-surface-gen -- ...
```

then rerun `--check`.

### Commit

```text
feat(typing): make System.print generic
```

---

# Task 3 — Register `NativeIntrinsicId::Print`

### Modify

```text
phalcom-native-meta/src/primitive.rs
phalcom-native-macros/src/lib.rs
phalcom-native-surface-gen/src/main.rs
phalcom-native-surface/src/lib.rs
phalcom-core/src/primitive/system.rs
```

Generated:

```text
phalcom-native-surface/src/generated.rs
```

or whichever generated catalog file is produced by the existing generator.

### Patch

Add:

```rust
Print
```

to:

```rust
NativeIntrinsicId
```

Add parser/generator arms:

```rust
Some("Print") => ...
```

Annotate System primitive:

```rust
intrinsic = Print
```

Add expected canonical validation row:

```rust
(
    NativeIntrinsicId::Print,
    UniverseKey::System,
    NativeDispatch::Class,
    "print(_)",
    1
)
```

### Tests

Reject attempts to bind `Print` intrinsic to:

- wrong owner;
- wrong dispatch side;
- wrong selector;
- wrong arity.

Assert canonical System row exposes:

```rust
Some(NativeIntrinsicId::Print)
```

### Verification

```bash
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-surface
cargo test -p phalcom-native-macros
cargo test -p phalcom-native-surface-gen
```

### Commit

```text
feat(native): register Print intrinsic identity
```

---

# Task 4 — Add the Canonical `print` Family Value

### Modify

```text
phalcom-core/core/universe/src/concurrency/fiber.ph
```

Add after the canonical `System` definition, at a source location compatible with initialization order:

```phalcom
const print = System::#print(_)
```

### Tests

At the source/compiler level prove that the initializer:

- resolves `System`;
- resolves exact Family selector `print(_)`;
- has Family/callable semantics;
- does not invoke the method during initialization.

At runtime prove:

```phalcom
let p = print
p("hello")
```

works when the binding is reached explicitly from its owning module before prelude work is added.

### Critical assertion

The initializer creates:

```text
Object::Family
```

not a bound `Method`, Closure, or new callable representation.

### Commit

```text
feat(universe): define canonical print family value
```

---

# Task 5 — Introduce Explicit Universe Value-Prelude Policy

### Modify

```text
phalcom-native-meta/src/universe.rs
```

Add:

```rust
pub struct UniversePreludeValueSpec {
    pub module_path: &'static [&'static str],
    pub name: &'static str,
}

pub const UNIVERSE_PRELUDE_VALUES: &[UniversePreludeValueSpec] = ...
```

Initial catalog:

```rust
[
    ("concurrency/fiber", "print")
]
```

Do not add `print` to `UniverseKey`.

`UniverseKey` currently describes canonical built-in class identities. Mixing arbitrary values into it would weaken its current invariant.

### Tests

Validate:

- no duplicate prelude value names;
- module path is valid;
- name non-empty;
- `print` appears exactly once.

### Commit

```text
feat(universe): add canonical value-prelude policy
```

---

# Task 6 — Build `PreludeValueMap`

### Create/modify

```text
phalcom-semantic/src/prelude.rs
phalcom-semantic/src/lib.rs
phalcom-semantic/tests/semantic/integration/prelude.rs
```

### Add

```rust
PreludeValueMap
```

whose values are canonical:

```rust
phalcom_modules::SymbolId
```

Do not change the existing `PreludeTypeMap` target model.

### Canonical construction

For every `UNIVERSE_PRELUDE_VALUES` row:

1. resolve canonical Universe `ModuleId`;
2. confirm the source module exists;
3. confirm a top-level source binding with that name exists;
4. create canonical `SymbolId`;
5. reject duplicate name policy.

### Test

```rust
assert_eq!(
    prelude_values.get("print"),
    Some(&SymbolId {
        module: canonical_fiber_module,
        name: "print"
    })
)
```

Also assert:

```text
PreludeTypeMap does not contain print
PreludeValueMap does contain print
```

This prevents accidental type/value namespace conflation.

### Commit

```text
feat(semantic): add canonical value prelude
```

---

# Task 7 — Link Value Prelude Bindings at Runtime

### Modify

```text
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/modules/linkage.rs
```

The existing:

```rust
prelude_bindings: HashMap<Symbol, BindingRef>
```

already has the correct generic runtime representation.

Refactor the class-centric synchronization function so value-prelude rows are added after their owning Universe modules are materialized.

For `print`:

```text
lookup Universe concurrency/fiber module
        ↓
find global slot "print"
        ↓
create BindingRef { module, slot }
        ↓
prelude_bindings["print"] = BindingRef
```

### Do not

```text
copy Family value into every user module
```

or:

```text
construct Family value on demand
```

### Tests

Across two modules:

```phalcom
// module A
let a = print

// module B
let b = print
```

prove both runtime reads ultimately originate from the same canonical Universe slot/value.

### Commit

```text
feat(vm): link Universe value prelude bindings
```

---

# Task 8 — Resolve Bare `print` as a Normal Semantic Binding

### Modify

Likely focused integration points:

```text
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/source_index/*
phalcom-semantic/src/editor.rs
```

depending on where outer/module bindings are currently injected.

Current variable expression checking already enters through ordinary binding lookup.

### Requirement

Seed the canonical value-prelude bindings into the module's outer semantic environment.

Then:

```phalcom
print
```

must resolve through exactly the same occurrence/binding machinery as any imported/module value.

### Shadowing tests

```phalcom
fn run(print) {
    print("x")
}
```

must resolve `print` to the parameter.

```phalcom
let print = custom
print("x")
```

must resolve according to ordinary lexical rules.

### Negative architecture test

There should be no checker branch of the form:

```rust
if name == "print"
```

### Commit

```text
feat(semantic): resolve print through value prelude
```

---

# Task 9 — Verify Ordinary Family Application Before Optimizing

### Tests

In the semantic family test package, add:

```phalcom
print(42)
```

and assert the call resolves as a Family application with:

```text
target callable = System.class::print(_)
T = Int
return = Unit
```

Add:

```phalcom
let p = print
p(42)
```

and prove it also behaves as an ordinary Family invocation.

This task is a hard correctness gate.

Do not add intrinsic bytecode until this test passes through the normal route.

### Commit

```text
test(semantic): prove print ordinary family semantics
```

---

# Task 10 — Preserve Intrinsic Provenance by `CallableId`

### Modify

Depending on the exact existing implementation-provenance ownership after SC-4.8:

```text
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/core_surface/*
phalcom-core/src/typing/side_table.rs
phalcom-core/src/modules/semantic_lowering.rs
```

The goal is one canonical query:

```rust
fn intrinsic_for_callable(
    callable: &CallableId
) -> Option<NativeIntrinsicId>
```

or an equivalent immutable map available during lowering.

### Non-negotiable rule

The query resolves from canonical implementation provenance.

It must not reconstruct a `PrimitiveKey` by stringifying a `CallableId` during backend compilation.

### Tests

```text
System.print CallableId → Some(Print)
unrelated "print" method → None
user method named print → None
Bool.and → BoolAnd
```

The final Bool assertion proves the mechanism generalizes existing intrinsic metadata instead of building print-only identity plumbing.

### Commit

```text
feat(semantic): retain callable intrinsic provenance
```

---

# Task 11 — Project Intrinsic Family Application Metadata

### Modify

```text
phalcom-core/src/modules/semantic_lowering.rs
```

Add:

```rust
PrintSpecialization
ExecutableIntrinsic
```

and:

```rust
intrinsic: Option<ExecutableIntrinsic>
```

to static Family application lowering.

### Selection

For a resolved static target:

```text
target.callable
    ↓
intrinsic_for_callable
```

If `Print`, derive specialization.

### Tests

Assertions over `ModuleLoweringSemantics`:

```phalcom
print(42)
```

→

```text
Static {
    target = System.print,
    intrinsic = Print(Int)
}
```

```phalcom
let x: Dynamic = ...
print(x)
```

→

```text
intrinsic = Print(Generic)
```

shadowed `print`:

```phalcom
fn run(print) {
    print(42)
}
```

→

```text
no Print intrinsic
```

### Commit

```text
feat(lowering): project print intrinsic applications
```

---

# Task 12 — Add the Print Bytecode

### Modify

```text
phalcom-core/src/bytecode.rs
```

Add:

```rust
Bytecode::Print(PrintSpecialization)
```

Update every exhaustive bytecode match:

- disassembly;
- tracing;
- verifier if present;
- bytecode size/accounting;
- debug formatting;
- VM dispatch.

### Test

Construct/disassemble all specialization variants and verify their encoding/debug representation.

### Commit

```text
feat(bytecode): add specialized print instruction
```

---

# Task 13 — Compile Proven Print Applications to the Intrinsic

### Modify

```text
phalcom-core/src/compiler/lib/associated.rs
```

Current static Family application lowering is the primary target.

Before emitting:

```text
InvokeAssociatedFamilyStatic
```

inspect the already-projected intrinsic field.

If:

```rust
Some(ExecutableIntrinsic::Print(mode))
```

compile the one argument and emit:

```rust
Bytecode::Print(mode)
```

### Compiler tests

Prove emitted bytecode:

```phalcom
print(42)
```

contains:

```text
Print(Int)
```

and not:

```text
MakeFamily
InvokeAssociatedFamilyStatic
Invoke
```

where those instructions would only exist for this call.

For:

```phalcom
let p = print
p(42)
```

version one may still produce ordinary Family invocation.

That is acceptable and expected.

### Commit

```text
feat(compiler): lower canonical print calls intrinsically
```

---

# Task 14 — Refactor the Runtime Print Engine

### Modify/create

```text
phalcom-core/src/primitive/system.rs
phalcom-core/src/vm/...
```

Prefer a focused file if the helper grows beyond a few lines, for example:

```text
phalcom-core/src/vm/output.rs
```

Do not dump specialized formatting logic into `vm/dispatch.rs`.

### Step 1

Move current generic semantics behind:

```rust
VM::print_value(...)
```

with:

```rust
PrintSpecialization::Generic
```

still calling:

```rust
Value::to_display_string(...)
```

### Step 2

Change native:

```rust
system_class_print
```

to delegate to the shared helper.

### Tests

Re-run Task 1 goldens before implementing any fast paths.

There should be zero output changes.

### Commit

```text
refactor(runtime): centralize print execution
```

---

# Task 15 — Execute `Bytecode::Print`

### Modify

VM bytecode dispatch:

```text
phalcom-core/src/vm/dispatch.rs
```

or the current opcode execution module.

Implement:

```text
pop/read argument
call shared runtime print engine
push Unit
continue
```

### Tests

Execute:

```phalcom
print("hello")
```

through actual VM bytecode.

Assert stdout equals:

```text
hello\n
```

and result is `Unit`.

### Equivalence tests

For representative values:

```text
print(v)
System.print(v)
```

must yield identical output.

### Commit

```text
feat(vm): execute print intrinsic bytecode
```

---

# Task 16 — Add Bool and String Fast Paths

Start with the lowest-risk representations.

### Bool

Avoid allocating display string:

```rust
true  → write "true"
false → write "false"
```

### String

Only after a golden test proves direct backing-string output exactly equals `to_display_string`.

### Tests first

For String include:

- empty string;
- ASCII;
- Unicode;
- newline-containing strings;
- quotes;
- backslashes.

No escaping behavior may change accidentally.

### Commit

```text
perf(runtime): specialize bool and string printing
```

---

# Task 17 — Add Int Fast Path

### Modify

Runtime print helper and, if needed, focused numeric rendering utilities.

Cover both:

- immediate integer representation;
- large arbitrary-precision Int representation.

### Tests

Boundary cases:

```text
0
1
-1
i64 min/max
first promoted LargeInt
negative LargeInt
very large positive/negative integers
```

For every case compare:

```text
specialized intrinsic bytes
==
generic display bytes
```

### Performance requirement

Do not materialize a heap Phalcom `String` merely to emit the number.

### Commit

```text
perf(runtime): specialize Int printing
```

---

# Task 18 — Add Float Fast Path

### Tests before implementation

Cover:

- `0.0`;
- `-0.0`;
- small values;
- large values;
- fractional values;
- infinities where constructible;
- NaN forms according to current language display contract.

The specialized implementation must match generic display byte-for-byte.

### Commit

```text
perf(runtime): specialize Float printing
```

---

# Task 19 — Add Semantic Indistinguishability Tests

Create a dedicated test family covering all laws.

Suggested organization:

```text
phalcom-semantic/tests/semantic/intrinsics/
    mod.rs
    print.rs

phalcom-core/tests/intrinsics/
    print.rs
```

### Semantic tests

1. bare prelude resolution;
2. canonical binding identity;
3. generic `T` solving;
4. return `Unit`;
5. shadowing;
6. first-class Family typing;
7. indirect application;
8. no spelling-based intrinsic recognition.

### Runtime tests

1. direct intrinsic;
2. `System.print`;
3. Family alias;
4. Family passed as parameter;
5. Dynamic argument;
6. primitive specializations;
7. generic object;
8. object display failure propagation;
9. exact output ordering.

### Key adversarial test

Create an unrelated method/family named `print`:

```phalcom
class Logger {
    print<T>(_ value: T) -> Unit {
        ...
    }
}
```

and prove it never gets `NativeIntrinsicId::Print` behavior.

### Commit

```text
test(intrinsics): verify print semantic equivalence
```

---

# Task 20 — Add Reflection and Editor Tests

### Modify/test

Likely:

```text
phalcom-semantic/src/editor.rs
phalcom-semantic/src/source_index/*
phalcom-lsp/... tests
```

### Verify

`print` appears in completion as a value/callable.

Hover exposes the Family/callable signature.

Go-to-definition reaches:

```phalcom
const print = System::#print(_)
```

not Rust.

Underlying `System.print` navigation continues to reach the canonical `.ph` declaration.

### Commit

```text
feat(tooling): expose print prelude family
```

---

# Task 21 — Add Performance Benchmarks

Benchmarks should separate the sources of improvement.

Suggested cases:

```text
A. System.print(Int)
B. print(Int) generic intrinsic
C. print(Int) specialized intrinsic
D. ordinary Family alias print(Int)

A. System.print(String)
B. print(String) generic intrinsic
C. print(String) specialized intrinsic
```

Because terminal I/O dominates wall-clock timing, direct terminal benchmarks are poor measurements.

Use either:

- a null/captured output backend if an existing testing seam permits it; or
- benchmark the formatting/runtime instruction machinery separately from physical write latency.

Measure at minimum:

```text
dispatch overhead
allocations per call
bytes formatted
instructions/op
```

The critical target is not "printing is 100× faster"—actual I/O will dominate.

The meaningful targets are:

```text
direct intrinsic:
    no Family dispatch

specialized primitive:
    no intermediate Phalcom String allocation
```

### Commit

```text
bench(runtime): add print intrinsic benchmarks
```

---

# Task 22 — Documentation

### Update

```text
docs/spec/callables/family.md
docs/spec/universe.md
```

and add a focused language/core document if appropriate:

```text
docs/spec/core/print.md
```

### Document

- `print` is a Family;
- generic signature;
- prelude visibility;
- shadowing;
- `System.print` equivalence;
- intrinsic implementation is non-semantic;
- formatting options are not currently part of the API.

### Also document general intrinsic laws

The reusable `INTR-xx` laws belong in a generic compiler/language document, not solely `print.md`.

### Commit

```text
docs(spec): define intrinsic print family semantics
```

---

# Task 23 — Final Verification Matrix

Run targeted packages first:

```bash
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-surface
cargo test -p phalcom-native-macros
cargo test -p phalcom-native-surface-gen

cargo test -p phalcom-semantic prelude
cargo test -p phalcom-semantic print
cargo test -p phalcom-semantic family

cargo test -p phalcom-core print
cargo test -p phalcom-core family
cargo test -p phalcom-core lowering
```

Then full workspace:

```bash
cargo test --workspace
```

Then formatting/lints using repository-standard commands:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
```

Then generated native-surface consistency:

```bash
cargo run -p phalcom-native-surface-gen -- --check
```

Finally run representative Phalcom programs:

```phalcom
print("hello")
print(42)
print(3.14)
print(true)

let p = print
p("alias")

fn consume(_ f) {
    f("first class")
}
consume(print)

fn shadow(print) {
    print("not builtin")
}
```

---

# 63. Implementation Dependency Graph

The work should proceed in this order:

```text
existing behavior tests
        │
        ▼
generic System.print
        │
        ├───────────────┐
        ▼               ▼
Print intrinsic ID   canonical print Family
        │               │
        │               ▼
        │          value prelude
        │               │
        └───────┬───────┘
                ▼
       ordinary semantic Family call
                │
                ▼
       intrinsic provenance
                │
                ▼
       lowering projection
                │
                ▼
         Print bytecode
                │
                ▼
      shared runtime engine
                │
        ┌───────┴────────┐
        ▼                ▼
   generic path     primitive paths
        │                │
        └───────┬────────┘
                ▼
        equivalence tests
                │
                ▼
          benchmarks/tooling
```

The hard gate is:

> **Do not implement intrinsic lowering until bare `print(...)` already works correctly as an ordinary first-class Family call.**

That guarantees the optimization remains removable without changing the language.

---

# 64. Recommended Patch Series

I would structure the eventual commits approximately as:

```text
01 test(runtime): lock System.print display semantics
02 feat(typing): make System.print generic
03 feat(native): register Print intrinsic identity
04 feat(universe): define canonical print family value
05 feat(universe): add canonical value-prelude policy
06 feat(semantic): add canonical value prelude
07 feat(vm): link Universe value prelude bindings
08 feat(semantic): resolve print through value prelude
09 test(semantic): prove print ordinary family semantics
10 feat(semantic): retain callable intrinsic provenance
11 feat(lowering): project print intrinsic applications
12 feat(bytecode): add specialized print instruction
13 feat(compiler): lower canonical print calls intrinsically
14 refactor(runtime): centralize print execution
15 feat(vm): execute print intrinsic bytecode
16 perf(runtime): specialize bool and string printing
17 perf(runtime): specialize Int printing
18 perf(runtime): specialize Float printing
19 test(intrinsics): verify print semantic equivalence
20 feat(tooling): expose print prelude family
21 bench(runtime): add print intrinsic benchmarks
22 docs(spec): define intrinsic print family semantics
```

Each commit should leave the tree internally valid. In particular, the initial addition of the `print` Family must work before bytecode optimization lands.

---

# 65. Final Architecture

The completed architecture is:

```text
SOURCE
────────────────────────────────────────────────────

const print = System::#print(_)

print(42)


LANGUAGE SEMANTICS
────────────────────────────────────────────────────

value-prelude resolution
        │
        ▼
canonical Universe value binding
        │
        ▼
ordinary Family value
        │
        ▼
Family application
        │
        ▼
System.class::print<T>(T) -> Unit
        │
        ▼
generic application
        │
        └── T = Int


IMPLEMENTATION PROVENANCE
────────────────────────────────────────────────────

CallableId(System.print)
        │
        ▼
Native implementation record
        │
        ▼
NativeIntrinsicId::Print


EXECUTABLE LOWERING
────────────────────────────────────────────────────

callee identity proven
argument representation proven
        │
        ▼
ExecutableIntrinsic::Print(Int)
        │
        ▼
Bytecode::Print(Int)


RUNTIME
────────────────────────────────────────────────────

Print(Int)
        │
        ▼
shared runtime print engine
        │
        ▼
direct integer formatting
        │
        ▼
stdout
        │
        ▼
Unit
```

while the completely ordinary first-class path remains:

```text
let p = print
p(42)
      │
      ▼
FamilyObject
      │
      ▼
ordinary Family call routing
      │
      ▼
System.print
      │
      ▼
native primitive
      │
      ▼
same shared runtime print engine
```

That is the property worth protecting.

`print` does not become a compiler pseudo-function that happens to masquerade as a Family. It is genuinely a Family. The compiler merely proves, at selected call sites, that it can execute the same operation more directly.

The resulting principle is reusable well beyond printing:

> **Phalcom intrinsic callables should be ordinary source-defined language objects with canonical callable identities and ordinary fallback implementations. Optimization privileges attach to resolved implementation identity after semantic resolution; they never redefine the object or the call semantics.**

For `print`, that gives Phalcom the desirable surface:

```phalcom
print(value)
```

with essentially the best execution path the compiler can prove, without sacrificing the language's existing Family model to achieve it.