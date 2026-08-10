// area: collections
// spec: collections/02-tuples.md §3, §12; collections/06-rest-spread-and-pack-operators.md §2.4
// status: PASS
// Pure expansion Tuples are already unambiguous, so their terminal comma is
// optional. Compilation must preserve the same tuple value in either spelling.

const positional = (1, 2)
const labels = (name: "Phalcom")
const complete = (3, version: 1)

const a = (*positional)
const b = (**labels)
const c = (***complete)

System.print(a == positional)
System.print(b == labels)
System.print(c == complete)
