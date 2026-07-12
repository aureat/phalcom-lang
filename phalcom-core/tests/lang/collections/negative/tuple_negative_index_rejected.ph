// area: collections
// spec: tuple-and-range.md §1; U-COLLTYPES plan.md Phase 2; ADR-0039
// status: NEGATIVE
// Adversarial: unlike a past-arity index (total, `None` — see
// `tuple_at_out_of_range.ph`), a *negative* index is not a valid `Number`
// index shape at all — `at(_)` raises a Type error rather than silently
// wrapping/clamping (mirrors List/Map's identical `expect_index` guard).

let t = Tuple.fromList(List.new().add(1).add(2))
t.at(-1)
