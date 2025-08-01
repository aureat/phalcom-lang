class Foo {
  static foo() => "foo"

  hello

  foo() => hello
}

// System.print(Foo.foo())

let foo = Foo.new()
System.print(foo.foo())