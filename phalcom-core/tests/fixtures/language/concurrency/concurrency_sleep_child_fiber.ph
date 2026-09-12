// area: concurrency
// spec: stdlib/reactor.md; concurrency.md
// status: PASS

const fiber = Fiber.new || {
  System.print("child: sleeping")
  System.sleep(10).await
  System.print("child: awake")
  "done"
}
System.schedule(fiber)
System.runScheduled()
System.print("child fiber done: " + fiber.isDone.toString)
