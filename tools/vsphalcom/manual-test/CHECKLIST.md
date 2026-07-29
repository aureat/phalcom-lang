# vsphalcom manual verification checklist

No VS Code / Antigravity dev host was available during implementation — every
leg below was checked via `tsc --noEmit` and by tracing real `phalcom check` /
`core-table.json` output by hand, never by actually loading the extension.
This folder plus this checklist closes that gap.

## Setup

```
cd tools/vsphalcom
npm install
npm run compile
```

Open `tools/vsphalcom` as the IDE workspace root, then **Run > Start
Debugging** (F5). This launches an Extension Development Host window with
the extension loaded live — no packaging/install needed for this pass.
(Works identically in VS Code and Antigravity — same extension host.)

In the Dev Host window, open this `manual-test/` folder and go through the
fixtures below in order.

Before the diagnostics tests: set `phalcom.executablePath` (Settings, search
"phalcom") to a real built binary, e.g. the repo-root-relative
`target/debug/phalcom` (build it first: `cargo build -p phalcom-core --bin
phalcom` from the repo root).

## 1. Syntax highlighting — `01-syntax-highlighting.ph`

Run **Developer: Inspect Editor Tokens and Scopes** (Command Palette) on
each of these tokens; confirm a distinct, specific scope (not bare
`source.phalcom`) and correct coloring per your theme:

- [ ] `class is super self static try catch on ensure throw break
      continue return while for var` — keyword scope
- [ ] `const` / `in` / `and` / `or` / `not` — nowhere in the file, and
      if you type them ad hoc, NOT colored as keywords (dead 2023 keywords)
- [ ] `_balance`, `_owner` — field scope, distinct from a bare identifier
- [ ] `@requires(...)`, `@ensures(...)` — attribute scope
- [ ] `#deposit`, `#deposit(_)`, `#==` — symbol scope
- [ ] `self::deposit`, `self::#deposit(_)` — method-reference scope
- [ ] `opt?.name`, `opt ?? "unknown"` — Option-operator scope
- [ ] `list[0]` (both read and `list[0] = ...` write) — index scope
- [ ] `"\(self._owner)"` — the `\(...)` part scoped distinctly from the
      surrounding string text
- [ ] `///` doc block above `class Account` — documentation-comment scope,
      visibly different from the plain `//` comments elsewhere
- [ ] `//!` inner-doc line — same documentation scope family as `///`,
      distinct from plain `//`

## 2. Diagnostics — `02-diagnostics-clean.ph`, `03-diagnostics-error.ph`

- [ ] Open and save `02-diagnostics-clean.ph` — no squiggle appears
- [ ] Open and save `03-diagnostics-error.ph` — exactly one red squiggle,
      anchored at the incomplete `let x =` line
- [ ] Message text is readable and matches what the CLI reports:
      `./target/debug/phalcom check -s 'let x =' --format json` (run from
      repo root) — compare the `message` field
- [ ] Fix the error (add e.g. `1` after `=`), save — squiggle clears
- [ ] Point `phalcom.executablePath` at a bogus path (e.g. `/nonexistent`),
      save either file — no error popup; check **Output > Phalcom** channel
      logs the missing-binary message instead

## 3. Autocomplete — `04-autocomplete.ph`

- [ ] Type `cla` on the marked blank line — `class` offered as a keyword
      completion
- [ ] Type `self.` inside `Probe.check` — full core-selector list appears;
      find and accept `isA(_)` — snippet inserts `isA(${1:_})` with a live
      tab-stop
- [ ] Find any multi-label selector in
      `tools/vsphalcom/src/generated/core-table.json` (grep for a comma
      inside a `"selector"` value) and confirm its completion renders
      keyword slots as `label: ${n:_}`, not just `${n:_}`

## 4. Hover — `05-hover.ph`

- [ ] Hover `throw` — keyword blurb mentioning it desugars to `expr.raise()`
- [ ] Hover `isA` on `obj.isA(Number)` — signature hover:
      `isA(_) — method on Object (core.ph)`
- [ ] Hover `describe` on its own declaration line — the `///` summary
      paragraph above it appears in the hover (only the first paragraph,
      not the second)
- [ ] Hover `describe` at the `callSite` call site (not the declaration) —
      no doc summary attached (single-document scanner, not a project
      index — expected, not a bug)
- [ ] Confirm no "contract" / "requires" / "ensures" section ever renders
      in any hover — the `renderContractView` seam is wired but inert until
      `U-ANNOT-CONTRACTS` lands upstream

## Known-out-of-scope (don't file these as bugs)

- No go-to-definition / find-references (needs a workspace symbol index +
  a real `phalcom-lsp` crate — future unit)
- No receiver-type-narrowed completions (flat core-selector set only)
- No `@param`/`@returns`/`@throws` tag rendering in hover (summary
  paragraph only)
- No `@requires`/`@ensures` contract-view hover (stub only, gated on
  `U-ANNOT-CONTRACTS`)
- TextMate grammar is regex-approximate, not parser-accurate (e.g. the `#`
  whitespace-adjacency rule is approximated) — DEC-VSP-C, accepted

Report anything that fails an item above (or crashes, or throws in the
Output/Console) — everything else is expected partial scope, see
`docs/forge/units/U-VSPHALCOM/plan.md` for the full unit breakdown.
