# Color scheme

How diagnostics use color. Companion to [`README.md`](README.md) §4.1, which covers the
mechanics (`--color`, `NO_COLOR`, TTY detection); this document covers what gets colored.

**Status:** design target. Gated on README §3.1 — the renderer decision (miette vs the incumbent
`color_print`) determines how this is expressed, not what it says.

---

## 1. The rule that governs everything else

**Color is emphasis, never information.**

Every distinction a reader needs must survive `--color=never`. Structure is carried by glyphs and
position — the `×` marker, the box rails, the caret, the `help:` prefix, indentation depth. Color
makes a correct rendering faster to scan; it never makes an uncolored one ambiguous.

Three reasons this is not negotiable:

- Roughly 8% of men have some form of red-green color deficiency. An error/success distinction
  carried only by hue is invisible to them.
- Output is piped into files, CI logs, and test fixtures constantly. Those are the uncolored path.
- Terminal themes vary enormously. A color that reads as "alarming red" in one theme is muddy
  brown in another.

Test for this: render every catalog example with color off and confirm nothing is lost. If
something is, the glyph layer is wrong — fix that, do not reach for a second color.

---

## 2. Semantic roles

Colors attach to **roles**, not to literal elements. A role gets one color everywhere it appears,
across tracebacks, compile errors, trace logs, and disassembly. This is what makes the whole tool
look like one program.

| Role | Applies to | ANSI | Weight |
|---|---|---|---|
| `severity.error` | `error:`, `×` marker | red | bold |
| `severity.warn` | `warning:` | yellow | bold |
| `severity.help` | `help:`, `note:` prefixes | cyan | bold |
| `location` | `shop.ph:3:48`, frame `file:line` | blue | — |
| `identifier` | frame names, selectors, class names | default | bold |
| `rail` | box drawing, gutters, `│ ╭ ╰ ·` | dim default | — |
| `line-number` | the ` 3 │` gutter number | dim default | — |
| `source` | the echoed source line | default | — |
| `span.primary` | the underline under the failing span | red | bold |
| `span.secondary` | a second label's underline | blue | — |
| `label` | text hanging off a caret | matches its span | — |
| `elision` | `[2 core frames elided …]` | dim default | italic |
| `chain` | `⤷ raised inside fiber #3 …` | magenta | — |

### Notes on specific choices

**`source` stays default.** The strongest instinct is to syntax-highlight the echoed line. Don't.
The span underline is the only thing that should draw the eye there; highlighting the whole line
competes with it and makes the actual error harder to find. The REPL highlights source because
the user is *writing* it — a diagnostic echoes source because the user is *locating* something in
it. Different jobs.

**`span.primary` is red and `span.secondary` is blue** — not two shades of red. Under
deuteranopia those would collapse; red/blue survives. The primary is also bold, so the ranking
holds with color off.

**`chain` is magenta** because a fiber boundary is genuinely a different kind of event from both a
frame and an error, and reusing either color makes it read as a subheading of the wrong thing.

**`elision` is dim and italic** so skipped content recedes without vanishing. A reader scanning
for their own code should skim past elided core frames; a reader debugging core should still spot
the line telling them how to expand it.

---

## 3. Palette discipline

**Use the 16 ANSI indices. Not 256, not truecolor.**

The indices are what the user's terminal theme remaps. Emitting `#d75f5f` overrides Solarized,
Nord, Gruvbox, and every high-contrast accessibility theme with one author's taste. Emitting
"index 1, bold" lets each of those render *their* red. The output looks native everywhere instead
of correct in one place.

Practical consequences:

- Never emit pure white or pure black. Both are unreadable on one of the two common backgrounds.
  "Default foreground" is the color that adapts.
- No background fills. They fight every theme and break selection and copy-paste.
- "Dim" means the SGR dim attribute, not a darker color. It composes with the user's theme.
- Bold is a real signal here and should be spent sparingly — severity, primary span, identifiers.
  Bold everywhere is bold nowhere.

---

## 4. Worked example

Catalog §1, annotated with roles:

```
Traceback (most recent call last):          ← severity.error (bold red), header only
  shop.ph:7   in <main>                     ← location (blue) + identifier (bold)
      cart.total                            ← source (default)
  shop.ph:2   in Cart.total
      total { self.sum(_items) }
  shop.ph:3   in Cart.sum(_)
      sum(items) { items.fold(0) { acc, it => acc + it.price } }
  [2 core frames elided — pass --trace-core to expand]    ← elision (dim italic)
  shop.ph:3   in <block in Cart.sum(_)>

  × 1 does not understand 'price'           ← × is severity.error; 'price' is identifier
   ╭─[shop.ph:3:48]                         ← rail (dim) + location (blue)
 3 │   sum(items) { items.fold(0) { acc, it => acc + it.price } }
   ·                                                ─────┬────   ← span.primary (bold red)
   ·                                                     ╰── Number has no method 'price'
   ╰────                                                          ← label (red, matches span)
  help: did you mean 'floor'?               ← severity.help (bold cyan) + identifier
```

With `--color=never` this is the same text, unchanged. Nothing above depends on hue.

---

## 5. Per-surface application

**Traceback** — as above. The innermost frame's caret block is the only place `span.*` appears;
outer frames are location + identifier + source only.

**Fiber switch log** — `[fiber]` prefix in `chain` magenta so trace lines are separable from
program stdout at a glance. Fiber ids in `identifier` bold. `spawn`/`switch`/`yield`/`done` in
default; `fail` in `severity.error`.

**Disassembly** — opcode mnemonics in `identifier` bold, operands default, `line N` in
`line-number` dim, the `└─` nesting rails in `rail`, capture annotations (`← captures: acc`) in
`elision` dim since they are commentary on the instruction rather than part of it.

**JSON trace stream** — never colored. It is a machine contract; escapes would corrupt it.
`--trace-format=json` must force `--color=never` for the stream regardless of TTY state.

---

## 6. Open

- **Renderer expression.** If miette is adopted (README §3.1) these roles map onto its severity
  and label system, and some are given rather than chosen. If the incumbent `color_print` path is
  extended, the roles need a small named-style layer so the markup does not scatter raw
  `<s,r!>`-style tags across every call site — which is the current pattern in
  [`diagnostics.rs`](../../../phalcom-core/src/diagnostics.rs) and does not survive a growing
  number of surfaces.
- **Whether `--color=always` should force the ASCII fallback off.** They are separate axes (color
  vs. glyph repertoire) and probably want separate flags, but a single `--plain` covering both is
  the friendlier surface. Decide before either ships.
