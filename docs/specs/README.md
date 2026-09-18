# Phalcom Specifications

`docs/specs/` is the authoritative specification corpus for the current Phalcom language, platform, and implementation model.

The documents in this directory define the semantics that implementations, tests, tooling, implementation plans, and language documentation are expected to preserve.

> **Authority rule**
>
> When a document in `docs/specs/` owns a semantic domain, it is the canonical specification for that domain. Historical specifications, design drafts, implementation plans, walkthroughs, audits, tests, and implementation details do not override it.

---

## 1. Specification layers

The specification corpus contains three distinct layers.

### Language specifications

Language specifications define source-visible and semantic behavior independent of a particular VM representation.

They include:

- [`generic-constraints.md`](generic-constraints.md) — generic `where` constraints, conformance bounds, associated specialization, generic projection, equivalence, and subtyping constraints;
- [`associated-types.md`](associated-types.md) — trait-owned associated declarations, conformance bindings, projection identity, and normalization;
- [`objects/`](objects/) — values, data objects, enums, traits, and implementation declarations;
- [`callables/`](callables/) — selectors, arguments, methods, callable values, closures, references, families, dispatch, execution context, binding, and callable reflection.

A conforming implementation may choose different internal representations or algorithms provided that the language semantics remain unchanged.

### Platform specification

[`universe.md`](universe.md) defines the canonical Phalcom Universe package and the semantic relationship among builtin source ownership, module identity, native implementation, prelude visibility, reflection, and tooling.

Universe is part of the language platform, but its package architecture is not the general source-language grammar.

### VM and implementation-internals specifications

[`vm/`](vm/) defines the authoritative implementation contract of the Phalcom VM and runtime.

These specifications may intentionally prescribe internal invariants that are not observable source-language semantics, including bytecode behavior, execution machinery, runtime value/object representation, and internal product representation.

An implementation-internals rule must not silently redefine language semantics owned by a language specification.

---

## 2. Authority and precedence

Specification authority follows semantic ownership, not file age, implementation status, or textual specificity.

When documents overlap:

1. the specification that explicitly owns the semantic concept governs that concept;
2. a neighboring specification may rely on that concept but must not redefine it incompatibly;
3. VM specifications define realization, not alternative source-language semantics;
4. implementation plans describe how to reach the specification, not what the language means;
5. tests demonstrate behavior but do not become normative when they conflict with the specification.

If two current `docs/specs/` documents appear to make incompatible claims about the same owned concept, that is a specification defect. It must be resolved explicitly rather than by choosing whichever document, implementation behavior, or test is more convenient.

### Older specification trees

Material outside `docs/specs/` may remain useful for history, rationale, implementation archaeology, or unfinished design work, including directories such as:

```text
docs/spec/
docs/implementation/
docs/decisions/
docs/__unstructured__/
docs/audit/
docs/learn/
docs/theory/
```

Such material is not authoritative over `docs/specs/`.

An accepted decision or implementation checkpoint that changes current language semantics must be reflected in the owning specification. The decision record preserves rationale and history; the specification preserves the current contract.

---

## 3. Semantic ownership map

| Domain | Owning specification |
| --- | --- |
| Generic `where` constraints and generic evidence | [`generic-constraints.md`](generic-constraints.md) |
| Associated declarations, bindings, projection identity, normalization | [`associated-types.md`](associated-types.md) |
| Source declaration modifiers, access domains, implementation origin, and legacy namespace retirement | [`source-declarations.md`](source-declarations.md) |
| Foundational value/object semantics and exactness | [`objects/values.md`](objects/values.md) |
| Nominal transparent data objects | [`objects/data-objects.md`](objects/data-objects.md) |
| Enums, variants, exact cases, and variant construction | [`objects/enums.md`](objects/enums.md) |
| Traits, requirements, witnesses, defaults, conformance, coherence | [`objects/traits.md`](objects/traits.md) |
| Inherent and conformance `impl` declarations | [`objects/impl.md`](objects/impl.md) |
| Callable specification index | [`callables/INDEX.md`](callables/INDEX.md) |
| Selector identity | [`callables/selectors.md`](callables/selectors.md) |
| Argument/parameter shape, rest, and spread | [`callables/arguments.md`](callables/arguments.md) |
| Exact reified methods | [`callables/methods.md`](callables/methods.md) |
| Complete callable values and `Function` semantics | [`callables/functions.md`](callables/functions.md) |
| Closures | [`callables/closures.md`](callables/closures.md) |
| Callable-reference expressions | [`callables/references.md`](callables/references.md) |
| First-class callable families | [`callables/families.md`](callables/families.md) |
| Bound methods | [`callables/bound-methods.md`](callables/bound-methods.md) |
| Ordinary selector dispatch | [`callables/dispatch.md`](callables/dispatch.md) |
| Callable lexical/dynamic execution context | [`callables/execution-contexts.md`](callables/execution-contexts.md) |
| Callable reflection | [`callables/reflection.md`](callables/reflection.md) |
| Canonical builtin Universe package | [`universe.md`](universe.md) |
| VM bytecode set and bytecode contracts | [`vm/bytecodes.md`](vm/bytecodes.md) |
| VM execution model | [`vm/vm-execution-model.md`](vm/vm-execution-model.md) |
| VM value/object representation | [`vm/values-and-objects.md`](vm/values-and-objects.md) |
| VM product representation | [`vm/products.md`](vm/products.md) |

