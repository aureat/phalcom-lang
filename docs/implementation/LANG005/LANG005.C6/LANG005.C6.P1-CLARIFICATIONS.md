# LANG005.C6.P1 — Mandatory Clarifications and Execution Amendments

Treat this message as an **authoritative clarification of the C6.P1 implementation plan**, not as a request to redesign the plan from scratch.

The existing `LANG005.C6.P1` plan remains the implementation plan. However, several assumptions/examples in the plan and surrounding C6 records require explicit correction or interpretation before implementation proceeds.

These clarifications override conflicting examples or wording in:

```text
LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
docs/specs/generic-constraints.md
```

where the conflict is specifically identified below.

Do not silently propagate the incorrect assumptions.

---

# 1. Critical language-model correction: Phalcom has no free functions

## 1.1 Fixed fact

Phalcom does **not** support top-level/free function declarations.

Executable user-defined callable behavior is method-based.

Therefore examples such as:

```phalcom
render<T>(_ value: T) -> String
    where T |= Printable
{
    value.print
}

render(User.new())
```

are **not valid Phalcom source examples**.

Likewise, examples such as:

```phalcom
print<T>(...)
map<F, A, B>(...)
foo<T>(...)
```

must not be implemented or tested as free-function declarations.

Do not add a free-function declaration form to support any C6 example.

Do not alter the AST to add top-level functions.

Do not treat a top-level function-shaped example in the plan or generic-constraints specification as evidence that free functions are intended.

The live language model is method-based.

---

# 2. How to interpret free-function-shaped examples in the generic-constraints spec

`docs/specs/generic-constraints.md` currently contains function-shaped examples and even language suggesting that functions can own `where` clauses.

For the purposes of C6.P1:

> Preserve the **generic-constraint semantics** expressed by those examples, but reinterpret their callable surface as methods.

For example, the conceptual example:

```phalcom
print<T>(_ value: T)
    where T |= Printable
{
    value.print
}
```

should be tested using a legal method declaration, such as conceptually:

```phalcom
class Renderer {
    render<T>(_ value: T) -> String
        where T |= Printable
    {
        value.print
    }
}
```

and invoked through ordinary Phalcom receiver semantics.

Use whatever exact class-side/instance-side invocation form is already valid in the repository.

Do not invent new syntax merely to reproduce this example.

The important proposition is:

```text
method-local T
+
where T |= Printable
+
value.print justified by assumed evidence
+
concrete method invocation discharges T |= Printable
```

The important proposition is **not**:

```text
Phalcom supports generic free functions
```

The latter is false.

---

# 3. This correction affects architecture, not merely examples

Do not treat the free-function issue as a cosmetic example rewrite.

C6.P1 must integrate generic conformance evidence with Phalcom's existing **receiver-specialized method-call architecture**.

The live semantic system already has canonical receiver-to-owner specialization machinery, including the repository-current equivalents of:

```text
specialize_receiver_to_owner(...)
ReceiverSpecialization
owner-relative generic environment
method-local generic application
```

C6.P1 must extend this architecture.

It must not create a receiverless generic-call subsystem alongside it.

---

# 4. Mandatory receiver-specialization integration

For a generic method invocation, reason in terms of this architecture:

```text
receiver expression
    ↓
resolve behavioral member / declaring owner
    ↓
specialize receiver to declaring owner
    ↓
obtain owner-relative generic substitution/environment
    ↓
compose owner generic contract/evidence
    +
method-local generic contract/evidence
    ↓
perform ordinary method-local generic inference
    ↓
instantiate direct conformance obligations
    ↓
discharge them from:
    - lexical assumed evidence, or
    - exact C4/C5 conformance evidence
    ↓
publish canonical call specialization
    +
canonical evidence source/recipe
    ↓
lower call environment
    ↓
execute method with type + conformance environments
```

The exact internal staging may adapt to the live code if mechanically necessary.

The architectural requirements may not.

In particular:

> C6 generic evidence must compose with `ReceiverSpecialization`; it must not bypass, duplicate, or replace receiver specialization.

