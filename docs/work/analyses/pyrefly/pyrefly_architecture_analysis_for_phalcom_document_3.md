# Pyrefly Architecture Analysis for Phalcom

## Document 3 --- Implementation Blueprint: Building a Semantic Intelligence Engine for Phalcom

## Purpose

The first two documents extracted architectural lessons from Pyrefly:

-   why a dynamic language can support deep static analysis;
-   how semantic identities, query engines, type arenas, and constraint
    solvers create performance;
-   why LSP performance depends on semantic architecture rather than
    IDE-specific optimizations.

This document translates those findings into a concrete architectural
blueprint for Phalcom.

The goal is not to copy Pyrefly's Python type checker.

Phalcom has fundamentally different semantics:

-   value/type unification;
-   message passing;
-   selector-based dispatch;
-   contracts;
-   reflection;
-   lazy evaluation;
-   higher-kinded abstractions;
-   optional but powerful static verification.

Therefore the goal is:

> Build a unified semantic intelligence engine where compiler, LSP,
> documentation, reflection, and verification all consume the same
> semantic database.

------------------------------------------------------------------------

# 1. The Core Architectural Shift

A traditional compiler pipeline:

    Source

     |

    AST

     |

    Analysis passes

     |

    Types

     |

    Code generation

does not scale for Phalcom's goals.

The recommended architecture:

                     Source

                        |

                      AST

                        |

              Semantic Index Database

                        |

                 Query Engine

                        |

           +------------+------------+

           |                         |

       Type Solver              Presentation

           |                         |

     Compiler / Runtime        LSP / Docs

The semantic database becomes the center of the ecosystem.

------------------------------------------------------------------------

# 2. Semantic Database

The central component should be:

    phalcom-semantic

Its responsibility:

-   declaration identity;
-   symbol resolution;
-   dispatch information;
-   type information;
-   contracts;
-   metadata;
-   dependencies.

It should not be a compiler pass.

It should be a persistent knowledge store.

------------------------------------------------------------------------

# 3. Stable Semantic Identities

Pyrefly's strongest architectural choice is replacing repeated discovery
with stable identities.

Phalcom should introduce:

``` rust
ModuleId

DeclarationId

ExpressionId

SelectorId

TypeId

ContractId

DispatchId

QueryId
```

Every semantic entity receives identity.

Example:

``` phalcom
class User {}
```

becomes:

    DeclarationId(42)

    TypeId(81)

    DispatchEntityId(100)

    ReflectionEntityId(200)

The same entity participates in different semantic domains.

------------------------------------------------------------------------

# 4. Semantic Entity Model

Because Phalcom unifies values and types, entities should have multiple
semantic facets.

Recommended model:

    SemanticEntity

            |

            +---- ValueFacet

            |

            +---- TypeFacet

            |

            +---- DispatchFacet

            |

            +---- ContractFacet

            |

            +---- ReflectionFacet

Example:

    User

    Value:
        constructor

    Type:
        User

    Dispatch:
        save(_)
        serialize()

    Reflection:
        fields
        methods
        attributes

    Contract:
        Entity requirements

------------------------------------------------------------------------

# 5. Binding and Index Layer

Before type checking, Phalcom should build semantic indexes.

Responsibilities:

-   scopes;
-   imports;
-   declarations;
-   selectors;
-   inheritance;
-   traits;
-   contracts.

Architecture:

    AST

     |

    Semantic Index Builder

     |

    Indexed Semantic Graph

The type solver should never rediscover declarations.

------------------------------------------------------------------------

# 6. Query Engine

The central abstraction:

    Query

        |

    Answer

        |

    Dependencies

Examples:

    TypeOf(ExpressionId)

    ResolveDeclaration(SymbolId)

    Members(TypeId)

    Dispatch(TypeId, SelectorId)

    ContractOf(DeclarationId)

Each query has:

    Unknown

     |

    Computing

     |

    Complete

and stores its answer.

------------------------------------------------------------------------

# 7. Answer Tables

Do not implement:

``` rust
HashMap<NodeId, Type>
```

Instead:

    QueryId

     |

    AnswerSlot

     |

    Result

Benefits:

-   caching;
-   incremental updates;
-   cycle detection;
-   parallel evaluation.

------------------------------------------------------------------------

# 8. Dependency Graph

Every answer records dependencies.

Example:

    User.name changed

            |

    invalidate

            |

    User serialization

            |

    API contract checks

Dependencies should be fine-grained.

Not:

    Module A depends on Module B

but:

    Function foo depends on:

    User.save selector

    User contract

    User.name type

------------------------------------------------------------------------

# 9. Type Representation

Adopt a canonical type universe.

Architecture:

    TypeId

     |

    TypeArena

     |

    TypeData

Example:

``` rust
enum TypeData {

    Primitive,

    Class {
        declaration: DeclarationId,
        arguments: Vec<TypeId>
    },

    Function {
        parameters: Vec<TypeId>,
        result: TypeId
    },

    Application {
        constructor: TypeId,
        arguments: Vec<TypeId>
    },

    Union(Vec<TypeId>)
}
```

------------------------------------------------------------------------

# 10. Type Interning

Equivalent types should share identity.

Example:

    List<Int>

should become:

    TypeId(500)

everywhere.

Benefits:

-   fast equality;
-   efficient caches;
-   reduced memory;
-   stable references.

------------------------------------------------------------------------

# 11. Constraint-Based Type Solver

The solver should not immediately assign types.

