// area: concurrency
// spec: CONC002.C4.P2
// status: NEGATIVE

const f = Future.new()
f.timeout(-5)
