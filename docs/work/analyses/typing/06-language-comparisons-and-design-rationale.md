# Phalcom Language Comparisons and Design Rationale

**Date:** 2026-08-22
**Status:** Ratified engineering decision audit; precedents are rationale, not imported semantics
**Authority:** rationale for decisions fixed by specifications 01–05; precedents do not override live Phalcom semantics
**Depends on:** [01 — Implementation Architecture](01-implementation-architecture.md), [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md), [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md), [04 — User-Facing Type Syntax and Lowering](04-user-facing-type-syntax-and-lowering.md), and [05 — Advanced Kinds, Constraints, Effects, and Proofs](05-advanced-kinds-constraints-effects-and-proofs.md)
**Owners:** language design, semantic analysis, compiler/runtime boundary, metadata/reflection, proof platform
**Non-goals:** language survey, compatibility promise with another language, runtime object-model rewrite, or proof-backend selection

## 1. Audit method

This document tests Phalcom decisions against mature design alternatives. Every comparison answers:

1. problem;
2. options;
3. relevant precedent;
4. selected Phalcom decision;
5. transfer;
6. rejection;
7. runtime/tooling implication;
8. future flexibility;
9. failure mode avoided.

Precedents are design evidence, not authority. Phalcom's live object model and the normative specifications in this series decide behavior.

For Pyrefly, primary local evidence is the pinned [transfer package](../pyrefly-transfer/README.md), especially [type representation and canonicalization](../pyrefly-transfer/04-type-representation-equality-and-canonicalization.md), [answer publication](../pyrefly-transfer/05-answer-tables-query-cells-and-cycle-publication.md), [invalidation](../pyrefly-transfer/06-dependency-graph-and-incremental-invalidation.md), [Phalcom transfer architecture](../pyrefly-transfer/11-phalcom-transfer-architecture-and-type-philosophy.md), and [phased implementation](../pyrefly-transfer/12-phased-implementation-specification.md). These dossiers are labeled **Pyrefly architectural transfer**. No external research was needed for this audit.

## 2. Haskell and ML: kinds, constructors, inference, and correctness

### Problem

Phalcom needs higher-kinded constructors and useful inference without turning every runtime value or language feature into a dependent type-level computation.

### Options

- atomic `Type` plus arrow kinds;
- universe hierarchy or `Type :: Type`;
- dependent kinds indexed by values;
- monomorphic explicit annotations only;
- whole-program principal-type inference;
- local/bidirectional inference with declared interfaces.

### Precedent

Haskell/ML traditions establish the utility of constructor kinds, quantified parameters, substitution, unification, algebraic inference, and partial-correctness reasoning. They also show that increasingly expressive kind/type computation changes elaboration, termination, diagnostics, and implementation trust.

### Selected Phalcom decision

**Ratified/normative design.** Use atomic `Type`, arrow kinds, stable owner-qualified type parameters, prenex kind schemes, solver-local variables, and local/bidirectional inference around declared module/callable surfaces. Ordinary contracts promise partial correctness; totality is explicit and separately proven.

### Transfer

- constructor versus proper-type distinction;
- kind checking before type relation checking;
- capture-avoiding substitution;
- occurs checks and zonking;
- prenex generalization and fresh instantiation;
- explicit logical distinction between partial and total correctness.

### Reject

- `Type :: Type`;
- universe polymorphism;
- dependent escalation;
- arbitrary type/kind-level evaluation;
- a global inference algorithm that owns module/public API design;
- treating a bottom return type as termination evidence.

### Runtime/tooling implications

Kinds and inference remain compiler-owned semantic data. No kind evaluation sends messages or constructs VM objects. Interfaces expose closed kind schemes; diagnostics can name unsolved variables before publication. Query budgets make inference failure explicit.

### Future flexibility

Prenex schemes leave room for higher-kinded libraries and later constrained generalization. They do not preclude an explicitly ratified higher-rank feature, but current metadata and reflection need not encode one.

### Failure mode avoided

Expressiveness does not force undecidable elaboration, hidden runtime evaluation, escaping metavariables, or an unsound `Type :: Type` loop.

## 3. OCaml: rows and restriction boundaries

### Problem

Phalcom wants open structural records and may later need variant/effect rows. A universal row representation is tempting but can erase different semantic rules.

