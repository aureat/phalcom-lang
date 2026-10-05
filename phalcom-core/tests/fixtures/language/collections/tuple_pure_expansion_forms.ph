// area: collections
// spec: collections/02-tuples.md §3, §12; collections/06-rest-spread-and-pack-operators.md §2.4
// status: PASS
// Pure expansion Tuples are already unambiguous, so their terminal comma is
// optional. Compilation must preserve the same tuple value in either spelling.

let positional = (1, 2)
let labels = (name: "Phalcom")
let complete = (3, version: 1)

let a = (*positional)
let b = (**labels)
let c = (***complete)

System.print(a == positional)
System.print(b == labels)
System.print(c == complete)
