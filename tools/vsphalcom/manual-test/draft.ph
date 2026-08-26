class Path {
  _segments: List<String>

  @constructor
  new() {
    _segments = []
  }

  __intercept__(
    _ sym: Symbol,
    _ args: List<Object>,
    _ proceed: () -> Unit
  ) -> Unit {
    const selector = Selector(selector)

  }

}
