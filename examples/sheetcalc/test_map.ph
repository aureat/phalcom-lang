class Main {
  static main {
    System.print("Testing Map")
    const m = Map.new()
    System.print("Map.new() created")
    const key = "test"
    m.at(key, put: "value")
    System.print("Map.at(key, put: value) done")
    System.print("Map.at(key) = " + m.at(key))
  }
}

Main.main
