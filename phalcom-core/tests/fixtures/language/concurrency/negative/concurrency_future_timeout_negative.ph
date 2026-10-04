// area: concurrency
// spec: CONC002.C4.P2
// status: NEGATIVE

let f = Future.new()
f.timeout(Duration.milliseconds(-5))