### Options

- closed records only;
- one universal labeled row kind;
- distinct record, variant, and effect row domains;
- nominal object layouts treated as rows;
- unrestricted polymorphic generalization across effects/mutation;
- conservative generalization at stable boundaries.

### Precedent

OCaml demonstrates practical open object/variant row reasoning and the importance of separating inference/generalization from effectful or mutable expressions. Its row machinery shows reusable algorithms without implying one semantic domain.

### Selected Phalcom decision

**Ratified/normative design.** Add `RecordRow` with sorted known fields and `RecordTail::{Closed, Parameter}`. Keep variant/effect rows distinct if introduced. Record rows describe structural record values, never nominal class layouts. Generalization occurs at defined declaration boundaries after effects/constraints are known.

### Transfer

- explicit open versus closed tails;
- field subtraction/unification with occurs checks;
- stable row-variable identity;
- caution around polymorphic generalization and mutable/effectful facts.

### Reject

- a universal public `Row` kind;
- cross-domain row IDs;
- structural exposure of class instance layout;
- generalizing incomplete mutable/effectful facts as universally reusable.

### Runtime/tooling implications

Record rows live in semantic and metadata domains. Runtime record representation need not change. Reflection labels a record row as such; it cannot be passed where an effect row is expected. Diagnostics explain domain and tail mismatch.

### Future flexibility

A typed generic row-solver utility may be extracted after separate domain wrappers and policies exist. Variant/effect syntax can evolve independently.

### Failure mode avoided

One convenient enum does not permit record fields to satisfy effects, nominal layouts to leak as structural contracts, or mutation to make generalized signatures unsound.

## 4. Scala: variance, HKTs, and ambient machinery

### Problem

Phalcom needs concise declaration-site variance and higher-kinded parameters without importing implicit resolution or type-directed runtime behavior.

### Options

- `+T`/`-T`/`T` declaration-site variance;
- keyword variance (`out`/`in`);
- use-site variance only;
- invariant generics only;
- explicit constraints/context;
- ambient implicit/given/type-class search affecting calls and construction.

### Precedent

Scala demonstrates compact sign variance, higher-kinded abstractions, F-bounds, and rich constraints. It also demonstrates the complexity cost of type members, path-dependent types, implicit search, and compiler behavior that can become difficult to predict.

### Selected Phalcom decision

**Ratified/normative design.** Use `+T`, `-T`, and `T`. Validate occurrence positions compositionally. Support constructor kinds and F-bounds behind cycle/budget checks. Keep runtime selector identity and ordinary dispatch independent of inferred type arguments. Reflection/construction requires explicit `TypingContext`.

### Transfer

- sign variance vocabulary;
- declaration-site validation;
- higher-kinded constructor parameter patterns;
- explicit bounds and F-bound caution.

### Reject

- `out`/`in` spellings;
- ambient implicit/given resolution;
- path-dependent/runtime-dependent type identity;
- type-directed overload/dispatch changes;
- type members in this phase.

### Runtime/tooling implications

`+`/`-` are contextual binder markers, not selector text and not overridable runtime messages. IDE displays variance from formal snapshots. Cache dependencies name bound/variance surfaces rather than hidden implicit environments.

### Future flexibility

Explicit protocol/evidence parameters could later model reusable capabilities without ambient search. HKT support remains available through kind arrows.

### Failure mode avoided

Inference cannot silently change which runtime method runs, which constructor is selected, or which hidden evidence enters a call.

## 5. Rust: identities, bounds, evidence, and trust

### Problem

Semantic IDs, generic constraints, and proof evidence require strong ownership and trust boundaries. Rust offers useful implementation discipline but a different language/runtime contract.

### Options

- parameter identity from text;
- parameter identity from owner/index;
- explicit normalized bounds;
- ambient conformance lookup;
- proof/trait evidence represented explicitly;
- clone specialized runtime classes per instantiation;
- preserve erased ordinary classes with semantic metadata.

### Precedent

Rust demonstrates owner-scoped generic identity, explicit bounds, trait obligations/evidence, monomorphization, lifetime/resource discipline, and the value of making unsafe/trusted boundaries visible.

### Selected Phalcom decision

