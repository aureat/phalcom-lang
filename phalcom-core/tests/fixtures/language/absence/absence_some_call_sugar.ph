// area: absence
// spec: values-and-absence.md §3.1; object-model.md; PDR-0033
// status: PASS
// `Some(value)` is canonical source syntax and lowers through the ordinary
// class-side `Some.call(_)` method. `Some.new(_)` remains compatibility-only.

System.print(Some(42))
System.print(Some.call(42))
System.print(Some.new(42))
System.print(Some(42) == Some.call(42))
System.print(Some(42).class == Some)
System.print(Some(42).is(Option))
