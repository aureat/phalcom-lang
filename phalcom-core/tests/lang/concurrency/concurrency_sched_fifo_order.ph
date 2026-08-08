// area: concurrency
// spec: U-SCHED plan.md §6 (FIFO order)
// status: PASS
// Three scheduled fibers run in enqueue order once drained — `VM::ready_queue`
// is a plain FIFO (`VecDeque`), no priority.

System.schedule(|| { System.print("a") })
System.schedule(|| { System.print("b") })
System.schedule(|| { System.print("c") })
