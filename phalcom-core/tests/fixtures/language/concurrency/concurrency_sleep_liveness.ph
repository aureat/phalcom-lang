// area: concurrency
// spec: stdlib/reactor.md; concurrency.md
// status: PASS

System.print("before sleep")
System.sleep(20).await
System.print("after sleep")
