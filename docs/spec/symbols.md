# Symbols

## Overview

Symbols are immutable symbolic values representing names, operator
spellings, selector spellings, and other symbolic identifiers.

A symbol is not a string and does not imply any runtime interpretation.
A symbol may be consumed by different parts of the system for different
purposes, including selector lookup, reflection, dispatch metadata,
parsers, protocols, or user-defined domains.

The `#` syntax creates symbolic values. It preserves the symbolic
structure written by the programmer and does not infer meaning from
expression grammar, operator behavior, or runtime method availability.

Important distinctions:

-   A symbol is not a string.
-   A symbol is not necessarily a selector.
-   A symbol may represent a selector spelling.
-   Selector interpretation belongs to selector-consuming APIs and
    reflection facilities.

Examples:

``` phalcom
#name
#+
#method(_)
#"arbitrary text"
```

------------------------------------------------------------------------

# Symbol Literal Syntax

A symbol literal begins with `#` followed by a valid symbolic name or
quoted symbolic content.

General forms:

``` text
#name
#"quoted symbol"
```

Some symbol forms overlap with selector syntax:

``` phalcom
#method
#method()
#method(_)
#property=(put)
```

These represent exact symbolic selector spellings.

Pattern forms are different:

``` phalcom
#method(...)
```

A selector pattern is not an exact symbol and produces a
`SelectorPattern` value.

------------------------------------------------------------------------

# Symbol Categories

## Identifier Symbols

Identifier-like names may be used directly.

Examples:

``` phalcom
#foo
#_foo
#__foo
```

Depending on the identifier grammar, keyword-like names may also be
valid symbolic names:

``` phalcom
#class
#is
#not
```

------------------------------------------------------------------------

## Operator Symbols

Operator spellings may be used as symbols.

Examples:

``` phalcom
#+
#-
#*
#**
#***
#/
#%
#==
#!=
#<
#<=
#>
#>=
#&
#|
#^
#~
#<<
#>>
```

The spelling is preserved exactly.

The parser must not infer expression fixity or selector arity.

These are distinct:

``` phalcom
#+
#+(_)
```

The first is the symbol representing the `+` spelling. The second is the
symbol representing a selector with one explicit argument slot.

------------------------------------------------------------------------

## Punctuation Symbols

Certain punctuation-based symbolic names are valid.

Examples:

``` phalcom
#!
#?
#?.
#??
#...
```

The validity of a symbol name is independent from whether the same
characters form an expression operator.

For example:

``` phalcom
!
```

may not be a valid expression, while:

``` phalcom
#!
```

may still be a valid symbol.

------------------------------------------------------------------------

## Quoted Symbols

Quoted symbols provide an escape hatch for symbolic names that cannot be
represented using bare syntax.

Examples:

``` phalcom
#"!"
#"?"
#"..."
#"hello world"
```

Quoted symbols allow arbitrary symbolic content while avoiding lexical
ambiguity.

------------------------------------------------------------------------

# Symbol Character Rules

Bare symbols use the language's symbolic-name grammar.

Supported categories include:

## Identifier forms

Examples:

``` phalcom
#method
#__field
#_$method
```

## Operator forms

Examples:

``` phalcom
#+
#**
#==
```

## Selector-compatible forms

Examples:

``` phalcom
#is!
#is!(_)
#method(_)
#property=(put)
```

The exact selector grammar is defined separately in the Selector
specification.

------------------------------------------------------------------------

# Symbol Identity and Semantics

Symbols are immutable values.

They support value identity, comparison, and hashing.

Examples:

``` phalcom
#foo == #foo
```

Symbols are distinct from strings:

``` phalcom
#foo != "foo"
```

where the language equality semantics define the corresponding
comparison behavior.

------------------------------------------------------------------------

# Symbols and Selectors

Some symbols represent selector spellings.

For example:

``` phalcom
#+
#+(_)
```

may later be interpreted by reflection or method lookup systems.

However, the symbol itself does not:

-   perform method lookup,
-   require a method to exist,
-   imply a dispatch operation,
-   determine runtime behavior.

A user may freely use:

``` phalcom
#+
```

as a domain-specific symbolic value.

Selector interpretation occurs only when a consumer explicitly operates
in the selector domain.

------------------------------------------------------------------------

# Examples

``` phalcom
const plus = #+
const comparison = #==
const setter = #value=(put)
const internal = #_$method
const arbitrary = #"not a bare symbol"
```

Summary:

``` text
#name
    creates a symbol

#operator
    creates a symbol

#name(signature)
    creates an exact selector symbol

#name(...)
    creates a selector pattern

#"text"
    creates an explicitly quoted symbol
```
