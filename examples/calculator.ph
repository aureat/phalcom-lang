class Calculator {

  @class
  new(_ ignored) {
    // a;
    return self.new();
  }

  add(_ a, _ b) {
    return a + b;
  }

  subtract(_ a, _ b) {
    return a - b;
  }

  @class
  pi => 3.1415

  +(_ other) {
    return 10 + other;
  }

  and(_ other) {
    return "and " + other;
  }

}

const calc = Calculator.new(0);

System.print(calc.add(5, 3));         // 8
System.print(calc.subtract(10, 4));   // 6

// System.print(calc.pi);                // 3.1415

System.print(calc + 20);              // 30

System.print(Calculator.pi)
