// area: concurrency
// spec: CONC002.C4.P2
// status: PASS

// Test 1: Backoff.none returns immediately
const bNone = Backoff.none
bNone.waitBefore(0)
bNone.waitBefore(5)
System.print("none completed")

// Test 2: Backoff.fixed waits with delay
const bFixed = Backoff.fixed(10)
bFixed.waitBefore(0)
bFixed.waitBefore(1)
System.print("fixed completed")

// Test 3: Backoff.exponential
const bExp = Backoff.exponential(5, 50)
bExp.waitBefore(0)
bExp.waitBefore(1)
System.print("exp completed")
