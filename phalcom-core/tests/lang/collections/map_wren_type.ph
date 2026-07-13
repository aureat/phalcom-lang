// area: collections
// spec: map-and-set.md §2; object-model.md §8; ADR-0039
// status: PASS
// Adapted from wren/test/core/map/type.wren. Wren's `is` type-test operator
// is not yet in Phalcom (U-IS is planned, not landed), so membership goes
// through `isA(_)` (landed U-CORE-1) instead; Wren's `.type` (a class
// getter) is Phalcom's `.class`.

System.print(Map.new().isA(Map))
System.print(Map.new().isA(Object))
System.print(Map.new().isA(Bool))
System.print(Map.new().class == Map)
