// area: collections
// spec: U-CORE-5 as-built.md §3.3(b), §5.2; R-INV-5.2 (L7)
// status: PASS
// `each(_:)` holds its cursor locally, never on the collection, so nested
// (self-)iteration composes with no shared/global cursor (concurrency.md §1).

const outer = List.new()
outer.add(1)
outer.add(2)
const inner = List.new()
inner.add(10)
inner.add(20)
let total = 0
outer.each { x =>
  inner.each { y =>
    total = total + (x * y)
  }
}
System.print(total)
