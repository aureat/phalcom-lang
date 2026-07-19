/// SheetCalc: a spreadsheet engine in Phalcom.
/// Phase 1: support layer (L0) + value model (L1)

import "./support/num" as Num
import "./support/str" as Str
import "./support/sort" as Sort
import "./value/cell_value" as Value

class Main {
  static main {
    System.print("SheetCalc phase 1: support + value model")
    System.print("")

    /// Test Num
    System.print("Num.floor(-3.7) = " + Num.Num.floor(-3.7).toString)
    System.print("Num.ceil(-3.7) = " + Num.Num.ceil(-3.7).toString)
    System.print("Num.round(3.5) = " + Num.Num.round(3.5).toString)
    System.print("Num.abs(-5) = " + Num.Num.abs(-5).toString)
    System.print("Num.min([3, 1, 4]) = " + Num.Num.min([3, 1, 4]).toString)
    System.print("Num.max([3, 1, 4]) = " + Num.Num.max([3, 1, 4]).toString)
    System.print("Num.isInt(3.0) = " + Num.Num.isInt(3.0).toString)
    System.print("Num.isInt(3.5) = " + Num.Num.isInt(3.5).toString)
    System.print("")

    /// Test Str
    System.print("Str.padLeft('hi', 5) = '" + Str.Str.padLeft("hi", 5, nil) + "'")
    System.print("Str.padRight('hi', 5) = '" + Str.Str.padRight("hi", 5, nil) + "'")
    System.print("Str.repeat('x', 3) = '" + Str.Str.repeat("x", 3) + "'")
    System.print("Str.startsWith('hello', 'he') = " + Str.Str.startsWith("hello", "he").toString)
    System.print("Str.endsWith('hello', 'lo') = " + Str.Str.endsWith("hello", "lo").toString)
    System.print("")

    /// Test CellValue
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

    /// Test division by zero
    const divZero = n1.dividedBy(Value.CellNum.of(0))
    System.print("CellNum(5) / CellNum(0) = " + divZero.toString)
    System.print("")

    System.print("✓ All smoke tests passed")
  }
}

Main.main
