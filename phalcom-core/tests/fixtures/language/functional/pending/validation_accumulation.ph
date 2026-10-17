// area: functional
// status: PENDING
from universe.functional import Validation

let good = Validation<Int, String>::Valid(2)
let bad = Validation<Int, String>::Invalid("left")
let rightBad = Validation<Int, String>::Invalid("right")
let mut values = 0
let mut errors = 0
let combine = |a: Int, b: Int| {
  values = values + 1
  a + b
}
let accumulate = |a: String, b: String| {
  errors = errors + 1
  a + ";" + b
}
System.print(good.zipWith(good, combine: combine, errors: accumulate).toResult.ok.unwrap)
System.print(bad.zipWith(good, combine: combine, errors: accumulate).toResult.err.unwrap)
System.print(good.zipWith(rightBad, combine: combine, errors: accumulate).toResult.err.unwrap)
System.print(bad.zipWith(rightBad, combine: combine, errors: accumulate).toResult.err.unwrap)
System.print(values)
System.print(errors)
System.print(good.map(|n| { n * 3 }).toResult.ok.unwrap)
System.print(bad.mapErr(|message| { message + "!" }).toResult.err.unwrap)
System.print(Validation<Int, String>.fromResult(Result<Int, String>::Ok(9)).isValid)
System.print(bad.fold(valid: |n| { n.toString }, invalid: |message| { message }))
