@sealed
class Unsupported is Error {
  @class mut _instance

  @class
  instance {
    if (_instance == None) {
      _instance = Unsupported.create()
    }

    _instance
  }

  @constructor
  private create() {
    // _message = "unsupported"
    // _kind = #unsupported
    // _cause = None
    // _displaced = None
    super.initialize(
      with: "unsupported",
      of: #unsupported,
      from: None,
      displaced: None
    )
  }

  @class new() {
    Error.new("There can be only one Unsupported instance").raise()
  }

  @class call() {
    Unsupported.instance
  }
}

let unsupported = Unsupported.instance

export unsupported