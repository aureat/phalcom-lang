# Document 1 --- ty Internal Architecture: Verified Repository Archaeology of the Incremental Semantic Database

## Investigation scope

This document is based on repository inspection of the Ruff
implementation containing `ty`.

The inspected implementation areas include:

-   `crates/ty_python_core/src/lib.rs`
-   semantic index construction
-   Salsa tracked queries
-   semantic entities
-   database layering

The purpose is to explain the actual architecture that exists in code,
not an inferred architecture from project descriptions.

------------------------------------------------------------------------

# 1. Repository reality: ty is implemented as a semantic database

The central architectural discovery is that `ty` is not structured as a
traditional compiler pass.

The core semantic model is built around persistent database queries.

The primary entry point inspected:

    crates/ty_python_core/src/lib.rs

contains the semantic index query:

``` rust
#[salsa::tracked(
    returns(ref),
    no_eq,
    heap_size=ruff_memory_usage::heap_size
)]
pub fn semantic_index<'db>(
    db: &'db dyn Db,
    file: ProgramFile<'db>
) -> SemanticIndex<'db>
```

The semantic index is therefore not a temporary compiler object. It is a
Salsa tracked computation.

The architecture is:

    Source File
         |
         v
    Parsed Module
         |
         v
    SemanticIndex
         |
         +----------------+
         |                |
         v                v
    Type Analysis     IDE Queries

------------------------------------------------------------------------

# 2. Semantic index construction

The implementation constructs the semantic model through:

``` rust
SemanticIndexBuilder::new(db, file, &module).build()
```

The semantic index stores semantic facts about the file.

Important fields include:

    place_tables

    scopes

    definitions_by_node

    expressions_by_node

    statements_by_node

    scopes_by_node

    use_def_maps

    ast_ids

    imported_modules

The semantic index therefore represents:

-   where declarations exist,
-   where expressions exist,
-   what scopes exist,
-   how names bind,
-   how control-flow relationships behave.

It is the program model consumed by later analysis.

------------------------------------------------------------------------

# 3. Fine-grained Salsa queries

The implementation does not expose only one large analysis query.

Instead, it provides smaller tracked computations.

Example:

``` rust
#[salsa::tracked]
pub fn place_table(
    db: &dyn Db,
    scope: ScopeId
)
```

The source comments explicitly explain the motivation:

Using a dedicated `place_table` query allows Salsa to avoid invalidating
dependent queries when a specific scope's place table is unchanged.

This is a critical optimization.

The difference:

## Coarse model

    semantic_index(file)

    file changed

    invalidate everything

## Fine-grained model

    semantic_index(file)

            |
            +-- place_table(scope A)
            |
            +-- place_table(scope B)
            |
            +-- use_def_map(scope A)

Only affected scopes invalidate.

------------------------------------------------------------------------

# 4. Semantic identities

The implementation uses stable semantic identifiers.

Examples:

    ProgramFile

    ScopeId

    FileScopeId

    Definition

    Expression

The semantic model does not repeatedly rediscover program objects from
syntax.

Instead:

    AST node

       |
       v

    Semantic identity

       |
       v

    Tracked analysis queries

This is the basis for incremental behavior.

------------------------------------------------------------------------

# 5. Definition and expression separation

The semantic model distinguishes:

## Definitions

Represent things introduced into the program:

-   variables,
-   functions,
-   classes,
-   bindings.

## Expressions

Represent computations and values.

This distinction matters because type inference depends on semantic
relationships.

Example:

    Expression:
        foo()

    depends on:

    Definition:
        foo

A change to the declaration invalidates dependent expression inference.

A change elsewhere does not.

------------------------------------------------------------------------

# 6. Use-def analysis as a first-class structure

The semantic index stores:

    use_def_maps

This indicates that ty performs explicit binding analysis.

The checker tracks:

-   where values are introduced,
-   where they are read,
-   where constraints apply.

This enables:

-   type propagation,
-   narrowing,
-   diagnostics,
-   control-flow reasoning.

------------------------------------------------------------------------

# 7. AST identity is not the semantic identity

The implementation contains:

    ast_ids

but the semantic model does not simply operate directly on AST nodes.

The AST is a source representation.

Semantic objects are the stable analysis representation.

This separation is required because:

-   syntax changes frequently,
-   semantic meaning may remain stable.

------------------------------------------------------------------------

# 8. Loop and control-flow analysis

The semantic index also contains structures for:

    LoopHeader
    UseDefMap
    Narrowing constraints

The implementation comments describe loop header bindings used to
represent values visible across iterations.

This demonstrates that the semantic model is not merely name resolution.

It already supports flow-sensitive reasoning required by the type
checker.

------------------------------------------------------------------------

# 9. Architecture extracted for Phalcom

The main lesson is not "copy Python typing."

The transferable architecture is:

                   Phalcom Semantic Database

    Source
     |
     v
    Parser
     |
     v
    Semantic Index

        |
        +-- declarations
        +-- expressions
        +-- scopes
        +-- selectors
        +-- contracts

        |
        v

    Incremental Type Queries

Recommended foundational entities:

    ModuleId

    PackageId

    ScopeId

    DeclarationId

    ExpressionId

    SelectorId

    ContractId

    TypeId

------------------------------------------------------------------------

# 10. Design principles extracted

## Principle 1

The semantic database is the foundation.

The type checker should consume semantic information, not construct its
own world.

------------------------------------------------------------------------

## Principle 2

Use small tracked queries.

Prefer:

    infer_expression(ExpressionId)

over:

    check_file(File)

------------------------------------------------------------------------

## Principle 3

Separate syntax from semantics.

AST nodes are inputs.

Semantic identities are the persistent world.

------------------------------------------------------------------------

## Principle 4

Store relationships explicitly.

Examples:

    definition -> uses

    scope -> declarations

    expression -> dependencies

Do not repeatedly rediscover them.

------------------------------------------------------------------------

# Conclusion

Repository archaeology confirms:

`ty` achieves performance by building a persistent semantic model and
exposing fine-grained incremental queries over that model.

The important architecture is:

    Persistent semantic database

            +

    Stable identities

            +

    Fine-grained tracked computations

            +

    Separate type inference layer

For Phalcom, this suggests that the semantic engine should be designed
before the complete type checker. The type system becomes a consumer of
a stable semantic universe.
