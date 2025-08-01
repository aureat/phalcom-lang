class Foo {
  // this is invalid syntax
  // method() { "foo" }

  method1() { "foo"
  }

  method2() {
    "foo"
  }

  method3() {
    return "foo"
  }

  method4() {
    one; two;
    three; four
    "foo";
  }

  foo => "foo"
}