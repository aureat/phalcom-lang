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

  method4() { "foo";
  }

  method5() {
    one; two;
    three; four
    "foo";
  }

  foo => "foo"
}