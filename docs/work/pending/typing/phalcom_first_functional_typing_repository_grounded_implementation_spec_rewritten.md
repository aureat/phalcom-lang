# Phalcom First Functional Typing System

## Repository-Grounded Implementation Specification (Rewritten)

**Purpose:** Engineering handoff specification for implementing the
first functional static typing layer in `aureat/phalcom-lang`.

**Baseline:** This document is derived from the uploaded
repository-grounded specification and the repository investigation
context. fileciteturn12file0

------------------------------------------------------------------------

# 1. Implementation Goal

Implement a VM-independent semantic typing layer that:

-   understands source annotations;
-   resolves type references through Phalcom's module/declaration
    system;
-   checks assignments, arguments, fields, and returns;
-   provides shared diagnostics to compiler and LSP;
-   preserves dynamic runtime semantics.

The type system must be an analysis of the existing Phalcom language,
not a second erased language.

The architecture must preserve:

    Parser
       |
       v
    AST
       |
       v
    Module resolution
       |
       v
    Semantic typing
       |
       v
    Bytecode compilation
       |
       v
    Runtime

------------------------------------------------------------------------

# 2. Current Repository Anchors

The implementation must integrate with these existing areas:

    phalcom-ast
    phalcom-common
    phalcom-modules
    phalcom-native-meta
    phalcom-native-surface
    phalcom-core
    phalcom-lsp

The semantic layer should become:

    phalcom-semantic

with dependencies flowing from semantic analysis toward
AST/module/native metadata, never toward the VM.

------------------------------------------------------------------------

# 3. Files To Add

Create:

    phalcom-semantic/
        Cargo.toml
        src/
            lib.rs
            identity.rs
            diagnostic.rs
            snapshot.rs
            dispatch.rs

            types/
                mod.rs
                id.rs
                kind.rs
                store.rs
                relation.rs
                evidence.rs
                annotation.rs
                native.rs

            checker/
                mod.rs
                context.rs
                expression.rs
                statement.rs
                call.rs
                declaration.rs

------------------------------------------------------------------------

# 4. Files To Modify

## AST

Modify:

    phalcom-ast/src/ast.rs
    phalcom-ast/src/parser.rs

Add:

``` rust
pub struct TypeAnnotation {
    pub expr: TypeAnnotationExpr,
    pub range: SourceRange,
}
```

Initially support:

``` rust
pub enum TypeAnnotationExpr {
    Reference(String),
}
```

Extend:

    LetBinding
    ParameterDef
    FieldDef
    MethodDef
    GetterDef
    SetterDef
    IndexMethodDef

with:

``` rust
annotation: Option<TypeAnnotation>
```

and:

``` rust
return_annotation: Option<TypeAnnotation>
```

where applicable.

------------------------------------------------------------------------

# 5. Canonical Identity

Do not create duplicate identities.

Use:

    phalcom_modules::ModuleId
    phalcom_modules::DeclarationId

for semantic identity.

Do not use:

-   URI strings;
-   VM class objects;
-   runtime heap identities.

------------------------------------------------------------------------

# 6. Type Store

Create:

    phalcom-semantic/src/types/store.rs

Implement:

``` rust
pub struct TypeStore
```

with interned:

``` rust
TypeId
KindId
InferVarId
```

Required canonical types:

    Never
    Unit
    Nominal(T)
    Union(...)

Do not store:

    Unknown
    Dynamic

as ordinary types.

They belong to type knowledge.

------------------------------------------------------------------------

# 7. Type Knowledge Model

Create:

    types/evidence.rs

Implement:

