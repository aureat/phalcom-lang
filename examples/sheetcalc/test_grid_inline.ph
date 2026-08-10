class TestGrid {
  @constructor
  new() {
    System.print("[TestGrid construct] running")
    _cells = Map.new()
    _minCol = -1
  }

  minCol { _minCol }
  cells { _cells }

  set(_ ref, _ cell) {
    System.print("[TestGrid.set] called")
    _cells.at(ref, put: cell)
    System.print("[TestGrid.set] done")
  }
}

class Main {
  @class
  main {
    System.print("Test inline Grid")
    const g = TestGrid.new()
    System.print("Grid created, minCol = " + g.minCol.toString)
    System.print("About to call set()")
    const r = "key"
    const c = "cell"
    g.set(r, c)
    System.print("set() done")
  }
}

Main.main
