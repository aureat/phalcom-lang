import {
    HoverProvider,
    Hover,
    MarkdownString,
    TextDocument,
    Position,
    CancellationToken,
    ProviderResult,
    Range
} from "vscode"
import coreTable from "./generated/core-table.json"

/**
 * Shape of the harvested core table (`src/generated/core-table.json`),
 * produced by `phalcom-core/bin/gen-core-table` from `core.ph` and
 * `primitives.rs`. Selectors are in ADR-0012 comma-form: a positional
 * parameter renders as `_`, a keyword parameter renders as its label
 * (e.g. `move(_,to,duration)`). A getter has no parens (`toString`); a
 * setter is `name=(_)`.
 *
 * This mirrors the interface in `completions.ts` — kept as a separate
 * copy here (rather than an import) because `completions.ts` does not
 * export its internals.
 */
interface CoreTableEntry {
    selector: string
    kind: "method" | "getter" | "setter" | "construct" | "static-method"
    source: "core.ph" | "native"
}

interface CoreTable {
    keywords: string[]
    classes: { [className: string]: CoreTableEntry[] }
}

const table = coreTable as CoreTable

/**
 * One-line blurbs for each of Phalcom's closed-set keywords
 * (`table.keywords`). Grounded in `docs/spec/v0.2/lexical-structure.md`
 * §13 (error-handling keywords), `docs/spec/v0.2/classes.md` §1
 * (constructors), `docs/spec/v0.2/iteration.md` §2-3 (`for`/`break`/
 * `continue`), and `docs/spec/v0.2/object-model.md` §5 (metaclass tower).
 */
const keywordDocs: Record<string, string> = {
    "class": "Declares a class: a name, an optional `extends` superclass, fields (implicitly declared by `_`-prefixed assignment), constructors, and methods.",
    "extends": "Names a class's single superclass (Phalcom has single inheritance; `Object` is the root of every hierarchy).",
    "super": "Refers to the receiver viewed through its superclass's method dictionary — used for `super.foo(...)` sends and `super(...)` constructor delegation.",
    "self": "The current receiver inside a method or constructor body — the object the message was sent to.",
    "static": "Declares a member on the class's metaclass rather than on instances (a class-side method or field), per the metaclass tower (Object Model §5).",
    "try": "Starts a block-handler statement that is pure sugar over `Block` sends (`on(_)(_)`, `ensure(_)`, `attempt()`) — not a generic try/catch, it carries no semantics the desugaring lacks (lexical-structure.md §13).",
    "catch": "Catch-all clause of a `try` statement, sugar for `.on(Error){ e => ... }`; a contextual keyword, reserved only inside `try`-clauses.",
    "on": "Typed handler clause of a `try` statement (`on T e { ... }`), sugar for `.on(T){ e => ... }`; a contextual keyword, reserved only inside `try`-clauses.",
    "ensure": "Cleanup clause of a `try` statement, sugar for `.ensure{ ... }`, always runs; a contextual keyword, reserved only inside `try`-clauses.",
    "throw": "Prefix statement/expression that raises an `Error` value — sugar for `expr.raise()`.",
    "break": "Loop-control keyword: leaves the enclosing `for`/`while` loop immediately (lowers to a direct jump, not a block send).",
    "continue": "Loop-control keyword: skips to the next iteration step of the enclosing `for`/`while` loop (re-runs the cursor's `iterate(_)` step).",
    "match": "Names the Option/Result eliminator selector (`match(some,none)` / `match(ok,err)`) that leaves Option/Result-world with a value; reserved as a keyword for pattern-matching surface syntax.",
    "return": "Returns a value from the enclosing method/constructor/block explicitly.",
    "while": "Loops while its condition evaluates truthy; `for` desugars to a `while` loop over the iteration protocol.",
    "for": "Iterates a collection via the cursor protocol: `for (x in coll) { ... }` desugars to a `while` loop calling `iterate(_)`/`iteratorValue(_)`, so `break`/`continue` work as loop-control.",
    "var": "Introduces a mutable binding; can be reassigned. `var x` with no initializer reads as `None` (there is no uninitialized-variable error)."
}

/** A selector deduplicated across every core class that defines it, keyed by the exact selector string (ADR-0012: `foo` and `foo(_)` are distinct entries). */
interface FlatSelector {
    selector: string
    kind: CoreTableEntry["kind"]
    classes: string[]
    sources: Set<CoreTableEntry["source"]>
}

