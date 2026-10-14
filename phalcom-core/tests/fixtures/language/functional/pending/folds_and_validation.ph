// status: PENDING
from universe.functional import Chain, FoldFactory, Folds, Validation
let source = Chain<Int>::Empty.prepend(3).prepend(2).prepend(1)
let combined = FoldFactory.count<Int>().zipWith(FoldFactory.sum<Int>(), combine: |count, sum| { count.toString + ":" + sum.toString })
System.print(combined.run(source))
System.print(FoldFactory.sum<Int>().filter(|n| { n > 1 }).run(source))
System.print(FoldFactory.collect<Int>().run(source) == source)
System.print(FoldFactory.first<Int>().run(source).unwrap)
System.print(FoldFactory.last<Int>().run(source).unwrap)
let mut calls = 0
let result = Folds.traverseResult<Chain<Int>, Int, String>(source, using: |n| {
  calls = calls + 1
  if n == 2 { Result<Int, String>::Error("bad") } else { Result<Int, String>::Ok(n) }
})
System.print(result.err.unwrap)
System.print(calls)
let validated = Folds.validateAll<Chain<Int>, Int, String>(source, using: |n| {
  if n % 2 == 0 { Validation<Int, String>::Valid(n) } else { Validation<Int, String>::Invalid(n.toString) }
})
System.print(validated.toResult.err.unwrap.size)
System.print(validated.toResult.err.unwrap.toChain.fold(initial: "", using: |s, e| { s + e }))
