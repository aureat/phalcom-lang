// area: concurrency
// spec: U-SCHED implementation-spec.md §4
// status: PASS
// `System.nextScheduled` on an empty queue answers the `None` singleton, not
// a raise or the private `nil` sentinel (Invariant 4).

System.print(System.nextScheduled)
