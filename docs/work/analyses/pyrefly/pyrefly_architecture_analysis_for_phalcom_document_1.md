# Pyrefly Architecture Analysis for Phalcom

## Document 1 --- Building a High-Performance Static Type System Through a Semantic Database Architecture

## Purpose

This document records the architectural findings from investigating
Pyrefly, a high-performance Python static type checker implemented in
Rust, and translates those findings into architectural guidance for
Phalcom.

The focus is not on copying Python typing semantics. Phalcom is a
different language with different goals: unified value/type semantics,
message dispatch, contracts, reflection, lazy evaluation, and optional
but powerful static verification.

The purpose is to understand how Pyrefly solves the fundamental
contradiction:

> Python is highly dynamic, but developers still expect deep static
> analysis, IDE intelligence, and fast feedback.

Pyrefly achieves this not by making inference shallow, but by changing
the architecture of the checker itself.

The central conclusion:

**A modern static analyzer should not be designed as a compiler pass. It
should be designed as an incremental semantic database.**

------------------------------------------------------------------------

# 1. Pyrefly's Core Architectural Idea

A traditional type checker looks like:

    Source
     |
    AST
     |
    Resolve symbols
     |
    Infer types
     |
    Check constraints
     |
    Emit diagnostics

This architecture becomes problematic for dynamic languages because
every operation depends on previous semantic discoveries.

Pyrefly instead resembles:

    Source
     |
    AST
     |
    Semantic indexing
     |
    Semantic graph
     |
    Demand-driven queries
     |
    Cached answers
     |
    Diagnostics / IDE / tooling

The analyzer is not repeatedly "checking a program".

It is maintaining a database of semantic facts.

Examples of facts:

-   What declaration does this name refer to?
-   What type does this expression have?
-   What methods exist on this object?
-   What overload was selected?
-   What constraints are known?
-   What dependencies exist?

Each fact receives a stable identity and can be computed lazily.

------------------------------------------------------------------------

# 2. Stable Semantic Identity

## What Pyrefly does

Pyrefly aggressively avoids using strings and large objects during
analysis.

Instead, semantic objects become indexed identities:

    Name
     |
     v
    Binding index
     |
     v
    Semantic object

The repository introduces indexed identity objects such as:

-   `Idx<K>`
-   `CalcId`
-   binding indexes
-   type indexes

These are compact integer-based identifiers.

The graph layer uses a `NonZeroU32` backed index representation.

## Why this matters

A dynamic language produces enormous amounts of semantic lookup.

A naive implementation repeatedly performs:

    lookup string
    hash
    compare
    follow pointers

A high-performance implementation performs:

    array[index]

The difference compounds across millions of type checks.

------------------------------------------------------------------------

# Phalcom adaptation

Phalcom should make the following identities foundational:

    ModuleId
    DeclarationId
    ExpressionId
    SelectorId
    TypeId
    ContractId
    DispatchId
    QueryId

Every subsystem should communicate through these identities.

Instead of:

    find class named "User"

the semantic layer should already know:

    ClassDeclarationId(153)

## Benefit

This improves:

-   dispatch lookup;
-   reflection;
-   LSP latency;
-   incremental recompilation;
-   type inference caching.

Phalcom has more semantic complexity than Python. Stable identities
become even more important.

------------------------------------------------------------------------

# 3. Semantic Indexing Before Type Checking

## What Pyrefly does

Pyrefly separates binding construction from solving.

The binding layer discovers:

-   scopes;
-   declarations;
-   imports;
-   relationships;
-   semantic metadata.

The solver consumes this prepared representation.

The solver does not repeatedly walk syntax trees.

------------------------------------------------------------------------

# Phalcom adaptation

Create a semantic indexing phase:

    Parser
     |
    AST
     |
    Semantic Index Builder
     |
    Semantic Database
     |
    Type Checker

The index builder should answer:

-   What declaration does this selector refer to?
-   What members exist?
-   What contracts exist?
-   What inheritance/conformance relationships exist?
-   What dispatch candidates exist?

The type checker should answer:

-   Are these types compatible?
-   Can this dispatch succeed?
-   Does this contract hold?

------------------------------------------------------------------------

# Why this improves Phalcom

Phalcom's semantics are richer than Python:

-   selectors;
-   pattern selectors;
-   message dispatch;
-   contracts;
-   attributes;
-   reflection;
-   lazy values.

If type checking discovers semantics and verifies semantics
simultaneously, complexity explodes.

Separating them creates reusable infrastructure.

The same semantic database can serve:

-   compiler;
-   LSP;
-   documentation;
-   reflection tooling;
-   runtime metadata.

------------------------------------------------------------------------

# 4. Demand-Driven Query Architecture

## What Pyrefly does

Pyrefly does not eagerly infer everything.

It computes facts only when requested.

A query has:

    Unknown
     |
    Computing
     |
    Complete

The result is stored permanently until invalidated.

Examples:

    TypeOf(ExpressionId)
    Resolve(SymbolId)
    Members(TypeId)

------------------------------------------------------------------------

# Phalcom adaptation

Introduce a query engine:

    Semantic Query

    resolve_declaration()
    infer_type()
    dispatch_method()
    compute_contract()
    compute_effect()

Each query produces a cached answer.