If implementing P1 appears to require a separate generic method application algorithm unrelated to the existing receiver-specialization path, STOP AND CONSULT.

---

# 5. Owner-generic evidence is a first-class P1 requirement

Do not limit P1 to method-local generic parameters.

C6 evidence must work for generic constraints owned by the receiver's declaration as well.

For example, conceptually:

```phalcom
class Box<T>
    where T |= Printable
{
    describe(_ value: T) -> String {
        value.print
    }
}
```

The exact legal Phalcom method syntax should follow the live parser/specification.

The semantic requirement is:

```text
Box<T>'s owner generic signature
    contains Conforms(T, Printable)

method body analysis
    sees that owner assumption

receiver specialization for Box<User>
    establishes T = User

method invocation
    obtains/forwards evidence for User |= Printable
```

This must not rely only on method-local `<T>` inference.

The following must compose correctly:

```text
owner-level generic evidence
method-level generic evidence
receiver specialization
method-local type inference
exact evidence
forwarded assumed evidence
```

---

# 6. Lexical evidence composition rule

The plan's statement that owner and callable constraints compose is intentional and remains fixed.

The effective evidence environment for a method body must include the semantically applicable combination of:

```text
declaring owner generic constraints
+
method generic constraints
+
other legitimate lexical generic-domain assumptions
```

Do not interpret this as simply concatenating raw source constraints.

The resulting environment must be canonical and evidence-aware.

Do not leave owner constraints accessible only through `ambient_constraints` while method constraints use a new evidence mechanism.

There must be one coherent generic evidence model.

---

# 7. Do not confuse receiver generic substitution with evidence

Keep these separate:

```text
ReceiverSpecialization / type environment
    tells us:
        T = User

GenericEvidenceEnvironment / conformance environment
    tells us:
        T |= Printable
```

After substitution, the latter can become an obligation such as:

```text
User |= Printable
```

but type substitution alone does not prove conformance.

Likewise, conformance evidence does not replace receiver specialization.

C6 must preserve this separation all the way through semantic specialization and runtime lowering.

---

# 8. Generic method-to-generic method forwarding must be tested as method dispatch

The plan requires forwarding assumed evidence from one bounded generic callable to another.

In Phalcom, exercise this through methods.

Conceptually:

```phalcom
class Renderer {
    inner<T>(_ value: T) -> String
        where T |= Printable
    {
        value.print
    }

    outer<T>(_ value: T) -> String
        where T |= Printable
    {
        self.inner(value)
    }
}
```

Use the exact valid receiver/method syntax from the live language.

The required semantic result is:

```text
outer's assumed evidence for T |= Printable
    ↓
satisfies inner's instantiated T |= Printable obligation
    ↓
call specialization records forwarded evidence
    ↓
runtime copies/forwards the appropriate caller evidence slot
```

There must be no fabricated exact `impl` for abstract `T`.

There must be no runtime conformance search.

---

# 9. Same trait on different subjects remains a required discriminator

This requirement from the original plan remains important.

Use a **method-based** fixture to prove it.

The semantic problem is:

```text
T |= Printable
U |= Printable
```

and the method body invokes Printable behavior independently on values of both types.

The two calls may require different runtime evidence.

Therefore this remains fixed:

```text
TraitRequirementId alone is not a complete generic evidence-slot identity.
```

The evidence slot must be keyed by the equivalent of:

```text
canonical conformance obligation
+
TraitRequirementId
```

For example:

```text
(Conforms(T, Printable), Printable.print)
(Conforms(U, Printable), Printable.print)
```

Do not collapse them because the requirement identity is the same.

---

# 10. Re-check first-class method references / bound methods when evidence is required

Phalcom has first-class receiver-bound method references.

C6.P1 must explicitly audit whether a method whose execution depends on generic conformance evidence can be safely reified and invoked later under the current runtime call-environment model.

This is a VERIFY-FIRST obligation.

Do not invent a new closure/bound-method evidence representation unless required.

First determine whether the existing method-reference runtime path already preserves or reconstructs the necessary:

