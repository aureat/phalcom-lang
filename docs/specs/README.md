# Phalcom Specifications

`docs/specs/` is the authoritative specification corpus for current Phalcom semantics.

Material under `docs/spec/`, `docs/implementation/`, `docs/decisions/`, drafts, audits, plans, and historical documents may explain provenance or implementation history, but it does not override an owning document in this directory.

## Authority and precedence

Each semantic concept has one primary owner. When documents overlap:

1. the owning `docs/specs/` document defines the concept;
2. neighboring specifications define only their interaction with that concept;
3. VM specifications may define a realization contract, but do not redefine source-language semantics;
4. older or non-authoritative material yields to the current owning specification.

A contradiction should be resolved by correcting ownership or cross-references, not by preserving parallel normative descriptions.

## Specification layers

### Language semantics

Language specifications define source-visible meaning independent of compiler or runtime strategy.

Core type and generic specifications:

- [Generic Constraints](generic-constraints.md)
- [Associated Types](associated-types.md)
- [Iterators and Iterables](iterator.md)

Object-model specifications:

- [Values](objects/values.md)
- [Data Objects](objects/data-objects.md)
- [Enumerated Types and Variants](objects/enums.md)
- [Traits and Conformance](objects/traits.md)
- [Implementation Declarations](objects/impl.md)

Callable semantics are indexed by [Callables](callables/INDEX.md). That subcorpus owns selectors, argument shapes, dispatch, callable values, Methods, BoundMethods, Closures, callable references, Families, execution contexts, and callable reflection.

### Platform semantics

Platform specifications define language-visible contracts provided by the standard environment rather than the core grammar alone.

- [Universe Package](universe.md)

### VM and implementation internals

`docs/specs/vm/` defines the canonical VM/internals layer. These documents may be more implementation-specific than language specifications because that is their subject.

- [Bytecodes](vm/bytecodes.md)
- [VM Execution Model](vm/vm-execution-model.md)
- [VM Values and Objects](vm/values-and-objects.md)
- [VM Products](vm/products.md)

VM documents must preserve source-language semantics owned elsewhere.

## Semantic ownership rules

Specifications should be concise, semantic, and non-redundant.

An owning specification should define:

- the semantic entities in its domain;
- their identities and relations;
- legal and illegal source forms where relevant;
- observable behavior and static obligations;
- boundaries with adjacent concepts.

It should not become an implementation plan, migration log, test plan, compiler architecture note, LSP design, or cache/runtime-data-structure specification unless that implementation machinery is itself the document's declared specification layer.

Cross-spec sections should delegate rather than reproduce another owner's rules.

## Source-language conventions

Phalcom has no source-level free-function declaration form. Named user-defined executable behavior is method-based.

`Function` is a callable value/runtime category, not a free-function declaration category. See [Callable Values and Function](callables/callable-values.md).

Generic constraints preserve distinct relations:

```text
trait conformance   |=
subtyping           <:
type equivalence    ==
kind classification :
```

Constraint syntax and evidence rules are owned by [Generic Constraints](generic-constraints.md). In particular, Phalcom does not use `<T: Trait>` as trait-bound syntax.

Associated declarations are trait-owned, conformances supply bindings, and projections identify their trait-owned associated declaration rather than only a subject and string name. See [Associated Types](associated-types.md).

Callable references use current forms such as:

```phalcom
&object.method(_)
&object.method(...)
&Option::Some(_)
&Option::Some...
```

Constructors remain ordinary class-side Methods where applicable; enum variant constructors and primary data constructors retain their distinct semantic categories.

## Editing discipline

When changing this corpus:

1. identify the document that owns the semantic fact;
2. verify current language syntax and repository behavior when the fact is disputed;
3. change the owner first;
4. replace duplicated neighboring prose with delegation;
5. keep examples legal under current Phalcom syntax;
6. validate relative links after file moves or renames;
7. keep implementation planning and migration machinery outside the language-semantic layer.

The goal is one coherent current specification, not preservation of every historical formulation.
