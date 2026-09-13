// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: ready input wins immediately
const f1 = Future.value("winner immediate")
const f2 = Future.new()
const r1 = Future.race([f1, f2])
System.print("race ready: " + r1.await)

// Test 2: first pending completion wins
const cs1 = CompletionSource.new()
const cs2 = CompletionSource.new()
const r2 = Future.race([cs1.future, cs2.future])

cs2.tryResolve("second resolved first")
cs1.tryResolve("first resolved later")

System.print("race async: " + r2.await)

// Test 3: rejection wins equally
const cs3 = CompletionSource.new()
const cs4 = CompletionSource.new()
const r3 = Future.race([cs3.future, cs4.future])

cs3.tryReject(Error.new("rejected winner"))
cs4.tryResolve("fulfilled loser")

try {
  r3.await
} catch e {
  System.print("race rejected: " + e.message)
}
