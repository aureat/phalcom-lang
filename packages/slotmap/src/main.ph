from .slot_map import SlotKey, SlotMap
from .dense_slot_map import DenseSlotMap

data Philosophers()

let philosophers = SlotMap<Philosophers, String>()

let socrates = philosophers.insert("Socrates")
let plato = philosophers.insert("Plato")

System.print(philosophers[socrates])
System.print(philosophers[plato])

philosophers.remove(socrates)

match philosophers.get(socrates) {
  Option::Some(value) => System.print(value)
  Option::None => System.print("Socrates key is stale")
}

data Propositions()

let propositions = DenseSlotMap<Propositions, String>()
propositions.insert("All humans are mortal")
propositions.insert("Knowledge is recollection")

for entry in propositions {
  let (key, proposition) = entry
  System.print("\(key): \(proposition)")
}
