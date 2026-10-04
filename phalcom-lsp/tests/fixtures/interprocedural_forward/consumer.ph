import .provider as Provider

class Product {
  @constructor new() { }
  productOnly() { }
}

let result = Provider.Relay.new().forward(Product.new())
result./*@result*/toString()
