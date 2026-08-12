class User {
  greet() {}
}

class Factory {
  make() { User.new() }
}

const factory = Factory.new()
factory.make()./*@completion*/greet()
