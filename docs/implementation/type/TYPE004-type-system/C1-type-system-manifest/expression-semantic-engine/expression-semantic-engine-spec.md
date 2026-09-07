# Phalcom Functional Typing Phase 2

# Semantic Expression Engine and Message-Based Type Inference Specification

Version: Phase 2

## 1. Purpose

Phase 1 established the semantic typing foundation:

-   canonical semantic identities;
-   interned type representation;
-   kind representation;
-   evidence-based type knowledge;
-   subtype and assignability relations;
-   annotation parsing;
-   initial semantic checking pipeline.

Phase 2 transforms this foundation into a usable semantic inference
engine.

The goal is not to add a large number of typing features. The goal is to
make the type system understand existing Phalcom semantics:

-   objects;
-   selectors;
-   message sends;
-   callable contracts;
-   fields;
-   getters;
-   setters;
-   dynamic behavior.

The central principle:

> The type system must model Phalcom, not create a separate statically
> typed language on top of Phalcom.

------------------------------------------------------------------------

# 2. Current Architecture

The current architecture contains:

    phalcom-ast
          |
          v
    phalcom-semantic
          |
          +--> phalcom-core
          |
          +--> phalcom-lsp

The semantic crate owns:

-   type representation;
-   semantic identities;
-   checking;
-   diagnostics;
-   inference infrastructure.

Phase 2 must extend this architecture rather than create parallel
systems.

------------------------------------------------------------------------

# 3. Phase 2 Objectives

Implement:

1.  Complete expression typing.
2.  Message-send based type resolution.
3.  Selector-aware callable checking.
4.  Field/getter/setter/index typing.
5.  Block and control-flow inference.
6.  Local constraint generation.
7.  Native callable integration.
8.  Semantic snapshot usage for incremental tooling.

------------------------------------------------------------------------

# 4. Typed Expression Model

The current model returns only:

    Expression -> TypeKnowledge

This is insufficient for future inference.

Introduce a semantic result model:

``` rust
struct TypedExpression {
    knowledge: TypeKnowledge,
    constraints: Vec<TypeConstraint>,
    provenance: EvidenceSet,
}
```

The initial implementation may only populate knowledge and provenance.

The architecture must allow constraints to be introduced later.

------------------------------------------------------------------------

# 5. Expression Typing Engine

## 5.1 Primitive literals

Support:

    1          Int
    1.0        Float
    "text"    String
    true       Bool

Literal typing must use semantic declarations through the existing type
resolver.

------------------------------------------------------------------------

## 5.2 Collection literals

Support:

    []

as:

    List<T>

where T is an inference variable.

Example:

    [1,2,3]

produces:

    List<Int>

------------------------------------------------------------------------

## 5.3 Tuple literals

Example:

    (1,"hello")

produces:

    (Int,String)

using the existing tuple type representation.

------------------------------------------------------------------------

## 5.4 Record literals

Example:

    {
        name:"Alice",
        age:30
    }

produces:

    {
        name:String,
        age:Int
    }

------------------------------------------------------------------------

# 6. Block and Control Flow Typing

Blocks must become expression-aware.

Current behavior:

    {
        123
    }

must not become Unit.

The rule:

    Block type =
        tail expression type
        OR Unit
        OR Never

Examples:

    {
        10
    }

returns:

    Int

Empty block:

    {
    }

returns:

    Unit

Diverging block:

    {
        throw error
    }

returns:

    Never

------------------------------------------------------------------------

# 7. Message Send Typing

This is the central Phase 2 feature.

All operations must eventually lower into message semantics.

Example:

    a + b

must behave as:

    a -> +(b)

The checker must not encode arithmetic operators as special cases.

------------------------------------------------------------------------

## 7.1 Send representation

Introduce:

``` rust
SendExpression {
    receiver,
    selector,
    arguments
}
```

Typing flow:

    1. Infer receiver type.

    2. Resolve selector.

    3. Find callable candidates.

    4. Match arguments.

    5. Produce return type.

    6. Record provenance.

------------------------------------------------------------------------

# 8. Dispatch Resolution

Introduce a semantic dispatch abstraction.

Conceptually:

``` rust
trait DispatchResolver {
    fn resolve(
        receiver: TypeId,
        selector: Selector
    ) -> DispatchResult;
}
```

