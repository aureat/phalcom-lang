// area: concurrency
// spec: CONC002.C4.P1
// status: PASS

let f = Future.new()
let mut ran1 = false
let mut ran2 = false

let sub1 = f.subscribeReady(|| { ran1 = true })
let sub2 = f.subscribeReady(|| { ran2 = true })

System.print(sub1.isActive)
System.print(sub1.isDetached)

System.print(sub1.detach())
System.print(sub1.isActive)
System.print(sub1.isDetached)
System.print(sub1.detach())

f.settleValue(123)

Future.async(|| { () }).await

System.print(f.await)
System.print(ran1)
System.print(ran2)

let mut ran3 = false
let sub3 = f.subscribeReady(|| { ran3 = true })
System.print(sub3.detach())
Future.async(|| { () }).await
System.print(ran3)
