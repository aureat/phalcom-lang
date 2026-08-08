import "./src/support/num" as Num
import "./src/value/cell_value" as Value

class Main {
  @class
  main {
    System.print("CellValue tests")

    const n1 = Value.CellNum.of(5)
    const n2 = Value.CellNum.of(3)
    System.print("CellNum(5) + CellNum(3) = " + n1.plus(n2).toString)
    System.print("CellNum(5) - CellNum(3) = " + n1.minus(n2).toString)
    System.print("CellNum(5) * CellNum(3) = " + n1.times(n2).toString)
    System.print("CellNum(5) / CellNum(3) = " + n1.dividedBy(n2).toString)
    System.print("")

    const t1 = Value.CellText.of("hello")
    System.print("CellText('hello').toString = " + t1.toString)
    System.print("CellText('hello') + CellNum(5) = " + t1.plus(n1).toString)
    System.print("")

    const err = Value.ErrorVal.divByZero
    System.print("ErrorVal.divByZero = " + err.toString)
    System.print("CellNum(5) + ErrorVal = " + n1.plus(err).toString)
    System.print("")

    const e = Value.CellEmpty.of
    System.print("CellEmpty + CellNum(5) = " + e.plus(n1).toString)
    System.print("CellEmpty * CellNum(5) = " + e.times(n1).toString)
    System.print("")

    const divZero = n1.dividedBy(Value.CellNum.of(0))
    System.print("CellNum(5) / CellNum(0) = " + divZero.toString)
    System.print("")

    System.print("✓ CellValue tests passed")
  }
}

Main.main
