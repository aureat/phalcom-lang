# Phalcom Type Algebra Registry — `type.function`

**Status:** Normative specification  
**Registry family:** `type.function`  
**Identifier range used here:** `T0160–T0177`

## Purpose

This file defines the semantic laws of function types.

Function subtyping is contravariant in parameter types and covariant in result types. Arity and parameter structure are semantically significant. The laws here describe callable type relations only; overload-set algebra, effect systems, calling conventions, closure capture, and runtime invocation mechanics are outside this file.

## Domain and notation

Unless stated otherwise:

- `A`, `B`, `C`, `D`, `R`, `R1`, and `R2` denote proper types.
- `(A) -> R` denotes a unary function type.
- `(A, B) -> R` denotes a binary function type.
- `() -> R` denotes a nullary function type.
- `<:` means semantic subtype.
- `==` means semantic type identity.

## Laws

```phalcom
@id(T0160)
@name(type.function.kind)
FunctionTypeKind<A, R> {
    kind((A) -> R) == Type
}

@id(T0161)
@name(type.function.identity)
FunctionIdentity<A, R> {
    (A) -> R == (A) -> R
}

@id(T0162)
@name(type.function.parameter.contravariant)
FunctionParameterContravariant<A, B, R> {
    if A <: B {
        (B) -> R <: (A) -> R
    }
}

@id(T0163)
@name(type.function.result.covariant)
FunctionResultCovariant<A, R1, R2> {
    if R1 <: R2 {
        (A) -> R1 <: (A) -> R2
    }
}

@id(T0164)
@name(type.function.subtype)
FunctionSubtype<A1, A2, R1, R2> {
    if A2 <: A1 and R1 <: R2 {
        (A1) -> R1 <: (A2) -> R2
    }
}

@id(T0165)
@name(type.function.parameter.identity-substitution)
FunctionParameterIdentitySubstitution<A, B, R> {
    if A == B {
        (A) -> R == (B) -> R
    }
}

@id(T0166)
@name(type.function.result.identity-substitution)
FunctionResultIdentitySubstitution<A, R1, R2> {
    if R1 == R2 {
        (A) -> R1 == (A) -> R2
    }
}

@id(T0167)
@name(type.function.arity.significant)
FunctionAritySignificant<A, B, R> {
    (A) -> R != (A, B) -> R
}

@id(T0168)
@name(type.function.nullary.distinct)
NullaryFunctionDistinctFromUnary<A, R> {
    () -> R != (A) -> R
}

@id(T0169)
@name(type.function.parameters.product-contravariant)
FunctionParameterTupleContravariant<P1, P2, R> {
    if P1 <: P2 {
        P2 -> R <: P1 -> R
    }
}

@id(T0170)
@name(type.function.never.result)
NeverResultFunctionSubtype<A, R> {
    (A) -> Never <: (A) -> R
}

@id(T0171)
@name(type.function.any.parameter)
AnyParameterFunctionSubtype<A, R> {
    (Any) -> R <: (A) -> R
}

@id(T0172)
@name(type.function.never.parameter)
NeverParameterFunctionSupertype<A, R> {
    (A) -> R <: (Never) -> R
}

@id(T0173)
@name(type.function.any.result)
AnyResultFunctionSupertype<A, R> {
    (A) -> R <: (A) -> Any
}

@id(T0174)
@name(type.function.subtype.transitive)
FunctionSubtypeTransitive<F1, F2, F3> {
    if F1 <: F2 and F2 <: F3 {
        F1 <: F3
    }
}

@id(T0175)
@name(type.function.subtype.identity)
FunctionMutualSubtypeIdentity<F1, F2> {
    if F1 <: F2 and F2 <: F1 {
        F1 == F2
    }
}

@id(T0176)
@name(type.function.union.result-covariance)
FunctionUnionResult<A, R1, R2> {
    (A) -> R1 <: (A) -> (R1 | R2)
    (A) -> R2 <: (A) -> (R1 | R2)
}

@id(T0177)
@name(type.function.intersection.parameter-acceptance)
FunctionIntersectionParameter<A, B, R> {
    (A | B) -> R <: (A) -> R
    (A | B) -> R <: (B) -> R
}
```

## Law notes

### T0160 — `FunctionTypeKind`

