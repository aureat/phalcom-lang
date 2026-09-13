# Selectors

> **Status:** Draft normative specification  
> **Semantic ownership:** selector identity, selector kinds, structural argument shape, exact selectors, selector patterns, and canonical selector notation.  
> **Related specifications:** [Dispatch](dispatch.md), [References](references.md), [Families](families.md).

A **selector** is Phalcom's structural identity for an operation that can participate in behavioral lookup. A selector identifies *which operation is meant*; it does not identify the receiver that will perform it, the implementation that will run, the values supplied as arguments, or the place where the operation was declared.

Selectors are used by ordinary methods, getters and setters, subscript accessors, operators, associated callables, callable references, reflection, and dispatch. The same selector identity is used wherever the same operation shape is meant.

This specification defines selectors themselves. It does not define how a selector is dispatched, how `&` constructs a callable reference, or how a `Family` value later activates a referenced selector. Those behaviors belong to the related specifications.

## Selector identity

An exact selector is determined by three things:

- its **base**;
- its **kind**;
- its ordered sequence of **structural slots**.

Nothing else participates in selector identity.

The receiver is not part of a selector. The declaring class or trait is not part of a selector. An associated owner such as `Option` in `Option::Some(42)` is not part of the selector. Visibility, implementation body, generic specialization, runtime argument values, and return type are not part of the selector.

Thus the selector in:

```phalcom
point.move(delta, to: destination)
```

is structurally:

```text
move(_,to)
```

The selector in:

```phalcom
Option::Some(42)
```

is structurally:

```text
Some(_)
```

`Option` determines where the associated operation is looked up; it does not become part of `Some(_)`.

**Invariant — `SEL-STRUCTURAL-IDENTITY`**

> Two exact selectors are identical if and only if they have the same base, the same selector kind, and the same ordered structural slots.

The selector kind is therefore identity-bearing. Operations that happen to use the same base and the same number of values are still different selectors when their kinds differ.

For example:

```text
value       Getter
value()     Method
value=(_)   Setter
value(_)    Method
```

are four distinct selectors.

Likewise:

```text
[_]         SubscriptGet
[_]=(_)     SubscriptSet
```

are distinct selectors.

This distinction is structural. Dispatch must not infer the selector kind from argument count.

## Selector bases

A selector has either a **named base** or the **subscript base**.

A named base is the non-subscript name around which related selectors are grouped. Ordinary identifier names and operator spellings can serve as named bases:

```text
render
size
+
==
>=
```

The term *named* distinguishes this category from the special subscript base; it does not mean that every base must be an identifier.

The subscript base is the distinguished bracket operation:

```text
[...]
```

It does not have a user-defined textual method name in selector notation. Its structural argument shape appears between brackets.

Selectors that share a base form a **selector family** in the structural sense used by this specification. For example, these selectors share the base `value`:

```text
value
value=(_)
value()
value(_)
value(_,debug)
```

This structural notion of a selector family is not the runtime `Family` value produced by callable references. Runtime `Family` values are specified in [Families](families.md).

## Selector kinds

Phalcom has five exact selector kinds:

```text
Getter
Setter
Method
SubscriptGet
SubscriptSet
```

The kind determines how the selector's shape is interpreted. It is not syntactic decoration that may be discarded after parsing.

### Getters

A Getter is a named selector with no structural slots.

Its selector notation is the base itself:

```text
size
name
current
```

A Getter is not a nullary Method. The following selectors are distinct:

```text
status      Getter
status()    Method
```

Whether both may coexist on the same receiver is a declaration and dispatch question, but selector identity preserves the distinction.

### Setters

A Setter is a named selector with no structural slots and exactly one assigned-value position.

Its exact selector notation is:

```text
name=(_)
```

For example:

```text
value=(_)
color=(_)
current=(_)
```

The `_` after `=` denotes the single assigned-value position. It is not a structural positional slot.

A Setter is not a unary Method:

```text
value=(_)   Setter
value(_)    Method
```

These remain distinct even though both consume one value when performed.

### Methods

A Method is a named selector whose structural slots appear inside parentheses.

A nullary Method has an empty structural shape:

```text
refresh()
reset()
```

A positional slot is written `_`:

```text
append(_)
compare(_,_)
```

A labeled slot is written by label name:

```text
move(_,to)
render(_,debug)
schedule(_,at,priority)
```

Selector notation records labels, not call-site values or parameter variable names.

For a call such as:

```phalcom
canvas.render(node, debug: true)
```

the Method selector is:

