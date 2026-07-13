// 03-diagnostics-error.ph
//
// Deliberately broken syntax. Saving this file should produce exactly one
// red squiggle, anchored at the `=` with nothing after it (an incomplete
// `let` initializer). Check the message text matches what `phalcom check
// -s '...' --format json` reports for the same snippet on the command line.
//
// Try editing the error away (e.g. add `1` after `=`) and re-saving — the
// squiggle should clear.

class Broken {
  test {
    let x =
    return x
  }
}
