// area: absence
// spec: values-and-absence.md §3.1; PDR-0033
// status: PASS
// The generic VM preserves seven nested immediate `Some` layers.

let v7 = Some(Some(Some(Some(Some(Some(Some(None)))))))
System.print(v7)
let v6 = v7.match(some: |x| { x }, none: || { None })
System.print(v6)
let v5 = v6.match(some: |x| { x }, none: || { None })
System.print(v5)
let v4 = v5.match(some: |x| { x }, none: || { None })
System.print(v4)
let v3 = v4.match(some: |x| { x }, none: || { None })
System.print(v3)
let v2 = v3.match(some: |x| { x }, none: || { None })
System.print(v2)
let v1 = v2.match(some: |x| { x }, none: || { None })
System.print(v1)
System.print(v1.match(some: |x| { x }, none: || { None }))
