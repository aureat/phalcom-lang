# `Array<T>`

> **Status:** Proposed COLL006 target contract; not yet normative or
> implemented on `main`. This page records the intended type boundary for
> documentation and planning. See the [COLL006 status](../../implementation/COLL006/STATUS.md);
> acceptance of COLL006 must ratify or revise this contract.

## Type contract

`Array<T>` is the homogeneous fixed-extent mutable sequence target in the
COLL006 collection design. Its extent is determined at construction and is
runtime data; once constructed, the number of elements cannot change. An
ordinary runtime Array permits element replacement, but has no append, remove,
or resize operation. The type argument `T` describes its element type.

Construction initializes exactly the supplied elements in their input order
and fails if it cannot produce a valid Array. `size` is constant for the
lifetime of the value. Indexed access
is bounds checked; assignment changes only the selected element. Iteration is
in increasing index order and conforms to the canonical `Iterable<T>` laws.

Equality is structural and order-sensitive. Since Array is mutable, mutation
may change its equality result; it cannot be used as a Map key or Set member
under the shared collection rules.

## Static values and syntax boundary

An Array may be formed as a static value where the static-value rules allow it.
This type specification does not authorize a mutable runtime alias to a frozen
static value. Static evaluation and materialization remain governed by
[`../static-evaluation.md`](../static-evaluation.md).

This page does not define bracket-literal selection or a constructor spelling.
The active `[...]` literal remains List construction under
[`../collection-literals.md`](../collection-literals.md) until a superseding
language specification changes that rule. The `Array<T>` type shape follows
the current COLL006 direction; it is not a claim that its runtime or compiler
implementation exists on `main`.

The source specification deliberately leaves physical contiguous layout,
allocation strategy, and native primitives to VM-level specifications.
