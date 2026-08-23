// `Method#attributes`/`#attributesOfType(_)` — the same reflection surface
// as `Behavior` above, for the reified `Method` object a class's method
// dictionary holds.
@native
class Method is Object {
  @class @native new(_ value: Dynamic) -> Method
  @native arity -> Int
  @native name -> Symbol
  @native invokeOn(_ receiver: Dynamic, ***args: Dynamic) -> Dynamic
  @native bind(_ receiver: Dynamic) -> BoundMethod
  @native selector -> Selector
  @native holder -> Dynamic
  @native isNative -> Bool
  @native isIntrinsic -> Bool
  @native implementationKind -> Symbol
  @internal @native _$attributes -> Dynamic
  attributes { self._$attributes }
  attributesOfType(_ cls) { self._$attributes.filter |a| { a.is(cls) } }
}

@native
class BoundMethod is Function {}
