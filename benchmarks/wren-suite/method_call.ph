// Ported from wren/test/benchmark/method_call.wren. `is` inheritance keyword
// -> `extends`; `!x` -> `not x` (Phalcom spells logical negation `not`);
// bare `super(startState)` call -> `super.new(startState)`
// (Phalcom constructors are dispatched by selector, so the super initializer
// must be named explicitly). `0...n` range-in-for replaced with a
// while-counter. `this` -> `self` (Phalcom's self-reference keyword).
class Toggle {
  construct new(startState) {
    _state = startState
  }

  value => _state
  activate {
    _state = not _state
    return self
  }
}

class NthToggle extends Toggle {
  construct new(startState, maxCounter) {
    super.new(startState)
    _countMax = maxCounter
    _count = 0
  }

  activate {
    _count = _count + 1
    if (_count >= _countMax) {
      super.activate
      _count = 0
    }

    return self
  }
}

let n = 100000
let val = true
let toggle = Toggle.new(val)

let i = 0
while (i < n) {
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  val = toggle.activate.value
  i = i + 1
}

System.print(toggle.value)

val = true
let ntoggle = NthToggle.new(val, 3)

i = 0
while (i < n) {
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  val = ntoggle.activate.value
  i = i + 1
}

System.print(ntoggle.value)
