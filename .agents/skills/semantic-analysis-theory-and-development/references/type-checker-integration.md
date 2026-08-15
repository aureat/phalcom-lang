# Type-Checker Integration

## 1. Shared semantic substrate, separate type domain

The future checker should reuse semantic identities, scope/name resolution, normalized calls, CFG/program points, module surfaces, call dependencies, effects, and provenance. It should **not** reinterpret current `ValueShape` as the Phalcom type language.

The bridge is:

```text
semantic identity/control foundation
        |
        +--> runtime-shape analysis (advisory/current)
        +--> Type facts + constraints (future checker)
        +--> effect/refinement/proof domains
```

## 2. Preserve distinct absence/uncertainty states

A checker must distinguish:

```text
annotation absent
Dynamic explicitly chosen by programmer
fresh inference variable α
unknown because analysis lacks information
blocked because dependency is missing
ambiguous resolution
inconsistent constraints
unreachable expression / bottom
```

Conflating these silently changes type-system policy.

## 3. Checker judgments

A future bidirectional checker may use:

```text
Γ ⊢ e ⇒ T      e synthesizes type T
Γ ⊢ e ⇐ T      e checks against expected type T
```

`Γ` should refer to resolved `BindingId`/declaration identities, not source strings. Semantic lowering can supply normalized sends, captures, and control targets. CFG refinements can refine the type environment per program point.

Example:

```phalcom
let x = source()
if x is String {
  x.length()
}
```

The semantic engine owns binding identity and branch/program point. The type system owns the meaning of `is`, the refinement `T ∩ String`, and member compatibility. A runtime-shape hint that `source()` often returns `String` is not a substitute for that proof.

## 4. Constraints and provenance

Constraint generation should preserve causal sources:

```text
α <: String        because parameter annotation
Number <: α        because argument expression synthesizes Number
```

When unsatisfiable, the diagnostic can cite both evidence chains. Semantic provenance from call/return/binding flow can augment this chain.

Do not store only final inferred types if future diagnostics need to answer why.

## 5. Dispatch remains dynamic semantics

Type checking may verify that a receiver type supports selector `S`, but types should not alter selector identity or runtime target selection unless a separate normative language decision introduces typed dispatch.

Possible rule shape:

```text
Γ ⊢ receiver ⇒ T
members(T, S) establishes call admissibility
--------------------------------------------
Γ ⊢ receiver.S(args) ⇒ R
```

This is a static judgment. Runtime still uses receiver object/class + selector semantics. For union types, the checker may require every reachable alternative to admit the send or use a specified gradual/dynamic rule.

## 6. `ValueShape` as evidence, not type truth

`ValueShape::Instance(C)` can be useful to improve IDE presentation or supply a low-trust inference hint. A correctness checker may consume it only through an explicit bridge with stated guarantees. Observed call-site shapes are especially open-world: future/unseen calls can change the set.

A safe architecture labels evidence sources and checker policy decides what may become a constraint.

## 7. Flow-sensitive typing

Reuse CFG and branch facts, but keep domains separate. Type refinement transfer:

```text
true edge:  Γ[x] := Γ[x] ∩ String
false edge: Γ[x] := Γ[x] \ String   // only if type algebra supports sound subtraction
```

The exact operators belong to Phalcom type theory. Semantic analysis supplies the branch predicate identity and program point.

Captured mutation can invalidate refinements. If a closure/native/dynamic send may mutate captured `x`, a checker must kill or weaken facts according to effect/alias rules.

## 8. Generics and specialization

Generic substitution should be keyed by canonical type parameters and semantic declaration IDs. Avoid cloning class/callable semantic identities per inferred use unless the normative type system defines specialization identity. Type specialization metadata and runtime class identity are different axes.

## 9. Incremental checking

Type-check queries depend on:

```text
resolved declaration/interface
annotation/type descriptor
body HIR/CFG
callee/type signatures
subtyping/protocol relations
effect/refinement facts
configuration/profile
```

Track these dependencies separately from runtime-shape summary dependencies so an advisory shape update does not unnecessarily invalidate formal typing unless the checker explicitly consumes it.

## 10. Tests

- explicit annotation versus absent annotation versus `Dynamic`;
- same source binding IDs used by LSP and checker;
- flow refinement killed by mutable capture/effect;
- type error provenance crosses call/return chain;
- union receiver call admissibility;
- annotations do not change selector/callable identity;
- inserting a correctly inferred explicit annotation preserves checker result;
- incremental/full checker equivalence.

## 11. Review questions

1. Is this fact a runtime shape or a language type?
2. Which semantic IDs/program points does the checker reuse?
3. What inference variable/constraint relation is generated?
4. Is missing information being silently turned into `Dynamic`?
5. Can captured/dynamic effects invalidate a refinement?
6. Does typing verify dispatch rather than redefine it?
7. Is provenance sufficient to explain the constraint failure?
