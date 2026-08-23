@!documentation("First-class dispatch selector representation.")
@native
class Selector is Object {
  @class @native call(_ value: Dynamic) -> Selector
  @class @native from(_ value: Dynamic) -> Selector
  @class @native new(_ value: Dynamic) -> Selector
  @native base -> Symbol
  @native kind -> Symbol
  @native slots -> Tuple
  @native toString -> String
  @native ==(_ other: Dynamic) -> Bool
  @native hash -> Int
}

@native
class SelectorPattern is Object {
  @class @native call(_ value: Dynamic) -> SelectorPattern
  @class @native from(_ value: Dynamic) -> SelectorPattern
  @class @native new(_ value: Dynamic) -> SelectorPattern
  @native base -> Symbol
  @native matches(_ other: Dynamic) -> Bool
  @native toString -> String
  @native ==(_ other: Dynamic) -> Bool
  @native hash -> Int
}
