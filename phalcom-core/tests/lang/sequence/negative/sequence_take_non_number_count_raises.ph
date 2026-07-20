// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// TakeView raises Error when count is not a Number

try {
  TakeView.new([1, 2, 3], "invalid")
  System.print("ERROR: no exception")
} on (Error) { e =>
  System.print(e.class.name)
}
