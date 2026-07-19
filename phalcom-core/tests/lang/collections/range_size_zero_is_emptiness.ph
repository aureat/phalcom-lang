// area: collections
// spec: tuple-and-range.md §2
// status: PASS
// Wren sequence/is_empty.wren's spirit, over `Range` (`Range` carries no
// dedicated `isEmpty` selector — not in tuple-and-range.md §2's table —
// so `size == 0` is the current emptiness test): a degenerate exclusive
// single-point range (`1...1`) is empty; the matching inclusive range
// (`1..1`) is not.

const empty = Range.new(1, 1, false)
System.print(empty.size == 0)
const full = Range.new(1, 1, true)
System.print(full.size == 0)
