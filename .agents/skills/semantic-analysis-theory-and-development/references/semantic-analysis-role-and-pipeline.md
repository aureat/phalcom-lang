# Semantic Analysis: Role, Pipeline, and Ownership

## 1. The problem semantic analysis solves

Parsing answers “what source structure was written?” Runtime execution answers “what happened for this concrete run?” Semantic analysis sits between them and answers durable questions about program entities and conservative knowledge: which declaration a use denotes, which member a send may target, which control-flow paths can reach a program point, what values may flow through a binding, which facts depend on which modules/callables, and which explanations support those facts.

For Phalcom this layer must be shared. The compiler, LSP, future checker, typed runner, linter, refactoring engine, optimizer, documentation/reflection tooling, and future prover should not each own a subtly different resolver or call graph. They may consume different **domains** over the same identities and control structure.

The architecture should therefore separate three questions:

1. **Semantic identity:** what entity does this source construct denote?
2. **Semantic structure:** what normalized control/data relationships exist independently of a particular consumer?
3. **Abstract knowledge:** what does a particular analysis know about those entities at a program point?

Those layers are related but not interchangeable.

## 2. Current repository anchor

**CURRENT (inspected main around `61dae340...`, 2026-08-15):** `phalcom-lsp/src/semantic/` already embodies much of this decomposition. `ids.rs` defines module-qualified class/callable/field identities. `scope.rs` and occurrence machinery establish lexical meaning. `surface.rs` extracts source-visible class/member structure. `flow.rs` performs structured flow and emits local, field, parameter, call, return, and summary products. `facts.rs` defines `ValueShape`, `InferredValue`, confidence, provenance, and contribution-indexed parameter evidence. `engine.rs` owns mutable worker state and publishes coherent immutable snapshots.

This is a useful present-day semantic substrate, but it remains **CURRENT LSP semantic infrastructure**, not proof that all future consumers should use the exact same concrete representation. In particular, `ValueShape` is explicitly advisory runtime-shape knowledge and is deliberately not the formal Phalcom type language.

## 3. A staged semantic pipeline

A productive conceptual pipeline is:

```text
text / source revision
        |
        v
parser + recovery
        |
        v
source AST / CST identities
        |
        v
source surface extraction
(declarations, imports, member signatures)
        |
        v
lexical scopes + bindings + occurrences
        |
        v
resolved semantic identities
        |
        +-------------------------------+
        |                               |
        v                               v
optional semantic HIR              module/declaration graph
        |                               |
        v                               |
body CFG / program points <-------------+
        |
        v
shared analyses
(local flow, calls, fields, effects, summaries)
        |
        v
immutable semantic generation / query API
        |
   +----+-------+---------+----------+---------+
   v            v         v          v         v
  LSP         checker   lints     refactor  optimizer/prover
```

The arrows are dependency relationships, not a mandate for one Rust crate per box. A small implementation can combine stages; the architectural requirement is that ownership and validity are explicit.

## 4. Source structure versus semantic structure

A source AST preserves authored syntax. A semantic representation should normalize syntax only when normalization removes distinctions that consumers should not repeatedly reimplement.

For example, if multiple consumers need to reason about all assignment forms as “write binding/field/global,” that is a candidate for lowering. If only one formatter needs to know whether a trailing closure was used, that belongs in source structure, not semantic HIR.

A useful criterion is **semantic duplication pressure**. Introduce shared representation when several consumers independently reconstruct any of:

- implicit receiver rules;
- selector canonicalization;
- desugared setter/subscript operations;
- loop back-edges and exits;
- return destinations and non-local returns;
- capture boundaries;
- pattern-bind operations;
- call argument/label normalization;
- abrupt completion edges;
- effect boundaries.

The goal is not fewer AST nodes. The goal is one place where meaning-changing normalization is specified and tested.

## 5. Judgments and queries

A semantic pipeline can be described with judgments without forcing the implementation to literally be a theorem prover. For name resolution:

```text
Σ ; Γ ⊢ name x ⇓ b
```

Read this as: under module/project environment `Σ` and lexical environment `Γ`, source name `x` resolves to semantic binding `b`.

For expression analysis:

```text
A ; p ⊢ e ⇝ f
```

where `A` is an analysis snapshot, `p` a program point, and `f` an abstract fact. `f` might be runtime-shape knowledge, constant information, an effect, or a type fact depending on the domain. The notation deliberately does not call every `f` a “type.”

A query API is the implementation counterpart:

```rust
fn resolve_name(file: FileId, offset: TextSize) -> Resolution;
fn binding_fact(binding: BindingId, point: ProgramPoint) -> Fact<ValueDomain>;
fn callable_summary(id: CallableId) -> Option<CallableSummary>;
fn dispatch_candidates(receiver: ReceiverFact, selector: SelectorId) -> CandidateSet;
```

The query should expose semantic data, not `tower_lsp` objects or VM heap handles. Consumers translate it to their own protocol.

