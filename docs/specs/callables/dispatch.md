# Dispatch

> **Status:** Draft normative specification  
> **Semantic ownership:** performing selector-identified operations: evaluation order, lookup, inheritance, access, exact and rest dispatch, activation, assignment, lexical `super`, terminal misses, and selector-based operator dispatch.  
> **Related specifications:** [Selectors](selectors.md), [References](references.md), [Families](families.md).

**Dispatch** is the process by which an operation on a receiver is resolved to applicable behavior and performed.

The operation being requested is identified by an exact selector as defined in [Selectors](selectors.md). Dispatch does not redefine selector identity. It takes that identity together with a receiver, evaluated operand values, and the caller's access context, resolves the behavior that handles the operation, and activates it.

The central distinction of this specification is between **selection** and **activation**:

```text
dispatch
    resolve the behavior that handles an exact operation

activation
    bind the already-evaluated values and execute that behavior
```

Ordinary message sends perform both. Reflective exact invocation may begin with behavior that has already been selected. Callable references and Families may perform additional routing before entering ordinary dispatch. Those mechanisms are specified separately, but when they perform an ordinary operation on a receiver, the dispatch laws in this document apply.

## Performing operations

Phalcom presents several surface forms that perform selector-identified behavior:

```phalcom
object.name

object.refresh()
object.send(value)
object.send(value, to: destination)

object.name = value

object[index]
object[index, default: fallback]

object[index] = value
object[index, default: fallback] = value

object + other
```

The corresponding selector kinds are:

| Operation | Exact selector example | Kind |
| --- | --- | --- |
| named read | `name` | Getter |
| named assignment | `name=(_)` | Setter |
| ordinary call | `refresh()`, `send(_)`, `send(_,to)` | Method |
| subscript read | `[_]`, `[_,default]` | SubscriptGet |
| subscript assignment | `[_]=(_)`, `[_,default]=(_)` | SubscriptSet |
| operator protocol call | `+(_)`, `+(from)` | Method |

The structural identity of each selector belongs to [Selectors](selectors.md). In particular, Getter and nullary Method selectors are distinct, as are Setter and unary Method selectors.

Dispatch preserves the selector that the operation requests. It does not fall back between selector kinds merely because two operations consume the same number of values.

Thus:

```phalcom
object.value
```

requests the Getter:

```text
value
```

whereas:

```phalcom
object.value()
```

requests the Method:

```text
value()
```

Similarly:

```phalcom
object.value = next
```

requests:

```text
value=(_)
```

while:

```phalcom
object.value(next)
```

requests:

```text
value(_)
```

If one of these selectors is absent, dispatch does not substitute another kind.

## Evaluation precedes behavioral activation

A performed operation evaluates the expressions that determine its receiver and operands before entering the selected behavior.

For an explicit receiver send:

```phalcom
receiver().send(first(), second(), debug: flag())
```

the observable evaluation order is:

```text
receiver()
first()
second()
flag()
behavioral activation
```

Each of these expressions is evaluated exactly once.

The presence of labels does not reorder evaluation. Labels participate in the selector's structural shape, but argument expressions retain source evaluation order.

**Invariant — `DSP-EVALUATION-ORDER`**

> An operation with an explicit receiver evaluates the receiver first and then evaluates its operand expressions exactly once in source order before behavioral activation.

This invariant concerns evaluation, not selector slot order. Selector structural shape remains governed by `SEL-SLOT-ORDER`.

If evaluation of the receiver or any operand fails, raises, transfers control, or otherwise does not complete normally, later operand expressions are not evaluated and behavioral activation does not begin.

### Assignment evaluation

Named assignment evaluates its receiver before its assigned value:

```phalcom
receiver().name = rhs()
```

has the observable order:

```text
receiver()
rhs()
Setter activation
```

Subscript assignment evaluates every structural subscript operand before the assigned value:

```phalcom
receiver()[row(), column(), debug: mode()] = rhs()
```

has the observable order:

```text
receiver()
row()
column()
mode()
rhs()
SubscriptSet activation
```

The assigned value is therefore evaluated after all structural subscript arguments and exactly once.

This execution rule is distinct from selector identity. Under `SEL-SETTER-VALUE-EXTERNAL`, the assigned value is not a structural selector slot even though dispatch evaluates and supplies it when performing the Setter.

### Spread and dynamically assembled calls

