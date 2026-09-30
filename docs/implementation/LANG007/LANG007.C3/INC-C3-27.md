# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P4.R1
- Task/Gate: T6-A / G5 — canonical result of `RangeIterator::toList`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Evidence revision: `c138c4c3b229e817f90695a21277b59141980a05`
- Incident and checkpoint updates are prepared for delivery after this evidence revision.

## 2. Escalation trigger

The adopted INC-C3-26 ruling places the ordinary body-bearing
`RangeIterator::toList` result probe inside R1. Its body starts with the
explicit source form `List<Dynamic>.new()`, but live type-form resolution
returns a whole-form Dynamic escape rather than a known applied `List` type.
That result flows into an unknown mutable binding and unknown normal return.
The current specs and type-form representation do not clearly decide whether
an explicit Dynamic generic argument should preserve the outer applied type.
Changing that representation or its proof/escape semantics is outside local
implementation authority.

## 3. Required invariants

- Preserve explicit Dynamic boundaries; do not default Unknown to Dynamic.
- Do not add an explicit result annotation or change the library body to conceal
  the result of the existing source expression.
- Do not add a `RangeIterator`, `toList`, or method-name special case.
- Do not weaken canonical callable-result, type-identity, or conformance
  authority to satisfy T6.
- Preserve INC-C3-26: inherent-before-default precedence and the Range public
  surface remain unchanged.
- Keep R1 and P4/C3 incomplete until T6-A and T6-B have evidence-backed
  dispositions.

## 4. Expected architecture

R1 publishes canonical inferred results from ordinary body-bearing callables.
The adopted INC-C3-26 ruling identifies the apparent current result of the
`RangeIterator::toList` body as the canonical equivalent of `List<Dynamic>`,
without narrowing its element type. The same ruling requires direct body,
normal-return, and `CallableSignatureTable` inspection, and says to treat a
specific upstream semantic obstacle as a separate issue rather than patching
result publication around it.

The unresolved question is prior to result publication: whether
`List<Dynamic>` is a well-formed known applied type in value-type formation,
or whether Dynamic as a nested generic type argument intentionally makes the
entire form a Dynamic escape. No local change to type-form representation is
authorized until this distinction is settled.

## 5. Observed behavior

The direct `RangeIterator::toList` body selector observed:

- `List<Dynamic>.new()` has `TypeKnowledge::Dynamic(ExplicitEscape)` and
  `AnalysisStatus::DynamicBoundary`; it has no resolved callable identity.
- `self.next` is Known/Assumed.
- `result` is a mutable binding with no contract and
  `Unknown(NoTypeEvidence)`.
- `result.append(value)` has no callable and is
  `Unknown(NoTypeEvidence)`/Ready.
- The terminal `result` and sole normal-return product are
  `Unknown(NoTypeEvidence)`/Ready.
- The canonical `CallableSignature` exists with declared
  `Unknown(UnannotatedDeclaration)` and `inferred_return = None`.

The corrected natural chain establishes a Known post-`take` receiver:
`TakeIterator<RangeIterator>`. Its terminal `.toList` expression is currently
`Unknown(UncheckedExpression)`/Ready with no callable or trait-dispatch site.
A direct resolver query returns `Incomplete` for the existing
`TakeIterator` Iterator conformance (`Universe::collections.iterator_laws`,
impl local 20). This is separate from the
`List<Dynamic>` type-form question and no conformance change was made.

## 6. Minimal reproductions and verification

The temporary direct body probe selected 1 test and passed as a diagnostic
selector:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration::range_iterator_to_list_publication_probe -- --exact --nocapture
```

The temporary nested-generic discriminator
`semantic::foundations::generic_inference_proof_integrity::nested_dynamic_generic_result_classification_probe`
selected 1 and passed. Its temporary fixture had a generic callable whose
result depends on `T`, instantiated with `T` from a Dynamic actual, and
produced `Dynamic(ExplicitEscape)` for the dependent `Box<T>` result. The
probe was removed after evidence collection; its exact Cargo invocation was
not retained.
Both temporary probes were removed after evidence collection; no production
edit was made for them.

Existing controls distinguish a fully dependent result from a fixed result:
`generic_dynamic_premise_produces_dynamic_dependent_result` establishes
Dynamic propagation for `T -> T`, while
`dynamic_non_return_generic_premise_keeps_fixed_return_known` preserves its
fixed `Int` result with DynamicBoundary status.

The natural-chain selector passed 1/1 for its asserted `.iter` and `.take`
products, including Exact `Range -> Iterable::iter`, inherent
`RangeIterator::take`, and Known `TakeIterator<RangeIterator>`. It does not
assert the terminal `.toList` selection.

The current T6-B selectors were each selected but stopped before their
Derived/lowering assertions at the test's global `semantic.has_errors()`
precondition on the recorded Universe bootstrap `ImplConformanceIncomplete`
diagnostics:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration::known_list_iterator_source_receiver_gets_guarded_iterable_to_list_evidence -- --exact
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::traits::ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence -- --exact
```

