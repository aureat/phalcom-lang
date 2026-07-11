# 3. Introduce `Behavior` as a shared kernel class

- Status: Accepted
- Date: 2026-07-11
- Related: [ADR-0002](0002-metaclass-tower-parallel-rule.md); `docs/object-model.md`

## Context

`Class` and `Metaclass` share most of their protocol: they both hold a method
dictionary, a superclass link, and the machinery for method lookup and
instantiation. Today that shared behavior has no common home, which produces
asymmetric metaclass chains and duplicated logic across `Class` and `Metaclass`.

Smalltalk-80 solves this with an abstract `Behavior` class that is the common
superclass of `Class` and `Metaclass` and owns the method-dictionary / lookup
protocol.

## Decision

Introduce `Behavior` as an abstract kernel class:

- `Behavior` owns the shared protocol: method dictionary, superclass link,
  method lookup, and instance creation.
- `Class` and `Metaclass` both inherit from `Behavior`.
- `Behavior` inherits from `Object`.

This unifies the kernel and removes the need for asymmetric special-casing of
`Metaclass` versus `Class`.

**Recommendation:** adopt. It is listed among the minimum correctness fixes in
the object-model spec because it is what makes the parallel tower
([ADR-0002](0002-metaclass-tower-parallel-rule.md)) express cleanly rather than
as a pile of special cases.

## Consequences

- Method-lookup and instantiation logic live in one place instead of being
  duplicated or branched on class-vs-metaclass.
- One additional class in the bootstrap sequence and the kernel hierarchy.
- Slightly deeper superclass chains (`Class` → `Behavior` → `Object`); negligible
  lookup cost, offset by simpler, more uniform code.

## Status note

Open question pending a go/no-go decision. Blocks a clean implementation of
[ADR-0002](0002-metaclass-tower-parallel-rule.md); recommended to accept together.
