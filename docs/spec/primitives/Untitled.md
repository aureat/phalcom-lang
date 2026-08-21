I would make ADTs fundamentally class/object-based, with attributes declaring the algebraic semantics. That feels much more Phalcom-like than adding a separate `enum`/`data` declaration universe.

The core idea would be:

> Classes and objects remain the runtime reality. Attributes tell the compiler that certain classes/objects form a closed algebraic family.

So `@sealed` defines the closed sum, and `@variant` identifies its constructors/cases.

Conceptually:

```
@sealed
class Option {
}

@variant
class Some is Option {
  value
}

@variant
class None is Option {
}
```

Then:

```
Option = Some(value) | None
```

But crucially, that algebra is not a separate runtime system. `Some` and `None` are ordinary Phalcom classes/objects with ordinary methods, reflection, inheritance constraints, attributes, etc.

That is the direction I would recommend.

The first design question is whether variants should be subclasses or declarations inside the sealed class.

There are three plausible designs.

1. Pure OOP:

```
@sealed
class Result {}

@variant
class Ok is Result {
  ...
}

@variant
class Err is Result {
  ...
}
```

This is the most transparent. `Ok` really is a class. `Ok(value)` creates an `Ok`. Pattern matching is essentially exhaustive discrimination over subclasses.

Advantages:

- extremely understandable
- no hidden runtime machinery
- reflection naturally works
- variants can have methods
- variants can override base behavior
- ordinary inheritance semantics apply
- easy lowering

The weakness is namespace clutter. `Ok`, `Err`, `Some`, `None`, etc. become normal declarations unless you introduce nesting or qualification.

2. Attribute-generated variants inside the base:

```
@sealed
class Result {
  @variant Ok(value)
  @variant Err(error)
}
```

This is much more concise and resembles Rust/Swift/Kotlin ADTs.

But internally you now have to answer:

```
What exactly is Result.Ok?
Is it a class?
Is Ok(value) syntax sugar for a hidden class?
Can I add methods to it?
Can reflection see it?
Can I subclass it?
```

If the compiler secretly generates classes, Phalcom begins having two kinds of classes: those the programmer wrote and those invented by ADT syntax.

I would avoid making this the primitive mechanism.

3. Hybrid: attributes over real nested classes/objects.

This is where I think Phalcom gets something distinctive.

Conceptually:

```
@sealed
class Result {
  @variant
  class Ok {
    ...
  }

  @variant
  class Err {
    ...
  }
}
```

with implicit:

```
Result.Ok is Result
Result.Err is Result
```

or explicit inheritance if you want zero magic:

```
@sealed
class Result {
  @variant
  class Ok is Result {
    ...
  }

  @variant
  class Err is Result {
    ...
  }
}
```

Then:

```
Result.Ok(...)
Result.Err(...)
```

gives you proper namespacing while preserving real classes.

I think this is the strongest long-term model.

The semantic equation becomes:

```
@sealed class Result

variants(Result) =
    { Result.Ok, Result.Err }

Result =
    Result.Ok(...) | Result.Err(...)
```

`@sealed` closes the hierarchy for algebraic reasoning. `@variant` marks which members participate in exhaustive decomposition.

There is an important distinction here: not every subclass of a sealed class necessarily needs to be a variant.

You could decide:

```
@sealed
class Result {}

@variant
class Ok is Result {}

@variant
class Err is Result {}
```

means the compiler specifically knows:

```
Result = Ok | Err
```

rather than blindly assuming "all subclasses = cases".

That explicitness is useful.

I would make `@variant` stronger than a marker. It should establish several invariants.

A variant should be:

- nominally part of exactly one sealed family;
- non-abstract;
- closed/final by default;
- destructurable according to its declared state;
- usable as a pattern constructor;
- included in exhaustiveness analysis.

So conceptually:

```
@variant
class Some is Option {
  value
}
```

means more than:

```
Some <: Option
```

It means:

```
Some is a constructor/case of the closed sum Option.
```

This is where attributes add semantics without replacing OOP.

Unit variants deserve special treatment.

For:

```
Option<T> = Some(T) | None
```

`None` carries no payload. You don't really want:

```
None.new()
None.new()
None.new()
```

producing arbitrarily many meaningless instances.

A unit variant should naturally be a singleton object.

So I would allow two forms:

```
@variant
class Some is Option {
  value
}

@variant
object None is Option {
}
```

or whatever Phalcom ultimately uses for singleton objects.

That gives a beautiful mapping:

```
product-carrying variant -> class
unit variant             -> object
```

For example:

```
@sealed
class Option {
  @variant
  class Some is Option {
    value
  }

  @variant
  object None is Option {
  }
}
```

Then the algebra is literally represented through existing OO concepts:

```
Option
  = Some(value)
  | None
```

Where:

- `Some` is a class constructor;
- `None` is an object.

That is very Phalcom-like.

For `Result`:

```
@sealed
class Result {
  @variant
  class Ok is Result {
    value
  }

  @variant
  class Err is Result {
    error
  }
}
```

For a richer ADT:

```
Expr =
    Literal(value)
  | Add(left, right)
  | Call(receiver, selector, args)
```

maps naturally to:

```
@sealed
class Expr {
  @variant
  class Literal is Expr {
    value
  }

  @variant
  class Add is Expr {
    left
    right
  }

  @variant
  class Call is Expr {
    receiver
    selector
    args
  }
}
```

Now pattern matching becomes interesting.

I would make pattern matching operate on the variant identity and destructuring protocol, rather than merely doing `is` checks.

Conceptually:

```
match result {
  Result.Ok(value) => use(value)
  Result.Err(error) => handle(error)
}
```

The compiler knows from `@sealed` + `@variant` that those two cases cover `Result`.

A missing case:

```
match result {
  Result.Ok(value) => use(value)
}
```

can be diagnosed as non-exhaustive:

```
missing pattern: Result.Err(_)
```

This is one of the major reasons `@variant` needs compiler meaning rather than merely being a decorative attribute.

Destructuring should be defined structurally by the variant's declared product.

A variant is really a named product embedded in a sum.

Mathematically:

```
Result<T,E>
  = Ok × T
  + Err × E
```

or more simply:

```
Result<T,E>
  = Ok(T) | Err(E)
```

And:

```
Expr
  = Literal(Value)
  | Add(Expr × Expr)
  | Call(Expr × Selector × List<Expr>)
```

This is why OOP maps surprisingly cleanly onto ADTs:

```
sealed superclass = sum
variant subclass   = constructor/tag
instance fields    = product payload
```

The real difference between conventional OO inheritance and an ADT is closure.

Ordinary:

```
Animal
  ├── Cat
  ├── Dog
  └── ??? future subclass
```

cannot be exhaustively matched.

Sealed:

```
Result
  ├── Ok
  └── Err
```

can.

So `@sealed` is what turns inheritance from an open-world subtype hierarchy into an algebraic sum.

I'd also make the variant payload explicitly available to reflection.

For example:

```
Result.variants
// [Result.Ok, Result.Err]

Result.Ok.variant?
// true

Result.Ok.variantOf
// Result

Result.Ok.fields
// [...]
```

You could then have:

```
Result.variant?
// false

Result.sealed?
// true
```

Pattern matching machinery could use the same metadata.

There is another decision: should a variant be allowed to define arbitrary additional behavior?

I would say absolutely yes.

For example:

```
@variant
class Ok is Result {
  value

  map(transform) {
    Ok(transform(value))
  }
}
```

This is where Phalcom's OOP-based ADTs become richer than a plain algebraic declaration.

The base can also define shared behavior:

```
@sealed
class Result {
  success? {
    self is Result.Ok
  }
}
```

while variants can override:

```
@variant
class Ok is Result {
  success? { true }
}

@variant
class Err is Result {
  success? { false }
}
```