```text
receiver specialization
runtime type environment
runtime conformance environment
```

for the eventual invocation.

Questions that must be answered by repository inspection:

```text
1. Can a generic method with direct `|=` bounds be referenced today?
2. What generic type environment is captured or reconstructed?
3. If the method body needs assumed trait evidence, where will that evidence come from at later invocation?
4. Does existing BoundMethod/call environment machinery already provide the correct lifetime/transport seam?
5. Does an owner-generic receiver provide enough information to materialize evidence at invocation time?
```

If ordinary C6 evidence transport naturally covers bound-method invocation, add focused coverage.

If correct support requires a new persistent evidence-capture architecture beyond P1's planned runtime environment mechanism, STOP AND CONSULT rather than inventing it.

Do not silently leave a soundness hole.

---

# 11. Do not introduce a free-function-specific generic call path anywhere

Audit names and abstractions carefully.

Generic semantic infrastructure may legitimately use generic names such as:

```text
CallableId
CallSpecialization
GenericSignature
TypeParameterOwner::Callable
```

These do not imply source-level free functions.

Do not rename them merely because the language is method-based.

However, do not interpret generic internal term `Callable` as authorization to create free-function syntax or semantics.

`CallableId` is an implementation abstraction over callable behavior.

Source language callables remain governed by Phalcom's actual declaration/member model.

---

# 12. CHECKPOINT lifecycle clarification

The generated `LANG005.C6-CHECKPOINT.md` currently lists future entries P2–P6 with status `PLANNED`.

This is not canonical lifecycle vocabulary.

The valid status vocabulary is:

```text
PROPOSED
IN_PROGRESS
BLOCKED
DEFERRED
COMPLETE
SUPERSEDED
ABANDONED
```

More importantly:

> Only P1 currently has an actual patch-grade implementation plan.

Therefore P2–P6 should currently be understood as an **anticipated checkpoint decomposition**, not as completed implementation-plan records.

Do not infer that patch-grade P2–P6 plans already exist.

When checkpoint bookkeeping is next legitimately updated, normalize this distinction.

Preferred interpretation:

```text
Actual plan ledger:
    P1 — PROPOSED / NOT_STARTED / UNVERIFIED

Anticipated decomposition:
    P2 — generic associated projection/equality
    P3 — conditional/nested proof
    P4 — Iterator/Iterable laws
    P5 — production iteration migration
    P6 — closure/certification
```

Do not spend implementation time rewriting the checkpoint solely for this unless the active workflow requires a durable checkpoint update.

When you next update it for a real durable event, correct the status vocabulary then.

---

# 13. CHECKPOINT "established invariant" clarification

The checkpoint currently combines three categories:

```text
A. inherited implemented C4/C5 facts
B. ratified/normative C6 language semantics
C. accepted-but-not-yet-implemented P1 architecture
```

Do not confuse category C with already-landed implementation.

At P1 start:

```text
C4/C5 takeover interfaces
    = implemented predecessor fact

`|=`, `+`, `::`, relation distinctions, no Dynamic-as-proof
    = ratified/normative semantic fact

GenericEvidenceEnvironment,
exact/forwarded call evidence,
complete generic evidence slot layout,
TraitRef identity/proof split as implemented code
    = P1 target architecture until actually landed
```

As P1 tasks complete, the checkpoint may promote specific target architecture into implemented takeover state.

Do not report planned architecture as already implemented merely because it appears under the checkpoint's invariant section.

---

# 14. `where`-kind syntax is not allowed to derail P1

The normative generic-constraints design places explicit kind constraints in `where`, for example:

```phalcom
where F: (Type) -> Type
```

However, C6.P1's primary acceptance objective is the direct conformance/evidence vertical:

```phalcom
where T |= Trait
```

The repository currently also has existing generic-parameter kind machinery.

Therefore:

> Treat `where`-kind reconciliation as VERIFY-FIRST and additive only for P1.

If implementing canonical `where F: Kind` can reuse existing parameter-kind authority with a bounded parser/formation change, it may be included as planned.

If it requires:

```text
a broad kind-system redesign
large syntax migration
replacement of existing binder-local kind machinery
new kind solver architecture
unrelated higher-kinded trait completion
```

STOP AND CONSULT or defer it according to the existing P1 rule.

Do not let kind-syntax migration consume or block the direct `|=` implementation.

Do not create `GenericConstraint::Kind` merely for source-shape preservation unless an architectural consultation explicitly changes the plan.

---

# 15. The `TraitRef` split remains intentional

The existing plan's separation remains fixed:

```text
TraitRef identity formation
!=
proof that the applied trait's generic constraints are satisfied
```

Do not undo this clarification because the live `TraitRef::form` currently performs both.

Target meaning:

```text
TraitRef
    = canonical applied trait identity

generic evidence/proof layer
    = proves whether that application is semantically admissible
```

Identity formation may still check structural formation facts such as:

```text
declaration really is a trait
ordinary generic argument count
ordinary generic argument kinds
canonical argument formation
```

It must not become the recursive generic evidence solver.

If separating these responsibilities causes an unexpectedly broad architectural cycle, STOP AND CONSULT.

---

# 16. Exact evidence and assumed evidence remain different

Do not use a generic lexical bound to fabricate:

```text
ImplId
ConformanceEvidence
ConformanceHeadMatch
source witness/default provenance
```

For:

```phalcom
where T |= Printable
```

the body knows only:

```text
T conforms to Printable
```

plus the canonical trait surface and requirement identities justified by that proposition.

It does not know which concrete implementation will eventually satisfy the obligation.

At a concrete invocation:

```text
exact evidence
```

may satisfy the obligation.

Inside another generic bounded context:

```text
assumed evidence
```

may be forwarded.

Those are different proof sources under one generic conformance model.

---

# 17. Structural compatibility remains insufficient

Do not infer conformance from selector shape.

For example, a type exposing:

```text
print -> String
```

does not satisfy:

```text
T |= Printable
```

without actual conformance evidence.

This remains true for generic bounds.

Do not accidentally reuse ordinary method lookup as a conformance prover.

The trait surface tells us what operations a proven conformance authorizes.

It does not prove the conformance itself.

---

# 18. Direct conformance remains post-inference in P1

Preserve the planned P1 staging:

```text
receiver specialization
+
ordinary argument inference
+
expected-type inference
+
existing subtype/equivalence constraints
    ↓
candidate type substitution
    ↓
instantiate direct `Conforms(...)` obligations
    ↓
prove them
```

Do not turn:

```text
T |= Printable
```

into a new P1 unification relation.

Do not use arbitrary conforming types to guess an otherwise unconstrained `T`.

If the ordinary inference problem is underconstrained, the conformance bound does not authorize picking a concrete conforming type.

P2 may later introduce associated-evidence feedback where explicitly required.

That is not P1.

---

# 19. Runtime boundary remains strict

The VM must not receive enough semantic machinery to redo generic proof.

Runtime may receive:

```text
selected method handles
caller evidence-slot references
deterministic environment recipes
runtime type-environment recipe
runtime conformance-environment recipe
```

Runtime must not perform:

```text
TraitRef construction
generic constraint interpretation
conformance candidate search
coherence
associated projection normalization
bound inference
ambiguity resolution
```

Reuse the existing runtime conformance environment and trait requirement invocation path whenever semantically compatible.

A new runtime proof system is not P1.

---

# 20. Keep runtime type and evidence environments separate

Do not merge:

```text
RuntimeTypeEnvironment
```

and:

```text
RuntimeConformanceEnvironment
```

Receiver specialization and generic inference determine type substitutions.

Evidence discharge determines conformance behavior.

They are related but distinct runtime facts.

The generic method call frame may need both.

---

# 21. Stable metadata remains semantic, not executable

Cross-module generic signatures must expose enough stable information to reconstruct:

```text
subject |= TraitApplication
```

They must not expose:

```text
ConformanceEvidence
runtime evidence slots
RuntimeConformanceEnvironmentId
snapshot-local TypeId
snapshot-local TraitRef
inference variables
```

