# U-NUMBERS-04 — Float semantics, text, and keyed collections

## Outcome

Implement exact PDR-0027 Float protocol, IEEE edge behavior, strict constructors, canonical
rendering, and coherent Int/Float Map/Set keys.

## Write set

- `phalcom-core/src/primitive/{float.rs,int.rs,mod.rs}` and `core/core.ph` derivable Int methods.
- `phalcom-core/src/value/{mod.rs,render.rs}`: equality/hash/render.
- `phalcom-core/src/primitive/{map.rs,set.rs,mod.rs}`: key equivalence and `send_hash` contract.
- constructor and numeric-text helpers plus protocol/key/text tests.

## Steps

1. Install ten Float-native bindings from the protocol; implement Int identities/predicates in
   `core.ph`. Narrow finite Float directly to BigInt then normalize; reject NaN/infinity.
2. Implement IEEE binary64 arithmetic and comparisons. Keep public NaN unordered. Add a distinct
   internal numeric-key comparator/canonical hash so NaN lookup works in Map/Set without changing
   `==`.
3. Require Int from `hash`; remove PDR-0012's temporary integral-Float return acceptance. Test
   `equal => same hash` for integral Float values beyond `2^53`.
4. Implement strict string grammar, byte-offset failures, canonical Int/Float render, special
   spelling, signed zero preservation, and Float overflow-to-infinity as the spec requires.
5. Float power uses binary64. Keep `0 ** negative` as an explicit numeric failure before calling
   host power; preserve all other IEEE domain outcomes.

## Acceptance matrix

- NaN equality/order, signed zero, infinities, subnormal/overflow boundary, and `is*` selectors.
- exact results of `floor`, `ceil`, `truncated`, `rounded` around halves, ±2^53, 1e300.
- Map/Set retrieval with two NaNs, both zeroes, and equal Int/Float keys; user hash returning
  Float fails `#invalidHash`.
- constructor accepts/rejects table: `"42"`, `"+42"`, `" 42"`, `"0x2a"`, `"NaN"`,
  `"Infinity"`, `"-Infinity"`, `"+Infinity"`, bad underscores, overflow decimal.
- canonical text round-trip property for sampled finite f64 values and exact golden special values.
