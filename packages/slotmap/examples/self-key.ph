from slotmap.slot_map import SlotKey, SlotMap

data Entities()
type EntityId = SlotKey<Entities>

data Entity(
  id: EntityId,
  name: String
)

let entities = SlotMap<Entities, Entity>()

let socrates = entities.insertWithKey |id| {
  Entity(id: id, name: "Socrates")
}

let plato = entities.insertWithKey |id| {
  Entity(id: id, name: "Plato")
}

for entry in entities {
  let (id, entity) = entry
  System.print("\(id) => \(entity.name)")
}

entities.remove(socrates)

// The type is still correct, but the generation is stale.
match entities.get(socrates) {
  Option::Some(_) => System.print("unexpected live key")
  Option::None => System.print("removed handle is safely stale")
}

System.print(entities[plato].name)
