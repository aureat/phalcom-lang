# Pyrefly Architecture Analysis for Phalcom

# Document 3 (Expanded) --- Semantic Intelligence Engine Blueprint

## Purpose

This document is an expanded revision of Document 3 after another
repository archaeology pass over Pyrefly.

The previous version described the architectural direction. This version
strengthens it with implementation-grounded observations:

-   Pyrefly's actual module-centric incremental model;
-   graph/calculation abstractions;
-   semantic pipeline boundaries;
-   why Pyrefly chooses coarse-grained incrementality over Salsa-style
    fine-grained dependency tracking;
-   how these choices should be adapted rather than copied into Phalcom.

Pyrefly's architecture documentation explicitly describes three major
stages:

1.  determine module exports;
2.  convert modules into bindings with scope and flow information;
3.  solve bindings, including cross-module dependencies.

It also explicitly states that Pyrefly does not use fine-grained
incrementality like Rust Analyzer/Salsa, instead choosing module-level
incrementality and raw performance. This is a critical design lesson for
Phalcom.

------------------------------------------------------------------------

# 1. Central Architectural Lesson

The mistake would be to copy Pyrefly as a "type checker".

The correct extraction is:

> Pyrefly is a semantic computation engine. The type checker is one
> consumer of that engine.

The architecture should become:

                      Source

                        |

                     Parser

                        |

              Semantic Index Database

                        |

                 Query / Calculation Graph

                        |

           +------------+------------+

           |                         |

     Type Solver              Presentation Layer

           |                         |

     Compiler                 LSP / Docs / Tools

The semantic layer becomes the shared truth source.

------------------------------------------------------------------------

# 2. Module-Centric Incrementality

Pyrefly makes an unusual but important tradeoff.

Many modern IDE engines use extremely fine-grained incremental systems:

    single syntax node changed

            |

    invalidate exact dependent nodes

Pyrefly instead chooses:

    module changed

            |

    recompute module semantic state

            |

    reuse unaffected modules

The repository architecture documentation explicitly describes this
decision:

-   large-scale incrementality;
-   module-level solving;
-   parallelism;
-   avoiding fine-grained Salsa-style incrementality.

This is important because Phalcom has a different optimization target.

For a language with:

-   reflection;
-   contracts;
-   message dispatch;
-   type/value unification;

a very fine-grained dependency graph could become more expensive than
recomputation.

Recommendation:

Use hybrid granularity:

    Project

     |

    Module semantic units

     |

    Declaration-level cached queries

     |

    Expression-level ephemeral queries

------------------------------------------------------------------------

# 3. Semantic Database Design

Create:

    phalcom-semantic

Responsibilities:

-   declarations;
-   symbols;
-   modules;
-   selectors;
-   dispatch targets;
-   contracts;
-   type relationships;
-   reflection metadata.

It should not own:

-   code generation;
-   diagnostics rendering;
-   IDE formatting.

------------------------------------------------------------------------

# 4. Stable Identity Model

Every important semantic object needs identity.

Recommended:

``` rust
ModuleId

DeclarationId

BindingId

ExpressionId

SelectorId

TypeId

ContractId

DispatchId

QueryId
```

Identity enables:

-   caching;
-   invalidation;
-   provenance;
-   cross-tool communication.

------------------------------------------------------------------------

# 5. Semantic Facets

Phalcom differs from Python.

Python separates:

    runtime object

    +

    static type

Phalcom intends:

    value semantics

    =

    type semantics

Therefore entities should expose facets:

    SemanticEntity

        ValueFacet

        TypeFacet

        DispatchFacet

        ContractFacet

        ReflectionFacet

Example:

    User

    Value:
        constructor

    Type:
        User

    Dispatch:
        save(_)

    Reflection:
        fields, methods

    Contract:
        Entity

------------------------------------------------------------------------

# 6. Query Engine Architecture

Pyrefly's graph layer contains cached calculations.

The important abstraction is:

    Calculation<T>

A calculation:

-   may not have been computed;
-   may currently be computing;
-   may have a final cached result.

It also handles recursive computation.

This pattern maps directly to Phalcom:

    Query<Key, Value>

    states:

    NotComputed

    Computing

    Complete

------------------------------------------------------------------------

# 7. Recursive Query Handling

Semantic systems are cyclic.

Examples:

    class A extends B

    class B extends A

or:

    type Tree = Node<Tree>

The engine must not deadlock.

The correct model:

    Query A

    requires

    Query B

    requires

    Query A


    insert placeholder

    solve fixed point

Phalcom should use:

    InferenceVar

    +

    DeferredAnswer

    +

    FixedPointSolver

------------------------------------------------------------------------

# 8. Binding Graph

Pyrefly transforms source into semantic bindings.

Example:

    x: int = 4

