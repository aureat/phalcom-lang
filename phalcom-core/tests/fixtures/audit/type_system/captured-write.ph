class Probe {
  @class
  run() -> Int {
    let mut x: Int | String = 1
    let mut action = || { x = "changed" }
    action()
    return x
  }
}
System.print(Probe.run())
