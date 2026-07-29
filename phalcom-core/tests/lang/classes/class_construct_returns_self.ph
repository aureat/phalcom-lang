// area: classes
// spec: classes.md; object-model.md
// status: PASS

// `construct` implicitly returns the freshly-allocated instance (`self`),
// so the constructor result can be messaged directly.
class Box {
  @constructor
  new(v:) { _v = v }
  v => _v
}
System.print(Box.new(v: "packed").v)
