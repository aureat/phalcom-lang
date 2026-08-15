# Static and Dynamic Semantics Correspondence

Static systems are useful only when their claims mean something about executions. Phalcom's future checker, typed runner, semantic analyzer, and prover must each state exactly which dynamic property a static fact guarantees.

## 1. Correspondence shape

A static rule:

```text
Γ ⊢ e : T
```

needs a dynamic interpretation such as:

```text
if e evaluates normally to v, then v ∈ ⟦T⟧
```

or a stronger call-safety property. The interpretation relation must be defined; the syntax `T` does not explain itself.

## 2. Runtime class versus static type

Separate:

```text
classOf(v) = C
C is represented by runtime Class object
T is language-level Type expression/descriptor
v ∈ ⟦T⟧ is semantic interpretation
```

A class type may denote exact instances, instances plus subclasses, or another set according to Phalcom type semantics. Do not infer relation from inheritance alone.

## 3. Runtime shape versus language type

Current LSP `ValueShape` is advisory runtime knowledge and explicitly not future language type.

A bridge may derive constraints:

```text
ValueShape fact -> evidence about possible Type inhabitants
Type fact       -> possible runtime shape approximation
```

but preserve provenance and precision loss.

Example:

```text
ValueShape::Union[IntClass, StringClass]
```

can help completion. It is not automatically identical to language type `Int | String`, because type semantics may include subclass closure, special types, generics, protocols, dynamic boundaries, and normalization.

## 4. Name-resolution correspondence

Static occurrence resolution:

```text
Γ ⊢ x ↦ BindingId b
```

should correspond to compiler runtime access to storage associated with `b`. If compiler and LSP disagree on shadowing, rename/hover and execution diverge.

## 5. Dispatch correspondence

A checker accepting a send should establish a statement like:

> For every runtime receiver admitted by static assumptions, ordinary selector lookup under relevant access context will select a compatible callable or encounter an explicitly modeled dynamic boundary/check.

Must account for:

- inheritance/metaclass side;
- selector identity;
- visibility;
- reflective/open-world mutation;
- unions/dynamic values;
- generic substitution affecting compatibility but not selector key.

## 6. Static target versus runtime target

A static analyzer can resolve:

```text
exact target
finite candidate set
open candidate set
unknown/dynamic
```

Only an exact target under stable dispatch assumptions justifies devirtualization without guards.

## 7. Gradual/dynamic boundaries

If `Dynamic` or absent annotations permit unchecked behavior:

```text
checked region + no Dynamic escape -> stronger guarantee
Dynamic boundary -> runtime uncertainty reintroduced
```

Do not turn epistemic unknown into `Any` silently and claim safety.

## 8. Runtime contract correspondence

Typed-runner checks can turn static expectations into explicit dynamic outcomes:

```text
checked code computes according to contract
or fails at explicit dynamic contract boundary
```

Exact blame/error semantics belongs in typing design.

## 9. Native boundary

If native primitive claims `String` return but implementation can return unrelated object, static soundness depends on trusted native conformance or runtime/generated validation.

Native metadata is part of trusted computing base until checked mechanically.

## 10. Reflection boundary

Method-table mutation can invalidate proven dispatch closure. Sound strategies include:

- prohibit mutation in sound checked mode;
- invalidate/recheck dependents;
- use version guards/deopt;
- weaken guarantee to dynamic.

Ignoring mutation is not a strategy.

## 11. Concurrency boundary

A static refinement such as:

```text
self._state is Ready
```

may cease to hold across `await` if shared object can be modified by another fiber. Type/flow/proof systems need effect/ownership assumptions before retaining it.

## 12. Analyzer confidence versus proof

LSP can expose confidence/provenance. Checker diagnostics require stronger standards:

```text
not inferred != type error
heuristic fact != proof
unknown != top type
approximate union != exact exhaustive type unless soundness established
```

## 13. Static semantics for modes

Phalcom may eventually have:

```text
normal dynamic run
checker
strict/typed runner
static prover
LSP advisory analysis
```

Each mode should document:

- accepted syntax/annotations;
- guarantees;
- runtime checks retained/inserted;
- dynamic escape behavior;
- native/reflection assumptions.

## 14. Differential conformance

Build fixtures where analyzer/checker predicts and runtime executes:

- exact class after construction;
- override dispatch;
- `super` target;
- Option branch refinement;
- module-qualified identity;
- field initialization;
- block non-local return behavior.

Mismatches expose semantic drift.

## 15. Competency checks

1. Why is `ValueShape::Instance(C)` not automatically language type?
2. What runtime statement justifies accepting selector send statically?
3. How can method mutation invalidate static proof?
4. Why does unchecked native primitive belong to trust boundary?
5. Difference between not-inferred and inconsistent?
6. Why may a field refinement fail across `await`?
