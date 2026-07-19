import "./src/grid/cell" as Cell
import "./src/value/cell_value" as Value

class Main {
  static main {
    System.print("Testing Cell import")
    const lit = Cell.LiteralCell.of(Value.CellNum.of(42))
    System.print("LiteralCell created: " + lit.cachedValue.toString)
  }
}

Main.main
