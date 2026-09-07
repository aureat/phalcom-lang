# AUDIT — Planned investigation of mixed ordering and string result shape

## Status

Documentation checkpoint, 2026-09-07. The user requested this record after reporting that continuation was again prevented. The exact warning trigger is unknown. No ordering or string-result probe was run in this continuation; the work completed before this checkpoint was targeted source navigation only. No runtime fixes or commits were made.

This document preserves the intended investigation of [S3 leads L20 and L21](AUD-RUNTIME-S3-new-numeric-and-string-leads.md). It does not promote either lead to an executed finding or change the provisional B assessment.

## L20 — Mixed numeric ordering

### What I was investigating

Whether comparisons between an exact Int and a Float lose information by converting the Int to Float before deciding their order.

The effective [numeric tower specification](../../spec/library/numbers/numeric-tower.md), section 8.2, requires finite numeric values to compare by exact mathematical value. A useful discriminator is Int `9007199254740993` versus Float `9007199254740992.0`: the integer is strictly greater, although converting it to binary64 can collapse that distinction.

### Source observations already obtained

- `primitive/number.rs::number_lt`, `number_le`, `number_gt` and `number_ge` use `promote_pair` and then host comparisons on the resulting pair.
- `promote_pair` converts mixed Int/Float operands through `BigInt::to_f64`.
- The previous quotient probe in [006](AUD-RUNTIME-006-numeric-domain-contracts.md) established conversion loss through this helper for floor division. It did not execute relational comparisons.
- `number_compare` also consumes a promoted pair, uses `partial_cmp` for Float and selects an Ordering result, including an `unordered` branch. Its registered selector is `compare(_)`.
- The specification separately defines `totalCompare(_)`. The relationship between those APIs was not traced. Their different names and handling of NaN must not be conflated.
- `value/mod.rs::Value::value_eq` was located but its implementation was not reviewed in this continuation. No claim about its precision follows from the relational helpers.

### How I intended to investigate

1. Refresh the relevant source diff and registration paths for the four relational selectors. Identify any compiler or dispatch special case that would select another implementation.
2. Send the same correctly shaped operand pair through `<`, `<=`, `>` and `>=`, then reverse the operands. Use native bootstrap for the native contract and ordinary finite values.
3. Include equal-value and integer-only controls. Expected results for the integer on the left are false, false, true, true, respectively.
4. Inspect equality and total comparison independently. Test consistency only after establishing their actual public selectors and implementations; do not substitute `compare(_)` for `totalCompare(_)`.
5. Add one compiled-source case if native dispatch confirms the discrepancy, so source reachability is supported by execution rather than inferred from registration.
6. Preserve the exact diagnostic source, observed values, command result and bootstrap tier in `evidence/`. Extend 006 if this is another consequence of the same promotion design, rather than inflating the issue count.

### Decision criteria

A wrong relational result through the selected shipping method confirms a runtime contract defect at that boundary. A compiled-source result additionally establishes source reachability. An alternative exact dispatch path could disprove the suspected source behavior while leaving the helper/API concern. Until those checks occur, L20 remains unresolved by execution.

## L21 — String byte accessor return shape

### What I was investigating

Whether the internal byte accessor's successful result agrees with its declared `Option<Int>` contract, and whether current library callers rely on a different result shape.

### Source observations already obtained

- `primitive/string.rs::string_raw_byte_at` declares `(Int) -> Option<Int>` in native metadata, returns bare `Value::int` for an in-bounds byte, and returns None otherwise.
- `universe/primitives.rs` registers the internal byte accessor.
- `primitive/option.rs::wrap_some` explicitly adds a Some layer in the runtime representation.
- The active-looking library file was located at `phalcom-core/core/universe/src/scalar/string.ph`. Search results show the declaration `_$byteAt(_ index: Int) -> Option<Int>` and calls inside `codePointAt` and nearby helpers.
- Those caller bodies and the bootstrap loading path were not yet read. Their actual unwrapping, arithmetic and error behavior remain unknown.
- Historical review comments in string.rs describe earlier failures. They are not current evidence; nearby corpus comments indicate some earlier string failures were fixed.

### How I intended to investigate

1. Verify that the located library file is loaded by current Universe bootstrap. Read the accessor declaration and the few methods that consume its results.
2. Determine whether successful callers explicitly unwrap Some, expect a bare integer, or pass through another conversion. Check the implementation's actual result contract against both native metadata and the library declaration.
3. Inspect native activation for any automatic result wrapping. Do not assume the primitive's raw return is necessarily the final caller-visible value without checking the boundary.
4. Use a small ASCII string and one multibyte UTF-8 string. Observe valid and out-of-range results through the appropriate internal or public path. If direct primitive inspection is used, label it as such; internal access is not an ordinary external source API.
5. Exercise a public consumer such as `codePointAt` only after establishing its accepted indexing and return rules. Separate an accessor mismatch from a caller defect or an intentional internal convention with stale metadata.
6. Retain actual result variants and values, not only printed text: a bare integer and Some(integer) must be distinguished by representation-aware observation.

### Decision criteria

If activation preserves a bare integer despite an effective optional contract, record a confirmed return-shape mismatch at that boundary. Promote public behavioral impact only when a supported caller demonstrates it. If activation wraps the value, the primitive-only suspicion is disproved. If bare values are intentional and callers agree, identify the precise declaration/metadata inconsistency instead of claiming broken public decoding.

## Boundaries and continuation

No broad tests, performance work, malformed-state probes or panic experiments are needed to answer these two questions. The intended checks use ordinary finite numbers and short valid strings. Their purpose is runtime contract verification.

Resume at the uncompleted caller and dispatch reads, not by repeating the prior quotient probe. Keep unrelated edits intact and remain audit-only. The latest user request was to write this document; no further investigation was performed for its preparation.
