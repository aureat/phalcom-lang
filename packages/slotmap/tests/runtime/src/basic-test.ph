from slotmap.slot_map import SlotKey, SlotMap
from .support import Test

data Philosophers()

let map = SlotMap<Philosophers, String>()

Test.expect(map.isEmpty, "new map is empty")

let socrates = map.insert("Socrates")
let plato = map.insert("Plato")

Test.expectEqual(map.len, 2)
Test.expectEqual(map.get(socrates), Option::Some("Socrates"))
Test.expectEqual(map.get(plato), Option::Some("Plato"))
Test.expect(map.containsKey(socrates), "inserted key is live")

Test.expectEqual(map.remove(socrates), Option::Some("Socrates"))
Test.expectEqual(map.len, 1)
Test.expectEqual(map.get(socrates), Option<String>::None)
Test.expect(not map.containsKey(socrates), "removed key is stale")
Test.expectEqual(map.remove(socrates), Option<String>::None)

// Reuse must never resurrect the stale key.
let aristotle = map.insert("Aristotle")
Test.expectEqual(map.get(socrates), Option<String>::None)
Test.expectEqual(map.get(aristotle), Option::Some("Aristotle"))

// insertWithKey can construct self-identifying values.
data Thinkers()

data Thinker(
  id: SlotKey<Thinkers>,
  name: String
)

let thinkers = SlotMap<Thinkers, Thinker>()
let epicurus = thinkers.insertWithKey |id| {
  Thinker(id: id, name: "Epicurus")
}

match thinkers.get(epicurus) {
  Option::None => Test.expect(false, "insertWithKey value should exist")
  Option::Some(thinker) => {
    Test.expectEqual(thinker.id, epicurus)
    Test.expectEqual(thinker.name, "Epicurus")
  }
}

// clear must invalidate existing handles without allowing resurrection.
let preClear = map.insert("Zeno")
map.clear
Test.expect(map.isEmpty, "clear empties the map")
Test.expectEqual(map.get(preClear), Option<String>::None)

let postClear = map.insert("Plotinus")
Test.expectEqual(map.get(preClear), Option<String>::None)
Test.expectEqual(map.get(postClear), Option::Some("Plotinus"))

// retain invalidates removed entries and keeps accepted entries.
let a = map.insert("A")
let b = map.insert("BB")
map.retain |_, value| { value.size > 1 }

Test.expectEqual(map.get(a), Option<String>::None)
Test.expectEqual(map.get(b), Option::Some("BB"))

// Sparse iteration yields every live entry exactly once.
let iterator = map.iterator
let mut seen = 0

while let Option::Some((_, _)) = iterator.next {
  seen = seen + 1
}

Test.expectEqual(seen, map.len)

Test.pass("slotmap-basic")
