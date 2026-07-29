class TestObj {
  @constructor
  new() {
    _x = 42
    _m = Map.new()
  }

  x => _x
  m => _m
}

class Main {
  static main {
    const obj = TestObj.new()
    System.print("obj.x = " + obj.x.toString)
    System.print("obj.m = " + obj.m.toString)
  }
}

Main.main
