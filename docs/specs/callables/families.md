# Families

> **Status:** Draft normative specification  
> **Semantic ownership:** first-class `Family` capabilities produced by callable references: exact and pattern Families, bound and associated routing, live lookup, Method/Getter/Setter/Subscript activation, accessor aliases, Tuple-shaped subscript APIs, failure behavior, and the distinction from reflective `MethodFamily` snapshots.  
> **Related specifications:** [Selectors](selectors.md), [Dispatch](dispatch.md), [References](references.md).

A **Family** is a first-class callable capability representing a selector specification together with the receiver or associated context required to perform it later.

Callable references create Families:

```phalcom
const render = &object.render(_)
const overloads = &object.render(...)
const accessors = &object.value=
const constructors = &Option::Some...
```

The reference expression determines what selector capability is captured; this document defines what the resulting Family means and how it is activated.

A Family is not a reflected `Method`. It need not contain a concrete currently selected implementation. A bound Family normally retains a receiver plus an exact selector or selector pattern and resolves behavior when activated.

A Family is also distinct from `MethodFamily`, the immutable reflective snapshot of concrete Methods described later in this document.

## Family capabilities

A Family retains enough information to answer two questions at activation time:

```text
which selector operation is being requested?
on which bound receiver or associated callable context?
```

The selector specification may be exact:

```phalcom
&object.render(_)
&object.value
&object[_, debug]=(_)
```

or patterned:

```phalcom
&object.render(...)
&object.render...
&object.value=
&object[...]=
```

The resulting Family preserves that distinction.

An exact Family represents one exact selector capability.

A pattern Family represents a selector predicate capable of routing multiple exact selector shapes.

The Family itself does not alter selector identity. Exact selectors and selector-pattern matching remain governed by [Selectors](selectors.md).

## Bound Families

A bound Family is produced from a receiver reference:

```phalcom
const route = &router.route(_)
```

Under `REF-RECEIVER-ONCE`, `router` has already been evaluated and retained when the Family is created.

The Family retains the receiver and selector specification. It does not normally freeze a concrete target Method.

For example:

```phalcom
const route = &router.route(_)
```

conceptually retains:

```text
receiver:
    router

selector specification:
    route(_)
```

not:

```text
the Method object currently implementing route(_)
```

When the Family is later activated, it routes through the receiver's current behavior according to the rules below.

**Invariant — `FAM-BOUND-LOOKUP-LIVE`**

> A bound Family retains its receiver and selector specification, not a frozen concrete Method; activation observes the receiver's current applicable behavior.

Thus, if the implementation of:

```text
route(_)
```

is replaced after Family construction, a later activation may observe that replacement.

Likewise, a pattern Family may observe newly available or replaced matching behavior.

This invariant applies to executable bound Families. It does not apply to `MethodFamily`, whose purpose is to be an immutable reflective snapshot.

## Exact Families

An exact Family retains one exact selector.

For example:

```phalcom
const render = &object.render(_)
```

retains:

```text
render(_)
```

When activated as a Method operation with one positional argument:

```phalcom
render(value)
```

the Family requests that exact Method selector on its stored receiver.

It does not derive a replacement selector from the runtime argument labels.

A call shape incompatible with the exact selector is a Family invocation-shape error rather than permission to choose another selector.

For example:

```phalcom
const render = &object.render(_)
```

does not become a reference to:

```text
render(_,debug)
```

merely because a later caller supplies:

```phalcom
render(value, debug: true)
```

The Family capability is exact.

### Exact lookup remains live

Although the selector identity is fixed, the Method implementation is selected when the Family performs the operation.

Therefore:

```phalcom
const render = &object.render(_)

// object.render(_) implementation is replaced

render(value)
```

uses the current implementation selected by ordinary dispatch at activation time.

The Family did not become a `BoundMethod` merely because its selector was exact.

## Pattern Families

A pattern Family retains an immutable selector predicate and its target context.

For example:

```phalcom
const render = &object.render(...)
```

retains a Method-only selector pattern for base `render`.

