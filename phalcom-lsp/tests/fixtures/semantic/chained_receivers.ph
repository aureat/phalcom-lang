class User {
  @constructor new() {}
  greet() {}
}

class Factory {
  @constructor new() {}
  make() { User.new() }
}

let factory = Factory.new()
factory.make()./*@completion*/greet()
