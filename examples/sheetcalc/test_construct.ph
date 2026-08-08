class MyGrid {
  @constructor
  new() {
    System.print("[MyGrid construct] running")
    _x = 99
  }

  x => _x
}

class Main {
  @class
  main {
    System.print("About to call MyGrid.new()")
    const g = MyGrid.new()
    System.print("Called MyGrid.new()")
    System.print("g.x = " + g.x.toString)
  }
}

Main.main