```text
render(_,debug)
```

not:

```text
render(_,debug:true)
```

and not a representation of the values `node` or `true`.

### Subscript getters

A SubscriptGet uses the distinguished subscript base. Its structural slots appear between brackets:

```text
[]
[_]
[_,_]
[_,default]
[_,_,debug]
```

Like Method selectors, the bracket shape records positional and labeled structure only.

For:

```phalcom
table[key, default: fallback]
```

the exact selector is:

```text
[_,default]
```

### Subscript setters

A SubscriptSet combines a structural bracket shape with exactly one assigned-value position:

```text
[]=(_)
[_]=(_)
[_,default]=(_)
[_,_,debug]=(_)
```

The bracket contents describe the subscript selector shape. The `(_)` after `=` is the single assigned-value position.

For:

```phalcom
table[row, column, debug: true] = value
```

the exact selector is:

```text
[_,_,debug]=(_)
```

Its structural slots are:

```text
Positional
Positional
Label(debug)
```

The assigned value is separate from those slots.

## Structural slots

A selector's structural argument shape is an ordered sequence of slots.

A slot is one of:

```text
Positional
Label(name)
```

Selector notation renders a positional slot as `_` and a labeled slot as its label:

```text
(_,_,debug)
```

means:

```text
Positional
Positional
Label(debug)
```

The slot sequence is ordered. Labels are not a map and are not normalized into a canonical alphabetical order.

These selectors are distinct:

```text
move(_,to,duration)
move(_,duration,to)
```

Likewise:

```text
configure(mode,debug)
configure(debug,mode)
```

are distinct selectors.

The order is part of structural identity under `SEL-STRUCTURAL-IDENTITY`.

### Positional slots precede labeled slots

An exact selector shape consists of a positional prefix followed by a labeled suffix.

These are valid shapes:

```text
()
(_)
(_,_)
(_,to)
(_,to,duration)
(mode)
(mode,debug)
```

A positional structural slot may not follow a labeled structural slot:

```text
move(to,_)          // invalid selector shape
configure(mode,_)   // invalid selector shape
```

**Invariant — `SEL-SLOT-ORDER`**

> In an exact selector, every positional structural slot precedes every labeled structural slot. The relative order of all structural slots is preserved as part of selector identity.

This law is about selector structure. It does not say that every operation must contain positional slots, or that labeled-only selectors are invalid.

### Labels are structural identities

A labeled slot records the label itself.

For example:

```text
send(_,to)
send(_,from)
```

are different selectors.

A parameter's local binding name does not participate in selector identity. If two declarations use different local variable names while exposing the same positional and labeled shape, their selector shape is the same.

The same applies to calls: selector formation depends on the argument lanes and label sequence, not on the runtime argument values.

## The assigned-value position of setters

Setters have one semantic feature that ordinary Method shapes do not: an assigned-value position.

For a named Setter:

```text
property=(_)
```

there are no structural selector slots.

For a SubscriptSet:

```text
[_,debug]=(_)
```

the structural slots are only:

```text
Positional
Label(debug)
```

The final assigned value does not become another positional slot after `debug`.

**Invariant — `SEL-SETTER-VALUE-EXTERNAL`**

> The assigned value of a Setter or SubscriptSet is intrinsic to the selector kind but is not a structural selector slot.

This invariant preserves the ordinary structural slot law. In particular, the following selector is valid:

```text
[_,debug]=(_)
```

without creating an impossible structural sequence equivalent to:

```text
Positional
Label(debug)
Positional
```

No such third slot exists. The assigned-value position is separate.

### Setter arity and selector slot count are different concepts

The number of values involved in performing a setter must not be confused with the number of structural selector slots.

For:

```text
value=(_)
```

the selector has:

```text
structural slots: 0
assigned values:  1
```

For:

```text
[_,debug]=(_)
```

the selector has:

```text
structural slots: 2
assigned values:  1
```

This distinction is observable in selector reflection and pattern matching. The runtime representation used to carry those values is outside this specification.

### Exactly one assigned value

Setter syntax denotes exactly one assigned-value position.

Forms attempting to give a Setter multiple assigned values do not denote valid exact selectors:

```text
value=(_,_)          // invalid
value=(_,_,debug)    // invalid
[_,debug]=(_,_)      // invalid
```

The structural argument shape of a SubscriptSet belongs inside the brackets; the one assigned-value position belongs after `=`.

## Operators

Operator selectors use the same structural Method identity as other named Method selectors.

Examples:

