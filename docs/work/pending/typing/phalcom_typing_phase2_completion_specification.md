# Phalcom Typing Phase 2 Completion Specification

## Repository-Grounded Implementation Plan

## Status

This document defines the implementation direction for completing
Phalcom's first functional typing phase.

The goal is not to transform Phalcom into a mandatory statically typed
language. The goal is to create a persistent semantic type layer over
the dynamic runtime.

Types are semantic information. They are preserved, queried, reflected
upon, and used by the compiler and IDE.

------------------------------------------------------------------------

## 1. Core Architecture

The repository already contains two important foundations:

-   an incremental semantic analysis system used by the language server;
-   a canonical semantic/type system in `phalcom-semantic`.

The remaining implementation work unifies them.

The final architecture is:

    Source
     |
     AST
     |
     Semantic Database
     |
     +----------------+
     |                |
    Compiler          LSP

The semantic database is the authority for program meaning.

------------------------------------------------------------------------

## 2. Semantic Database

The final semantic snapshot contains:

-   source information;
-   modules;
-   declarations;
-   scopes;
-   declaration surfaces;
-   dispatch information;
-   type store;
-   inference facts;
-   flow facts;
-   diagnostics;
-   invalidation dependencies.

The snapshot remains immutable. New analysis generations create new
snapshots.

------------------------------------------------------------------------

## 3. Type Knowledge

All semantic values are represented by type knowledge.

Conceptually:

``` rust
enum TypeKnowledge {
    Known {
        type_id: TypeId,
        evidence: EvidenceSet,
    },
    Unknown {
        reason: UnknownReason,
    },
    Dynamic,
    Never,
}
```

Unknown means the compiler lacks evidence.

Dynamic means the developer intentionally requests dynamic behavior.

------------------------------------------------------------------------

## 4. Annotation Lowering

Existing annotation syntax is lowered into canonical types.

Examples:

    Int

becomes:

    TypeId(Int)

    List<User>

becomes:

    Applied(List, User)

    (Int) -> String

becomes:

    Callable(Int -> String)

No type information is erased.

------------------------------------------------------------------------

## 5. Declaration Signatures

Every callable declaration receives a semantic signature.

Example:

    rename(to value: String) -> String

produces:

    parameters:
        value: String

    return:
        String

These signatures are consumed by:

-   compiler;
-   LSP;
-   future runtime contracts.

------------------------------------------------------------------------

## 6. Expression Typing

Every expression produces TypeKnowledge.

Literal:

    42

produces:

    Known(Int)

Calls resolve through dispatch:

    receiver type
     |
    dispatch resolver
     |
    callable signature
     |
    return type

Operators remain ordinary message sends.

    a + b

is typed through:

    +(_)

------------------------------------------------------------------------

## 7. Assignability

All checking uses centralized assignability.

Never:

    actual == expected

Always:

    check_assignability(actual, expected)

The result may be:

-   assignable;
-   refuted;
-   uncertain.

------------------------------------------------------------------------

## 8. Flow Integration

Control-flow analysis refines types.

Example:

    if value != None

changes:

    Option<User>

into:

    User

inside the branch.

------------------------------------------------------------------------

## 9. LSP Integration

The language server becomes a consumer of semantic truth.

It queries:

-   type of expression;
-   callable signature;
-   members of type;
-   diagnostics.

Hover, completion, and inlay hints all use the same semantic model.

------------------------------------------------------------------------

## 10. Completion Criteria

Phase 2 is complete when:

-   annotations are stored and resolved;
-   inferred types are represented;
-   declarations expose signatures;
-   calls are checked through dispatch;
-   flow refines types;
-   compiler and LSP share semantics;
-   runtime remains dynamically executed.
