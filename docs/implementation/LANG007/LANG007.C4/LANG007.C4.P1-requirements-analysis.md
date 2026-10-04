# LANG007.C4.P1 — Binding, Mutability, Field Lifecycle, and Corpus Cutover
## Implementation Requirements Analysis

**Project:** Telos / `aureat/phalcom-lang`  
**Checkpoint:** LANG007.C4 — Structural / Static Upgrade  
**Plan item:** C4.P1  
**Repository baseline:** `main` @ `8c3a98aab41ed8631e9f8746a6b29984cf77581d`  
**Analysis date:** 2026-10-04  
**Status:** Implementation requirements analysis; supersedes the P1 portions of the earlier whole-C4 analysis where later decisions changed O1/O2/O3 or where the earlier audit assumed `mut` was not already parsed contextually.

---

# 1. Executive conclusion

C4.P1 is not a cosmetic keyword migration. It is a coordinated semantic cutover across five existing authorities:

1. **surface binding syntax and AST**;
2. **semantic binding flow state**;
3. **runtime local/global/upvalue storage**;
4. **field declaration/layout and constructor lifecycle**;
5. **module/source/tooling metadata and corpus migration**.

The repository is unusually well positioned for this work because several difficult prerequisites already exist:

- `FlowState` is already the sole owner of path-local binding facts;
- branch joins and loop fixed-point/widening already exist;
- fields already use an explicit `Uninitialized / MaybeInitialized / DefinitelyInitialized` lifecycle;
- `FieldSemanticSignature` already publishes field mutability and includes it in its semantic fingerprint;
- constructor analyses already publish normal-exit field flow;
- `mut` already exists as a **contextual word** in the parser for fields, trait properties, and delegation;
- runtime locals, upvalues, globals, and fields already have identifiable read boundaries.

The main architectural work is therefore **integration, semantic correction, and migration**, not creation of a new flow-analysis framework.

The most important repository findings that materially alter the implementation from the earlier whole-C4 draft are:

- **Do not introduce a reserved `Token::Mut` by default.** `mut` is already a contextual word. Extend that mechanism to `let mut`.
- **Current initializer-less `let` is not actually backed by a raw private sentinel in its compiled local slot.** The compiler emits `Bytecode::Nil`, and the VM defines that opcode to push the guest-visible `None` value. C4.P1 needs a real uninitialized-binding representation/lowering path.
- **The semantic field signature already contains `mutable: bool`.** Field-default inversion does not require a new metadata concept.
- **The compiler still infers fields from assignments when explicit declarations are absent.** O3 requires deleting that authority and migrating affected classes to explicit declarations.
- **Constructor lifecycle proof exists but is weaker than O2.** It proves initialization state across constructor exits, but semantic field assignment does not yet enforce immutable write-once rules.
- **Constructor-exit proof alone is not sufficient for sound field invariants if `self` can be used before construction completes.** This is the principal correctness edge that P1 must close or explicitly retain a runtime backstop for.

Recommended implementation direction:

> Preserve surface syntax in `BindingKind::{Let, LetMut, Const}`, normalize immediately into separate **phase** and **mutability** semantics, add initialization as an orthogonal flow-state dimension, make semantic analysis publish read-safety facts, and have lowering consume those facts to choose checked versus unchecked runtime reads.

P1 should leave C4.P2 a clean seam:

- `const` is recognized as a **static-phase declaration**;
- it is never silently lowered as an ordinary runtime immutable binding;
- the existing corpus's runtime `const` declarations are migrated to immutable `let`;
- actual general static evaluation/materialization of `const` belongs to P2.

---

# 2. Frozen C4 decisions governing P1

## 2.1 O1 — runtime bindings and initialization

Canonical surface:

```telos
let x = expr          // runtime immutable; initializer required

let mut x = expr      // runtime mutable; initialized
let mut x             // runtime mutable; initially uninitialized
let mut x: T          // runtime mutable; initially uninitialized

let x                 // error

const x = expr        // static-phase constant; initializer required
const x               // error
```

Canonical initialization semantics:

> **Uninitialized is a storage state, not a Telos value.**

Therefore:

```telos
let mut x
print(x)
```

does **not** mean that `x == None`. It is a runtime invalid read unless static control flow proves the read unreachable.

`None` remains an ordinary Telos value:

```telos
let mut x: Int | None = None
```

is initialized and semantically different from an uninitialized binding.

P1 also implements local definite-initialization proof:

```text
Uninitialized
MaybeInitialized
DefinitelyInitialized
```

Read policy:

```text
DefinitelyInitialized
    -> statically proven safe
    -> lowering may use unchecked read

Uninitialized
    -> compile-time diagnostic for a guaranteed invalid read

MaybeInitialized
    -> program remains legal
    -> lowering emits/retains runtime checked read

Unprovable across closure/module boundaries
    -> program remains legal
    -> runtime checked read
```

This is **not** global strict definite-assignment. Runtime trapping remains the language backstop.

## 2.2 O2 — immutable fields

Choice A is frozen:

> **Constructor-lifecycle initialization.**

For instance fields:

- fields are immutable by default;
- an immutable field with a declaration default is already initialized;
- an immutable field without a default may be initialized during the canonical construction lifecycle;
- immutable field writes are not ordinary mutation;
- after successful construction, no later write is legal;
- constructor control-flow must prove required immutable fields initialized on every normal construction exit.

## 2.3 O3 — explicit fields only

Choice A is frozen:

> Every stored field must be explicitly declared. Assignment never creates storage.

Consequences:

- implicit instance field inference from `_field = ...` is removed;
- implicit class-side/static field inference is removed;
- unknown field reads/writes are diagnostics;
- reflection, layout, metadata, and tooling all observe one explicit field surface.

## 2.4 Field surface

Canonical field direction:

```telos
class Counter {
    _name: String          // immutable
    mut _count: Int        // mutable
}
```

Legacy current forms are migrated:

```text
current bare mutable field     -> new `mut _field`
current `const _field`         -> new bare `_field`
```

---

# 3. Scope and non-scope

## 3.1 P1 must implement

P1 owns:

- `let` runtime immutability;
- `let mut` runtime mutability;
- initializer-less `let mut`;
- local definite-initialization flow;
- runtime trapping for uninitialized mutable binding reads;
- checked-read elimination where initialization is proven;
- static-phase classification of `const`;
- removal of runtime lowering semantics from `const`;
- field immutable-by-default syntax;
- contextual `mut` field syntax as canonical mutable field form;
- immutable instance-field constructor lifecycle enforcement;
- explicit stored fields only;
- module/source-index correction for the new binding taxonomy;
- compiler/VM mutability correction;
- corpus migration;
- LSP/REPL/TextMate/docs/test migration required to keep the repository coherent.

