# Dispatch, Members, Call Sites, and Callable Summaries

## Selector identity is load-bearing

Phalcom ordinary dispatch is selector-oriented. Preserve canonical selector formation
through every semantic layer.

Do not use:

- base name only;
- display spelling guessed by the LSP;
- parameter type annotations;
- source ranges;
- arbitrary method declaration indices.

Use the same selector rules as the compiler/runtime/spec.

## Dispatch side

Instance-side and class-side behavior are distinct semantic spaces.

Receiver categories should make this explicit, for example conceptually:

```text
Instance(ClassId)
ClassObject(ClassId)
Module(ModuleId)
Unknown/Dynamic
Union(...)
```

A class-side constructor or factory is not an instance method simply because it returns
an instance.

## Inheritance lookup

Resolution should return the declaration actually selected, not merely the class named by
the receiver fact.

A resolved dispatch commonly needs:

- receiver abstraction;
- requested selector;
- member surface;
- actual declaring owner;
- dispatch side;
- visibility/access result;
- whether resolution is exact or conservative.

Keep lookup semantics synchronized with runtime MRO/super semantics.

## `super`

`super` is not a normal runtime object value. It modifies lookup origin while preserving
the current receiver. Model it as a distinct dispatch mode/receiver context rather than
as an instance of the superclass.

This matters for:

- completion;
- goto-definition;
- type/return inference;
- protected/private access;
- optimizer assumptions.

## Implicit-self sends

An unresolved bare name may participate in implicit-self semantics depending on Phalcom's
syntax/spec. Name resolution and dispatch must coordinate:

```text
lexical binding wins when valid
otherwise implicit-self candidate if syntax permits
otherwise global/unresolved
```

Do not classify all unknown names as implicit-self just to improve completion.

## Families and open callables

Phalcom's `Family` notion represents future receiver-side dispatch. A family can retain:

- receiver knowledge;
- base selector name/pattern;
- call-time label construction.

It is not the same as a captured concrete `Method`/`MethodFamily`. Semantic facts must
respect that conceptual split.

Dynamic/computed argument labels or pack expansion may prevent constructing one exact
selector statically. Mark the call dynamic/conservative rather than fabricating a selector.

## Call-site argument mapping

A call analyzer should preserve:

- source evaluation order;
- static labels when known;
- positional slots;
- pack/dynamic expansions;
- exact argument ranges;
- binding identity when argument is a reference;
- block/closure effects when literal callable arguments are passed.

This enables parameter inference and future type checking without re-walking syntax with
a different interpretation.

## Callable summaries

A callable summary is a compact semantic contract for analysis, not runtime reflection by
default.

Current fields already include:

- callable identity;
- parameter inferred values;
- inferred return value;
- callable dependencies;
- conservative effects;
- semantic generation/revision.

Future additions may include:

- declared parameter/result types;
- inferred checker types;
- throws/non-returning behavior;
- field/global mutation set;
- yield/block effects;
- closure escape/capture summaries;
- proof postconditions;
- generic constraints/substitution templates.

Do not add fields until the semantics and invalidation rules are clear.

## Return inference

Return analysis must combine only reachable return/tail paths according to language
semantics. Constructors may have special return conventions; model those explicitly.

Future typed return inference must distinguish:

- runtime-shape summary;
- language inferred type;
- declared result type;
- recursive inference guard;
- `Unit`/self-return convention as normatively specified.

## Higher-order callables

Current semantic effects can record that callable parameter position `N` is invoked.
This is powerful. It allows a caller passing a literal block to propagate its effects only
when the callee is known to invoke that parameter.

Future extensions can model:

- invoked once/maybe/many times if needed;
- escaping/stored callback;
- synchronous versus deferred invocation;
- non-local returns from block bodies;
- fiber transfer/yield behavior.

Do not assume a block executes merely because it is constructed.

## Recursive summaries

Recursive/mutually recursive callables require a fixed point. Safe approaches:

- initialize to conservative unknown/bottom as appropriate;
- iterate SCC until summary equality;
- widen if the domain can grow;
- require explicit type annotations for checker return inference if the typing spec chooses
  that restriction, even while advisory shape analysis continues conservatively.

Typing and advisory LSP inference may intentionally use different recursion policies.

## Dynamic reflection

A call involving reflective method lookup, dynamic selector construction, `doesNotUnderstand`
semantics, runtime class mutation, or unbounded metaprogramming may invalidate assumptions
about a closed call graph.

Summary effect such as `dynamic_send` should be conservative and contagious where needed.
Never pretend a dependency set is closed when semantics allow unknown targets.
