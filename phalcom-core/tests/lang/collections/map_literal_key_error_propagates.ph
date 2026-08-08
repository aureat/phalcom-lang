// area: collections
// spec: collection-literals-and-map-spec.md §8; B.3 §7
// status: PASS
// A key hash failure remains its original Error. It occurs after this entry's
// value evaluation and stops later associations from running.

let trace = []

class ExplodingKey {
  hash {
    trace.append("hash")
    throw Error.new("hash failed")
  }
}

const value = || { trace.append("value"); 1 }
const later = || { trace.append("later"); 2 }
const caught = || {
  const ignored = {
    [ExplodingKey.new()]: value.call(),
    [#later]: later.call(),
  }
}.on(Error) |e| { e.message }

System.print(caught)
System.print(trace)
