class Test {
  _name
  _age

  @construct
  init(name, age) {
    _name = name
    _age = age
  }

  static method {
    self.method2 and "hello"
  }

  static method2 {
    System.new()
  }
}

// Test.method
Test.superclass = Test