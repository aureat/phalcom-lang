# TELOS — Class-Side, Initialization, Data-Class, `via`, Structural Patterns, Concise Bodies, and Cached Getter Upgrade

**Detailed repository-grounded implementation plan**  
**Status:** PROPOSED / NOT IMPLEMENTED  
**Prepared:** 2026-10-11  
**Repository:** [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang) (the project is named Telos; repository paths still use `phalcom`)  
**Inspected reference:** default branch `main`, commit [`c2dd635ee34f2cef3e64e86c87a5e77992f29fa7`](https://github.com/aureat/phalcom-lang/commit/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7)  
**Normative input:** attached *Telos Class-Side, Initialization, Data-Class, and Concise Member Syntax Specification*, sections 1–56 and invariants C1–C34.  
**Audience:** advanced reasoning implementer agents, technical reviewer, semantic/compiler/runtime maintainers.  
**Scope:** a design-to-implementation program, **not** a claim that code or tests have already changed or passed.

> **Priority rule.** The attached language specification is normative. This document supplies architecture, sequencing, repo-specific implementation instructions, validation strategy, and proposed resolutions to underspecified corners. Where a recommendation goes beyond the specification, it is explicitly labeled **proposed ruling** and must not silently become a language rule.

---

## 0. Executive directive

Implement this as a **single coordinated semantic-model upgrade delivered in independently verifiable vertical slices**. Do not treat the work as an attribute rename or source-to-source desugaring pass. The project already possesses canonical selector identities, constructor/factory semantic products, field lifecycle proofs, labeled-rest dispatch, first-class immutable `data`, and a class/metaclass runtime. Preserve these authorities; remove competing AST-scanning implementations.

### 0.1 What must remain true

1. `receiver(...)` is ordinary `receiver.call(...)` dispatch. `Self(...)` and `self(...)` are applications, not intrinsically allocation.
2. Primary `init Self(...)` establishes a class-side `call(...)` factory plus a distinct hidden, same-instance initializer. Named `init Self.name(...)` establishes `name(...)`; `new` has no intrinsic privilege.
3. Constructor state is **Initializing → Completing → Published**. No source-originating reference to the initializing receiver may flow or dispatch as an ordinary value.
4. `@derive(Init)` derives a constructor for an opaque ordinary class only. `data class` retains ordinary object identity while introducing a **canonical structural component product**. First-class immutable `data` remains separate and unchanged.
5. Structural matching, equality, hashing, rendering, `with(...)`, and default primary construction consume the **same canonical ordered component product**. These operations read backing storage without executing user-defined getters/setters.
6. `via` separates backing-storage mutability from exposed writability; writable exposure implies mutable storage, not conversely. Fused `via` creates one `FieldId`.
7. `MemberBody::Expression` remains source-distinguishable; `const` means class-side **cached getter**, not compile-time `BindingKind::Const`.
8. Compiler derivation fills absent selectors and never overwrites explicit source behavior. Equality/hash are a coupled pair.

### 0.2 Explicit non-goals

No generic procedural derive framework, open-ended constructor escape analysis, publicly expressible `Fresh<T>`, class-object call opcode, separate runtime `data class` tagged value, class-specific duck matching, automatic memoization of normal methods, constant methods/instance `const` getters, arbitrary opaque-class structural inheritance, or silent compatibility semantics for removed attributes. Do not introduce new `Value` tags or a parallel object hierarchy.

### 0.3 Completion definition

This program is finished only when: all C1–C34 invariants have tests; the full canonical Universe and maintained `.ph` corpus compile under the new surface; legacy `@class`, `@constructor`, `@construct`, `@data class`, and implicit construction `.new` routes are absent or emit deliberately specified migration diagnostics; the semantic snapshots and executable lowering agree; no old canonical `Class.new()` signature remains; and all mandated negative diagnostics and runtime proofs pass the certification matrix in §19.

---

## 1. Evidence base and impact map

Repository observations below were checked against the stated `main` commit using the connected GitHub repository. Line numbers are **navigation hints**, not durable APIs. Before applying a patch, reopen symbols on the actual implementation branch and record a fresh commit SHA.

| Subsystem | Observed current implementation | Required change |
|---|---|---|
| AST body | [`phalcom-ast/src/ast.rs`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-ast/src/ast.rs#L2063), `MemberBody::{Declaration, Block}` | Add `Expression(Expr)` and update every exhaustive consumer. |
| AST class/members | `ClassDef` ~L234, `ClassMember` ~L1696, `GetterDef` ~L2120 | Distinguish `data class`, explicit initializer source form, callable origin, cached getter, side-marker syntax. |
| `via` AST | `DelegatedAccessorDef` ~L631 contains name, kind, target field, attributes | Preserve target-side `mut`, fused backing type/default, exposure mutability, visibility, source ranges. |
| Member parser | [`parser.rs::parse_class_member`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-ast/src/parser.rs#L5945) | `Self` heads, `self(...)`, `init`, class-body `=`, `via`, new field disambiguation, `const`. |
| Index parser | `parser.rs::parse_index_member` ~L6192 | Expression-bodied subscripts, unnamed assigned-value parameter → `value`, class-side `Self[...]`. |
| Body parser | `parser.rs::parse_member_body` ~L6779 accepts block/declaration | Add explicit `= expr`, without conflating with setter-head `=`. |
| Attribute parser | `parser.rs::parse_attribute` ~L5768 special-cases `@construct`, `@class`, `@data` | Bounded `@derive(Init)`; remove old attributes after migration, avoid `construct` keyword hack. |
| Selectors | `phalcom-common/src/selector.rs` with `SelectorBase`/`SelectorSlot` | Reuse exactly; canonical selector notation `call(x, y)`, never colon-suffixed strings. |
| Constructor authority | [`phalcom-semantic/src/signature.rs::ConstructorInitializationProduct`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-semantic/src/signature.rs#L248) ~L248–L290 | Reuse factory+initializer pair, canonical field mappings and fingerprints; evolve origin names. |
| Intrinsic `new` | [`declaration_signature.rs::canonical_core_class_new_signature`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-semantic/src/checker/declaration_signature.rs#L417) | Delete universal `Class.new()` semantic privilege at migration cutover. |
| Super proof | `phalcom-semantic/src/checker/body.rs` ~L299 checks first `super.new(...)` | Resolve `super(...)` and named super factories via canonical products, not text `new`. |
| Field proof | [`checker/field_lifecycle.rs`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-semantic/src/checker/field_lifecycle.rs) | Retain definitely/maybe initialized field flow; layer receiver confinement, completion. |
| Constructor lowering | [`phalcom-core/src/compiler/attributes.rs::lower_constructors`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-core/src/compiler/attributes.rs#L2794) | Keep verified allocate → hidden initializer → factory result; migrate selector/provenance/completion. |
| Legacy `@data` | `attributes.rs::derive_data` ~L1854; `build_data_eq/hash/to_string/with` | **Replace**, do not merely rename: today uses raw field scans, peer getter calls, `None` omission, and `.new` reconstruction. |
| Runtime allocator | [`primitive/class.rs::class_new_`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-core/src/primitive/class.rs#L175) actually binds internal `Class.allocate()` | Keep internal allocator and native/abstract refusal guards. Do **not** expose as normal construction. |
| Class runtime | [`heap/class.rs::ClassObject`](https://github.com/aureat/phalcom-lang/blob/c2dd635ee34f2cef3e64e86c87a5e77992f29fa7/phalcom-core/src/heap/class.rs#L25): methods, rest methods, instance/static slots | Attach/resolve runtime structural layout and cached getter state without new object kind. |
| Rest dispatch | `RestMode::Labeled`; `ClassObject.rest_methods`; `vm/dispatch.rs` | Reuse for a single generated `with(**changes)` family. |
| Pattern semantics | `checker/pattern.rs::resolve_pattern` and variant handling; `compiler/lib/patterns.rs` | Extend nominal patterns with data-class products; don't disguise classes as enum variants. |
| Primitive equality/hash | `primitive/object.rs::{object_eq,object_hash,object_same}` | Preserve `===`; structural `==` and coherent type-stable mutable hash in derived class behavior. |
| Executable metadata | `modules/semantic_lowering.rs` carries constructor products, super resolutions and binding maps | Extend projection for structural layout/generated behavior; prohibit compiler rediscovery of authority. |
| Module `const` | `StaticConstantId`, static evaluator/materialization, `BindingKind::Const` | **Do not reuse** for runtime lazy class-side `const` getter. |

### 1.1 Architecture that must be respected

- The selector identity is **base + positional/labeled slot shape**, never parameter type or result type. Literal selectors in docs/tests must use `call(x, y)` / `call(_, _, options)`, not `call(x:y:)`.
- Semantic checking can be more precise than advisory/LSP heuristics; preserve **one canonical semantic authority**, then project to codegen, inlays, source index, and runtime metadata.
- The runtime uses ordinary classes/metaclasses and heap handles. Generic allocation refuses native-representation and abstract classes; all newly introduced factory paths must preserve those refusal paths.
- The existing first-class `Statement::Data` / `DataObject` / product storage is a different language category; avoid unifying runtime representation merely because the new spelling contains `data`.
- Direct storage operations and cross-instance field access have different authority. Structural operators may use compiler-authorized slot projection but must not grant arbitrary user code a private-field-peeking API.

---

## 2. Semantic design records to freeze before disruptive changes

Create a short decision ledger in the implementation branch; freeze **required** contracts and explicitly rule the **underspecified** cases below. Decisions may live in normal lifecycle records if a program/checkpoint is assigned; don't invent an ID inconsistent with the existing lifecycle conventions.

| ID | Question | Proposed bounded ruling | Why it matters |
|---|---|---|---|
| D01 | May two data-class exposed components reference one backing `FieldId`? | **Proposed:** reject duplicate structural backing, allowing explicit `via` to *replace/refine* default exposure. | `with(first:)` otherwise also mutates unspecified alias `second` and violates preservation. |
| D02 | What is `with()` with no updates? | **Proposed:** if generated, return a *fresh* identical instance; never return `self`. | The specification requires fresh copy/update; single labeled-rest family can accept empty pack. |
| D03 | What if a data-class pattern provides only some labels without `..`? | **Proposed:** require full shape or explicit rest/omission syntax according to current pattern grammar; no implicit partial match. | Prevent unresolved arity ambiguity. |
| D04 | Can a primary constructor exist as a bare `Self(...)` declaration? | **Proposed:** only `init Self(...)` defines construction; `Self.call(...)` is an ordinary class method; disallow `Self(...)` **declaration-head** sugar pending a separate decision. | Spec permits `Self(...)` declaration only conditionally. |
| D05 | What happens after a throwing `const` getter? | **Proposed:** clear `Evaluating`, revert to `Unevaluated`, retry on later access; no successful result cached. | Spec fixes successful-only cache; rethrow policy deferred. |
| D06 | Recursive access while a cached getter is evaluating? | **Proposed:** deterministic cycle/reentrancy diagnostic, not a deadlock or `None`. | Needed for safe lazy evaluation even without full concurrency semantics. |
| D07 | How does `with` copy nonstructural hidden state, if present? | **Needs explicit ruling.** Conservative initial scope: permit only fully represented object state or define separate safe handling for nonstructural slots. | Component-only copy cannot silently leave hidden fields uninitialized. See §14.4. |
| D08 | How do derived constructors deal with mutable parent state and inherited required inputs? | Use parent canonical constructor product and superclass-first layout; reject unprovable generated paths. | Avoid re-creating constructors through synthesized bodies with unchecked parent paths. |
| D09 | What is the meaning of `const` in class-body field position? | A class-side getter modifier only; module/local `const` semantics stay unchanged. | Avoid accidental static-evaluator integration. |
| D10 | Dynamic `with(**pack)` labels? | Validate exact statically known labels at compile time; for dynamic labels, perform runtime validation before allocation or any copy side effect. | Label safety cannot be lost in rest-family fallback. |

**Required gate:** No structural generator implementation until D01, D07, and D08 have a coherent written decision. Other proposed rulings can be confirmed as each owner slice begins. These are *not* claims that the specification already resolved them.

---

## 3. Target authority/product design

### 3.1 Canonical product dependency graph

```text
Parsed ClassDef (+ ClassFlavor, InitDef, ViaDef, MemberBody)
  ├── Declaration identity + member-side classification
  ├── Canonical FieldSignatureProduct
  │      └── resolved via requests / visibility / mutability / defaults
  ├── StructuralClassProduct?   [data class only]
  │      ├── ordered component identities
  │      ├── inherited prefix
  │      ├── canonical FieldId projections
  │      └── exposed label/access/mutability/default metadata
  ├── ConstructorInitializationProduct(s)
  │      ├── class-side factory CallableId
  │      ├── hidden initializer CallableId
  │      ├── parameter→field mapping
  │      └── parent initializer resolution
  ├── CallableSignatureProducts + source-selector occupancy
  └── GeneratedBehaviorPlan
         ├── public component accessors (where absent)
         ├── structural == / hash pair
         ├── toString
         └── with(**changes)
                 ↓
        ExecutableModuleLowering
                 ↓
        Class/metaclass methods + runtime slot layout + cache policies
```

### 3.2 Proposed types (conceptual Rust, not copy-paste patches)

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassFlavor { Ordinary, DataClass }

pub enum MemberBody {
    Declaration,
    Expression(Expr),
    Block(Vec<Statement>),
}

pub struct InitializerDef {
    pub factory_name: Option<String>, // None => primary call(...)
    pub params: Vec<ParameterDef>,
    pub body: MemberBody,
    // common attributes, visibility, generics and exact source ranges
}

pub enum GetterEvaluation { Ordinary, Cached }

pub struct StructuralClassProduct {
    pub owner: DeclarationId,
    pub inherited_component_count: usize,
    pub components: Box<[StructuralComponent]>,
    pub input_fingerprint: ProductFingerprint,
}

pub struct StructuralComponent {
    pub id: StructuralComponentId,
    pub name: Arc<str>,
    pub backing_field: FieldId,
    pub declared_type: TypeId,
    pub storage_mutable: bool,
    pub writable_exposure: bool,
    pub visibility: Visibility,
    pub origin: ComponentOrigin,
    pub declaration_order: u32,
    // default information belongs to field identity, not selector identity
}

pub enum ConstructorOrigin {
    Source,
    DerivedFields,
    DataClassPrimary,
    DefaultOnly,
}
```

**Do not** force all of these onto existing public structs if a better internal decomposition exists; preserve exact identity/provenance and avoid copies with divergent semantics. `StructuralComponentId` should be owner-qualified and stable across source edits that do not change the declaration identity; executable slot projections are derived separately.

### 3.3 Selector occupancy and source provenance

Construct a canonical occupancy map keyed by `(owner, dispatch_side, Selector)` **before** requesting generated members. Record `Source` or generated origin separately from method semantics. Source wins per selector; the special `==`/`hash` pair rejects half-override. A `const` getter is `origin=Source, evaluation=Cached`, not necessarily `origin=ConstGetter`; do not confuse provenance and evaluation policy.

### 3.4 Invalidation authority

Add semantic DB query/product fingerprinting for structural class shape. Include: ordered component identities, labels, types, access, storage mutability, defaults, superclass structural product dependency, and relevant source selector occupancy. A change to an implementation-only getter body must not pretend to change component slots, but a change to `via`/field mutability must invalidate all structural consumers. Freeze and assert dependency propagation in incremental tests.

### 3.5 Runtime projection

Produce a compact, read-only runtime structural layout by projecting canonical `FieldId`s to physical instance slot indices after normal class layout. Retain enough exact class identity to distinguish inherited prefix projections and exact-class equality. Avoid constructing runtime layout by parsing emitted getter bytecode or matching field names. Preserve GC tracing for any new cache roots and new runtime-held values.

---

## 4. Delivery model and ownership rules

This should be executed as a **13-package ordered program**, with narrow parallelism only after signature contracts are frozen. The packages below are implementation units, not mandatory commit counts.

| Package | Scope | Depends on | Review difficulty |
|---|---|---|---|
| P0 | Baseline, contracts, decision ledger, test fixtures | — | Medium |
| P1 | AST/parser, concise bodies, `self`/`Self` syntax, argument label inference | P0 | Medium–high |
| P2 | Class object ordinary `call`, class-side semantic side, `new` de-privileging scaffold | P1 | High |
| P3 | `init`, named factories, canonical constructor products, parent selection, lowering | P2 | **Very high** |
| P4 | Receiver confinement, initialization field flow, completion/invariants | P3 | **Very high** |
| P5 | `@derive(Init)` and generated default path | P3–P4 | Medium–high |
| P6 | `via`/field canonicalization and StructuralClassProduct | P1, P4, decisions D01/D07 | **Very high** |
| P7 | `data class` primary init, flattened inheritance, generated accessor provenance | P3–P6 | **Very high** |
| P8 | Nominal structural patterns/irrefutability/visibility | P6–P7 | High |
| P9 | Structural equality/hash/toString | P6–P7 | High |
| P10 | True `with(**changes)` copy/update | P4, P6–P7 | **Very high** |
| P11 | Cached class-side `const` getter | P1–P2, completion ownership decision | High |
| P12 | Universe/packages/examples migration, LSP/docs, whole-program certification | all earlier | High |

**Parallel window:** P8 (patterns), P9 (equality/hash/render), and P11 (cache) may proceed in isolated worktrees after P6/P7 APIs are stable, provided they each consume the same published product and do not edit shared compiler core simultaneously. P10 needs an explicit layout/copy review and should not be delegated as routine boilerplate.

**Branch discipline:** each package records actual base SHA, touched files, tests executed, failures and reasons, semantic evidence, and any spec deviations. No package may label full certification passed based on focused tests alone.

---

## 5. P0 — baseline, specifications, and executable contracts

### Required actions

1. Create a clean implementation worktree/branch from the intended base. Record `git status`, exact HEAD, `cargo test`/`scripts/verify.sh` availability and baseline failures before edits.
2. Inventory all live `@class`, `@constructor`, `@construct`, `@data`, `Class.new`, `Type.new`, `super.new`, `BuiltinAttr::{Class,Constructor,Construct,Data}`, `is_constructor`, compiler `new` assumptions and current `MemberBody` matches; categorize *construction* uses versus ordinary `new` methods, because `new` remains legal.
3. Inventory source metadata and public semantics consumers (semantic shard, source index, LSP, native descriptors, static constants, Universe bootstrap, method reflection, generated constructors).
4. Add minimal fixture classes representing opaque, derived, `data class`, inheritance, mutable/immutable fields, generic class application, native/abstract classes and first-class immutable `data` regressions.
5. Write/approve the D01–D10 ledger above and record the exact expected parser/semantic diagnostic shape.
6. Verify repo's current test command registration and golden fixture naming before adding new test files.

### Acceptance

- Baseline results and known unrelated failures recorded.
- Every new semantic feature has a named owner subsystem and planned negative tests.
- No ambiguity between source spec and implementer-proposed rulings.
- No source mutation from this package unless it adds tests/inert types/decision records.

---

## 6. P1 — language surface, AST truth, concise member bodies

### 6.1 Parser and AST changes

**Owned anchors:** `phalcom-ast/src/{token,lexer,ast,parser,selector}.rs`, parser snapshots, LSP lexical highlighting later in P12.

1. Parse `data class C` into `Statement::Class(ClassDef { flavor: DataClass, ... })`, not `Statement::Data`. Keep `data Name(...)` on existing first-class immutable-data grammar; use lookahead to disambiguate.
2. Parse `init Self(...) { ... }` and `init Self.name(...) { ... }` into an explicit initializer source AST. Reject authored `-> ReturnType` on `init`; admit contextual `init` only in declaration grammar unless lexing requires a minimal contextual token strategy.
3. Parse `Self.method`, `Self.method(...)`, class-side getter/setter/subscript heads and record `DispatchSide::Class` directly in syntax/semantic indexing. The prefix must not become a selector segment.
4. Parse instance `self(...)` declaration as the `call` selector on instance side. Preserve exact source spelling for diagnostics, but canonical selector must be identical to `call(...)`.
5. Preserve expression-side `self(...)` and `Self(...)` as calls to `call`, not allocation.
6. Parse `_: x` at a call argument site as **label-inferred** from the source binding `x`. Do not confuse this with anonymous parameter `_ x` or positional `x`. Reject `_: arbitraryExpression` unless separately specified. Compute selector slot `Label("x")` before dispatch.
7. Add `MemberBody::Expression(Expr)` and parse `= expression` where a member executable body is allowed. Keep `name = expr` getter distinct from `name: T = expr` field default and `mut name = expr` mutable field default.
8. Parse property/subscript setter **head** `=` and body `=` separately; unnamed assigned-value parameter binds `value` (or retain an implicit-origin flag and synthesize in the semantic binder). Explicit local parameter name wins.
9. Recognize `const` only for class-side getter declarations and reject its use on methods, setters, subscripts, instance getters, and `self` call heads. No module `const` changes.
10. Capture the full `via` syntax now (including declaration order, source spans, target `mut`, fused type and default), but defer normalization and semantic publication to P6.

### 6.2 Expression-bodied contract

A normal getter/method expression body returns its expression result. A setter expression body must preserve the language-defined setter result, regardless of the value produced by assignment. An `init` body remains a statement/block form unless the language specification explicitly admits expression-bodied `init` (it does not). Do not turn expression bodies into blocks during parse; introduce a **single canonical executable-body lowering adapter** consumed by ordinary compiler codegen and decorator weaving.

### 6.3 Cross-consumer audit checklist

Search all `match MemberBody`, `.statements()`, `.statements_mut()`, AST cloning/serialization, fingerprints, source index, inlay, contract weaving, evaluator, pretty printer and tests. No exhaustive match may quietly classify `Expression` as `Declaration` or skip `@requires` / `@ensures`. Generated body origins must not use malformed fake source ranges.

### 6.4 Tests and acceptance

- Parser positive matrix: all member classes (getter, setter, method, index), `Self` side, `self` call head, `init` primary/named, `data class` vs immutable `data`, all `via` forms, inferred labels, expression body, field grammar, const getter.
- Parser negatives: `init ... -> T`, `const method(...)`, `const setter`, `const index`, malformed `via`, `_: complex.expr`, duplicate/conflicting head `=`; stable ranges.
- AST snapshots preserve `Expression` and source spelling; binder records implicit `value` without claiming an authored identifier span.
- Integration proof that normal module `const`, existing immutable `data`, and selectors with mixed slot shapes remain unchanged.

**Exit gate P1:** AST representation complete, parsing deterministic, no silently swallowed trailing tokens or field/getter ambiguities; all parser tests green. Old runtime need not yet understand new nodes.

---

## 7. P2 — canonical ordinary application, class-side placement, `new` transition

### 7.1 Semantic changes

**Owned anchors:** `phalcom-semantic/src/checker/{expression,declaration,declaration_signature,call}.rs`, `semantic_shard.rs`, `source_index/builder.rs`, `phalcom-common/src/selector.rs` (read-only unless a genuine gap), `phalcom-core/src/compiler/lib/expr.rs`.

- Resolve a class expression (`Point`, expression-position `Self`) to its class object; `Point(...)`/`Self(...)` is an ordinary method application with selector base `call`. The current `synthesize_unqualified_call` has class-name special cases; normalize through the existing callable application path without inventing constructor bytecode.
- Bind `Self` expression according to the lexical declaring class (including generic specialization); do not represent this as arbitrary `Var("Self")` or confuse with type-position `Self`/dispatch-side declaration head.
- Canonicalize source `Self.foo` to class-side method identity immediately. Remove dispersed tests for attached `@class` once the migration switch is complete.
- Ensure `self(...)` source declaration and `call(...)` both resolve to exactly the same selector identity; emit duplicate-selector diagnostics across the spellings.
- Maintain ordinary method families, dynamic dispatch, receiver authorization, signature checking, type knowledge, and reflection of callable members. `Self(...)` must obey the same access and selector-shape rules as `Self.call(...)`.
- Stop **semantic privilege** from originating merely in the method name `new`. Audit `canonical_core_class_new_signature` and all registrations/callers; plan removal **with** the Universe corpus switch, not prematurely in a still-legacy bootstrap.

### 7.2 Runtime/Universe transition strategy

The repository still uses `Class.new()` heavily and the internal native allocator is actually `Class.allocate()` (`primitive/class.rs::class_new_`). Keep **internal** `allocate` and abstract/native guards. During the controlled migration window, a compatibility path may keep legacy Universe bootstrapping operational **only** under a named migration gate; it cannot be counted as final language behavior. Do not expose a new public bare allocator or silently let `.new` allocate after the final cutover.

### 7.3 Tests and acceptance

- Exact equivalence of `receiver(args)` and `receiver.call(args)` for class objects, callable instances and `Self`/`self`; method overrides and access control respected.
- `new` is an ordinary identifier: source `Self.new(...)` may return an arbitrary non-class value, has no implicit allocation/type evidence, and behaves as a normal method.
- `call(x, y)` vs `call(_, y)` and argument `_: x` retain different exact selector shapes where appropriate.
- Class-side `Self` getters/setters/subscripts use metaclass side; instance-side remain instance side.
- Bootstrap/native/abstract refusal tests retain their behavior; no uncontrolled naked allocation accessible via class application.

**Exit gate P2:** Application semantics uses ordinary callable resolution; remaining `new` privileges are explicitly isolated behind migration status and not presented as final-state supported behavior.

---

## 8. P3 — `init`, constructor authority, named parent selection, exact lowering

### 8.1 Semantic signature formation

**Owned anchors:** `phalcom-semantic/src/{signature,session}.rs`; `checker/{declaration_signature,body,expression,context}.rs`; `db/{fingerprint,query,product}.rs`; `phalcom-core/src/compiler/{attributes.rs,lib/class_decl.rs}`; `modules/semantic_lowering.rs`.

1. Change `ConstructorOrigin::{Source,Construct,Data,DefaultOnly}` to `Source/DerivedFields/DataClassPrimary/DefaultOnly`, migrating persisted interfaces and tests safely.
2. Register every `InitializerDef` using the existing `ConstructorInitializationProduct`; retain canonical factory `CallableId`, hidden initializer `CallableId`, `default_fields`, `parameter_fields`, and input fingerprints.
3. Primary `init Self(x: T, y: U)` claims **class-side `call(x, y)`**. Named `init Self.origin()` claims `origin()`. Selector collisions check side+slots, not parameter/result types.
4. Hidden initializer identity must distinguish multiple factories while remaining compiler-only and bound to the declaring instance owner. Its exact internal encoding is implementation-specific; never publish the hidden initializer as an arbitrary public `init` method.
5. Reject explicit `-> T` in source; the public factory result is compiler-established `Self`, independent of the last body expression and advisory/inlay guesses.
6. Preserve the current lowerer: internally allocate, call the matched hidden initializer on **the same instance**, complete, then return exactly the allocated instance. No source factory/body return value replaces it.
7. Recognize `super(...)` / `super.name(...)` only as parent initializer selection within an initializer. Use parent canonical factory signature metadata to obtain its **hidden initializer**. Do not issue ordinary dynamic `super.call` on the metaclass.
8. Parent selection required exactly once where parent has a construction obligation, before subclass state formation. A child's named initializer does not select an equally named parent by implication.
9. Preserve checked default assignment **after** explicit parent initialization for a source initializer: current `lower_constructors` already splices defaults after recognized super. Generalize its detector from `super.new` to canonical super selection, and assert checker/lowerer use the same semantic resolution site.
10. Use checked/proven parent availability; reject missing, duplicate, ambiguous, incorrect-access, and cyclic parent construction. Derive-generated child primary initialization must invoke the appropriate parent path or fail closed rather than bypassing it.

### 8.2 Mandatory selector/identity evidence

```text
init Self(x: Int, y: Int) => class-side call(x, y)
init Self.origin()        => class-side origin()
Self.call(x: Int, y: Int)  => same occupied selector as primary init
Self.new(...)             => ordinary, unrelated selector
self(_: Int)              => instance-side call(_)
super(_: x, _: y)          => inferred labels x, y; selects parent call(x, y)
```

### 8.3 Compiler product consumption

The compiler's constructor expansion may still produce temporary internal executable method projections, **but only from canonical products**. It must not decide factories by scanning `@constructor` attributes or matching `name == "new"`. Preserve range-to-product mapping only as a compiler bridge, and reject missing/unused products. Generated field mapping and default ordering use canonical `FieldId`s, not string-driven rediscovery.

### 8.4 Tests and acceptance

- Construction: primary, named, multiple distinct named, same selector collision, generic `Self`, default-only, subclasses, abstract/native refusal, exact returns and throw behavior.
- Negative: authored return type, wrong return, duplicate primary, insufficient label/arity, missing parent, double super, ordinary super call, mismatched parent factory, source/default field mapping mismatch.
- Product assertions: factory/initializer IDs, origin, parameter mapping, default order, parent link, source site, fingerprint on field edits. Compiler output must match semantic product, including codegen re-run.
- `super(_: x, _: y)` must be tested against `super(x: x, y: y)` as equal selection; no Objective-C-style selector serialization.

**Exit gate P3:** constructor semantics and emitted methods agree structurally; no fabricated construction inference from selector text `new`; parent-init proof enforced.

---

## 9. P4 — confined receiver, field proof, lifecycle completion

### 9.1 Receiver-phase model

Introduce an internal checking phase, keyed by canonical initializer owner:

```rust
enum ReceiverPhase {
    Ordinary,
    Initializing { owner: DeclarationId, initializer: CallableId },
    Completing { owner: DeclarationId }, // compiler-only
}
```

A source-initializer receiver is `Initializing` for **the entire body**, even after all fields become definitely initialized. Field completeness is distinct from publication eligibility.

### 9.2 Conservative confinement algorithm

- In `Initializing`, reject every ordinary source-level use of `self` as a value, argument, receiver, return value, alias, closure capture, container element, global/class-side store, `Dynamic`/`Any` conversion, or continuation transfer. Reject ordinary `super` dispatch, too. Allow only the compiler-recognized parent initialization form.
- Direct `_field` reads/writes are special checked storage operations, **not** methods, getters, setters or subscripts; validate ownership, visibility, mutability, definite initialization and write uniqueness using existing `field_lifecycle` flow states.
- Calls not carrying/capturing the initializing receiver are legal. Do **not** ban all function invocation inside initializers.
- Explicit `return` (no value) is a normal successful exit only if all required fields are proven complete; `return self` and `return value` are static errors. `throw` may abort without publication.
- Field declaration defaults run under the same confinement policy and in canonical execution order. Default reads of prior initialized fields may be legal; reads of later/uninitialized fields are rejected or fail the same proof.
- Reject suspension/continuation transfer (`await`, yield, escaping block home token, or future equivalents) even if current parser does not yet expose all forms; tests/guards should fail closed as syntax evolves.
- Do not implement full graph/interprocedural escape analysis. The bounded receiver rule is intentionally simpler and stronger.

### 9.3 Runtime completion boundary

Compiler-only lifecycle:

```text
allocate unpublished
  -> execute exactly selected parent initializer (same receiver)
  -> initialize this class's defaults/fields
  -> verify definite initialization/immutable uniqueness
  -> enter Completing
  -> compiler-authorized completion/invariant checks
  -> Published
  -> return from public factory
```

The current invariant rewriter `attributes.rs::weave_invariant_checks` uses sends such as `self.invariantEnter()` inside constructor execution. Move/adjust these checks so they are not source-level ordinary dispatch on an `Initializing` receiver; do not suppress invariants. Completion must also be reused by `with(...)`.

**Implementation choice:** This lifecycle may use compiler-validated control flow plus hidden runtime bookkeeping instead of a new public/heap Value variant. Choose the minimal mechanism that actually prevents premature observable publication; document whether runtime guards are needed for dynamic or foreign/native escape paths. A merely documented `Unpublished` concept without enforcing confinement is insufficient.

### 9.4 Tests and acceptance

Positive: direct writes, previous-field reads, computation, unrelated calls, successful bare return after complete proof, failed/throwing init has no externally reachable `self`.

Negative: `registry.add(self)`, `callback(self)`, `let alias = self`, `|| self`, `return self`, `self.validate()`, `self.x`, `self(...)`, `super.method()`, global/class-side store, data-structure element/key, unchecked dynamic cast/capture, early return before required field, read-before-init, second immutable write, duplicate super, ordinary getter/setter inside init.

Proof-model tests: conditional paths, loops, throw-only paths, try/catch/finally/ensure, inherited fields, defaults, fixed-point stability, diagnostic source attribution, and no stale advisory replacement of constructor `Self` by tail assignment result.

**Exit gate P4:** no source-level initializing receiver escape/dispatch reaches runtime; completion/invariants occur once and after valid state; constructor success paths have checked field proofs.

---
## 10. P5 — bounded `@derive(Init)` implementation

### 10.1 Scope

The source grammar already accepts general positional attribute arguments. Recognize **only** the exact built-in derive request `@derive(Init)` here; do not implement a generic macro/trait/procedural derive framework. If the supplied derive argument resolves ambiguously or is not the accepted `Init` marker, reject with a targeted diagnostic rather than speculatively executing it.

### 10.2 Semantic behavior

- Apply only to ordinary `class` (unless a separate compatibility decision changes that); a `data class` already derives primary construction without the annotation.
- Derive a primary **class-side `call(...)` factory + paired hidden initializer**, with `ConstructorOrigin::DerivedFields`, through the same semantic product insertion as P3.
- Consume canonical own-field identities and declaration defaults in order. Fields with defaults are still state but not required constructor arguments. Getter/setter availability does not control constructor parameters.
- An explicit source initializer or source-authored class-side `call` occupying the would-be factory selector prevents generation; record whether it is a valid source initializer or merely an ordinary factory. Do not generate an inaccessible orphan initializer.
- Preserve parent construction/visibility proof. Do not silently skip parent initialization if inherited required state is not accounted for.
- Do **not** generate equality, hash, matching, accessor pairs, `toString`, `with`, or a structural product.

### 10.3 Tests

```telos
@derive(Init)
class Coordinate {
    _x: Int
    _y: Int
    x = _x
    y = _y
}

let c = Coordinate(x: 10, y: 20)
```

Assert exact `call(x, y)` selector, fields written once, opaque class remains identity-equal by default, `Coordinate(_: x, _: y)` is **not** a data-class structural pattern, no implicit `with`, and no data-class reflection. Repeat with defaults, generic fields, explicit override, named extra initializer, superclass, no fields, and invalid duplicate/unknown derive inputs.

**Exit gate P5:** derived initializer is a semantic constructor product, not a compiler-only fabricated `new` method; scope remains bounded to `Init`.

---

## 11. P6 — field canonicalization, `via`, and `StructuralClassProduct`

### 11.1 Most important dependency

**Do not derive accessors, constructors, patterns, or structural methods before field/via relationships have become canonical semantic products.** This package is the architecture pivot of the upgrade.

**Owned anchors:** `phalcom-ast/src/ast.rs::{FieldDef,DelegatedAccessorDef,ClassDef}`, field parsers; `phalcom-semantic/src/{field_key,identity,db,...}`, `checker/{declaration,declaration_signature,field_lifecycle}.rs`; source indexing. Add a dedicated `structural_class.rs` or similarly scoped semantic module rather than inflating an unrelated existing checker file.

### 11.2 Data model and phases

Separate these three facts:

1. **Storage** — `FieldId`, declaring owner, field type, default expression, physical mutability, physical slot identity, visibility.
2. **Exposed component** — `StructuralComponentId`, exposed name, canonical ordered position, backing `FieldId`, structural visibility, optional writable exposure.
3. **Behavioral accessor** — ordinary selector with source/generated provenance; it can be overridden and is **not** the source of structural state.

Canonical assembly algorithm:

```text
collect source FieldDef and ViaDef in declaration order
  → validate all fused target declarations
  → resolve existing target FieldIds / create exactly one fused FieldId
  → apply physical mutability OR rule before FieldSignatureProduct publication
  → record explicit via claims of backing fields
  → calculate bare-field default component exposures for unclaimed fields
  → assemble ordered own components (explicit and implicit)
  → validate duplicates, access, types, field ownership, aliasing ruling
  → prepend canonical superclass components for data-class inheritance
  → publish StructuralClassProduct with stable identities/fingerprint
  → request accessor/constructor/behavior derivations
```

### 11.3 Mutability law

```text
storage_mutable = source_field_mut
               OR via_target_mut
               OR exposed_writable_mut

writable_exposure = exposed_writable_mut
```

`count via mut _count` provides mutable storage but no synthesized setter. `mut count via _count` implies mutable backing and generated setter. `mut count via mut _count` is legal redundancy. An explicit source setter may occupy the selector even if the `via` exposure itself is getter-only; ordinary setter behavior must still obey the field mutability rules.

### 11.4 Fused declarations

`mut count via _count: Int = 0` creates **one** canonical backing field and one exposure relationship in the same declaration. Reject another authored declaration of `_count`; the non-fused `mut count via _count` is the valid form when storage already exists. Respect declaration default order and exactly-once immutable initialization. A fused default is the field default; never treat it as a getter default or reconstruct it from synthesized accessor code.

### 11.5 Component order and alias collisions

- Preserve source-defined component order; flatten superclass prefix before own components.
- An explicit `via` to an otherwise bare default-exposed field **refines/replaces** that bare exposure, rather than causing a duplicate default component.
- Reject two components with the same exposed name, overlapping inherited names unless a separate overriding rule exists, or two backing aliases if D01 adopts the recommended one-to-one constraint.
- Do not filter structural components by caller accessibility when creating the product or assigning positions.
- Do not include arbitrary computed getters; `area = ...` in `Circle` is behavior, not structural state.

### 11.6 Ordinary class `via` vs data-class `via`

The `via` surface works as accessor/storage delegation on ordinary classes, too. **Only** `data class` publishes components for structural consumers. Avoid accidentally making `@derive(Init) class` or any ordinary `class` structurally matchable because it contains `via`. One internal relation can support both storage delegation and structural-data mapping, but the class flavor determines which structural product exists.

### 11.7 Diagnostics and tests

| Positive surface | Expected canonical fact |
|---|---|
| `_x: T` in data class | component `x`, immutable storage, read-only |
| `mut _x: T` | component `x`, mutable storage, writable exposure |
| `count via _count` (existing field) | explicit read-only exposure, exact backing `FieldId` |
| `count via mut _count` | mutable backing, still read-only exposure |
| `mut count via _count` | writable exposure and mutable backing |
| `mut count via mut _count` | same effective state as previous |
| `count via _count: T` | one fused immutable field |
| `count via mut _count: T = e` | one fused mutable defaulted field, no implicit setter |
| `mut count via _count: T = e` | one fused mutable defaulted field with getter+setter |

Negatives: duplicate fused+ordinary backing field, absent target in non-fused form, conflicting backing type/default, unknown/malformed `via`, duplicate component names, cross-owner illegal field alias, inaccessible field reference, incompatible inherited component, and (after D01 approval) duplicate structural backing.

Proof tests must inspect **canonical field identities, component identities, mutability, visibility, default order and semantic fingerprints**. Runtime accessor output alone is insufficient.

**Exit gate P6:** one authoritative structural product; consumer code can request component metadata without scanning raw field AST or generated getters.

---

## 12. P7 — `data class` derivation, primary construction, inheritance, accessor precedence

### 12.1 Structural ordinary class identity

`data class` is `Statement::Class` and `ClassId`/ordinary heap instance, with `ClassFlavor::DataClass`. Preserve object identity, aliasing, ordinary subclassing within the allowed data-class ancestry, and dynamic dispatch. Never lower it to the `data` immutable-product runtime arm.

### 12.2 Generated primary initializer

- Every data class requests a **primary `ConstructorOrigin::DataClassPrimary`** path from canonical components; do not require `@derive(Init)`.
- Parameter labels use component **exposed names**, not backing field names; nondefaulted components form required input set. Defaults remain complete state but may be omitted from the required input selector.
- Generated hidden initializer writes canonical backing `FieldId`s directly. It never invokes generated/source setters, computed getters, or arbitrary `call` methods.
- Explicit `init Self(...)` with the same selector suppresses generated primary construction; differently selectored named initializers coexist. An ordinary authored `Self.call(...)` occupying the generated selector also triggers source-precedence handling; surface diagnostics should distinguish a custom factory from a verified initializer.
- Source getter/setter occupying a generated selector suppresses only that accessor; never deletes/rewrites the component.

### 12.3 Inheritance

- Initially allow only `Object` or another `data class` as direct parent; reject opaque ordinary-stateful parents.
- Flatten `parent.components ++ own.components`, retaining parent `StructuralComponentId`/`FieldId` and stable indices.
- Ensure the actual **construction sequence** initializes parent required state before subclass-specific defaults/fields. For generated subclass initialization, explicitly resolve a compatible parent constructor product rather than writing inherited slots from the child without proof.
- Data-class equality later requires exact dynamic class identity. Base-class pattern matching may accept subclasses by nominal compatibility while projecting the inherited prefix; those are intentionally different rules.
- Product fingerprints/dependencies include parent component layout and visibility changes; child invalidation must occur after parent edits, not after unrelated sibling edits.

### 12.4 Source/generated collision handling

Centralize selector occupancy before derivation. Accessors are independent: source `y=(_)` suppresses generated setter only; source `y` suppresses generated getter only; the `y` component stays present. `==` and `hash` are a coupled exception (P9). Additional computed getters are not structural components.

### 12.5 Tests and acceptance

Assert `data class` instances are ordinary reference objects (`===` distinguishable), initializers return the allocated receiver, accesses use generated methods only where missing, custom getters do not alter underlying component projection, defaults can be supplied/omitted only as the canonical constructor input shape dictates, and subclass component order is parent-first. Test explicit init with transforms, named init, source setter override, superclass field obligations, no-field data class, visibility collisions, and generic data-class applications.

**Exit gate P7:** `data class` and `@derive(Init)` have distinct behavior despite using the same constructor authority; no first-class `data` regression.

---

## 13. P8 — nominal structural matching and refutability

### 13.1 Extend existing pattern infrastructure

**Owned anchors:** `phalcom-ast/src/ast.rs::{Pattern,VariantPattern,...}`, parser pattern atom resolution, `phalcom-semantic/src/checker/pattern.rs` and coverage/refutability, `phalcom-core/src/compiler/lib/patterns.rs`, advisory/source-index pattern walkers.

The current AST already represents constructor-shaped variant patterns; share source grammar only where possible. Semantic resolution must produce a **distinct data-class nominal structural pattern product**, not lie that a class is an enum variant.

Conceptually:

```rust
enum ResolvedNominalPattern {
    Variant(/* existing */),
    DataClass {
        owner: DeclarationId,
        projected: Box<ResolvedStructuralProjection>,
        nominal_test: NominalTypeConstraint,
    },
}
```

### 13.2 Matching algorithm

1. Evaluate the subject **once**.
2. Resolve the pattern nominal target to a class declaration and confirm that it is `DataClass`.
3. Check subject nominal compatibility, not duck-shape equality. `Size(x,y)` does not match `Point(x,y)`.
4. Resolve labels against canonical component names, and positional slots against the complete declaration-order component list. `Point(_: x, _: y)` corresponds to `Point(x: x, y: y)`. Argument `_: x` in **call** syntax, by contrast, infers a label from the `x` binding; do not conflate these two grammatical uses of `_`.
5. Enforce component visibility *after* resolving the canonical index; private positions remain occupied and must not shift indices.
6. Project the selected `FieldId` → runtime slot directly. Do not send getters.
7. Recursively evaluate nested subpatterns/bindings under existing match semantics. Respect existing rest/partial-pattern syntax; do not introduce new omission semantics without D03.
8. Class compatibility for patterns may permit a data-class subclass to match a base data-class prefix; equality remains exact-class only.
9. Classify irrefutability using resolved subject type and pattern coverage, **not** source AST shape alone. `let Point(...) = value` is accepted only when guaranteed to match (including arity and nested irrefutability); otherwise use `if let`/`match`.

### 13.3 Tests

- Positional ⇔ labeled equivalence, component order, nested patterns, computed getter exclusion, backing `_x` exclusion, inaccessible label/position, visible label after private slot, mixed visibility, direct storage projection under overridden getter, exact runtime type vs subtype matching, class with same shape but different nominal owner, full/partial arity and rest handling.
- Refutability: static `Point` source allows verified irrefutable `let`; `Object`/`Dynamic` or uncertain types require refutable form; nested literal mismatches remain refutable.
- Exhaustiveness/coverage: existing enum/first-class `data` patterns unchanged; class patterns do not claim impossible closed-world coverage of open subclasses unless already proven by sealed hierarchy metadata.
- Lowering identity: subject evaluated once and getter side-effect counters stay zero during matching.

**Exit gate P8:** `PatternResolution` and executable projection consume the same `StructuralClassProduct`; privacy cannot be bypassed positionally.

---

## 14. P9 — structural equality, stable hashing, rendering

### 14.1 Equality plan

Implement compiler-derived **exact-type nominal structural equality**:

```text
a == b := exact runtime data-class identity agrees
          AND compare canonical component values pairwise with ordinary value ==
```

Use compiler-authorized direct projection for both operands. Do not call custom getters. Maintain short-circuit evaluation and propagate component comparison failure/throw semantics consistently with ordinary value equality. Keep `===` strict identity/sameness. Do not replace `Object#eq` or first-class immutable-data comparison globally.

### 14.2 Hash plan

The equality law is mandatory: `a == b ⇒ a.hash == b.hash`.

- If **all canonical backing storage** is immutable, hash `(exact runtime data-class identity, component hashes in declared order)` with an established stable combining scheme and matching equality protocol. Note that getter-only exposure over mutable storage is **mutable for hash classification**.
- If **any backing storage is mutable**, derive a stable hash from exact runtime class identity alone (or another proven immutable discriminator). Never hash mutable components, and never use per-instance identity digest: two distinct equal instances must hash equal.
- Keep mutable-key admission policy for Map/Set separate. A hash/equality law does **not** imply it is safe for arbitrary mutable instances to be keys when their equality relation can change; preserve existing keyed-collection restrictions unless explicitly redesigned in another program.
- Avoid reentrant heap borrow hazards while hashing nested component objects through ordinary dispatch. Use existing controlled dynamic send machinery where appropriate.

### 14.3 Derivation pair and `toString`

- Neither authored `==` nor `hash`: generate both.
- Both authored: generate neither.
- Exactly one authored: error with exact missing selector; do not pair hand-authored equality with automatically derived hash or vice versa.
- Derive `toString` only when absent; output `ClassName(x: value, y: value)` from canonical labels and backing field projections. The class-name formatting convention for generic/applied classes should match canonical runtime reflection (define test expectations before implementing).
- **Computed getters are absent** from all structural folds. A source getter override that transforms `x` must not change structural pattern match/equality/hash/string output for backing `x`.

### 14.4 Tests

```telos
let a = Point(x: 1, y: 2)
let b = Point(x: 1, y: 2)
// a !== b by identity; a == b; a.hash == b.hash
```

Add mutation tests for stable class-only derived hash; two equal distinct mutable instances, equality changing after mutation while hash remains stable; different exact subclasses not equal; inherited components; immutable nested values; computed getter side effects not executed; source overrides; half-pair compile errors; no-field class; collisions permitted but never law-breaking. Check `Map`/`Set` behavior separately, including key-admission guard. Fuzz/property test equality symmetry and hash implication across constructed values, constrained to component types with valid own equality/hash behavior.

**Exit gate P9:** generated algorithms consume canonical product/slot metadata and preserve equality/hash coherence under mutation; old `build_data_eq/hash/to_string` field scanner is no longer active.

---

## 15. P10 — true `with(**changes)` structural copy/update

### 15.1 Design boundary

The legacy `build_data_with` currently uses `None` as an omission sentinel, expects a full declared label selector, reads getters and re-invokes `.new`. **All four aspects are incompatible** with the required semantics. Retire its behavior, not just its naming.

The repository's labeled-rest selector machinery makes one public generated `with(**changes)` family feasible; the **semantic analyzer** must additionally validate member labels and value types against the canonical structural product.

### 15.2 Required algorithm

```text
check supplied changes: known component labels, no duplicates, type/access policy
  ↓
validate dynamic keys as necessary before allocating
  ↓
allocate fresh unpublished instance of the exact dynamic class
  ↓
copy current canonical structural backing state directly
  ↓
apply replacements by resolved component IDs / physical slots
  ↓
initialize or preserve any additional allowed nonstructural state (D07)
  ↓
compiler-authorized completion/invariant checks
  ↓
publish and return distinct new object
```

**Do not:** read ordinary getters, call setters, call arbitrary public constructors, re-evaluate field defaults, let `None` stand for absence, alias the original instance, return `self`, or copy uninitialized/private Nil storage by accident.

### 15.3 Storage completeness blocker (D07)

The spec says `with(...)` copies **all current structural components** into a newly completed object. It does not explicitly define additional nonstructural fields a data class may declare or acquire through inheritance/reopen. If such state can exist, `with` must not leave it uninitialized or silently discard it. Before implementing, prove one of:

- the canonical component product covers *all* initializable instance fields in the eligible `data class` hierarchy; or
- a separate, verified copy policy covers nonstructural slots (immutable + mutable + ownership semantics); or
- the language temporarily rejects data-class configurations whose state cannot be fully copied safely.

Document which rule is selected. Never assume `components.len() == physical slots` without a semantic proof: inherited layout, static fields and hidden compiler slots may differ.

### 15.4 Semantics of rest labels

- Static known labeled changes: validate at check time with `StructuralClassProduct` and persist component IDs into lowering.
- Dynamic/computed/expanded labels: fail closed on unknown/duplicate labels at runtime, ideally **before** mutation/allocation. Preserve original expression evaluation order exactly once and ordinary rest-family call behavior.
- Generic classes: replacement values must satisfy instantiated component types, not uninstantiated template types.
- `with` is instance-side, returns concrete appropriate `Self`/exact receiver class application; dynamic subclass copies retain actual class where the runtime model permits this. Inspect whether a base-class-generated `with` can safely cover a subclass with extra components; if not, dispatch to the subclass's own generated product.

### 15.5 Tests and acceptance

- Single, multiple, and zero changes; value `None` as a legitimate replacement distinct from absence.
- Default preservation: mutate `retries = 20`, `with(host: "b")` retains 20, not default 3.
- Source getter/setter/constructor counters remain unchanged; completion invariant executes once.
- Distinct identity of result; original unchanged; shallow component identity retained except replaced values.
- Inherited components and subclass-only components; private component access policy consistent with normal labeled arguments and `with`'s public contract.
- Unknown/duplicate/static/dynamic labels, incompatible types, throwing completion, no partially published object, GC stress during copy, open generic specialization and many-field layout.
- On failure, original state remains unchanged and no externally accessible incomplete fresh instance exists.

**Exit gate P10:** copy/update is direct storage-based and certified against D07, with checked static/dynamic labels and true fresh identity.

---

## 16. P11 — class-side cached `const` getter

### 16.1 Keep semantic categories distinct

`const zero -> Point = Point(...)` is **a getter selector** with an evaluation policy. It is **not** a module `const`, a compile-time static evaluator request, a field, or a new selector category. `GetterEvaluation::Cached` belongs in the verified callable signature/executable descriptor. Existing `StaticConstantId` / static evaluation / compile-time materialization must not be enlisted as its implementation.

**Owned anchors:** getter AST/checker, `phalcom-core/src/method/{mod,object}.rs`, class/metaclass storage (or method-owned cache), `vm/{dispatch,send}.rs`, `heap/trace.rs`, class method replacement/world-version logic, runtime errors and LSP hover information.

### 16.2 Cache identity

A cache is scoped semantically by the **effective getter implementation** and **semantic class application/receiver**. The specification requires `Container<Int>` and `Container<String>` not to share a result when their application affects meaning; inherited getters using `Self` must respect receiver class/application as well. Do not cache globally by `selector="zero"`, module, or declaring source name alone.

Proposed internal model:

```rust
struct ConstGetterCacheKey {
    implementation: MethodIdentity,
    receiver_application: AppliedClassIdentity,
}

enum ConstGetterState {
    Unevaluated,
    Evaluating, // detect direct or indirect recursion
    Ready(Value),
}
```

Choose whether the cache lives with the installed method object, in a GC-visible class-specific map, or in another canonical runtime owner. Evaluate memory retention and method replacement carefully. The cache result `Value` must be traced by GC. If the language/runtime has actor/thread/fiber sharing, document whether concurrent first accesses can coevaluate or are serialized; the source contract is **at most one successful evaluation for the semantic cache key**.

### 16.3 Evaluation transitions

```text
Unevaluated -> Evaluating -> Ready(value)       on success
Unevaluated -> Evaluating -> Unevaluated         on throw [D05 proposed]
Evaluating -> deterministic reentrancy error    on cycle [D06 proposed]
Ready(value) -> same cached Value               on later reads
```

A thrown computation must never publish a partial result. A redefined method implementation receives a new cache identity or an explicit invalidation event; do not rely only on the selector string, and do not let stale cache values survive reopened class methods. Ordinary getters still re-evaluate and may return distinct newly allocated objects.

### 16.4 Tests and acceptance

- Cached `Point.zero === Point.zero` true when a reference object is returned; ordinary `Self.origin` getter may allocate afresh.
- Counter demonstrates one successful evaluation despite repeat access; throw/retry transition; recursion produces deterministic diagnostic.
- Generic `C<Int>` vs `C<String>` isolation; inherited getter `Self` receiver specializations; two separate class rows with same name in separate modules; runtime method redefinition; access-control/invocation authorization preserved.
- GC torture: cached heap object remains alive when reachable only through cache, no use-after-free, no untraced pointer, no accidental permanent global root when class/method is collectible.
- Negative grammar: const method, setter, subscript, instance getter, `self(...)` head; module `const` continues to use static evaluator.

**Exit gate P11:** cached getter is observable lazy getter semantics, not optimistically memoized ordinary behavior or a compile-time constant.

---

## 17. P12 — migration, downstream consumers, documentation, and certification

### 17.1 Corpus migration order

Do this as a controlled, reviewable sequence with mechanical transformations only where semantically safe:

1. Introduce new canonical syntax and semantic hooks while old Universe boots behind a **temporary documented migration bridge**.
2. Migrate canonical core/Universe source declarations, including native interface descriptors, class factories, errors, `String` construction, and inherited constructor calls. Rewrite construction `Type.new(...) → Type(...)`, but **never** rewrite ordinary methods merely named `new`.
3. Migrate compiler-generated helper source expressions: error construction in contracts, string interpolation, generated class APIs, attribute instantiation, enum/variant helper code, and all `build_data_*` successors.
4. Migrate packages, examples, test fixtures and benchmarks. Separate source-authored construction from arbitrary `.new` methods using semantic/reference evidence, not blind `sed`.
5. Replace `@construct → @derive(Init)` where the class is intended to stay opaque; replace `@data class → data class` only when new data-class invariants (including hash/copy/inheritance) can be satisfied.
6. Migrate `.ph` docs, published syntax samples, parser highlighting, formatter snippets, reflection/docs generation, LSP completion, hover, semantic tokens, source index, rename, signature help, inlay hints, and workspace symbol presentation.
7. Remove the canonical `Class.new()` signature, runtime compatibility aliases with construction privilege, attribute-expander registrations, legacy constructor-name shortcuts, and source/semantic fallback probes **only after** the corpus is migrated and baseline gates are green.
8. Search for remaining legacy semantic authorities (not merely syntax). Any retained compatibility must have a named owner, expiration gate, and negative test ensuring it cannot perform intrinsic construction in final mode.

### 17.2 LSP and semantic parity

For every new source form, test hover/definition/references/rename/symbols/completion/signature help against the canonical semantic snapshot:

- `Self` declaration-side markers vs expression/type positions;
- `self(...)` aliasing `call(...)` selector;
- `_: x` inferred label and binding reference;
- hidden generated initializer not misleadingly exposed as a source method;
- structural component vs backing field vs generated accessor as separate identities;
- source `y=(_)` override of generated setter;
- `MemberBody::Expression` exact source span;
- `const` getter caching signature and class-side placement;
- generated vs source provenance.

Disallow shadow `name == "new"` inference in editor/advisory paths. Historical constructor inlay bugs show why public factory `Self` result needs language-rule authority independent of initializer tail expression.

### 17.3 Documentation updates

Synchronize normative spec/ADR status, constructor docs, object model, field and selector docs, pattern semantics, `data`/`data class` separation, derived hash laws, GC/cache architecture, source format migration examples, diagnostics catalog and relevant lifecycle records. Old files that deliberately describe **historical** behavior may remain but must be marked non-current.

### 17.4 Exit gate P12

Full Universe and maintained package corpus compile and execute in canonical mode. Full workspace parser/semantic/core/LSP suites, existing verification script, and all targeted stress/property tests pass or unrelated failures are explicitly compared to the baseline. No undocumented legacy constructor privilege remains. Reviewer signs off the complete C1–C34 traceability matrix.

---

## 18. Cross-package invariants and proof obligations

The following must be treated as **global** acceptance contracts, not tests confined to one implementation unit.

### 18.1 Identity and selectors

**I01 — Ordinary call:** `x(args)` and `x.call(args)` have the same dispatch shape, resolution, access, arguments, exception semantics and result.  
**I02 — Class/instance shorthand:** `Self(args)` invokes class-object `call`; `self(args)` invokes instance `call`.  
**I03 — Declaration symmetry:** `self(params)` and `call(params)` declaration heads claim one instance selector.  
**I04 — Constructor factory:** `init Self(params)` claims class-side `call(slots)`; named init claims its own named factory.  
**I05 — Neutral `new`:** no intrinsic allocator, type evidence or constructor origin based on text `new`.  
**I06 — Correct selector notation:** slots encoded as `call(x, y)` or `call(_, _, options)`; types/results do not distinguish method identity.  
**I07 — `_: binding`:** inferred-label call argument remains labeled and references the source binding once.

### 18.2 Initialization and publication

**I08 — Canonical constructor product:** every generated/source verified factory has exactly one canonical hidden initializer pairing.  
**I09 — Same receiver:** parent/default/child initializer segments write the one allocated instance.  
**I10 — No escape:** no initializing receiver enters ordinary source value flow/capture/dispatch/dynamic escape.  
**I11 — Complete state:** all construction-required fields definitely initialized on every successful exit.  
**I12 — Unique immutable write:** at most one initialization of each immutable slot.  
**I13 — Super exactness:** required parent initializer executes canonically once before child-specific state.  
**I14 — Completion:** invariants and completion execute after valid state, before observable publication.  
**I15 — Factory result:** normal return is the fresh allocated instance, independent of initializer body tail.

### 18.3 Structural product and derived behavior

**I16 — Distinct kinds:** immutable `data`, structural `data class`, opaque `class` remain different categories.  
**I17 — Single component authority:** component order/identity/backing/visibility come from one canonical semantic product.  
**I18 — Direct storage:** derived structural operations never call source-overridable getters/setters.  
**I19 — `via` mutability:** writable exposure implies mutable backing; mutable backing does not imply setter.  
**I20 — Fused identity:** one fused backing field, one identity, one default and no duplicate hidden field.  
**I21 — Nominal matching:** subtype-compatible pattern projection and exact-type equality are deliberately distinct.  
**I22 — Privacy stable order:** inaccessible components keep their index; positional extraction cannot bypass visibility.  
**I23 — Derived selector precedence:** authored behavior is not overwritten by generated behavior.  
**I24 — Equality/hash:** exact class comparison and `a == b ⇒ a.hash == b.hash` always hold for valid component protocols.  
**I25 — Stable mutable hash:** mutable structural updates do not alter derived hash.  
**I26 — Copy/update:** `with` returns fresh identity and preserves *current* unspecified values; defaults are not rerun.

### 18.4 Evaluation and tooling

**I27 — Source body fidelity:** `Expression` remains distinguishable and executes with appropriate return/setter semantics.  
**I28 — Implicit setter binding:** `value` only when the assigned-value lane is unnamed.  
**I29 — Cached getter only:** `const` modifies only class-side getters and caches successful result for correct application identity.  
**I30 — Ordinary getters not cached:** ordinary repeated evaluation remains observable.  
**I31 — GC/cache correctness:** every retained `Value` is traced and caches invalidated on implementation replacement.  
**I32 — Incremental correctness:** source/body/layout/superclass edits invalidate exactly their semantic consumers; no stale products.  
**I33 — Parity:** LSP and compiler consume the same semantic identity and provenance, not independent guesses.  
**I34 — Migration closure:** maintained source no longer depends on intrinsic `.new` or removed decorators.

---

## 19. Mandatory test and certification matrix

**Rule:** positive runtime goldens cannot replace semantic-product assertions; negative diagnostic tests cannot replace heap/GC/property tests; a changed parser snapshot cannot prove correct execution.

| Layer | Test lanes (minimum) | Gate |
|---|---|---|
| Lexer/parser/AST | All new heads, body/field `=`, `via` fused and non-fused, `_: x`, setter `value`, `const` restrictions, type `Self`/expression `Self`, `data` vs `data class`; malformed negatives | P1 |
| Selector algebra | exact arity + positional/labeled slots, class/instance side, collisions, `self` vs `call`, primary `init` vs `Self.call` | P2–P3 |
| Constructor semantic products | stable factory/hidden initializer ID, origin, default/parameter mapping, parent target, receiver/result knowledge, fingerprints | P3–P5 |
| Construction runtime | allocate once, same receiver parent/child, default order, subclass layout, invariant completion, early return/throw, native/abstract refusal | P3–P4/P7 |
| Receiver negative flow | every `self` escape/capture/send/conversion/continuation and wrong `super` form rejected; branching and loops | P4 |
| Field lifecycle | field read-before-init, mutable reassignment, exactly-once immutable write, defaults, all normal exits, parent obligations | P4 |
| Derive Init | opaque class only; no extra data-class features; source collision/precedence; defaulted inputs | P5 |
| Structural product | direct and fused `via`, mutability matrix, duplicate field/exposure, component order, visibility, inheritance, incremental invalidation | P6–P7 |
| Structural patterns | nominal not duck, position-label equivalence, private index stable, no getter invocation, irrefutable/refutable contexts, subclass prefix | P8 |
| Equality/hash/render | exact class, `===` unchanged, value equality, stable mutable hash, paired overrides, toString formatting, no user getter calls | P9 |
| `with` | current-value preservation, zero/one/many labels, `None` valid data, no behavior/default invocation, runtime dynamic labels, GC/invariant failure, subclass | P10 |
| Cached getter | one successful evaluation, retry/cycle policy, generic application, inheritance, method replacement, cross-module identity, GC root | P11 |
| Tooling | hover/rename/completion/source refs, generated provenance, implicit binding spans, semantic token parity | P12 |
| Whole system | Universe startup, all stdlib packages, native method table, first-class `data`, old independent collections/math/compiler tests | P12 |

### 19.1 Suggested command families

After confirming workspace command names and harness registration on the implementation branch, run these **examples** with focused filters first, then full suites:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test -p phalcom-ast
cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./scripts/verify.sh
```

If repository baseline already has clippy warnings or unrelated failures, report exact baseline/after comparison; do not suppress new defects or claim unrelated failures are harmless without evidence. Verify corpus bootstrap separately, because a passing AST test never proves the Universe compiles. If the workspace does not support one exact command, document the closest repo-sanctioned equivalent.

### 19.2 Property/metamorphic requirements

- **Selector equivalence:** for generated legal labeled argument packs, `C(args)` and `C.call(args)` select the same callable; shorthand binding labels preserve shape.
- **Pattern equivalence:** for any canonical accessible component prefix, positional structural pattern and corresponding labeled pattern extract the same raw values; explicit overridden getter cannot affect either.
- **Equality/hash:** generate pairs of equal and unequal instances with immutable/mutable components, nested values, inheritance; whenever `==` true, hashes equal; after mutable assignment, *derived* hash unchanged.
- **Copy law:** for each component `c`, `with({k:v}).c == v` if replaced; otherwise equals the object's **current** `c` at copy time (subject to ordinary component equality). Fresh object identity must differ.
- **Constructor field proof:** any accepted successful exit must have required fields initialized, no repeated immutable assignment and no receiver leakage; compare inferred proof state against instrumented runtime for bounded generated control-flow scenarios.
- **Incremental edit law:** editing a field type/default, `via` target, visibility, superclass, getter body and selector should invalidate precisely affected products; stale snapshot must never report success for an invalid program.

### 19.3 Negative safety golden set

Mandatory named fixture cases:

```telos
// Illegal receiver movement while initializing
init Self(_ x: Int) { registry.add(self) }
init Self(_ x: Int) { let a = self }
init Self(_ x: Int) { let f = || self }
init Self(_ x: Int) { self.validate() }
init Self(_ x: Int) { return self }
init Self(_ x: Int) { return 3 }

// Wrong source selectors / binding
class Duplicate {
    call(_ n: Int) = n
    self(_ n: Int) = n
}

// Incorrect data-class backing reuse
// Add D01 ruling fixture for two structural components aliasing one FieldId.

// Incorrect half-pair override
// data class has only == or only hash explicitly.

// `const` non-getter / non-class-side
// const self(...), const x=(...), const [_ n] ...
```

Every expected rejection should carry a stable diagnostic ID, precise primary span, explanation, and actionable rewrite hint. Avoid fragile substring-only tests when a structured diagnostic facility exists.

### 19.4 Runtime safety and adversarial stress

Run stress tests for: deep inheritance, many component fields approaching current slot/selector encoding bounds; nested generic application; reentrant `const`; map/set interaction with mutable structural objects; GC during `with` and cached getter evaluation; thrown invariants in completion; throwing component equality/hash/toString; redefinitions/inline-cache invalidation; first-class `data` value representation; and packages depending on old construction idioms. Any exception during a new object path must leave no externally reachable unpublished instance.

---

## 20. Implementation pitfalls and mandatory reviewer probes

| Risk | What can go wrong | Reviewer probe / required response |
|---|---|---|
| R01 `new` ghost semantics | `Class.new()` still special in checker, editor or VM | Search canonical signature, constructor type evidence, `name == "new"`, fallback dispatch; prove `new` arbitrary return works. |
| R02 dual constructor authority | parser/compiler synthesizes factories independently of semantic product | Product-to-lowering trace; no orphan/unused product; stable IDs under edits. |
| R03 `self` escape missed | method/field helper rewrites bypass receiver-phase check | Enumerate every `Expr` and statement form that can carry/capture receiver; add exhaustive validation visitor and negative tests. |
| R04 parent default order | defaults run before `super` due to AST emission or rewrite order | Inspect actual emitted execution order, not just source syntax. |
| R05 invariant side effects | invariant wrapper performs regular self dispatch before completion | Trace `weave_invariant_checks` after refactor; invariant called from compiler-only completion. |
| R06 structural getters | equality/pattern/hash reads transformed `x` getter rather than `_x` backing | Instrument getter side effects; verify zero sends and direct slot projection. |
| R07 wrong mutability | getter-only `via mut` wrongly generates setter, or writable via fails to set field mutable | Four-way mutability matrix at both product and runtime. |
| R08 aliasing structural slots | `with` replacement of one alias silently changes unspecified alias | D01 rule and negative fixture. |
| R09 mutable hash | current field hash changes after mutation / instance identity violates equal-key law | Mutation, distinct equal objects, class identity hash contract tests. |
| R10 `with` re-creation | public constructor/default/getter/setter called during copy | Instrument every behavior path; counters must stay unchanged. |
| R11 hidden state lost | `with` copies components but not required nonstructural slots | D07 proof; reject unsupported layouts. |
| R12 `const` = static constant | runtime expression evaluated at compile time or static constant slot used | Check semantic category/bytecode; compare module const and class getter runtime. |
| R13 cached Value untraced | GC collects cached object | Force full GC with cached-only reachability and verify handle alive. |
| R14 generic cache collision | `Container<Int>` returns `Container<String>` cached result | Class-application cache identity tests. |
| R15 source precedence drift | generator silently overwrites source setter or equality half-pair | Canonical selector occupancy assertions and diagnostics. |
| R16 frozen snapshot | field/via edits don't propagate product invalidation | Edit/recheck tests across parent→child and module boundaries. |
| R17 LSP shadow semantics | source indexing guesses constructor from `new` or stores synthetic parameter span | Verify source-index/hover/rename parity from canonical product. |
| R18 regression to immutable `data` | `data class` parser shadows `data` declaration or routes runtime into DataObject | Separate parse and execution fixtures for both kinds. |

---

## 21. Handoff contract for every implementer unit

Each implementation handoff must start with:

```text
Unit ID and purpose:
Base commit / branch / worktree:
Normative specification sections + invariants:
Current code symbols actually inspected:
Owned write-set and forbidden overlap:
Canonical products consumed/produced:
Semantic decisions already approved:
Known baseline failures:
Focused tests to add before implementation:
```

Then execute this bounded loop:

1. **Inspect** real code on the pinned branch, re-evaluate assumptions from this plan, and note changed anchors. Do not paste speculative symbols as if current.
2. **State invariants** and negative cases before editing.
3. **Implement smallest coherent vertical slice**, using canonical product/identity APIs; no backdoor AST or source-name authority.
4. **Run focused tests** and inspect every failure; fix underlying semantics instead of weakening diagnostic assertions.
5. **Run adjacent regressions** (parser + semantic + compiler/runtime) before treating any unit complete.
6. **Inspect GC, incremental fingerprints, access control, provenance and LSP implications** if applicable to the touched code.
7. **Commit only verifiable state**. A WIP commit must say WIP and preserve an explicit unresolved list.
8. **Write handoff evidence**: exact tests and outcomes, changed APIs, decisions, rejected alternatives, remaining risks, next agent's bounded next action.

### 21.1 Mandatory evidence format

```markdown
### Implementer evidence — Pn
- Base SHA / result SHA:
- Worktree status:
- Files/symbols changed:
- Semantic product additions/changes:
- Invariants proven:
- Tests: exact commands; passed/failed/skipped counts:
- Diagnostic goldens:
- Runtime/GC/incremental tests:
- Discovered pre-existing failures:
- Scope deviations and decision IDs:
- Open issues and known unsound cases:
- Reviewer disposition: PASS / BLOCKED / CONDITIONAL
```

### 21.2 Reviewer independence

Highest-risk changes (P3, P4, P6, P7, P10) require a reviewer to independently inspect the **actual product-producing and product-consuming code**, then challenge with a negative or counterexample test. A code review that checks formatting and green happy-path goldens only is insufficient. Explicitly look for semantic authority leaks, reentrancy, ghost `new`, getter-backed structural reads, and unpublished receiver escape.

---

## 22. Recommended integration milestones

| Milestone | Packages | What reviewers can meaningfully certify |
|---|---|---|
| M1 Surface-ready | P0–P1 | New source grammar and AST fidelity, no runtime claims |
| M2 Invocation-ready | P2 | Ordinary callable class objects, class/instance shorthand selector parity; migration bridge inventoried |
| M3 Constructor-safe | P3–P5 | Product-authoritative constructors, parent/field proofs, confinement, completion, bounded derive |
| M4 Structural-authority | P6–P7 | Complete component identity/field relation, data-class construction and hierarchy |
| M5 Structural-behavior | P8–P10 | Matching, equality/hash/render, true copy/update from same product |
| M6 Lazy-getter | P11 | Successful runtime caching, cache identity, GC roots and invalidation |
| M7 Canonical closure | P12 | Corpus migration, removed old privileges, tooling parity and workspace regression gate |

**Integration policy:** Do not merge a structural consumer from M5 against an unstable or provisional M4 product signature. All three consumer packages must observe one published product schema.

---

## 23. Final checklist — specification C1–C34 traceability

This table is the formal audit register for closure. A row requires both a code evidence reference and passing positive/negative test identifiers in the final handoff; a checkbox is not proof by itself.

| Spec invariant | Principal owner | Proof obligation |
|---|---|---|
| C1 ordinary call | P2 | `receiver(...)` ordinary `call` selection |
| C2 class `Self(...)` | P2 | `Self.call` equivalence and application identity |
| C3 instance `self(...)` | P1–P2 | ordinary `self.call` equivalence |
| C4 `self(...)` declaration | P1–P2 | same instance selector/collision as `call(...)` |
| C5 selector notation | P1–P3 | canonical comma-separated slot encoding |
| C6 `new` neutrality | P2/P12 | no semantic privilege, final absence of old rule |
| C7 publication boundary | P3–P4 | unpublished until validated completion |
| C8 receiver confinement | P4 | all negative escape/dispatch tests rejected |
| C9 full initialization | P4 | all successful exits definitely initialized |
| C10 immutable uniqueness | P4 | double immutable writes rejected |
| C11 parent formation | P3–P4 | exact once/first parent initializer proof |
| C12 `_: x` inference | P1–P3 | label preserved, not positional |
| C13 canonical derivation | P3/P5–P10 | product authority across compiler/runtime |
| C14 source precedence | P5/P7/P9 | generated selector occupancy never overwrites source |
| C15 structural independence | P6/P8–P10 | getters not structural state |
| C16 nominal patterns | P8 | nominal class type tested |
| C17 exposed patterns | P6/P8 | component names, not backing names |
| C18 positional equivalence | P8 | identical canonical projections |
| C19 visibility | P8 | private positional extraction rejected |
| C20 stable order | P6/P8 | private positions not renumbered |
| C21 equality/hash coherence | P9 | property law even for equal distinct instances |
| C22 stable mutable hash | P9 | no change after mutable rebinding |
| C23 true copy/update | P10 | unspecified current components preserved |
| C24 `data` separation | P1/P7/P12 | `data` runtime/regression unmodified |
| C25 writable implies storage mut | P6 | `_field` canonical mutable field signature |
| C26 storage mut not setter | P6–P7 | `name via mut field` read-only generated exposure |
| C27 redundant mut | P1/P6 | accepted equivalent semantics |
| C28 fused identity | P6 | exactly one `FieldId` |
| C29 expression-body equivalence | P1/P12 | results, contracts and setter semantics consistent |
| C30 implicit setter `value` | P1/P12 | correct binding/source provenance |
| C31 getter-only `const` | P1/P11 | all invalid const member kinds rejected |
| C32 success cache | P11 | result stored once per class application |
| C33 `const` remains getter | P11 | no module-binding/static const corruption |
| C34 ordinary evaluation | P1/P11 | normal getter/method not implicitly memoized |

---

## 24. Stop conditions and non-negotiable blockers

Stop a package and escalate its scope/design rather than landing a shortcut when any of the following happens:

1. A new factory can succeed without a canonical `ConstructorInitializationProduct`, or generated initialization can differ from the semantic field product.
2. A source-level initializing `self` alias/call/capture is accepted, including through a closure/indirect AST form not covered by the checker.
3. The same `FieldId` has conflicting mutability facts published by different paths, or component order differs between pattern and `toString`.
4. Structural matching/equality/hash/with executes an arbitrary source getter or setter to read canonical state.
5. Derived mutable hash depends on mutable component content or object-handle identity under value equality.
6. `with` has no proof that every mandatory field in the fresh object is safely constructed/copied, including nonstructural state.
7. A cached getter's `Value` root is untraced, or cache keys conflate distinct generic class applications.
8. A migration patch preserves implicit `new` construction “for convenience” after the compatibility cutover.
9. A passing fixture only establishes parser acceptance while semantic products/runtime behavior remain unverified.
10. A spec-unspecified choice becomes silently normative without a recorded ruling.

The correct response to a stop condition is a bounded root-cause investigation, a small failing test, and either a sound repair or an explicit blocked handoff; never silently relax the specification.

---

## 25. Expected implementation outcome

The intended end state has a deliberately small surface vocabulary with a rigorous compiler substrate:

```telos
// Ordinary opaque class, derived field construction only.
@derive(Init)
class Coordinate {
    _x: Int
    _y: Int
    x = _x
    y = _y
}

// Structural ordinary object, storage relationships are compiler-known.
data class Point {
    mut x via _x: Int
    mut y via _y: Int

    init Self.origin() {
        _x = 0
        _y = 0
    }

    const zero -> Point = Point(x: 0, y: 0)
    magnitude -> Float = sqrt(x * x + y * y)
}

// Callable instance: one canonical call selector.
class Scale {
    _factor: Int
    init Self(_ factor: Int) { _factor = factor }
    self(_ x: Int) -> Int = x * _factor
}

let p = Point(x: 3, y: 4)
let Point(_: x, _: y) = p
let changed = p.with(x: 5)
let scale = Scale(2)
let result = scale(10)
```

**Authoritative architecture:** selector identity governs `call`; canonical constructor products govern initialization; canonical structural products govern data-class state; runtime completion guards publication; `via` governs storage/exposure relationships; and `const` is a cached getter evaluation policy. Ordinary dynamic dispatch and nominal object identity remain intact. The repository's first-class immutable `data` category remains undisturbed.

---

## Appendix A — prioritized source inspection itinerary

Before P1, read in order:

1. `phalcom-ast/src/{lexer,token,ast,parser,selector}.rs` — class member grammar, pattern parser, `MemberBody`, `ClassMember`, `DelegatedAccessorDef`, parameter labels.
2. `phalcom-common/src/selector.rs` — selector shape and encoder/decoder invariants.
3. `phalcom-semantic/src/signature.rs` and `session.rs` — current constructor product creation, defaulting and query publication.
4. `phalcom-semantic/src/checker/{declaration_signature,declaration,body,expression,context,field_lifecycle,pattern}.rs` — signature side, direct-super recognition, receiver field proof, nominal patterns.
5. `phalcom-semantic/src/db/{fingerprint,query,product}.rs`, `semantic_shard.rs`, source index — stable identity and incremental correctness.
6. `phalcom-core/src/compiler/attributes.rs`, `compiler/lib/{class_decl,expr,patterns}.rs` — synthesized constructors, legacy data behavior, default/parent ordering, body compilation.
7. `phalcom-core/src/modules/semantic_lowering.rs` — checked products consumed by codegen.
8. `phalcom-core/src/{primitive/class,primitive/object,heap/class,heap/trace,vm/dispatch,vm/send}.rs` — internal allocation, identity/hash, slot layout, rest dispatch, cache roots.
9. Canonical `.ph` source under `phalcom-core/core/universe/` and current packages — actual migration and bootstrap shape.
10. `phalcom-ast/tests`, `phalcom-semantic/tests/semantic`, `phalcom-core/tests/core`, `phalcom-lsp/tests` — harness conventions and regression boundaries.

## Appendix B — explicit distinctions the implementation must preserve

| Spelling / concept | This feature's meaning | Do **not** conflate with |
|---|---|---|
| `Self.foo` declaration | class-side placement | `Self` self type / expression variable |
| `Self(...)` expression | ordinary class-object `call` | intrinsic allocation |
| `self(...)` declaration | instance `call` selector | a selector literally named `self` |
| `init Self(...)` | verified primary construction | ordinary method returning arbitrary object |
| `Self.new(...)` | ordinary source method | historical magical `Class.new()` |
| `_: x` in call arguments | infer label `x` | positional argument slot `_` |
| `Point(_: p)` in pattern | next exposed component | raw physical slot/bare backing field |
| `@derive(Init)` | derived construction, opaque class | data-class structural derivation |
| `data class` | structural ordinary object | first-class immutable `data` |
| `mut count via _count` | writable exposure, mutable backing | `count via mut _count` getter-only |
| `member = expr` | source expression body | class field initializer |
| `const zero = expr` | cached class-side getter | module compile-time `const` binding |
| mutable structural hash | stable class-compatible hash | content hash that changes on mutation |
| `with(x: v)` | direct fresh copy/update | public constructor + rerun defaults |

## Appendix C — reproducibility declaration

This plan used the attached 56-section specification and on-the-ground GitHub source inspection. It is **not** an execution report: no repository files were modified, and no workspace tests were run to prepare it. Verify exact source anchors and baseline status at the implementer's branch SHA, particularly if `feat/telos-collection-upgrade`, `feat/complex-math`, or other concurrent branches are being merged. All commands and identifiers in this plan are implementation guidance, not claims of green certification.
