import "./src/grid/ref" as Ref
import "./src/grid/cell" as Cell
import "./src/grid/grid" as Grid
import "./src/value/cell_value" as Value

class Main {
  static main {
    System.print("Simple Grid test")
    let grid = Grid.Grid.new()
    System.print("Grid created, type: " + grid.toString)
    System.print("Grid.isEmpty = " + grid.isEmpty.toString)

    let r = Ref.Ref.at(1, 1)
    System.print("Ref created: " + r.toString)

    let v = Value.CellNum.of(42)
    System.print("CellValue created: " + v.toString)

    let cell = Cell.LiteralCell.of(v)
    System.print("Cell created")

    System.print("About to call grid.set()")
    grid.set(r, cell)
    System.print("grid.set() done")
  }
}

Main.main