```text
+()
+(_)
+(from)

==(_)
>=(_)
<=(_)
```

Parenthesized operator selectors are Method selectors. Punctuation inside the operator base remains part of that base.

In particular, the presence of `=` inside an operator spelling does not make the selector a Setter:

```text
==(_)    Method
>=(_)    Method
<=(_)    Method
```

The dedicated Setter form is an accessor-shaped base followed by the setter suffix:

```text
property=(_)
```

The selector parser must therefore distinguish a Setter suffix from an operator token whose base itself contains `=`.

Operator bases participate in selector families and selector patterns in the same structural manner as other non-subscript bases where the corresponding selector kind is valid.

## Exact selector notation

Exact selector notation is the compact structural notation used throughout the specification.

The core forms are:

```text
getter:
    base

setter:
    base=(_)

method:
    base(slots)

subscript getter:
    [slots]

subscript setter:
    [slots]=(_)
```

where:

```text
slot := _ | label
```

and a sequence of exact slots obeys `SEL-SLOT-ORDER`.

Examples:

```text
size
size=(_)

refresh()
append(_)
move(_,to,duration)

[]
[_]
[_,default]

[]=(_)
[_]=(_)
[_,default]=(_)
```

Whitespace may be inserted in explanatory source-like notation:

```text
move(_, to, duration)
```

but the canonical compact exact-selector text omits that whitespace:

```text
move(_,to,duration)
```

### Source declarations and exact selector identity

Source syntax may contain local parameter names and types that are not part of selector identity.

Conceptually:

```phalcom
move(_ delta, to destination) {
    ...
}
```

defines a Method whose selector is:

```text
move(_,to)
```

Likewise a setter declaration conceptually shaped as:

```phalcom
value=(_ next) {
    ...
}
```

defines:

```text
value=(_)
```

and a subscript setter shaped as:

```phalcom
[_ key, debug enabled]=(_ value) {
    ...
}
```

defines:

```text
[_,debug]=(_)
```

Only the structural operation shape enters the selector. Local binding names and annotations belong to the callable declaration, not the selector.

## Selector families

Selectors sharing a base form a structural family.

For an ordinary named base such as `value`, the family may contain selectors of different kinds and shapes:

```text
value
value=(_)
value()
value(_)
value(_,debug)
```

For the subscript base, the family contains SubscriptGet and SubscriptSet selectors of their various structural shapes:

```text
[]
[_]
[_,default]

[]=(_)
[_]=(_)
[_,default]=(_)
```

A selector family is a conceptual set. It is useful when defining selector patterns, references, and dispatch, but it is not itself an exact selector.

## Selector patterns

A **selector pattern** denotes a set of exact selectors.

Patterns are used only in syntactic contexts that admit selector patterns. They are not exact selectors and do not become exact lookup identities merely because they have a textual representation.

A pattern constrains:

- a selector base;
- a selector-kind predicate;
- optionally, a structural slot prefix;
- optionally, a structural slot suffix;
- optionally, one gap between the fixed prefix and suffix.

The gap is written:

```text
...
```

and matches zero or more structural slots.

The ellipsis in a selector pattern is a structural wildcard. It is not Phalcom's rest/spread syntax. `*`, `**`, and `***` have separate callable-argument meanings.

### Method patterns

A parenthesized selector pattern is Method-only.

```text
render(...)
```

matches Method selectors with base `render`, regardless of structural slot count or labels, subject to the ordinary validity of those exact selectors.

It does not match:

```text
render        Getter
render=(_)    Setter
```

Constrained Method patterns retain fixed slots around the gap:

```text
route(_, ...)
route(..., debug)
route(_, ..., debug)
route(..., _, target)
```

Examples:

```text
route(_, ...)
```

matches:

```text
route(_)
route(_,_)
route(_,target)
route(_,_,debug)
```

but not a Method selector with no leading positional slot.

Because the gap may absorb zero structural slots, the fixed prefix and suffix alone may satisfy the pattern.

For example:

```text
route(...,debug)
```

matches:

```text
route(debug)
```

as well as:

```text
route(_,debug)
route(_,_,debug)
```

provided the exact selector remains structurally valid.

### Whole named-base patterns

A trailing ellipsis outside parentheses denotes the complete named selector family for the base:

```text
render...
value...
+...
```

For an ordinary accessor-capable named base, this includes selectors whose kinds are:

```text
Getter
Setter
Method
```

The pattern still requires the same base.

This is deliberately different from:

```text
render(...)
```

which is Method-only.

