/// String utilities missing from the core String class.
class Str {
  /// Pad str on the left to width characters, using padChar (default space).
  static padLeft(str, width, padChar) {
    let pc = if (padChar == nil) { " " } else { padChar }
    let current = str.size
    if (current >= width) {
      return str
    }
    let pad = Str.repeat(pc, width - current)
    return pad + str
  }

  /// Pad str on the right to width characters, using padChar (default space).
  static padRight(str, width, padChar) {
    let pc = if (padChar == nil) { " " } else { padChar }
    let current = str.size
    if (current >= width) {
      return str
    }
    let pad = Str.repeat(pc, width - current)
    return str + pad
  }

  /// Repeat str count times.
  static repeat(str, count) {
    var result = ""
    var i = 0
    while (i < count) {
      result = result + str
      i = i + 1
    }
    return result
  }

  /// Test whether str starts with prefix.
  static startsWith(str, prefix) {
    if (prefix.size > str.size) {
      return false
    }
    let check = str.slice_(0, prefix.size)
    return check == prefix
  }

  /// Test whether str ends with suffix.
  static endsWith(str, suffix) {
    if (suffix.size > str.size) {
      return false
    }
    let start = str.size - suffix.size
    let check = str.slice_(start, str.size)
    return check == suffix
  }
}
