import "./provider" as Provider

class Dog {
  @constructor new() { }
  dogOnly() { }
}

const result = Provider.Service.new().consume(Dog.new())
result./*@result*/toString()
