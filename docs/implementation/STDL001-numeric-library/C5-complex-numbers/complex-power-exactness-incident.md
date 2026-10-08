# IMPLEMENTATION INCIDENT: power execution and large-integer exactness

## 1. Identity

- Program/checkpoint/plan: STDL001 / STDL001.C5 / STDL001.C5.P1.
- Task: T4 powers; G2 numerical acceptance.
- Repository: `.worktrees/complex-math`, branch `feat/complex-math`.
- Starting/current revision: `ad1db231` plus the uncommitted checkpoint patch.

## 2. Escalation trigger

The exact power discriminator now exposes a conflict between the general
representation-independent exactness contract and the runtime scalar fallback.
Correcting Int exactness is broader than implementing the Complex power kernel;
the user requires approval for significant choices.

## 3. Required invariant

Exact Int powers remain Int and produce arbitrary-precision integer components.
Complex exactness uses exact applied type and recursive component `===`.
`docs/specs/objects/values.md` requires semantic exactness to be independent of
incidental allocation. Float signed-zero exactness must remain bit-sensitive.

## 4. Expected architecture

Source-owned IntegerPower conformances use squaring. Existing VM semantic
exactness owns component identity. The numerical library must not introduce a
separate scalar exactness implementation or use coercive equality for `===`.

## 5. Observed behavior

The original GetUpvalue panic is resolved by the compiler's body-kind check.
The same exact power test now executes and reports `huge == expected` true,
but `huge === expected` false for independently produced 10^100 components.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex::complex_integer_and_principal_powers_preserve_exactness_and_zero_endpoints -- --exact --nocapture
```

Relevant input:

```phalcom
let actual = Complex(10 ** 50, 0) ** 2
let expected = Complex(10 ** 100, 0)
let numeric = actual == expected
let exact = actual === expected
```

Observed: `numeric` true, `exact` false. Expected: both true under
allocation-independent Int exactness.

## 7. Relevant code path

`VM::semantic_same` -> `Value::same_as` raw tagged/handle fast path -> recursive
data component `VM::semantic_same` -> no LargeInt scalar content case -> false.
`Value::value_eq` already compares LargeInt mathematical content, but also
performs Float/Int coercion and cannot be used as the general exactness fallback.

## 8. Scoped implementation diff

Powers use existing distinct conformance callable identities (D8). The compiler
now emits GetSelf for method bodies and resolves receiver upvalues only for block
bodies, using the established `FunctionState::is_block` flag. No VM capture
fallback was introduced; temporary diagnostics were removed.

## 9. Evidence

- Temporary VM diagnostics identified `numbers.complex` `/(_)` at ip 1 with
  `UpvalueDescriptor { is_local: true, index: 0 }` and zero runtime upvalues.
- Reentrant conditional compilation places this method beneath the Float power
  witness. The old stack-depth heuristic captured the outer receiver despite
  the method having its own slot-0 receiver.
- After the body-kind fix, the power test executes to the huge exactness
  assertion. Its separately checked ordinary equality is true.
- `semantic_same` handles structural products/data recursively and otherwise
  falls back to raw identity; independently allocated LargeInts have no case.

## 10. Causal hypothesis

The power calculation is exact. Independently normalized BigInts receive distinct
heap handles, and missing scalar content exactness refutes recursive data exactness.

## 11. Alternatives

Changing only the Complex oracle to numeric equality would verify arithmetic,
but would leave the observed scalar exactness divergence unresolved. General
`value_eq` fallback would incorrectly broaden exactness to numeric coercion.

## 12. Attempts already made

One compiler correction: gate `emit_self` capture on body kind. The panic ceased.
No scalar exactness correction has been attempted.

## 13. Why local authority is insufficient

Int exactness is a general runtime value contract, not a Complex-specific kernel
detail. The correction must be approved and placed at that owner.

## 14. Decision requested

Approve a narrow `VM::semantic_same` Int-content case, covering independently
allocated LargeInts and immediate/heap representations without Float coercion,
with a scalar regression and the unchanged Complex exact-power discriminator.

## Decision and outcome

The user approved this correction as D9. `VM::semantic_same` now compares
LargeInt content and equivalent immediate Int content before structural recursion.
The kernel-tier scalar test was observed failing before correction and passes
afterward. The Complex exact-power discriminator passes, including 10^100
components. Float signed-zero and Int/Float exactness boundary assertions pass.
The reentrant compiler regression was also observed red under the old receiver
heuristic and green with the body-kind check. Both issues are resolved; there is
no remaining powers blocker.