So users can choose between:

- pattern-oriented functional consumption;
- polymorphic OO consumption.

That duality is powerful.

For example, functional:

```
match result {
  Result.Ok(value) => ...
  Result.Err(error) => ...
}
```

versus OO:

```
result.map(...)
result.unwrapOr(...)
```

Both operate on the same values.

That is exactly where Phalcom can unify the OO and FP models instead of forcing users to choose one.

I would not make `@variant` a method attribute initially.

Something like:

```
@sealed
class Option {
  @variant
  @class
  some(value) { ... }
}
```

where `some` secretly acts as a constructor is tempting, but it's less transparent.

The method would have to magically imply a hidden runtime type:

```
Option.some(value)
    ↓
<Some variant instance>
```

Then reflection gets muddy:

```
What is the class of that result?
Where was it declared?
Where do its methods live?
```

Real classes solve all of that.

So my preference is:

```
classes/objects provide representation
attributes provide algebraic semantics
```

not:

```
attributes generate representation
```

That's the core design principle I would ratify.

A possible final surface could therefore look like:

```
@sealed
class Option {
  @variant
  class Some is Option {
    value

    @constructor
    new(value) {
      _value = value
    }
  }

  @variant
  object None is Option {
  }
}
```

Potentially with some boilerplate reduction later:

```
@variant(value)
class Some is Option {}
```

where `@variant(value)` can synthesize destructuring metadata or constructor plumbing.

That is a much safer use of attributes: reducing repetition while keeping the actual class visible.

You can even have progressively richer sugar:

Primitive semantic form:

```
@variant
class Some is Option {
  value
}
```

Convenience:

```
@variant(value)
class Some is Option {}
```

Perhaps eventually:

```
@variant(value)
class Some {}
```

inside a sealed class, with `is Option` inferred lexically.

But all of these lower to the same explicit semantic model.

I would also make sealed families prohibit arbitrary extension outside the declaring scope/module, depending on Phalcom's module visibility rules.

Something like:

```
@sealed guarantees that the compiler can enumerate every runtime variant.
```

That requirement is stronger than simply preventing subclasses "by convention".

Otherwise exhaustive matching becomes unsound.

There should also be a distinction between `@sealed` and `@final`:

```
@sealed
    hierarchy may have declared variants,
    but the hierarchy is closed.

@final
    no subclasses/variants at all.
```

So:

```
@final class Foo
```

means one leaf type.

Whereas:

```
@sealed class Result
```

means a finite sum of known leaves.

For nested inheritance:

```
@sealed Animal
  Mammal
    Dog
    Cat
```

you need precise exhaustiveness rules.

I'd recommend every non-leaf branch inside a sealed family must itself either be sealed or abstract, and the compiler ultimately expands to concrete variants.

For example:

```
Expr
  ├── Literal
  └── Binary
       ├── Add
       └── Multiply
```

could be represented as:

```
Expr = Literal | Add | Multiply
```

if `Binary` is abstract/sealed infrastructure rather than an actual inhabitable variant.

That matters for exhaustiveness.

Finally, this model meshes well with symbol unions without forcing a decision yet.

You could eventually have two kinds of finite algebraic domain:

```
symbol union:
Status = #active | #pending | #disabled

sealed ADT:
Result = Ok(value) | Err(error)
```

And possibly treat both uniformly under pattern matching and exhaustiveness.

The deeper common abstraction is:

```
closed set of alternatives
```

One uses existing atomic values; the other uses constructors/classes.

That gives Phalcom room to support lightweight enums later without compromising the richer ADT model.

My preferred design in one sentence:

> Make ADTs a semantic interpretation of ordinary Phalcom class/object hierarchies: `@sealed` closes the sum, `@variant` marks its constructors, variant classes carry product data, unit variants are singleton objects, and pattern matching/exhaustiveness are compiler capabilities built on that metadata.

That gets full algebraic data types without inventing a second object model.