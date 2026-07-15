import "./src/grid/ref" as Ref

class Main {
  static main {
    System.print("Testing Ref import")
    let r = Ref.Ref.at(1, 1)
    System.print("Ref created: " + r.toString)
    System.print("Ref.toA1: " + r.toA1)
  }
}

Main.main
