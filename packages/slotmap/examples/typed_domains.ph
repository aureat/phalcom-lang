from slotmap.slot_map import SlotKey, SlotMap

data Philosophers()
data Propositions()

type PhilosopherId = SlotKey<Philosophers>
type PropositionId = SlotKey<Propositions>

let philosophers = SlotMap<Philosophers, String>()
let propositions = SlotMap<Propositions, String>()

let socrates: PhilosopherId = philosophers.insert("Socrates")
let mortality: PropositionId = propositions.insert("All humans are mortal")

System.print(philosophers[socrates])
System.print(propositions[mortality])

// Intentionally does not type-check:
// propositions.get(socrates)
//
// expected SlotKey<Propositions>
// found    SlotKey<Philosophers>