Contrast:

```text
value...       complete named family
value(...)     Method selectors only
```

The distinction between these forms is structural and must be preserved by references and Families.

### Named accessor patterns

A trailing `=` with no assigned-value placeholder denotes the named accessor kinds for a base:

```text
value=
name=
current=
```

Such a pattern accepts:

```text
Getter
Setter
```

for the same base and excludes Method selectors.

Thus:

```text
value=
```

may match:

```text
value
value=(_)
```

but not:

```text
value()
value(_)
```

The pattern:

```text
value=
```

must not be confused with the exact Setter selector:

```text
value=(_)
```

### Subscript getter patterns

A bracket pattern with an ellipsis and no setter suffix is SubscriptGet-only:

```text
[...]
[_, ...]
[..., debug]
[_, ..., debug]
```

For example:

```text
[_, ..., debug]
```

matches SubscriptGet selectors whose structural shape begins with a positional slot and ends with the label `debug`.

It does not match SubscriptSet selectors.

### Subscript setter patterns

A bracket pattern followed by `=(_)` is SubscriptSet-only:

```text
[...]=(_)
[_, ...]=(_)
[..., debug]=(_)
[_, ..., debug]=(_)
```

The `(_)` after `=` continues to denote the single assigned-value position. It is not part of the gap and is not matched as a structural slot.

For example:

```text
[_, ..., debug]=(_)
```

matches SubscriptSet selectors such as:

```text
[_,debug]=(_)
[_,_,debug]=(_)
[_,_,_,debug]=(_)
```

but not SubscriptGet selectors.

### Whole subscript-accessor patterns

A bracket pattern followed by a trailing `=` and no `(_)` accepts both subscript accessor kinds:

```text
[...]=
[_, ...]=
[..., debug]=
[_, ..., debug]=
```

For example:

```text
[_, ..., debug]=
```

may match both:

```text
[_,debug]
[_,debug]=(_)
```

and wider corresponding shapes, so long as the structural prefix and suffix match.

This is the subscript counterpart of a named accessor pattern such as:

```text
value=
```

Unlike named bases, the subscript base has no Method kind. Its accessor kinds are exactly:

```text
SubscriptGet
SubscriptSet
```

### Pattern matching

A selector pattern matches an exact selector only when all of the following hold:

- the selector base matches the pattern base;
- the selector kind satisfies the pattern's kind predicate;
- the exact selector contains at least as many structural slots as the pattern's fixed prefix and suffix require;
- its structural slots begin with the pattern prefix;
- its structural slots end with the pattern suffix;
- any remaining structural slots are absorbed by the single gap.

Pattern matching compares structural slots in order. It does not reorder labels, sort them, reinterpret a label as positional, or inspect runtime argument values.

For a pattern with prefix `P`, suffix `S`, and one gap:

```text
P ... S
```

an exact slot sequence `X` matches when:

```text
X starts with P
X ends with S
length(X) >= length(P) + length(S)
```

and the selector base and kind predicate also match.

The gap therefore denotes zero or more whole structural slots, not arbitrary text.

### Pattern validity

A selector pattern must itself be compatible with valid exact selector structure.

For example, a pattern cannot require a labeled fixed prefix followed later by a positional fixed suffix, because any matching exact selector would violate `SEL-SLOT-ORDER`.

A slot-gap pattern contains one structural gap. Whole-family and accessor patterns such as:

```text
value...
value=
```

are dedicated kind/base patterns rather than attempts to encode an ordinary slot gap.

## Exact selectors and patterns are different semantic objects

An exact selector identifies one operation shape:

```text
render(_,debug)
```

A selector pattern denotes a set of such shapes:

```text
render(...)
render(...,debug)
render...
```

This distinction is important even when a pattern happens, in a particular program, to match only one existing selector.

The existence or absence of matching implementations does not turn a pattern into an exact selector.

Dispatch uses exact selectors as operation identities. How pattern-based references select and later perform matching operations is specified by [References](references.md) and [Families](families.md).

## Canonical selector representation

Every exact selector has a canonical textual representation corresponding to its structural identity.

For ordinary labels and bases, canonical exact selector text uses:

```text
name
name=(_)
name()
name(_)
name(_,label)
[_,label]
[_,label]=(_)
```

Canonical exact selector text:

- preserves the selector kind;
- preserves the base;
- preserves the structural slot sequence;
- preserves label order;
- omits explanatory whitespace.

Under `SEL-STRUCTURAL-IDENTITY`, two selectors with different canonical structural components must not canonicalize to the same exact selector text.

