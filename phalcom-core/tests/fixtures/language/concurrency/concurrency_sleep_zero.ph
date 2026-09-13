// area: concurrency
// spec: stdlib/reactor.md; concurrency.md
// status: PASS

const sleepFut = System.sleep(Duration.milliseconds(0))
System.print("sleep0 isReady immediately: " + sleepFut.isReady.toString)
sleepFut.await
System.print("sleep0 isReady after await: " + sleepFut.isReady.toString)
