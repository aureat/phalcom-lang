# LANG007.C3.P4 audit remediation — nested derived conformance runtime layout

Date: 2026-09-27

## Finding

The audit's CV50 regression has a conditional `Combined` conformance whose
selected implementation requires two distinct nested conformances (`A |= First`
and `B |= Second`). Core projected each nested proof into runtime slot `0`, so
the runtime environment could not provide the selected implementation with its
full evidence layout. The failure occurred when `Forwarder.s` requested the
second witness after `Forwarder.f` had completed.

## Correction

The runtime recipe builder now obtains the selected source impl's generic
signature (or the guarded rule's signature), builds its canonical
`GenericEvidenceLayout`, and aligns the `DerivedConformanceEvidence.nested`
proofs with the original source-constraint sequence. Equivalent and subtype
proof entries remain in alignment but do not consume conformance slots. Each
executable requirement is mapped by its canonical
`GenericEvidenceSlotKey`; selector spelling and iteration order are not used as
slot identity. Non-callable `DataComponent` selections remain absent from
runtime method recipes.

This is projection of established semantic evidence. The compiler does not
rerun conformance proof search or synthesize a second evidence authority.

## Evidence

Commits:

- `4f0edc63` — preserve nested conditional evidence slots;
- `250c4ad4` — preserve omission of non-callable nested evidence slots.

Focused command:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core language::conditional_conformance::cv50_slot_isolation_multiple_evidence_values -- --exact --nocapture
```

Result: 1 test selected, 1 passed, 681 filtered. The supervisor independently
reran this exact selector after both commits. No broad core, workspace, or
release gate was run.

## Priority 2 focused regression outcomes

The nine audit-named core selectors were each run individually and passed after
the applicable narrow source-fixture or projection correction. These are
focused results only; no broad core, workspace, or release gate was run.

| Audit selector | Correction / classification | Evidence |
|---|---|---|
| `language::associated_traits::direct_field_delegation_executes_as_ordinary_accessors` | Changed only the receiver from `let counter` to `const counter`. The index contained an eligible contribution; the receiver had become `Unknown(NoTypeEvidence)` before resolver entry after opaque sends. This was not a missing default-method installation. | `293c8766`; 1 selected, 1 passed |
| `language::conditional_conformance::cv50_slot_isolation_multiple_evidence_values` | Nested conditional proofs now map through the selected impl's canonical evidence layout; non-callable `DataComponent` slots remain omitted. | `4f0edc63`, `250c4ad4`; 1 selected, 1 passed |
| `language::conditional_conformance::cv51_environment_reuse_repeated_invocations` | Stabilized the intended inputs as `const`; assertions and flow policy are unchanged. | `8b680710`, `3edca835`; 1 selected, 1 passed |
| `language::derived_conformance::cv44_repeated_guarded_derived_calls_reuse_runtime_environments` | Changed only `c1` and `c2` from `let` to `const`; assertions are unchanged. | `c0fd12b2`; 1 selected, 1 passed |
| `language::generic_conformance::bounded_generic_function_dispatches_exact_witness_and_default` | Stabilized the intended input as `const`; assertions and flow policy are unchanged. | `8b680710`, `3edca835`; 1 selected, 1 passed |
| `language::generic_conformance::multiple_type_parameters_with_distinct_traits_execute` | Stabilized only the inputs whose current facts were invalidated before the calls; assertions are unchanged. | `84051c01`; 1 selected, 1 passed |
| `language::generic_conformance::owner_and_method_local_evidence_execute_together` | Stabilized `a` and `b`; `w` remains `let` because it has no intervening opaque call. Assertions are unchanged. The test is not ignored in the live tree: `--ignored` selected 0, while the normal exact selector selected and passed 1. | `ae5409b9`; normal exact selector 1 selected, 1 passed |
| `language::generic_conformance::runtime_conformance_environment_reuses_identical_environments` | Stabilized the intended input as `const`; assertions and flow policy are unchanged. | `8b680710`, `3edca835`; 1 selected, 1 passed |
| `language::generic_conformance::same_trait_on_distinct_type_parameters_selects_distinct_witnesses` | Stabilized the relevant inputs as `const`; both distinct-witness assertions are unchanged. | `72565cb7`; 1 selected, 1 passed |

## Forwarding boundary and next action

The selected-fallback branches of `Bytecode::InvokeConditional` and
`dispatch_conditional_family_entry` already carry their explicit runtime type
and conformance environments. Ordinary overrides remain evidence-neutral.

No environment-erasing runtime fix was established for rest or module-export
forwarding. The existing
`p4_specialized_rest_dispatch_preserves_conformance_environment` probe
publishes `Unknown(DynamicMessageSend)` without a callable or call
specialization. A separate bounded-caller-to-rest-target probe likewise had
`callable=None` and `call_specialization=None` at the `Sink.consume` expression
(`[330,354]`), so projection supplied no recipe. In the module-export probe,
`provider.proxy(provider.User.new())` (`[44,79]`) also had no callable or call
specialization, and the entry recipe map was empty. Both runtime probes stopped
at recipe-presence checks before execution. No speculative rest or
module-export adapter change was made; neither probe proves a runtime
environment drop.

Priority 2's nine named selectors are individually green, not a broad runtime
certification. Next classify the audit's 40 semantic failures (Priority 3),
then resume the remaining T4/T5 obligations as directed by the audit. P4
remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; overall audit and release gates
remain open.
