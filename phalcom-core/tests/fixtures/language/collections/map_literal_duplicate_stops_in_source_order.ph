// area: collections
// spec: collection-literals-and-map-spec.md §6-§8; B.3 §7
// status: PASS
// Each association evaluates key then value, then checks uniqueness. The
// duplicate second key aborts construction before either expression in the
// third association can run.

let mut trace: List<String> = []
let key1 = || { trace.append("key1"); #same }
let value1 = || { trace.append("value1"); 1 }
let key2 = || { trace.append("key2"); #same }
let value2 = || { trace.append("value2"); 2 }
let laterKey = || { trace.append("later-key"); #later }
let laterValue = || { trace.append("later-value"); 3 }

let caught = || {
  let ignored = Map {
    [key1.call()]: value1.call(),
    [key2.call()]: value2.call(),
    [laterKey.call()]: laterValue.call(),
  }
}.on(DuplicateKeyError) |e| { e.message }

System.print(caught)
System.print(trace)