Each selected exactly one test and failed before the focused assertion on the
recorded 23 Universe bootstrap diagnostics (20 `ImplConformanceIncomplete`,
3 `type.generic.underconstrained`). This is not acceptance evidence for T6-B
and is not a reason to bypass the precondition or modify conformance.

## 7. Relevant code and semantic path

```text
List<Dynamic>.new()
  -> resolve_type_form sees Dynamic generic argument
  -> TypeFormResolution::Dynamic
  -> resolve_type_annotation preserves Dynamic(ExplicitEscape)
  -> constructor has no resolved callable
  -> result binding / append / normal return remain Unknown
  -> canonical inferred_return is absent
```

Anchors:

- `phalcom-semantic/src/types/annotation.rs`: `resolve_type_form` Dynamic
  generic-argument early return (line 2767 in evidence revision) and
  `resolve_type_annotation` mapping (line 3995).
- `phalcom-semantic/src/checker/call.rs`: Dynamic proof handling for dependent
  generic results (line 2301 in evidence revision).
- `phalcom-core/core/universe/src/collections/list.ph`: canonical
  `List<T>.new() -> List<T>` declaration.
- `phalcom-core/core/universe/src/collections/range.ph`: the
  `RangeIterator::toList` body.
- `docs/specs/objects/data-objects.md` and
  `docs/specs/objects/enums.md`: Dynamic-boundary/non-erasure rules concern
  already-established applied runtime identity but do not explicitly settle
  static formation of an application whose written argument is Dynamic.

## 8. Scoped implementation diff

The existing local commit `c138c4c3` repairs `RangeIterator::take` through the
missing `TakeIterator` import and a shared non-generic-call rule for an
untyped `Unknown(NoTypeEvidence)` formal, with direct and natural-stage tests.
That commit does not alter Dynamic type formation, callable-result publication
for `toList`, trait dispatch, or conformance. The diagnostic probes for this
incident were removed. This incident authorizes no production change.

## 9. Implementer's causal hypothesis

The direct `toList` result is not presently lost in callable-signature
publication: its body never establishes a Known applied `List` value. The
type-form layer turns the Dynamic generic argument into a whole-form Dynamic
escape, and the constructor application likewise has a dependent Dynamic
result. The current mutable binding then has no static contract and remains
Unknown. The missing `inferred_return` is downstream of that boundary.

This is an attribution, not a settled language rule. The specifications cited
above do not determine whether explicit `List<Dynamic>` must instead retain a
Known outer nominal type.

## 10. Decision requested

Please decide:

1. Should explicit Dynamic as a nested generic type argument (for example,
   `List<Dynamic>`) intentionally make the entire applied type-form result a
   Dynamic escape, or should it preserve a Known outer applied type while
   retaining Dynamic for the inner argument? If the latter, what representation
   and static proof rules must be used without conflating Dynamic with a proper
   `TypeId`?
2. Given that decision, how should R1 classify `RangeIterator::toList` under
   T6-A: as a result-publication defect to fix through a specified shared
   mechanism, or as a source body whose explicit Dynamic boundary does not
   establish the canonical result assumed by the ruling?

Do not authorize a `RangeIterator` special case, result annotation workaround,
Dynamic default, or conformance/dispatch change as a substitute for resolving
the type-form contract.

## 11. Interim disposition

**PENDING ARCHITECT RULING.** No production change is made. T6-A has focused
Known results for `.iter` and `.take`; the direct `toList` result remains
unresolved at type-form formation. The natural terminal dispatch is separately
blocked by the existing incomplete `TakeIterator` Iterator conformance. T6-B
selectors are currently blocked before their assertions by the recorded
Universe diagnostics. R1/G5 and P4/C3 remain `IN_PROGRESS / PARTIAL /
UNVERIFIED`; no corpus or workspace gate was run.
