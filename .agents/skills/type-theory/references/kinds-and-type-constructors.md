# Kinds and Type Constructors

## Purpose

Kinds classify type-level entities. They prevent generic application from devolving into scattered arity checks and clarify the difference between a proper type, a generic declaration/type constructor, a higher-kinded parameter, and a runtime object describing any of those.

Use kinds where they buy precision. Do not add higher-kinded type features merely because the formalism can express them.

## 1. Value types versus type constructors

A proper type has kind:

```text
Type
```

A unary constructor:

```text
List : Type -> Type
```

A binary constructor:

```text
Map : Type -> Type -> Type
```

Application:

```text
List<Int> : Type
Map<String, Int> : Type
```

Bare `List` and applied `List<Int>` are different categories at the type level even if both are represented by first-class runtime descriptors.

## 2. Kinding judgment

Write:

```text
Δ ⊢ T : κ
```

meaning type-level expression `T` has kind `κ` under type-level environment `Δ`.

### Proper type

```text
Int recognized as proper type
─────────────────────────────
Δ ⊢ Int : Type
```

### Constructor application

```text
Δ ⊢ F : κ1 -> κ2
Δ ⊢ A : κ1
────────────────
Δ ⊢ F<A> : κ2
```

For first-order Phalcom generics where every parameter expects `Type`, kinding may reduce to constructor arity plus proper-type validation. Still, the conceptual model prevents category mistakes.

## 3. Curried versus n-ary constructor kinds

Mathematics often writes:

```text
Map : Type -> Type -> Type
```

as curried. A language can instead represent n-ary constructor signatures:

```text
Map : (Type, Type) => Type
```

if partial type application is unsupported.

Phalcom's current generic design direction says exact application arity and no partial application in the first version. Therefore implementation need not expose curried partial constructors just because notation uses arrows.

## 4. Bare generic origins

Current proposed Phalcom type-expression design treats a bare generic declaration such as `Box` as the closed declaration/type-constructor object itself, not implicit `Box<T>`.

This yields two views:

```text
reflection/object view:
  Box is a Class descriptor satisfying Type behavior

constructor capability:
  Box declares arity 1 and can be applied by the reserved type-application mechanism
```

A kind system can model its application capability without forcing `Box` to be a proper inhabited instance type in every static context. Exact source rules must be ratified by the typing series.

## 5. Arity as a first-order kind check

Errors:

```text
Map<Int>          # too few args when partial application unsupported
Map<Int,String,X> # too many
Int<String>       # Int not applicable
```

These are type-formation/kinding errors, not subtype errors.

A centralized constructor descriptor can expose:

```text
arity
parameter TypeParamIds
parameter restrictions
result kind
```

so parser/checker/reflection do not each invent arity logic.

## 6. Higher-kinded parameters

A higher-kinded parameter can range over constructors:

```text
F : Type -> Type
```

Then generic abstraction can express:

```text
Functor<F>
map : (A -> B, F<A>) -> F<B>
```

Costs:

- syntax for constructor-kinded parameters;
- kind inference/checking;
- higher-order unification or more annotations;
- variance across constructor parameters;
- reflection of kinds;
- type application of unknown constructors;
- protocol/associated-type complexity.

Do not add HKT support without concrete Phalcom library abstractions that justify this complexity.

## 7. Kind polymorphism

Advanced systems allow variables over kinds:

```text
κ
```

or constructors generic over arity/kind. This is far beyond first-order generics and should be a separate explicit feature if ever needed.

## 8. Associated types and kinding

A protocol/type member can itself be a proper type:

```text
Iterator.Element : Type
```

or a constructor:

```text
Container.Rebind : Type -> Type
```

Kinds become important once type members can return constructors. Without kind metadata, member selection can produce hard-to-diagnose application errors.

Do not introduce associated types accidentally by treating arbitrary type-valued reflective fields as static type members.

## 9. Type constructors versus runtime constructors

Phalcom has runtime/class constructors such as `new`. A **type constructor** is different:

```text
Box<T>        # constructs a type expression
Box.new(value) # constructs a runtime instance
```

The two can coexist on the same class descriptor object but must have different semantics, dispatch, and security/override rules.

A reserved type-application intrinsic should not become an ordinary user-overridable method if the type system depends on canonical, pure formation.

## 10. Reflection objects versus kinds

A runtime object representing `List` may itself have runtime class `Class` and satisfy protocol `Type`. That runtime classification is not its type-theoretic kind.

Axes:

```text
runtime object class: Class
static descriptor role: recognized Type expression / type constructor
kind: arity-1 constructor capability
```

Do not infer kind from runtime metaclass hierarchy alone.

## 11. Constructor restrictions

Kinding/forming `F<A>` may require more than kind equality:

```text
A : Type
A <: Bound
A in finite exact constraints
```

Separate:

1. kind/arity admissibility;
2. generic restriction validity;
3. later subtyping/conformance of the applied result.

This gives better diagnostics:

```text
expected 2 type arguments, got 1
```

versus:

```text
type argument Int violates bound Hashable
```

## 12. Type lambdas

A type-level lambda:

```text
λX. Map<String,X>
```

has kind:

```text
Type -> Type
```

Type lambdas enable partial application and HKT programming. They also require binding/substitution at the type-operator level.

Do not introduce them merely to model internal substitution; internal constructors can be represented by origin+arguments without public type lambda syntax.

## 13. Variance and kinds

Kind tells what arguments an entity accepts, not how subtyping flows through it.

Two unary constructors can both have:

```text
Type -> Type
```

while one is covariant, one contravariant, one invariant.

Store variance in generic signature/type constructor metadata separately from kind/arity.

## 14. Implementation representation

For first-order generics, a lightweight internal representation may be enough:

```text
TypeConstructorInfo {
  origin: TypeId/descriptor ID,
  params: [TypeParamId],
  arity: u16,
  result_kind: Type,
}
```

If HKT is added later:

```text
Kind = Type | Arrow(KindId, KindId) | Tupled([...])
```

Do not pay recursive-kind complexity before the language needs it.

## 15. Formation diagnostics and recovery

Kinding errors should produce a recovery type/descriptor only internally. Do not let malformed `Map<Int>` normalize into a valid raw `Map` or `Dynamic` silently.

Live editor recovery may keep a partially applied syntax node and report missing arguments while preserving unrelated semantic facts. That recovery is not normative partial type application.

## 16. Testing obligations

- proper type application;
- too few/many arguments;
- applying non-constructor;
- nested constructors;
- bound/finite-constraint error after arity success;
- reflection reports declared arity/parameters consistently;
- malformed source partial application does not become valid semantics;
- if HKT ever exists, wrong-kind argument tests and type-lambda substitution.

## 17. Failure modes

- Treating every runtime `Class` object as kind `Type` with no constructor distinction.
- Scattering arity checks across parser, checker, VM, and LSP.
- Allowing partial application accidentally through error recovery.
- Conflating runtime `new` with type application.
- Adding HKT machinery before a Phalcom use case exists.

## 18. Competency questions

1. What is the kind difference between `List` and `List<Int>`?
2. Why can first-version no-partial-application semantics use n-ary constructor signatures rather than curried public constructors?
3. Why is runtime class `Class` not a type-theoretic kind?
4. Which errors belong to kind/formation checking versus generic bound checking?
5. What additional complexity appears with higher-kinded parameters?