**Ratified/normative design.** Stable binders use semantic owner plus index. Bounds and proof trust are explicit. Solver-local variables cannot escape. Runtime ordinary classes remain ordinary class objects; generic applications do not clone class/metaclass hierarchies. Rust borrow/lifetime semantics are not inferred as Phalcom defaults.

### Transfer

- owner-qualified identity;
- explicit obligation/result boundaries;
- separation of safe claims from trusted/unsafe assumptions;
- deterministic metadata and reproducible evidence;
- rigorous publishability validation.

### Reject

- Rust's borrow checker as Phalcom's default effect/resource model;
- monomorphized class identity as runtime generic semantics;
- trait coherence/orphan rules before Phalcom protocol rules are designed;
- compile-time evidence silently granting runtime authority.

### Runtime/tooling implications

Generic specialization can remain an optimizer detail only if externally observable class identity and dispatch stay unchanged. Proof UIs distinguish kernel-checked, backend-trusted, and assumed evidence. APIs use different ID newtypes for persistent and solver-local domains.

### Future flexibility

Phalcom may later add ownership/resource analyses as explicit capability/effect features. Protocol coherence can be designed against open-world reflection rather than copied wholesale.

### Failure mode avoided

Textual binder capture, store-ID leakage, false proof authority, and generic instantiation changing runtime identity are excluded structurally.

## 6. Swift: metatypes and reflection stratification

### Problem

Phalcom must describe types and class objects without asserting that static type forms and runtime metatypes are the same entity.

### Options

- every static type is a runtime class object;
- wrap every class object in a separate type object;
- preserve nominal class objects and add synthetic descriptors only for non-nominal forms;
- per-value retained generic tokens;
- explicit contextual reflection.

### Precedent

Swift's metatype/reflection distinctions show that a language must state whether it refers to an instance type, a metatype value, or reflective metadata. Reified generic metadata also exposes code-size, identity, and runtime availability trade-offs.

### Selected Phalcom decision

**Ratified/normative design.** `value.class`, `value : T`, `T :: K`, and `value ⇝ T` remain distinct. Reifying nominal `Int` returns existing class object `Int`. Synthetic unions/tuples/callables/rows use immutable descriptor objects. Static `TypeForm` is a semantic/behavioral role, not the superclass of class objects.

### Transfer

- explicit metatype-versus-instance-type reasoning;
- reflection metadata as a versioned runtime boundary;
- caution around erased versus reified generic information.

### Reject

- collapsing every semantic type into a class object;
- wrapping nominal classes solely for typing uniformity;
- claiming erased per-instance information is recoverable;
- generic metadata changing ordinary class identity.

### Runtime/tooling implications

Metadata DAG and loader registry reify only retained facts. `value.class` remains truthful runtime identity. Reflection queries return explicit known/opaque/missing results and require capabilities where needed.

### Future flexibility

Metadata profiles can retain more generic context for debugging or explicit construction without changing base object representation.

### Failure mode avoided

Reflection never lies that a runtime class object encodes a complete static application, and static type equality never becomes pointer equality over wrappers.

## 7. Smalltalk: objects, messages, DNU, and live reflection

### Problem

Static typing must coexist with Phalcom's message-oriented runtime, class/metaclass tower, DNU, and live reflection.

### Options

- replace message dispatch with type-directed static dispatch;
- model everything as dynamic and give up formal facts;
- preserve runtime semantics while adding explicit compiler-owned approximations and query outcomes;
- inject ambient type application into class-side calls.

### Precedent

Smalltalk demonstrates uniform runtime objects, classes as objects, metaclasses, message sends, `doesNotUnderstand`, and live reflection. Its strength is runtime coherence; static analysis must not counterfeit a different object model.

### Selected Phalcom decision

**Ratified/normative design.** Preserve class/metaclass identity, selector identity, ordinary message lookup, DNU, and reflection. Static analysis records nominal/class-object forms, dispatch summaries, and dynamic/open-world outcomes. Explicit `TypingContext` mediates reflective type construction; no `Type.currentApplication`.

### Transfer

- runtime class/metaclass model remains authority;
- live reflective objects remain ordinary objects;
- dynamic dispatch and DNU remain explicit open-world boundaries;
- selectors stay receiver/type-independent identities.

### Reject

