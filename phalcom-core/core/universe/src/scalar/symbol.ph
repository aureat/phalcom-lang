@native
class Symbol is Object {
  @native
  toString -> String
  @native
  hash -> Int
  @native
  isSelector -> Bool
  @native
  isSelectorPattern -> Bool
  @class
  @native
  new(_ value: Dynamic) -> Symbol
}