Argument expansion does not introduce a different evaluation model.

For a call such as:

```phalcom
target(before(), *values(), marker: after(), ***more())
```

the argument-producing expressions are evaluated once, from left to right according to their position in the source operation. Their contributions are assembled into the final positional and labeled lanes.

Only after the final argument shape is known can the exact selector for a dynamically shaped send be determined.

Once assembled, the resulting send follows the same selector lookup, access, rest fallback, activation, and miss rules as a statically shaped send.

A runtime mechanism used to assemble argument packs is not part of the language-level dispatch model.

Duplicate or otherwise invalid labels in a dynamically assembled argument shape fail before behavioral lookup proceeds.

## Value application

Applying a value uses ordinary dispatch through the Function call protocol.

For a value expression `f`:

```phalcom
f(a, label: b)
```

performs the equivalent callable operation:

```phalcom
f.call(a, label: b)
```

and therefore requests the exact Method selector:

```text
call(_,label)
```

A nullary application requests:

```text
call()
```

Concrete callable representations may differ in how they ultimately activate their target, but value application enters through the ordinary selector-bearing Function protocol rather than through an arity-only calling convention.

### Unqualified call syntax

An unqualified call:

```phalcom
helper(value)
```

is resolved before dispatch decides whether there is an implicit receiver send.

A lexical or module value named `helper` is applied as a value. If no such value binding resolves and the surrounding context admits an implicit receiver, the expression is an ordinary send to that receiver.

Consequently, a local binding shadows an implicit receiver method of the same source name:

```phalcom
class Example {
  helper() { 10 }

  useMethod() {
    helper()          // implicit self.helper()
  }

  useValue() {
    const helper = || { 20 }
    helper()          // value application
  }
}
```

This name-resolution rule determines the target operation before ordinary dispatch begins. Once the receiver and selector are established, the normal dispatch rules in this document apply.

## Exact lookup

Ordinary dispatch begins with an exact selector.

For an ordinary send, lookup starts at the runtime class of the receiver and proceeds through its superclass chain.

At each class, lookup asks for the exact selector identity requested by the operation. Selector comparison follows `SEL-STRUCTURAL-IDENTITY`.

Conceptually:

```text
receiver runtime class
    ↓
its superclass
    ↓
next superclass
    ↓
...
```

until the exact selector is found or the hierarchy is exhausted.

A method on a more-derived class therefore overrides an inherited method only for the exact selector it defines.

For example:

```phalcom
class Parent {
  render(_ value) { ... }
}

class Child is Parent {
  render(_ value, debug enabled) { ... }
}
```

`Child` has its own:

```text
render(_,debug)
```

but that declaration does not erase the inherited:

```text
render(_)
```

A send:

```phalcom
child.render(value)
```

may therefore select the inherited exact Method, while:

```phalcom
child.render(value, debug: true)
```

selects the Child Method.

This follows from exact selector identity; overriding is not performed by base name alone.

Likewise, defining:

```text
value()
```

does not suppress an inherited Getter:

```text
value
```

and defining:

```text
value(_)
```

does not suppress an inherited Setter:

```text
value=(_)
```

The kinds are part of selector identity.

## Exact lookup precedes rest-capable lookup

A rest-capable Method can accept more than one exact incoming structural shape. It therefore participates in dispatch only after the complete exact-selector search fails.

The lookup phases are:

```text
complete exact-selector search through the eligible hierarchy
    ↓ on complete miss
compatible rest-capable search through the eligible hierarchy
    ↓ on complete miss
terminal message miss
```

**Invariant — `DSP-EXACT-BEFORE-REST`**

> A compatible rest-capable Method is considered only after exact-selector lookup has failed across the entire eligible inheritance chain.

This means an inherited exact Method takes precedence over a compatible rest-capable Method declared on a more-derived class.

For example, given conceptually:

```phalcom
class Parent {
  format(_ value) { ... }
}

class Child is Parent {
  format(*arguments) { ... }
}
```

a send whose exact selector is:

```text
format(_)
```

selects the inherited exact Method before the Child rest-capable Method is considered.

This precedence is deliberate. Rest capability is a fallback for otherwise-unhandled structural shapes, not an override of exact selector identity.

### Rest lookup through inheritance

After exact lookup has missed completely, rest lookup begins at the same lookup origin that exact dispatch used and proceeds toward ancestors.

