@sealed
class Unimplemented is Error {
  @class
  call() {
    return Unimplemented.instance
  }

  @class
  new() {
    Error.new("Not implemented").raise()
  }
}
