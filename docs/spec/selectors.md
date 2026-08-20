# Selectors

## Overview

Selectors are symbolic identifiers used by Phalcom's message-send model,
method lookup, reflection system, and dispatch machinery.

A selector describes the name and signature of a message. Selectors are
represented through symbolic values, but selector semantics are distinct
from general symbol semantics.

The symbol syntax:

``` phalcom
#name
```

creates a symbolic value.

When the symbolic value describes an exact selector spelling, it can be
interpreted by selector-aware APIs.

Important distinctions:

-   A selector is not a method implementation.
-   A selector does not require a method to exist.
-   A selector symbol does not perform lookup by itself.
-   A selector pattern is not an exact selector.

Selector interpretation happens in selector-consuming contexts such as
reflection, dispatch, and method lookup.

------------------------------------------------------------------------

# Selector Structure

A selector consists of:

1.  A selector name.
2.  An optional argument signature.

The signature determines the message shape.

Examples:

``` phalcom
#method
#method()
#method(_)
#method(_, _)
```

These are different selectors.

They represent:

``` text
method getter

method nullary method

method unary method

method two-argument method
```

The parser preserves these distinctions.

------------------------------------------------------------------------

# Getter Selectors

A selector without an explicit argument signature is a getter selector.

Example:

``` phalcom
#name
```

A getter selector represents a message with no explicit argument slots.

It is distinct from a nullary method.

``` phalcom
#name != #name()
```

The parentheses are meaningful selector syntax.

------------------------------------------------------------------------

# Nullary Method Selectors

A selector with an empty argument list is a nullary method selector.

Example:

``` phalcom
#name()
```

This is different from:

``` phalcom
#name
```

The distinction allows Phalcom to differentiate getter-style access from
an explicitly callable method with zero arguments.

------------------------------------------------------------------------

# Positional Parameters

Selectors may contain positional argument slots.

Examples:

``` phalcom
#method(_)
#method(_, _)
```

Each `_` represents a positional parameter slot.

Selector arity is determined only by the explicit signature.

A bare selector name never gains arguments implicitly.

Incorrect:

``` text
#+  ->  #+(_)
```

Correct:

``` phalcom
#+
#+(_)
```

------------------------------------------------------------------------

# Labeled Parameters

Phalcom selectors may contain labeled parameters.

Positional parameters precede labeled parameters.

Example:

``` phalcom
#method(_, duration)
```

The ordering rule is:

``` text
positional parameters
        followed by
labeled parameters
```

Therefore forms where labeled parameters appear before later positional
parameters are invalid.

Example:

``` phalcom
#method(duration, _)
```

is not a valid selector structure if the first component is a label.

------------------------------------------------------------------------

# Setter Selectors

Setter methods are represented as selector symbols.

Examples:

``` phalcom
#property=(put)
```

Setter selectors remain exact selector symbols.

They do not have special runtime behavior at the symbol level.

A selector-consuming API may interpret them as assignment selectors.

------------------------------------------------------------------------

# Index Selectors

Indexing operations are represented as selectors as well.

Examples:

``` phalcom
#[index]
#[x, y]

#[_]

#[x, y]=(put)
#[_]=(put)
```

Index selectors allow indexing and assignment operations to participate
fully in reflection and selector APIs.

If an operation exists as a message, its selector should be
representable symbolically.

------------------------------------------------------------------------

# Operator Selectors

Operators are selectors when used in message sends.

Examples:

``` phalcom
#+
#+(_)

#-
#-(_)
```

Unary operators:

``` phalcom
+value
-value
```

lower to getter-form message sends:

``` phalcom
value.+
value.-
```

Binary operators:

``` phalcom
left + right
```

lower to argument-bearing selectors:

``` phalcom
left.+(right)
```

However, symbol syntax does not infer this distinction.

These remain separate:

``` phalcom
#+
#+(_)
```

------------------------------------------------------------------------

# Selector Symbols

Exact selectors are represented as symbols.

Examples:

``` phalcom
#method
#method()
#method(_)

#+
#+(_)

#property=(put)

#[x,y]
```

The symbol preserves the written selector structure.

The parser does not:

-   check whether a method exists,
-   perform lookup,
-   bind the selector to a class,
-   infer missing arguments.

------------------------------------------------------------------------

# Selector Patterns

Selector patterns represent a set of possible selectors rather than one
exact selector.

Pattern syntax uses selector-pattern metasyntax such as `...`.

Examples:

``` phalcom
#method(...)
#method(..., _)
```

These produce:

``` text
SelectorPattern
```

not:

``` text
Symbol
```

A selector pattern is conceptually a matcher.

Example:

``` phalcom
pattern = #method(...)

pattern.matches(#method)
pattern.matches(#method(_))
pattern.matches(#method(_, _))
```

A selector pattern does not contain a single selector.

Therefore operations such as:

``` phalcom
#method(...).selector
```

are not meaningful.

Expanding a pattern into matching selectors requires an external
context, such as a class method dictionary.

------------------------------------------------------------------------

# Selector Pattern Ordering Rules

Pattern arguments follow the same positional/labeled ordering rules as
exact selectors.

The variadic pattern component:

``` phalcom
...
```

may appear only where permitted by the selector grammar.

Examples:

Valid:

``` phalcom
#method(...)
#method(_, ...)
```

Invalid:

``` phalcom
#method(label, ...)
#method(_, ..., label)
```

The exact placement rules are determined by selector signature grammar.

------------------------------------------------------------------------

# Whitespace Rules

Whitespace inside selector literals is insignificant.

Equivalent forms:

``` phalcom
#method(_, _)

#method(_, _)

#method ( _, _ )
```

all represent the same selector.

Whitespace must not determine whether parentheses belong to the selector
or represent a method call.

------------------------------------------------------------------------

# Selector Literal Completion and Calls

The parser consumes the complete selector literal first.

A later postfix call applies to the resulting value.

Example:

``` phalcom
#method(_ )()
```

means:

``` text
construct selector symbol #method(_)
then send call()
```

It does not mean that the first parentheses were a function call.

To call the getter symbol itself:

``` phalcom
(#method)()
```

explicit grouping can be used.

------------------------------------------------------------------------

# Selector Names

Selector names may include:

-   identifiers
-   valid operator spellings
-   supported punctuation forms
-   internal namespace forms

Examples:

``` phalcom
#is!
#try!
#+
#==
#_$method
#__field
#__method__
```

The selector grammar is independent from expression operator grammar.

A character sequence may be valid as a selector name without being a
standalone expression operator.

------------------------------------------------------------------------

# Relationship Between Symbols and Selectors

Symbols are general-purpose symbolic values.

Selectors are one domain that symbols can represent.

The relationship:

``` text
symbol syntax
       |
       v
exact selector spelling
       |
       v
selector-consuming context
       |
       v
method lookup / reflection / dispatch
```

The parser preserves symbolic structure.

The consumer decides whether that symbol participates in selector
semantics.

------------------------------------------------------------------------

# Summary

``` text
#name
        getter selector symbol

#name()
        nullary selector symbol

#name(_)
        positional selector symbol

#name(_, label)
        mixed selector symbol

#name=(put)
        setter selector symbol

#[x,y]
        index selector symbol

#name(...)
        SelectorPattern
```

Core rule:

> Selectors are explicit symbolic structures. Their arity, labels, and
> pattern behavior come only from syntax written by the programmer.
