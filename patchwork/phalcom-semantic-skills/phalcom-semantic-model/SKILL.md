---
name: phalcom-semantic-model
description: >-
  Canonical semantic doctrine for Phalcom. Use when designing or reviewing any
  compiler, LSP, checker, static-analysis, lint, refactoring, module, typing,
  runtime-contract, optimizer, or language feature that needs to know what a
  source construct means. Defines semantic identities, scope/binding rules,
  runtime value knowledge versus language types, dispatch facts, flow facts,
  callable summaries, provenance, uncertainty, module/project identity,
  incrementality, and the contract shared by all semantic consumers.
compatibility: Designed for coding agents working on the Phalcom repository (aureat/phalcom-lang).
---

# Phalcom Semantic Model

This skill defines the mental model that every Phalcom semantic consumer must share.
It is deliberately broader than the current LSP implementation and deliberately
narrower than the whole language specification. Its job is to prevent the compiler,
LSP, checker, lints, static prover, runtime contracts, and optimizer from inventing
parallel meanings for the same program.

The central rule is:

> **One semantic truth, many consumers.** Parse once into source structure, assign
> semantic identities once, derive facts through shared analysis, and let each
> consumer render or enforce those facts according to its own contract.

Do not treat this skill as permission to redesign settled Phalcom semantics. Before
changing language behavior, read the current specifications/ADRs and the installed
`language-design` skill. This skill governs the architecture of semantic knowledge,
not the authority to change the language.

## When to use this skill

Use it before work involving any of the following:

- lexical scopes, bindings, shadowing, imports, modules, classes, members, selectors;
- hover, completion, go-to-definition, references, rename, inlay hints, diagnostics;
- receiver/member inference, callable return inference, field/parameter inference;
- flow-sensitive facts, control-flow analysis, reachability, narrowing or refinement;
- optional typing, checker architecture, typed-runner contracts, type-driven diagnostics;
- static proving, contracts, invariants, exhaustiveness, effect analysis;
- semantic lints, refactorings, code actions, symbol indexing;
- module/package/project graphs and incremental invalidation;
- optimizer assumptions that depend on semantic facts;
- semantic modeling of the core/standard library, native primitives, FFI, fibers, or effects.

## Read first

For repository work, orient from the current tree rather than assuming paths or
structures are unchanged:

1. `CLAUDE.md` / `AGENTS.md` for repository layout and graphify rules.
2. `docs/spec/current/` and relevant ADR/PDR/spec documents for normative behavior.
3. `phalcom-lsp/src/semantic/` for the current live semantic implementation.
4. `phalcom-core/src/` for runtime/compiler behavior that semantics must describe.
5. `phalcom-ast/src/` for the source representation and ranges the semantic layer consumes.
6. `docs/spec/typing/` for typing proposals; distinguish ratified/current behavior from proposed design.

If `graphify-out/graph.json` exists, query graphify before broad source scanning.
Use structural queries for definitions/callers/impact, then inspect the actual source.

## Non-negotiable invariants

### 1. Syntax is not identity

Never use source spelling alone once a semantic identity exists.

Examples:

- a lexical name occurrence resolves to a `BindingId`, not merely `"x"`;
- a class is module-qualified, not merely `"Point"`;
- a callable is owner + canonical selector + dispatch side, not merely `"draw"`;
- a field is owner + name + storage/dispatch side;
- a module is identified by canonical module identity, not an arbitrary path string;
- source ranges locate occurrences; they are not the semantic identity themselves.

Source text is a presentation key. Semantic identity is the durable key.

### 2. Runtime shape is not language type

The current LSP `ValueShape` is advisory knowledge about possible runtime values.
It is explicitly not Phalcom's future language type representation.

Keep these concepts separate:

- runtime class/shape knowledge;
- declared type expressions and resolved types;
- inference variables/constraints used by the checker;
- gradual/dynamic escape states;
- proof facts/refinements;
- runtime contract evidence.

