class Foo {
  static bar() {
    return "bar";
  }
}

let foo = Foo.new();
System.print(foo);
System.print(foo.new());
System.print(Foo.bar());