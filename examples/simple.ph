class Test {
  _name
  _age

  @construct
  init(_ name, _ age) {
    _name = name
    _age = age
  }

  @class
  method {
    self.method2 and "hello"
  }

  @class
  method2 {
    System.new()
  }
}

// Test.method
Test.superclass = Test