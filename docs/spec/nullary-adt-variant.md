# Nullary Variant Constructors

A nullary variant constructor is a variant constructor whose call shape accepts no arguments and whose constructed values contain no variant payload fields.

```phalcom
enum Variants {
    @variant Nullary()
}
```

The declaration above introduces the variant constructor:

```phalcom
Variants::Nullary
```

with a conceptual constructor signature equivalent to:

```text
() -> Variants::Nullary
```

`Variants::Nullary` refers to the constructor identity. It is not a value of the variant.

A value is produced by invoking the constructor:

```phalcom
const nullary = Variants::Nullary()
```

Each invocation constructs a new value.

```phalcom
const a = Variants::Nullary()
const b = Variants::Nullary()
```

`a` and `b` are distinct constructed values of the same variant. A nullary variant is therefore not implicitly a singleton variant.

The term *nullary* describes constructor arity only:

```text
arity(Variants::Nullary) = 0
```

It does not imply that the variant has only one possible runtime instance.

## Payload

A nullary variant contains no variant payload fields.

```phalcom
enum Signal {
    @variant Wake()
    @variant Cancel()
}

channel.send(Signal::Wake())
channel.send(Signal::Wake())
```

`Signal::Wake()` carries no data corresponding to constructor arguments.

This does not prevent an instance from having identity or from participating in ordinary object semantics provided by the language.

A nullary variant may therefore be useful when distinct occurrences of the same nominal variant are required even though no intrinsic payload is necessary.

Examples include events, messages, tokens, graph nodes, compiler nodes, effects, protocol requests, and other identity-bearing values.

```phalcom
enum Event {
    @variant Tick()
    @variant Data(bytes: Bytes)
}

const first = Event::Tick()
const second = Event::Tick()
```

`first` and `second` represent two distinct `Tick` occurrences.

## Constructor Semantics

Nullary constructors participate in the same constructor model as variants of higher arity.

Conceptually:

```text
None         : Option::None
Nullary       : () -> Variants::Nullary()
Some         : (T) -> Option::Some<T>
Pair         : (A, B) -> PairEnum::Pair<A, B>
```

Zero-argument construction is not a special case in the variant system. It is the arity-zero member of the general variant-constructor model.

Consequently, nullary constructors participate normally in:

- constructor families,
- callable-family modeling,
- generic substitution,
- overload and call-shape analysis,
- reflection,
- matching,
- lowering,
- GADT constructor signatures.

## Matching

Nullary variants are matched using their zero-argument variant pattern.

```phalcom
match event {
    Event::Tick() => handleTick()
    Event::Data(bytes) => handleData(bytes)
}
```

The empty argument list indicates that the variant has no payload to destructure.

Matching a nullary variant tests variant identity in the same manner that matching a payload-carrying variant tests its constructor identity before binding its payload.

## Generic and GADT Use

A nullary constructor may convey type information even when it carries no runtime payload.

```phalcom
enum TypeRep<T> {
    @variant Int() -> TypeRep<Int>
    @variant String() -> TypeRep<String>
}
```

Matching such a constructor may refine generic parameters according to its declared result type.

```phalcom
match rep {
    TypeRep::Int() => {
        // T is refined to Int
    }

    TypeRep::String() => {
        // T is refined to String
    }
}
```

The absence of payload therefore does not imply the absence of semantic or type-level information.

## Runtime Representation

The language does not require a particular physical representation for nullary variant values.

An implementation may use compact tags, unboxed representations, allocation elimination, scalar replacement, or other optimizations where these preserve observable semantics.

However, the implementation must not canonicalize separately constructed nullary values into a single observable object when doing so would change object-identity behavior.

For example:

```phalcom
const a = Variants::Nullary()
const b = Variants::Nullary()
```

must retain the semantics of two constructor invocations.

Any optimization that removes or merges their physical allocations is valid only when the distinction cannot be observed under Phalcom's identity semantics.

## Nullary Variants and Singleton Variants

Nullary variants and singleton variants are distinct concepts.

A nullary variant constructor means:

```text
constructor arity = 0
payload field count = 0
number of constructible instances = unbounded
```

A singleton variant, if provided by the language, instead means that exactly one canonical value exists for that variant.

Therefore:

```phalcom
@variant Nullary()
```

must not implicitly acquire singleton semantics merely because it has no payload.

The intended distinction is:

```text
nullary constructor
    zero constructor arguments
    produces a fresh value when invoked

singleton variant
    one canonical value
    does not represent repeated fresh construction
```

Use a nullary constructor when independently constructed occurrences of a variant are meaningful despite requiring no payload. Use a singleton variant when the variant itself denotes one canonical value.

## Examples

```phalcom
enum Expr {
    @variant Missing()
    @variant Literal(value: Object)
    @variant Call(target: Expr, args: List<Expr>)
}

const left = Expr::Missing()
const right = Expr::Missing()

sourceMap[left] = leftSpan
sourceMap[right] = rightSpan
```

```phalcom
enum Node {
    @variant Placeholder()
    @variant Named(name: String)
}

const x = Node::Placeholder()
const y = Node::Placeholder()

graph.connect(x, y)
```

```phalcom
enum Request {
    @variant Ping()
    @variant Query(sql: String)
}

const request = Request::Ping()

timestamps[request] = Clock.now()
traceIds[request] = Trace.next()
```

```phalcom
enum Effect {
    @variant Yield()
    @variant Read(path: String)
}

perform Effect::Yield()
perform Effect::Yield()
```

```phalcom
enum WorkerMessage {
    @variant Poll()
    @variant Stop()
    @variant Process(job: Job)
}

worker.send(WorkerMessage::Poll())
```