Instead:

    Expression

     |

    Constraints

     |

    Constraint Graph

     |

    Solver

     |

    TypeId

Example:

    identity(10)

    creates:

    T

    10 conforms to T

    return = T

Solver produces:

    T = Int

------------------------------------------------------------------------

# 12. Bidirectional Typing + Constraints

Phalcom should combine:

-   bidirectional inference;
-   constraint solving;
-   lazy queries.

Expected flow:

    Expected Type

           |

    Checking

           |

    Constraints


    Unknown Type

           |

    Inference

           |

    Constraints

This gives precise inference without requiring annotations everywhere.

------------------------------------------------------------------------

# 13. Semantic Subtyping Boundary

The subtype engine should not know the whole program.

Create:

    SemanticOrder

Responsibilities:

    is_subtype()

    conforms()

    lookup members()

    resolve dispatch()

    get contracts()

The solver handles theory.

The semantic layer handles language facts.

------------------------------------------------------------------------

# 14. Selector Dispatch Integration

This is where Phalcom differs from Pyrefly.

Dispatch should be a query:

    DispatchQuery

    receiver TypeId

    selector SelectorId

    returns:

    DispatchResult

Example:

``` phalcom
account.save()
```

becomes:

    receiver:

    Account


    selector:

    save(_)


    result:

    AccountRepository.save

------------------------------------------------------------------------

# 15. Recursion Safety

All semantic computations need bounded evaluation.

Introduce:

``` rust
AnalysisBudget {

    max_depth,

    max_constraints,

    max_dispatch_expansions,

}
```

Used for:

-   recursive types;
-   recursive contracts;
-   generic expansion;
-   dispatch cycles.

------------------------------------------------------------------------

# 16. Type Normalization

Create:

    TypeNormalizer

Responsibilities:

-   union flattening;
-   duplicate removal;
-   generic normalization;
-   canonicalization.

Example:

    Int | Int

    =

    Int

This improves caching.

------------------------------------------------------------------------

# 17. Inference Provenance

The semantic engine should preserve:

    Why does this type exist?

Example:

    Type:

    List<User>


    Reason:

    - list literal contained User
    - append(User) constrained element type
    - User satisfied Entity contract

This enables advanced IDE explanations.

------------------------------------------------------------------------

# 18. LSP Architecture

The LSP should be a client of semantic queries.

Architecture:

    Semantic Database

            |

     Query API

            |

     Presentation Layer

            |

     LSP

The LSP should never contain independent inference.

------------------------------------------------------------------------

# 19. Semantic Query API

Example:

``` rust
trait SemanticQueries {

    fn resolve_symbol(
        symbol: SymbolId
    ) -> SymbolInfo;

    fn type_of(
        expression: ExpressionId
    ) -> TypeId;

    fn members(
        ty: TypeId
    ) -> Vec<MemberId>;

    fn dispatch(
        receiver: TypeId,
        selector: SelectorId
    ) -> DispatchResult;
}
```

------------------------------------------------------------------------

# 20. Presentation Layer

Separate:

    Semantic Facts

    from

    Human Representation

Example:

    TypeId(500)

            |

    TypeRenderer

            |

    List<User>

The same type can render differently:

    compact:

    User

    expanded:

    phalcom.core.User

------------------------------------------------------------------------

# 21. Rich Hover and IDE Intelligence

With this architecture:

Example:

``` phalcom
orders.map(validate)
```

Hover can display:

    Type:

    List<ValidatedOrder>


    Dispatch:

    Iterable.map


    Inference:

    T = ValidatedOrder


    Contracts:

    Order conforms Entity


    Effects:

    pure

------------------------------------------------------------------------

# 22. Compiler, Runtime, and Tooling Integration

The semantic engine should serve:

    Compiler

    LSP

    Documentation Generator

    Reflection System

    Static Verifier

    Runtime Metadata

One semantic truth source.

------------------------------------------------------------------------

# 23. Testing Strategy

Required tests:

## Semantic database

-   identity stability;
-   dependency invalidation;
-   query caching.

## Type system

-   inference;
-   subtyping;
-   generics;
-   recursive types.

## Dispatch

-   selector resolution;
-   overload selection;
-   contracts.

## LSP

-   hover;
-   completion;
-   diagnostics;
-   rename.

## Performance

Benchmarks:

-   initial analysis;
-   incremental edit;
-   large workspace;
-   completion latency.

------------------------------------------------------------------------

# 24. Migration Strategy

Recommended implementation order:

## Phase 1

Create:

    phalcom-semantic

with:

-   identities;
-   arenas;
-   semantic indexes.

## Phase 2

Create:

    phalcom-query

with:

-   answer tables;
-   dependency graph;
-   incremental evaluation.

## Phase 3

Create:

    phalcom-types

with:

-   TypeId;
-   TypeArena;
-   constraints;
-   normalization.

## Phase 4

Integrate:

    phalcom-lsp

as a query consumer.

------------------------------------------------------------------------

# Final Conclusion

The most important lesson from Pyrefly is:

> The fastest type checker is not one with the cleverest inference
> algorithm. It is one where semantic knowledge is represented once,
> assigned identity, cached, invalidated precisely, and reused
> everywhere.

For Phalcom, this architecture provides the foundation for:

-   optional static typing;
-   precise inference;
-   rich IDE support;
-   contracts;
-   reflection;
-   efficient compilation;
-   future language evolution.

The correct investment is not only a type checker.

It is a semantic intelligence engine for the entire language ecosystem.
