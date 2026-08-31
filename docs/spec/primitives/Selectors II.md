I’d make both classes small, immutable, structural, and reflection-friendly. The key is to avoid turning them into bags of parser internals.

`Selector` should represent one exact message identity. `SelectorPattern` should represent a structural matcher over selectors.

The most important interface decision is this:

```
#method(_).class
// Symbol
```

Exact selector syntax still produces a `Symbol`.

Then selector-aware code converts it:

```
Selector.from(#method(_))
```

Whereas:

```
#method(...).class
// SelectorPattern
```

can directly produce a pattern object because `...` is explicitly pattern syntax.

## `Selector`

I would aim for something close to:

```
@sealed
class Selector {
  @class parse(symbol)
  @class from(symbol)

  symbol
  name
  kind
  form

  parameters
  positional
  labels

  arity
  positionalArity
  labeledArity

  getter?
  nullary?
  method?
  setter?
  index?
  operator?

  matches(pattern)

  toString
  hash
  ==(_)
}
```

The minimum useful surface is smaller:

```
@sealed
class Selector {
  @class from(symbol)

  symbol
  name
  form
  parameters
  arity

  getter?
  nullary?
  setter?
  index?
  operator?

  toString
  hash
  ==(_)
}
```

### `Selector.from(symbol)`

Primary conversion:

```
const selector = Selector.from(#method(_, timeout))
```

It validates that the symbol represents one exact selector.

Examples:

```
Selector.from(#foo)
Selector.from(#foo())
Selector.from(#foo(_))
Selector.from(#+(_))
Selector.from(#[_, default]=(put))
```

It should reject arbitrary symbols that cannot be interpreted as selectors.

I would also expose a non-throwing/non-failing version:

```
Selector.parse(symbol)
```

returning something like:

```
Option<Selector>
```

So:

```
Selector.parse(#method(_))
// Some(<Selector #method(_)>)

Selector.parse(#someNonSelectorSymbol)
// None
```

This is useful for reflective and generic symbolic APIs.

### `.symbol`

Returns the canonical exact symbolic representation:

```
selector.symbol
// #method(_,timeout)
```

This should always be lossless:

```
Selector.from(selector.symbol) == selector
```

This is probably the most important projection.

### `.name`

Returns the selector's symbolic base name:

```
Selector.from(#method(_, timeout)).name
// #method
```

For:

```
Selector.from(#+(_)).name
// #+
```

I would return `Symbol`, never `String`.

Index selectors are the awkward case. I would either return a canonical symbolic head such as:

```
#[]
```

if Phalcom formally supports it, or make `.name` optional for non-named selectors and expose `.kind`.

Do not invent a fake string `"[]"`.

### `.form`

This should encode the most fundamental syntactic distinction:

```
Selector.from(#foo).form
// #getter

Selector.from(#foo()).form
// #call

Selector.from(#foo(_)).form
// #call

Selector.from(#foo=(put)).form
// #setter

Selector.from(#[x]).form
// #index

Selector.from(#[x]=(put)).form
// perhaps #indexSetter
```

You could collapse index/index-setter into `.kind` + `.setter?`, but some explicit structural representation is important.

Critically:

```
#foo
```

and:

```
#foo()
```

cannot both become:

```
name = #foo
arity = 0
```

because that loses the getter/nullary distinction.

### `.parameters`

This should expose the selector's signature structurally.

For:

```
#method(_, timeout, retry)
```

perhaps:

```
selector.parameters
// [#_, #timeout, #retry]
```

But I would probably not represent positional slots as the literal symbol `#_`.

A better API may be:

```
selector.positional
// 1

selector.labels
// [#timeout, #retry]
```

That is simple and semantically useful.

If tooling eventually needs richer information, add:

```
selector.parameters
```

returning structured parameter descriptors.

For v1, I would keep:

```
positionalArity
labels
arity
```

### `.arity`

Total explicit parameters:

```
Selector.from(#foo).arity
// 0

Selector.from(#foo()).arity
// 0

Selector.from(#foo(_)).arity
// 1

Selector.from(#foo(_, timeout)).arity
// 2
```

Again, `.arity` cannot determine getter versus nullary. That's why `.form` exists.

### Predicates

These are useful because users should not constantly compare internal tags:

```
selector.getter?
selector.nullary?
selector.setter?
selector.index?
selector.operator?
```

I would be cautious with:

```
method?
```

because nearly every selector eventually identifies a method/message. It may not distinguish anything meaningful.

Better distinctions are structural.

For example:

```
selector.call?
selector.getter?
selector.assignment?
selector.index?
selector.operator?
```

### Equality and hashing

Selectors should have structural value identity:

```
Selector.from(#foo(_)) == Selector.from(#foo(_))
// true

Selector.from(#foo) == Selector.from(#foo())
// false
```

Hashing must follow the same canonical structure.

That makes:

```
Map<Selector, Method>
```

a natural runtime representation for method dictionaries.

---

# `SelectorPattern`

This should be even smaller.

I would start with:

