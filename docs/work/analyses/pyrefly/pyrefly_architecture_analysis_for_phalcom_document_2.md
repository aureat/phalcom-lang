# Pyrefly Architecture Analysis for Phalcom

## Document 2 --- Type System, Solver Architecture, and LSP Semantic Query Design

## Purpose

This document continues the architectural analysis of Pyrefly, focusing
on the deeper implementation choices behind its static type checker:

-   type representation;
-   constraint solving;
-   subtype checking;
-   generic inference;
-   recursion handling;
-   LSP integration.

The goal is not to reproduce Python typing semantics in Phalcom. The
goal is to extract architectural techniques that allow a difficult
dynamic language to receive deep static analysis while remaining fast
and responsive.

The central lesson from this document:

> A high-performance type checker is not primarily a type inference
> algorithm. It is a semantic computation system that manages
> identities, constraints, cached answers, and explanations.

------------------------------------------------------------------------

# 1. Type Checking Dynamic Languages Requires a Semantic Database

Python creates a difficult static analysis problem:

-   objects are dynamic;
-   imports are flexible;
-   attributes are discovered dynamically;
-   classes participate in runtime behavior;
-   generic relationships can become recursive.

A naive architecture repeatedly asks:

    What is this object?

    What members does it have?

    What type does this expression produce?

Pyrefly avoids this by building semantic facts first.

The architecture becomes:

    Source

     |

    AST

     |

    Semantic Index

     |

    Type Queries

     |

    Constraint Solver

     |

    Canonical Type Answers

The type checker operates on a semantic world that has already been
indexed.

------------------------------------------------------------------------

# 2. Type Representation: Types Are Semantic Values

A naive type representation:

``` rust
enum Type {
    List(Box<Type>),
    Function(Box<Type>, Box<Type>)
}
```

creates major problems:

-   recursive ownership;
-   expensive cloning;
-   poor equality performance;
-   difficult caching.

Pyrefly introduces a type heap abstraction.

Conceptually:

    TypeId

     |

    TypeHeap

     |

    Canonical Type Data

The important design decision:

Types should have stable identity.

------------------------------------------------------------------------

# 3. Phalcom Adaptation: TypeArena and TypeId

Phalcom should use:

``` rust
struct TypeId(u32);

struct TypeArena {
    types: Vec<TypeData>
}
```

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

Benefits:

-   cheap equality;
-   fast hashing;
-   stable references;
-   recursive type support;
-   efficient caching.

------------------------------------------------------------------------

# 4. Generic Types Are Semantic Objects

Pyrefly treats generic parameters as first-class semantic entities.

A type parameter is not merely a placeholder.

It carries:

-   origin;
-   constraints;
-   bounds;
-   variance;
-   scope.

Example:

    class Box[T]

    function identity[T]

    type Alias[T]

These are different semantic situations.

------------------------------------------------------------------------

# 5. Phalcom Adaptation: Generic Variables Need Provenance

Phalcom should represent:

    TypeVariableId

    +

    Origin

    +

    Constraints

    +

    Variance

    +

    Scope

Example:

    T created from:

    method map()

    belongs to:

    FunctionDeclarationId(42)

    constraints:

    User conforms Entity

This enables better diagnostics and LSP explanations.

------------------------------------------------------------------------

# 6. Constraint-Based Inference

Pyrefly does not directly assign types while traversing expressions.

Instead:

    Expression

     |

    Constraint Generation

     |

    Constraint Graph

     |

    Solver

     |

    Type Answer

Example:

``` phalcom
identity(10)
```

creates:

    T

    argument <: T

    return = T

The solver later decides:

    T = Int

------------------------------------------------------------------------

# 7. Why Constraint Solving Matters for Phalcom

Phalcom plans:

-   generics;
-   higher-kinded types;
-   monadic abstractions;
-   contracts;
-   optional verification.

Direct inference becomes fragile.

Constraint solving allows:

    unknown information

    +

    later discoveries

    =

    consistent solution

------------------------------------------------------------------------

# 8. Subtyping Is Separate From Type Representation

Pyrefly introduces a semantic ordering layer.

The subtype engine does not know the entire program.

Instead:

    Subtype Algorithm

            |

    TypeOrder abstraction

            |

    Semantic Database

The solver asks:

-   Is this class a subtype?
-   Does this protocol apply?
-   What members exist?

The semantic layer answers.

------------------------------------------------------------------------

# 9. Phalcom Adaptation: SemanticOrder

Phalcom should introduce a similar boundary:

``` rust
trait SemanticOrder {

    fn conforms(
        actual: TypeId,
        expected: TypeId
    );

    fn dispatch_candidates(
        receiver: TypeId,
        selector: SelectorId
    );

    fn contract_of(
        declaration: DeclarationId
    );
}
```

Why:

The type theory stays independent from language implementation details.

------------------------------------------------------------------------

# 10. Recursive Types Need Explicit Safety

Type systems naturally contain cycles:

    A -> B -> C -> A

or:

    Node[List[Node]]

Pyrefly protects itself using:

-   recursion guards;
-   computation budgets;
-   bounded solving.

A compiler should not crash because a user writes a pathological type.

------------------------------------------------------------------------

# 11. Phalcom Adaptation: Semantic Budgets

Introduce:

``` rust
struct AnalysisBudget {

    max_depth,

    max_constraints,

    max_dispatch_expansions,

}
```

Use this for:

-   recursive contracts;
-   recursive types;
-   dispatch expansion;
-   generic normalization.

------------------------------------------------------------------------

# 12. Canonical Type Normalization

Pyrefly performs normalization of:

-   unions;
-   intersections;
-   equivalent structures.

Example:

    Int | Int

becomes:

    Int

This improves:

-   equality;
-   caching;
-   solver stability.

------------------------------------------------------------------------

# 13. Phalcom Adaptation: TypeNormalizer

Introduce:

    TypeNormalizer

        normalize(TypeId)

Responsibilities:

-   flatten unions;
-   remove duplicates;
-   normalize generic applications;
-   preserve canonical TypeIds.

------------------------------------------------------------------------

# 14. Inference Provenance

One of the most valuable ideas for developer experience is preserving
why a type was inferred.

Instead of:

    Type:
    List<User>

provide:

    Type:
    List<User>

    Reason:

    - list literal contained User values
    - append(User) established element type
    - contract Entity was satisfied

------------------------------------------------------------------------

# 15. LSP Architecture: The IDE Is a Query Client

Pyrefly's LSP does not implement a second analyzer.

The architecture:

                    Semantic Database

                           |

              +------------+------------+

              |                         |

          Compiler                   LSP

The LSP asks semantic questions.

Examples:

    TypeOf(ExpressionId)

    Resolve(SymbolId)

    Members(TypeId)

------------------------------------------------------------------------

# 16. Phalcom LSP Architecture

Phalcom should follow:

                     Semantic Engine

                           |

                  Query Infrastructure

                           |

           +---------------+---------------+

           |                               |

       Compiler                         LSP

The LSP should never independently infer types.

------------------------------------------------------------------------

# 17. Hover Architecture

Bad:

    Hover request

        |
    run inference

Good:

    Hover request

        |

    TypeOf(ExpressionId)

        |

    Cached TypeId

        |

    Render explanation

------------------------------------------------------------------------

# 18. Rich Phalcom Hover Possibilities

Because Phalcom will have richer semantics:

    User.save()

could display:

    Method:
        save()

    Receiver:
        User

    Dispatch:
        UserPersistence.save

    Contract:
        requires persisted entity

    Effects:
        IO

    Inference:
        receiver matched Repository<User>

------------------------------------------------------------------------

# 19. Completion Architecture

Completion should be:

    cursor position

     |

    semantic context

     |

    receiver TypeId

     |

    member query

     |

    completion items

Not text guessing.

------------------------------------------------------------------------

# 20. Final Architecture Extracted From Pyrefly

The recommended Phalcom architecture:

                             Source

                               |

                             AST

                               |

                  Semantic Index Database

                               |

                        Query Engine

                               |

                 +-------------+-------------+

                 |                           |

          Type Solver                 LSP Queries


                 |

            Canonical Types

                 |

          Compiler / Tooling / Runtime

------------------------------------------------------------------------

# Final Conclusions

The most important ideas taken from Pyrefly:

  Pyrefly Concept              Phalcom Adaptation
  ---------------------------- ------------------------------
  Type heap                    Type arena
  Indexed identities           TypeId and semantic IDs
  Constraint solver            Unified type inference
  TypeOrder                    SemanticOrder
  Generic witnesses            Inference provenance
  Recursion guards             Analysis budgets
  Type normalization           Canonical type universe
  LSP over semantic database   Unified tooling architecture

The central architectural lesson:

> Build the semantic knowledge engine first. The compiler, LSP,
> documentation system, and runtime reflection layer should all become
> clients of that engine.

This is the architecture that allows a highly dynamic language to
provide deep static intelligence without sacrificing performance.
