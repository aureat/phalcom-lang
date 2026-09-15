# Arguments and Parameters

> **Status:** Draft normative specification  
> **Semantic ownership:** argument shape, argument evaluation and contribution, positional and labeled lanes, `*` / `**` / `***` expansion, complete argument products, parameter shape, rest capture, shape acceptance, and structural argument binding.  
> **Related specifications:** [Selectors](selectors.md), [Dispatch](dispatch.md), [Functions](functions.md), [Families](families.md).

An invocation in Phalcom does not supply an unstructured list of values. It supplies an **argument shape**: an ordered positional lane together with an ordered labeled lane.

A callable declaration, in turn, has a **parameter shape** describing which argument shapes it accepts and how accepted values bind to its parameters.

These concepts are intentionally separate:

```text
argument shape
    what the caller supplied

parameter shape
    what the callable accepts
```

Selector identity depends on the structural shape of a send, while parameter binding determines whether and how a selected callable accepts the resulting values. Runtime argument values and their types do not participate in selector identity or structural parameter matching.

This specification defines call-shape construction and parameter-shape acceptance. Selector identity belongs to [Selectors](selectors.md); behavioral lookup belongs to [Dispatch](dispatch.md); the common first-class callable protocol belongs to [Functions](functions.md).

## Argument shape

An argument shape contains two ordered logical lanes:

```text
positional lane:
    value₀, value₁, ...

labeled lane:
    label₀: value₀
    label₁: value₁
    ...
```

The labels and labeled values are aligned one-to-one and retain their order.

For:

```phalcom
target(10, 20, to: point, duration: timeout)
```

the argument shape is conceptually:

```text
positionals:
    10
    20

labeled:
    to: point
    duration: timeout
```

The shape exists independently of any particular callee. Constructing the shape does not answer whether the eventual target accepts it.

The corresponding Method selector is structurally:

```text
target(_,_,to,duration)
```

because positional count and ordered labels contribute to selector identity as defined by [Selectors](selectors.md).

### Positional arguments

An ordinary unlabeled argument contributes one value to the positional lane:

```phalcom
target(first, second)
```

produces:

```text
positionals:
    first
    second

labeled:
    <empty>
```

Positional order is significant.

### Labeled arguments

A labeled argument contributes one label and one value to the labeled lane:

```phalcom
target(to: destination, duration: timeout)
```

produces:

```text
positionals:
    <empty>

labeled:
    to: destination
    duration: timeout
```

Label order is significant.

These calls therefore have different shapes:

```phalcom
target(to: destination, duration: timeout)
target(duration: timeout, to: destination)
```

Their ordered label sequences differ, so their selectors differ under `SEL-STRUCTURAL-IDENTITY`.

A label is part of the call's structural identity. The runtime value supplied under the label is not.

## Evaluation order and lane organization

Argument expressions are evaluated under `DSP-EVALUATION-ORDER`: an explicit receiver is evaluated first, then each operand expression exactly once in source order, before behavioral activation.

Lane organization does not change this evaluation order.

For:

```phalcom
target(first(), to: destination(), duration: timeout())
```

evaluation proceeds:

```text
first()
destination()
timeout()
```

The resulting values are then represented in their positional and labeled lanes.

This distinction matters when expansions contribute more than one value. The source expression is evaluated once at its source position; the values it contributes retain the order defined by that expansion form.

## Argument source phases

The source grammar maintains a **positional phase** followed, if needed, by a **labeled phase**.

During the positional phase, a call may contain:

```text
ordinary positional arguments
* expansion
*** expansion
```

An explicit or computed labeled argument, or a `**` expansion, begins the labeled phase.

After the labeled phase has begun, the call may contain labeled contributions, but it may not return to positional source forms. In particular, these are not permitted after the labeled phase begins:

```text
ordinary positional argument
* expansion
*** expansion
```

Thus a shape such as:

```phalcom
target(first, *more, ***forwarded, debug: enabled, **options)
```

is structurally valid with respect to source phases.

A form conceptually like:

```phalcom
target(debug: enabled, later)
```

is invalid because an ordinary positional argument follows the beginning of the labeled phase.

Likewise:

```phalcom
target(debug: enabled, *more)
target(debug: enabled, ***more)
```

are invalid source ordering.

### Complete expansion does not begin the labeled source phase

`***` is unusual because it may contribute both positional and labeled values, but the `***` syntax itself does not switch subsequent source syntax into the labeled phase.

Therefore:

```phalcom
target(***prefix, trailing)
```

is syntactically permitted while the source remains in its positional phase.

