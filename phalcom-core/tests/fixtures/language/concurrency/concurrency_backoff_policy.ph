// area: concurrency
// spec: CONC002.C4.P1
// status: PASS

const bNone = Backoff.none
System.print(bNone.kind)
System.print(bNone.delayFor(0))
System.print(bNone.delayFor(5))

const bFixed = Backoff.fixed(150)
System.print(bFixed.kind)
System.print(bFixed.delayFor(0))
System.print(bFixed.delayFor(5))

const bExp = Backoff.exponential(10, 80)
System.print(bExp.kind)
System.print(bExp.delayFor(0))
System.print(bExp.delayFor(1))
System.print(bExp.delayFor(2))
System.print(bExp.delayFor(3))
System.print(bExp.delayFor(4))
System.print(bExp.delayFor(10))

try {
  Backoff.fixed(-5)
} catch e {
  System.print("caught negative fixed: " + e.message)
}

try {
  Backoff.exponential(-1, 100)
} catch e {
  System.print("caught negative exponential base: " + e.message)
}

try {
  Backoff.exponential(100, 50)
} catch e {
  System.print("caught invalid exponential max: " + e.message)
}

try {
  bExp.delayFor(-1)
} catch e {
  System.print("caught negative attempt: " + e.message)
}
