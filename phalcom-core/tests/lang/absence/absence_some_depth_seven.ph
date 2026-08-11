// area: absence
// spec: values-and-absence.md §3.1; PDR-0033
// status: PASS
// The generic VM preserves seven nested immediate `Some` layers.

const v7 = Some(Some(Some(Some(Some(Some(Some(None)))))))
System.print(v7)
const v6 = v7.match(some: |x| { x }, none: || { None })
System.print(v6)
const v5 = v6.match(some: |x| { x }, none: || { None })
System.print(v5)
const v4 = v5.match(some: |x| { x }, none: || { None })
System.print(v4)
const v3 = v4.match(some: |x| { x }, none: || { None })
System.print(v3)
const v2 = v3.match(some: |x| { x }, none: || { None })
System.print(v2)
const v1 = v2.match(some: |x| { x }, none: || { None })
System.print(v1)
System.print(v1.match(some: |x| { x }, none: || { None }))
