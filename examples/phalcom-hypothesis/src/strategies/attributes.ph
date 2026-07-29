// passive reflected metadata for opt-in automatic derivation and custom
// strategy providers. These attributes never wrap methods, alter dispatch,
// or add runtime value checks.

@On(Class)
class arbitrary is Attribute {
  @constructor
  new() {}
}

@On(Method)
class strategy is Attribute {
  @constructor
  new(targetType: Any) {
    _targetType = targetType
  }

  targetType -> Any => _targetType
}
