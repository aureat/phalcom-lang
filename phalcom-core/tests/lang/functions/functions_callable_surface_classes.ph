// area: functions
// spec: callable-surface-object-model-and-parameter-foundations.md §2, §6–7
// status: PASS
// User-defined call methods remain callable without inheriting Function, while
// each VM-backed callable representation reports its own public class.

class CallableObject {
  call(_ value) { value + 1 }
}

const object = CallableObject.new()
const method = object.methodFor(#call(_))
const bound = method.bind(object)
const family = object::call

System.print(object(4))
System.print(object.isA(Function))
System.print(|| { 0 }.class == Closure)
System.print(method.class == Method)
System.print(bound.class == BoundMethod)
System.print(family.class == Family)
