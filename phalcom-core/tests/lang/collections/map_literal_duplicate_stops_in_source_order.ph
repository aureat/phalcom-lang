// area: collections
// spec: collection-literals-and-map-spec.md §6-§8; B.3 §7
// status: PASS
// Each association evaluates key then value, then checks uniqueness. The
// duplicate second key aborts construction before either expression in the
// third association can run.

let trace = []
const key1 = || { trace.append("key1"); #same }
const value1 = || { trace.append("value1"); 1 }
const key2 = || { trace.append("key2"); #same }
const value2 = || { trace.append("value2"); 2 }
const laterKey = || { trace.append("later-key"); #later }
const laterValue = || { trace.append("later-value"); 3 }

const caught = || {
  const ignored = {
    [key1.call()]: value1.call(),
    [key2.call()]: value2.call(),
    [laterKey.call()]: laterValue.call(),
  }
}.on(DuplicateKeyError) |e| { e.message }

System.print(caught)
System.print(trace)
