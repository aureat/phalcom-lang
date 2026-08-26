// LAW CHAIN
// 1. Registry.lookup crosses from a partially known class-side call into an opaque result.
// 2. Inspector performs class, membership, and selector reflection on that result.
// 3. Reflection observations remain advisory/dynamic and do not become formal static types.
// 4. Service -> Inspector -> Registry composes the boundary while a known constructor path survives.
//
// OBSERVATIONS
// 01 User has a concrete constructor and name member.
// 02 Admin retains nominal User ancestry.
// 03 Registry.lookup publishes an opaque candidate through mystery().
// 04 candidate.class records runtime class reflection, not static type identity.
// 05 candidate.respondsTo(#new) is a selector capability query.
// 06 candidate.isA(User) traverses nominal membership at runtime.
// 07 candidate.respondsTo(#name) is independent from class reflection.
// 08 the reflection branch joins labels without strengthening candidate knowledge.
// 09 Inspector publishes a tuple containing dynamic and boolean observations.
// 10 Service preserves the independent User.new().name() result as String.
// 11 Registry -> Inspector -> Service dependency chain crosses the dynamic boundary.
// 12 Probe publishes the composed reflection result without treating `.class` as a type.

class User {
  _name: String

  @constructor
  new(_ name: String) {
    _name = name
  }

  name() -> String { _name }
}

class Admin is User {}

class Registry {
  @class
  lookup(_ name: String) {
    if name == "user" {
      User.new("Ada")
    } else {
      mystery()
    }
  }
}

class Inspector {
  @class
  inspect(_ candidate) {
    let runtimeClass = candidate.class
    let canConstruct = candidate.respondsTo(#new)
    let isUser = candidate.isA(User)
    let hasName = candidate.respondsTo(#name)

    let category = if isUser {
      "user"
    } else {
      "opaque"
    }

    (runtimeClass, canConstruct, hasName, category)
  }
}

class Service {
  @class
  run(_ name: String) {
    let candidate = Registry.lookup(name)
    let reflection = Inspector.inspect(candidate)
    let knownName = User.new("known").name()
    (reflection, knownName)
  }
}

class Probe {
  @class
  run(_ name: String) {
    Service.run(name)
  }
}
