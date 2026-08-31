// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §11-15
// status: PASS
// A fixed-prefix positional rest (`format(fmt, *args)`) exercises capture
// after the fixed prefix. Called with one and three residual arguments.

class Formatter {
  format(_ fmt, *args) {
    return args.size
  }
}
const f = Formatter.new()
System.print(f.format("x", "a"))
System.print(f.format("x", "a", "b", "c"))
