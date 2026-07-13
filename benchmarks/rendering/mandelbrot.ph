// ============================================================
// mandelbrot — ASCII Mandelbrot renderer, ported from Wren.
// Benchmark corpus — NOT wired into CI. Run directly:
//   cargo run -r -p phalcom-core --bin phalcom -- benchmarks/rendering/mandelbrot.ph
// Not self-verifying via identities like benchmarks/math/* — correctness is
// eyeballing the rendered fractal against the known Wren-original output
// (symmetric bulb shape, same escape-time shading).
//
// Deviations from the original Wren source (docs/wren mirror), and why:
//   - `" .:;+=xX$&"[(iter / 8).floor]` — Phalcom's String has no `[]`
//     indexing and no `.at(_)` (primitives are only hash/add/new — see
//     phalcom-core/src/primitive/string.rs), and Number has no `.floor` at
//     all. Worked around with a `List` of one-char strings + `.at(idx)`,
//     and `.floor` emulated as `(iter - (iter % 8)) / 8` (Modulo exists).
//   - `System.write` — does not exist (only `System.print`, which always
//     appends a newline). Rows are accumulated as a string and printed once
//     per line instead of streamed pixel-by-pixel.
//   - No `[]` postfix indexing operator anywhere in the parser (only
//     `[a, b, c]` list *literals* — Token::LBracket is wired to
//     parse_list_literal, never to a postfix-index production).
// ============================================================

var chars = List.new().add(" ").add(".").add(":").add(";").add("+").add("=").add("x").add("X").add("$").add("&")

var yMin = 0 - 0.2
var yMax = 0.1
var xMin = 0 - 1.5
var xMax = 0 - 1.1

var yPixel = 0
while (yPixel < 24) {
  var y = (yPixel / 24) * (yMax - yMin) + yMin
  var row = ""
  var xPixel = 0
  while (xPixel < 80) {
    var x = (xPixel / 79) * (xMax - xMin) + xMin
    var pixel = " "
    var x0 = x
    var y0 = y
    var iter = 0
    var escaped = false
    while (iter < 80 and (escaped == false)) {
      var x1 = (x0 * x0) - (y0 * y0)
      var y1 = 2 * x0 * y0

      x1 = x1 + x
      y1 = y1 + y

      x0 = x1
      y0 = y1

      var d = (x0 * x0) + (y0 * y0)
      if (d > 4) {
        var idx = (iter - (iter % 8)) / 8
        pixel = chars.at(idx)
        escaped = true
      }
      iter = iter + 1
    }

    row = row + pixel
    xPixel = xPixel + 1
  }

  System.print(row)
  yPixel = yPixel + 1
}
