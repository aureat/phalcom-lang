// status: PENDING
from universe.functional import Chain, Additive, Folds, Writer, Journal, Endo
let source = Chain<Int>::Empty.prepend(3).prepend(2).prepend(1)
System.print(Folds.foldMap<Chain<Int>, Additive<Int>>(source, using: |n| { Additive<Int>(n) }).value)
let a = Journal<String>::Entry("a")
let b = Journal<String>::Entry("b")
let c = Journal<String>::Entry("c")
System.print(a.combine(b).combine(c) == a.combine(b.combine(c)))
System.print(a.combine(b).combine(c).hash == a.combine(b.combine(c)).hash)
let logged = Writer<Journal<String>, Int>(value: 2, log: a).flatMap(|n| {
  Writer<Journal<String>, Int>(value: n + 3, log: b)
})
System.print(logged.value)
System.print(logged.log.fold(initial: "", using: |text, entry| { text + entry }))
System.print(logged.listen.value[0])
System.print(Writer<Journal<String>, Int>.pure(7).log.isEmpty)
let increment = Endo<Int>.new(|n| { n + 1 })
let double = Endo<Int>.new(|n| { n * 2 })
System.print(increment.combine(double).call(3))