If `prefix` contributes labeled entries, those entries still belong to the final labeled lane. The later ordinary positional contribution belongs to the final positional lane.

This is not reordering of expression evaluation. `prefix` is evaluated before `trailing`; only their logical contributions are organized into their respective final lanes.

An explicit labeled argument or `**` contribution still begins the labeled source phase and prevents later positional source forms.

## Expansion markers

Phalcom has exactly three argument expansion markers:

```text
*      positional expansion
**     labeled expansion
***    complete expansion
```

The same markers are used for corresponding rest-parameter forms where those forms are supported.

The selector-pattern ellipsis:

```text
...
```

is unrelated. It is not argument spread syntax.

### Positional expansion with `*`

A positional expansion:

```phalcom
target(*source)
```

contributes zero or more values to the positional lane.

The operand is evaluated once.

A valid `*` source is one of:

- `Unit`, representing zero positional values;
- a `Tuple`, contributing its positional lane;
- another value implementing Phalcom's iteration protocol, whose yielded values are contributed in iteration order.

For an iterable expansion source, expansion uses the ordinary iteration behavior of that value. Errors or control effects produced by a real iterator implementation propagate normally.

A value that is neither `Unit`, a `Tuple`, nor iterable is an invalid positional expansion source.

`*` does not consume or synthesize labels. Applying `*` to a Tuple contributes only that Tuple's positional lane.

### Labeled expansion with `**`

A labeled expansion:

```phalcom
target(**source)
```

contributes zero or more entries to the labeled lane.

The operand is evaluated once.

A valid `**` source is one of:

- `Unit`, contributing no labeled entries;
- a `Tuple`, contributing its labeled lane;
- a `Record`, contributing its fields in their defined order;
- a `Map`, contributing its ordered entries when every key is a `Symbol`.

For a Map source, a non-`Symbol` key makes the expansion invalid.

`**` does not contribute positional values. Applying `**` to a Tuple contributes only its labeled lane.

The order exposed by the source is preserved in the final labeled lane.

### Complete expansion with `***`

A complete expansion:

```phalcom
target(***source)
```

contributes both lanes of a complete argument product.

The source must be:

- `Unit`, representing an empty complete product; or
- a `Tuple`, contributing both its positional and labeled lanes.

`Record`, `Map`, arbitrary `Iterable`, and other values are not complete argument products merely because they can contribute one of the lanes through `*` or `**`.

A complete expansion therefore preserves the Tuple's argument shape rather than converting it to another collection form.

Multiple complete expansions may appear in one positional source phase:

```phalcom
target(***first, ***second)
```

Their positional contributions are appended to the positional lane in source order, and their labeled contributions are appended to the labeled lane in source order, subject to label uniqueness.

## Labels are unique within an argument shape

An argument shape may contain each label at most once.

This applies regardless of how the label was contributed.

The following is invalid:

```phalcom
target(debug: true, debug: false)
```

So is a dynamically conflicting expansion:

```phalcom
target(debug: true, **options)
```

when `options` also contributes `debug`.

The same rule applies across:

```text
explicit label + explicit label
explicit label + ** expansion
** expansion + ** expansion
*** expansion + explicit/*** labeled contribution
```

For a statically evident duplicate, the program is rejected before evaluating the affected call's argument expressions.

For a duplicate that can only be discovered while expanding runtime values, shape construction fails when the duplicate contribution is encountered and target behavioral activation does not begin.

A duplicate label is not resolved by keeping the first value, keeping the last value, or reordering entries.

## Complete argument products

Phalcom uses its product model to make a complete argument shape first-class.

The canonical empty product is:

```phalcom
()
```

which is `Unit`.

A non-empty complete product is a `Tuple`, which may carry:

```text
ordered positional values
ordered labeled entries
```

For example:

```phalcom
const arguments = (10, 20, debug: true)
target(***arguments)
```

forwards:

```text
positionals:
    10
    20

labeled:
    debug: true
```

A complete argument product is not a List of values and is not a Map of labels. Both lanes remain semantically distinct.

**Invariant — `ARG-COMPLETE-SHAPE-PRESERVED`**

> Complete argument transport preserves the positional lane, ordered label identities, and corresponding labeled values without flattening them into one positional sequence.

This invariant applies to `***` forwarding and to other APIs that explicitly transport complete call shapes, including Function `callWith`, exact Method forwarding, and dynamic `perform`-style operations.

### Empty and non-empty products

Where the language captures or materializes an argument product:

```text
no captured entries
    → Unit / ()

one or more captured positional or labeled entries
    → Tuple
```

