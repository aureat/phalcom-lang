class Foo {
  test() { class }
}

class Bar is Foo {}

let mut f = Foo.new()
let mut b = Bar.new()
System.print(f.test().name)
System.print(b.test().name)