At activation time, the Family forms the candidate exact selector implied by the activation kind and incoming structural shape, verifies that the selector satisfies the retained pattern, and then performs that exact operation using the target's current behavior.

Conceptually:

```text
incoming activation
    ↓
derive candidate exact selector
    ↓
candidate matches retained selector pattern?
    ↓ yes
perform current target dispatch
```

The pattern itself does not change when Methods are added or replaced.

The target behavior consulted by activation remains live.

Therefore a Family may be created before a matching Method exists:

```phalcom
const route = &object.route(...)
```

and later activation may succeed if matching behavior has become available by then, subject to the language's behavior-mutation rules.

Construction is not an eager snapshot of matching Methods.

## Family activation has an explicit operation kind

A Family may represent more than one selector kind.

For example:

```phalcom
const family = &object.value...
```

can represent Getter, Setter, and Method selectors of base `value`.

Phalcom does not choose among those kinds by guessing from argument count.

Activation syntax selects the intended operation kind.

**Invariant — `FAM-ACTIVATION-KIND-EXPLICIT`**

> Family activation preserves an explicit selector kind; activation must not fall back to a different kind merely because another kind could consume the same number of values.

This is essential for the distinction between:

```text
value       Getter
value()     nullary Method
```

and:

```text
value=(_)   Setter
value(_)    unary Method
```

The Family API therefore provides kind-specific activation surfaces.

## Method activation

Ordinary Family call syntax is Method-kind activation:

```phalcom
family()
family(value)
family(value, debug: true)
```

For a Family whose selector specification admits Methods, the incoming argument shape forms the candidate Method selector.

Examples:

```phalcom
const exact = &object.render(_)
exact(value)
```

requests exact:

```text
render(_)
```

A pattern Family:

```phalcom
const anyRender = &object.render(...)
```

may accept:

```phalcom
anyRender()
anyRender(value)
anyRender(value, debug: true)
```

when the corresponding exact Method selectors satisfy the retained pattern:

```text
render()
render(_)
render(_,debug)
```

A whole named Family:

```phalcom
const value = &object.value...
```

may include Getter and Setter capabilities as well, but:

```phalcom
value()
```

still requests the nullary Method:

```text
value()
```

It does not fall back to Getter:

```text
value
```

if `value()` is absent.

## Getter activation

Named Getter capability is activated explicitly:

```phalcom
family.get()
```

or through the property-style alias:

```phalcom
family.value
```

These operations are equivalent with respect to the referenced Family capability:

```text
family.get()
family.value
```

both request Getter-kind activation.

For:

```phalcom
const getter = &object.name
```

either form requests exact Getter:

```text
name
```

For an accessor or whole-family capability:

```phalcom
const accessors = &object.name=
const whole = &object.name...
```

the same Getter activation requests the Getter member permitted by the retained selector pattern.

Getter activation is not Method activation.

Thus:

```phalcom
family.get()
```

and:

```phalcom
family()
```

are distinct operations even though both supply no structural argument values.

## Setter activation

Named Setter capability is activated through:

```phalcom
family.set(value)
```

or the property-style alias:

```phalcom
family.value = value
```

These are equivalent with respect to the target Setter capability.

For:

```phalcom
const setter = &object.name=(_)
```

both forms request exact:

```text
name=(_)
```

For:

```phalcom
const accessors = &object.name=
```

the setter forms request the Setter member selected by that accessor pattern.

The assigned value is the Setter's assigned-value position under `SEL-SETTER-VALUE-EXTERNAL`. It is not converted into a unary Method selector:

```text
name(_)
```

and does not participate as a selector structural slot.

A Setter Family may therefore coexist with a unary Method Family under the same base without ambiguity.

### Assignment result

The source expression:

```phalcom
family.value = rhs
```

is assignment syntax.

After successful Setter activation it evaluates to `Unit` under `DSP-ASSIGNMENT-UNIT`, independently of the selected Setter Method's own callable return value.

The explicit protocol call:

```phalcom
family.set(rhs)
```

is an ordinary Method call on the Family value rather than assignment syntax. Its return value follows that protocol Method's contract.

The language must not infer assignment-result semantics merely from the fact that both forms ultimately request the same Setter capability.