/**
 * Flattens `table.classes` into a selector -> {kind, classes, sources} map.
 * Same flattening shape as `completions.ts`'s `buildFlatSelectors`, kept as
 * an independent copy here since that module does not export its internals.
 * Built once at module load.
 */
function buildFlatSelectors(): Map<string, FlatSelector> {
    const flat = new Map<string, FlatSelector>()
    for (const className of Object.keys(table.classes)) {
        for (const entry of table.classes[className]) {
            const existing = flat.get(entry.selector)
            if (existing) {
                existing.classes.push(className)
                existing.sources.add(entry.source)
            } else {
                flat.set(entry.selector, {
                    selector: entry.selector,
                    kind: entry.kind,
                    classes: [className],
                    sources: new Set([entry.source])
                })
            }
        }
    }
    return flat
}

const flatSelectors = buildFlatSelectors()

/**
 * Splits a comma-form selector into its bare name and parameter slots.
 * A getter (no parens) yields `params: null`. A zero-arg method/setter
 * (e.g. `call()`) yields `params: []`. Same shape as `completions.ts`'s
 * `parseSelector`.
 */
function parseSelector(selector: string): { name: string; params: string[] | null } {
    const parenIndex = selector.indexOf("(")
    if (parenIndex === -1) {
        return { name: selector, params: null }
    }
    const closeIndex = selector.lastIndexOf(")")
    const name = selector.slice(0, parenIndex)
    const inner = selector.slice(parenIndex + 1, closeIndex)
    const params = inner.length ? inner.split(",").map(p => p.trim()) : []
    return { name, params }
}

/**
 * Every flattened selector whose bare name matches `name`, sorted for
 * stable rendering. A bare name can carry several distinct selectors
 * (different arity/kind/labels) per ADR-0012 — e.g. `perform(_)` and
 * `perform(_,_)` — each rendered as its own signature line, never
 * collapsed.
 */
function selectorsForName(name: string): FlatSelector[] {
    const matches: FlatSelector[] = []
    for (const flat of flatSelectors.values()) {
        if (parseSelector(flat.selector).name === name) {
            matches.push(flat)
        }
    }
    matches.sort((a, b) => a.selector.localeCompare(b.selector))
    return matches
}

/**
 * Renders the contract-view layer of a hover: selector -> summary ->
 * signature+types -> requires -> ensures -> invariant -> raises -> detail
 * -> example (`docs/spec/v0.2/experimental/doc-comments-phaldoc.md` §8).
 *
 * TODO: gated on U-ANNOT-CONTRACTS landing upstream — see
 * docs/spec/v0.2/experimental/doc-comments-phaldoc.md §8 for the harvest
 * order this will implement (selector -> summary -> signature+types ->
 * requires -> ensures -> invariant -> raises -> detail -> example). Always
 * returns `undefined` today; the `@`-attribute contract parser it depends
 * on does not exist yet. Wired into the hover composition path (below) so
 * the contract layer can drop in later without restructuring the pipeline.
 */
function renderContractView(selector: string): string | undefined {
    return undefined
}

/**
 * A harvested `///` doc block: the summary paragraph (first
 * blank-line-delimited paragraph of the block's markdown body) and the
 * 0-based line number of the declaration it documents (the next
 * non-blank, non-`///` line — adjacency per doc-comments-phaldoc.md
 * §Decision/§5).
 */
interface PhaldocBlock {
    summary: string
    declLine: number
}

/**
 * Scans a document's raw text for `///` (outer doc) blocks and associates
 * each with the declaration line immediately below it, per
 * `docs/spec/v0.2/experimental/doc-comments-phaldoc.md`:
 *
 * - `///` documents the item on the **next non-blank, non-`///` line**.
 *   A blank line between the block and that line breaks adjacency — the
 *   block is a dangling doc and is dropped.
 * - `//!` is an *inner* doc (documents the enclosing scope, not a
 *   below-line item) and is never treated as an outer `///` block.
 * - The first blank-line-delimited paragraph of the block's markdown body
 *   is the summary; that's all this leg surfaces (no `@tag` rendering).
 *
 * This is a per-document scanner only (no project-wide index), so it can
 * only attach a summary when the hover target's line IS the declaration
 * line a `///` block documents in the *currently open* document.
 */
