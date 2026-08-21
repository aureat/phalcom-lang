I would make `Selector` and `SelectorPattern` first-class semantic types, but I would not change the surface rule we just ratified: exact `#...` syntax still evaluates to `Symbol`.

That gives a clean three-layer model:

```
Symbol
    domain-neutral symbolic value

Selector
    parsed/validated semantic interpretation of an exact Symbol

SelectorPattern
    semantic matcher over Selectors
```

So:

```
#method(_, _).class
// Symbol
```

while a selector-consuming API can normalize it:

```
Selector.from(#method(_, _))
// <Selector #method(_,_)>
```

This is better than making every selector-looking symbol automatically instantiate `Selector`, because it preserves the domain-neutrality of symbols.

The two class definitions I would aim for are conceptually like this.

```
@sealed
class Selector {
  @private symbol
  @private name
  @private kind
  @private positional
  @private labels
  @private assignment

  @class
  from(symbol) {
    // Validate that `symbol` describes one exact selector.
    // Reject selector patterns and non-selector symbols.
  }

  symbol {
    symbol
  }

  name {
    name
  }

  kind {
    kind
  }

  positional {
    positional
  }

  labels {
    labels
  }

  assignment? {
    assignment
  }

  getter? {
    kind == #getter
  }

  nullary? {
    kind == #nullary
  }

  method? {
    kind == #method
  }

  index? {
    kind == #index
  }

  operator? {
    kind == #operator
  }

  arity {
    positional + labels.length
  }

  matches(pattern) {
    pattern.matches(self)
  }

  toSymbol {
    symbol
  }

  toString {
    symbol.toString
  }

  hash {
    symbol.hash
  }

  ==(other) {
    other is Selector and symbol == other.symbol
  }
}
```

And:

```
@sealed
class SelectorPattern {
  @private namePattern
  @private positionalPattern
  @private labelPattern
  @private assignmentPattern
  @private source

  @class
  from(symbol) {
    // Parse/validate selector-pattern syntax.
  }

  matches(selector) {
    // Exact structural matching.
  }

  namePattern {
    namePattern
  }

  positionalPattern {
    positionalPattern
  }

  labelPattern {
    labelPattern
  }

  assignmentPattern {
    assignmentPattern
  }

  exact? {
    false
  }

  toString {
    source.toString
  }

  ==(other) {
    // structural equality
  }

  hash {
    // structural hash
  }
}
```

I would not treat those exact fields as final API names yet, but that is the right conceptual split.

The most important design decision is what `Selector` actually represents.

A selector is not merely its base name.

These are all distinct selectors:

```
#method
#method()
#method(_)
#method(_, _)
#method(_, duration)

#value=(put)

#+
#+(_)

#[index]
#[index]=(put)
```

So `Selector` should represent the complete dispatch identity.

Conceptually:

```
Selector {
    head
    argument shape
    assignment shape
}
```

A stronger internal representation might actually be:

```
Selector
├── head
│   ├── Named("method")
│   ├── Operator("+")
│   └── Index
│
├── form
│   ├── Getter
│   ├── Call
│   └── Assignment
│
└── parameters
    ├── positional count
    └── labels
```

For example:

```
Selector.from(#method)
```

could internally be:

```
head       = Named("method")
form       = Getter
positional = 0
labels     = []
```

Whereas:

```
Selector.from(#method())
```

is:

```
head       = Named("method")
form       = Call
positional = 0
labels     = []
```

That distinction is essential. If you model both as merely `name = "method", arity = 0`, you lose the getter/nullary distinction.

Likewise:

```
Selector.from(#+)
```

and:

```
Selector.from(#+(_))
```

are distinct even though their base name is identical.

I would therefore avoid exposing selector identity through just `.name` and `.arity`. Those are useful projections, not the selector itself.

A richer interface could look like this:

```
selector = Selector.from(#fetch(_, timeout))

selector.symbol
// #fetch(_,timeout)

selector.name
// #fetch

selector.kind
// #method

selector.form
// #call

selector.positionalArity
// 1

selector.labels
// [#timeout]

selector.arity
// 2

selector.getter?
// false

selector.nullary?
// false

selector.assignment?
// false

selector.index?
// false
```

