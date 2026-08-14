# Metatypes, `Self`, Class Objects, and Phalcom's Object Model

## Purpose

Phalcom is class-based and Smalltalk-inspired: classes are objects, metaclasses/class-side behavior exists, and reflection is first-class. A type system designed only for ordinary instances will become inconsistent as soon as it meets constructors, class-side protocol requirements, `Self`, or reflective class values.

This reference provides the conceptual model. Exact Phalcom object-model and typing specs remain authoritative.

## 1. Distinguish three levels

For class declaration `Person`, separate:

```text
1. ordinary instances:        p = Person.new(...)
2. class object/descripor:    Person
3. runtime class of Person:   metaclass/class-side object model
```

Typing questions differ:

```text
what type has p?
what type has value Person?
what type has the metaclass descriptor?
what does Self mean in instance method?
what does Self mean in class-side method?
```

Do not answer all with `Person`.

## 2. Nominal instance type

A source annotation `Person` may denote the instance contract associated with class descriptor `Person` when used in value type position, while reflection value `Person` itself is a class object.

Phalcom's proposed type-expression design also makes the `Person` class object directly satisfy the reflective `Type` protocol. This creates an intentional dual role:

```text
runtime value: class descriptor Person
reflective type expression: Person
static denotation in annotation position: instance type associated with Person
```

The language specification must define context, not implementation guesswork.

## 3. Class-object types

To type a variable holding class objects, a system may need a metatype constructor:

```text
TypeOf<Person>
Class<Person>
Person.Type
```

or an object-model-specific rule.

Semantics should express:

- value is a class object capable of class-side selectors;
- construction returns an instance related to represented class;
- reflection exposes superclass/type parameters;
- class-side protocol conformance can be checked.

Do not use plain instance type `Person` for class object `Person` unless Phalcom explicitly adopts a uniform dependent/object rule that makes that precise.

## 4. Metaclass runtime identity versus static metatype

Smalltalk-style runtime metaclass chain is an implementation/object-model fact. Static metatype relation need not mirror every runtime metaclass object 1:1.

Possible model:

```text
runtime: class object Person has runtime class Person class/metaclass
static:  Person value synthesizes ClassObject<Person> or suitable singleton/meta type
```

This abstraction can type class-side sends without exposing internal metaclass tower in every annotation.

But reflection APIs may still need exact runtime metaclass descriptors.

## 5. Instance-side `Self`

Consider:

```text
class Animal {
  clone() -> Self
}
class Cat is Animal {}
```

Desired fluent behavior may be:

```text
cat.clone() : Cat
```

not merely `Animal`.

This suggests **dynamic self type** bound to receiver's most precise static/dynamic subtype under inheritance.

Alternative lexical semantics would return `Animal` even on inherited call.

The difference must be ratified because it affects override compatibility and substitution.

## 6. `Self` as a binder

Represent conceptually:

```text
SelfType(owner=Animal, side=Instance)
```

When viewing inherited member through receiver type `Cat`, substitute/refine:

```text
Self[receiver=Cat] -> Cat
```

Subject to exact semantics.

This is like a distinguished type parameter constrained by receiver class, not a global alias.

## 7. Class-side `Self`

Class-side constructor/factory:

```text
class-side new(...) -> Self
```

Here `Self` refers to class object receiver/represented instance relationship.

Questions:

- Does inherited `Cat.new()` return instance type `Cat`?
- Does class-side method returning receiver itself have type metatype-of-Cat rather than instance `Cat`?
- Are there distinct `Self` forms for "represented instance" and "class object receiver"?

Avoid one ambiguous `Self` meaning if both are needed.

A design can distinguish:

```text
Self             dynamic instance/receiver type
Self.Type / metatype(Self)   class object type
```

or another Phalcom-native syntax.

## 8. Constructors

Externally visible constructor:

```text
Person.new(name:String) -> Person/Self
```

Internally initializer body may return `()` on normal completion under Phalcom's normative unit semantics.

Typing must track observable wrapper contract, not infer constructor return from initializer fallthrough.

Subclass inheritance of constructors interacts with `Self`: a generic factory should produce subclass instances only if runtime allocation semantics actually do so.

## 9. Class-side protocol requirements

Protocol may require:

```text
class-side parse(String) -> Self
```

Conformance target is class object/member surface, while result `Self` may refer to instance represented by candidate class.

Substitution pipeline:

```text
protocol requirement Self
candidate class C
instantiate requirement Self -> C (if semantics says represented instance)
compare candidate class-side callable
```

Do not inspect instance-side method table for class-side requirement.

## 10. `Self` in protocols

Structural protocol:

```text
Cloneable {
  clone() -> Self
}
```

Candidate `C` should satisfy with result compatible with `C` under self substitution.

This resembles F-bounded/self-type conformance:

```text
C conforms Cloneable<C>
```

but can be implemented as a distinguished substitution rule without exposing explicit generic parameter.

Recursive relation/cache must include self-substitution context.

## 11. Generic classes and `Self`

