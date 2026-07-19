import "./src/grid/ref" as Ref
import "./src/grid/cell" as Cell
import "./src/grid/grid" as Grid
import "./src/grid/ref_range" as RefRange
import "./src/value/cell_value" as Value

class Main {
  static main {
    System.print("L2 Grid layer tests")
    System.print("")

    /// Test Ref A1 encoding/decoding
    System.print("Ref.encodeCol(1) = " + Ref.Ref.encodeCol(1))
    System.print("Ref.encodeCol(26) = " + Ref.Ref.encodeCol(26))
    System.print("Ref.encodeCol(27) = " + Ref.Ref.encodeCol(27))
    System.print("Ref.decodeCol_('A') = " + Ref.Ref.decodeCol_("A").toString)
    System.print("Ref.decodeCol_('Z') = " + Ref.Ref.decodeCol_("Z").toString)
    System.print("Ref.decodeCol_('AA') = " + Ref.Ref.decodeCol_("AA").toString)
    System.print("")

    /// Test Ref fromA1
    const r1 = Ref.Ref.fromA1("A1")
    System.print("Ref.fromA1('A1') col=" + r1.col.toString + " row=" + r1.row.toString)
    const r2 = Ref.Ref.fromA1("$A$1")
    System.print("Ref.fromA1('$A$1') colAbs=" + r2.colAbs.toString + " rowAbs=" + r2.rowAbs.toString)
    System.print("r1.toA1 = " + r1.toA1)
    System.print("r2.toA1 = " + r2.toA1)
    System.print("")

    /// Test Ref equality (address-only)
    const r3 = Ref.Ref.at(1, 1)
    const r4 = Ref.Ref.full(1, 1, true, true)
    System.print("Ref.at(1,1) == Ref.full(1,1,true,true) = " + (r3 == r4).toString)
    System.print("")

    /// Test Ref offset
    const r5 = Ref.Ref.at(2, 3)
    const r6 = r5.offset(1, 2)
    System.print("Ref(2,3).offset(1,2) = " + r6.col.toString + "," + r6.row.toString)
    System.print("")

    /// Test Cell
    const lit = Cell.LiteralCell.of(Value.CellNum.of(42))
    System.print("LiteralCell(42).isFormula = " + lit.isFormula.toString)
    System.print("LiteralCell(42).cachedValue.toString = " + lit.cachedValue.toString)
    System.print("")

    /// Test Grid
    const grid = Grid.Grid.new()
    const cell1 = Cell.LiteralCell.of(Value.CellNum.of(5))
    grid.set(Ref.Ref.at(1, 1), cell1)
    System.print("Grid set(A1, 5) done")
    System.print("Grid.valueAt(A1) = " + grid.valueAt(Ref.Ref.at(1, 1)).toString)
    System.print("Grid.isEmpty = " + grid.isEmpty.toString)
    System.print("Grid.minCol = " + grid.minCol.toString)
    System.print("Grid.maxCol = " + grid.maxCol.toString)
    System.print("")

    /// Test RefRange
    const range = RefRange.RefRange.fromTo(Ref.Ref.at(1, 1), Ref.Ref.at(2, 2))
    System.print("RefRange A1:B2 size = " + range.size.toString)
    System.print("RefRange toString = " + range.toString)
    System.print("")

    System.print("✓ L2 tests complete")
  }
}

Main.main
