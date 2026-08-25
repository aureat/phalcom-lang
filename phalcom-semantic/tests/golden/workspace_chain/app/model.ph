// LAW CHAIN
// User inherits Entity across module identity and exports UserId provenance.

class Entity {
  id() -> Int { 1 }
}

class User is Entity {
  @constructor new() {}
  name() -> String { "Ada" }
}

type UserId = Int
export Entity, User, UserId
