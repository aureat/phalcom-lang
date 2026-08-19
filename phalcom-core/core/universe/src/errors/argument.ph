class ArgumentError is Error {}

// Raised by strict Map subscript lookup when no equal key is present.
class KeyError is Error {}

// Raised when a sequence index is out of bounds.
class IndexError is Error {}

// Raised when a Range cannot describe a sequence slice or replacement.
class SliceError is Error {}

// Raised while building an association Map literal when a logically equal key
// was already contributed. Ordinary post-construction Map insertion still
// overwrites by design.
class DuplicateKeyError is Error {
  @constructor
  new(_ key) {
    super.new("Duplicate key: " + key.toString)
    _key = key
  }
  key { _key }
}
