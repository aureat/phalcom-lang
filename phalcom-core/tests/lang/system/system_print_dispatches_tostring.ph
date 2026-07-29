// area: system
// spec: system.md
// status: PASS
// Ported from Wren `test/core/system/print.wren` (first half): `System.print`
// renders an instance by sending it `toString`, not a generic default.

class Foo {
  @constructor
  new() {}

  toString => "Foo.toString"
}

System.print(Foo.new())