```
@sealed
class SelectorPattern {
  matches(selector)

  name
  form
  parameters

  exact?
  variadic?

  toString
  hash
  ==(_)
}
```

Potentially:

```
@sealed
class SelectorPattern {
  matches(selector)

  symbol
  name
  form

  minimumArity
  labels

  variadic?

  toString
  hash
  ==(_)
}
```

The most important operation is unquestionably:

```
pattern.matches(selector)
```

Everything else is reflection.

## `.matches(selector)`

Primitive matching operation:

```
const pattern = #method(...)

pattern.matches(Selector.from(#method))
// depending on exact pattern semantics

pattern.matches(Selector.from(#method()))
pattern.matches(Selector.from(#method(_)))
pattern.matches(Selector.from(#method(_, _)))

pattern.matches(Selector.from(#other(_)))
// false
```

It should probably accept both `Selector` and exact selector `Symbol` for convenience:

```
pattern.matches(#method(_))
```

Internally:

```
Symbol -> Selector.from -> match
```

That ergonomics is worth having because literal selectors are Symbols in normal source code.

### Don't make Selector responsible for matching

You could support:

```
selector.matches(pattern)
```

but I would make it convenience only.

The semantic owner is the pattern:

```
pattern.matches(selector)
```

because the pattern defines the matching rules.

### `.name`

For:

```
#method(...)
```

return:

```
#method(...).name
// #method
```

Again, Symbol rather than String.

### `.form`

Patterns should preserve structural category too.

For example:

```
#method(...).form
// #call

#[...].form
// #index
```

if index patterns are eventually supported.

### `.variadic?`

Very useful:

```
#method(...).variadic?
// true

#method(_, ...).variadic?
// true
```

This is better than making users inspect raw pattern nodes.

### `.minimumArity`

For:

```
#method(...)
```

perhaps:

```
minimumArity
// 0
```

For:

```
#method(_, ...)
```

perhaps:

```
minimumArity
// 1
```

This is useful for reflection and LSP tooling.

But only expose it if `...` actually means "zero or more remaining slots". If its semantics are richer, don't force everything through arity arithmetic.

---

# I would not expose parser AST internals

Avoid APIs like:

```
pattern.nodes
pattern.tokens
pattern.wildcards
pattern.ast
```

at least on the main semantic classes.

Those are compiler implementation details.

The public API should expose semantic structure:

```
name
form
arity
labels
variadic?
matches(_)
```

If the parser/LSP needs deeper details, it can use compiler AST types separately.

---

# I'd add one common protocol

It may be worth having both classes conform to some common selector-description protocol, conceptually:

```
SelectorSpec
```

with:

```
name
form
toString
```

Then:

```
SelectorSpec
├── Selector
└── SelectorPattern
```

But I would not necessarily introduce the class publicly yet.

The distinction is strong enough that forcing too much commonality may hurt.

For example:

```
Selector.arity
```

has one exact value.

```
SelectorPattern.arity
```

often does not.

So do not fake a shared `.arity` returning `None` everywhere just to make the hierarchy symmetrical.

---

# Recommended concrete interface

I would ratify roughly this:

```
@sealed
class Selector {
  @class parse(symbol)
  @class from(symbol)

  symbol
  name

  form

  positionalArity
  labels
  arity

  getter?
  nullary?
  call?
  assignment?
  index?
  operator?

  matches(pattern)

  toString
  hash
  ==(_)
}
```

And:

```
@sealed
class SelectorPattern {
  name
  form

  minimumArity
  labels

  variadic?

  matches(selector)

  toString
  hash
  ==(_)
}
```

I would probably omit `Selector.matches(pattern)` from the first implementation too, leaving only:

```
pattern.matches(selector)
```

unless ergonomics shows a need.

---

# Example

```
const symbol = #send(_, to, timeout)

const selector = Selector.from(symbol)

selector.symbol
// #send(_,to,timeout)

selector.name
// #send

selector.form
// #call

selector.positionalArity
// 1

selector.labels
// [#to, #timeout]

selector.arity
// 3

selector.getter?
// false

selector.assignment?
// false
```

Pattern:

```
const pattern = #send(_, ...)

pattern.class
// SelectorPattern

pattern.name
// #send

pattern.minimumArity
// 1

pattern.variadic?
// true

pattern.matches(#send(_))
// true

pattern.matches(#send(_, to))
// true

pattern.matches(#receive(_))
// false
```

Reflection becomes particularly clean:

```
const method = Object.class >> #is(_)

method.selector
// <Selector #is(_)>

method.selector.symbol
// #is(_)

method.selector.name
// #is

method.selector.arity
// 1
```

And pattern-based reflection:

```
Object.class.methods matching: #is(...)
```

The important boundary is:

```
Symbol
    exact symbolic representation

Selector
    validated exact dispatch identity

SelectorPattern
    structural predicate over Selector
```

That separation is strong enough to support the parser, runtime method tables, reflection, LSP, pattern matching, and future typing without making any one class carry too many responsibilities.