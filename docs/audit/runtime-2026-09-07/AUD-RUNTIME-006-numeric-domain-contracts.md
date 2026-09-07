# AUD-RUNTIME-006 — Numeric domain dispatch violates quotient and remainder contracts

## Classification

- Severity: High for arithmetic correctness.
- Confidence: Confirmed through valid-arity public VM sends to shipping native methods.
- Audited HEAD: `94b9f14361333dfda06cd8abd792f672d12db06a`, with unrelated working-tree changes preserved.
- Scope: Numeric runtime semantics. Source compilation was not exercised in this checkpoint.

## Finding

Two native arithmetic paths substitute host floating-point operations for the specified mathematical operations. Mixed floor division rounds an Int operand before computing an exact Int result. Float remainder uses truncating host remainder instead of the specified floor remainder.

The normative [numeric tower](../../spec/library/numbers/numeric-tower.md), sections 5.2 and 6.2, explicitly requires exact represented-value floor division, including mixed operands beyond 2^53, and floor remainder with the divisor's sign. These are implementation divergences from those rules; specification status is not evidence of implementation completion.

## Executed evidence

[Probe source](evidence/numeric_contract_probe.rs) and [completed output](evidence/numeric-contract-output.txt) use `VM::new_native`, ordinary selectors and correctly shaped arguments. No malformed bytecode, panic injection or resource-exhaustion test is involved.

| Operation | Required result | Observed result |
| --- | --- | --- |
| `-5.0 % 3.0` | `1.0` | `-2.0` |
| `5.0 % -3.0` | `-1.0` | `2.0` |
| `9007199254740993 ~/ 1.0` | `9007199254740993` | `9007199254740992` |
| `9007199254740993 ~/ 1` control | `9007199254740993` | `9007199254740993` |

The notation describes the operands and selectors sent; it is not a claim that these source expressions were compiled in the probe. Exit 0 means the diagnostic completed, not that regression assertions passed.

## Producer and consumer trace

In `phalcom-core/src/primitive/number.rs`:

- `promote_pair` converts mixed Int/Float operands with `BigInt::to_f64` and selects `PromotedPair::Float`.
- `number_floor_div` uses that promotion through `cooperative_pair`, then calculates `(a / b).floor()` and converts the rounded result back to BigInt. The original exact integer is already lost.
- `number_mod` returns `Value::float(a % b)` for the Float branch. The host operation retains the dividend's sign for these nonzero cases.
- The integer floor-division control takes a separate integer branch and preserves the sample value.

## Underlying design problem

Promotion is operation-dependent. A helper suitable for Float-result arithmetic is also used for an operation whose result and calculation must remain exact. Remainder additionally needs an explicit language algorithm rather than directly inheriting the host operator.

Keep these two corrections distinct within the shared numeric-domain contract: replacing mixed promotion alone does not correct float remainder, and patching remainder signs does not establish correctly rounded exact remainder across all operands.

## Recommended direction and verification

Give exact floor division a represented-value path that retains integer precision and decodes finite Float operands exactly. Give float remainder its specified exact calculation and final rounding, including signed-zero and non-finite rules. Preserve the existing small-int/large-int representation and controlled numeric errors.

Future regression coverage should include both signs, 2^53 neighbors, exact finite Float ratios, zero signs, non-finite operands and configured limits. Add compiler-to-runtime coverage to establish ordinary-source behavior; this checkpoint verifies the native dispatch boundary only.

No implementation fixes or broad validation were performed. Nearby fixed-width overflow and mixed comparison paths are separate unexecuted leads in [S3](AUD-RUNTIME-S3-new-numeric-and-string-leads.md).
