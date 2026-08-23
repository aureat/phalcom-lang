@!documentation("Method family reference produced by selector binding.")
@native
class Family is Function {
  @native receiver -> Dynamic
  @native selector -> Selector
  @native pattern -> SelectorPattern
  @native isExact -> Bool
  @native get() -> Dynamic
  @native set(_ method: Dynamic) -> Dynamic
}

@native
class MethodFamily is Object {
  @native bind(_ receiver: Dynamic) -> BoundMethodFamily
  @native selectors -> List
  @native size -> Int
  @native methodFor(_ selector: Selector) -> Method
}

@native
class BoundMethodFamily is Function {}