The nearest compatible rest-capable Method is selected.

A rest Method is compatible according to its declared parameter-shape rules. Compatibility is structural: it depends on positional count, ordered labels, and the Method's rest declaration. It does not inspect runtime argument value types to decide which Method wins.

The detailed meaning of `*`, `**`, and `***` parameter capture belongs to the callable argument specification. Dispatch owns when rest-capable behavior is consulted and how it participates in lookup.

If a selected rest-capable Method cannot be accessed by the caller, that is an access violation. Dispatch does not continue searching for an ancestor merely to evade the selected member's visibility.

## Access is checked on selected behavior

Methods carry access rules independently of selector lookup.

The caller's lexical access authority determines whether the selected behavior may be activated.

Access control does not transform an existing selector into a message miss.

**Invariant — `DSP-ACCESS-NOT-MISS`**

> Selecting behavior that exists but is inaccessible produces an access violation. It must not be treated as an absent selector and must not be forwarded to `doesNotUnderstand`.

This applies to exact behavior and rest-capable behavior.

Consequently, a proxy or `doesNotUnderstand` implementation cannot use a visibility failure as though the receiver simply did not implement the operation.

Access is checked for every invocation path that enters the selected Method, including ordinary sends and forwarding mechanisms that ultimately activate a selected Method.

A forwarding callable must preserve the access authority of the code that invoked the forwarding operation rather than silently substituting the forwarding primitive's own authority.

The exact visibility categories and declaration syntax are defined by the object/member specification. Dispatch defines their consequence: a selected inaccessible Method cannot be activated.

## Selection and activation are distinct

Ordinary sends perform selection and then activation.

Selection determines:

```text
which Method handles this receiver + exact selector + shape?
```

Activation then:

- verifies access;
- binds the already-evaluated argument values according to the selected Method's parameter shape;
- establishes the execution context;
- executes the selected implementation.

This distinction matters because some operations begin with an already-reified Method.

For example, reflective exact invocation may conceptually perform:

```phalcom
method.invokeOn(receiver, ***arguments)
```

without re-running ordinary selector lookup for that Method.

Such an operation must still enforce the Method's access and receiver compatibility and must bind the supplied argument shape correctly, but the identity of the Method has already been chosen.

By contrast, an ordinary send always begins from the receiver and selector.

A bound callable that stores a concrete Method therefore behaves differently from a Family that stores a selector capability and performs live routing. Those callable-value semantics belong to [Families](families.md).

## Named accessors

### Getter dispatch

A named read:

```phalcom
object.name
```

performs exact lookup for the Getter selector:

```text
name
```

It does not search for:

```text
name()
```

if the Getter is absent.

The result of the selected Getter implementation becomes the value of the read expression if execution returns normally.

### Setter dispatch

A named assignment:

```phalcom
object.name = rhs
```

performs exact lookup for:

```text
name=(_)
```

after evaluating the receiver and RHS according to the assignment evaluation rules above.

The assigned value is supplied to the selected Setter's assigned-value position. It is not reinterpreted as an ordinary Method positional slot.

The return contract of the Setter Method and the value of the assignment expression are separate concerns.

A Setter implementation may, as a callable, return any value permitted by its declared contract. Assignment syntax does not expose that return value.

**Invariant — `DSP-ASSIGNMENT-UNIT`**

> After successful Setter or SubscriptSet activation, assignment syntax evaluates to `Unit`, independently of the selected Setter implementation's own return value.

Thus:

```phalcom
class Box {
  value=(_ next) {
    _value = next
    self
  }
}

const result = box.value = 42
```

binds `result` to `Unit`, even if direct reflective activation of the underlying Setter Method would return `self`.

If Setter activation raises or otherwise does not return normally, the assignment expression does not produce `Unit`; the abnormal transfer propagates.

## Subscript dispatch

Subscript syntax participates in the same behavioral model as named operations. It is not a separate collection-only lookup mechanism.

A subscript read:

```phalcom
object[key, default: fallback]
```

requests the exact SubscriptGet selector:

```text
[_,default]
```

A subscript assignment:

```phalcom
object[key, default: fallback] = replacement
```

requests:

```text
[_,default]=(_)
```

The receiver's runtime class and inheritance hierarchy are searched under the same exact-before-rest and access rules as other dispatch, subject to the selector kinds and callable forms that are valid for subscript operations.

