// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: empty list resolves to empty list
const empty = Future.all([])
System.print("empty size: " + empty.await.size.toString)

// Test 2: already fulfilled inputs preserve order
const f1 = Future.value(10)
const f2 = Future.value(20)
const f3 = Future.value(30)
const allFulfilled = Future.all([f1, f2, f3])
const res1 = allFulfilled.await
System.print("res1: " + res1.at(0).toString + ", " + res1.at(1).toString + ", " + res1.at(2).toString)

// Test 3: asynchronous pending inputs resolve in order
const a1 = Future.async(|| { 1 })
const a2 = Future.async(|| { 2 })
const a3 = Future.async(|| { 3 })
const allAsync = Future.all([a1, a2, a3])
const res2 = allAsync.await
System.print("res2: " + res2.at(0).toString + ", " + res2.at(1).toString + ", " + res2.at(2).toString)

// Test 4: duplicates maintain distinct logical entries
const d1 = Future.value(99)
const allDups = Future.all([d1, d1])
const res3 = allDups.await
System.print("res3 size: " + res3.size.toString + ", " + res3.at(0).toString + ", " + res3.at(1).toString)

// Test 5: first observed rejection rejects output
const okFut = Future.async(|| { 100 })
const errFut = Future.async(|| { throw Error.new("failure in all") })
const okFut2 = Future.async(|| { 200 })
const allRejected = Future.all([okFut, errFut, okFut2])
try {
  allRejected.await
} catch e {
  System.print("all rejected: " + e.message)
}
