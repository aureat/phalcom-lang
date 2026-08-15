# Expression Inference and Dispatch Development

## Goal

Expression analysis should derive an `InferredValue` (or future typed/proof fact) from
explicit semantic context without executing the VM.

## Context design

Prefer passing an explicit context containing only semantic dependencies:

```text
current class
current dispatch side
query/program point
binding values/local facts
scope graph
known class/module surface
callable return summaries
field facts
dispatch resolver
```

Avoid direct access to LSP backend state or mutable project globals from expression analysis.

## Exact syntax facts

Use exact facts for constructs whose runtime shape is guaranteed by language semantics:

- numeric/string/symbol/bool/none/unit literals as specified;
- class declaration reference -> class object identity;
- known module binding -> module value;
- constructor result where constructor semantics guarantee class instance;
- literal tuple/list/record/map/set/range structure, joined conservatively.

Attach source provenance.

## Identifiers

Resolve name first:

```text
Binding(id) -> flow/local fact at program point
Class(id) -> ClassObject(id)
Module(id) -> Module(id)
ImplicitSelf -> handled in send/member context
Global -> known core/global model or Unknown
Unresolved -> Unknown with recovery provenance if represented
```

Do not infer by searching same-spelled declarations.

## Assignments

Separate:

- evaluating RHS;
- resolving assignment target identity;
- mutating abstract flow state;
- recording occurrence role/write fact;
- expression result semantics.

A member/field assignment is not a lexical binding assignment.

## Collection literals

Preserve structure only while useful and bounded.

Examples:

```text
[1, 2, 3] -> List<Number-like shape>
[Point(), Circle()] -> List<Point | Circle>
(a, b) -> Tuple(shape(a), shape(b))
#{name: x} -> Record("name" -> shape(x))
map literal -> Map(join keys, join values)
```

Dynamic spreads/rest entries require joining source element knowledge or widening to unknown
when boundedness/contents are not statically knowable.

## Operators

If operators are messages in Phalcom semantics, infer through selector/dispatch rather than
hardcoding a second operator type table, except for exact language/compiler-special forms
whose semantics are normatively guaranteed.

Compiler optimizations of `+` do not by themselves change semantic dispatch.

## Member sends

Pipeline:

```text
analyze receiver
build canonical selector from syntax/static pack labels
classify dispatch receiver (instance/class/super/etc.)
resolve via shared DispatchResolver
read callable/member summary
produce return fact
```

If selector cannot be known due to computed labels/expansion, return conservative dynamic
call semantics and mark dynamic effects/dependencies.

## `self`

`self` should synthesize receiver knowledge from current class + dispatch side/context, not
from a global name lookup.

## `super`

Model super send as:

```text
same runtime receiver
lookup starts above current defining class
```

Do not synthesize `Instance(superclass)` as the receiver value.

## Class-side sends

`ClassObject(C)` selects class-side behavior. Constructor/factory return may be `Instance(C)`
when guaranteed, but callable lookup remains class-side.

## Fields

Field read should query `FieldId(owner, name, side)` through field facts/surface.

When field storage is inherited or declared owner differs from current receiver class, define
whether identity keys by declaring class or storage owner according to runtime model.
Do not guess.

## Callables and methods

A concrete callable value can retain `CallableId`. A method family/open dispatch object needs
its own shape because future call selector is not yet fixed.

Do not conflate:

```text
Callable(MethodId)
Family(receiver, base selector)
Method reflection object
BoundMethod/BoundMethodFamily
```

unless current specs define equivalence.

## Unions

When receiver shape is a union, choose policy per semantic query:

- advisory return inference: resolve each alternative and join known results, widen on unknown
  as domain specifies;
- safe-member query: member available on all alternatives;
- candidate completion: may include some-alternative members with lower confidence;
- checker: use normative type/union rules, not LSP union cap.

Keep one resolver API capable of expressing partial/candidate results rather than silently
selecting first match.

## Native/core operations

If native method source surface exists, prefer it. If native semantics require special return
shape, centralize that mapping in core/native semantic metadata (`core_source` or successor),
not in hover/completion.

## Error recovery

Incomplete sends may have:

- missing selector part;
- half-written label;
- missing receiver;
- malformed pack;
- unresolved class/member.

Return partial/unknown semantic facts without panicking. Completion often needs exactly these
states.

## Tests

For every new expression form test:

- exact fact;
- nested expression;
- binding propagation;
- class/module identity;
- dynamic/unknown operand;
- union operand;
- source range/provenance;
- malformed/recovered AST;
- incremental update changing operand class.