A well-formed function type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0018

### T0161 — `FunctionIdentity`

Function types participate in ordinary semantic identity.

**Kind:** Semantic law  
**Depends on:** T0001

### T0162 — `FunctionParameterContravariant`

Function parameters are contravariant.

If `A <: B`, a function that accepts every `B` can safely stand where a function accepting only `A` is required.

**Kind:** Semantic variance law  
**Depends on:** T0020–T0023

### T0163 — `FunctionResultCovariant`

Function results are covariant.

A function returning a narrower result may stand where a broader result is required.

**Kind:** Semantic variance law  
**Depends on:** T0020–T0023

### T0164 — `FunctionSubtype`

Function subtyping combines parameter contravariance and result covariance.

**Kind:** Semantic subtype law  
**Depends on:** T0162, T0163

### T0165 / T0166 — Identity substitution

Replacing a parameter or result with a semantically identical type preserves the function type.

**Kind:** Derived coherence laws  
**Depends on:** T0001–T0003, T0161

### T0167 — `FunctionAritySignificant`

Function arity is part of semantic identity.

A unary callable and binary callable are not the same function type even if some component types coincide.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0168 — `NullaryFunctionDistinctFromUnary`

A nullary function type is distinct from a unary function type.

**Kind:** Semantic identity law  
**Depends on:** T0167

### T0169 — `FunctionParameterTupleContravariant`

When a function's parameter domain is represented as a product/tuple type, ordinary function contravariance applies to that parameter product as a whole.

This law does not redefine tuple variance; it consumes the tuple subtyping relation defined by `type.tuple`.

**Kind:** Cross-family variance law  
**Depends on:** T0162 and `type.tuple`

### T0170 — `NeverResultFunctionSubtype`

A function that never returns normally satisfies any result-type expectation for the same parameter domain.

**Kind:** Derived semantic law  
**Depends on:** T0031, T0163

### T0171 — `AnyParameterFunctionSubtype`

A function accepting `Any` can substitute for a function required to accept any narrower parameter type.

**Kind:** Derived semantic law  
**Depends on:** T0041, T0162

### T0172 — `NeverParameterFunctionSupertype`

Because every proper type has `Never` as a subtype, a function whose parameter domain is `Never` is maximally narrow in accepted inputs and therefore sits high in the function-subtyping order for fixed result type.

**Kind:** Derived semantic law  
**Depends on:** T0031, T0162

### T0173 — `AnyResultFunctionSupertype`

For fixed parameters, a function returning `Any` is a result-side supertype of a function returning any narrower proper type.

**Kind:** Derived semantic law  
**Depends on:** T0041, T0163

### T0174 — `FunctionSubtypeTransitive`

Function-specific subtype reasoning must preserve the global transitivity law.

**Kind:** Derived coherence law  
**Depends on:** T0021, T0164

### T0175 — `FunctionMutualSubtypeIdentity`

Mutual function subtyping implies semantic function-type identity.

**Kind:** Derived coherence law  
**Depends on:** T0023, T0164

### T0176 — `FunctionUnionResult`

Each function returning one branch of a union is a subtype of the corresponding function returning the union.

**Kind:** Cross-family derived law  
**Depends on:** T0106, T0107, T0163

### T0177 — `FunctionIntersectionParameter`

A function capable of accepting the union `A | B` can substitute for a function that only needs to accept `A`, and likewise for `B`.

**Kind:** Cross-family derived law  
**Depends on:** T0106, T0107, T0162

## Required implementation invariants

Function subtyping must not accidentally apply covariance to parameters.

The following implication is generally invalid and must not be introduced:

```phalcom
if A <: B {
    // invalid in general:
    (A) -> R <: (B) -> R
}
```

Likewise, result contravariance is invalid unless another language feature explicitly changes the callable contract.

Arity, parameter ordering, labels, variadic shape, and calling convention must be included in semantic identity wherever Phalcom treats them as callable-type distinctions.

## Explicitly deferred laws

- labeled parameter compatibility
- optional/default parameter subtyping
- variadic parameter subtyping
- keyword-only parameter structure
- effect typing
- throwing/non-throwing callable relations
- async/coroutine callable relations
- overload/intersection-of-callables semantics
- closure capture and environment identity