## 3.2 P1 prepares but does not complete

P1 establishes seams for:

- general static evaluation of `const` — P2;
- cross-module serialized static values — P8;
- generalized `FieldKey` — P4;
- Product/Sum reflection — P5;
- static control flow — P6.

P1 must not fake any of these by preserving legacy runtime `const` semantics.

## 3.3 Explicit non-goals

P1 does **not** need:

- full interprocedural definite-assignment through arbitrary higher-order calls;
- whole-program proof of global initialization;
- a new type-relation engine;
- a second field-flow system;
- a new keyword token solely because `let mut` is introduced;
- dynamic runtime write-once checks for every immutable field if semantic/runtime construction boundaries can enforce the invariant more narrowly;
- P2's static evaluator.

---

# 4. Repository baseline and affected architecture

The audited HEAD is:

```text
8c3a98aab41ed8631e9f8746a6b29984cf77581d
```

Primary implementation paths:

```text
phalcom-ast/src/
    ast.rs
    parser.rs
    token.rs
    lexer.rs

phalcom-semantic/src/
    checker/
        analysis.rs
        binding.rs
        context.rs
        statement.rs
        expression.rs
        flow/state.rs
        field_lifecycle.rs
        declaration.rs
        declaration_signature.rs
        body.rs
        control.rs
    signature.rs
    diagnostic.rs
    source_index/
        scope.rs
        builder.rs
        occurrence.rs
        reference.rs
    db/
        fingerprint.rs
        query.rs
        product.rs
    metadata/export.rs
    presentation.rs

phalcom-modules/src/
    interface.rs
    fingerprint.rs
    linker.rs / linked interface consumers

phalcom-core/src/
    compiler/lib/
        mod.rs
        scope.rs
        expr.rs
        class_decl.rs
        product_opt.rs
        patterns.rs
        error.rs
    compiler/attributes.rs
    bytecode.rs
    value/
        repr.rs
        mod.rs
        nil.rs
    vm/
        dispatch.rs
        send.rs
    heap/
        module.rs
        instance.rs
        upvalue.rs

phalcom-lsp/src/
    hover.rs
    semantic_tokens.rs
    completion.rs
    ...

phalcom-repl/src/
    highlighter.rs
    completer.rs
    ...

tools/vsphalcom/syntaxes/
    phalcom.tmLanguage.json
```

P1 also affects a broad source corpus and tests because the current spelling has the opposite mutability default.

---

# 5. Critical repository findings

## 5.1 `mut` already exists contextually

There is no dedicated `Token::Mut`, but `parser.rs` already recognizes contextual `mut` for:

- class fields;
- trait properties;
- delegated members.

The parser has reusable contextual-word helpers and a `starts_mut_member()` guard.

### Requirement

**P1-R001** — Extend the existing contextual-word mechanism to parse:

```telos
let mut x = ...
```

Do not reserve `mut` globally unless a separate language decision explicitly requires that.

### Why

This:

- preserves identifiers named `mut` where currently legal;
- avoids unnecessary lexer/token churn;
- matches already shipped field/property/delegation syntax;
- minimizes syntax-highlighting and parser recovery regressions.

The AST should still preserve `LetMut` explicitly; contextual lexing does not imply semantic ambiguity.

---

## 5.2 Binding AST currently conflates spelling and semantics

Current:

```rust
BindingKind::Let   // mutable
BindingKind::Const // runtime immutable
```

C4 target:

```rust
enum BindingKind {
    Let,
    LetMut,
    Const,
}
```

But downstream code should not repeatedly encode meaning with ad hoc matches.

### Requirement

**P1-R002** — Define canonical semantic projection helpers:

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

Conceptually:

```text
Let     -> Runtime + Immutable
LetMut  -> Runtime + Mutable
Const   -> Static
```

Every semantic/compiler consumer should use normalized meaning rather than assuming:

```rust
kind == Let
```

means mutable.

This is necessary because the repository currently contains many direct assumptions such as:

```text
matches!(binding.kind, BindingKind::Let)
```

in semantic checking, compiler lowering, source indexing, and product optimization.

---

## 5.3 Initializer-less current `let` does not preserve true uninitialized state

The current compiler comments describe initializer-less `let` as sentinel-backed, but the actual lowering emits `Bytecode::Nil`.

The VM's `Bytecode::Nil` handler pushes guest-visible `None`.

Therefore current source:

```telos
let x
```

is effectively initialized to `None` as a runtime value.

### Requirement

**P1-R003** — Introduce a real runtime representation for **uninitialized user binding storage**.

P1 must not reuse surface `None`.

Preferred designs, in order:

1. **dedicated internal `ValueTag::UninitializedBinding`**;
2. an equivalent private sentinel distinct from generic allocator `Nil`;
3. a separate initialization bit/slot metadata representation if value-tag policy rejects a new tag.

Do **not** globally reinterpret current `Value::Nil` without auditing its many internal uses. `Nil` currently backs:

- allocator-unwritten object slots;
- scratch locals;
- module slots;
- other internal completion/default paths.

A binding-specific state is safer.

### Required lowering seam

Initializer-less:

```telos
let mut x
```

must seed the runtime slot with internal uninitialized-binding state, not `Bytecode::Nil`.

This likely warrants one of:

```text
Bytecode::UninitializedBinding
```

or direct local/global declaration seeding that never exposes the sentinel as an expression value.

It must be impossible for the sentinel to become an ordinary Telos value, enter `Some`, participate in `==`, be printed, or cross FFI/native boundaries.

---

# 6. Binding syntax and parsing requirements

## 6.1 Grammar

Target:

```text
runtime_binding :=
    "let" ["mut"] pattern [":" type] "=" expr
  | "let" "mut" name [":" type]

static_binding :=
    "const" pattern [":" static_domain_or_type] "=" expr
```

For P1, the existing ordinary type annotation AST remains usable; P2 broadens static result domains.

### Requirements

**P1-R004** — `let` without `mut` requires an initializer.

**P1-R005** — `let mut` may omit its initializer for a simple name binding.

**P1-R006** — destructuring without an initializer remains invalid regardless of mutability because there is no source product to destructure.

Examples:

```telos
let mut x              // valid
let mut x: Int         // valid
let mut (a, b)         // invalid
let (a, b) = pair      // valid immutable leaves
let mut (a, b) = pair  // valid mutable leaves
```

**P1-R007** — `const` always requires an initializer.

**P1-R008** — parsing should preserve precise `mut` source range if diagnostics/semantic tokens need it. If `LetBinding` does not need the range semantically, source indexing may derive it from tokens.

## 6.2 Parser recovery

