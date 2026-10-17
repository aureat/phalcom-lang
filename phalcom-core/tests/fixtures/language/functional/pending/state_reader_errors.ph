// status: PENDING
from universe.functional import StateResult, StateStep, ReaderResult
let mut later = 0
let fail = StateResult<Int, Int, String>.new(|state| {
  StateStep<Int, Result<Int, String>>(state: state + 1, value: Result<Int, String>::Error("bad"))
})
let chained = fail.flatMap(|value| {
  later = later + 1
  StateResult<Int, Int, String>.pure(value + 10)
})
let failed = chained.run(5)
System.print(failed.state)
System.print(failed.value.err.unwrap)
System.print(later)
System.print(chained.transactional().run(5).state)
let recovered = fail.orElse(|error| { StateResult<Int, Int, String>.pure(error.rawByteCount) }).run(5)
System.print(recovered.state)
System.print(recovered.value.ok.unwrap)
let reader = ReaderResult<Int, Int, String>.new(|environment| {
  if environment < 0 { Result<Int, String>::Error("negative") } else { Result<Int, String>::Ok(environment) }
})
let pipeline = reader.flatMap(|value| { ReaderResult<Int, Int, String>.pure(value * 2) })
System.print(pipeline.run(3).ok.unwrap)
System.print(pipeline.run(-1).err.unwrap)
System.print(pipeline.local(|n| { n + 1 }).run(3).ok.unwrap)
