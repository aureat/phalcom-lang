import .provider as Provider

class Dog {
  @constructor new() { }
  dogOnly() { }
}

let result = Provider.Service.new().consume(Dog.new())
result./*@result*/toString()
