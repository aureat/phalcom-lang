// area: errors
// spec: annotations-data.md §"@data" (Hazards: "with(...) field-order
// sensitivity" / "with(...) is shallow")
// status: PASS
// U-ANNOT-LAYOUT step 6: `with(...)` is a shallow functional update — a
// `with(...)`-produced instance shares heap-object field values with its
// source. Mutating a `List`-valued field through the copy must be visible
// through the source, proving no deep clone snuck in.

@data
class Basket {
  mut _items
  mut _owner
}

let items: List<String> = []
items.append("apple")

let source = Basket.new(items: items, owner: "alice")
let copy = source.with(items: None, owner: "bob")

copy.items.append("banana")

System.print(source.items)
System.print(copy.items)
System.print(source.owner)
System.print(copy.owner)
