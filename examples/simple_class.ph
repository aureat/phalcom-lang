class Foo {
  static foo() => "foo"

  bar() {
    "bar"
  }
}

System.print(Foo.foo())

let foo = Foo.new()
System.print(foo.bar())
