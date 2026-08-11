// area: absence
// spec: values-and-absence.md §3.1-3.2; PDR-0033
// status: PASS
// Each immediate wrapper depth remains distinct. `match` peels exactly one
// layer, including when the payload is another Option.

const v = Some(Some(Some(None)))
System.print(None)
System.print(Some(None))
System.print(v)
System.print(None == Some(None))
System.print(Some(None) == v)
System.print(v.class == Some)
System.print(v.isA(Option))
System.print(v.match(some: |x| { x }, none: || { None }))
