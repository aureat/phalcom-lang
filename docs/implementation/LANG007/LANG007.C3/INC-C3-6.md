# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P2
- Task/Gate: T7 — source type-member projections in all declaration annotations
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `db6529f215357702944fa0367351155f95ab7b9b`
- Current revision: `db6529f215357702944fa0367351155f95ab7b9b` (working tree contains the focused test and incident/checkpoint updates)

## 2. Escalation trigger

Mandatory trigger 7: the user-approved resolver seam appears to require a material
change to the alias/type-member dependency contract to make the canonical surface
and checked signature available in every annotation phase. Trigger 16 also applies
if an implementation changes accepted cycle behavior without a normative rule.

## 3. Required invariant

- One `AssociatedSurface` owns associated-name selection and collisions.
- The checked source signature remains a separate product, selected by stable
  `TypeMemberDeclarationId`.
- Consuming projections preserve that identity and invalidate on both surface and
  selected signature changes.
- Existing alias/generic-header dependency behavior is preserved unless separately
  amended; no second selector authority is introduced.

## 4. Expected architecture

The user directed session phase reorganization so source-member projection selection
works in all declaration annotations through the INC-C3-3 resolver seam. The plan
requires the canonical surface and checked product before each consumer, including
aliases, generic headers, member signatures, callable signatures, and fields.

## 5. Observed behavior

The focused field-annotation regression initially produced a symbolic projection
without source declaration identity. After publishing the checked source-member
product/surface before formal signatures, the field, alias, and generic-header
projection identity regressions pass.

A probe with `data Tensor<T: Type, S: Shape>()` and
`type V: Type = Tensor<Int, Shape::Scalar>` in the same module fails with
`AnnotationUnresolved: unresolved type Tensor` before publishing the alias. An
equivalent importing-module probe failed at the same point. Inspection confirms the
ordering cause: aliases are lowered around line 2681 before source nominal generic
signatures are built around lines 2977–3000. This is distinct from imported static-case
identity and demonstrates that “all declaration annotations” requires the generic
header/application environment as well as source-member surfaces.

The phase graph has a deeper ordering constraint. Alias lowering is an ordered pass
over the existing alias dependency graph and publishes expanded forms into
`LinkedTypeResolver`. `build_type_member_declaration` resolves parameter/result
annotations through that same resolver. Moving checked members before alias lowering
can remove aliases required by member signatures; lowering aliases first cannot
resolve source projections until member signatures and their AssociatedSurface
selections exist. Current records do not define how a dependency cycle crossing an
alias and a member signature is handled, or whether such a cycle is admissible.

## 6. Minimal reproductions

Command:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic source_type_member_projection_is_available_in_field_annotations -- --nocapture
```

Field repro, alias repro, and generic-header repros now select one test and pass. The
field's canonical projection retains `Some(TypeMemberDeclarationId(Geometry, Wrap))`.
The mixed alias/member cycle repro selects one test and confirms one
`TypeSignatureCycle` diagnostic and no published alias/member product.

Fixture:

```phalcom
class Geometry {
  type Wrap<T: Type>: Type
}
class Consumer<T: Type> {
  _value: Geometry::Wrap<T>
}
```

Additional same-module phase-order probe (failed; not retained as a regression):

```phalcom
type data Shape { Scalar }
data Tensor<T: Type, S: Shape>()
type V: Type = Tensor<Int, Shape::Scalar>
```

Command:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic source_alias_preserves_nominal_static_case_in_heterogeneous_application -- --nocapture
```

Observed diagnostic: `AnnotationUnresolved: unresolved type Tensor` at the alias
body. This is not evidence about cross-module identity; it fails before the alias's
generic application can be formed.

## 7. Relevant code path

```text
session alias lowering (session.rs around 2366)
  -> resolver receives expanded alias forms
session generic headers (around 2522)
session formal field/callable signatures (around 3119)
  -> resolve_type_member_projection sees no published member context
late AssociatedSurface + TypeMemberDeclarationProduct publication (around 3980–4460)
  -> resolver.update_associated_type_members
```

`build_type_member_declaration` calls `resolve_contextual_kind_syntax` and
`resolve_type_form` against the resolver while constructing the checked product.
Alias expansion is session-managed separately through `TypeAliasTable` and the
resolver's alias forms.

## 8. Scoped implementation diff

Eight focused tests are currently uncommitted:

- `source_type_member_projection_is_available_in_field_annotations`,
  `source_type_member_projection_is_available_in_alias_annotations`, and
  `source_type_member_projection_is_available_in_generic_header_annotations`;
  all retain the selected source declaration identity.