Example:

    TypeOf(ExpressionId(400))
            |
            v
    AnswerSlot<Type>

------------------------------------------------------------------------

# Why this improves Phalcom

Phalcom wants:

-   rich IDE support;
-   optional typing;
-   exact inference;
-   contracts;
-   static verification.

A developer does not need the whole program analyzed immediately.

When hovering:

    foo.bar()

only the necessary semantic facts should be computed.

------------------------------------------------------------------------

# 5. Answer Tables Instead of Ordinary Caches

## What Pyrefly does

Pyrefly uses answer tables.

A computation has:

-   identity;
-   state;
-   dependencies;
-   result.

It is not:

    HashMap<Node, Type>

It is:

    QueryId
     |
     v
    AnswerSlot
     |
     v
    Result

------------------------------------------------------------------------

# Phalcom adaptation

Introduce:

    SemanticAnswerTable

    QueryKey
     |
     AnswerSlot
     |
     Cached Result

Examples:

    ExpressionTypeQuery
    DispatchQuery
    ConstraintQuery
    ContractQuery

------------------------------------------------------------------------

# Why this improves Phalcom

Future Phalcom features are expensive:

-   higher-kinded types;
-   type constructors;
-   contracts;
-   effects;
-   overload dispatch;
-   reflection.

Without persistent answers, these features become unusably slow.

------------------------------------------------------------------------

# 6. Incremental Dependency Tracking

## What Pyrefly does

Pyrefly tracks dependencies at fine granularity.

Not:

    Module A depends on Module B

but:

    A depends on:

    B.User type
    B.User constructor
    B.User metadata

When something changes, only affected computations are invalidated.

------------------------------------------------------------------------

# Phalcom adaptation

The dependency graph should track:

    DeclarationId
     |
    depends on
     |
    QueryId

Examples:

    Method signature changed

            |
            v

    invalidate dispatch queries

            |
            v

    invalidate affected type checks

------------------------------------------------------------------------

# Why this improves Phalcom

Phalcom will eventually have large projects with:

-   modules;
-   packages;
-   contracts;
-   generated metadata.

Whole-program invalidation will not scale.

------------------------------------------------------------------------

# 7. Type Representation Strategy

## What Pyrefly suggests

Large type systems should avoid recursive heap allocations.

Prefer:

    TypeId
     |
    Type Arena
     |
    Type Data

over:

    Box<Type>
    Rc<Type>
    nested structures

------------------------------------------------------------------------

# Phalcom adaptation

Use:

    TypeId

    TypeArena

    enum TypeData {
        Primitive,
        Class,
        Function,
        Generic,
        AppliedType,
        Union
    }

------------------------------------------------------------------------

# Why this improves Phalcom

Phalcom's type system may contain:

-   generic types;
-   higher-kinded types;
-   value/type duality;
-   lazy types.

Stable IDs make recursive structures manageable.

------------------------------------------------------------------------

# 8. Recursive and Cyclic Type Analysis

## What Pyrefly does

The calculation system explicitly handles cycles.

Examples:

    A -> B -> C -> A

Instead of infinite recursion:

    Calculation
     |
    cycle detection
     |
    fixed point resolution

------------------------------------------------------------------------

# Phalcom adaptation

The semantic engine should support:

    QueryState:

    Unknown
    Computing
    Solved

Cycles should be expected.

They will appear in:

-   recursive types;
-   inheritance;
-   contracts;
-   module imports;
-   dispatch graphs.

------------------------------------------------------------------------

# 9. The Final Phalcom Architecture Inspired by Pyrefly

The target architecture:

                     Source

                       |
                       v

                  AST Layer

                       |
                       v

            Semantic Index Database

                       |
           +-----------+-----------+

           |                       |

     Declaration Graph       Type Graph

           |                       |

     Dispatch Graph          Contract Graph


                       |
                       v

              Query / Answer Engine

                       |
                       v

          Compiler / LSP / Runtime Tools

------------------------------------------------------------------------

# Summary of Architectural Decisions

  -----------------------------------------------------------------------
  Pyrefly idea            Phalcom adaptation      Benefit
  ----------------------- ----------------------- -----------------------
  Indexed identities      DeclarationId, TypeId,  Fast semantic
                          QueryId                 operations

  Binding database        Semantic index layer    Separation of concerns

  Lazy calculations       Demand-driven queries   IDE responsiveness

  Answer tables           Persistent semantic     Incremental analysis
                          cache

  Dependency graph        Fine-grained            Large project
                          invalidation            scalability

  Type heap               Type arena              Cheap recursive types

  Cycle-aware solver      Fixed-point semantic    Recursive semantics
                          engine
  -----------------------------------------------------------------------

------------------------------------------------------------------------

# Final conclusion

The most important lesson from Pyrefly is:

**Performance does not come from optimizing the type checker.
Performance comes from designing the entire semantic architecture so
that expensive work becomes reusable knowledge.**

For Phalcom, the equivalent goal should not be "make inference faster".

The goal should be:

**Build a semantic knowledge engine where every language fact has
identity, every computation is queryable, every result is reusable, and
every dependency is explicit.**

That architecture can support Phalcom's future goals: powerful typing,
reflection, contracts, IDE intelligence, and runtime semantic
integration without sacrificing performance.
