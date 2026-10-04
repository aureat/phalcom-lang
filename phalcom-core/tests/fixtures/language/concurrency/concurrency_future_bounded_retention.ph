// area: concurrency
// spec: CONC002.C4.P1
// status: PASS

let f = Future.new()
let mut detachedRuns = 0
let mut activeRuns = 0

let mut i = 0
while (i < 50) {
  let sub = f.subscribeReady(|| { detachedRuns = detachedRuns + 1 })
  sub.detach()
  i = i + 1
}

let activeSub = f.subscribeReady(|| { activeRuns = activeRuns + 1 })

f.settleValue("done")

System.print(f.await)
System.print(detachedRuns)
System.print(activeRuns)