Add targeted errors for:

```telos
let mut mut x
let x
const x
let mut (a, b)
```

Avoid allowing malformed `let mut` to fall through to an identifier named `mut` after the parser has already committed to binding syntax.

---

# 7. Semantic binding model

## 7.1 Add initialization as an orthogonal dimension

Current `BindingState` tracks:

- contract;
- current type knowledge;
- denotation;
- consistency;
- invalidity;
- mutability;
- version.

It does not track initialization independently.

### Requirement

**P1-R009** — Add:

```rust
initialization: InitializationState
```

to `BindingState`.

Prefer generalizing the existing field lifecycle enum:

```rust
enum InitializationState {
    Uninitialized,
    MaybeInitialized,
    DefinitelyInitialized,
}
```

rather than creating a second lattice with identical semantics.

If renaming `FieldInitialization` is too disruptive in the first patch, the same algebra may be temporarily reused, but the end state should not maintain two subtly divergent initialization definitions.

## 7.2 Declaration seeds

Canonical seeds:

| Declaration | Mutability | Initialization | Current knowledge |
|---|---|---|---|
| `let x = e` | immutable | definite | RHS knowledge |
| `let mut x = e` | mutable | definite | RHS knowledge |
| `let mut x` | mutable | uninitialized | missing-initializer/unknown |
| `let mut x: T` | mutable | uninitialized | missing-initializer/unknown, persistent contract `T` |
| parameter | immutable unless separately specified | definite | parameter knowledge |
| pattern binder with source value | according to construct | definite | extracted value knowledge |

An unannotated initializer-less mutable binding may have no persistent contract until later flow writes establish current knowledge. P1 does not need to invent an implicit `None` type.

## 7.3 Assignment transfer

**P1-R010** — A successful assignment to a mutable runtime binding performs:

```text
current = RHS knowledge
initialization = DefinitelyInitialized
version += 1
dependent facts invalidated
```

The existing `FlowState::write` is the correct mutation seam.

The assignment must not mark the binding initialized if the assignment itself is causally invalid.

## 7.4 Branch join

**P1-R011** — Binding initialization joins identically to the existing field lattice:

```text
Definite + Definite       -> Definite
Uninitialized + Uninit    -> Uninitialized
anything mixed            -> MaybeInitialized
```

Unreachable predecessors are excluded exactly as current `FlowState::join_impl()` already excludes them.

Initialization must be joined separately from `TypeKnowledge`; `Unknown(MissingInitializer)` is not a substitute for initialization state.

## 7.5 Loops

`FlowState::widen_loop_state_with_hierarchy()` already performs fixed-point/widening for mutable binding facts.

**P1-R012** — Initialization state must participate in loop convergence.

Examples:

```telos
let mut x: Int
while cond {
    x = 1
}
use(x)
```

post-loop:

```text
MaybeInitialized
```

unless control-flow structure proves the loop executes and exits only after assignment.

`break`, `continue`, `return`, and `throw` already have path-sensitive flow machinery and must preserve initialization facts through their existing exit channels.

## 7.6 Read classification

Semantic analysis should publish a read-safety classification, conceptually:

```rust
enum BindingReadSafety {
    ProvenInitialized,
    DefinitelyUninitialized,
    RuntimeCheckRequired,
}
```

### Requirement

**P1-R013** — Do not make bytecode lowering re-run flow analysis.

The semantic layer owns the proof. Lowering consumes a frozen semantic result keyed by source expression/binding identity.

This may live in:

- `CallableAnalysis`;
- a binding-read resolution index;
- expression analysis metadata;
- an equivalent stable semantic product.

---

# 8. Diagnostics for local initialization

Recommended canonical diagnostics:

```text
binding.immutable_requires_initializer
binding.const_requires_initializer
binding.read_before_initialization
binding.may_be_uninitialized          // optional warning/explanation, not compile error
binding.assign_to_immutable
```

## 8.1 Guaranteed invalid read

```telos
let mut x: Int
print(x)
```

Semantic state is definitely uninitialized at the read.

**P1-R014** — Emit a compile-time error.

The program should not be lowered merely to guarantee a runtime trap that the compiler has already proved unavoidable.

## 8.2 Maybe-initialized read

```telos
let mut x: Int

if cond {
    x = 1
}

print(x)
```

**P1-R015** — Do not reject solely because initialization is uncertain.

Emit a runtime-checked read.

Tooling may expose the state as explanatory information:

```text
x: Int
possibly uninitialized on this path
```

but `MaybeInitialized` is compiler epistemology, not a Telos value.

---

# 9. Runtime checked-read design

## 9.1 Ordinary proven local reads must remain fast

A fundamental performance requirement:

> Adding initializer-less `let mut` must not add a conditional branch to every ordinary local read.

### Requirement

**P1-R016** — Preserve an unchecked `GetLocal` path for proven-initialized reads.

Use a distinct checked operation where required, conceptually:

```text
GetLocal(slot)            // proof says safe
GetLocalChecked(slot)     // traps on UninitializedBinding
```

Equivalent fused instructions must preserve the distinction.

The current VM has fused forms such as `InvokeLocal`. A checked local used immediately as a receiver cannot be optimized into an unchecked fused operation.

Either add a checked fused form or suppress that fusion.

## 9.2 Upvalues

Current `GetUpvalue` surfaces absence.

### Requirement

**P1-R017** — Captured initializer-less bindings retain the uninitialized state in the shared upvalue cell.

A closure may therefore observe:

```telos
let mut x: Int
let f = { print(x) }

f()       // runtime error
x = 10
f()       // 10
```

P1 is not required to prove initialization through arbitrary closure calls.

Reads whose safety is not proven use checked upvalue semantics.

## 9.3 Globals

Current module slots can be unread/uninitialized and `GetGlobal` surfaces absence.

### Requirement

**P1-R018** — Runtime mutable global bindings may use checked reads where initialization is not statically guaranteed.

P1 is not required to perform whole-module/cross-module definite-assignment proof.

Do not confuse:

- an undeclared/undefined global;
- a declared but uninitialized mutable global.

They need distinct runtime errors.

## 9.4 Runtime error

Add an explicit error category, e.g.:

```text
UninitializedBindingError
```

It should be distinct from:

```text
UndefinedVariable
```

and should preserve enough source/debug context to identify the binding where feasible.

---

# 10. `const` phase boundary

The current repository uses `const` for runtime immutable bindings. C4 changes its meaning fundamentally.

## 10.1 Corpus first principle

All existing runtime-immutable `const` declarations must migrate to:

```telos
let ...
```

before `const` acquires static semantics.

This avoids trying to preserve two meanings under one spelling.

## 10.2 P1 semantic requirement

