// area: family
// spec: docs/spec/callables/reflection.md §2
// status: PASS
// AnyNamed includes getter/setter/method selectors. `name(...)` is method-only.

class Box {
  value { 1 }
  value() { 2 }
  value(_ x) { 3 }
  value=(put x) { 4 }
}

System.print((Box >> #value...).size)
System.print((Box >> #value(...)).size)
