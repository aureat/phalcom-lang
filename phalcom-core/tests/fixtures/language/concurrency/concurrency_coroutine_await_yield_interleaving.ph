// area: concurrency
// spec: concurrency.md §2; CONC002.C2.P2
// status: PASS
// A manually consumed Fiber retains its coroutine consumer across executor park/wake/resume.
// Pattern: yield -> pending await -> resume by executor -> yield -> return.

const fut = Future.new()

const worker = Fiber.new || {
  System.print("coroutine: started")
  Fiber.yield("yield-1")
  System.print("coroutine: awaiting future")
  const value = fut.await
  System.print("coroutine: resumed after await with " + value.toString)
  Fiber.yield("yield-2")
  System.print("coroutine: completing")
  "done"
}

// 1. Initial call -> yield-1
const y1 = worker.call()
System.print("caller received: " + y1.toString)

// 2. Second call -> starts pending await and parks
// Schedule a background task to settle fut
System.schedule(Fiber.new || {
  System.print("scheduler task: settling future")
  fut.settleValue(99)
})

// Now call worker again; worker awaits fut and parks, executor runs scheduled task which settles fut,
// executor resumes worker, worker yields "yield-2" back to manual consumer!
const y2 = worker.call()
System.print("caller received: " + y2.toString)

// 3. Third call -> worker completes and returns "done"
const y3 = worker.call()
System.print("caller received: " + y3.toString)
System.print("worker isDone: " + worker.isDone.toString)