A future `TypeId` may be informed by `ValueShape`, and `ValueShape` may be sharpened
by a declared type, but neither silently becomes the other.

Read [references/knowledge-domains.md](references/knowledge-domains.md).

### 3. Static typing describes dynamic Phalcom; it does not create a second dispatcher

Type metadata must not silently change ordinary selector identity or message lookup.
If a future feature intentionally introduces typed dispatch, multimethod dispatch,
or specialization, it must be an explicit language feature with explicit semantics,
not an accidental consequence of the checker knowing more.

The ordinary dynamic object model remains the semantic baseline unless a normative
spec says otherwise.

### 4. Binding before inference

Resolve lexical/module/class identities before inferring what values they may hold.
Do not infer a variable by repeatedly searching source spelling from scratch.

Canonical order:

```text
syntax/source target
    -> scope
    -> semantic identity
    -> control-flow position
    -> value/type/effect/proof facts
```

Inference may use unresolved/dynamic states during recovery, but it must never replace
name resolution.

### 5. Facts carry uncertainty explicitly

Do not collapse all missing knowledge into one vague `None`.
Distinguish states that have different semantic consequences.

At minimum, reason about the difference between:

- known exactly;
- known from flow;
- known interprocedurally;
- heuristic/advisory;
- unknown because no useful evidence exists;
- widened because analysis deliberately lost precision;
- unresolved because dependency/name resolution failed;
- ambiguous because several candidates remain;
- inconsistent/error because facts contradict a required contract;
- unreachable/impossible (`bottom`) where no runtime value can occur;
- deliberately dynamic/unchecked once the type system introduces that concept.

The concrete enum structure may evolve. The distinctions must not be erased simply
because one consumer does not need all of them.

### 6. Facts carry provenance

A semantic fact should be able to answer, within practical bounds, **why it is believed**.

Examples:

- literal syntax;
- binding initializer or assignment;
- branch refinement;
- constructor field write;
- resolved call site;
- callable summary;
- declared type annotation;
- protocol/constraint requirement;
- imported declaration;
- trusted native/core signature.

Provenance enables diagnostics, debugging the analyzer, confidence rendering,
explainability, and safe invalidation. Keep it bounded and compact on hot paths.

### 7. Joins are semantic operations, not collection concatenation

When paths or call sites merge, combine facts using the abstract domain's join.
Do not "pick the latest" across incomparable paths and do not build unbounded unions.

Every fact domain must define:

- ordering / precision relation;
- bottom if meaningful;
- top/unknown if meaningful;
- join;
- widening policy if a loop/recursive solve can grow indefinitely;
- equality used for fixed-point convergence.

Read [references/abstract-knowledge.md](references/abstract-knowledge.md).

### 8. Control-flow facts belong to program points

A binding's value before an assignment is not necessarily its value after the
assignment. A fact inside one branch is not automatically valid after the merge.

Never implement flow-sensitive behavior as a file-global `name -> fact` map.
Attach facts to semantic identities and a program point, explicit CFG state, ordered
fact stream, or an equivalent representation with well-defined reachability semantics.

### 9. Interprocedural analysis uses summaries and dependency edges

Do not recursively re-analyze arbitrary callee ASTs from every call site.
Use callable summaries whose inputs, output, effects, dependencies, provenance, and
revision/invalidation behavior are explicit.

Recursive call graphs require fixed-point/SCC reasoning or conservative cutoffs.

### 10. LSP is a semantic consumer, not the owner of semantics

Hover, completion, definition, references, rename, signature help, semantic tokens,
and diagnostics should query shared semantic facts.

Avoid feature-specific inference paths such as:

```text
hover inference
completion inference
lint inference
checker inference
```

Prefer:

```text
shared semantic query -> consumer-specific rendering/policy
```

### 11. Recovery is first-class

Editors analyze incomplete and invalid programs. Semantic construction must tolerate
recovered ASTs, unresolved imports, missing members, half-written selectors, and
syntax errors without panicking or manufacturing false certainty.

