class Relay {
  @constructor new() { }
  sink(_ value) { value }
  forward(_ value) { sink(value) }
}
