# Random delivery handoff

Plan state: IN_PROGRESS / PARTIAL / FOCUSED_TESTED.
Branch: `main`; starting HEAD `800474d8`; stable-owner prerequisite `fbf057ea`.
The user requested commit/push at this checkpoint. The delivery commit contains
this handoff and the source patch; find it with `git log --oneline --grep='random'`.
No subagents were used. The random source API is present; plan acceptance is
still incomplete.

## Incident and established repair

The original R0 probe exposed incomplete stable callable ownership projection.
Schema v11 now preserves declaration, variant and source impl owners. A subsequent
method-generic call failed because inference retained requirement binders while
execution expected witness binders; selected trait invocation also dropped its
call-entry recipe. Both are repaired and regression-tested. A later bootstrap/
client mismatch required trait-local default slot numbering. Qualified module
class exports now retain canonical class-object/type-form denotation.

The earlier constant blocker was an unsupported static shift, not unsafe LargeInt
materialization. Named constants now use exact literal values. Xoshiro's private
state stays in its class-owned `nextU64`; ordinary inherent-witness conformance
supplies Generator. The trait retains all source-authored derived defaults.

## Last completed step

`RUSTFLAGS='' cargo test -p phalcom-core --test core random_ -- --nocapture`
passed 4/4 after correcting test diagnostic-text matching. It covers facade
evidence, secure public behavior, secure nonconformance/hidden traversal/wrong
types, and numeric-policy sampling without candidate overshoot.

Previously passed: random/syntax corpus 2/2, core traits 18/18, secure native
success and injected host-failure/error-lifting 2/2, semantic binder/proof and
cold/incremental regressions, four crate checks, native surface check (400 rows),
and diff hygiene.

## Remaining next steps

1. Measure the plan's nextU64/nextFloat/bounded Int/128-bit/bytes/shuffle workloads,
   including allocation/GC evidence and source MINSTD comparison.
2. Add focused effect/native-source identity assertions and remaining byte
   allocation/resource acceptance. Native declaration already carries mutation,
   nondeterminism and blocking; assertions are still needed.
3. Run provider/module, native-source verification, numeric-resource and full
   corpus gates. Run or explicitly defer optional broad core. Review the final
   scoped diff and promote lifecycle only when required evidence is acquired.
4. Investigate retained-session field-dependency validation if it blocks broader
   corpus evidence. A stale-snapshot fallback is recorded in the incident; the
   independent syntax lane is fixture isolation, not an incremental fix.

## Preserve

- No default/global RNG or implicit seed; standard frozen SplitMix64/xoshiro256++.
- No reservoirs; high-bit bool, top-53 float, masked-first-word arbitrary Int,
  big-endian bytes, descending Fisher–Yates, zero-consumption special cases.
- SecureRandom never conforms to Generator or falls back; source owns allocation
  and EntropyError lifting; getrandom 0.3.3 only fills existing bytes.
- No compiler conformance inference or fabricated callable owner identity.

Main files: `phalcom-core/core/universe/src/random/`, `primitive/random.rs`,
`phalcom-modules/src/builtin.rs`, native-meta/surface, checker call/context/
expression, and core semantic lowering/materialization/dispatch.

Four semantic trait failures and workspace formatting drift are baseline; see
walkthrough/incident. Do not spend random acceptance work repairing them.

First commands:

```sh
git status --short
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus random -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test core random_ -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --lib secure_random -- --nocapture
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check
```
