// area: values
// spec: values-and-absence.md §3.2; PDR-0033
// status: PASS
// Raw display rendering and the ordinary `toString` message agree for nested
// immediate Options, while the payload remains recursively rendered.

const value = Some(Some("payload"))
System.print(value)
System.print(value.toString)
System.print(Some(Some(None)).toString)
