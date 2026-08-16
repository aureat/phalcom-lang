import .provider as Provider

class Cat {
  @constructor new() { }
  catOnly() { }
}

const result = Provider.Service.new().consume(Cat.new())
result./*@result*/toString()