A semantic query may return partial knowledge. It must remain structurally coherent.

### 12. Incrementality is dependency-driven

An edit should invalidate facts that depend on the changed semantic contribution,
not arbitrary unrelated files.

At the same time, correctness beats micro-incrementality. Whole-file recomputation is
acceptable where the front end provides whole-file AST replacement; stale semantic
facts are not.

### 13. Published query state is coherent and immutable

Long-running/editor consumers should observe a coherent semantic generation rather
than a mixture of old and new facts. Prefer mutable worker state + immutable published
snapshots (the current semantic database already follows this model).

### 14. Runtime/compiler agreement is mandatory

When semantics models dispatch, module identity, visibility, class-side behavior,
field ownership, selector formation, core/native behavior, or control flow, compare
against the compiler/VM implementation and normative spec.

An LSP answer that is elegant but disagrees with execution is a bug.

### 15. Do not optimize away semantic distinctions prematurely

Compact representation is valuable, but do not merge concepts because they happen
to have the same current representation. Examples:

- `Unknown` versus impossible;
- source annotation absent versus explicit `Dynamic`;
- class name versus class identity;
- class-side storage versus instance-side storage;
- getter selector versus method selector;
- module namespace dependency versus execution dependency;
- flow fact versus declared contract.

## Semantic layers

Think in layers. A feature may span several, but it must say where each piece belongs.

```text
Source text
  -> tokens / recovered AST
  -> source targets and exact ranges
  -> lexical scopes and declarations
  -> semantic identities and name resolution
  -> module/class/member surfaces
  -> occurrences / references
  -> local value facts
  -> structured flow / CFG facts
  -> dispatch resolution
  -> callable/field/parameter summaries
  -> project/module fixed point
  -> typed facts / proof facts / effects (future)
  -> immutable semantic snapshot / query model
  -> LSP, checker, lints, refactorings, optimizer, typed runner
```

Read [references/semantic-layers.md](references/semantic-layers.md).

## Semantic identity hierarchy

Use the smallest identity that is stable for the lifetime of the fact.

Typical categories:

```text
Project/Workspace identity
  ModuleId
    ClassId / protocol/type declaration identity
      CallableId
      FieldId
    module-level BindingId
  file-snapshot ScopeId / BindingId
  source occurrence identity = (module/file generation, range, semantic target)
```

Current `ScopeId`/`BindingId` are intentionally file-snapshot-local. Do not serialize
or cache them across reparses unless a future stable-ID design explicitly guarantees it.

Read [references/identities-and-resolution.md](references/identities-and-resolution.md).

## Core semantic products

A mature Phalcom semantic snapshot should conceptually be able to provide these
products even if some are implemented lazily or are not yet present:

- module/project graph;
- source declaration surfaces;
- lexical scope graph;
- semantic occurrence index;
- class hierarchy / behavior surface;
- exact selector identities;
- dispatch candidates and resolved targets;
- local binding facts by program point;
- field facts and write evidence;
- parameter facts;
- callable return/effect summaries;
- dependency graph and invalidation edges;
- declared/resolved type metadata (future);
- flow-refined type facts (future);
- contract/proof obligations and proof results (future);
- effect facts such as may-throw/may-yield/may-block/may-mutate when ratified;
- diagnostic provenance.

Consumers should request the smallest product they need.

## Knowledge domains and bridges

Do not build one giant `SemanticValue` enum that conflates every domain.
Instead define bridges explicitly.

Examples:

```text
runtime shape -> candidate class surface -> completion members
runtime shape + selector -> dispatch candidates
runtime shape + callable summary -> advisory result shape
annotation syntax -> resolved language TypeId
language type + flow predicate -> refined language type
language type -> runtime contract/check plan
runtime shape + language type -> consistency evidence / diagnostic
callable summary + effect domain -> may-yield/may-throw/may-block
contract + path facts -> proof obligation
```