A rest capture does not become a List merely because it contains multiple positional values.

A labeled rest capture does not become a Map merely because it contains labels.

The resulting Tuple preserves whichever lanes were captured.

## Parameter shape

A parameter shape describes the structural argument shapes accepted by a callable declaration.

For fixed parameters, it contains:

```text
fixed positional count
fixed ordered labels
```

A rest-capable parameter shape additionally describes how residual values may be accepted and captured.

Parameter shape is distinct from:

- the parameter's local binding names;
- parameter type annotations;
- default-value semantics, where separately specified;
- local frame layout;
- selector lookup order.

For an ordinary fixed Method:

```phalcom
move(_ delta, to destination)
```

the parameter bindings are:

```text
positional parameter:
    local binding delta

labeled parameter:
    external label to
    local binding destination
```

The structural selector is:

```text
move(_,to)
```

The local names `delta` and `destination` are not selector components.

### Positional parameter syntax

An explicitly positional Method parameter uses `_` as its external call-site position:

```phalcom
method(_ value)
```

The local binding is `value`.

### Labeled parameter syntax

A labeled Method parameter exposes a call-site label and has a local binding:

```phalcom
method(to destination)
```

The call site uses:

```phalcom
receiver.method(to: value)
```

and the body refers to:

```phalcom
destination
```

The external label `to` participates in selector identity; the local binding name does not.

Where the language permits a shorthand with identical external label and local binding, it is semantically the same parameter shape as writing those identities explicitly.

## Parameter shape forms

You can optionally discard a parameter's local binding by omitting its name, yielding the following 7 total parameter forms:

```md
_ value: T
    external label: _
    local binding: value

_: T
    external label: _
    local binding: [*discarded*]

_ _: T [verbose form]
    external label: _
    local binding: [*discarded*]

from origin: T
    external label: from
    local binding: origin

from: T
    external label: from
    local binding: from

from from: T [verbose form]
    external label: from
    local binding: from

from _: T
    external label: from
    local binding: [*discarded*]
```

## Exact parameter acceptance

Let an argument shape contain:

```text
P = positional count
L = ordered label sequence
```

and let a fixed parameter shape require:

```text
F = fixed positional count
K = ordered fixed-label sequence
```

A non-rest fixed parameter shape accepts the argument shape exactly when:

```text
P == F
L == K
```

Labels are compared by identity and in order.

A call is not accepted merely because it supplies the same number of total values.

For example, a parameter shape requiring:

```text
one positional
label to
```

does not accept:

```text
one positional
label from
```

or:

```text
two positional
no labels
```

even though each case carries two values.

## Structural acceptance does not inspect argument values

Parameter acceptance is structural.

It may inspect:

```text
positional count
ordered labels
rest mode
```

It does not inspect runtime argument values or runtime value types in order to choose whether the shape matches.

Type annotations and generic constraints participate in the type system and callable contract, but they do not turn ordinary selector/rest resolution into runtime value-type dispatch.

**Invariant — `ARG-ACCEPTANCE-STRUCTURAL`**

> Argument-shape acceptance and rest matching depend on structural call shape, not on runtime argument values or runtime value types.

This keeps structural callable selection deterministic and consistent with selector identity.

## Rest parameters on named Methods

Ordinary named Methods may declare residual argument capture with:

```text
*rest      positional residual capture
**rest     labeled residual capture
***rest    complete residual capture
```

A Method may use:

```text
*rest only
**rest only
*rest together with **rest
***rest only
```

The combination of `*rest` and `**rest` is a **split rest** shape: the residual positional and labeled lanes are captured separately.

`***rest` is a **complete rest** shape: both residual lanes are captured together as one complete product.

### Fixed prefixes

A rest-capable Method may still require fixed parameters before the residual capture.

The fixed positional parameters consume a positional prefix.

Fixed labeled parameters consume an ordered labeled prefix.

Residual capture begins only after those fixed portions have been satisfied.

For example:

```phalcom
route(_ first, *middle, to destination, **options)
```

has:

```text
fixed positional prefix:
    first

fixed labeled prefix:
    to

residual positional capture:
    middle

residual labeled capture:
    options
```

The local binding names do not affect matching.

## Positional rest

A Method with positional rest and no labeled rest accepts extra positional values while requiring its labeled lane to match the fixed labeled shape exactly.

Using the notation above, positional rest accepts when:

```text
P >= F
L == K
```

It captures:

```text
positionals[F..P]
```

For:

```phalcom
collect(_ head, *tail)
```

