# Phalcom First Functional Type System Implementation Specification

## Status

Draft implementation blueprint for the first functional static typing
milestone.

This document defines the implementation path for introducing Phalcom's
foundational type system layer.

The target milestone is:

-   source-level annotations;
-   semantic type representation;
-   annotation resolution;
-   consistency checking;
-   compiler diagnostics;
-   LSP integration.

This milestone does **not** introduce runtime contract enforcement yet.

------------------------------------------------------------------------

# 1. Semantic Goal

Phalcom remains a dynamically executing object-oriented language.

The type system adds a semantic reasoning layer.

The architecture must preserve:

    dynamic runtime
    +
    persistent reflective types
    +
    compiler evidence checking
    +
    IDE semantic intelligence

Types are not erased metadata. They are semantic objects that can
participate in reflection and tooling.

------------------------------------------------------------------------

# 2. Existing Repository Architecture

The current workspace contains:

-   phalcom-ast
-   phalcom-common
-   phalcom-type-syntax
-   phalcom-native-meta
-   phalcom-native-macros
-   phalcom-native-surface
-   phalcom-core
-   phalcom-repl
-   phalcom-lsp
-   phalcom-modules

The root Cargo workspace already includes these crates.

The implementation must extend existing architecture instead of
introducing a separate compiler-only type checker.

------------------------------------------------------------------------

# 3. Existing Type Infrastructure

## phalcom-type-syntax

Location:

    phalcom-type-syntax/src/lib.rs

Current responsibility:

Parsing symbolic type expressions.

Existing concepts:

``` rust
TypeExpr
CallableType
ParameterTuple
TypeParameter
```

Existing type forms:

``` rust
Unknown
Never
Named
Applied
Union
Tuple
```

Existing callable support:

``` text
(T) -> ReturnType
<T>(T) -> ReturnType
```

This crate remains the syntax layer.

It must not become the semantic checker.

------------------------------------------------------------------------

# 4. New Semantic Type Layer

Create a new crate:

    phalcom-semantic

Add to root Cargo workspace:

    Cargo.toml

The crate owns:

-   canonical type identity;
-   type interning;
-   subtype relations;
-   consistency checking;
-   evidence tracking;
-   diagnostics shared by compiler and LSP.

Dependency direction:

    phalcom-ast
          |
          v
    phalcom-semantic
          ^
          |
    +-----+------+
    |            |
    core        lsp

The semantic crate must not depend on runtime execution.

------------------------------------------------------------------------

# 5. Core Type Representation

Create:

    phalcom-semantic/src/types.rs

Initial representation:

``` rust
pub struct TypeId(pub u32);

pub enum TypeNode {
    Never,
    Any,
    Unit,

    Nominal(NominalType),

    Tuple(Vec<TypeId>),

    Record(Vec<RecordField>),

    Union(Vec<TypeId>),

    Function(FunctionType),

    Applied {
        constructor: TypeId,
        arguments: Vec<TypeId>,
    },

    Parameter(TypeParameterId),
}
```

Important:

`Unknown` is not a TypeNode.

Unknown represents missing compiler knowledge.

------------------------------------------------------------------------

# 6. Knowledge and Evidence Model

Create:

    phalcom-semantic/src/evidence.rs

Implement:

``` rust
pub enum Knowledge {
    Unknown,
    Known(TypeId),
    Dynamic,
}
```

Implement:

``` rust
pub enum CheckResult {
    Proven,
    Refuted,
    Unproven,
    Dynamic,
}
```

Meaning:

## Proven

The compiler can establish correctness.

## Refuted

The compiler proves contradiction.

## Unproven

The compiler lacks enough information.

## Dynamic

The developer explicitly disabled static reasoning.

------------------------------------------------------------------------

# 7. AST Annotation Support

Modify:

    phalcom-ast/src/ast.rs

Add annotations to:

-   parameters;
-   fields;
-   local bindings;
-   methods;
-   getters.

Example:

Before:

``` rust
struct Parameter {
    name: Symbol,
}
```

After:

``` rust
struct Parameter {
    name: Symbol,
    annotation: Option<TypeExpr>,
}
```

Do not resolve types inside AST.

AST stores syntax only.

------------------------------------------------------------------------

# 8. Parser Changes

Modify:

    phalcom-ast/src/parser.rs

Add parsing support for:

``` phalcom
value: Int
```

``` phalcom
method(x: Int) -> String
```

