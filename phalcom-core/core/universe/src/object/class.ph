native primitive class Class is Behavior {
  native +(_ member: Dynamic) -> Dynamic

  internal native primitive allocate() -> Dynamic

  new() { allocate() }
}
