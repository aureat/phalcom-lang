@sealed
class Ellipsis {
  @class mut _instance

  @class
  instance {
    if (_instance == None) {
      _instance = Ellipsis.create()
    }
    _instance
  }

  @constructor
  private create() { self }

  @class
  new() { Error.new("There can be only one Ellipsis instance").raise() }

  @class
  call() { Ellipsis.instance }

  ==(_ other) { self === other }

  hash { #ellipsis.hash }

  toString { "..." }

  toRepr { "..." }
}

let ellipsis = Ellipsis.instance

export Ellipsis, ellipsis
