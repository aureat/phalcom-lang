class Foo {
  test() => class
}

class Bar is Foo {}

let f = Foo.new()
let b = Bar.new()
System.print(f.test().name)
System.print(b.test().name)