## Subscript Getter activation

A Family whose selector specification admits SubscriptGet can be activated with direct bracket syntax:

```phalcom
family[index]
family[index, debug: mode]
```

The bracket shape determines the candidate exact SubscriptGet selector:

```text
[_]
[_,debug]
```

which must satisfy the Family's retained exact selector or selector pattern.

For example:

```phalcom
const exact = &object[_, debug]
exact[key, debug: true]
```

requests:

```text
[_,debug]
```

on the stored receiver.

A pattern:

```phalcom
const getter = &object[...]
```

may route multiple SubscriptGet structural shapes.

## Subscript Setter activation

A Family whose selector specification admits SubscriptSet can be activated with direct bracket assignment:

```phalcom
family[index] = value
family[index, debug: mode] = value
```

For:

```phalcom
const exact = &object[_, debug]=(_)
```

the expression:

```phalcom
exact[key, debug: true] = replacement
```

requests:

```text
[_,debug]=(_)
```

on the stored receiver.

The structural bracket arguments are:

```text
key
debug: true
```

and the assigned value is:

```text
replacement
```

under `SEL-SETTER-VALUE-EXTERNAL`.

After successful activation, the assignment expression evaluates to `Unit` under `DSP-ASSIGNMENT-UNIT`.

## Explicit Tuple-shaped subscript APIs

Subscript Families also expose explicit APIs:

```phalcom
family.get(shape)
family.set(shape, value)
```

where `shape` is a Tuple/product containing the complete structural subscript argument shape.

For example:

```phalcom
family.get((index, debug: mode))
```

corresponds to direct SubscriptGet activation:

```phalcom
family[index, debug: mode]
```

and:

```phalcom
family.set((index, debug: mode), rhs)
```

corresponds in selector routing to:

```phalcom
family[index, debug: mode] = rhs
```

The first explicit API argument preserves both positional and labeled Tuple components and their order.

The Setter RHS is outside the Tuple:

```phalcom
family.set(
  (index, debug: mode),
  rhs
)
```

The shape Tuple contains:

```text
index
debug: mode
```

and does not contain `rhs`.

This follows directly from `SEL-SETTER-VALUE-EXTERNAL`.

An ordinary List is not interchangeable with the shape Tuple merely because it contains the same positional values; labels are part of the selector shape.

## Named and subscript accessor Families

Accessor-pattern references produce Families that admit both getter and setter kinds.

Named:

```phalcom
const access = &object.name=
```

supports:

```phalcom
access.get()
access.value

access.set(newValue)
access.value = newValue
```

Subscript:

```phalcom
const access = &object[...]=
```

supports:

```phalcom
access[index]
access.get((index,))

access[index] = value
access.set((index,), value)
```

where the specific candidate selector must satisfy the retained selector pattern.

Accessor Families do not imply Method capability.

For example:

```phalcom
const access = &object.name=
```

does not make:

```phalcom
access()
```

a request for `name` Getter. Ordinary `()` remains Method-kind and therefore fails the Family's kind predicate.

## Whole named Families

A whole named Family:

```phalcom
const family = &object.name...
```

retains the selector predicate for the complete named base family.

It may therefore support:

```text
Getter
Setter
Method
```

activation through the corresponding Family surfaces.

For example:

```phalcom
family.get()
family.set(value)

family()
family(value)
family(value, debug: true)
```

each requests a different selector kind and/or structural shape under the same base.

There is no generic “best overload” fallback across kinds.

The operation surface chosen by the caller determines the selector kind before lookup begins.

## Failure to satisfy the Family specification

A Family activation can fail before ordinary receiver dispatch if the activation does not satisfy the retained selector specification.

Examples:

```phalcom
const exact = &object.render(_)
exact()                    // shape mismatch

const methods = &object.render(...)
methods.get()              // kind mismatch

const accessor = &object.value=
accessor(1)                // Method kind excluded

const getter = &object[...]
getter[index] = value      // SubscriptSet kind excluded
```

These are capability-shape failures. The Family is not allowed to broaden itself to a different selector pattern merely to make the invocation succeed.

