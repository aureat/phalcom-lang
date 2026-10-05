import .provider as Provider

class Cat {
  @constructor new() { }
  catOnly() { }
}

let result = Provider.Service.new().consume(Cat.new())
result./*@result*/toString()
