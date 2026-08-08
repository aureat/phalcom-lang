// area: variadics
// spec: U9-implementation-spec.md §2, §6
// status: PASS
// A fixed-prefix variadic (`format(fmt, *args)`, `F = 1`) exercises the call
// prologue's non-zero-`F` math: `receiver_idx + 1 + F` must land exactly past
// the fixed prefix before the trailing args are collapsed into the rest
// `List`. Called with 1 and 3 trailing args.

class Formatter {
  format(_ fmt, *args) {
    return args.size
  }
}
const f = Formatter.new()
System.print(f.format("x", "a"))
System.print(f.format("x", "a", "b", "c"))
