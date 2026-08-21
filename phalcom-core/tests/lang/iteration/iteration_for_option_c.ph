// area: iteration
// spec: for option C, indexed, zipped

// 1. Single lane with ordinal
let items = ["apple", "banana", "cherry"]
for item at i in items {
  System.print("\(i): \(item)")
}

// 2. Parallel lanes (Option C lockstep)
let nums = [1, 2, 3]
let names = ["one", "two", "three"]
for n in nums, name in names {
  System.print("\(n) -> \(name)")
}

// 3. Parallel lanes with ordinals
for n at i in nums, name at j in names {
  System.print("[\(i),\(j)]: \(n) -> \(name)")
}

// 4. Equivalence with .indexed
for (idx, item) in items.indexed {
  System.print("indexed \(idx): \(item)")
}

// 5. Equivalence with .zipped
for (n, name) in (nums, names).zipped {
  System.print("zipped \(n) -> \(name)")
}

// 6. Destructuring in for lane
let pairs = [(10, 100), (20, 200)]
for (a, b) in pairs {
  System.print("pair \(a) + \(b) = \(a + b)")
}