There is a subtle question around `.name`.

I would return a `Symbol`, not a string:

```
Selector.from(#fetch(_, timeout)).name
// #fetch
```

not:

```
"fetch"
```

because the selector/reflection system should stay symbolic.

For operator selectors:

```
Selector.from(#+(_)).name
// #+
```

For index selectors, you need a decision. You could use a distinguished symbolic head:

```
Selector.from(#[_, _]).name
// #[]
```

but inventing `#[]` if it isn't otherwise valid syntax may be awkward.

An alternative is:

```
selector.head
// #index
```

while `.symbol` preserves the exact syntax.

I slightly prefer a structured `.head` rather than forcing every selector category through `.name`.

Something like:

```
selector.head.kind
// #named
// #operator
// #index
```

may be too object-heavy for Phalcom, though. A simpler API could expose predicates plus `.name`.

The constructor should also be carefully designed.

I would not make this valid:

```
Selector.new("method", 2)
```

because it allows construction of selector states that may not correspond cleanly to Phalcom syntax.

Prefer parsing from a symbolic value:

```
Selector.from(#method(_, _))
```

Possibly also:

```
#method(_, _).asSelector
```

if that conversion is common enough.

But I would be cautious about putting selector-specific behavior directly on `Symbol`, because that pushes selector semantics back into the general symbol abstraction.

The cleanest API is probably:

```
Selector.from(symbol)
```

returning an `Option` or `Result`.

Given Phalcom's direction around explicit failure handling, something like:

```
Selector.parse(#method(_, _))
// Some(<Selector ...>)

Selector.parse(#hello-world-that-is-not-a-selector)
// None
```

could work if "not a selector" is expected rather than exceptional.

Or:

```
Selector.from(symbol)
// Result<Selector, InvalidSelector>
```

if invalid input is considered programmer error.

I prefer two operations:

```
Selector.parse(symbol)
// Option<Selector>
```

and:

```
Selector.from(symbol)
// Selector or raises/Result, depending on Phalcom convention
```

but that's secondary.

`SelectorPattern` is more interesting.

I would explicitly model it as a structural pattern over selector components rather than as a regex/string matcher.

For example:

```
#method(...)
```

is not "text beginning with `method(`".

It is a selector-language pattern.

So:

```
pattern = #method(...)

pattern.matches(Selector.from(#method))
// perhaps true or false depending on exact pattern semantics

pattern.matches(Selector.from(#method()))
pattern.matches(Selector.from(#method(_)))
pattern.matches(Selector.from(#method(_, _)))
```

The matching semantics should be defined structurally.

Likewise:

```
#method(_, ...)
```

should mean something like:

```
name = exactly "method"
first positional slot = required
remaining admissible slots = wildcard
```

not:

```
string starts with "method(_,"
```

That distinction becomes crucial once labeled parameters and index selectors enter the system.

I would also make `SelectorPattern` composable eventually, but not initially.

Useful future operations:

```
pattern.matches(selector)

pattern & otherPattern
pattern | otherPattern
pattern.not
```

But I would not put those in v1 unless there is a concrete consumer. It is easy to overbuild a pattern algebra before the selector-pattern grammar has stabilized.

The minimal `SelectorPattern` interface should probably be:

```
pattern.matches(selector)
pattern.toString
pattern.hash
pattern == other
```

plus structural introspection:

```
pattern.name
pattern.kind
pattern.positional
pattern.labels
```

where wildcard positions remain explicitly represented.

For example:

```
pattern = #method(_, ...)

pattern.positional
// [Exact(_), Rest]
```

Internally you may have something like:

```
PatternSlot
├── Exact
├── Wildcard
└── Rest
```

but I would not necessarily expose `PatternSlot` as a public class immediately.

There is also a useful relationship between the two types:

```
selector.matches(pattern)
```

versus:

```
pattern.matches(selector)
```

