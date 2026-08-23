import sheetcalc.grid.cell as Cell
import sheetcalc.value.cell_value as Value

class Main {
  @class
  main {
    System.print("Testing Cell import")
    const lit = Cell.LiteralCell.of(Value.CellNum.of(42))
    System.print("LiteralCell created: " + lit.cachedValue.toString)
  }
}

Main.main
