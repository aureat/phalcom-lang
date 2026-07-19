// Range does not implement [] — it has no at(_) method,
// so r[0] correctly doesNotUnderstand on the [] selector.
const r = Range.new(1, 5, true)
r[0]
