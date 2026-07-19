// area: concurrency
// spec: concurrency.md
// status: PENDING

const f = Future.async { 42 }
System.print(f.await)
