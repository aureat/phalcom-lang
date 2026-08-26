# Constructor Advisory Corruption and System Primitive Audit

- Date: 2026-08-26
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Status: Cause identified and solved; spec-conformant follow-up repair remains subject to approval

## Summary

The reported constructor/LSP behavior was traced to advisory corruption, not incorrect formal constructor semantics. Constructor instance facts were seeded correctly, then replaced when normal-return flow became known. Constructor tail assignments evaluate to their right-hand side, so `Point` and `Weight` constructors could publish `Int` instead of the constructed instance and contaminate downstream results such as `Parcel.new`.

The diagnosis was corrected in one important respect: constructor identities must remain distinct. The public class-side factory is `C.new(...)`; the internal instance-side initializer is `C.init new(...)`. They must not collapse into one class-side `CallableId`.

## Verified findings

### Constructor semantics and advisory flow

- Formal constructor semantics are correct. Constructors publish class-side `Self` returns with `ConstructorSemantics`; 14 focused constructor tests passed. See [`declaration.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/declaration.rs:61) and [`call.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/call.rs:33).
- Advisory corruption is real. The seeded constructor instance fact is replaced whenever `flow.normal_return()` is known. Assignments evaluate to their RHS, allowing constructor tail assignments to publish `Int`. See [`session.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs:1335), [`analyzer.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/analyzer.rs:146), and [`weight.ph`](/Users/altunhasanli/dev/phalcom/phalcom/examples/ide-golden/deps/units/src/weight.ph:5).
- Constructor seeding and class-to-instance advisory translation rely on `"new"` heuristics. Named constructors are unsupported by this path. See [`session.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs:1121) and [`analyzer.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/analyzer.rs:461).
- Identity splitting exists, but the current synthetic instance identity is incomplete because it retains the original selector. The architectural completion specification requires separate factory and initializer identities. See [`architectural completion spec`](/Users/altunhasanli/dev/phalcom/phalcom/docs/work/analyses/phalcom_compiler_lsp_incremental_semantics_architectural_completion_spec.md:2234).

### LSP presentation and authority

- Inlay diagnosis is correct: legacy `local_facts` must be available before compiler formal data is consulted. See [`inlay_hints.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp/src/inlay_hints.rs:468).
- `SelfType` presentation is lost in callable hover/signature paths. See [`snapshot.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp/src/semantic/snapshot.rs:769).
- Existing constructor LSP regression passes. The failure is path-specific: compiler formal paths work, while project/compiler-advisory fallback remains broken.

## System primitive audit

`System.print` is correct at runtime, in generated metadata, and in formal checking:

- successful result: `Unit`;
- failure remains possible through display conversion;
- three core native tests passed;
- formal trusted-return test passed.

`System.gc` correctly returns `None`, not `Unit`.

| Primitive | Correct result | Current status |
| --- | --- | --- |
| `print(_)` | `Unit` | correct |
| `gc` | `None` | correct |
| `new()` | `Never` / always fails | correct |
| `schedule(_)` | `Fiber` | correct |
| `nextScheduled` | `Option<Fiber>` | correct |
| `_$write(_)` | `Unit` | runtime/authored declaration correct; generated artifact says `Option` |
| `_$leakReport` | `List<String>` | metadata `Unknown` |
| `_$strictResources(_)` | `None` | metadata `Unknown` |

Generator verification currently fails because [`generated.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-native-surface/src/generated.rs:10729) is stale. Active core source also declares `gc` and internal `System` primitives as `Dynamic` in [`fiber.ph`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/universe/src/concurrency/fiber.ph:3).

## Recommended repair design

1. Model public constructor factories and internal initializers as separate canonical identities, matching compiler lowering.
2. Carry `CallableSemanticKind::Constructor`; remove every `"new"` inference heuristic.
3. Keep initializer-body returns separate. Public constructor result always projects `Self` to constructed owner.
4. Make exact compiler snapshots primary for inlays; use legacy facts only for uncovered documents.
5. Present constructor `SelfType` contextually as its owner.
6. Precisely annotate all eight installed `System` primitives, regenerate native surfaces, and add runtime/metadata/formal table tests.
7. Add semantic-first constructor regressions, then hover/inlay/signature parity tests.

The smaller alternative is to preserve current synthetic instance IDs and add only constructor-kind metadata plus result dominance. It is cheaper but retains identity debt and conflicts with the repository's target design.

## Closure boundary

The root cause and immediate diagnosis are recorded as solved. This log does not claim completion of the broader constructor identity, LSP parity, or native metadata cleanup. Those remain explicit follow-up work and require approval before edits under the recommended spec-conformant design.
