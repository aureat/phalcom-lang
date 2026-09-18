native class Class is Behavior {
  native +(_ member: Dynamic) -> Dynamic

  internal native allocate() -> Dynamic

  new() { allocate() }
}
