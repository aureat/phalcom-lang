---
id: LANG007.C4.algebraic-literals-generic-reification-handoff
kind: handoff
checkpoint: LANG007.C4
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
created: 2026-10-09
---

# Generic reification repair handoff

The generic covering-impl runtime reification blocker is resolved and recorded
in `algebraic-literals-generic-reification-walkthrough.md`.

## Stable invariant

Semantic call specialization remains declaration-canonical. A covering-impl
source body may retain an impl-owned parameter identity. At executable call-entry
projection, both coordinates receive the same actual runtime type recipe. The
VM does not know about covering impls and retains its intentional `Dynamic`
fallback when a genuinely erased parameter is absent.

## Evidence

The focused generic product, nested call, outer forwarding, independent binder,
semantic canonical-specialization, Quaternion exact descriptor, and conditional
operator probes pass. The four-crate check and changed-file rustfmt check pass.

## Next work

Continue the algebraic-literals follow-on at T3:

1. Resolve `algebraic-literals-t3-capability-boundary-incident.md`: the
   proposal's class-side trait requirements conflict with the normative trait
   specification's metatype/conformance boundary. Do not add an instance-side
   workaround.
2. add source-owned `Zero` and `One` capability vocabulary and Int/Float
   conformances;
3. add target-side `BasisRealizable<B>` and `RealEmbeddable` APIs, following
   existing trait syntax and associated-type support;
4. add Complex I-axis and Quaternion I/J/K direct realization implementations;
5. add explicit semantic tests for capability absence (`Complex<J/K>`), exact
   callable/conformance identity, and scalar-domain preservation;
6. only then implement the durable T4 contextual-realization product and T5
   compiler projection.

Do not mark the follow-on complete: typed basis assignments and imaginary suffix
execution are still intentionally unfinished.