Do not confuse runtime evidence layout with public generic metadata layout.

---

# 22. Reflection must not pretend conformance is an ordinary type/type relation

Current generic-constraint reflection was designed for relations such as:

```text
Subtype(left, right)
Equivalent(left, right)
```

A conformance constraint is:

```text
Conforms(subject, trait_application)
```

The right side is a trait application, not merely "another ordinary type".

Do not fake it through an inaccurate left/right descriptor API.

Extend reflection in a relation-aware way while preserving compatibility where possible.

If doing this correctly requires a breaking redesign beyond P1's scope, consult.

---

# 23. Canonicalization requirement remains strict

These must be semantically identical:

```phalcom
where T |= A + B
```

```phalcom
where T |= B + A
```

and the equivalent separately written obligations.

Likewise duplicate bounds must not duplicate evidence:

```phalcom
where
    T |= Printable,
    T |= Printable
```

Canonicalization must happen before any deterministic evidence layout is published.

Do not let:

```text
source order
parser grouping
`+` grouping
duplicate source clauses
```

leak into public semantic identity or runtime slot numbering.

---

# 24. Test fixtures must use legal Phalcom source

Every new `.ph` test written for C6.P1 must use actual supported Phalcom declaration forms.

Before writing a fixture:

1. inspect nearby current tests;
2. confirm the legal method/class-side/instance-side syntax;
3. follow repository conventions;
4. do not copy free-function-shaped examples from `generic-constraints.md`.

This applies especially to:

```text
direct generic bound success
generic bound failure
generic-to-generic forwarding
same trait on two generic subjects
owner-level generic evidence
cross-module generic evidence
runtime generic evidence
Dynamic negative case
exact enum-case case
```

---

# 25. Minimum P1 evidence matrix after this clarification

P1 should prove at least the following semantic dimensions, using methods.

## A. Method-local direct bound

```text
method<T>(value: T)
where T |= Trait
```

Body uses Trait behavior.

Concrete conforming invocation succeeds.

Nonconforming invocation fails.

---

## B. Method-local same-subject conjunction

```text
where T |= A + B
```

Both trait surfaces are available.

Order/regrouping does not change semantic identity.

---

## C. Owner-generic bound

Generic owner:

```text
Owner<T>
where T |= Trait
```

A method body uses Trait behavior through `T`.

Invocation through a specialized receiver transports the correct evidence.

---

## D. Owner + method composition

Generic owner contributes one evidence assumption.

Generic method contributes another.

Both are simultaneously available and independently keyed.

---

## E. Generic method forwarding

Outer bounded method calls another bounded method.

Assumed evidence is forwarded rather than resolved as an exact conformance.

---

## F. Same trait, different subjects

```text
T |= Trait
U |= Trait
```

Both trait calls use the correct independent evidence.

---

## G. Exact concrete discharge

A concrete receiver/method invocation where the inferred generic argument has an existing exact conformance uses C4/C5 `ConformanceEvidence`.

---

## H. Structural negative

Matching method names without declared conformance do not satisfy the bound.

---

## I. Dynamic negative

`Dynamic` does not statically discharge `|=`.

---

## J. Receiver-specialization integration

A generic method inherited or reached through the normal owner/receiver specialization path retains correct evidence semantics.

Use the live inheritance/member architecture to choose the smallest discriminating fixture.

---

## K. Bound method / method reference

At minimum audit and classify whether generic-evidence-dependent bound method reification is already correct.

Add a test if supported without new architecture.

Consult if it exposes an unplanned evidence-lifetime problem.

---

# 26. Do not overreact to these clarifications

These clarifications do **not** authorize a redesign of:

```text
generic callable inference
receiver specialization
TraitRef identity
C4/C5 exact conformance
TraitDispatchSelection
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
associated projection normalization
source index
LSP architecture
```

Use existing authorities.

Extend the smallest correct seam.

---

# 27. Required implementation posture

Before changing production code:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then inspect the repository-current forms of:

```text
GenericSignature
GenericConstraint
GenericConstraintSyntax
TypeParameterOwner
CallSpecialization
specialize_receiver_to_owner
ReceiverSpecialization
CheckingContext generic/ambient constraint state
TraitRef formation
TraitDispatchSite
RuntimeCallEnvironmentRecipe
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
TraitRequirementInvocationSpec / InvokeTraitRequirement
BoundMethod / callable-reference lowering/runtime path
```

Do not assume the planning-time filenames remain exact.

---

# 28. Explicit consultation triggers added by this clarification

STOP AND CONSULT if any of the following becomes true:

```text
1. Direct generic evidence cannot compose with ReceiverSpecialization
   without replacing the existing receiver-specialization architecture.

2. Owner-level conformance evidence would require permanently storing
   per-instance evidence solely to make ordinary generic class methods work.

3. Bound-method reification requires a new evidence-lifetime/capture
   architecture that the P1 plan did not anticipate.

4. Correct method invocation would require runtime conformance search.

5. The only apparent way to support plan examples is to add free functions.

6. `where F: Kind` requires broad kind-system redesign before `|=` can land.

7. Generic assumptions can only be represented by fabricating ImplId or
   ConformanceEvidence.

8. TraitRef identity formation cannot be separated from recursive generic
   proof without an architectural cycle.

9. Evidence-slot identity cannot distinguish the same TraitRequirementId
   applied to two distinct conformance subjects.

10. Existing call-environment machinery cannot transport both receiver-derived
    type substitutions and generic conformance evidence without changing the
    runtime ownership model.

11. A generic method body can type-check from assumed evidence but the
    compiler has no semantically sound way to associate its trait requirement
    invocation with the correct generic evidence slot.

12. Fixing any issue above would require weakening existing C4/C5 exact
    conformance, ambiguity, exact-case, or associated-type semantics.
```

Do not invent around these conditions.

---

# 29. What is still fixed from the original P1 plan

The following architecture remains authoritative unless a genuine STOP/CONSULT condition disproves it:

```text
GenericSignature remains the generic contract authority.

Direct conformance is a general:
    Conforms(subject, TraitRef)

`|=` is conformance/evidence.

`+` is same-subject conjunction sugar.

Conformance is distinct from:
    kinding,
    subtyping,
    equivalence,
    structural method matching.

Generic assumptions and exact evidence are distinct.

TraitRef identity and generic proof are distinct responsibilities.

One lexical generic evidence model must serve bodies and applications.

Generic body trait behavior remains trait behavior identified by
TraitRequirementId.

Generic call evidence is discharged after ordinary type inference.

Generic assumed evidence can be forwarded across generic method calls.

Dynamic is not static conformance proof.

Runtime type and conformance environments remain distinct.

The VM does not prove conformances.

Existing trait-requirement execution machinery should be reused.

Evidence slot identity includes both:
    conformance obligation,
    TraitRequirementId.

Stable metadata contains stable semantic identity only.

Canonical generic semantic identity is independent of source bound ordering.

P2, not P1, owns generic associated projections and associated-evidence
feedback into inference.

P3, not P1, owns conditional recursive conformance proof.

P4+, not P1, owns Iterator/Iterable generic laws and production migration.
```

---

# 30. Immediate execution instruction

Proceed with `LANG005.C6.P1`, but interpret and implement it through the **actual Phalcom method/receiver architecture**.

The first implementation round should explicitly re-ground:

```text
generic method declaration formation
receiver-to-owner specialization
owner generic environment composition
method-local generic inference
call specialization publication
trait dispatch publication
runtime call environment construction
bound-method/callable-reference interaction
```

before deciding where P1 evidence data should attach.

Do not simply convert the existing free-function-shaped tests into parser fixtures.

The implementation is correct only if the new evidence model is a natural extension of Phalcom's existing method dispatch and receiver specialization model.

---

# 31. Final rule

Whenever an example, plan sentence, or older specification appears to assume a conventional free-function-oriented language architecture, translate the **semantic requirement** into Phalcom's actual object/method model before implementation.

Do not translate Phalcom's architecture into the example.

The language architecture is authoritative; the erroneous example is not.