## 6. Semantic consumers and permissible strength

One shared foundation does not mean every consumer receives facts with the same trust threshold.

| Consumer | Can use heuristic facts? | Can reject a program? | Can justify unsafe optimization? |
|---|---:|---:|---:|
| completion/hover | yes, if labeled | no | no |
| lint | often, with confidence policy | usually no | no |
| refactoring | only if identity/conflict conditions are sound | may refuse action | no |
| checker | no for correctness errors unless rule permits uncertainty | yes | no |
| typed runner | only through defined runtime policy | yes/raise | no |
| prover | only trusted/sound premises | yes for proof obligation | potentially, if proof contract allows |
| optimizer | only sound facts or guarded speculation | no semantic change | yes, within guards/assumptions |

The same `BindingId` may feed all of them; the same heuristic `ValueShape` should not.

## 7. The semantic truth invariant

The system should satisfy a correspondence property:

> If two consumers claim to refer to the same semantic concept in the same published generation, they must use the same semantic identity or an explicit, deterministic bridge to it.

Examples:

- rename and references use the same binding/declaration identity;
- checker diagnostics about a method and LSP navigation point at the same `CallableId`/declaration;
- a call summary dependency is keyed by the same callable identity used for dispatch candidate lookup;
- module import invalidation uses the same canonical module identity used by class qualification.

Violations create “split brain” behavior: hover says one thing, go-to-definition another, checker a third.

## 8. When not to introduce HIR or CFG

A direct AST analysis is often correct for local, syntax-shaped questions. Do not introduce HIR merely because mature compilers have one. Prefer AST + side tables when:

- the fact is local and syntax preserving;
- evaluation order is already explicit in AST shape;
- no consumer duplicates desugaring;
- recovery/source ranges are easier to retain directly;
- the representation would otherwise be one-to-one boilerplate.

Introduce a CFG when control joins, reachability, loop back-edges, definite assignment, refinements, or abrupt completion become central. A “structured recursive visitor with copied state” can approximate CFG analysis for a while; once it must reimplement joins/back-edges across several analyses, an explicit CFG normally pays for itself.

## 9. Failure modes

### Parallel semantic worlds

A checker creates `CheckerSymbolId`, the LSP keeps `BindingId`, and the compiler tracks strings. All three resolve independently. This seems modular but makes cross-tool consistency and incremental invalidation expensive. Prefer one shared identity layer and domain-specific facts keyed by it.

### Representation leakage

A VM `Handle<ClassObject>` is stored in an LSP fact because it is “the class.” This ties semantic analysis to one executing VM and confuses runtime identity with source semantic identity. Use `ClassId`; bridge to runtime metadata only where a runtime-backed consumer needs it.

### Source-range identity

A declaration is identified by `(file, start, end)`. Editing a comment above it changes identity, destroying references/caches even though the declaration is semantically unchanged. Ranges are locations and provenance, not durable identity.

### Heuristic escalation

Completion inference guesses that `x` is `String`; the checker later reuses this representation and rejects `x + 1`. A heuristic domain has silently become a correctness domain. Keep trust/domain tags explicit.

## 10. Phalcom application example

Suppose:

```phalcom
class Greeter {
  greet(name) { "hello " + name }
}

let g = Greeter()
g.greet("Ada")
```

A robust pipeline records:

1. a `ClassId(module, "Greeter")`;
2. a `CallableId(owner=Greeter, selector=<canonical greet selector>, side=Instance)`;
3. a lexical `BindingId` for `g`;
4. a source occurrence linking `Greeter` in the constructor expression to the class identity;
5. a flow fact for `g` with advisory shape `Instance(Greeter)`;
6. dispatch candidate resolution for `g.greet(...)` using selector semantics and class hierarchy;
7. a call edge/call-site parameter contribution to the callable summary;
8. provenance that can explain why the return shape was inferred;
9. dependency edges so changing the body of `greet` invalidates its dependents rather than unrelated classes.

Future formal typing can add `TypeFact`s keyed by the same identities/program points without declaring `ValueShape::Instance(Greeter)` equal to the type `Greeter`.

## 11. Review questions

An implementation agent should be able to answer:

1. Which part of this feature is source syntax, semantic identity, normalized meaning, abstract fact, or runtime behavior?
2. Which consumers need the distinction, and can they share a representation?
3. Does the analysis make a sound claim, a best-effort advisory claim, or a speculative optimization claim?
4. What is the smallest representation that prevents duplicated semantics?
5. What invalidates the fact? Can the dependency be named precisely?
6. What source/provenance must survive normalization?
7. If the program is incomplete, which facts remain valid?
8. If runtime reflection changes method/class state, which static assumptions remain trustworthy?
9. Can an incremental rebuild produce the same final truth as a clean rebuild?
10. What test would fail if another consumer accidentally implemented a second semantic system?
