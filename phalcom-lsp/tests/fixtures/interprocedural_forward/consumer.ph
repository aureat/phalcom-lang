import .provider as Provider

class Product {
  @constructor new() { }
  productOnly() { }
}

const result = Provider.Relay.new().forward(Product.new())
result./*@result*/toString()
