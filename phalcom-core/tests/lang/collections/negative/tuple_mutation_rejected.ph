// area: collections
// spec: tuple-and-range.md §1; U-COLLTYPES plan.md Phase 2; ADR-0039
// status: NEGATIVE
// Adversarial: `Tuple` exposes no mutation selector (immutability, RG-2's
// Tuple twin) — attempting `at(_, put:)` (List's mutator shape) is a plain
// dNU, never a silent mutation.

let t = Tuple.fromList(List.new().add(1).add(2))
t.at(0, put: 9)
