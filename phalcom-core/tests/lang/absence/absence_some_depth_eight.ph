// area: absence
// spec: 2-sixteen-byte-value.md §5
// status: PASS
// Option nesting supports depths beyond seven layers.

const v8 = Some(Some(Some(Some(Some(Some(Some(Some(None))))))))
System.print(v8)
