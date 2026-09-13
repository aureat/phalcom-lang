// area: concurrency
// spec: CONC002.C4.P2
// status: NEGATIVE

const f = Future.race([])
f.await
