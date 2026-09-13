// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: source ready before call wins immediately
const ready = Future.value(42)
const t1 = ready.timeout(Duration.milliseconds(100))
System.print("ready timeout: " + t1.await.toString)

// Test 2: source resolves before timeout
const cs = CompletionSource.new()
const t2 = cs.future.timeout(Duration.milliseconds(500))
cs.tryResolve(99)
System.print("resolved before timeout: " + t2.await.toString)

// Test 3: deadline wins over pending source
const slowCs = CompletionSource.new()
const t3 = slowCs.future.timeout(Duration.milliseconds(10))
try {
  t3.await
} catch e {
  System.print("timeout occurred: " + e.is(TimeoutError).toString)
  System.print("timeout is Error: " + e.is(Error).toString)
}
