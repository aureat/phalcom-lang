// area: collections
// spec: map-and-set.md §3; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Boundary: an empty Set — size 0, `includes` false, `remove` on an absent
// element is a no-op. Adversarial: adding a *value-equal* (not
// identical-instance) element is deduped by structural `==`/`hash`, not
// reference identity.

let s = Set.new()
System.print(s.size)
System.print(s.includes(1))
s.remove(1)
System.print(s.size)

s.add(1).add(1).add(1)
System.print(s.size)
System.print(s.includes(1))

// Value-equal strings built independently — dedup must key on structural
// equality, not object identity.
let a = "dup"
let b = "d" + "up"
s.add(a).add(b)
System.print(s.size)
