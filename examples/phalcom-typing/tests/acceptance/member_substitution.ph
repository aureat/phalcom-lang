import "../../src/typing" as Typing

class Pair<A, B> {
  const _first: A
  const _second: B

  first -> A { return _first }
  second -> B { return _second }
}

const bareFirst = Pair.methodFor(#first).unwrap
const appliedFirst = Pair<String, Int>.methodFor(#first).unwrap
const appliedSecond = Pair<String, Int>.methodFor(#second).unwrap

assert(bareFirst.returnType == Pair.typeParameters.first)
assert(appliedFirst.returnType == String)
assert(appliedSecond.returnType == Int)
assert(appliedFirst.selector == bareFirst.selector)
assert(appliedFirst.executable === bareFirst.executable)
