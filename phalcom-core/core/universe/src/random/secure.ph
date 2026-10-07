/// Failure to acquire cryptographically suitable bytes from the OS.
class EntropyError is Error {}

/// Explicit OS-backed secure randomness capability. It does not implement
/// Generator and never falls back to deterministic randomness.
class SecureRandom {
  @class mut _instance

  @constructor
  private create() { self }

  @class
  system -> SecureRandom {
    if _instance == None { _instance = SecureRandom.create() }
    _instance
  }

  internal native rawFill(_ bytes: Bytes) -> Option<String>

  /// On host failure raises EntropyError; existing buffer contents are then
  /// unspecified. Successful calls fill the complete buffer.
  fill(_ bytes: Bytes) -> Unit {
    self.rawFill(bytes).match(
      some: |message| { EntropyError.new(message).raise() },
      none: || { () }
    )
    ()
  }

  /// Raises on failure, so no partial Bytes value is returned.
  bytes(_ count: Int) -> Bytes {
    if count < 0 { ArgumentError.new("SecureRandom.bytes: count must be non-negative").raise() }
    let out = Bytes.new(count)
    self.fill(out)
    out
  }
}

export SecureRandom, EntropyError
