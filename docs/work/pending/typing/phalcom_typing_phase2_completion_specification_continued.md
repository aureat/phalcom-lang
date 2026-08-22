# Phalcom Typing Phase 2 Completion Specification

## Repository-Grounded Implementation Continuation

## Status of This Artifact

This document is an expanded engineering specification draft for
completing the functional Phase 2 typing layer.

The purpose of this phase is to connect the existing semantic
infrastructure, type representation, inference machinery, and language
tooling into one coherent semantic system.

This document intentionally separates:

-   repository-derived architectural observations;
-   implementation recommendations;
-   future design extensions.

The implementation target is a dynamic runtime with a strong semantic
layer.

------------------------------------------------------------------------

# 1. Phase 2 Completion Goal

The completed Phase 2 system must establish the following invariant:

Every meaningful semantic entity in a Phalcom program has a stable
representation that can answer:

-   what declaration does this refer to?
-   what type information is known?
-   where did that information originate?
-   is the information proven, inferred, unknown, or intentionally
    dynamic?
-   what diagnostics follow from combining declared intent and inferred
    behavior?

The compiler and language server must query the same semantic model.

------------------------------------------------------------------------

# 2. Existing Architecture Direction

The repository direction contains two important layers:

## Existing semantic analysis

The language tooling already requires:

-   incremental analysis;
-   declaration surfaces;
-   flow information;
-   callable summaries;
-   dependency tracking;
-   invalidation.

## Existing type semantics

The semantic type layer already requires:

-   canonical type identities;
-   type storage;
-   subtype relations;
-   assignability;
-   evidence-backed knowledge.

Phase 2 completion joins these systems.

------------------------------------------------------------------------

# 3. Target Architecture

The target architecture is:

    Source Files

          |

    AST

          |

    Semantic Database

          |
          +----------------+
          |                |

     Compiler             LSP

The semantic database is not an implementation detail. It is the
language's semantic memory.

------------------------------------------------------------------------

# 4. Semantic Snapshot

The snapshot must become the immutable published representation of
analysis.

Conceptual structure:

``` rust
pub struct SemanticSnapshot {
    pub generation: SemanticGeneration,

    pub sources: SourceDatabase,

    pub modules: ModuleDatabase,

    pub declarations: DeclarationDatabase,

    pub scopes: ScopeDatabase,

    pub surfaces: SurfaceDatabase,

    pub types: TypeStoreSnapshot,

    pub inference: InferenceDatabase,

    pub flow: FlowDatabase,

    pub diagnostics: DiagnosticDatabase,

    pub dependencies: DependencyGraph,
}
```

The snapshot must not contain temporary mutable analysis state.

------------------------------------------------------------------------

# 5. Type Knowledge Model

Phalcom does not treat types as erased compiler hints.

Types are persistent semantic facts.

Conceptually:

``` rust
pub enum TypeKnowledge {
    Known {
        ty: TypeId,
        evidence: EvidenceSet,
    },

    Unknown {
        reason: UnknownReason,
    },

    Dynamic,

    Never,
}
```

------------------------------------------------------------------------

# 6. Evidence Model

Every type fact records provenance.

Examples:

## Explicit annotation

``` phalcom
let user: User = value
```

Evidence:

    authority:
        declared

    source:
        user annotation

## Literal inference

``` phalcom
let x = 10
```

Evidence:

    authority:
        inferred

    source:
        integer literal

## Future runtime validation

    authority:
        runtime

    source:
        debug contract observation

------------------------------------------------------------------------

# 7. Annotation Lowering

The AST already provides the syntax layer.

The semantic layer completes lowering.

## Reference

Input:

    User

Output:

    TypeId(User)

## Application

Input:

    List<User>

Output:

    Applied(List, User)

## Callable

Input:

    (Int) -> String

Output:

    Callable(
        parameter = Int,
        result = String
    )

## Tuple

Input:

    (Int, String)

Output:

    Tuple(Int, String)

------------------------------------------------------------------------

# 8. Declaration Signatures

All callable declarations must expose canonical signatures.

Example:

``` phalcom
rename(to value: String) -> String
```

Semantic form:

    selector:
        rename(to:)

    parameters:
        value:
            String

    return:
        String

This signature is used by:

-   call checking;
-   completion;
-   hover;
-   documentation;
-   future optimization.

------------------------------------------------------------------------

# 9. Expression Typing Pipeline

Every expression follows:

    Expression

        |

    Semantic analysis

        |

    TypeKnowledge

        |

    Diagnostics and constraints

------------------------------------------------------------------------

# 10. Member Access

Example:

``` phalcom
user.name
```

Algorithm:

1.  Determine receiver TypeId.
2.  Ask declaration surface for members.
3.  Resolve field/getter.
4.  Return member type.

The checker must not inspect syntax-only information.

------------------------------------------------------------------------

# 11. Call Checking

Example:

``` phalcom
user.rename("Bob")
```

Pipeline:

    receiver type

            |

    dispatch resolution

            |

    callable signature

            |

    argument validation

            |

    return type

------------------------------------------------------------------------

# 12. Operator Semantics

Operators remain message sends.

Example:

``` phalcom
a + b
```

means:

    a.receive +(b)

The type system does not contain special primitive rules for operators.

------------------------------------------------------------------------

# 13. Assignability

All compatibility checking uses the central relation engine.

The checker must not perform direct equality checks.

The question is:

    Can actual satisfy expected?

not:

    Are they identical?

------------------------------------------------------------------------

# 14. Flow Refinement

Flow analysis contributes type evidence.

Example:

``` phalcom
if value != None {
    value.name
}
```

Before branch:

    value:
        Option<User>

Inside branch:

    value:
        User

------------------------------------------------------------------------

# 15. LSP Integration

The LSP becomes a consumer.

Required queries:

    type_of_expression()

    signature_of_callable()

    members_of_type()

    diagnostics_at_location()

Features:

-   richer hover;
-   accurate completion;
-   inline annotations;
-   semantic diagnostics.

------------------------------------------------------------------------

# 16. Implementation Sequence

The implementation should proceed in this order:

1.  Consolidate semantic ownership.
2.  Expand semantic snapshot.
3.  Complete annotation lowering.
4.  Attach declared types to surfaces.
5.  Convert expression inference to TypeKnowledge.
6.  Integrate dispatch.
7.  Integrate flow refinement.
8.  Expose semantic queries to LSP.
9.  Add compiler diagnostics.
10. Add regression tests.

------------------------------------------------------------------------

# 17. Verification Criteria

Phase 2 is complete when:

-   compiler and LSP agree on types;
-   annotations are preserved;
-   inferred types are queryable;
-   unknown differs from dynamic;
-   dispatch participates in checking;
-   diagnostics explain contradictions;
-   runtime remains dynamic.