**P1-R019** — `const` must be represented as **static phase**, not `mutable=false`.

It must not be lowered through:

```text
compile_expr(initializer)
DefineGlobal / add_local
runtime module initializer
```

as an ordinary binding.

P1 may gate executable use of new static constants until P2, but it must fail closed with a targeted diagnostic rather than executing the initializer at runtime.

Recommended temporary P1 seam:

```text
parse `const`
    ↓
publish Static binding phase
    ↓
require initializer
    ↓
do not runtime-lower
    ↓
P2 supplies StaticValue evaluation/materialization
```

## 10.3 Generated compiler bindings

Search all compiler-generated `LetBinding` values, especially `compiler/attributes.rs`.

Do not mechanically change every synthetic `BindingKind::Let` to `LetMut`.

Classify each generated binding by whether it is reassigned.

Compiler temporaries that are single-assignment should become immutable `Let`.

Only synthetic bindings that are actually written after declaration should become `LetMut`.

---

# 11. Source index requirements

Current source categories encode only `Let` versus `Const`, while `SourceBindingInfo` separately has `mutable`.

P1 must prevent stale source tooling from claiming immutable `let` is mutable.

### Requirements

**P1-R020** — Extend source binding taxonomy to represent `LetMut`.

A minimal source-preserving approach:

```text
TopLevelLet
TopLevelLetMut
TopLevelConst
LocalLet
LocalLetMut
LocalConst
```

`SourceBindingInfo.mutable` remains the normalized mutability fact.

**P1-R021** — Reconcile existing source-index mutability mismatches for implicit binders.

At audited HEAD:

- semantic callable/block parameters are created immutable;
- source-index callable parameters are currently marked mutable;
- `for` binders/index bindings are currently marked mutable.

P1 should make source-index metadata reflect canonical semantic mutability rather than historical parser/compiler behavior.

If Telos intends any implicit binder to be mutable, that should be explicit policy, not an accidental source-index divergence.

---

# 12. Module interface correction

Current `DeclarationSurface` contains:

```text
is_const: bool
```

and the same bit is also used for declarations such as classes/data/traits/type aliases where it effectively means “not mutable”, not “source `const`”.

That abstraction becomes invalid once `const` means static phase.

### Requirement

**P1-R022** — Replace ambiguous `is_const` with an explicit module binding category.

Recommended shape:

```rust
enum ModuleBindingSemantics {
    Runtime { mutable: bool },
    StaticConstant,
    Declaration,
}
```

or an equivalent normalized representation.

This distinguishes:

```text
let x = ...
let mut y = ...
const z = ...
class C ...
data D ...
type T = ...
```

without overloading one boolean.

### Export/linking requirement

**P1-R023** — A static constant must never be represented as if it were an ordinary runtime global slot.

If the linked interface cannot yet carry static products until P2/P8, P1 must preserve the category and gate unsupported export/materialization rather than fabricating runtime linkage.

Module interface/fingerprint tests must prove that changing:

```text
let -> let mut
```

changes the relevant semantic interface identity even when the inferred type remains the same.

---

# 13. Field syntax cutover

## 13.1 Current state

HEAD already parses contextual:

```telos
mut _field
```

but current canonical semantics are still:

```telos
_field        // mutable
const _field  // immutable
```

## 13.2 Target

```telos
_field        // immutable
mut _field    // mutable
```

### Requirements

**P1-R024** — Refactor `parse_field_decl` to accept positive `mutable` semantics rather than an inverted `is_const` argument.

This avoids carrying the old default into new code.

**P1-R025** — bare `FieldDef` sets:

```text
mutable = false
```

**P1-R026** — contextual `mut` sets:

```text
mutable = true
```

**P1-R027** — legacy `const _field` should receive a targeted migration diagnostic during the cutover:

```text
fields are immutable by default; remove `const`
```

rather than being silently interpreted under old semantics.

`let _field` remains invalid.

## 13.3 Field semantic metadata

`FieldSemanticSignature` already contains:

```text
mutable: bool
```

and field signature fingerprints already hash this fact.

Therefore no new semantic field-mutability carrier is needed.

The parser/default inversion automatically flows through:

```text
FieldDef.mutable
    ↓
FieldSemanticSignature.mutable
    ↓
metadata FieldMutabilityRef
```

provided all constructors/builders continue to consume the AST value rather than reconstructing defaults.

---

# 14. Explicit fields only

## 14.1 Current compiler behavior

`compiler/lib/class_decl.rs` currently infers stored fields from assignments when explicit declarations are absent, and scans class-side bodies for static fields.

This must be removed as storage authority.

### Requirement

**P1-R028** — `ClassLayout` must derive stored fields solely from explicit `ClassMember::Field` declarations.

This applies to:

- instance fields;
- class-side fields.

Body assignments may still be inspected for diagnostics, but may not allocate slots.

## 14.2 Unknown field accesses

**P1-R029** — Add canonical semantic diagnostics for undeclared field reads/writes.

Suggested codes:

```text
field.undeclared_read
field.undeclared_write
```

The compiler must fail closed if semantic validation is bypassed.

An assignment:

```telos
_name = value
```

must never create `_name`.

## 14.3 Layout order

The explicit declaration order becomes the only source of own-class field order.

Superclass prefix behavior remains unchanged.

This reduces layout ambiguity and eliminates whole-body assignment scans from normal layout construction.

---

# 15. Corpus migration for explicit fields

Removing implicit fields requires an AST-aware codemod, not text substitution.

For every class whose runtime layout currently depends on assignment inference:

1. reproduce the current compiler's inferred field set and order;
2. insert explicit declarations;
3. mark them mutable, because legacy inferred fields were mutable;
4. preserve side:
   - instance -> `mut _field`;
   - class-side -> canonical `@class mut _field` form;
5. do not add a declaration initializer merely because the first historical assignment appears in a constructor/method;
6. preserve comments/attributes/source formatting where possible.

The migration must run **before** deleting inference so the old compiler can still serve as an oracle for parity tests.

### Requirement

**P1-R030** — Build a migration audit that compares old and new class layouts for the migrated corpus:

```text
field names
field side
slot order
field count
superclass offset prefix
```

Any mismatch is a migration failure unless intentionally reviewed.

---

# 16. Immutable instance-field lifecycle

## 16.1 Existing foundation

The repository already has:

```text
FieldInitialization:
    Uninitialized
    MaybeInitialized
    DefinitelyInitialized
```

`default_field_seeds()`:

- default initializer -> definite;
- no default -> uninitialized.

`finalize_instance_field_lifecycle()`:

- joins normal constructor exits.

This is the correct base for O2.

## 16.2 Missing enforcement