``` phalcom
(Int, Int, name: String)
```

The parser produces TypeExpr.

The semantic resolver later produces TypeId.

------------------------------------------------------------------------

# 9. Type Resolution

Create:

    phalcom-semantic/src/resolver.rs

Responsibilities:

Input:

``` rust
TypeExpr
```

Output:

``` rust
TypeId
```

Initial supported forms:

-   named classes;
-   built-in types;
-   Unit;
-   Never;
-   tuples;
-   records;
-   callable types.

Future forms:

-   generic application;
-   higher-kinded types;
-   type operators.

------------------------------------------------------------------------

# 10. Type Relations

Create:

    phalcom-semantic/src/relation.rs

Implement:

``` rust
fn equivalent(a: TypeId, b: TypeId) -> bool

fn subtype(a: TypeId, b: TypeId) -> Relation

fn consistent(a: TypeEvidence, b: TypeId) -> CheckResult
```

Initial rules:

    T <: T

    Never <: T

    Subclass <: Parent

Do not create one generic compatibility function.

------------------------------------------------------------------------

# 11. Existing LSP Integration

The existing LSP inference system remains.

It already computes value-shape knowledge.

Do not replace it.

Add bridge:

    ValueShape -> TypeEvidence

Mapping:

    Instance(Int)
        |
        v
    Known(Int)


    Unknown
        |
        v
    Unknown evidence

The LSP then displays:

    Declared:
        String

    Inferred:
        Int

    Status:
        contradiction

------------------------------------------------------------------------

# 12. Compiler Integration

The compiler pipeline gains a semantic checking stage.

Flow:

    parse
     |
    resolve
     |
    infer
     |
    check declarations
     |
    compile

Initial checks:

## Local initialization

``` phalcom
let x: Int = 1
```

## Assignment

``` phalcom
x = "hello"
```

## Arguments

``` phalcom
foo("hello")
```

## Returns

``` phalcom
foo() -> String {
    1
}
```

## Fields

``` phalcom
_count: Int
```

------------------------------------------------------------------------

# 13. Reflection Preparation

Do not create runtime wrappers for ordinary classes.

A class remains its own type representation.

Synthetic types later create descriptors:

    List<Int>

    Int | String

    Int -> String

Initial reflection metadata stores:

-   declared TypeId;
-   source information;
-   declaration ownership.

------------------------------------------------------------------------

# 14. Future Compatibility Requirements

The first implementation must preserve:

## Generic application

    List<Int>

internally:

    Applied {
     constructor: List,
     arguments: [Int]
    }

## Function types

    (Int, String) -> Bool

## Callable types

    Callable<...>

## Kinds

    Int : Type

    List : Type -> Type

## Variance

Future syntax:

    class Producer<+T>

    class Consumer<-T>

------------------------------------------------------------------------

# 15. Implementation Order

## Phase 1

Create semantic crate.

Add:

-   TypeId;
-   TypeNode;
-   Knowledge;
-   Evidence.

Tests must pass.

------------------------------------------------------------------------

## Phase 2

Add annotation storage.

Modify AST.

Add parser tests.

------------------------------------------------------------------------

## Phase 3

Implement resolution.

Resolve:

-   Int;
-   String;
-   user classes.

------------------------------------------------------------------------

## Phase 4

Implement checker.

Validate:

-   assignments;
-   arguments;
-   returns.

------------------------------------------------------------------------

## Phase 5

Connect compiler diagnostics.

------------------------------------------------------------------------

## Phase 6

Connect LSP.

------------------------------------------------------------------------

# 16. Required Tests

Parser tests:

    x: Int
    x: List<Int>
    (Int)->String

Semantic tests:

    Int == Int

    Int != String

    Never <: String

Checker tests:

Valid:

``` phalcom
const x: Int = 1
```

Invalid:

``` phalcom
const x: String = 1
```

Unknown:

``` phalcom
const x: User = external()
```

LSP tests:

-   hover shows declared type;
-   hover shows inferred evidence;
-   diagnostics match compiler.

------------------------------------------------------------------------

# 17. Completion Criteria

This milestone is complete when:

-   annotations parse;
-   annotations resolve;
-   compiler checks consistency;
-   LSP uses the same semantic results;
-   contradictions are reported;
-   uncertain cases remain executable;
-   runtime behavior remains dynamic;
-   future generics/HKT/type reflection are not blocked.