Collection types such as List, Map, Tuple, or user-defined indexable objects participate by providing the relevant SubscriptGet or SubscriptSet behavior. Dispatch itself does not assign collection-specific meaning to the bracket syntax.

### Subscript assignment

For:

```phalcom
object[row, column, debug: enabled] = replacement
```

dispatch preserves the distinction specified by `SEL-SETTER-VALUE-EXTERNAL`:

```text
structural selector:
    [_,_,debug]=(_)

structural subscript operands:
    row
    column
    enabled

assigned value:
    replacement
```

All structural operands are evaluated before `replacement`. The selected SubscriptSet receives both the structural subscript values and the one assigned value.

If it completes normally, the assignment expression produces `Unit` under `DSP-ASSIGNMENT-UNIT`.

## Class objects use ordinary dispatch

A class is a runtime object and may receive ordinary messages.

For example:

```phalcom
Fiber.new()
```

is an ordinary Method send to the class object bound to `Fiber`.

It does not use a separate Java-like static-dispatch mechanism merely because the receiver is a class.

The runtime class of a class object—its class-side behavior—determines ordinary method lookup in the same general way that an instance's runtime class determines instance-side lookup.

This is distinct from associated lookup:

```phalcom
Option::Some(42)
```

`::` selects behavior associated with a namespace-bearing owner and is not ordinary receiver-dot dispatch. The associated lookup mechanism is specified separately from the ordinary dispatch rules in this document.

## Lexical `super`

A `super` send changes the origin of lookup while retaining the current runtime receiver.

Conceptually, within behavior defined by `Child`:

```phalcom
super.render(value)
```

does not begin lookup at the runtime class of `self`. Lookup begins after the lexically defining holder—normally at that holder's superclass.

The value received as `self` by the selected ancestor Method remains the original runtime receiver.

**Invariant — `DSP-SUPER-RECEIVER`**

> A lexical `super` send changes the lookup starting point without replacing the runtime receiver.

For example:

```phalcom
class Parent {
  identify() {
    self
  }
}

class Child is Parent {
  identify() {
    super.identify()
  }
}
```

the `Parent` implementation executes with the original `Child` instance as `self`.

`super` does not mean “construct or substitute a parent instance.”

### Exact and rest lookup under `super`

The lookup origin changes, but the ordinary phase ordering remains:

```text
exact selector lookup from the super origin through ancestors
    ↓ on complete exact miss
compatible rest lookup from the same super origin through ancestors
    ↓ on complete miss
terminal miss behavior
```

The lexical anchor belongs to the Method definition, not to the dynamic class of the receiver. Exact invocation of a Method on a subclass therefore does not move that Method's lexical `super` anchor.

## Missing operations

A **message miss** occurs only after the operation has failed to resolve through the applicable dispatch phases.

For ordinary selector dispatch, this means:

```text
exact lookup misses through the full eligible hierarchy
and
compatible rest lookup misses through the full eligible hierarchy
```

An access failure is not a message miss under `DSP-ACCESS-NOT-MISS`.

A Method that is found and then raises is not a message miss.

A Method that returns a sentinel or ordinary value is not a message miss unless a separate protocol explicitly defines that value as a cooperative decline, as bilateral operator dispatch does below.

### `doesNotUnderstand`

A terminal ordinary message miss is forwarded to the receiver's `doesNotUnderstand(_)` behavior.

The forwarded message preserves the operation that was attempted:

- the original exact selector;
- the evaluated operand values;
- their structural positional and labeled shape;
- for Setter/SubscriptSet, the assigned value as the assigned value of that attempted operation.

The miss does not rewrite:

```text
[_,debug]=(_)
```

into another selector form, and it does not invent a synthetic label for the assigned value.

`doesNotUnderstand` is a miss hook. It is not a substitute for ordinary access checks, selector reflection, Family construction, `respondsTo`/`understands` queries, or exact Method invocation.

If the terminal DNU path itself cannot be resolved according to the language's root failure rules, execution terminates with a method-not-understood runtime failure rather than recursively manufacturing an unbounded sequence of misses.

## Dynamic and static sends have the same dispatch semantics

A compiler or runtime may have different machinery for a statically known argument shape and for a shape assembled through spread.

That distinction is not observable dispatch semantics.

For example, if:

```phalcom
receiver.send(1, debug: true)
```

and:

```phalcom
const arguments = (1, debug: true)
receiver.send(***arguments)
```