Current semantic assignment marks a field definitely initialized after a valid write, but does not enforce immutable write-once semantics.

### Requirement

**P1-R031** — An immutable instance field assignment is legal only when:

1. the current callable is in the canonical initialization lifecycle for that instance; and
2. the field's current initialization state is **definitely `Uninitialized`**.

If state is:

```text
DefinitelyInitialized
```

the write is a reinitialization error.

If state is:

```text
MaybeInitialized
```

the write is also rejected, because it overwrites the value on some reachable predecessor.

This rule is stronger and clearer than “constructor methods may write const fields”.

## 16.3 Declaration defaults

An immutable field with:

```telos
_id: Int = 5
```

is already definitely initialized at constructor entry.

Any constructor assignment to `_id` is therefore illegal.

## 16.4 Constructor exits

**P1-R032** — Every normal successful construction path for a concrete instance must establish all required immutable instance fields as definitely initialized.

A field that remains:

```text
Uninitialized
MaybeInitialized
```

on any successful construction exit is a compile-time construction error.

Throwing/non-returning exits do not create successfully constructed objects and do not contribute as successful exits.

## 16.5 Mutable fields

Mutable fields are not subject to write-once enforcement.

Their type/flow validation continues through existing field assignment reconciliation.

---

# 17. Constructor identity and lifecycle authority

Do not key P1 field legality on method spelling such as `new`.

Use canonical constructor identity already published by parser/attributes/semantic callable products.

Required audit targets:

```text
MethodDef.is_constructor
CallableSemanticKind::Constructor or current equivalent
compiler FunctionState.is_constructor
attribute lowering in compiler/attributes.rs
derived/expanded constructors
super-constructor calls
```

### Requirement

**P1-R033** — semantic analysis is the authority for whether a write occurs in an initialization lifecycle.

The compiler's existing `in_constructor` flag becomes a lowering assertion/backstop, not the sole correctness rule.

---

# 18. Constructor-time `self` use: correctness blocker

This is the most important P1 edge not solved by simply joining constructor exits.

Consider:

```telos
class C {
    _x: Int

    @constructor
    new() {
        observeSelf()   // reads _x
        _x = 1
    }

    observeSelf() {
        print(_x)
    }
}
```

The class-level lifecycle may eventually prove `_x` definitely initialized on every normal constructor exit.

An ordinary method body may therefore be analyzed with the finalized invariant:

```text
_x : Int, definitely initialized
```

But that method can execute **during construction before the write**.

The current VM uses raw uninitialized slots and currently surfaces them as `None`, so this can violate the static field contract.

### P1 must not ignore this.

There are three sound strategies:

### Strategy A — restrict construction-phase escape/calls

Statically forbid use of `self` that can reach ordinary behavior until all required immutable fields are initialized.

This requires substantially more escape/effect reasoning.

### Strategy B — object construction-state tracking

Carry a runtime “under construction” state and guard field reads / escape boundaries.

This adds runtime object-state machinery.

### Strategy C — checked uninitialized immutable-field reads

Retain a runtime check on immutable field reads that can observe an uninitialized slot; trap rather than surface `None`.

This is the smallest P1-compatible backstop.

## Recommendation

**P1-R034 — Use Strategy C for P1 unless the implementation deliberately adds Strategy A.**

This keeps field type soundness without requiring full constructor escape analysis.

It also means the compiler must not remove every immutable-field initialization check merely because class-level constructor exits prove eventual initialization.

A later optimization may eliminate checks where it can prove the receiver is post-construction.

This is separate from O1's binding rule, but it is required to make O2's stronger field invariant sound under current object/constructor behavior.

---

# 19. Class-side immutable fields

Instance constructor lifecycle does not naturally initialize class-side storage.

Recommended P1 rule:

```telos
@class _limit: Int = 10    // valid immutable class-side field
@class _limit: Int         // invalid unless a canonical class-init lifecycle exists
@class mut _cache: Cache   // mutable class-side storage
```

### Requirement

**P1-R035** — In the absence of a canonical class-initialization lifecycle, an immutable class-side field must have a declaration initializer.

Do not pretend an instance constructor is the lifecycle for class-side storage.

If Telos introduces explicit class initialization later, this rule can be generalized without changing field syntax.

---

# 20. Field runtime/layout changes

Current `ClassLayout` tracks legacy `const_fields` / related mutability information for compiler checks.

### Requirements

**P1-R036** — Rename/reframe runtime/compiler layout metadata around **immutable fields**, not the retired `const` spelling.

Prefer slot-oriented data for read checks:

```text
immutable instance slots
immutable class-side slots
```

rather than only a name set if the VM read opcode operates by slot.

**P1-R037** — Unknown field slot accesses must remain hard failures; explicit-field migration must not introduce a fallback allocation path.

**P1-R038** — If P1 uses checked immutable-field reads, provide an opcode/lowering form that checks the slot sentinel without adding a name lookup.

---

# 21. Binding mutability in compiler lowering

Current compiler paths repeatedly derive:

```text
mutable = BindingKind::Let
```

This must be eliminated.

### Requirement

**P1-R039** — Centralize binding kind queries:

```text
is_runtime()
is_static()
runtime_mutability()
requires_initializer()
```

Update at least:

```text
compiler/lib/mod.rs
compiler/lib/product_opt.rs
compiler/lib/patterns.rs
compiler/lib/scope.rs
compiler/lib/expr.rs
compiler/attributes.rs
```

Product optimization consequence:

- immutable `let` becomes eligible for optimizations currently reserved for old `const`;
- `let mut` remains mutation-sensitive;
- static `const` is not a runtime virtual-product candidate in P1.

---

# 22. Assignment diagnostics

Current messages still teach:

```text
immutable `const`; use `let`
```

C4 target:

```text
immutable `let`; use `let mut`
```

### Requirement

**P1-R040** — Update semantic and compiler diagnostics consistently.

Suggested messages:

```text
cannot assign to immutable binding `x`; declare it with `let mut` if reassignment is required

immutable `let` binding `x` requires an initializer

static `const` binding `x` requires an initializer

field `_x` is immutable after initialization

immutable field `_x` is already initialized

immutable field `_x` is not definitely initialized on every successful constructor exit
```

---

# 23. Module/global behavior and REPL

Current global binding tables store a mutability boolean and REPL has special rebinding behavior.

P1 must audit:

```text
Compiler::global_bindings
ModuleObject global binding metadata
SetGlobal legality
REPL cell redeclaration/rebinding policy
linked import writes
```

### Requirements

**P1-R041** — `let` globals are runtime immutable.

**P1-R042** — `let mut` globals are runtime mutable.

**P1-R043** — `const` does not allocate an ordinary mutable/immutable runtime global slot as its semantic representation.

