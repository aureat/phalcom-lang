from slotmap.dense_slot_map import DenseSlotMap
from .support import Test

data Concepts()

let map = DenseSlotMap<Concepts, String>()

let form = map.insert("Form")
let matter = map.insert("Matter")
let cause = map.insert("Cause")

Test.expectEqual(map.len, 3)
Test.expectEqual(map.get(form), Option::Some("Form"))
Test.expectEqual(map.get(matter), Option::Some("Matter"))
Test.expectEqual(map.get(cause), Option::Some("Cause"))

// Remove the middle dense element. The final element moves into its position,
// but its stable key must continue to resolve correctly.
Test.expectEqual(map.remove(matter), Option::Some("Matter"))
Test.expectEqual(map.get(matter), Option::None)
Test.expectEqual(map.get(form), Option::Some("Form"))
Test.expectEqual(map.get(cause), Option::Some("Cause"))
Test.expectEqual(map.len, 2)

// Reuse must never revive the removed stable key.
let essence = map.insert("Essence")
Test.expectEqual(map.get(matter), Option::None)
Test.expectEqual(map.get(essence), Option::Some("Essence"))

// Dense iteration visits len entries and is independent of stable-slot holes.
let iterator = map.iterator
let mut seen = 0

while let Option::Some((key, value)) = iterator.next {
  Test.expectEqual(map.get(key), Option::Some(value))
  seen = seen + 1
}

Test.expectEqual(seen, map.len)

// clear must not resurrect old keys.
let stale = cause
map.clear
Test.expect(map.isEmpty, "dense clear empties the map")
Test.expectEqual(map.get(stale), Option::None)

let actuality = map.insert("Actuality")
Test.expectEqual(map.get(stale), Option::None)
Test.expectEqual(map.get(actuality), Option::Some("Actuality"))

Test.pass("dense-slotmap-basic")
