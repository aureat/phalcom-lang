// area: variadics
// spec: F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
// status: PASS
// Dynamic-pack scratch locals must preserve every already-evaluated enclosing
// operand: receiver, binary LHS, prior argument, and prior List element.

class Adder {
  add(_ a, _ b, _ c) => a + b + c
}

class Pairer {
  pair(_ left, _ right) => "\(left):\(right)"
}

const args = [1, 2, 3]

System.print(Adder.new().add(*args))
System.print(100 + Adder.new().add(*args))
System.print(Pairer.new().pair("kept", Adder.new().add(*args)))

const values = ["kept", Adder.new().add(*args)]
System.print(values[0])
System.print(values[1])