**P1-R044** — REPL special rebinding policy must be updated by **phase and mutability**, not by old `let`/`const` spelling.

Do not accidentally let the REPL turn a static constant into an ordinary mutable global merely for interactive convenience.

---

# 24. Incremental and fingerprint implications

## 24.1 Binding flow

If `FlowBindingSummary` participates in callable result fingerprints, initialization state must be included.

### Requirement

**P1-R045** — A change from:

```text
DefinitelyInitialized
```

to:

```text
MaybeInitialized
```

must invalidate downstream semantic consumers even if the current type is unchanged.

Update:

```text
FlowBindingSummary
callable/body semantic fingerprinting
flow fixpoint keys if present
incremental tests
```

## 24.2 Fields

Field mutability is already included in canonical `FieldSemanticSignature` fingerprints.

This is a positive repository property: the parser-default flip naturally invalidates semantic field dependents.

## 24.3 Module interfaces

Replacing `is_const` changes interface fingerprints.

Tests must prove:

- immutable runtime `let`;
- mutable runtime `let mut`;
- static `const`;

produce distinct interface semantics where appropriate.

---

# 25. Tooling requirements

## 25.1 LSP hover

Current hover documentation says `let` is mutable and missing initialization reads `None`.

Update keyword/contextual documentation:

```text
let       runtime immutable binding
let mut   runtime mutable binding
const     compile-time/static constant
mut       contextual mutability modifier
```

For initializer-less mutable locals, hover should display initialization state when useful:

```text
x: Int
mutable; possibly uninitialized here
```

Do not manufacture `Int | None`.

## 25.2 Semantic tokens

Because `mut` remains contextual, lexer token kind alone cannot classify it as a keyword.

The LSP already has contextual-word handling elsewhere.

### Requirement

**P1-R046** — syntax-aware semantic token logic must classify `mut` in:

- `let mut`;
- mutable fields;
- trait mutable properties;
- mutable delegations;

without classifying arbitrary identifier `mut` as a keyword.

## 25.3 REPL

Update highlighting/completion/help.

The REPL must understand that initializer-less `let mut` can parse as a complete declaration even though it contains no `=`.

## 25.4 TextMate

Update fallback grammar to recognize canonical forms, especially:

```text
let mut
mut _field
```

while accepting that semantic tokens are the authoritative contextual classifier.

---

# 26. Corpus cutover

This is a hard cutover, not a compatibility period.

## 26.1 Binding rewrite

Current meaning -> C4 meaning:

```text
current `let x = e`     -> `let mut x = e`
current `let x`         -> `let mut x`

current `const x = e`   -> `let x = e`
```

This is a context-sensitive **swap**. It cannot be safely implemented as two independent textual replacement passes.

## 26.2 Field rewrite

```text
current `_x`            -> `mut _x`
current `const _x`      -> `_x`
current `mut _x`        -> `mut _x`
```

## 26.3 Implicit fields

Insert explicit `mut` declarations using an AST/layout-aware migration as described in §15.

## 26.4 Corpus scale observed at audited revision

GitHub code search at the audited revision reports:

- 338 `.ph` files containing `"let "`;
- 621 `.ph` files containing `"const "`;
- 52 `.ph` files containing `"const _"`;
- 0 `.ph` files containing the exact `"let mut "` sequence;
- 0 `.ph` files containing the exact `"mut _"` sequence under that search form.

These are **file-match counts, not declaration counts**. They establish that the cutover is repository-wide and that new canonical spellings are effectively unused in the `.ph` corpus at this baseline.

### Requirement

**P1-R047** — The codemod must be parser/AST-aware and idempotent.

Run twice -> no further changes.

---

# 27. Codemod validation

The migration should produce three independent parity reports.

## 27.1 Binding parity

Before/after normalized facts:

```text
runtime/static phase
mutability
initializer presence
annotation
pattern shape
```

Legacy runtime `const` must map to runtime immutable `let`, not static `const`.

## 27.2 Field layout parity

For migrated existing programs:

```text
class identity
instance field slot order
class-side field slot order
field count
mutability
```

must remain equivalent except where O3 intentionally rejects previously accidental/ambiguous implicit behavior.

## 27.3 Runtime behavior parity

Programs not relying on old uninitialized-reads-as-`None` should preserve behavior.

Programs that do rely on it are intentional semantic breaks and should get explicit migration notes/tests.

---

# 28. Test requirements

## 28.1 Parser / AST

Required cases:

```text
let x = 1                    -> Let
let mut x = 1                -> LetMut
let mut x                    -> LetMut, no initializer
let mut x: Int               -> LetMut, no initializer
let x                        -> diagnostic
const x = 1                  -> Const
const x                      -> diagnostic

let (a, b) = p               -> immutable leaves
let mut (a, b) = p           -> mutable leaves
let mut (a, b)               -> diagnostic

_field: Int                  -> immutable
mut _field: Int              -> mutable
const _field                 -> migration diagnostic
```

Also contextual-word regressions proving `mut` can still be an identifier where grammar permits.

## 28.2 Binding flow unit tests

Test state algebra directly:

```text
U ⊔ U = U
D ⊔ D = D
U ⊔ D = M
M ⊔ D = M
```

where:

```text
U = Uninitialized
M = MaybeInitialized
D = DefinitelyInitialized
```

Test unreachable predecessor exclusion.

Test assignment transition.

Test loop widening/fixed-point convergence.

## 28.3 Semantic source tests

Examples:

```telos
let mut x: Int
x = 1
use(x)
```

-> safe.

```telos
let mut x: Int
use(x)
```

-> compile-time guaranteed-uninitialized diagnostic.

```telos
let mut x: Int
if cond { x = 1 }
use(x)
```

-> no static rejection; runtime check required.

```telos
let x = 1
x = 2
```

-> immutable assignment diagnostic.

## 28.4 Runtime tests

Prove:

- uninitialized binding sentinel is not `None`;
- uninitialized local checked read traps;
- assignment then checked read succeeds;
- captured uninitialized upvalue traps;
- captured value after assignment succeeds;
- uninitialized global checked read traps;
- explicit `None` never traps;
- private sentinel cannot be wrapped in `Some`, printed, hashed, compared, or returned as a normal value.

## 28.5 Optimization tests

Disassemble/inspect bytecode:

```telos
let mut x = 1
print(x)
```

must use unchecked proven read.

Maybe-initialized read must use checked form.

Ensure `InvokeLocal` or other fusion never erases a required initialization check.

## 28.6 Field tests

