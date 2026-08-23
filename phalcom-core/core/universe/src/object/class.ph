@native
class Class is Behavior {
  @native
  +(_ member: Dynamic) -> Dynamic

  @internal @native
  _$new() -> Dynamic

  new() { _$new() }
}
