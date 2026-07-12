// area: errors
// spec: result.md §2
// status: PASS
// `Result`/`Ok`/`Err` combinators: `map`/`mapErr`/`andThen`/`isOk`/`isErr`/
// `unwrapOr`/`toString`, mirroring `Option`/`Some`/`None`.

System.print(Ok.new(2).map { n => n * 2 }.toString)
System.print(Err.new("bad").map { n => n * 2 }.toString)
System.print(Ok.new(2).mapErr { e => "wrapped:" + e }.toString)
System.print(Err.new("bad").mapErr { e => "wrapped:" + e }.toString)
System.print(Ok.new(2).andThen { n => Ok.new(n + 1) }.toString)
System.print(Err.new("bad").andThen { n => Ok.new(n + 1) }.toString)
System.print(Ok.new(2).isOk)
System.print(Ok.new(2).isErr)
System.print(Err.new("bad").isOk)
System.print(Err.new("bad").isErr)
System.print(Ok.new(2).unwrapOr(0))
System.print(Err.new("bad").unwrapOr(0))