- bare field immutable;
- `mut` field mutable;
- unknown field assignment rejected;
- implicit field is not added to layout;
- immutable default + constructor rewrite rejected;
- immutable constructor first write accepted;
- immutable constructor second write rejected;
- branch that initializes on only one path rejected at successful constructor exit;
- every normal path initializes -> accepted;
- throwing-only path does not force initialization;
- mutable field may be reassigned;
- class-side immutable field without required initialization rejected per P1 class-side rule.

## 28.7 Constructor early-use regression

Mandatory regression for §18:

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

The implementation must not allow `_x`'s uninitialized storage to surface as an `Int`/`None` value contrary to the field contract.

## 28.8 Module/interface tests

Inspect interface products:

```text
let      -> Runtime immutable
let mut  -> Runtime mutable
const    -> Static
class    -> Declaration
```

No `is_const` ambiguity.

## 28.9 Tooling tests

- hover docs;
- semantic token for contextual `mut`;
- rename/references preserve binding identity;
- completion does not suggest obsolete field `const`;
- REPL highlighter recognizes `let mut`;
- TextMate fallback updated.

---

# 29. Performance requirements

P1 must be effectively neutral on ordinary runtime programs.

## 29.1 Local reads

**Hard gate:** a proven-initialized local load must not gain a runtime initialization branch.

Baseline hot path should remain equivalent to indexed stack load + push.

## 29.2 Flow analysis

Initialization adds one tiny enum per binding.

Expected complexity:

```text
straight-line transfer     O(1) per write/read
branch join                O(bindings), existing pass
loop join                  existing fixed-point + constant-size lattice
```

No new CFG engine is justified.

## 29.3 Field layout

Removing implicit field discovery should reduce compiler class-body scanning work.

Explicit declarations allow direct deterministic layout construction.

## 29.4 Runtime representation

A dedicated uninitialized-binding value tag is a zero-allocation immediate state if chosen.

Do not allocate an object per uninitialized binding.

## 29.5 Benchmarks

At minimum rerun:

```text
phalcom-core VM benchmark suite
compiler/parser benchmark suite if available
semantic incremental/flow benchmarks
```

Add microbenchmarks for:

```text
proven GetLocal
checked GetLocal
ordinary immutable field read if P1 adds a check
```

Performance acceptance:

- no measurable regression on ordinary proven local access outside noise budget;
- checked-read overhead confined to semantically uncertain initialization sites;
- no new heap allocation for binding initialization tracking.

---

# 30. Implementation staging

Recommended patch order:

## Stage P1.1 — semantic taxonomy and parser surface

- add `BindingKind::LetMut`;
- parse contextual `let mut`;
- flip `let` meaning;
- flip field default;
- retire field `const`;
- add normalized binding phase/mutability helpers;
- update parser tests.

Keep workspace compiling with exhaustive matches.

## Stage P1.2 — module/source/tooling taxonomy

- source index `LetMut`;
- replace module `is_const`;
- field/source mutability parity;
- interface fingerprints;
- basic LSP contextual tokens/hover.

This prevents downstream code from rebuilding old assumptions while runtime changes land.

## Stage P1.3 — binding initialization flow

- shared `InitializationState`;
- `BindingState.initialization`;
- declaration seeds;
- assignment transition;
- joins;
- loop widening/fixpoints;
- read-safety semantic product;
- diagnostics;
- fingerprint updates.

## Stage P1.4 — runtime uninitialized binding representation

- dedicated internal representation;
- checked local/upvalue/global reads;
- compiler lowering consumes semantic read safety;
- bytecode fusion audit;
- runtime error.

## Stage P1.5 — explicit field authority

- insert/migrate explicit field declarations first;
- layout parity test;
- remove assignment-derived field allocation;
- undeclared-field diagnostics.

## Stage P1.6 — immutable field lifecycle enforcement

- semantic write-once checks;
- constructor-exit obligations;
- class-side immutable rule;
- early-`self` soundness backstop;
- compiler assertions/layout metadata cleanup.

## Stage P1.7 — full corpus cutover and deletion

- AST-aware binding/field codemod;
- remove compatibility branches;
- update docs/goldens/examples;
- repository-wide test/bench certification.

This ordering minimizes periods where source spelling and semantic meaning disagree.

---

# 31. File-by-file change inventory

## `phalcom-ast`

### `ast.rs`

- add `BindingKind::LetMut`;
- update docs;
- keep `FieldDef.mutable`, but document immutable default;
- remove legacy comments equating uninitialized with `None`.

### `parser.rs`

- contextual `mut` after `let`;
- immutable `let` initializer requirement;
- field default inversion;
- retire `const _field`;
- keep contextual `mut` non-reserved.

### `token.rs` / `lexer.rs`

- likely **no `Token::Mut` change**;
- update keyword comments/tests only if needed.

## `phalcom-semantic`

### `checker/analysis.rs`

- binding initialization state;
- flow summary initialization state;
- read-safety product if owned here.

### `checker/flow/state.rs`

- shared initialization lattice;
- binding joins;
- binding writes;
- loop widening;
- fixpoint key/fingerprint effects.

### `checker/statement.rs`

- new binding-kind semantics;
- initializer rules;
- declaration seed initialization.

### `checker/expression.rs`

- read-before-init classification;
- assignment transition;
- immutable field write rules;
- undeclared field diagnostics.

### `checker/field_lifecycle.rs`

- retain existing lifecycle engine;
- strengthen immutable semantics;
- constructor exit validation.

### `checker/body.rs`

- verify constructor lifecycle seeding and final products.

### `signature.rs`

- field mutability already sufficient;
- no new concept expected.

### `diagnostic.rs`

- new/renamed binding and field diagnostics.

### `source_index/*`

- `LetMut` source kind;
- normalized mutability;
- implicit binder mutability cleanup.

### `db/fingerprint.rs`

- initialization state in callable flow products;
- module/interface kind changes.

### `metadata/export.rs`

- field mutability is already exported;
- update any binding/module representation only if P1 changes exported interface metadata.

## `phalcom-modules`

### `interface.rs`

- remove ambiguous `is_const`;
- introduce explicit binding semantics/category;
- ensure static constant is not treated as runtime slot.

### fingerprints/linker

- preserve category;
- reject/gate unsupported static materialization until P2 rather than lying.

## `phalcom-core`

### `compiler/lib/mod.rs`

- `Let`/`LetMut`/`Const` split;
- no runtime static `Const` lowering;
- real uninitialized-binding seed;
- consume semantic read-safety products.

### `compiler/lib/scope.rs`

- comments and mutability source;
- global/local binding tables.

### `compiler/lib/expr.rs`

- immutable `let` assignment errors;
- checked/unchecked reads;
- field write assertions.

### `compiler/lib/class_decl.rs`

- explicit fields only;
- delete layout inference from assignments;
- immutable-slot metadata.

