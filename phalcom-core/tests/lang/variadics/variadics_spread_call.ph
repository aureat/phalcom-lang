// area: variadics
// spec: F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
// status: PASS
// Regression: a nested dynamic-pack send must not alias or overwrite the
// already-evaluated receiver window of the enclosing static send.

class Adder {
  add(_ a, _ b, _ c) {
    return a + b + c
  }
}

const args = [1, 2, 3]
System.print(Adder.new().add(*args))
