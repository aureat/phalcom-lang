// area: functional
// status: PENDING
let mut calls = 0
let fallback = || { calls = calls + 1; 8 }
let present: Option<Int> = Option::Some(3)
let absent: Option<Int> = Option::None
System.print(present.unwrapOrElse(fallback))
System.print(calls)
System.print(absent.unwrapOrElse(fallback))
System.print(calls)
System.print(present.isSomeAnd(|n| { n > 2 }))
System.print(absent.isSomeAnd(|n| { calls = calls + 1; true }))
System.print(calls)
System.print(present.zipWith(Option<Int>::Some(4), combine: |a, b| { a + b }).unwrap)
System.print(absent.zipWith(present, combine: |a, b| { calls = calls + 1; a + b }).isNone)
System.print(calls)
let nested: Option<Option<Int>> = Option::Some(present)
System.print(nested.flatten<Int>().unwrap)
let optionalResult: Option<Result<Int, String>> = Option::Some(Result<Int, String>::Error("bad"))
System.print(optionalResult.transpose<Int, String>().err.unwrap)
let emptyResult: Option<Result<Int, String>> = Option::None
System.print(emptyResult.transpose<Int, String>().ok.unwrap.isNone)
System.print(present.xor(absent).unwrap)
System.print(present.xor(present).isNone)
System.print(absent.xor(absent).isNone)
System.print(present.okOrElse(|| { calls = calls + 1; "missing" }).ok.unwrap)
System.print(calls)
System.print(absent.okOrElse(|| { calls = calls + 1; "missing" }).err.unwrap)
System.print(calls)
let left: Result<Int, String> = Result::Error("first")
let right: Result<Int, String> = Result::Error("second")
System.print(left.zipWith(right, combine: |a, b| { calls = calls + 1; a + b }).err.unwrap)
System.print(calls)
