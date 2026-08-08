// Typed bundle descriptors. A Bundle owns no process-global values: every
// evaluation stores references in its fresh _StatefulContext.

@data
@immutable
class Bundle<T> {
  const _name: Symbol
  const _elementType: Option<Any>

  @constructor
  new(name: Symbol) {
    _name = name
    _elementType = None
  }

  @constructor
  new(name: Symbol, elementType: Any) {
    _name = name
    _elementType = Some.new(elementType)
  }

  select -> _BundleSelection {
    return _BundleSelection.new(bundle: self, consuming: false)
  }

  consume -> _BundleSelection {
    return _BundleSelection.new(bundle: self, consuming: true)
  }

  publish -> _BundleTarget {
    return _BundleTarget.new(bundle: self)
  }

  fingerprint -> String {
    let typePart = "dynamic"
    if _elementType.isSome || {
      typePart = _elementType.unwrap.toString
    }
    return "bundle(" + _name.toString + ":" + typePart + ")"
  }

  toString -> String => "Bundle<" + _name.toString + ">"
}

@data
@immutable
class _BundleSelection {
  const _bundle: Bundle<Any>
  const _consuming: Bool

  fingerprint -> String {
    if _consuming {
      return "consume(" + _bundle.fingerprint + ")"
    }
    return "select(" + _bundle.fingerprint + ")"
  }
}

@data
@immutable
class _BundleTarget {
  const _bundle: Bundle<Any>

  fingerprint -> String => "publish(" + _bundle.fingerprint + ")"
}
