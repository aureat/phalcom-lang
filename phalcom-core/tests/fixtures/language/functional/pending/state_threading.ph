// area: functional
// status: PENDING
// Deferred transitions run once per stage, threading state left to right.
from universe.functional import State, StateStep

let mut calls = 0
let increment = State<Int, Int>.new(|state| {
  calls = calls + 1
  StateStep<Int, Int>(state: state + 1, value: state)
})
let pipeline = increment.flatMap(|first| { increment.map(|second| { first + second }) })
System.print(calls)
let first = pipeline.run(10)
System.print(first.state)
System.print(first.value)
System.print(calls)
let second = pipeline.run(20)
System.print(second.state)
System.print(second.value)
System.print(calls)
System.print(State<Int, Int>.get().eval(7))
System.print(State<Int, Unit>.put(4).exec(9))
System.print(State<Int, Unit>.modify(|n| { n + 3 }).exec(9))
System.print(State<Int, String>.pure("same").eval(9))