- type-directed selector rewriting;
- class specialization visible as new class objects;
- static closed-world claims across DNU/reflection;
- ambient generic context on fibers or call stacks.

### Runtime/tooling implications

Compiler diagnostics state when dispatch is known, dynamic, unresolved, or budget-limited. Runtime reflection uses capability-checked methods and immutable descriptors without inserting a superclass into existing class hierarchy.

### Future flexibility

Richer dispatch summaries and protocol facts can improve tooling while preserving fallback behavior. Explicit contexts can support generic construction where metadata permits it.

### Failure mode avoided

Typing cannot subtly fork runtime semantics, break metaclass invariants, make selector caches type-dependent, or mishandle reentrancy/fibers through ambient context.

## 8. TypeScript: structural convenience and relation discipline

### Problem

Structural records, gradual boundaries, and editor responsiveness tempt one permissive compatibility relation and broad `any` propagation.

### Options

- collapse subtype/assignable/consistent/equivalent into compatibility;
- expose booleans only;
- use explicit relation kinds and terminal results;
- make `any` both top and bottom-like fallback;
- separate `Any`, `Dynamic`, unknown, missing, invalid, and operational failure.

### Precedent

TypeScript demonstrates practical structural typing and productive editor feedback over dynamic code. It also makes visible the cost of intentionally unsound `any`, erasure, and compatibility rules that can blur semantic guarantees.

### Selected Phalcom decision

**Ratified/normative design.** Support structural records under explicit capability/relation rules. Keep equivalence, subtyping, assignability, and consistency separate with reasoned outcomes. `Dynamic` is explicit; `Any` remains a gated proper top type; unknown/missing/invalid/budget/cancel are not types and not success.

### Transfer

- ergonomic structural descriptions;
- flow-sensitive editor facts;
- fast incremental feedback;
- explicit escape boundary where dynamic interoperability is intended.

### Reject

- permissive `any` propagation;
- a single compatibility predicate;
- presenting erased facts as runtime reification;
- editor convenience overriding compiler semantics.

### Runtime/tooling implications

LSP displays partial results/status rather than fabricating types. Dynamic operations preserve runtime checks and open effects. Metadata does not serialize epistemic failure as a canonical type node.

### Future flexibility

An `Any` top may be admitted after lattice/diagnostic policy ratification. Intersections and protocols may be added behind distinct normalization/coherence gates.

### Failure mode avoided

One dynamic annotation or budget limit cannot silently validate arbitrary assignments, prove a contract, or mislead reflection.

## 9. Python and Pyrefly: architecture without Python semantics

### Problem

Phalcom needs responsive whole-project analysis, recursive solving, invalidation, and snapshots. Pyrefly offers current implementation evidence but serves Python's semantics.

### Options

- copy Python type rules and data structures;
- retain whole-workspace fresh recomputation;
- transfer architectural seams and adapt semantic products;
- let LSP own an independent checker/cache graph;
- make compiler semantic DB sole formal owner.

### Precedent

**Pyrefly architectural transfer.** Local dossiers document dense identities, staged module products, calculation/answer cells, SCC publication, canonical type operations, dependency keys, reverse invalidation, immutable generation products, cancellation, observability, and explicit complexity caps.

### Selected Phalcom decision

**Ratified/normative design.** Transfer staged queries, stable/dense identities, bounded worklists, SCC-local fixed points, dependency fingerprints, reverse invalidation, immutable snapshots, stale-result rejection, and measurement discipline into `phalcom-semantic`. Adapt them to selectors, class/instance sides, native metadata, modules, reflection, effects, and proof facts. Compiler DB owns formal facts; LSP adapts published snapshots.

### Transfer

- query keys with stamped dependencies;
- canonical store boundaries without assuming pointer identity;
- cycle-aware answer state and deterministic batch publication;
- existence/type/metadata dependency distinctions;
- cancellation and bounded normalization/relations;
- differential clean-versus-incremental testing.

### Reject

- Python `Any`/unknown behavior;
- Python protocols, descriptors, attributes, overload selection, and import fallbacks;
- closed-world assumptions for Phalcom messages/reflection;
- unsafe pointer-tagged publication before evidence and profiling;
- LSP-owned formal linking/checking.

### Runtime/tooling implications

