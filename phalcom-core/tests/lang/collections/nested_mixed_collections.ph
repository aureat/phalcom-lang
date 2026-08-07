// area: collections
// spec: map-and-set.md §2/§3; tuple-and-range.md §1/§2; U-COLLTYPES plan.md; ADR-0039
// status: PASS
// Cross-kind nesting: a List of Tuples, a Map whose values are Lists, and a
// Tuple whose elements are a Range and a Set — every combinator composes
// with every other native collection with no special-casing.

const pairs = List.new()
pairs.add(Tuple.__fromList(List.new().add(1).add(2)))
pairs.add(Tuple.__fromList(List.new().add(3).add(4)))
System.print(pairs.size)
System.print(pairs.at(0).at(0))
System.print(pairs.at(1).at(1))
System.print(pairs)

const byGroup = Map.new()
byGroup.at("evens", put: List.new().add(2).add(4))
byGroup.at("odds", put: List.new().add(1).add(3))
System.print(byGroup.at("evens"))
System.print(byGroup.at("odds").size)

const mixed = Tuple.__fromList(List.new().add(Range.new(1, 3, true)).add(Set.new().add(1).add(2)))
System.print(mixed.size)
System.print(mixed.at(0).toList)
System.print(mixed.at(1).size)
System.print(mixed.at(1).includes(2))
