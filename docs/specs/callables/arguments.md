# Arguments and Parameters

> **Status:** Draft normative specification  
> **Semantic ownership:** argument shape, argument evaluation and contribution, positional and labeled lanes, `*` / `**` / `***` expansion, complete argument products, parameter shape, rest capture, shape acceptance, and structural argument binding.  
> **Related specifications:** [Selectors](selectors.md), [Dispatch](dispatch.md), [Callable Values and Function](callable-values.md), [Families](families.md).

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

This specification defines call-shape construction and parameter-shape acceptance. Selector identity belongs to [Selectors](selectors.md); behavioral lookup belongs to [Dispatch](dispatch.md); the common first-class callable protocol belongs to [Callable Values and Function](callable-values.md).

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

Telos has exactly three argument expansion markers:

```text
*      positional Pack expansion
**     labeled Pack expansion
***    complete Pack expansion
```

The same markers are used for corresponding rest-parameter forms. The selector-pattern
ellipsis `...` is unrelated.

Argument expansion is a native Pack operation. Record, Map, Tuple, Unit, and arbitrary
Iterable values are not implicitly converted into Pack merely because they could be
enumerated into similar values. Explicit conversion APIs may be defined separately.

### Positional expansion with `*`

```telos
target(*source)
```

`source` must be a positional Pack-compatible value. It contributes its positional
entries in Pack order.

The operand is evaluated once.

### Labeled expansion with `**`

```telos
target(**source)
```

`source` must be a labeled Pack-compatible value. It contributes its ordered
Symbol-labeled entries in Pack order.

The operand is evaluated once.

Duplicate labels are invalid.

Record is an unordered product and is not an implicit `**` source.

### Complete expansion with `***`

```telos
target(***source)
```

`source` must be a complete Pack-compatible value. Both Pack lanes are contributed
without flattening one lane into the other.

Multiple complete expansions preserve contribution order within each lane.

### Fused expansion

Expansion does not semantically require repeated immutable Pack concatenation.

For:

```telos
target(*a, *b, *c)
```

an implementation may form one argument-construction plan from the three Pack segments
and fill the final argument window once.

The required complexity is O(total contributed entries), subject to ordinary expression
evaluation cost.

An ordinary fixed-shape call does not allocate a Pack solely because its argument shape
is semantically Pack-shaped.

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

The first-class semantic representation of an argument product is Pack.

```telos
const arguments =
    Pack(10, 20, debug: true)

target(***arguments)
```

A complete argument Pack preserves two independent lanes:

```text
positionals:
    10
    20

labeled:
    debug: true
```

A complete argument product is not Unit, Tuple, List, Record, or Map. `Pack()` is the
empty complete argument product.

**Invariant — `ARG-COMPLETE-SHAPE-PRESERVED`**

> Complete argument transport preserves the positional lane, ordered Symbol labels,
> and corresponding labeled values without flattening or reordering them.

The same invariant applies to rest capture, forwarding, dynamic invocation, exact
Method forwarding, and Function call-shape transport.

### Materialization freedom

A semantic argument Pack does not imply a materialized runtime Pack object.

A fixed call may use a stack/register argument window directly. A Pack that is
immediately spread may remain compiler-virtual. A captured or escaping Pack may be
materialized.

These choices are unobservable.

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

Rest parameters capture the unmatched suffix of an accepted argument shape as Pack.

```text
*rest   -> positional Pack
**rest  -> labeled Pack
***rest -> complete Pack
```

The empty capture is `Pack()`, specialized to the corresponding lane shape. Rest
capture does not produce Tuple, Record, Map, List, or `None`.

A rest binding is an ordinary immutable binding unless the declaration syntax
explicitly makes another binding mutable.

### Discarding rest parameters with `_`

```telos
consume(*_)
consume(**_)
consume(***_)
```

A discarded rest parameter accepts the corresponding remainder without introducing a
source binding. The implementation need not materialize a Pack when the remainder is
unobserved.

### Fixed prefixes

Fixed parameters bind before their corresponding rest capture. The captured Pack
contains only unmatched entries and preserves original order within each lane.

## Positional rest

```telos
method(first, *rest) {
    ...
}
```

`rest` is a positional Pack.

Where the runtime calling convention permits it, rest may be represented as an O(1)
slice/view over the incoming argument window. Capturing rest does not require copying
the suffix merely to produce Pack semantics.

## Labeled rest

```telos
method(debug: flag, **rest) {
    ...
}
```

`rest` is a labeled Pack containing unmatched ordered Symbol-labeled entries.

Duplicate labels are already invalid at argument-shape construction, so a successfully
bound labeled rest is duplicate-free.

## Split rest

A declaration may capture positional and labeled remainders separately when the
ordinary parameter grammar permits both:

```telos
method(*positionals, **labeled) {
    ...
}
```

The two captures are independent Pack lanes.

## Complete rest

```telos
method(***rest) {
    ...
}
```

`rest` is one complete two-lane Pack.

A complete rest capture preserves lane identity and ordering. It is not implemented
semantically as “positional tuple plus labeled record”.

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

is specified in [Callable Values and Function](callable-values.md).

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

Argument-shape construction has one Pack semantics and multiple permitted physical
strategies.

```text
known fixed shape
    -> direct argument window
    -> no Pack allocation required

expanded but non-escaping shape
    -> virtual Pack / segmented argument plan

captured or escaping shape
    -> materialized immutable Pack
```

Dynamic construction may use a private mutable builder, but that builder is not the
language Pack value.

When labels are assembled dynamically, duplicate detection must remain efficient.
Implementations may use linear checks for small shapes and a transient index for larger
shapes; the final Pack itself need not carry a permanent duplicate-detection table.

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

- **ARG-TWO-LANES** — every argument shape has independent ordered positional and
  labeled lanes.
- **ARG-LABEL-SYMBOL** — ordered call labels are Symbols.
- **ARG-LABEL-UNIQUE** — a final argument shape contains no duplicate labels.
- **ARG-PACK** — first-class complete, positional, and labeled argument products are
  Pack values.
- **ARG-NO-IMPLICIT-RECORD-PACK** — unordered Record is not implicitly converted into
  labeled Pack for invocation.
- **ARG-REST-PACK** — `*rest`, `**rest`, and `***rest` capture Pack.
- **ARG-REST-SLICE** — semantics permit O(1) rest slices/views.
- **ARG-FUSED-SPREAD** — multiple spreads need not construct intermediate Packs.
- **ARG-FIXED-FASTPATH** — fixed-shape calls do not require Pack allocation.
- **ARG-EVALUATE-ONCE** — each argument/expansion source expression is evaluated
  exactly once in source evaluation order.
- **ARG-SHAPE-BEFORE-BEHAVIOR** — duplicate/shape construction failure occurs before
  target behavioral activation.

