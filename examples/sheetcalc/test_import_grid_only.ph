import "./src/grid/grid" as GridMod

class Main {
  static main {
    System.print("Testing Grid import")
    let g = GridMod.Grid.new()
    System.print("Grid created: " + g.toString)
    System.print("Grid.isEmpty: " + g.isEmpty.toString)
  }
}

Main.main
