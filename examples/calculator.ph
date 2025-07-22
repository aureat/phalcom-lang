class Calculator {

  static new(ignored) {
    // a;
    return self.new();
  }

  add(a, b) {
    return a + b;
  }

  subtract(a, b) {
    return a - b;
  }

  static pi { return 3.1415; }

  +(other) {
    return 10 + other;
  }

  and(other) {
    return "and " + other;
  }

}

System.print(Calculator.pi);

let calc = Calculator.new(0);

System.print(calc.add(5, 3));         // 8
System.print(calc.subtract(10, 4));   // 6

// System.print(calc.pi);                // 3.1415

System.print(calc + 20);              // 30