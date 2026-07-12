// area: collections
// spec: lexical-structure.md §7; ADR-0032 §1, §3.2; DEC-COLL-A
// status: PENDING — graduates when U-COLLTYPES lands the native `Tuple` arm
// The tuple `(a, b)` arm desugars to `Tuple.fromList(List.new().add(a).add(b))`.
// The parser lowering ships in U-COLL, but the native `Tuple` class is built by
// the collection-runtime unit (U-COLLTYPES), so this currently fails with
// "Undefined variable 'Tuple'". These assertions pin the intended runtime:
// a `Tuple` distinct from `List` (the typing surface requires the distinction).

let t = (3, 4)
System.print(t.size)
System.print(t.at(0))
System.print(t.class)
