class Probe {
  @class
  run() -> Int {
    let x: Int | String = 1
    let action = || { x = "changed" }
    action()
    return x
  }
}
System.print(Probe.run())