`SemanticDb` publishes one snapshot used by compiler, CLI, REPL, and LSP. `ValueShape` remains advisory. Dynamic/DNU/native facts are modeled explicitly. Failed projects/interfaces/imports/links become module states and diagnostics, never skipped files.

### Future flexibility

Safe query cells can later gain parallel/atomic optimizations after profiling. Dependency keys can become finer without changing query result semantics.

### Failure mode avoided

Performance work does not import Python semantics, split formal authority, deadlock on recursive calculations, publish stale diagnostics, or turn complexity widening into explicit `Dynamic`.

## 10. Java, Kotlin, and C#: erasure, variance, and reification

### Problem

Generic metadata can be erased, partly retained, reified at call sites, or stored per object. Each choice affects object layout, class identity, reflection, and interoperability.

### Options

- fully erase generic applications;
- retain tokens per instance;
- specialize runtime classes;
- reify selected context explicitly;
- declaration-site or use-site variance;
- strong global descriptor caching.

### Precedent

Java demonstrates erasure and reflective limits; Kotlin demonstrates declaration/use-site variance plus inline reified operations; C# demonstrates runtime generic metadata and reflection. Together they show no single reification choice is free.

### Selected Phalcom decision

**Ratified/normative design.** Ordinary objects/classes remain unspecialized. Compiler metadata retains generic semantic surfaces according to profile. Explicit `TypingContext` carries a requested application when reflection/construction needs it. Nominal reification returns existing class objects; synthetic descriptor identity is VM-local and weakly canonicalized.

### Transfer

- explicit declaration-site variance;
- versioned reflection metadata;
- call/context-scoped reification where intentionally requested;
- truthful APIs when generic information is unavailable.

### Reject

- mandatory per-instance generic tokens;
- specialized runtime class cloning;
- pretending erasure can recover exact applications;
- one immortal global cache of every descriptor ever requested.

### Runtime/tooling implications

Object layout and class identity remain stable. Metadata profile controls cost. Cache ownership belongs to bounded loader/VM contexts; synthetic descriptors use weak references and explicit world/schema validity.

### Future flexibility

Selected APIs may accept richer explicit contexts or specialized optimization internally if observational identity remains unchanged.

### Failure mode avoided

Typing does not impose universal object bloat, cache leaks, class explosion, or false reflective precision.

## 11. Proof-oriented languages: evidence without dependent APIs

### Problem

Contracts need static evidence, counterexamples, caching, and trust while Phalcom remains an ordinary object/message language.

### Options

- runtime guards only;
- ephemeral solver verdicts;
- persistent fingerprinted evidence;
- proof terms and `Prop` in user types;
- kernel-checked certificates, trusted-backend attestations, and explicit assumptions.

### Precedent

Refinement/proof systems demonstrate the value of separating specification, verification conditions, solver interaction, certificates, assumptions, and totality. They also show that proof terms/dependent programming reshape the language and trusted computing base.

### Selected Phalcom decision

**Ratified/normative design.** Lower one canonical contract IR to runtime guards and proof IR. Generate explicit VCs. Persist results keyed by canonical VC, assumptions, interfaces, semantic model, backend, and kernel. Distinguish `Proven`, `Disproven`, `Unknown`, cancel, budget, and internal failure. Preserve `KernelChecked`, `TrustedBackend`, and `Assumed` trust.

### Transfer

- explicit proof obligations;
- counterexample models;
- certificate/trust distinction;
- partial versus total correctness;
- persistent reproducible evidence and invalidation.

### Reject

- `Prop` as a new kind/type domain in this phase;
- proof terms in ordinary APIs;
- Curry–Howard/dependent programming;
- runtime object identity granting proof authority;
- stripped runtime checks counting as proof.

### Runtime/tooling implications

Proof artifacts are immutable metadata values with bounded decoding. CLI/LSP report trust and unknown reasons. Runtime can reflect evidence summaries without becoming proof checker unless an explicit kernel API checks a certificate.

### Future flexibility

Logic theories and certificate formats can expand behind versioned backend/kernel contracts. The ordinary language need not commit to dependent types.

### Failure mode avoided

Solver success cannot become unaudited truth, stale evidence cannot survive semantic changes, and proof ambitions do not silently redesign every type/runtime API.

