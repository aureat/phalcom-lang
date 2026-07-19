// area: errors
// spec: annotations-data.md §"@data"
// status: PASS
// U-ANNOT-LAYOUT step 6: `@data` derives, from declared fields alone, a
// `new` constructor (reusing `@construct`'s own-fields-only shape),
// structural `==`, a consistent `hash`, a default `toString`, and a shallow
// functional-update `with(...)`.

@data
class Money {
  _cents
  _currency
}

const a = Money.new(cents: 500, currency: "USD")
const b = Money.new(cents: 500, currency: "USD")
const c = Money.new(cents: 100, currency: "USD")

System.print(a.toString)
System.print(a == b)
System.print(a == c)
System.print(a.hash == b.hash)

const d = a.with(cents: 700, currency: None)
System.print(d.toString)
System.print(a.toString)
