# PDR-0033 — Make Option an immediate bounded sum value

- Status: Accepted
- Date: 2026-08-11
- Amends: ADR-0007 (runtime representation and construction surface);
  ADR-0010 (Value representation admits immediate bounded Option state);
  ADR-0044 (bootstrap representation and niche-encoding deferral);
  ADR-0050 (a non-Obj Value may carry one traceable ObjRef through Some)
- Related: `docs/spec/current/values-and-absence.md`;
  `docs/spec/current/object-model.md`;
  `phalcom-core/src/value/mod.rs`;
  `phalcom-core/src/primitive/nil.rs`

## Context

The ratified Option hierarchy and absence semantics remain useful, but heap-backed
`Some` wrappers and a heap `None` singleton make the common absence protocol pay for
object allocation and identity plumbing. The current `Value` enum cannot contain a
recursive `Some(Value)` arm without either an allocation or a redesign of its physical
layout. The implementation therefore needs an allocation-free correctness substrate
while leaving NaN-boxing and final bit layout open.

## Decision

1. `Option` remains an abstract, sealed primitive root. `Some` and `None` are final
   immediate variants with zero instance fields; no surface Option value is an
   `Object::Instance`, and `None` has no heap singleton.
2. `Some(x)` is canonical source syntax and lowers through ordinary class-side
   `Some.call(x)` dispatch. `Some.new(x)` remains a temporary compatibility alias.
3. The generic VM represents `Some` nesting at depths 1 through 7. Each depth is
   distinct, including `None`, `Some(None)`, and deeper forms. An eighth wrapper
   raises `Option nesting limit exceeded (7)` with `flatMap(_)` guidance. No fallback
   heap boxing is allowed.
4. Class lookup, reflection, ordinary `.ph` dispatch, equality, rendering, and
   `Option.match` remain surface operations. Immediate receivers use
   `CallContext::Immediate`.
5. `Value::Nil` remains private uninitialized storage and cannot be wrapped. The
   collector reaches a wrapped heap payload through the single `Value::gc_obj_ref()`
   seam, including VM roots and heap edges.
6. `Some1` through `Some7` plus non-recursive `OptionPayload` are the correctness-first
   landing substrate. Final physical layout, NaN-boxing, pointer tagging, niches, and
   typed `Option<T>` layouts remain deferred.

## Consequences

Option wrapping itself performs zero heap allocation, and `None` no longer needs a
  pinned object handle. Dispatch and reflection retain ordinary class metadata while
  the VM carries the variant state directly.

The cost, named plainly: the generic VM now has a finite Option nesting limit of
seven, and every future `Value` representation must reserve enough state to
distinguish the eight wrapper levels 0–7 without allocating Option wrappers.

This precludes unlimited generic nested Option values, fallback heap representation
for overdeep Option, and treating `Value::Obj` as the only possible `Value` carrying a
GC edge.

## Alternatives rejected

- **Heap-box every `Some`.** Preserves unlimited nesting but violates the zero-wrapper
  allocation invariant and keeps absence coupled to object identity.
- **Recursive `Value::Some(Value)`.** Expresses the semantics directly but is
  recursively sized in Rust and requires a physical-layout redesign outside this
  unit.
- **Implicitly flatten nested `Some`.** Avoids the depth bound by changing meaning;
  it makes `map` and `flatMap` indistinguishable and loses the ratified distinction
  between `None` and `Some(None)`.
- **Finalize NaN-boxing here.** Couples semantic migration to a physical encoding
  decision before GC tracing, reflection, dispatch, and nested behavior are proven.
- **Defer the representation change.** Leaves the allocation and singleton costs in
  every Option-producing path and keeps the old GC root model authoritative.
