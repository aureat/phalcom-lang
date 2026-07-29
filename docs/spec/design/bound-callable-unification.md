# `Family` vs `Method.bind` unification (proposed resolution of open-Q14 / functions §3)

- Status: Proposed · resolves the "two bound-callable routes coexist" open question
- Hazard: **least-surprise + stdlib surface** — two mental models for one concept

## Problem

Two routes produce "a callable closed over a receiver": `::` builds an `Open`/
`Pinned` `Family` (selectors §3); `Method.bind`/`invokeOn` operate on a reified
`Method` (functions §3). Every higher-order API must accept both, and users learn
two models for one idea.

## Decision

**Two views of one concept, each convertible — not two types.** Mirror the
Block/Method-under-`Function` move:

- `Family` = the **reference** — lazy `(receiver, selector)`, resolves on call,
  *survives redefinition* (late-bound). Cheap; no `Method` reified.
- `Method.bind(recv)` = the **reified** form — an actual `Method` + pinned
  receiver, *early-bound* to today's definition.
- Bridges: `family.reify → Method`, `method.bind → Family`. Both answer
  `Function`'s `call`/`arity` protocol, so higher-order APIs take a `Function` and
  never branch on which route produced it.

## Precludes

Independent evolution of the two forms — intentional. The late/early-bound
distinction stays *observable* (redefinition semantics differ); collapsing that
away would be wrong.
