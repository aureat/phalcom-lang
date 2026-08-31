// area: collections
// spec: D.1 §14-15
// status: PASS

let fold_calls = 0
System.print([].fold(initial: 7, using: |acc, x| {
  fold_calls = fold_calls + 1
  acc + x
}))
System.print(fold_calls)
System.print([1, 2, 3].fold(initial: 10, using: |acc, x| { acc - x }))

System.print([].reduce(using: |a, b| { a + b }).isNone)

let reduce_calls = 0
const singleton = [42].reduce(using: |a, b| {
  reduce_calls = reduce_calls + 1
  a + b
})
System.print(singleton.unwrapOr(-1))
System.print(reduce_calls)
System.print([1, 2, 3].reduce(using: |a, b| { a + b }).unwrapOr(-1))
System.print([None].reduce(using: |a, b| { a }).isSome)
