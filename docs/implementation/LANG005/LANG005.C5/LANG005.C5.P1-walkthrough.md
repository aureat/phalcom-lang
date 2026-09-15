---
id: LANG005.C5.P1
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
repository_head: e5152196d716ed70fc47d104a0fd789e602285db
---

# LANG005.C5.P1 — Implementation Walkthrough

## Outcome

P1 is complete as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. The shared
checkpoint remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` because P2 projection
and P3 integration/certification remain.

The implementation establishes first-class associated declarations and
conformance bindings, exact associated evidence, trait-property elaboration,
direct-field `via` accessors, incremental/source-index support, and executable
ordinary accessor lowering without adding storage, projection solving, or a
runtime conformance authority.

## Architecture actually implemented

The AST now preserves heterogeneous source categories:

```text
TraitMember::Behavior(BehaviorMember)
TraitMember::Property(TraitPropertyRequirement)
TraitMember::AssociatedType(AssociatedTypeDeclaration)

ImplMember::Behavior(BehaviorMember)
ImplMember::Delegation(DelegatedAccessorDef)
ImplMember::AssociatedTypeBinding(AssociatedTypeBinding)

ClassMember::Delegation(DelegatedAccessorDef)
```

Trait properties elaborate to ordinary getter/setter behavior and retain no
storage identity. A class or inherent implementation `via` declaration resolves
one exact directly owned `FieldId`; the canonical field signature supplies its
type and mutability. The resulting getter/setter is an ordinary callable and
ordinary selector conflicts apply. Conformance-local `via` is diagnosed and
does not acquire target-private-field authority.

Associated declarations use the trait-owned identity:

```text
AssociatedTypeRequirementId { owner: DeclarationId, index: u32 }
```

`TraitSurface.associated_types` is separate from callable requirements. A
conformance source binding is retained as:

```text
AssociatedTypeBindingTemplate {
    requirement,
    value_template,
    source_impl,
    source,
}
```

The per-`ImplId` `ConformanceAssociatedTypePlan` resolves names against the
exact trait surface, checks the RHS in impl-owned generic scope, retains
diagnostic failures, and records a fingerprint. Exact conformance evidence
materializes the template into:

```text
ConformanceEvidence.associated_types:
    BTreeMap<AssociatedTypeRequirementId, ExactAssociatedTypeBinding>
```

Behavioral and associated failures share `ConformanceCompleteness` through
`ConformanceFailure`; no parallel completeness query was introduced.

The source index maps associated declaration and conformance-binding locations
to the canonical associated requirement identity. Compiler lowering consumes
accepted inherent delegation definitions and emits ordinary generated getter
and setter closures using existing field access bytecode. There is no new VM
delegation category or runtime associated-type lookup.

## Important files and products

- `phalcom-ast/src/ast.rs` and parser consumers: member taxonomy and P1 syntax.
- `phalcom-semantic/src/traits.rs`: associated requirement identity/table and
  property requirement elaboration.
- `phalcom-semantic/src/impls.rs`: source binding plans, exact associated
  evidence, combined completeness, and inherent delegation contributions.
- `phalcom-semantic/src/checker/declaration.rs` and
  `checker/declaration_signature.rs`: canonical field signatures and ordinary
  generated accessor signatures.
- `phalcom-semantic/src/session.rs`, DB/query/fingerprint, and source-index
  modules: owner-complete incremental and editor products.
- `phalcom-core/src/compiler/lib/class_decl.rs`, `impl_decl.rs`, `mod.rs`, and
  `modules/semantic_lowering.rs`: ordinary accessor code generation and exact
  accepted-definition handoff.
- `docs/specs/objects/traits.md`: ratified P1 property/associated binding
  surface.

## Tests added

- AST associated declarations, bindings, properties, direct-field delegation,
  and unsupported-form parser coverage.
- Semantic associated identity/evidence/completeness coverage.
- Semantic negative matrix for duplicate/unknown/invalid associated bindings,
  missing/untyped/immutable/inherited delegation targets, and accessor
  conflicts.
- Incremental replacement/removal/readdition and cold-parity coverage in
  `incremental/associated_types.rs`.
- Core executable ordinary-accessor and custom-complementary-setter verticals
  in `traits_p3_closure.rs`.
- Source-index declaration/binding target identity coverage.

## Verification actually executed

The following completed successfully:

```text
phalcom-ast trait_syntax: 6/6
phalcom-ast integration impl_syntax selector: 16/16
phalcom-semantic capabilities::traits: 21/23; all new P1 tests passed
phalcom-semantic impls::queries: 53/53
phalcom-semantic incremental::associated_types: 3/3
phalcom-semantic incremental::db: 14/14
phalcom-core direct_field_delegation_executes_as_ordinary_accessors: 1/1
phalcom-core delegated_getter_coexists_with_custom_inherent_setter: 1/1
phalcom-core exact generic C4 runtime regression: 1/1
affected-crate cargo check: passed for ast, modules, semantic, core, and lsp
```

The plan's standalone `phalcom-ast --test impl_syntax` target does not exist in
the live manifest; the existing `integration` test binary with the
`impl_syntax` module selector is the equivalent and passed.

## Consultation and deviation record

The T2/T9 conformance-local `via` path was superseded after consultation:
class-body and inherent-impl `via` remain supported; conformance-local `via` is
rejected deterministically because conformance bodies have no target-class
private-field authority.

During T9, the generated accessor had no source-body `CallableAnalysis`, so an
accepted delegation definition was initially discarded by a body-analysis
gate. The adviser task `01a0a645-6654-7b20-99dc-973a16164e05` decided
`PROCEED`: retain only an exact accepted `(ImplId, source_member_index)` whose
AST member is `ImplMember::Delegation`; ordinary body-backed members still
require analysis. No synthetic analysis and no plan amendment were introduced.

## Deferred and residual failures

The full semantic trait gate has two inherited baseline failures:

```text
trait_default_replays_external_nominal_callable_dependencies
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Both report unexpected Universe `Bool` declaration dependencies/capabilities.
They are recorded as `C5-BL-07`, classification D, and were not changed or
weakened. Workspace/release certification, P2 projection, C6 trait-bound proof,
and P3 integration remain deliberately deferred.

## Final classification and follow-up

P1: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

Next: author and execute `LANG005.C5.P2` for `Self::Item` projection formation
and normalization. P2 must consume the associated identity, source plan, exact
evidence, and combined completeness products above rather than recompute them
from syntax.
