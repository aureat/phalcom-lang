class User {
  @constructor new() {}
  greet() {}
}

class Factory {
  @constructor new() {}
  make() { User.new() }
}

const factory = Factory.new()
factory.make()./*@completion*/greet()