```text
class Box<T> {
  copy() -> Self
}
```

For receiver `Box<Int>`, likely applied self view is:

```text
Self -> Box<Int>
```

not raw `Box`.

For subclass:

```text
class FancyBox<T> is Box<T>
```

receiver `FancyBox<Int>` may refine `Self` further. Exact semantics depend on generic inheritance design.

This is why replacing `Self` with lexical origin during annotation normalization loses needed information.

## 12. `Self` and variance

`Self` often appears covariantly in fluent results. Consuming `Self`:

```text
merge(other: Self)
```

creates binary-method problem: inherited `Animal.merge(Animal)` specialized to `Cat.merge(Cat)` can violate substitutability if method is callable through an `Animal` view with arbitrary `Animal` argument.

Classic self types need careful method-subtyping rules.

Do not automatically substitute parameter `Self` to dynamic subtype without analyzing dispatch contract.

Possible policies:

- restrict `Self` in contravariant positions;
- use lexical/base interpretation for consumed Self;
- accept F-bounded semantics with explicit rules;
- treat binary methods specially.

This is a major design decision.

## 13. Singleton/class-object types

A precise type for exact class object can be singleton-like:

```text
{Person}
```

or `TypeOf<Person>`.

This enables:

```text
factory: TypeOf<T> -> T
```

but introduces dependent relationship between value-level descriptor and instance result.

Phalcom can avoid full dependent types by using generic metatype constructor semantics recognized by checker.

## 14. Reflection `Type` protocol versus metatype

Phalcom proposed design declares `Type` as a protocol satisfied by type-expression descriptors. This answers:

```text
can object be used reflectively as type expression?
```

It does **not** alone answer:

```text
what instances can this class object construct?
what is static type of class-side send result?
```

Do not overload reflective `Type` conformance as a full metatype calculus.

## 15. Metaclass inheritance and static subtyping

Runtime metaclass inheritance may imply class-side method lookup inheritance. Static subtype relation between class-object types should reflect safe substitutability of class-side contracts, not blindly copy runtime superclass edges.

Example: if subclass class object supports all inherited class-side operations, a metatype subtyping rule may be natural:

```text
Cat <: Animal  => TypeOf<Cat> <: TypeOf<Animal>
```

But constructors returning `Self`, mutable class-side state, and overridden class-side signatures can affect safety. Ratify with full callable rules.

## 16. Class-side mutable state

Applied generic type descriptors and runtime class objects may share origin class-side state under Phalcom's design direction.

Type application must not silently imply per-specialization class-side storage.

Static type distinctions:

```text
Box<Int>
Box<String>
```

can coexist with one runtime origin class object and shared class-side fields. Reflection should state that clearly.

## 17. First-class methods and bound methods

A `Method` descriptor is an implementation object, not an instance of its callable result type.

Binding method to receiver can produce callable contract:

```text
Method<Person, (String)->Unit>
BoundMethod<(String)->Unit>
```

Ordinary message send typing should follow receiver+selector semantic lookup; reflective method objects should not replace lookup semantics.

## 18. Selector families / reflective dispatch objects

If Phalcom exposes `Family`/`MethodFamily`-like objects, type theory should represent their callable/reflection roles distinctly:

- future send family bound to receiver;
- captured implementation set;
- bound implementation family.

Do not collapse these into one `Function` type merely because invocation syntax exists.

## 19. Implementation representation

Conceptual types:

```text
TypeData::NominalInstance(ClassId)
TypeData::ClassObject(ClassId or TypeId)
TypeData::SelfType { owner, side, role }
TypeData::MethodDescriptor(...)
TypeData::BoundCallable(...)
```

Phalcom may choose a more elegant algebra. The invariant is to preserve distinctions needed by runtime/reflective semantics.

## 20. Testing obligations

- instance `Person` versus class object `Person` typing;
- class-side send completion/resolution;
- inherited `Self` result on subclass;
- `Self` in generic applied receiver;
- `Self` in protocol conformance;
- binary method/contravariant `Self` negative case;
- constructor external return versus initializer unit;
- class-side protocol requirement does not match instance side;
- reflection `Type` conformance does not imply wrong metatype operations;
- class-side state sharing remains runtime semantic, not inferred per specialization.

## 21. Failure modes

- `Person` instance type and class object value treated identically everywhere.
- `Self` replaced by lexical class during parse.
- Class-side requirement searched on instance method table.
- `Self` in parameter specialized covariantly without binary-method analysis.
- Reified `Box<Int>` assumed to own separate class-side state.
- Reflective `Type` protocol mistaken for static metatype constructor.

## 22. Competency questions

1. What are the three object-model levels around a class declaration?
2. Why is `Self` a binder/context rather than a global alias?
3. What is the binary-method problem for `Self` in parameter position?
4. Why can reflective `Type` conformance coexist with a separate static class-object/metatype type?
5. How should `Self` behave for `Box<Int>.copy()` if it denotes dynamic receiver type?
6. Why does runtime metaclass inheritance not automatically define all static metatype subtyping rules?
