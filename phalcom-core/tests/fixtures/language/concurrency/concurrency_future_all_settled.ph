// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: empty list resolves to empty list
let empty = Future.allSettled([])
System.print("empty size: " + empty.await.size.toString)

// Test 2: mixed fulfilled and rejected inputs
let ok1 = Future.value(10)
let err1 = Future.error(Error.new("rejected 1"))
let ok2 = Future.async(|| { 20 })
let err2 = Future.async(|| { throw Error.new("rejected 2") })

let settled = Future.allSettled([ok1, err1, ok2, err2])
let res = settled.await

System.print("settled size: " + res.size.toString)

let mut i = 0
while (i < res.size) {
  let item = res.at(i)
  item.match(
    ok: |v| { System.print("index " + i.toString + ": Ok(" + v.toString + ")") },
    err: |e| { System.print("index " + i.toString + ": Err(" + e.message + ")") }
  )
  i = i + 1
}