## 12. Direct alternative decisions

### 12.1 `Type` protocol versus atomic kind plus `TypeForm`

| Option | Benefit | Cost |
|---|---|---|
| `Type` as runtime protocol/superclass | superficially uniform message surface | confuses kinding with object inheritance; pressures class wrapper/runtime changes |
| atomic `Type` kind + `TypeForm` semantic/reflective role | preserves formal/runtime strata | reflection needs explicit adapters |

**Decision:** atomic kind plus role. Transfer uniform reflective operations through [03](03-reflection-api-and-capabilities.md); reject runtime superclass fiction. Avoids `value.class`/`value : T` collapse.

### 12.2 Class wrappers versus direct nominal denotation

| Option | Benefit | Cost |
|---|---|---|
| wrap every class as a separate type object | one descriptor class | duplicate nominal identity; wrapper/cache/GC complexity |
| class object directly reifies nominal form | identity remains truthful | synthetic forms need separate descriptors |

**Decision:** direct nominal reification. `Int ⇝ TypeForm` returns `Int`; `TypeData::ClassObject` stays internal static representation and is not a universal wrapper.

### 12.3 `Type.currentApplication` versus explicit `TypingContext`

| Option | Benefit | Cost |
|---|---|---|
| ambient current application | concise generic class-side code | fiber/reentrancy/security/cache invalidation ambiguity |
| explicit immutable context | call intent and authority visible | extra parameter/API step |

**Decision:** explicit context. Reject forwarding applied type descriptors into ordinary class-side dispatch.

### 12.4 Class specialization versus erased ordinary runtime classes

**Decision:** preserve ordinary runtime classes. Static applications and metadata do not create new class/metaclass objects. Internal optimization may specialize code only if `class`, identity, dispatch, reflection, and serialization stay observationally unchanged.

### 12.5 Strong global descriptor cache versus bounded weak context cache

**Decision:** loader/VM-owned weak canonicalization for synthetic descriptors; nominal objects need no cache entry. Strong global cache risks unbounded retention, cross-world staleness, and teardown cycles.

### 12.6 Singleton literal types versus constant facts

**Decision:** syntax fixes nominal runtime type (`1 : Int`, `1.0 : Float`); exact value is `ConstantFact`. Singleton/refinement types remain future explicit features. Avoids polluting canonical `TypeStore`, fake numeric coercion, and union explosions.

### 12.7 Total-by-default versus partial default plus explicit totality

**Decision:** partial default. Explicit total declaration requires termination proof. Avoids claiming termination for recursion, dynamic calls, FFI, blocking, or open-world dispatch.

### 12.8 Ephemeral verdicts versus persistent trust-aware artifacts

**Decision:** persistent artifacts. Cache key names VC, assumptions, interfaces, model, backend, and kernel; trust tier is mandatory. Avoids stale, unreproducible, or overstated proofs.

### 12.9 Universal row kind versus domain-specific rows

**Decision:** separate `RecordRow`, future `EffectRow`, and future `VariantRow`. Share algorithms only through typed policy wrappers. Avoids semantic cross-contamination.

### 12.10 Duplicate complete grammars versus shared normalized lowering

**Decision:** source and native parsers remain distinct front ends because recovery/provenance differ; both lower into a shared semantic term/result vocabulary. Avoids both divergence and a lowest-common-denominator parser.

### 12.11 Boolean relations versus explicit terminal outcomes

**Decision:** result-rich equivalence/subtype/assignable/consistent APIs with proven yes/no, unknown reason, cancel, budget, and internal states. Avoids dynamic, recursion, or budget being coerced into true/false.

### 12.12 Editor-owned checker versus compiler-owned semantic DB

**Decision:** `phalcom-semantic` owns formal query DB and snapshots. Compiler/CLI/REPL/LSP share them; LSP `ValueShape` remains advisory. Avoids contradictory diagnostics, duplicate linking/cache graphs, and stale editor-only truth.

## 13. Cross-decision consequences

These choices reinforce each other:

