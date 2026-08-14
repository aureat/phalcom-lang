# Abstract Interpretation Foundations

## Concrete and abstract domains

Concrete semantics manipulates sets of possible runtime states `C`.
Static analysis uses a tractable abstract domain `A`.

Conceptually:

```text
α : C -> A      abstraction
γ : A -> C      concretization
```

A Galois connection/order relation can formalize when abstraction is sound:

```text
α(c) ⊑ a  iff  c ⊆ γ(a)
```

You rarely need to implement `α`/`γ` functions explicitly, but thinking in these terms exposes unsound shortcuts.

## Sound abstract transfer

If concrete transfer is `F` and abstract transfer is `F#`, soundness requires the abstract result to cover every concrete result represented by the input.

Intuitively:

```text
F(γ(a)) ⊆ γ(F#(a))
```

For may-analysis, losing precision is acceptable; excluding a real behavior is unsound.

## Best correct approximation

The ideal abstract transformer is the most precise sound abstraction of concrete execution. Implementation may use a coarser transformer for performance, but know what precision was sacrificed.

## Collecting semantics

The collecting semantics at a program point is the set of all concrete states that can reach it. Static analysis approximates this set.

This clarifies why:

- branch merge uses union-like join;
- unreachable has empty collecting set;
- loop analysis seeks a fixed point of repeated transitions.

## Forward versus backward abstraction

Forward analysis approximates states reachable from inputs.
Backward analysis reasons about states required to reach an outcome/property.

Static proving often uses backward weakest-precondition reasoning; classic dataflow often uses forward facts.

## Relational versus non-relational domains

Non-relational:

```text
x in [0,10]
y in [0,10]
```

Relational:

```text
x < y
x - y <= -1
```

Relational domains prove more but cost more.

## Trace partitioning

Path sensitivity can preserve selected branch distinctions instead of joining immediately. Abstract interpretation frames this as partitioning traces/states. Use bounded partitions to prevent exponential explosion.

## Abstract garbage collection

In heap/state analyses, periodically remove abstract addresses no longer reachable from roots to recover precision and bound state growth. Relevant only for sufficiently detailed heap analyses.

## Phalcom use

The current `ValueShape` domain is an advisory abstract domain over possible runtime shapes with bounded unions. Future correctness analyses need their own domains and soundness contracts rather than assuming this domain is a full type/proof abstraction.