When a new specification is added, this table should be updated if the document becomes the canonical owner of a distinct semantic domain.

---

## 4. Cross-specification rules

### One concept, one owner

A semantic concept should have one primary owning specification.

Other specifications may summarize the rule needed for local context, but they should link to the owner rather than reproducing a second full definition.

For example:

```text
associated declaration identity
    owned by associated-types.md

generic T::Item / <T as Trait>::Item syntax
    owned by generic-constraints.md

trait conformance and witness selection
    owned by objects/traits.md
```

The three documents compose; none should invent a parallel conformance or projection model.

### Surface semantics before implementation mechanism

Language specifications state observable and semantic requirements. They should not prescribe:

- query scheduling;
- cache layout;
- incremental invalidation algorithms;
- cancellation propagation;
- solver worklists;
- internal IDs;
- bytecode encodings;
- object layout;
- GC strategy;

unless the implementation choice is itself part of the specified contract.

Such matters belong in implementation plans or `vm/` specifications as appropriate.

### Internal mechanisms cannot create language semantics

Compiler, semantic-analyzer, VM, LSP, reflection, or runtime implementation machinery may realize a language rule, but cannot independently invent a competing rule.

In particular:

```text
parser shape
AST representation
semantic IDs
lowering products
bytecodes
runtime metadata
reflection descriptors
```

are representations of semantic decisions, not independent sources of language meaning.

---

## 5. Source-language conventions

Examples in language specifications must use current Phalcom syntax and semantic categories.

In particular:

- Phalcom has no source-level free-function declaration form;
- named user-defined executable behavior is method-based;
- runtime/value category names such as `Function` do not imply a source-level function declaration category;
- ordinary method references use `&object.method(...)` forms;
- associated callable references such as `&Option::Some(_)` refer to supported associated callables;
- trait conformance, subtyping, kinding, and type equivalence are distinct relations.

A specification example that requires unsupported syntax is a specification bug, not permission to extend the language implicitly.

---

## 6. Normative and non-normative content

A specification may contain explanatory examples, diagrams, or implementation notes, but its normative contract should remain identifiable.

Terms such as **must**, **must not**, **is**, **is not**, **requires**, and explicitly named invariants express normative requirements when used to define language or VM behavior.

Examples illustrate the rules. An example does not create a new rule that contradicts the surrounding normative text.

Conceptual data structures or pseudocode are non-normative unless the document explicitly states that their shape is part of the implementation contract.

---

## 7. Specification changes

A change to language or VM semantics should update the owning specification as part of the same design transition.

When changing a specification:

1. identify the semantic owner;
2. update the normative rule there;
3. update dependent specifications only where their local summaries or examples become stale;
4. update this index if ownership or the specification set changes;
5. keep historical plans and decision records as history rather than rewriting them to appear current;
6. update tests and implementation to conform to the resulting specification.

A new feature should not be considered fully specified merely because an implementation plan, ADR, test, or implementation exists.

---

## 8. Specification quality

Authoritative specifications should be:

- **concise** — state the contract without reproducing implementation planning;
- **complete** — cover the syntax and semantic edge cases owned by the document;
- **non-redundant** — delegate neighboring concepts to their owners;
- **representation-independent** for language semantics;
- **explicit about boundaries** — say what the document owns and what it does not;
- **internally consistent** — examples and terminology must obey the rules they illustrate;
- **composable** — related specifications should describe one semantic system rather than parallel models.

The goal of `docs/specs/` is not to preserve every design discussion. It is to provide the smallest complete authoritative description of the current Phalcom contract.