### Label-component escaping

Selector labels may have textual names that would otherwise collide with structural notation. Canonical selector representation must therefore encode label components reversibly.

A label that can be represented unambiguously as an ordinary selector component is emitted directly.

A label component that would collide with reserved structural markers or delimiters is escaped. The canonical escaped form is:

```text
~<lowercase hexadecimal UTF-8 bytes>
```

For example, a label whose literal text is `_` cannot be emitted as `_`, because `_` denotes a positional structural slot. Its label representation must use the escaped component rather than becoming positional.

The exact encoder must be reversible: canonical selector text must recover the same label sequence and therefore the same selector identity.

The following spellings are structurally reserved and, when intended as literal label text, require escaping:

```text
_
*
**
***
```

A label beginning with `~`, containing selector delimiters, or otherwise not representable as an unambiguous ordinary component likewise uses the canonical escaped form.

This escaping belongs to selector textual representation. It does not change the logical label identity.

## Rest-capable callables and selector patterns

Phalcom's callable syntax also uses:

```text
*
**
***
```

for positional, labeled, and complete rest/spread behavior.

Those markers describe callable parameter or argument transport. They are not selector-pattern ellipses and do not change the meaning of:

```text
...
```

inside a selector pattern.

A selector pattern such as:

```text
call(...)
```

means “Method selectors with base `call` whose structural slots satisfy this pattern.” It does not mean that the target declaration has a rest parameter.

Conversely, a rest-capable Method is still part of a named selector family, but the rules by which an actual argument shape is accepted by a rest-capable Method belong to [Dispatch](dispatch.md). Selector patterns do not perform rest-parameter binding.

## Selector notation and operation syntax

Selector notation describes identity; ordinary Phalcom expressions use operation syntax.

For example:

```phalcom
object.move(delta, to: target)
```

performs a Method operation whose exact selector is:

```text
move(_,to)
```

```phalcom
object.name
```

performs a Getter operation whose selector is:

```text
name
```

```phalcom
object.name = value
```

performs a Setter operation whose selector is:

```text
name=(_)
```

```phalcom
object[index, default: fallback]
```

performs a SubscriptGet operation whose selector is:

```text
[_,default]
```

```phalcom
object[index, default: fallback] = value
```

performs a SubscriptSet operation whose selector is:

```text
[_,default]=(_)
```

This specification stops at selector identity. Receiver evaluation, argument evaluation, lookup, visibility, setter execution, failure, and the value produced by an assignment expression are defined in [Dispatch](dispatch.md).

Likewise, syntax such as:

```phalcom
&object.move(_)
&object.move...
&object[...]=(_)
```

uses selector specifications in a reference expression, but the meaning of `&`, receiver capture, and the value produced by the reference are defined in [References](references.md) and [Families](families.md).

## Invalid selector forms

A form fails to denote a valid exact selector when its structural identity is impossible or ambiguous under the rules above.

Examples include a positional slot after a label:

```text
move(to,_)
```

multiple assigned-value positions:

```text
value=(_,_)
[_,debug]=(_,_)
```

and a selector pattern with an impossible fixed slot ordering:

```text
method(label,...,_)
```

when the fixed label before the gap and fixed positional slot after it would require every matching exact selector to violate `SEL-SLOT-ORDER`.

A malformed selector must not be silently normalized into a different valid selector. In particular:

- label order must not be sorted;
- a labeled slot must not be converted into positional form;
- a Setter must not be converted into a unary Method;
- a Getter must not be converted into a nullary Method;
- the assigned-value position of a Setter must not be inserted into the structural slot sequence.

## Invariants defined by this specification

The following coded invariants are defined here because they are fundamental, durable, independently testable, and relied upon by other selector-consuming semantics.

**`SEL-STRUCTURAL-IDENTITY`**

> Two exact selectors are identical if and only if they have the same base, the same selector kind, and the same ordered structural slots.

**`SEL-SLOT-ORDER`**

> In an exact selector, every positional structural slot precedes every labeled structural slot. The relative order of all structural slots is preserved as part of selector identity.

**`SEL-SETTER-VALUE-EXTERNAL`**

> The assigned value of a Setter or SubscriptSet is intrinsic to the selector kind but is not a structural selector slot.

Other rules in this document are specifications of selector syntax, taxonomy, validity, and matching. They are not assigned invariant codes merely for uniformity. A coded invariant is reserved for a semantic law that other specifications and implementation work have a genuine reason to cite directly.
