// area: concurrency
// spec: system.md §2 (Scheduler row); U-SCHED plan.md §6
// status: PASS
// A scheduled fiber does not run at the `System.schedule(_)` call site —
// only enqueued (`VM::ready_queue`) — so no side effect is observed until
// the next drain point (here, the implicit root-drive at program exit).

System.schedule(|| { System.print("scheduled") })
System.print("main")
