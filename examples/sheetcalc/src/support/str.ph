/// String utilities missing from the core String class.
class Str {
  /// Pad str on the left to width characters, using padChar (default space).
  @class
  padLeft(_ str, _ width, _ padChar) {
    const pc = if (padChar == nil) { " " } else { padChar }
    const current = str.size
    if (current >= width) {
      return str
    }
    const pad = Str.repeat(pc, width - current)
    return pad + str
  }

  /// Pad str on the right to width characters, using padChar (default space).
  @class
  padRight(_ str, _ width, _ padChar) {
    const pc = if (padChar == nil) { " " } else { padChar }
    const current = str.size
    if (current >= width) {
      return str
    }
    const pad = Str.repeat(pc, width - current)
    return str + pad
  }

  /// Repeat str count times.
  @class
  repeat(_ str, _ count) {
    let result = ""
    let i = 0
    while (i < count) {
      result = result + str
      i = i + 1
    }
    return result
  }

  /// Test whether str starts with prefix.
  @class
  startsWith(_ str, _ prefix) {
    if (prefix.size > str.size) {
      return false
    }
    const check = str.slice(0, prefix.size)
    return check == prefix
  }

  /// Test whether str ends with suffix.
  @class
  endsWith(_ str, _ suffix) {
    if (suffix.size > str.size) {
      return false
    }
    const start = str.size - suffix.size
    const check = str.slice(start, str.size)
    return check == suffix
  }
}
