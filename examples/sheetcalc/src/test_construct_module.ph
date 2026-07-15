class TestConstructModule {
  construct new() {
    _x = 42
    System.print("[TestConstructModule] construct ran, _x = " + _x.toString)
  }

  x => _x
}