### `compiler/lib/product_opt.rs`

- mutable only for `LetMut`;
- immutable `let` optimization eligibility.

### `compiler/attributes.rs`

- generated binding audit;
- constructor identity/lifecycle integration.

### `bytecode.rs`

Likely new uninitialized seed/check operations or checked load forms.

Audit all local-load fusion.

### `value/repr.rs`

If using recommended representation:

- add private uninitialized-binding tag;
- equality/hash/display/Some wrapping must reject or keep unreachable from user semantics.

### `vm/dispatch.rs`

- checked local/global/upvalue reads;
- do not use generic `surface_absence` for binding-uninitialized state.

### `vm/send.rs`

- module/linked read paths must not convert uninitialized bindings to `None`.

## Tooling

### `phalcom-lsp`

- hover;
- contextual semantic tokens;
- completion/docs.

### `phalcom-repl`

- highlighter;
- parser-completeness expectations;
- completion/help.

### `tools/vsphalcom`

- TextMate fallback grammar.

---

# 32. Correctness hazards and required guards

## H1 — `const` accidentally remains runtime immutable

Guard:

```text
no `Const` path reaches ordinary runtime DefineGlobal/add_local lowering
```

until P2 materialization explicitly supplies a runtime value.

## H2 — initialization encoded only as unknown type

Guard:

```text
InitializationState exists independently of TypeKnowledge
```

because a binding can be definitely initialized with unknown/dynamic type, or uninitialized with a known declared contract.

## H3 — checked read optimized away

Guard:

- bytecode/disassembly tests;
- fusion pass tests;
- semantic-lowering handoff test.

## H4 — generic `Nil` semantics changed globally

Guard:

- separate binding sentinel or equivalent;
- tests for scratch locals, bare return, Option, object slots.

## H5 — source index and semantic checker disagree on mutability

Guard:

- parity tests over source binding IDs and semantic binding states.

## H6 — field assignment still creates layout

Guard:

- class layout constructed before/independently of body assignment scan;
- undeclared write test.

## H7 — immutable field write allowed twice inside constructor

Guard:

```text
write allowed only from DefinitelyUninitialized
```

not merely `is_constructor`.

## H8 — constructor branch joins allow overwrite

If state is `MaybeInitialized`, another immutable write is rejected.

## H9 — constructor-time method call observes uninitialized field

Guard with §18 runtime backstop or stronger escape/call rule.

## H10 — corpus swap implemented as textual two-pass rename

Guard with AST-driven codemod and semantic parity report.

## H11 — class-side immutable field has no initialization lifecycle

Guard with declaration-initializer requirement unless/until explicit class initialization exists.

---

# 33. Acceptance criteria

C4.P1 is complete only when all of the following are true.

## Surface

- [ ] `let` is runtime immutable.
- [ ] `let mut` is runtime mutable.
- [ ] `const` is static-phase.
- [ ] `let`/`const` require initializers.
- [ ] `let mut` may omit initializer.
- [ ] fields are immutable by default.
- [ ] `mut _field` is the canonical mutable field form.
- [ ] legacy field `const` is removed from canonical syntax.

## Binding semantics

- [ ] uninitialized binding is not represented as `None`.
- [ ] initialization has an explicit semantic state.
- [ ] branch and loop flow are sound.
- [ ] definitely uninitialized read is diagnosed.
- [ ] maybe-initialized read remains legal and checked.
- [ ] proven initialized read has no runtime check.
- [ ] upvalue/global fallback trapping is correct.

## Fields

- [ ] assignment cannot declare storage.
- [ ] class layouts derive only from declarations.
- [ ] immutable field first initialization is lifecycle-controlled.
- [ ] immutable reinitialization is rejected.
- [ ] every successful construction path initializes required immutable instance fields.
- [ ] constructor early-use cannot surface an uninitialized field as a value contrary to its contract.
- [ ] class-side immutable fields have a valid initialization rule.

## Architecture

- [ ] module interfaces distinguish runtime immutable, runtime mutable, static constant, and non-value declarations.
- [ ] compiler does not reinterpret static `const` as runtime immutable.
- [ ] semantic analysis, not lowering, owns initialization proof.
- [ ] lowering consumes a frozen read-safety product.
- [ ] incremental fingerprints include initialization where semantically relevant.

## Migration/tooling

- [ ] AST-aware corpus codemod completed.
- [ ] old/new class layout parity verified.
- [ ] no stale user-facing docs say `let` is mutable.
- [ ] LSP/REPL/TextMate understand contextual `mut`.
- [ ] workspace tests/goldens updated.
- [ ] ordinary local-read performance remains at baseline.

---

# 34. Recommended P1 semantic model

The target can be summarized as four orthogonal axes.

```text
Binding
├── phase
│   ├── Runtime
│   └── Static
│
├── runtime mutability
│   ├── Immutable
│   └── Mutable
│
├── initialization
│   ├── Uninitialized
│   ├── MaybeInitialized
│   └── DefinitelyInitialized
│
└── value/type knowledge
    ├── known / established
    ├── assumed
    ├── unknown
    └── dynamic
```

Do not collapse any two axes.

Examples:

```text
let x = 1
    phase = Runtime
    mutable = false
    init = Definitely
    knowledge = Int

let mut x: Int
    phase = Runtime
    mutable = true
    init = Uninitialized
    contract = Int
    current = no value knowledge

let mut x
if c { x = 1 }
    post-join init = Maybe
    current type = join of reachable value facts
    runtime read = checked

const x = ...
    phase = Static
    runtime mutability = N/A
    P2 owns evaluation
```

This decomposition fits the existing semantic architecture and prevents future C4 phases from inheriting another overloaded `let`/`const` boolean.

---

# 35. Final recommendation

Proceed with C4.P1 as a **semantic cutover**, not a syntax patch.

The repository already provides enough flow and field-lifecycle infrastructure that the strongest parts of the design are practical:

- local definite initialization is a small extension of `FlowState`;
- immutable field lifecycle is a strengthening of an existing lifecycle table;
- explicit fields simplify compiler layout construction;
- contextual `mut` avoids a lexer redesign;
- field mutability already participates in canonical fingerprints.

The two areas that require deliberate new runtime/compiler machinery are:

1. **true uninitialized mutable bindings**, because current initializer-less `let` is effectively populated with `None`; and
2. **sound early-construction field access**, because constructor-exit proof alone does not prevent pre-initialization method calls/escapes from observing an uninitialized slot.

If those two seams are implemented explicitly—and static `const` is prevented from falling back to legacy runtime lowering—C4.P1 provides a clean, durable foundation for P2 static values/evaluation and the remainder of the structural upgrade.
