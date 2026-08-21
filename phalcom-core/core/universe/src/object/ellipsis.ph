@sealed
class Ellipsis {
  @class _instance

  @class
  instance {
    if (_instance == None) {
      _instance = Ellipsis.create()
    }
    _instance
  }

  @private
  @constructor
  create() { self }

  @class
  new() { Error.new("There can be only one Ellipsis instance").raise() }

  @class
  call() { Ellipsis.instance }

  ==(_ other) { self === other }
  hash { #ellipsis.hash }
  toString { "..." }
  toRepr { "..." }
}

const ellipsis = Ellipsis.instance

export Ellipsis
export ellipsis