I strongly prefer the second as the primitive operation:

```
pattern.matches(selector)
```

because the pattern owns the matching semantics.

You can still add convenience:

```
selector.matches(pattern)
```

later, but semantically `SelectorPattern` is the matcher.

For reflection, these classes become very useful.

Instead of APIs repeatedly reparsing symbols:

```
System.class >> #print(_)
```

the lookup implementation can normalize once:

```
Symbol
  ↓
Selector
  ↓
method dictionary lookup
```

This gives a stable internal boundary.

Likewise:

```
System.class.methods matching: #print(...)
```

can become:

```
Symbol syntax
  ↓ parser
SelectorPattern
  ↓
method dictionary structural matching
```

A reflection API could expose:

```
method.selector
// Selector
```

or possibly:

```
method.selector
// #print(_)
```

There are arguments for both.

I would return `Selector`, not `Symbol`, from reflection metadata if `Selector` exists as a real semantic class.

Why?

Because this:

```
method.selector.arity
method.selector.labels
method.selector.getter?
```

is far more useful than forcing users to reinterpret a Symbol manually.

Meanwhile:

```
method.selector.symbol
```

gets the domain-neutral symbolic representation when needed.

So I would have this split:

```
method.selector
// <Selector #print(_)>

method.selector.symbol
// #print(_)
```

That is a strong reason for having a real `Selector` class even though `#print(_)` itself evaluates to `Symbol`.

Another important use is method dictionaries.

Internally:

```
Map<Selector, Method>
```

is semantically clearer than:

```
Map<Symbol, Method>
```

even if both ultimately hash through the same interned symbolic representation.

It prevents arbitrary symbols such as:

```
#coffee
#???
```

from accidentally being treated as dispatch keys without validation.

The class can effectively act as a validated, normalized dispatch key.

That is probably the strongest architectural justification for `Selector`.

I would therefore define their roles this way:

|Type|Meaning|Created by|
|---|---|---|
|`Symbol`|Domain-neutral symbolic value|`#foo`, `#+`, `#method(_)`|
|`Selector`|Validated exact dispatch/message identity|selector consumers / `Selector.from`|
|`SelectorPattern`|Structural matcher over selectors|`#method(...)`|

There is one asymmetry here: `SelectorPattern` is created directly by syntax while `Selector` is not.

I think that asymmetry is justified.

Why?

Because:

```
#method(_)
```

has a perfectly useful domain-neutral interpretation as a Symbol.

But:

```
#method(...)
```

contains actual pattern metasyntax. It cannot naturally denote one exact symbolic selector.

So pattern syntax has crossed into a semantic language construct.

That is a principled boundary, not an arbitrary one.

If you wanted absolute symmetry, you could instead make:

```
#method(...)
```

produce a Symbol and require:

```
SelectorPattern.from(#method(...))
```

But I would not recommend it. You would be encoding a pattern AST inside a general-purpose Symbol and reparsing it later.

One last point: both classes should probably be sealed and immutable.

Conceptually:

```
@sealed
class Selector { ... }

@sealed
class SelectorPattern { ... }
```

I would not allow arbitrary subclassing because selector equality, hashing, dispatch identity, and pattern matching are VM/compiler-sensitive semantics. User extension belongs in APIs consuming selectors, not in redefining what a selector is.

My recommended initial public surface is therefore approximately:

```
@sealed
class Selector {
  @class parse(symbol)
  @class from(symbol)

  symbol
  name
  kind
  form

  arity
  positionalArity
  labels

  getter?
  nullary?
  assignment?
  index?
  operator?

  toString
  hash
  ==(_)
}
```

and:

```
@sealed
class SelectorPattern {
  matches(selector)

  name
  kind

  toString
  hash
  ==(_)
}
```

with richer structural pattern introspection added only when reflection or tooling actually needs it.

The core architectural payoff is substantial: `Symbol` remains general-purpose, `Selector` becomes the validated dispatch key, and `SelectorPattern` becomes the structural matching language. That is a clean separation of syntax, identity, and matching semantics.