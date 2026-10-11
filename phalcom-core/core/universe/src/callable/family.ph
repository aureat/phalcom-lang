@!documentation("Method family reference produced by selector binding.")
native primitive class Family is Function {
  native receiver -> Dynamic
  native selector -> Selector
  native pattern -> SelectorPattern
  native isExact -> Bool
  native get() -> Dynamic
  native set(_ method: Dynamic) -> Dynamic
  native value -> Dynamic
  native value=(_ newValue: Dynamic) -> Dynamic
  /// The call shape is selected at runtime and has no fixed Tuple specialization here.
  native get(_ shape: Dynamic) -> Dynamic
  /// The call shape is selected at runtime and has no fixed Tuple specialization here.
  native set(_ shape: Dynamic, _ value: Dynamic) -> Dynamic
}

native primitive class MethodFamily is Object {
  native bind(_ receiver: Dynamic) -> BoundMethodFamily
  native selectors -> List
  native size -> Int
  native methodFor(_ selector: Selector) -> Method
}

native primitive class BoundMethodFamily is Function {}
