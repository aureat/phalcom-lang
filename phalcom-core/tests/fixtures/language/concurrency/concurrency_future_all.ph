// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: empty list resolves to empty list
let empty = Future.all([])
System.print("empty size: " + empty.await.size.toString)

// Test 2: already fulfilled inputs preserve order
let f1 = Future.value(10)
let f2 = Future.value(20)
let f3 = Future.value(30)
let allFulfilled = Future.all([f1, f2, f3])
let res1 = allFulfilled.await
System.print("res1: " + res1.at(0).toString + ", " + res1.at(1).toString + ", " + res1.at(2).toString)

// Test 3: asynchronous pending inputs resolve in order
let a1 = Future.async(|| { 1 })
let a2 = Future.async(|| { 2 })
let a3 = Future.async(|| { 3 })
let allAsync = Future.all([a1, a2, a3])
let res2 = allAsync.await
System.print("res2: " + res2.at(0).toString + ", " + res2.at(1).toString + ", " + res2.at(2).toString)

// Test 4: duplicates maintain distinct logical entries
let d1 = Future.value(99)
let allDups = Future.all([d1, d1])
let res3 = allDups.await
System.print("res3 size: " + res3.size.toString + ", " + res3.at(0).toString + ", " + res3.at(1).toString)

// Test 5: first observed rejection rejects output
let okFut = Future.async(|| { 100 })
let errFut = Future.async(|| { throw Error.new("failure in all") })
let okFut2 = Future.async(|| { 200 })
let allRejected = Future.all([okFut, errFut, okFut2])
try {
  allRejected.await
} catch e {
  System.print("all rejected: " + e.message)
}