Possible results:

    Found(callables)
    Missing
    Dynamic

The resolver must remain independent from concrete class implementation.

Future dispatch mechanisms must participate:

-   inheritance;
-   traits;
-   interception;
-   native primitives.

------------------------------------------------------------------------

# 9. Callable Signature Model

Replace positional parameter checking.

Current:

    argument[0] -> parameter[0]

is insufficient.

Introduce:

``` rust
CallableSignature {
    selector,
    parameters,
    return_type
}
```

Parameters:

``` rust
CallableParameter {
    external_label,
    local_name,
    type,
    rest
}
```

Argument matching becomes:

    argument label
            |
            v
    parameter external label

------------------------------------------------------------------------

Example:

Declaration:

    move from source to destination

creates:

    from -> Point
    to   -> Point

Call:

    move(from:a,to:b)

matches by label.

------------------------------------------------------------------------

# 10. Member Typing

Unify:

-   fields;
-   getters;
-   setters;
-   indexers.

Introduce:

    MemberResolver

------------------------------------------------------------------------

## Read access

Example:

    object.value

resolution order:

    Field
    Getter
    Intercepted message

------------------------------------------------------------------------

## Write access

Example:

    object.value = x

resolution order:

    Field setter
    Setter method
    Error

------------------------------------------------------------------------

## Index access

Example:

    map[key]

must resolve through semantic member lookup rather than special compiler
logic.

------------------------------------------------------------------------

# 11. Constraint System

The repository already contains infrastructure for future inference
variables.

Activate local constraints.

Introduce:

``` rust
enum TypeConstraint {
    Equal(TypeExpr, TypeExpr),
    Subtype(TypeExpr, TypeExpr),
    HasMember(TypeExpr, Selector)
}
```

------------------------------------------------------------------------

Example:

    let x = []

    x.add(1)

Initial:

    x : List<T>

Constraint:

    T = Int

Solved:

    x : List<Int>

------------------------------------------------------------------------

# 12. Native Metadata Integration

Native methods must enter through the same semantic path.

Target:

    Native metadata

          |

    CallableSignature

          |

    Dispatch resolver

          |

    Type checker

No native-specific checker branches should exist.

------------------------------------------------------------------------

# 13. Semantic Snapshots and Incremental Analysis

Phase 1 introduced:

-   semantic snapshots;
-   declaration fingerprints;
-   invalidation structures.

Phase 2 should connect them.

Target:

    Source change

          |

    Declaration invalidation

          |

    Recompute affected semantic facts

          |

    Publish diagnostics and LSP information

------------------------------------------------------------------------

# 14. Testing Requirements

## Expression tests

Examples:

    let x = 1

expects:

    Int

------------------------------------------------------------------------

    let values = [1,2]

expects:

    List<Int>

------------------------------------------------------------------------

    {
        1
    }

expects:

    Int

------------------------------------------------------------------------

## Dispatch tests

Verify:

    1 + 2

uses selector resolution:

    #+(_)

rather than arithmetic-specific handling.

------------------------------------------------------------------------

## Callable tests

Verify:

    move(from:a,to:b)

matches labels.

------------------------------------------------------------------------

## Dynamic tests

Must remain valid:

    let x: Int = dynamic_value

No false rejection.

------------------------------------------------------------------------

# 15. Implementation Order

## Milestone 2.1

Expression engine:

-   literals;
-   collections;
-   tuples;
-   records;
-   blocks.

------------------------------------------------------------------------

## Milestone 2.2

Message semantics:

-   send representation;
-   dispatch resolution;
-   callable signatures.

------------------------------------------------------------------------

## Milestone 2.3

Object semantics:

-   fields;
-   getters;
-   setters;
-   indexes.

------------------------------------------------------------------------

## Milestone 2.4

Inference:

-   inference variables;
-   constraints;
-   local solver.

------------------------------------------------------------------------

## Milestone 2.5

Tooling:

-   semantic snapshots;
-   invalidation;
-   provenance;
-   richer LSP information.

------------------------------------------------------------------------

# Final Design Rule

Phase 2 must preserve the following semantic hierarchy:

    Objects
       |
    Selectors
       |
    Dispatch
       |
    Callable contracts
       |
    Types

Types describe Phalcom semantics.

They do not replace them.
