// area: string
// spec: core/core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.3
// status: PENDING
// Wren precedent: test/core/string/split.wren, trim.wren, multiply.wren.
// `String` has no `split(_)`/`trim()`/`*(count)` today — the native floor is
// exactly `+(_)`/`hash`/`toString`/static `new` (primitive/string.rs, 53
// lines; confirmed by `docs/forge/units/U-STRING/plan.md` §0: "no length, no
// index, no byte accessor at all today"). This fixture folds three of Wren's
// per-method files into one representative case pinning the intended
// surface once U-STRING lands: `split(_)` returns a `List` of `String`
// segments (Wren's own worked example, `"a,b,,c".split(",")`, is the
// mandatory cross-check U-STRING's own test-strategy section names), `trim()`
// strips leading/trailing whitespace, and `*(count)` repeats the receiver
// `count` times. Not a mechanical 1:1 port of every Wren split/trim/multiply
// file (empty-delimiter guards, custom trim charsets, negative/fractional
// counts) — those become U-STRING's own `strings` corpus once it lands
// (see plan.md §6); this is the single "does the shape work" smoke case.
const parts = "a,b,,c".split(",")
System.print(parts.size)
System.print(parts.at(0))
System.print(parts.at(1))
System.print(parts.at(2))
System.print(parts.at(3))
System.print("  hi  ".trim())
System.print("ab" * 3)
