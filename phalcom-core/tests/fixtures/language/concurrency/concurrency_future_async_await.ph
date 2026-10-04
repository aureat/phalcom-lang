// area: concurrency
// spec: concurrency.md
// status: PASS

let f = Future.async || { 42 }
System.print(f.await)
