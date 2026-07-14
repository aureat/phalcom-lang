// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// SkipView raises Error when count is negative

try {
  SkipView.new([1, 2, 3], -1)
  System.print("ERROR: no exception")
} on (e) {
  System.print(e.class.name)
}
