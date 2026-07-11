// area: concurrency
// spec: concurrency.md
// status: PENDING

let f = Future.async { 42 }
System.print(f.await)