Bridges are where unsound assumptions tend to hide. Document them.

## Current Phalcom implementation anchors

As of the current repository generation, `phalcom-lsp/src/semantic/` already provides
important pieces of this doctrine:

- `ids.rs`: module-qualified `ModuleId`, `ClassId`, `CallableId`, `FieldId`, dispatch side;
- `scope.rs`: lexical `ScopeId`/`BindingId`, binding metadata, source-order resolution;
- `surface.rs`: source class/member/field/module surfaces;
- `occurrence.rs`: exact semantic occurrences and targets;
- `facts.rs`: `ValueShape`, confidence, provenance, local/field/parameter facts;
- `dispatch.rs`: semantic dispatch resolution;
- `flow.rs`: structured flow shared by local, summary, field, and call-site analysis;
- `callable.rs`: callable summaries and conservative effects;
- `infer.rs`: fixed-point/interprocedural inference helpers;
- `module_graph.rs`: dependency graph;
- `engine.rs`: mutable semantic worker state and affected-frontier rebuilds;
- `snapshot.rs` / `query.rs`: immutable coherent query surface;
- `invalidation.rs`: invalidation support.

Do not duplicate these concepts in a second subsystem without a migration plan.
Read [references/current-implementation-map.md](references/current-implementation-map.md).

## Designing a new semantic fact

Before adding any semantic fact, answer all of these:

1. **Question:** What exact semantic question does the fact answer?
2. **Identity:** What entity is the fact keyed by?
3. **Program point:** Is it global, declaration-local, or flow-sensitive?
4. **Domain:** What values can it take? What does unknown mean?
5. **Precision order:** When is one fact more informative than another?
6. **Join:** How are facts merged across control-flow/call sites?
7. **Convergence:** Can the domain grow indefinitely? What widens it?
8. **Provenance:** What evidence produced the fact?
9. **Dependencies:** What changes invalidate it?
10. **Recovery:** What happens on malformed/incomplete source?
11. **Consumer:** Which subsystem needs it, and what will it do with uncertainty?
12. **Runtime agreement:** Which compiler/VM/spec behavior constrains it?
13. **Typing bridge:** Could future typing sharpen or consume it? Keep that bridge explicit.
14. **Testing:** Which positive, negative, incremental and metamorphic fixtures prove it?
15. **Performance:** Is it hot? Can it be represented by compact IDs rather than cloned trees/strings?

If several consumers need the same answer, add the fact to shared semantics rather than
to the first consumer that requests it.

## Static versus dynamic semantic contract

For every future typed or statically-provable feature, write a semantic contract with
at least these columns during design/review:

| Dimension | Required question |
|---|---|
| Dynamic semantics | What does the existing VM/program do at runtime? |
| Static legality | What can be rejected without executing the program? |
| Inference | What facts can be derived when annotations are absent? |
| Unknown/dynamic boundary | What happens when proof is unavailable? |
| Runtime contract | Does typed-runner/check mode validate anything dynamically? |
| Reflection | What metadata remains observable? |
| LSP | What editor behavior is justified by the available facts? |
| Optimization | Which facts may be used as guards/assumptions, and how are they invalidated? |

Read [references/typing-and-proving.md](references/typing-and-proving.md).

## Consumer policy

Shared facts do not imply identical consumer behavior.

Examples:

- **Completion:** may show plausible members under advisory/union knowledge, clearly
  ranked and filtered; it should not assert a diagnostic merely because knowledge is weak.
- **Hover:** may display inferred shape/type plus confidence/provenance.
- **Checker:** must distinguish "cannot prove" from "proved invalid" according to the
  typing mode; it cannot turn heuristic evidence into a correctness judgment.
- **Lint:** must declare whether it is syntax-, binding-, flow-, type-, or project-level
  and usually suppress when required proof is unavailable.