- `alias_and_type_member_signature_cycle_is_rejected_as_one_component`; confirms
  the ratified deterministic rejection and withholds both products.
- Nested structural static-constructor inference, preservation of an
  underconstrained static-domain variable, and inference through a generic
  application with a static-domain argument; all pass.
- An independent two-module T8 source-index regression proves imported nominal
  static-case identity survives through canonical resolution; it passes.

The semantic session now builds the mixed alias/member dependency graph, rejects
cycle components, publishes ready checked member products through
`AssociatedSurface` before alias and formal signature consumers, and topologically
waits for member/alias prerequisites. Incremental invalidation and all associated
category collision interactions remain to be closed.

## 9. Evidence

- `incremental::type_members` selected 12 tests and passed, including field, alias,
  generic-header, cycle, collision, and existing incremental cases.
- The same-module Tensor alias phase-order probe selected one test and failed with
  unresolved `Tensor`; the temporary test was removed and the failure is recorded as
  a blocker, not a passing gate.
- `adts::associated::lookup` selected 1 test and passed.
- `type_data_case_edit_matches_cold_static_domain_analysis` selected 1 test and passed.
- `foundations::static_domain_inference` selected 6 tests and passed.
- `foundations::generic_application` selected 11 tests and passed.
- `foundations::generics_core` selected 8 tests and passed.
- `session.rs` confirms aliases are lowered before generic headers and formal
  signatures; associated surfaces/member products are published later.
- `build_type_member_declaration` resolves domains through the session resolver.
- The discarded imported generic-application probe failed at `Tensor` resolution,
  before it could test provider static-domain identity; no test gate is claimed from it.

## 10. Implementer's causal hypothesis

The dependency graph now handles acyclic alias/member prerequisites in topological
order and identifies mixed SCCs before publication. The rule and initial tests are
in place. Remaining risk is exact incremental dependency invalidation and ensuring
the early surface incorporates every associated category before consumers run.

## 11. Alternative hypotheses

- The accepted language may already prohibit alias/member-signature dependency
  cycles; no such rule or diagnostic was found in the inspected specification and
  plan sections.
- The graph may be acyclic for all well-formed declarations, allowing a topological
  phase order. This has not been established by source/spec evidence.
- A unified dependency query may support cycles without changing observable
  semantics, but its authority and invalidation contract need to be specified.

## 12. Attempts already made

The accepted alias/member cycle policy was implemented and the focused type-member
suite passes. The generic-header dependency blocker was only probed and traced; no
phase-graph change was attempted because its cycle and invalidation effects exceed
the ratified decision. T4 progressed independently with additional semantic tests;
broader T4/type-member convergence remains part of G3.

## 13. Why local authority is insufficient

The accepted cycle decision resolves alias/source-member SCC behavior. The newly
confirmed same-module generic-application failure exposes a wider dependency graph:
nominal generic headers are produced after aliases, while source-member signatures
may themselves need those headers. Extending the scheduler to generic headers raises
cycle and invalidation semantics not covered by the accepted alias/member rule. The
implementer cannot silently broaden which declaration cycles are rejected or how
header-dependent products are invalidated.

## 14. Decision requested

On 2026-09-23 the user accepted rejection of declaration-signature cycles crossing
transparent aliases and source type-member signatures, with a deterministic
diagnostic, while preserving ordinary symbolic projection normalization. The
`docs/specs/type-members.md` amendment makes the source rule normative. The plan
requires topological ordering, withheld products for cycle participants, one
diagnostic per component anchored to its lowest stable declaration identity, and
invalidation/rechecking of the affected component and dependents. The dependency
graph schedules checking only; `AssociatedSurface` remains the sole lookup authority.

Additional decision required: should the single declaration-signature dependency
graph include nominal/trait generic headers alongside aliases and source type-member
signatures, so all declaration annotations can resolve complete generic applications?
If yes, specify whether cycles involving generic headers follow the same deterministic
rejection rule or retain their existing diagnostics/behavior. Recommendation: include
the headers and reject any declaration-signature SCC involving a source type-member
signature with the same one-diagnostic/withheld-products policy, while preserving
existing alias-only and generic-header-only cycle behavior until separately specified.

## 15. Resume decision

The user ratified rejection of alias/source-member signature cycles, and the rule,
diagnostic, plan, checkpoint, and focused mixed-cycle regression were implemented.
T7 is stopped on the newly identified generic-header/application phase dependency
pending the decision above. The field, alias, and generic-header source-projection
identity regressions pass; the Tensor alias probe is not a passing gate.

## 16. Durable record

P2 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. Focused source projection and
cycle tests pass, but T7 and P2 are not complete. This report does not claim full
annotation-phase coverage, complete incremental invalidation, or P2 acceptance.
