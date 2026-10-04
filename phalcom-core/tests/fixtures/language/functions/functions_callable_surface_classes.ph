// area: functions
// spec: callable-surface-object-model-and-parameter-foundations.md §2, §6–7
// status: PASS
// User-defined call methods remain callable without inheriting Function, while
// each VM-backed callable representation reports its own public class.

class CallableObject {
  call(_ value) { value + 1 }
}

let object = CallableObject.new()
let method = object.methodFor(#call(_))
let bound = method.bind(object)
let family = &CallableObject.call...;

System.print(object(4))
System.print(object.is(Function))
System.print(|| { 0 }.class == Closure)
System.print(method.class == Method)
System.print(bound.class == BoundMethod)
System.print(family.class == Family)
