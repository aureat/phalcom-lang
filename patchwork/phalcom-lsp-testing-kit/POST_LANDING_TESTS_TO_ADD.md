# Tests to add after the semantic implementation exposes its internal API

The included kit focuses on public LSP behavior because the implementation
agent may choose different internal SemanticDb APIs. Once those APIs are known,
add fast unit tests for these exact semantic laws.

## Value-shape joins

```text
join(Cat, Cat) = Cat
join(Cat, Dog) = Cat | Dog
join(Cat | Dog, Cat) = Cat | Dog
```

Test the exact widening boundary chosen by the implementation spec.

## Flow

- alias propagation;
- lexical shadowing;
- reassignment at different program points;
- branch joins;
- loop element projection;
- destructuring projection.

## Constructors

Positive:
```phalcom
const x = User.new()
```
when the selected `new` is semantically a constructor.

Negative:
```phalcom
const x = Parser.parse(source)
```
must not infer `Parser` merely because the call receiver is `Parser`.

## Call summaries

- one concrete return;
- multiple returns -> union;
- forwarding chains;
- direct recursion terminates;
- mutual recursion terminates;
- recursive path with concrete base result converges.

## Parameter inference

- one call site;
- multiple call sites -> union;
- stale call-site facts removed on edit;
- unrelated dynamically unresolved calls do not contaminate the summary.

## Fields

- constructor assignment;
- assignment in another method if the design supports class-wide field facts;
- multiple assignments -> union or specified flow result;
- inherited field visibility.

## Unknown / confidence

- `Unknown` is absence of knowledge, not `Any`;
- heuristic facts do not become stable hints by default;
- provenance survives joins if the API models provenance.

## Invalidation

At the SemanticDb layer, assert exact dependency invalidation so a leaf edit
does not force a whole-workspace semantic rebuild.
