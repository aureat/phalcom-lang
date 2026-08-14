# Heap, Alias, and Escape Analysis

## Why locals are easier

A lexical `BindingId` often denotes one known storage cell. Heap fields can be accessed through many aliases:

```text
a = object
b = a
b._x = 2
```

An analysis that tracks `a._x` separately from `b._x` without alias reasoning becomes unsound.

## Points-to abstraction

Map references to abstract allocation sites/objects:

```text
Pts(a) = {AllocSite#12}
Pts(b) = {AllocSite#12}
```

Then field updates operate on abstract locations.

## Allocation-site abstraction

All objects created at one source allocation site share one abstract object. Fast, but loops merge many concrete objects.

Object sensitivity/context can split them at higher cost.

## Strong/weak heap updates

Strong update only when an abstract reference denotes exactly one concrete location under the analysis assumptions. Otherwise weak update joins.

## Escape analysis

Determine whether an allocation/value escapes:

- current expression;
- callable;
- fiber/thread;
- module/global;
- FFI.

Uses:

- stack allocation/scalar replacement;
- closure capture decisions;
- proving no external alias mutation;
- concurrency safety.

## Closures

Captured locals can become heap-like cells with aliases held by blocks. Escaping blocks extend lifetime beyond current frame.

## Reflection

General reflective field access/mutation can force broad havoc. Phalcom's restriction that source field access is receiver-local can significantly simplify static alias semantics if reflection preserves privilege boundaries.

## GC versus analysis

Runtime GC reachability and static points-to analysis are different. However semantic IDs/ObjRef handles and escape information can guide optimizer decisions.

## FFI

Passing object/buffer handles to Rust may create aliases or mutation unknown to Phalcom analysis. Native signatures need ownership/mutation effect contracts before strong reasoning is safe.
