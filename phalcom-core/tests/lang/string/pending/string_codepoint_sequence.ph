// area: string
// spec: core/core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.4; ADR-0048
// status: PENDING
// Wren precedent: test/core/string_byte_sequence/*.wren,
// test/core/string_code_point_sequence/*.wren (~30 files). Wren splits a
// string's iteration into two *distinct* sequence types (raw byte vs decoded
// Unicode codepoint) reached via `.bytes`/`.codePoints` and Wren's native
// `[]` subscript sugar — Phalcom has no subscript operator at all (confirmed
// permanently out of scope, `docs/forge/units/U-STRING/plan.md` §0: "Wren's
// `this[a...b]` / `this[i]` sugar has no Phalcom equivalent and adding one is
// out of scope") and today has no `bytes`/`codePoints` accessors, no
// `byteAt_`/`codePointAt` at all. Rather than mechanically porting every
// per-selector Wren file (`iterate`, `iterator_value`, `subscript`, and their
// four OOB/wrong-type variants, times two sequence types), this single
// fixture folds the *meaningful* behavior — unicode-aware codepoint
// iteration over a string mixing 1/2/3/4-byte UTF-8 sequences — into one
// representative case, matching the exact mixed-width example
// (`"a\u{20AC}\u{1F389}"`, i.e. ASCII + EUR SIGN + PARTY POPPER) the
// U-STRING plan itself pins for `codePointAt` correctness (§2.1 rubric).
// Written as literal UTF-8 source text, not a `\u{...}` escape — Phalcom's
// lexer has no unicode-escape syntax at all (only `\\` and the `\(expr)`
// interpolation escape; string_concatenation.ph's note), and source files
// are read as UTF-8 already (confirmed working today via non-ASCII content,
// string_equality.ph's "vålue" case).
// The raw byte-sequence half (`.bytes`) is not ported separately — it is a
// dense `0..byteCount_` walk with no unicode subtlety, adequately covered
// by U-STRING's own corpus once it lands.
let cps = "a€🎉".codePoints
System.print(cps.size)
cps.each { cp => System.print(cp) }
