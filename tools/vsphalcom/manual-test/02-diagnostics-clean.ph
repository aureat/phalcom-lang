// 02-diagnostics-clean.ph
//
// Valid syntax. Save this file with `phalcom.executablePath` set to a
// working `phalcom` binary (e.g. target/debug/phalcom): no diagnostic
// squiggle should appear anywhere. If one already exists from a previous
// test file, saving this one should clear it for this file's URI.

class Counter {
  construct new() {
    _value = 0
  }

  increment {
    self._value = self._value + 1
    return self
  }

  value {
    return self._value
  }
}
