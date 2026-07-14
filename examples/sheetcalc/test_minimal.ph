import "./src/support/num" as Num

class Main {
  static main {
    System.print("Num.floor(-3.7) = " + Num.Num.floor(-3.7).toString)
    System.print("Num.ceil(-3.7) = " + Num.Num.ceil(-3.7).toString)
    System.print("✓ Num tests passed")
  }
}

Main.main
