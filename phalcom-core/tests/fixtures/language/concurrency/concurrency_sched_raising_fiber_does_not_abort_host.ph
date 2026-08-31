// area: concurrency
// spec: U-SCHED plan.md §2 rubric (fiber_try capture-not-propagate), §6
// status: PASS
// The first scheduled fiber raises uncaught; the root-drive pump resumes it
// via `fiber_try` (capture, not propagate), so the second scheduled fiber
// still runs and the host program still exits cleanly.

System.schedule(|| { throw Error.new() })
System.schedule(|| { System.print("second-ran") })
System.print("main-exits-cleanly")
