// area: concurrency
// spec: concurrency.md
// status: PASS

const f = Future.async || { 42 }
System.print(f.await)