If the activation *does* satisfy the retained exact selector or pattern but ordinary target dispatch finds no current implementation, the failure is a normal target message miss and follows [Dispatch](dispatch.md), including `doesNotUnderstand`.

This distinction matters:

```text
Family predicate rejects operation
    Family capability mismatch

Family predicate accepts exact operation,
but receiver has no current implementation
    ordinary target dispatch miss
```

## Bound Family lookup is live

Under `FAM-BOUND-LOOKUP-LIVE`, bound Family activation consults the target receiver's current behavior.

For exact Family:

```phalcom
const route = &router.route(_)
```

the selector remains:

```text
route(_)
```

but the Method implementing that selector is selected at activation time.

For pattern Family:

```phalcom
const route = &router.route(...)
```

both the candidate exact selector and its current implementation are considered at activation.

Therefore:

- replacing a matching Method may change later activation;
- adding a matching Method may make a previously missing accepted route succeed;
- removing a matching Method may make a previously successful route reach ordinary miss behavior.

The Family's retained receiver and selector specification remain unchanged through such mutations.

## Access authority during Family activation

A Family is not an authority escalation mechanism.

When activation routes to a Method, access is checked according to the ordinary dispatch rules and the authority of the code invoking the Family.

The fact that the Family was constructed in a more privileged context does not, by itself, grant later callers that context's private or protected authority unless another language feature explicitly defines such capture.

Likewise, implementation details of the Family gateway must not substitute their own internal privilege for the caller's access context.

An inaccessible selected Method therefore produces an access violation under `DSP-ACCESS-NOT-MISS`, not a target DNU miss.

## Ordinary sends inside selected Methods remain dynamic

Once a bound Family has selected and activated a Method, `self` is the Family's stored receiver.

Ordinary sends executed inside the selected Method dispatch dynamically on that receiver according to [Dispatch](dispatch.md).

A Family does not freeze the receiver's entire behavior graph merely because it has selected one Method.

Lexical `super` inside the selected Method retains that Method's lexical holder and obeys `DSP-SUPER-RECEIVER`.

## Associated Families

An associated callable reference produces a Family capability whose target context is an associated declaration environment rather than an ordinary bound receiver:

```phalcom
&Option::Some(_)
&Option::Some...
&Parser::parse(...)
&Option::None
```

Associated Families preserve:

- the associated owner;
- any owner specialization or generic environment;
- the exact selector or selector pattern requested by the reference;
- the associated callable targets permitted by that declaration surface.

They do not reinterpret associated lookup as dot dispatch on the owner object.

### Associated exact constructor Families

For:

```phalcom
const some = &Option::Some(_)
```

the Family represents the exact constructor callable shape associated with `Some`.

Activation:

```phalcom
some(42)
```

constructs according to that exact associated callable.

The Family is not a bound receiver lookup for an instance Method named `Some`.

### Associated constructor-family patterns

A whole constructor-family reference:

```phalcom
const some = &Option::Some...
```

retains the associated constructor family under base `Some`.

Invocation shape and type/generic context select the associated callable member according to associated callable semantics.

Unlike a bound pattern Family, an associated Family may be backed by declaration-resolved candidates rather than an open receiver method table. The observable contract is that it represents the associated callable family of the referenced declaration context.

The implementation may specialize or pre-resolve associated targets when semantics prove them without changing Family behavior.

### Associated singleton Getter Families

For a canonical associated singleton:

```phalcom
const none = &Option::None
```

the Family is Getter-shaped.

It may be activated through:

```phalcom
none.get()
none.value
```

to obtain the canonical associated singleton value.

It must not treat:

```phalcom
none()
```

as equivalent. Ordinary call syntax remains Method-kind under `FAM-ACTIVATION-KIND-EXPLICIT`.

This preserves the distinction among:

```phalcom
Option::None           // associated value
&Option::None          // associated Getter Family
(&Option::None).get()  // activation yielding the value
```

### Nullary constructor Families remain Method-shaped

If an associated declaration provides a nullary constructor:

```phalcom
Variants::Nullary()
```