- **Static prover:** must be conservative: failure to prove is not proof of falsehood.
- **Optimizer:** may use speculative facts only behind guards/invalidation that preserve
  exact slow-path behavior.

Read [references/consumers.md](references/consumers.md).

## Semantic hazards

Watch for these recurring failure modes:

- keying classes or members by bare names across modules;
- reparsing/re-inference inside each LSP feature;
- treating union members as all definitely available instead of conditionally available;
- treating `Unknown` as `Any`, `Dynamic`, or bottom;
- letting heuristic use-site inference become checker proof;
- recording assignment facts without branch reachability;
- losing source-order declaration constraints in scope lookup;
- recursive summary solving without convergence bounds;
- stale copied facts across module updates;
- copying a callee's internal fact graph into callers instead of referencing summaries;
- using type annotations to alter selector identity accidentally;
- modeling native/core primitives with stronger semantics than runtime guarantees;
- forgetting dynamic sends/reflection when claiming a closed dependency set;
- invalidating only syntax dependents when call summaries or field facts changed;
- leaking file-local `BindingId` across reparses;
- assuming module namespace, source file, package, and runtime module are forever identical;
- using source range as identity after edits;
- suppressing analyzer panics by returning bogus exact facts.

Read [references/hazards-and-invariants.md](references/hazards-and-invariants.md).

## Review rubric

A semantic change is incomplete until the reviewer can answer:

1. What semantic identity does it introduce or consume?
2. Which layer owns the truth?
3. What is the uncertainty model?
4. What are the join and convergence rules?
5. What source/runtime behavior makes the fact sound?
6. Which dependencies invalidate it?
7. Does it remain correct under incomplete source?
8. Does it accidentally change dynamic dispatch or object semantics?
9. Does future typing/proving have a clean bridge rather than a forced migration?
10. Can LSP/checker/lints reuse it without duplicating inference?
11. Are provenance and diagnostics possible?
12. Are negative and incremental tests present?
13. Is the representation compact enough for editor use?
14. What future feature does the chosen representation preclude?

For a full checklist, read [references/review-checklist.md](references/review-checklist.md).

## Navigation

Load references selectively.

| Reference | Use it for |
|---|---|
| [semantic-layers.md](references/semantic-layers.md) | Pipeline ownership and boundaries between syntax, semantics, typing, proving, runtime and consumers |
| [identities-and-resolution.md](references/identities-and-resolution.md) | Modules, scopes, bindings, classes, callables, fields, occurrences, stable/local IDs |
| [knowledge-domains.md](references/knowledge-domains.md) | Value shape vs language type vs dynamic/unknown vs proof/effect facts |
| [abstract-knowledge.md](references/abstract-knowledge.md) | Lattices, joins, widening, flow sensitivity, fixed points, confidence/provenance |
| [dispatch-and-callables.md](references/dispatch-and-callables.md) | Selector identity, instance/class side, dispatch resolution, summaries, higher-order calls |
| [modules-and-incrementality.md](references/modules-and-incrementality.md) | Module graph, project identity, invalidation, snapshots, cyclic dependencies |
| [typing-and-proving.md](references/typing-and-proving.md) | Integration with optional types, checker, typed-runner, contracts and static proving |
| [consumers.md](references/consumers.md) | LSP, checker, lints, refactors, formatter/parser relationships, optimizer use |
| [current-implementation-map.md](references/current-implementation-map.md) | Current repository semantic architecture and migration discipline |
| [language-precedents.md](references/language-precedents.md) | Lessons from rustc/rust-analyzer, Roslyn, TypeScript, Kotlin, Pyright/mypy, Julia, GHC/OCaml/Swift |
| [theory-reading.md](references/theory-reading.md) | Canonical type-system, dataflow, abstract-interpretation, CFG, proving and incremental-analysis concepts |
| [hazards-and-invariants.md](references/hazards-and-invariants.md) | Cross-feature traps and forbidden shortcuts |
| [review-checklist.md](references/review-checklist.md) | Design and implementation review gates |
