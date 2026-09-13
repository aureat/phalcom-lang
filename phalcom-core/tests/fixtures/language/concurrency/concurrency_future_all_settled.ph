// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: empty list resolves to empty list
const empty = Future.allSettled([])
System.print("empty size: " + empty.await.size.toString)

// Test 2: mixed fulfilled and rejected inputs
const ok1 = Future.value(10)
const err1 = Future.error(Error.new("rejected 1"))
const ok2 = Future.async(|| { 20 })
const err2 = Future.async(|| { throw Error.new("rejected 2") })

const settled = Future.allSettled([ok1, err1, ok2, err2])
const res = settled.await

System.print("settled size: " + res.size.toString)

let i = 0
while (i < res.size) {
  const item = res.at(i)
  item.match(
    ok: |v| { System.print("index " + i.toString + ": Ok(" + v.toString + ")") },
    err: |e| { System.print("index " + i.toString + ": Err(" + e.message + ")") }
  )
  i = i + 1
}
