// area: errors
// spec: annotations-data.md §"@data"
// status: PASS
// U-ANNOT-LAYOUT step 6: `@data` derives, from declared fields alone, a
// `new` constructor (reusing `@construct`'s own-fields-only shape),
// structural `==`, a consistent `hash`, a default `toString`, and a shallow
// functional-update `with(...)`.

@data
class Money {
  var _cents
  var _currency
}

let a = Money.new(cents: 500, currency: "USD")
let b = Money.new(cents: 500, currency: "USD")
let c = Money.new(cents: 100, currency: "USD")

System.print(a.toString)
System.print(a == b)
System.print(a == c)
System.print(a.hash == b.hash)

let d = a.with(cents: 700, currency: None)
System.print(d.toString)
System.print(a.toString)