a call equivalent to:

```phalcom
collect(1, 2, 3)
```

binds:

```text
head = 1
tail = (2, 3)
```

A call with no residual values binds:

```text
tail = ()
```

Positional rest by itself does not absorb unexpected labels.

## Labeled rest

A Method with labeled rest and no positional rest accepts residual labels after its fixed labeled prefix, but it does not absorb extra positional values.

It accepts when:

```text
P == F
L starts with K
```

It captures the labeled suffix after `K`.

For:

```phalcom
configure(mode selectedMode, **options)
```

an accepted call may supply additional labels after `mode`, and those residual labeled entries are captured into `options` in order.

No residual labeled entries produce:

```phalcom
()
```

One or more residual labeled entries produce a Tuple carrying that labeled lane.

## Split rest

A Method with both positional and labeled rest accepts residual values in each lane independently.

It accepts when:

```text
P >= F
L starts with K
```

It captures:

```text
residual positionals:
    positionals[F..P]

residual labels:
    labels[K.length..] with their values
```

The two captures remain distinct.

For:

```phalcom
route(_ first, *middle, to destination, **options)
```

a matching call may have additional positionals absorbed by `middle` and additional labels after `to` absorbed by `options`.

Each capture is normalized independently:

```text
empty residual lane
    → ()

non-empty residual lane
    → Tuple with that lane
```

## Complete rest

A Method with complete rest accepts residual values from both lanes as one complete product.

It accepts when:

```text
P >= F
L starts with K
```

It captures:

```text
residual positionals after F
+
residual labeled entries after K
```

as one complete product.

For:

```phalcom
forward(***arguments)
```

the complete call shape is captured unchanged:

```text
no arguments
    → ()

non-empty shape
    → Tuple
```

A declaration may also have fixed prefixes before a terminal complete rest where the declaration grammar permits them. Only the residual portion enters the `***` binding.

## Rest-parameter ordering and validity

Rest declarations obey lane structure rather than arbitrary parameter order.

The following rules apply to ordinary named Method rest parameters:

- there may be at most one positional `*rest` parameter;
- there may be at most one labeled `**rest` parameter;
- `***rest` cannot coexist with `*rest` or `**rest`;
- no parameter may follow `**rest`;
- no parameter may follow `***rest`;
- `*rest` must occur before fixed labeled parameters;
- an ordinary positional parameter may not occur after a labeled parameter or after `*rest`;
- when both `*rest` and `**rest` are present, `*rest` captures the residual positional lane and `**rest` terminates the labeled lane.

Examples of valid rest structures include:

```phalcom
sum(*numbers)

configure(**options)

forward(***arguments)

route(_ first, *middle, to destination, **options)
```

Examples of invalid structures include conceptually:

```phalcom
method(**labels, _ later)
method(***all, later)
method(*first, *second)
method(**first, **second)
method(*positional, ***all)
method(**labels, ***all)
```

Invalid rest declarations are rejected rather than normalized into a different parameter shape.

## Rest-capable Methods and dispatch

Rest parameter shape does not itself define hierarchy lookup order.

Ordinary dispatch first performs full exact-selector lookup and only then considers compatible rest-capable Methods under `DSP-EXACT-BEFORE-REST`.

Once rest lookup considers a candidate, this specification's structural acceptance rules determine whether its parameter shape accepts the incoming argument shape.

A rest Method therefore does not represent an infinite set of exact selectors installed into the method table. It is one callable declaration with a structural acceptance relation.

## Restrictions by callable kind

The existence of `*`, `**`, and `***` parameter semantics does not imply that every callable declaration kind accepts every form.

### Ordinary named Methods

Ordinary named Methods support the positional, labeled, split, and complete rest forms defined above.

### Closures

Closure parameter syntax is intentionally narrower.

A Closure currently supports:

```phalcom
|| { ... }
|value| { ... }
|first, second| { ... }
|head, *tail| { ... }
```

It accepts only positional arguments.

A Closure may have at most one terminal positional rest parameter.

It does not admit:

```text
labeled parameters
**rest
***rest
multiple *rest parameters
fixed parameters after *rest
```

For a Closure without rest:

```text
labeled lane must be empty
P == fixed positional count
```

For a Closure with terminal `*rest`:

```text
labeled lane must be empty
P >= fixed positional count
```

Residual Closure rest uses the same canonical product rule:

```text
zero residual values
    → ()

one or more residual values
    → Tuple
```

Detailed lexical-capture and execution semantics belong to the Closure specification.

### Subscript declarations

