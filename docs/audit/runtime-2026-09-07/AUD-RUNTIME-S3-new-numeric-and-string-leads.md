# AUD-RUNTIME-S3 — New numeric and string leads

Checkpoint: 2026-09-07, HEAD `94b9f14361333dfda06cd8abd792f672d12db06a`. This bounded slice adds new investigations rather than reopening S2. Source was inspected; only the four arithmetic cases in [006](AUD-RUNTIME-006-numeric-domain-contracts.md) were executed.

## L19 — Fixed-width floor division and remainder edge

**Status: unresolved execution; concrete source-level defect candidate.**

`primitive/number.rs::floor_div_i64` evaluates `a / b` and `a % b`; `floor_mod_i64` evaluates `a % b`. Their native callers reject zero but do not special-case the minimum signed integer divided by -1. Both immediate integers select `PromotedPair::Int` in `promote_pair`. The host signed operations cannot handle that pair, whereas Phalcom's exact Int quotient should promote and its remainder should be zero. In contrast, `number_negate` explicitly promotes the minimum integer.

Next: verify the public selector path and source construction of that boundary, then distinguish host failure from controlled numeric-limit behavior. No boundary execution or panic reproducer was run. This is arithmetic result representability, distinct from bytecode operand narrowing in issue 001.

## L20 — Mixed relational comparison rounds before ordering

**Status: unresolved execution; implementation/spec mismatch candidate.**

`primitive/number.rs::number_lt`, `number_le`, `number_gt` and `number_ge` call `promote_pair`, then compare the Float pair with host operators. Mixed integers beyond 2^53 may collapse onto the neighboring Float value. The numeric tower's section 8 requires exact numeric comparisons. The quotient probe confirms conversion loss in the shared helper, but no relational result was executed here.

Next: compare `9007199254740993` with Float `9007199254740992.0` through all four relations and compare the result with equality and total comparison. Do not assume these other protocols use the same implementation.

## L21 — String byte accessor result shape

**Status: unresolved caller contract.**

`primitive/string.rs::string_raw_byte_at` declares `(Int) -> Option<Int>` in native metadata but returns bare `Value::int` on a successful read and None otherwise. `universe/primitives.rs` registers this internal accessor. `primitive/option.rs::wrap_some` establishes that successful Some construction is an explicit operation in the runtime representation.

Next: locate the active bootstrap declaration and callers, establish whether they expect an optional wrapper or a bare cursor-like value, and compare exported native metadata with the actual result. A declaration/implementation discrepancy is visible, but no public source failure is established. The internal visibility boundary matters. Do not infer caller behavior from the historical review comment at the top of string.rs.

## L22 — Immediate arithmetic goes through BigInt temporaries

**Status: unresolved performance significance; confirmed source operations.**

`cooperative_pair` calls `extract_int` to recognize operands; `promote_pair` calls `expect_number_big_or_float`, which calls `extract_int` again before selecting the immediate Int branch. `extract_int` constructs BigInt from immediate integers. This work occurs even when the final result fits an immediate integer.

Next: inspect optimized code or measure allocations and timings for ordinary immediate arithmetic before assigning an optimization priority. BigInt construction alone does not prove a heap allocation on every call. Keep this separate from the bilateral dispatch-cache lead in S2.

## Counterevidence: UTF-8 slice admission

**Status: disproved for the inspected bounds/boundary suspicion.**

`primitive/string.rs::string_raw_slice` checks start/end order, byte-length bounds and both UTF-8 character boundaries before Rust string slicing. No unchecked UTF-8 slicing defect was established. This is source inspection, not exhaustive testing of string APIs or index conversion.

## Next slice

Resolve L20 and L21 before broadening numeric coverage. Retain L19 as an unexecuted boundary question and L22 as an unmeasured cost. The integrated assessment remains provisional B; these findings do not constitute release certification.