its exact Family is Method-shaped:

```phalcom
&Variants::Nullary()
```

or may be included in:

```phalcom
&Variants::Nullary...
```

That is different from a Getter-shaped singleton Family.

A nullary Method constructor does not become a Getter merely because its structural Method shape is empty.

## Generic specialization and associated Families

An associated Family may carry a generic or applied owner context:

```phalcom
const some = &Option<Int>::Some(_)
```

The specialization constrains associated callable typing and result formation.

It does not alter the structural selector:

```text
Some(_)
```

The same selector may therefore participate under different associated type environments while remaining identical as a selector under `SEL-STRUCTURAL-IDENTITY`.

The type-system specification owns inference, specialization, constructor-local generics, and GADT result refinements. Family semantics require that the associated environment be preserved through later activation.

## Family is a Function capability

Family participates in Phalcom's first-class callable model.

Method-capable Family activation through:

```phalcom
family(...)
```

uses the ordinary Function-call surface of the Family value.

This does not mean every Family operation is expressible through ordinary `()` syntax. Getter, Setter, SubscriptGet, and SubscriptSet are selector kinds in their own right and use the explicit surfaces defined above.

The Function abstraction therefore does not erase selector kind.

A Family can be passed, stored, returned, and composed as a first-class value wherever its type permits.

## Family is not Method

A `Method` is a reified concrete behavior.

A Family is a capability for routing selector operations.

This distinction is visible:

```text
Method
    concrete selected implementation

Family
    receiver/associated context + selector specification
    routes when activated
```

An exact Family may eventually select a concrete Method, but that does not make the Family itself that Method.

Consequently:

- Method reflection can expose the declaring holder and concrete implementation identity;
- Family can remain valid while a matching Method implementation is replaced;
- exact Method invocation does not redo ordinary selector lookup;
- bound Family activation normally does.

## Family is not MethodFamily

`MethodFamily` is a reflective snapshot of concrete Methods matching a selector pattern on a behavior.

Conceptually:

```text
Family
    executable capability
    live bound routing or associated callable capability

MethodFamily
    immutable reflection snapshot
    concrete captured routes
```

A `MethodFamily` may expose reflective operations such as:

```phalcom
snapshot.selectors
snapshot.size
snapshot.methodFor(selector)
snapshot.bind(receiver)
```

The exact reflection API may be specified separately, but its semantic distinction is important here.

Constructing:

```phalcom
&object.render(...)
```

does **not** produce `MethodFamily`.

It produces executable `Family`.

A reflective operation on a Behavior may instead capture a `MethodFamily` snapshot of currently effective matching Methods.

Later mutation can therefore affect the two differently:

```text
Family:
    live activation may observe changed behavior

MethodFamily:
    captured Method set remains the snapshot it represented
```

## BoundMethodFamily

Binding an immutable `MethodFamily` snapshot to a receiver produces a `BoundMethodFamily`.

A `BoundMethodFamily` is callable because it combines:

```text
captured concrete Method snapshot
+
receiver
```

It is still not the same as a normal `Family`.

A normal bound Family resolves through its live selector capability.

A BoundMethodFamily routes only among the concrete Methods captured in its MethodFamily snapshot, subject to receiver compatibility and activation rules.

This distinction prevents reflective snapshots from silently acquiring live-family mutation semantics.

## Family construction has no empty-family failure

A bound pattern reference need not have a matching Method at construction time.

For example:

```phalcom
const futureRoute = &object.route(...)
```

is a valid Family construction independent of whether `object` currently has a matching `route` Method.

Construction retains the receiver and predicate.

If activation later produces an exact candidate accepted by the pattern but no current Method handles it, that activation reaches the ordinary target miss path.

There is therefore no general “empty Family” construction error for bound references.

Associated references remain subject to associated declaration resolution; this rule does not invent associated callable surfaces that do not exist.

## Pattern Families do not perform reflection snapshots

A pattern Family's retained selector predicate is immutable, but it is not a list of captured Methods.

For:

```phalcom
const f = &object.render(_, ..., debug)
```

the retained capability contains the structural predicate conceptually:

