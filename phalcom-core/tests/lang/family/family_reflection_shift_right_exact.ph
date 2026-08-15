// area: family
// spec: docs/spec/callables/reflection.md §§1–4
// status: PASS
// `>>` is ordinary polymorphic reflection on Behavior. Exact selector Symbols
// yield one effective Method (or None), while integer `>>` keeps shift meaning.

class Reflector {
  value { 10 }
  value() { 11 }
  double(_ n) { n * 2 }
}
class Empty {}

const receiver = Reflector.new()
const getter = Reflector >> #value
const nullary = Reflector >> #value()
const double = Reflector >> #double(_)
System.print(getter.bind(receiver)())
System.print(nullary.bind(receiver)())
System.print(double.bind(receiver)(6))
System.print((Empty >> #missing()) == None)
System.print(16 >> 2)
