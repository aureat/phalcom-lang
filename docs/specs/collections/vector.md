# `Vector<T,N>`

> **Status:** Proposed COLL006 target contract; not yet normative or
> implemented on `main`. This page records the intended type boundary for
> documentation and planning. See the [COLL006 status](../../implementation/COLL006/STATUS.md);
> acceptance of COLL006 must ratify or revise this contract.

## Type contract

`Vector<T,N>` is the homogeneous immutable sequence target in the COLL006
collection design. `T` is the element type and `N` is a static `Int` extent.
Every value of the exact applied type contains exactly `N` elements, including
the valid zero-length case. Construction with a different cardinality is
invalid. `size` is exactly `N`.

Elements preserve input order. Indexed access is bounds checked and does not
permit in-place mutation. Operations that conceptually add, remove, or replace
elements return a new Vector or another explicitly specified collection type;
they never mutate an existing Vector. Iteration is in increasing index order
and conforms to the canonical `Iterable<T>` laws.

Equality is structural and order-sensitive. A Vector is immutable and may be a
value-hashable key only when its elements satisfy the corresponding equality
and hash laws. Its hash must agree with equality and must include the exact
applied element type and ordered contents.

## Static extent and syntax boundary

The extent is a kinded/static argument, not a runtime element and not a proper
type. `Vector<T,3>` and `Vector<T,4>` are distinct applied types. A runtime
Vector retains the same exact static extent in its type. Static evaluation and
materialization follow [`../static-evaluation.md`](../static-evaluation.md).

This page does not define bracket-literal selection or constructor spelling.
The active `[...]` literal remains List construction under
[`../collection-literals.md`](../collection-literals.md) until a superseding
language specification changes that rule. `Vector<T,N>` records the current
COLL006 design direction and does not claim that runtime/compiler support has
shipped on `main`.

The source contract does not prescribe physical contiguous layout, structural
sharing, or copy-on-write strategy.