produce the same final receiver, exact selector, and argument values, they enter the same exact lookup, access, exact-before-rest, activation, and terminal miss rules after argument evaluation has completed.

Differences in when the selector becomes known do not create a second method-resolution model.

## Rest-capable Methods are shape fallbacks, not selector patterns

Selector patterns and rest-capable Method declarations are different mechanisms.

A selector pattern such as:

```text
render(...)
```

describes a set of exact Method selectors. Its semantics belong to [Selectors](selectors.md) and, when referenced, to [References](references.md) and [Families](families.md).

A rest-capable declaration such as conceptually:

```phalcom
render(*values) { ... }
```

declares behavior able to accept a range of incoming argument shapes.

Ordinary dispatch never turns a selector pattern into a lookup key. It first attempts the actual exact selector generated by the send. Rest metadata is consulted only after the exact phase has missed under `DSP-EXACT-BEFORE-REST`.

Rest acceptance is based on structural argument shape, not argument value types.

## Operator syntax and dispatch

Operator syntax does not imply one universal dispatch strategy. Different operator classes have different language semantics.

Where an operator is defined as an ordinary selector operation, its selector follows [Selectors](selectors.md) and its Method or Getter activation follows the normal dispatch rules in this document.

### Unary operators

The ordinary overloadable unary operators use Getter-shaped selector dispatch.

Conceptually:

```phalcom
+value
-value
~value
not value
```

request the corresponding bare operator Getter on `value`:

```text
+
-
~
not
```

A unary operator Getter is distinct from a Method selector such as:

```text
+()
+(_)
```

under `SEL-STRUCTURAL-IDENTITY`.

### Direct Method-form operator calls

An explicit operator Method send such as:

```phalcom
left.+(right)
```

is an ordinary exact Method send for:

```text
+(_)
```

Likewise:

```phalcom
right.+(from: left)
```

is an ordinary exact Method send for:

```text
+(from)
```

These explicit sends perform exactly the selector written. They do not recursively invoke the cooperative binary-operator protocol described below.

### Bilateral arithmetic and bitwise operators

Arithmetic and bitwise binary operator expressions use cooperative bilateral dispatch.

For an operator `OP`:

```phalcom
lhs OP rhs
```

the protocol has a direct candidate and a reflected candidate:

```phalcom
lhs.OP(rhs)
rhs.OP(from: lhs)
```

For example:

```phalcom
lhs + rhs
```

uses candidate selectors:

```text
+(_)
+(from)
```

The reflected Method is defined to compute the original expression `lhs OP rhs`; it does not mean `rhs OP lhs`.

This distinction is significant for non-commutative operations:

```phalcom
lhs - rhs
```

may consult:

```phalcom
lhs.-(rhs)
rhs.-(from: lhs)
```

where the reflected implementation still defines `lhs - rhs`.

The bilateral protocol applies to the arithmetic and bitwise operator families defined by the language, including their direct and reflected selectors. It does not change the meaning of manually sending one of those selectors.

### Cooperative decline

A bilateral protocol candidate can decline the operation by returning the canonical `unsupported` value.

Returning `unsupported` is not an error. It tells the operator-resolution protocol to try its next permitted candidate.

Raising an error is different. A raised error is a real failure and stops bilateral fallback.

Absence of a candidate likewise permits the protocol to consider its other candidate according to the bilateral precedence rules.

If every permitted candidate is absent or declines with `unsupported`, the operator expression fails with an unsupported-operands error identifying the operation and operand kinds.

### Reflected priority for a stricter right-hand subtype

Ordinarily the direct candidate receives the first opportunity.

A stricter right-hand subtype may receive reflected priority when it defines a more-specific reflected implementation for the mixed-type operation. This allows a subtype to participate in an operation with a broader left-hand implementation without being swallowed by that broader direct behavior.

The priority rule does not convert the reflected selector into a different operation. The reflected candidate still computes the original `lhs OP rhs`.

This priority is part of bilateral operator resolution, not ordinary message dispatch. A raw send:

```phalcom
lhs.+(rhs)
```

still performs only ordinary direct dispatch for `+(_)`.

### Three-way comparison

The three-way comparison operator:

```phalcom
lhs <=> rhs
```

is also bilateral, but its protocol is based on `compare(_)`.

