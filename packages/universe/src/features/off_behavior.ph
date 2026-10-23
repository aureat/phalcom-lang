/// Configurable behavior for disabled feature flags.
class OffBehavior {
  mut _kind
  mut _payload

  /// Raises an error when the feature flag is disabled.
  @class
  raise -> OffBehavior { OffBehavior.new("raise", Option<Dynamic>::None) }

  /// Falls back to invoking an alternative selector when disabled.
  @class
  fallback(_ sel: Symbol) -> OffBehavior { OffBehavior.new("fallback", Option<Symbol>::Some(sel)) }

  /// Skips execution and returns a default value when disabled.
  @class
  skip(_ value: Dynamic) -> OffBehavior { OffBehavior.new("skip", Option<Dynamic>::Some(value)) }

  @constructor
  new(_ kind: String, _ payload: Dynamic) {
    _kind = kind
    _payload = payload
  }

  /// Kind of off-behavior (`"raise"`, `"fallback"`, or `"skip"`).
  kind -> String { _kind }

  /// Payload value or fallback selector associated with this behavior.
  payload -> Dynamic { _payload }
}

export OffBehavior
