// area: concurrency
// spec: concurrency.md; system.md
// status: PASS
// Raw scheduler dequeue is an internal runtime seam. Public code can admit
// work and run the scheduler, but cannot pop a queued Fiber for manual try().

System.print(System.respondsTo(Symbol.new("nextScheduled")))