It may ask either operand to compare against the other. A successful comparison performed from the right-hand side is reversed so the resulting `Ordering` still describes the original `lhs <=> rhs` expression.

A candidate may decline with `unsupported` under the same cooperative rules.

The detailed semantics of `Ordering` and derived relational operators belong to the comparison specification; the dispatch fact established here is that `<=>` is cooperative bilateral behavior rather than an arity-only intrinsic.

### Operations that are not ordinary overridable dispatch

Not every operator-looking expression is an ordinary selector send.

For example, exact runtime sameness:

```phalcom
a === b
```

has language-defined non-overridable semantics. It must not become ordinary user-overridable Method selection merely because a reflective method-shaped surface may exist.

Likewise lazy logical forms and other compiler-guaranteed semantic relations may have dedicated evaluation rules.

The presence of operator punctuation in source therefore does not by itself determine the dispatch strategy. This document specifies ordinary selector dispatch and the bilateral operator protocol; other intrinsic or control-flow operators are governed by their own semantic specifications.

## Behavioral mutation and later dispatch

Ordinary dispatch uses the receiver's current behavioral hierarchy at the time the operation is performed.

A selector reference or Family may have been constructed earlier, but where its semantics call for live ordinary dispatch, later replacement of the matching Method affects subsequent operations.

For example, conceptually:

```phalcom
const route = &object.render(_)

// behavior for render(_) is replaced

route(value)
```

the Family specification determines whether `route` performs live routing. If it does, the resulting send uses the current ordinary dispatch rules in this document.

By contrast, a `BoundMethod` or exact reified Method represents already-selected behavior and does not redo selector lookup merely because it is later activated.

This distinction is why selection and activation are kept separate throughout the callable model.

## Dispatch guarantees inherited by Families

This document does not define Family construction or Family routing. It does define the ordinary dispatch behavior that a Family uses after it has resolved an exact target operation.

When a bound Family performs live ordinary dispatch, it inherits:

- exact selector identity from [Selectors](selectors.md);
- exact-before-rest precedence under `DSP-EXACT-BEFORE-REST`;
- ordinary access behavior under `DSP-ACCESS-NOT-MISS`;
- lexical caller authority appropriate to the Family invocation;
- terminal DNU behavior on a real dispatch miss;
- assignment result behavior when the source operation is assignment syntax.

The Family specification defines how an exact or pattern Family determines the operation it intends to perform and which Family activation surface selects Getter, Setter, Method, SubscriptGet, or SubscriptSet behavior.

## Dispatch guarantees for implementations

A conforming implementation may optimize lookup and activation, including caching previously selected behavior, provided the optimization is observationally equivalent to the semantics above.

An optimization must not:

- collapse selectors of different kind;
- perform rest fallback before a possible exact inherited match;
- turn an access violation into a miss;
- reorder source operand evaluation;
- evaluate an operand more than once;
- make a previous miss permanent when the receiver's behavior can later change;
- expose a Setter implementation's return value as the result of assignment syntax;
- make static and dynamically assembled argument shapes follow different lookup laws;
- move a lexical `super` lookup origin to the receiver's runtime class.

The means by which the implementation represents selectors, call layouts, method tables, caches, stack windows, or dynamic argument packs are outside this specification.

## Invariants defined by this specification

The following coded invariants are defined here because they constrain multiple independent dispatch paths, are directly testable, and are useful to other specifications and implementation work.

**`DSP-EVALUATION-ORDER`**

> An operation with an explicit receiver evaluates the receiver first and then evaluates its operand expressions exactly once in source order before behavioral activation.

**`DSP-EXACT-BEFORE-REST`**

> A compatible rest-capable Method is considered only after exact-selector lookup has failed across the entire eligible inheritance chain.

**`DSP-ACCESS-NOT-MISS`**

> Selecting behavior that exists but is inaccessible produces an access violation. It must not be treated as an absent selector and must not be forwarded to `doesNotUnderstand`.

**`DSP-ASSIGNMENT-UNIT`**

> After successful Setter or SubscriptSet activation, assignment syntax evaluates to `Unit`, independently of the selected Setter implementation's own return value.

**`DSP-SUPER-RECEIVER`**

> A lexical `super` send changes the lookup starting point without replacing the runtime receiver.

Other rules in this document specify lookup order, activation, rest compatibility, DNU behavior, operator protocols, and source forms. They are not assigned coded invariant names merely to make the document appear more formal.