Subscript declarations do not currently admit rest parameters.

Forms attempting to declare subscript `*rest`, `**rest`, or `***rest` are invalid.

Subscript invocation may still use dynamic argument expansion at a call site where the resulting exact structural shape is valid; that is different from declaring a rest-capable Subscript Method.

### Variant payloads

Variant payload declarations do not admit rest parameters.

Variant constructors may have their own generic and callable semantics, but `*rest`, `**rest`, and `***rest` are not inferred merely because ordinary named Methods support them.

Other declaration categories may impose their own restrictions and should specify them in their owning documents.

## Setter assigned values are not ordinary argument slots

Setter and SubscriptSet operations carry the dedicated assigned-value position defined by `SEL-SETTER-VALUE-EXTERNAL`.

For:

```phalcom
object.name = rhs
```

the Setter selector:

```text
name=(_)
```

has:

```text
structural argument shape:
    empty

assigned value:
    rhs
```

For:

```phalcom
object[key, debug: mode] = rhs
```

the SubscriptSet selector:

```text
[_,debug]=(_)
```

has:

```text
structural argument shape:
    one positional
    label debug

assigned value:
    rhs
```

The assigned value is not appended to the ordinary positional lane.

This distinction must survive dynamic forwarding and rest/pack transport. A SubscriptSet with a labeled structural slot does not become an impossible structural sequence in which an extra positional argument follows that label.

Family setter APIs are specified in [Families](families.md).

## Shape forwarding

Several language APIs forward an already-constructed argument shape rather than spelling its members individually.

The semantic requirement is always the same: positional and labeled lanes remain distinct under `ARG-COMPLETE-SHAPE-PRESERVED`.

### Function complete forwarding

For a Function:

```phalcom
function.callWith(arguments)
```

forwards the complete product represented by `arguments`.

Its Function-level equivalence to:

```phalcom
function(***arguments)
```

is specified in [Functions](functions.md).

`arguments` must be `Unit` or a `Tuple` complete product.

### Exact Method forwarding

An exact Method API may accept complete forwarding in a form such as:

```phalcom
method.invokeOn(receiver, ***arguments)
```

The forwarded labels remain labels; they are not flattened into a List.

The Method specification owns receiver compatibility, access, and exact invocation.

### Dynamic send forwarding

A reflective send may similarly perform:

```phalcom
receiver.perform(selector, ***arguments)
```

The complete product determines the actual send shape. Reflection semantics belong to the reflection specification, but argument transport remains governed by this document.

## Static and dynamic shape construction

A call whose complete shape is visible statically and a call whose shape is assembled through expansions use the same argument-shape model.

For example:

```phalcom
target(1, debug: true)
```

and:

```phalcom
const args = (1, debug: true)
target(***args)
```

produce equivalent final argument shapes.

The implementation may construct them differently, but once shape construction succeeds:

```text
positional count
ordered labels
argument values
```

have the same semantic meaning.

Dispatch then follows the same rules as specified in [Dispatch](dispatch.md).

## Invalid argument forms

An invocation is invalid when its source argument syntax or resulting shape violates these rules.

Examples include:

```phalcom
target(label: value, positional)
```

because positional source syntax follows the labeled phase;

```phalcom
target(label: value, *more)
```

because positional expansion follows the labeled phase;

```phalcom
target(label: value, ***more)
```

because complete expansion follows the labeled phase;

and:

```phalcom
target(debug: true, debug: false)
```

because one final shape cannot contain duplicate labels.

Expansion also fails when an operand cannot supply the lane requested by its marker:

```text
*source
    source is neither Unit, Tuple, nor iterable

**source
    source is neither Unit, Tuple, Record, nor Map
    or a Map key is not Symbol

***source
    source is neither Unit nor Tuple
```

Such failures occur before target behavioral activation.

## Invariants defined by this specification

The following coded invariants are defined here because they constrain multiple independent forwarding and invocation paths and are useful to other callable specifications.

**`ARG-COMPLETE-SHAPE-PRESERVED`**

> Complete argument transport preserves the positional lane, ordered label identities, and corresponding labeled values without flattening them into one positional sequence.

**`ARG-ACCEPTANCE-STRUCTURAL`**

> Argument-shape acceptance and rest matching depend on structural call shape, not on runtime argument values or runtime value types.

Evaluation order remains owned by `DSP-EVALUATION-ORDER`; selector slot ordering remains owned by `SEL-SLOT-ORDER`; the Setter assigned-value distinction remains owned by `SEL-SETTER-VALUE-EXTERNAL`.
