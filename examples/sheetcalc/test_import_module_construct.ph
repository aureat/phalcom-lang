import "./src/test_construct_module" as TCM

class Main {
  @class
  main {
    System.print("About to call TCM.TestConstructModule.new()")
    const obj = TCM.TestConstructModule.new()
    System.print("Called, got: " + obj.toString)
    System.print("obj.x = " + obj.x.toString)
  }
}

Main.main
