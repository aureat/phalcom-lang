// area: bindings
// spec: LANG007.C4 P1 immutable-by-default field surface
// status: PASS
// Mutable storage is explicit even when it has a declaration default.

class Origin {
  mut _x = 10
  @constructor
  new(_ v) { _x = v }
  get { return _x }
}

System.print(Origin.new(10).get)