```text
atomic Type kind + direct nominal reification
    -> no class wrapper or runtime tower change
    -> explicit TypingContext for retained generic intent
    -> metadata DAG records truthfully retained facts
    -> weak synthetic descriptor cache is sufficient

compiler-owned SemanticDb + explicit outcomes
    -> one relation/effect/proof authority
    -> stamped snapshots and structured module failures
    -> LSP adapts formal facts; ValueShape remains advisory
    -> persistent proofs key against semantic fingerprints

separate solver domains + prenex publication
    -> no escaping variables
    -> record/effect/proof states cannot masquerade as types
    -> diagnostics retain exact failure domain
```

Changing one root decision requires auditing downstream decisions rather than patching a local enum.

## 14. What this must not preclude

Current choices preserve room for:

- higher-kinded libraries under prenex kind schemes;
- separately designed protocols and coherence;
- explicit refinement/singleton types built above `ConstantFact`;
- variant and effect rows with distinct domains;
- an `Any` top after lattice policy ratification;
- intersections after normalization/conformance rules;
- richer effect handlers and termination measures;
- multiple proof backends and a later certificate kernel;
- metadata profiles with different retention levels;
- safe parallel query publication after profiling;
- internal code specialization that preserves runtime identity.

They intentionally do not preserve compatibility with ambient generic context, type-directed selector identity, a universal row kind, Python `any` behavior, or proof-by-assumption defaults.

## 15. Risks and review gates

| Risk | Gate |
|---|---|
| syntax commits ahead of semantics | [04](04-user-facing-type-syntax-and-lowering.md) phase gates and parser/semantic parity tests |
| HKT complexity leaks into runtime | proper-type boundary and no dispatch/identity changes |
| structural records become nominal layout reflection | record-domain wrappers and capability-aware relations |
| dynamic/open world becomes false precision | explicit unknown/consistent outcomes |
| metadata claims erased facts | retention profile and opaque/missing statuses |
| cache architecture copies unsafe mechanisms | safe first publication, profiling, memory-model review |
| proof UI overstates evidence | mandatory trust tier and artifact validation |
| language accretes dependent/proof-term features accidentally | named ratification gates in [05](05-advanced-kinds-constraints-effects-and-proofs.md) |

Independent review should ask for each implementation: which precedent is being transferred, which Phalcom invariant limits it, which rejected mechanism might have entered indirectly, and which measurement proves the operational trade-off.

## 16. Take directly / Adapt / Reject

### Take directly

- owner-qualified binder identity;
- type-constructor/proper-type/kind distinctions;
- declaration-site sign variance;
- explicit bounds and obligation results;
- row-tail and occurs-check discipline;
- partial versus total correctness;
- verification conditions, counterexamples, certificates, and trust tiers;
- Pyrefly staged queries, snapshots, dependencies, bounded solving, observability, and differential incrementality tests.

### Adapt

- ML/Haskell inference to declared, incremental, open-world Phalcom surfaces;
- OCaml row machinery to separate Phalcom row domains;
- Scala HKT/variance ideas without ambient machinery;
- Rust trust/identity discipline without Rust runtime or borrow semantics;
- Swift/JVM/.NET reification lessons to explicit contexts and profile-controlled metadata;
- Smalltalk reflection/DNU into result-rich static summaries;
- TypeScript structural ergonomics without permissive relation collapse;
- proof-system artifacts without dependent public APIs.

### Reject

- `Type` as runtime protocol/superclass;
- wrappers around every class object;
- `Type.currentApplication` and transparent applied-class forwarding;
- runtime class specialization as language identity;
- strong immortal descriptor caches;
- literal singleton types by default;
- totality by default;
- ephemeral untrusted proof verdicts presented as facts;
- universal row identity;
- one source/native recovery parser;
- boolean/coarse relation APIs;
- LSP-owned formal checking;
- Python semantics, ambient implicits, dependent escalation, and proof-term programming.

## 17. Phalcom typing philosophy

Phalcom typing is compiler-owned knowledge about a message-oriented runtime, not a replacement runtime. It keeps kinds, types, class objects, reflected descriptors, flow facts, dynamic boundaries, effects, and proofs in explicit domains. Stable identities and immutable snapshots support incremental tools; bounded result-rich queries preserve uncertainty honestly. Runtime identity and dispatch remain unchanged unless a separate runtime decision says otherwise. Reification is explicit, metadata is truthful about erasure, and proof authority travels with persistent checked or named-trust evidence.