becomes concepts:

    define x

    value expression

    use references

    export information

Phalcom should introduce a similar intermediate representation:

    SemanticBindingGraph

before type solving.

This allows:

-   control flow;
-   imports;
-   dispatch;
-   contracts;

to operate on stable semantic objects.

------------------------------------------------------------------------

# 9. Type Universe

Create:

    phalcom-types

with:

    TypeId

    TypeArena

    TypeData

    TypeNormalizer

    SubtypeEngine

Example:

``` rust
enum TypeData {

 Primitive,

 Class {
    declaration: DeclarationId,
    arguments: Vec<TypeId>
 },

 Function {
    params: Vec<TypeId>,
    result: TypeId
 },

 Application {
    constructor: TypeId,
    arguments: Vec<TypeId>
 }

}
```

------------------------------------------------------------------------

# 10. Type Interning

Equivalent types should have one identity.

Example:

    List<Int>

always becomes:

    TypeId(500)

Benefits:

-   constant-time equality;
-   better cache locality;
-   smaller memory usage;
-   faster LSP queries.

------------------------------------------------------------------------

# 11. Constraint Solver

Do not infer directly.

Use:

    Expression

     |

    Constraint generation

     |

    Constraint graph

     |

    Solver

     |

    TypeId

This supports:

-   generics;
-   higher-kinded types;
-   contracts;
-   overloaded dispatch.

------------------------------------------------------------------------

# 12. Semantic Ordering Boundary

Subtype logic should not contain program knowledge.

Create:

    SemanticOrder

Responsibilities:

    is_subtype()

    conforms()

    members()

    dispatch_candidates()

    contracts()

The solver understands rules.

The semantic database understands facts.

------------------------------------------------------------------------

# 13. Selector Dispatch Integration

Phalcom's equivalent of Python attribute lookup is selector resolution.

Use:

    DispatchQuery

Input:

    receiver TypeId

    selector SelectorId

Output:

    DispatchTarget

    or

    AmbiguousDispatch

    or

    MissingDispatch

Example:

    account.save()

becomes:

    Account

    +

    save(_)

    =

    AccountRepository.save

------------------------------------------------------------------------

# 14. LSP Architecture

The LSP should not contain analysis logic.

Architecture:

    Semantic Database

           |

    Query API

           |

    Presentation Layer

           |

    LSP

The LSP asks:

    TypeOf(expression)

    Members(type)

    Definition(symbol)

    Dispatch(receiver, selector)

------------------------------------------------------------------------

# 15. Semantic Presentation Layer

Keep:

    semantic facts

    separate from

    human rendering

Example:

    TypeId(500)

           |

    Renderer

           |

    List<User>

This allows:

-   compact display;
-   verbose explanations;
-   documentation generation;
-   different IDE clients.

------------------------------------------------------------------------

# 16. Inference Provenance

The semantic engine should retain:

    Why was this answer produced?

Example:

    Type:

    List<Order>


    Evidence:

    - list literal element type
    - append constraint
    - contract satisfaction

This enables exceptional IDE experiences.

------------------------------------------------------------------------

# 17. Diagnostics

Diagnostics should be semantic queries.

Not:

    LSP runs checker

but:

    DiagnosticQuery

            |

    Diagnostic objects

            |

    LSP rendering

The compiler, CLI, and IDE share diagnostics.

------------------------------------------------------------------------

# 18. Performance Strategy

The main optimizations to adopt:

## Stable identities

Avoid repeated discovery.

## Cached semantic calculations

Avoid repeated solving.

## Coarse-grained invalidation

Avoid dependency explosion.

## Parallel module analysis

Exploit independent work.

## Canonical types

Avoid duplicate structures.

------------------------------------------------------------------------

# 19. Implementation Roadmap

## Phase 1

Create:

    phalcom-semantic

Implement:

-   IDs;
-   arenas;
-   declaration index.

## Phase 2

Create:

    phalcom-query

Implement:

-   cached calculations;
-   dependencies;
-   invalidation.

## Phase 3

Create:

    phalcom-types

Implement:

-   TypeId;
-   TypeArena;
-   constraints;
-   subtype.

## Phase 4

Integrate:

    phalcom-lsp

as a query client.

------------------------------------------------------------------------

# Final Conclusion

The deepest lesson from Pyrefly is not a specific type algorithm.

It is architectural discipline:

1.  discover semantic facts once;
2.  assign stable identity;
3.  store answers;
4.  invalidate precisely;
5.  expose everything through queries.

For Phalcom, this becomes the foundation for:

-   optional static typing;
-   powerful inference;
-   contracts;
-   reflection;
-   IDE intelligence;
-   compilation;
-   runtime optimization.

The correct long-term architecture is not a type checker attached to a
compiler.

It is a semantic intelligence engine shared by the entire language
ecosystem.
