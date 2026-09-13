// area: concurrency
// spec: CONC002.C4.P1
// status: PASS

const f = Future.new()
let detachedRuns = 0
let activeRuns = 0

let i = 0
while (i < 50) {
  const sub = f.subscribeReady(|| { detachedRuns = detachedRuns + 1 })
  sub.detach()
  i = i + 1
}

const activeSub = f.subscribeReady(|| { activeRuns = activeRuns + 1 })

f.settleValue("done")

System.print(f.await)
System.print(detachedRuns)
System.print(activeRuns)
