class Foo {
  static foo() => "foo"

  foo() { "hello"
  }

  bar() {
    "bar"; "hello";
  }
}

System.print(Foo.foo())

let foo = Foo.new()
System.print(foo.bar())