function harvestPhaldocBlocks(document: TextDocument): PhaldocBlock[] {
    const blocks: PhaldocBlock[] = []
    const lineCount = document.lineCount
    let i = 0
    while (i < lineCount) {
        const text = document.lineAt(i).text
        const trimmed = text.trim()

        // `//!` is an inner doc — never parsed as an outer `///` block.
        if (trimmed.startsWith("//!")) {
            i++
            continue
        }

        if (trimmed.startsWith("///")) {
            const bodyLines: string[] = []
            let j = i
            while (j < lineCount) {
                const lineText = document.lineAt(j).text.trim()
                if (!lineText.startsWith("///")) {
                    break
                }
                // Strip the leading `///` marker and at most one following space.
                bodyLines.push(lineText.replace(/^\/\/\/\s?/, ""))
                j++
            }

            // The next line after the block, if non-blank and not itself
            // another doc-marker line, is the declaration adjacency
            // requires. A blank line breaks adjacency (dangling doc).
            if (j < lineCount) {
                const nextLineText = document.lineAt(j).text
                if (nextLineText.trim().length > 0) {
                    const summary = firstParagraph(bodyLines)
                    if (summary.length > 0) {
                        blocks.push({ summary, declLine: j })
                    }
                }
            }

            i = j
            continue
        }

        i++
    }
    return blocks
}

/** Extracts the first blank-line-delimited paragraph from a `///` block's body lines. */
function firstParagraph(bodyLines: string[]): string {
    const paragraph: string[] = []
    for (const line of bodyLines) {
        if (line.trim().length === 0) {
            break
        }
        paragraph.push(line)
    }
    return paragraph.join(" ").trim()
}

/**
 * Finds the Phaldoc summary (if any) attached to `line` in `document` —
 * i.e. a `///` block whose declaration-adjacency line is exactly `line`.
 * Per the scanner's scope (see `harvestPhaldocBlocks`), this only resolves
 * when the hover target's line is itself a declaration line documented by
 * a `///` block in the currently open document; use-sites elsewhere in
 * the file, or selectors with no local declaration, yield `undefined`.
 */
function findPhaldocSummaryForLine(document: TextDocument, line: number): string | undefined {
    const blocks = harvestPhaldocBlocks(document)
    const block = blocks.find(b => b.declLine === line)
    return block ? block.summary : undefined
}

export class PhalcomHoverProvider implements HoverProvider {

    provideHover(
        document: TextDocument,
        position: Position,
        token: CancellationToken): ProviderResult<Hover>
    {
        const wordRange = document.getWordRangeAtPosition(position)
        if (!wordRange) {
            return undefined
        }
        const word = document.getText(wordRange)
        if (!word) {
            return undefined
        }

        // Keyword hover.
        if (Object.prototype.hasOwnProperty.call(keywordDocs, word)) {
            const md = new MarkdownString()
            md.appendMarkdown(`**${word}** _(keyword)_\n\n${keywordDocs[word]}`)
            return new Hover(md, wordRange)
        }

        // Selector signature hover.
        const matches = selectorsForName(word)
        if (matches.length > 0) {
            return new Hover(renderSelectorHover(document, wordRange, matches), wordRange)
        }

        return undefined
    }
}

/**
 * Composes the hover markdown for a matched selector name: one signature
 * line per distinct selector (kind + defining class(es)), a Phaldoc
 * summary when the hover line is itself a local `///`-documented
 * declaration, and the (currently inert) contract-view seam.
 */
function renderSelectorHover(document: TextDocument, wordRange: Range, matches: FlatSelector[]): MarkdownString {
    const md = new MarkdownString()
    md.isTrusted = false

    const lines = matches.map(flat => {
        const sources = Array.from(flat.sources).join(", ")
        return `\`${flat.selector}\` — ${flat.kind} on ${flat.classes.join(", ")} (${sources})`
    })
    md.appendMarkdown(lines.join("\n\n"))

    // Phaldoc layer: only resolves when the hovered line is itself a
    // local declaration line documented by a `///` block in this
    // document (see `findPhaldocSummaryForLine`).
    const summary = findPhaldocSummaryForLine(document, wordRange.start.line)
    if (summary) {
        md.appendMarkdown(`\n\n---\n\n${summary}`)
    }

    // Contract-view seam — always inert today, wired so it drops in later
    // without restructuring this composition path.
    for (const flat of matches) {
        const contractView = renderContractView(flat.selector)
        if (contractView) {
            md.appendMarkdown(`\n\n---\n\n${contractView}`)
        }
    }

    return md
}