``` rust
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

Evidence:

``` rust
pub struct TypeEvidence {
    pub ty: TypeId,
    pub provenance: EvidenceSet,
}
```

The checker must distinguish:

-   declared information;
-   inferred information;
-   advisory information.

------------------------------------------------------------------------

# 8. Annotation Resolution

Create:

    types/annotation.rs

Responsibilities:

-   resolve `Int`, `String`, `User`;
-   resolve imported declarations;
-   resolve module-qualified references;
-   resolve builtin universe types;
-   resolve `Never`;
-   resolve `Dynamic`.

Failure:

    type.annotation.unresolved_name

must be a semantic diagnostic.

------------------------------------------------------------------------

# 9. Relation Engine

Create:

    types/relation.rs

Implement:

``` rust
is_subtype()
is_assignable()
equivalent()
```

Required laws:

    T <: T

    Never <: T

Nominal inheritance:

    Child <: Parent

Union:

    A <: B | C

when:

    A <: B OR A <: C

Dynamic and Unknown must return uncertainty, not false errors.

------------------------------------------------------------------------

# 10. Expression Checker

Replace the current expression classification approach with semantic
checking.

Modify:

    phalcom-semantic/src/checker/expression.rs

Expressions must lower into Phalcom semantics.

Example:

    a + b

must become conceptually:

    receiver a
    selector +(_)
    argument b
    dispatch
    return type

Do not hardcode arithmetic tables.

------------------------------------------------------------------------

# 11. Callable Checking

Modify:

    checker/call.rs

Replace positional-only checking.

Introduce:

``` rust
CallableSignature
CallableParameter
```

matching:

-   selector labels;
-   positional parameters;
-   rest parameters;
-   return annotations.

------------------------------------------------------------------------

# 12. Field Checking

Implement:

    type.field.initializer_mismatch
    type.field.assignment_mismatch

Checks:

    field annotation
            |
            v
    initializer/write
            |
            v
    assignability

Observed runtime writes remain advisory unless explicitly declared.

------------------------------------------------------------------------

# 13. Flow Integration

Do not create a second control-flow engine.

Reuse/extract existing LSP semantic flow:

    phalcom-lsp/src/semantic/flow.rs

The shared semantic layer should own:

-   reachability;
-   branch joins;
-   return analysis;
-   mutation invalidation.

------------------------------------------------------------------------

# 14. Compiler Integration

Modify:

    phalcom-core

pipeline:

    parse once

            |
            v

    module resolution

            |
            v

    semantic checking

            |
            v

    bytecode compilation

Do not parse source again during lowering.

Do not insert runtime type checks.

------------------------------------------------------------------------

# 15. LSP Integration

Modify:

    phalcom-lsp/src/semantic/*

The LSP should consume shared semantic results.

Hover:

    Declared: Int
    Inferred: Int

Diagnostics:

    same semantic diagnostic
    same code
    same source range

No separate editor-only type checker.

------------------------------------------------------------------------

# 16. Native Metadata

Use:

    phalcom-native-meta

as canonical native type metadata.

Normalize:

    TypeExprSpec
            |
            v
    TypeStore
            |
            v
    TypeKnowledge

Do not make `NativeReturnShape` the authoritative type system.

------------------------------------------------------------------------

# 17. Tests Required

Add:

    phalcom-semantic/tests/

with:

    type_store.rs
    type_resolution.rs
    relations.rs
    bindings.rs
    returns.rs
    fields.rs
    calls.rs
    native.rs
    flow.rs
    lsp_consistency.rs

Required cases:

``` phalcom
const x: Int = 1
```

valid.

``` phalcom
const x: String = 1
```

diagnostic.

``` phalcom
fn() -> Int {
    "wrong"
}
```

diagnostic.

Unknown and Dynamic must not produce false positives.

------------------------------------------------------------------------

# 18. Completion Criteria

The milestone is complete when:

-   annotations parse;
-   types resolve;
-   semantic IDs are canonical;
-   TypeStore exists;
-   assignments are checked;
-   arguments are checked;
-   returns are checked;
-   compiler and LSP share diagnostics;
-   runtime semantics remain unchanged;
-   incremental analysis remains performant.

------------------------------------------------------------------------

# Final Architectural Rule

Phalcom remains:

    one dynamic object/message language
    +
    one semantic analysis layer

not:

    dynamic runtime language
    +
    separate static language

Static typing describes and verifies Phalcom programs. It does not
replace their runtime model.
