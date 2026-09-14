# 4. Represent booleans as abstract `Bool` + `True`/`False`

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/object-model.md`; `phalcom-core/src/boolean.rs`, `value.rs`

## Context

Booleans are currently a single `Bool` class backed by the VM's `Value::Bool(b)`
variant. A more Smalltalk-faithful model splits this into an abstract `Bool`
superclass with two singleton subclasses, `True` and `False`, each carrying its
own methods (so `and:`/`or:`/`ifTrue:` dispatch by class rather than by an `if`).

Crucially, this does **not** require a new `Value` variant: `Value::Bool(b)` can
select `True` or `False` as its class at runtime from the boolean payload.

## Decision

**Recommendation (pending approval):** adopt the abstract `Bool` + `True`/`False`
model.

- `Bool` is abstract; `True` and `False` are its singleton subclasses.
- The class of `Value::Bool(true)` is `True`; of `Value::Bool(false)` is `False`.
- Boolean control-flow methods are defined per subclass, dispatched by class.

This is classified as a refinement "recommended for a finished language," not a
minimum correctness fix — so it can follow the [ADR-0002](0002-metaclass-tower-parallel-rule.md)
/ [ADR-0003](0003-introduce-behavior-kernel-class.md) kernel work.

## Consequences

- Boolean logic becomes ordinary polymorphic message dispatch, consistent with
  the rest of the object model, and user code can meaningfully reason about
  `True`/`False` as classes.
- No new `Value` variant and no representation change — only class selection and
  method installation.
- Two extra singleton classes to bootstrap and keep invariant-checked.

## Alternatives considered

- **Keep a single `Bool` class.** Simpler, fewer classes, but boolean behavior
  stays special-cased instead of uniform message dispatch.
