# Phalcom First Functional Type System Implementation Specification

## Document Status

This document is the repository-grounded implementation blueprint for
the first functional Phalcom type system.

Target milestone:

-   source type annotations;
-   semantic type representation;
-   annotation resolution;
-   compiler consistency validation;
-   LSP integration;
-   future-proof architecture for generics, variance, higher-kinded
    types, callable types, and reflective type objects.

This milestone deliberately does not implement runtime contract
insertion.

## Repository Investigation Summary

Repository examined: `aureat/phalcom-lang`

Current workspace structure:

-   `phalcom-ast`
-   `phalcom-common`
-   `phalcom-type-syntax`
-   `phalcom-native-meta`
-   `phalcom-native-macros`
-   `phalcom-native-surface`
-   `phalcom-core`
-   `phalcom-repl`
-   `phalcom-lsp`
-   `phalcom-modules`

The workspace is declared in the root `Cargo.toml`.

Relevant existing architecture:

### phalcom-ast

Important files:

-   `phalcom-ast/src/ast.rs`
-   `phalcom-ast/src/parser.rs`
-   `phalcom-ast/src/lexer.rs`

This crate owns source syntax representation and parsing.

### phalcom-type-syntax

Important file:

-   `phalcom-type-syntax/src/lib.rs`

This already contains a symbolic type syntax layer:

-   `TypeExpr`
-   `CallableType`
-   `ParameterTuple`
-   `TypeParameter`

It currently supports:

-   named types;
-   unions;
-   applied types;
-   tuples;
-   callable syntax;
-   `Unknown`;
-   `Never`.

This existing abstraction should be extended rather than replaced.

## Architectural Decision

Do not create a compiler-only type checker disconnected from existing
semantic infrastructure.

Create a semantic type layer above symbolic syntax.

The flow becomes:

source -\> AST annotation -\> TypeExpr -\> resolved semantic Type -\>
evidence checking -\> compiler diagnostics/LSP diagnostics

## New Crate

Add:

`phalcom-semantic`

Responsibility:

-   canonical TypeId;
-   type arena;
-   type interning;
-   subtype relations;
-   consistency checking;
-   evidence tracking;
-   shared diagnostics.

Do not place this in `phalcom-core`.

The runtime should not own compile-time reasoning.

## Initial Rust Data Model

Create:

`phalcom-semantic/src/type.rs`

``` rust
pub struct TypeId(pub u32);

pub enum TypeKind {
    Never,
    Unit,
    Any,
    Nominal(NominalType),
    Tuple(Vec<TypeId>),
    Record(Vec<FieldType>),
    Union(Vec<TypeId>),
    Function(FunctionType),
    Applied {
        origin: TypeId,
        arguments: Vec<TypeId>,
    },
    Parameter(TypeParameterId),
}
```

Unknown is not a TypeKind. It belongs to analysis state.

Create:

`phalcom-semantic/src/evidence.rs`

``` rust
pub enum KnowledgeState {
    Unknown,
    Known(TypeId),
    Dynamic,
}
```

Create:

``` rust
pub enum TypeEvidence {
    Proven(TypeId),
    Refuted {
        expected: TypeId,
        actual: TypeId,
    },
    Unproven(TypeId),
}
```

## AST Changes

Modify:

`phalcom-ast/src/ast.rs`

Add optional type annotations to declarations.

Required targets:

-   local bindings;
-   parameters;
-   return declarations;
-   fields.

The annotation must preserve:

-   original source span;
-   parsed TypeExpr.

Do not immediately resolve types inside AST.

AST represents syntax only.

## Parser Changes

Modify:

`phalcom-ast/src/parser.rs`

Extend existing declaration parsing to accept:

``` phalcom
value: Int
```

``` phalcom
fn(x: Int) -> String
```

``` phalcom
field: (Int, Int, name: String)
```

Parsing should produce:

``` rust
Option<TypeExpr>
```

not semantic TypeId.

## Type Resolution

Add:

`phalcom-semantic/src/resolver.rs`

Responsibilities:

-   convert TypeExpr into TypeId;
-   resolve nominal names;
-   resolve built-ins;
-   preserve unresolved names for diagnostics.

Initial supported forms:

-   named types;
-   Unit;
-   Never;
-   tuples;
-   records;
-   callable types.

Generic application is represented but not type-checked yet.

## Type Relations

Add:

`phalcom-semantic/src/relation.rs`

Implement:

-   equivalent;
-   subtype;
-   consistent.

Do not implement one universal compatibility method.

Initial rules:

    T <: T

    Never <: T

    Nominal inheritance creates subtype relations

## Compiler Integration

Modify `phalcom-core`.

The compiler should:

1.  parse annotations;
2.  resolve annotations;
3.  infer available evidence;
4.  compare evidence;
5.  emit diagnostics.

Normal execution remains unchanged.

## LSP Integration

Modify:

`phalcom-lsp`

The LSP must consume the same semantic type engine.

Existing ValueShape inference remains useful.

Create an adapter:

ValueShape -\> TypeEvidence

Hover should display:

-   declared type;
-   inferred type;
-   mismatch information.

Diagnostics should match compiler semantics.

## Future Compatibility Requirements

The implementation must preserve space for:

### Generic application

    List<Int>

represented internally as:

    TypeKind::Applied

### Higher kinds

Examples:

    List : Type -> Type
    Map : Type -> Type -> Type

### Callable types

Support:

    (Int, Int, name: String, **: Int) -> ReturnType

The existing `phalcom-type-syntax` callable representation should become
the input representation.

### Type-level operations

Future syntax:

    List<Int>

may lower semantically to:

    List.<>(Int)

and:

    Int -> String

to:

    Int.->(String)

The first implementation must not block this.

## Implementation Sequence

Phase 1:

-   add phalcom-semantic crate;
-   add TypeId and TypeKind;
-   add tests.

Phase 2:

-   connect TypeExpr resolution.

Phase 3:

-   add declaration annotations.

Phase 4:

-   add consistency checking.

Phase 5:

-   connect LSP.

Phase 6:

-   add reflection metadata storage.

## Testing

Required tests:

Parser:

-   `x: Int`
-   callable annotations;
-   tuple annotations;
-   record annotations.

Semantic:

-   Int equals Int;
-   Int incompatible with String;
-   Never subtype;
-   Unit identity;
-   Unknown state.

Compiler:

-   accepted annotations;
-   rejected contradictions.

LSP:

-   hover shows types;
-   diagnostics match compiler.

## Acceptance Criteria

The milestone is complete when:

-   Phalcom source can express type annotations;
-   annotations survive parsing;
-   annotations resolve into semantic types;
-   compiler validates consistency;
-   LSP uses identical reasoning;
-   runtime behavior remains dynamic;
-   architecture supports future generics, HKTs, variance, reflection,
    and contracts.
