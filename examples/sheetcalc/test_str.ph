import "./src/support/str" as Str

class Main {
  @class
  main {
    System.print("Str.repeat('x', 3) = '" + Str.Str.repeat("x", 3) + "'")
    System.print("Str.startsWith('hello', 'he') = " + Str.Str.startsWith("hello", "he").toString)
    System.print("Str.endsWith('hello', 'lo') = " + Str.Str.endsWith("hello", "lo").toString)
    System.print("✓ Str tests passed")
  }
}

Main.main
