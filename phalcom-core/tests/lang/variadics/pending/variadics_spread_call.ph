// area: variadics
// spec: F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
// status: PENDING
// Static outgoing `*` forwarding returns correctly from `add`, but currently
// leaves the outer receiver window misbound for `System.print`.

class Adder {
  add(_ a, _ b, _ c) {
    return a + b + c;
  }
}
const args = [1, 2, 3]
System.print(Adder.new().add(*args))
