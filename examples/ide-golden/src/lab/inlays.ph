const /*@inlay.value.inferred*/inferredValue = 42
const /*@inlay.value.explicit*/explicitValue: Int = 42

class InlayLab {
  inferred(_ /*@inlay.parameter.inferred*/value) {
    value
  }

  explicit(_ /*@inlay.parameter.explicit*/value: Int) -> Int {
    value
  }
}

export inferredValue, explicitValue, InlayLab
