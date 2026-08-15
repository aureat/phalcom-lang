// area: family
// spec: docs/spec/callables/reflection.md §2
// status: PASS
// `Behavior >> pattern` uses the same structural prefix/suffix matching laws
// as receiver-bound Family patterns. These counts also prove zero-width gaps.

class Shape {
  pick(_ a) { 1 }
  pick(_ a, _ b) { 2 }
  pick(foo) { 3 }
  pick(_ a, foo) { 4 }
  pick(_ a, mid, foo) { 5 }
  pick(bar) { 6 }
}

System.print((Shape >> #pick(_, ...)).size)
System.print((Shape >> #pick(..., foo)).size)
System.print((Shape >> #pick(_, ..., foo)).size)