```text
base:
    render

kind:
    Method

prefix:
    Positional

suffix:
    Label(debug)

gap:
    present
```

It does not need to enumerate:

```text
render(_,debug)
render(_,_,debug)
render(_,_,_,debug)
...
```

At activation, the actual call shape determines one candidate exact selector, and the pattern tests that candidate.

This is why selector patterns are predicates rather than dispatch keys.

## Failure behavior

Family activation can fail through several distinct mechanisms.

### Capability mismatch

The requested activation kind or shape is not admitted by the retained selector specification.

Examples:

```phalcom
const getter = &object.name
getter()                   // Method activation against Getter capability

const exact = &object.route(_)
exact(1, debug: true)      // shape mismatch
```

This is a Family invocation mismatch.

### Target message miss

The Family admits the operation, but live ordinary dispatch on the stored receiver cannot resolve it.

Example:

```phalcom
const route = &object.route(_)

// no route(_) implementation exists at activation

route(value)
```

The exact operation is routed to the target and reaches the ordinary miss semantics specified in [Dispatch](dispatch.md).

### Access violation

The Family admits the operation and target behavior exists, but the caller cannot access it.

This is an access violation under `DSP-ACCESS-NOT-MISS`, not a target miss.

### Target execution failure

The selected Method runs and raises or otherwise fails.

That failure propagates. Family routing does not reinterpret it as a request to try another selector unless a distinct protocol explicitly defines such behavior.

## Invocation does not search across selector kinds

A Family may have a broad selector predicate, but a single activation has one operation kind.

For example:

```phalcom
const family = &object.value...
```

Given:

```phalcom
family()
```

the Family considers Method selector:

```text
value()
```

It does not attempt:

```text
value
```

if `value()` is absent.

Given:

```phalcom
family.get()
```

the Family considers Getter:

```text
value
```

It does not attempt:

```text
value()
```

if the Getter is absent.

Given:

```phalcom
family.set(x)
```

the Family considers Setter:

```text
value=(_)
```

It does not attempt unary Method:

```text
value(_)
```

This behavior is the central consequence of `FAM-ACTIVATION-KIND-EXPLICIT`.

## Dynamic argument shapes

Method and subscript Family activation may receive dynamically assembled positional and labeled shapes through the normal callable and argument-expansion mechanisms.

Once the complete activation shape is known, the Family derives the same candidate exact selector that an equivalent statically shaped activation would derive.

For example, an exact or pattern Family activated with:

```phalcom
family(***arguments)
```

does not enter a separate selector model merely because the argument shape was assembled dynamically.

The candidate selector must still satisfy the retained Family capability, and target dispatch then follows the ordinary rules in [Dispatch](dispatch.md).

Likewise, a dynamically assembled SubscriptSet preserves the distinction between structural subscript arguments and the assigned value under `SEL-SETTER-VALUE-EXTERNAL`.

## Family equality and identity are not selector equality

Two Families may retain structurally identical selector specifications while referring to different target contexts.

For example:

```phalcom
const left = &leftObject.render(_)
const right = &rightObject.render(_)
```

both retain selector:

```text
render(_)
```

but they are not therefore the same capability.

Selector equality answers whether the operation identity is the same.

Family identity also includes the capability's target context and runtime value identity.

Likewise, two independently constructed Families over the same receiver and selector need not be the same object merely because they are semantically equivalent capabilities.

This specification does not require structural value equality for Family objects unless another general equality specification defines it.

## Invariants defined by this specification

The following coded invariants are defined here because they distinguish Family semantics from reflected Methods and prevent independent activation paths from collapsing selector kinds.

**`FAM-BOUND-LOOKUP-LIVE`**

> A bound Family retains its receiver and selector specification, not a frozen concrete Method; activation observes the receiver's current applicable behavior.

**`FAM-ACTIVATION-KIND-EXPLICIT`**

> Family activation preserves an explicit selector kind; activation must not fall back to a different kind merely because another kind could consume the same number of values.

Rules about selector structure, setter value position, assignment result, access failures, and reference construction remain owned by their respective specifications and are cited rather than renamed here.
