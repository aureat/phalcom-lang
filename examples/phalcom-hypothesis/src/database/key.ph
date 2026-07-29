// Stable typed identity for persisted property examples.

@data
@immutable
class DatabaseKey {
  const _package: Symbol
  const _module: Symbol
  const _suite: Symbol
  const _selector: Symbol
  const _strategyFingerprint: String
  const _engineFormatVersion: Int

  @class
  @requires(engineFormatVersion > 0)
  create(
    package: Symbol,
    module: Symbol,
    suite: Symbol,
    selector: Symbol,
    strategyFingerprint: String,
    engineFormatVersion: Int
  ) -> DatabaseKey {
    return DatabaseKey.new(
      package: package,
      module: module,
      suite: suite,
      selector: selector,
      strategyFingerprint: strategyFingerprint,
      engineFormatVersion: engineFormatVersion
    )
  }

  canonical -> String {
    return "db-key-v1|" +
      _DatabaseKeyText.field(_package.toString) +
      _DatabaseKeyText.field(_module.toString) +
      _DatabaseKeyText.field(_suite.toString) +
      _DatabaseKeyText.field(_selector.toString) +
      _DatabaseKeyText.field(_strategyFingerprint) +
      _DatabaseKeyText.field(_engineFormatVersion.toString)
  }

  fileStem -> String {
    return "k-" + _DatabaseKeyHash.hex(
      _DatabaseKeyHash.fnv1a(self.canonical)
    )
  }
}

class _DatabaseKeyText {
  @class
  field(value: String) -> String {
    return value.codePoints.size.toString + ":" + value + "|"
  }
}

class _DatabaseKeyHash {
  @class
  fnv1a(value: String) -> Int {
    let hash = 2166136261
    for point in value.codePoints {
      hash = ((hash ^ point) * 16777619) % 4294967296
    }
    if hash < 0 {
      return hash + 4294967296
    }
    return hash
  }

  @class
  hex(value: Int) -> String {
    const digits = "0123456789abcdef".codePoints
    const out = List.new()
    let remaining = value
    let position = 0
    while position < 8 {
      out.add(digits.at(remaining % 16))
      remaining = remaining ~/ 16
      position++
    }
    const ordered = List.new()
    let index = out.size - 1
    while index >= 0 {
      ordered.add(out.at(index))
      index--
    }
    return String.fromCodePoints(ordered)
  }
}
