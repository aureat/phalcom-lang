import "./src/value/cell_value" as Value

class Main {
  @class
  main {
    System.print("Testing CellNum")
    const n1 = Value.CellNum.of(5)
    System.print("Created CellNum(5)")
    const n2 = Value.CellNum.of(3)
    System.print("Created CellNum(3)")
    const result = n1.plus(n2)
    System.print("Called plus: " + result.toString)
  }
}

Main.main
