import {
    CompletionItemProvider,
    CompletionItemKind,
    TextDocument,
    Position,
    CompletionItem,
    CancellationToken,
    ProviderResult,
    SnippetString
} from "vscode"
import { isPositionInComment, isPositionInString } from "./util"
import coreTable from "./generated/core-table.json"

/**
 * Shape of the harvested core table (`src/generated/core-table.json`),
 * produced by `phalcom-core/bin/gen-core-table` from `core.ph` and
 * `primitives.rs`. Selectors are in ADR-0012 comma-form: a positional
 * parameter renders as `_`, a keyword parameter renders as its label
 * (e.g. `move(_,to,duration)`). A getter has no parens (`toString`); a
 * setter is `name=(_)`.
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

export const PhalcomCompletionKind = {
    Keyword: CompletionItemKind.Keyword,
    Operator: CompletionItemKind.Operator,
    Type: CompletionItemKind.Class,
    Var: CompletionItemKind.Variable,
    Method: CompletionItemKind.Method,
    Field: CompletionItemKind.Field,
    Property: CompletionItemKind.Property,
    Constructor: CompletionItemKind.Constructor,
    PseudoVar: CompletionItemKind.Value,
    PseudoField: CompletionItemKind.Struct
}

/** Maps a harvested selector's `kind` to the closest VS Code completion kind. */
const kindToCompletionKind: { [kind in CoreTableEntry["kind"]]: CompletionItemKind } = {
    "method": PhalcomCompletionKind.Method,
    "static-method": PhalcomCompletionKind.Method,
    "getter": PhalcomCompletionKind.Field,
    "setter": PhalcomCompletionKind.Property,
    "construct": PhalcomCompletionKind.Constructor
}

/** A selector deduplicated across every core class that defines it, for the flat completion set. */
interface FlatSelector {
    selector: string
    kind: CoreTableEntry["kind"]
    classes: string[]
    sources: Set<CoreTableEntry["source"]>
}

/**
 * Flattens `table.classes` into a selector -> {kind, classes, sources} map.
 *
 * This unit has no type inference (out of scope, see U-VSPHALCOM plan), so
 * completions are not narrowed by receiver type: the same selector offered
 * on `List` is offered on `String`. Built once at module load.
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
 * (e.g. `call()`) yields `params: []`.
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
 * Builds a tab-stop snippet insert text from a comma-form selector, so the
 * user can tab through each argument position. A positional slot (`_`)
 * becomes `${n:_}`; a keyword slot (`to`) becomes `to: ${n:_}`. Getters
 * (no parens) insert as their bare name.
 */
function getInsertTextFromSelector(selector: string): string {
    const { name, params } = parseSelector(selector)
    if (params === null) {
        return name
    }
    if (params.length === 0) {
        return `${name}()`
    }
    const slots = params.map((param, index) => {
        const tabStop = index + 1
        return param === "_" ? `\${${tabStop}:_}` : `${param}: \${${tabStop}:_}`
    })
    return `${name}(${slots.join(", ")})`
}

export class PhalcomCompletionProvider implements CompletionItemProvider {

    // `phalcomContext` is retained only so the constructor call in
    // extension.ts (`new PhalcomCompletionProvider(createContext())`)
    // keeps compiling; this leg's completions are table-driven and do not
    // narrow by receiver type, so the context is otherwise unused here.
    constructor(private phalcomContext: PhalcomContext) {
    }

    provideCompletionItems(
        document: TextDocument,
        position: Position,
        token: CancellationToken): ProviderResult<CompletionItem[]>
    {
        const lineText = document.lineAt(position.line).text
        const lineTillCurrentPosition = lineText.substring(0, position.character)

        // Check if we're in a comment or string
        if (isPositionInComment(document, position) || isPositionInString(document, position)) {
            return []
        }

        // Get current word and check if it's a number
        const currentWord = getCurrentWord(document, position)
        if (currentWord.match(/^\d+$/)) {
            return []
        }

        const completions: CompletionItem[] = []

        // After a `.`, offer the full flat selector set regardless of the
        // (empty) current word, since there is no receiver-type narrowing.
        if (lineTillCurrentPosition.endsWith(".")) {
            completions.push(...getSelectorCompletions(""))
            return completions
        }

        completions.push(...getKeywordCompletions(currentWord))
        completions.push(...getSelectorCompletions(currentWord))

        return completions
    }
}

function getCurrentWord(document: TextDocument, position: Position): string {
    const wordAtPosition = document.getWordRangeAtPosition(position)
    let currentWord = ''
    if (wordAtPosition && wordAtPosition.start.character < position.character) {
        const word = document.getText(wordAtPosition)
        currentWord = word.substring(0, position.character - wordAtPosition.start.character)
    }

    return currentWord
}

function getKeywordCompletions(currentWord: string): CompletionItem[] {
    if (!currentWord.length) {
        return []
    }

    return table.keywords.filter(keyword => keyword.startsWith(currentWord)).map(keyword => {
        return new CompletionItem(keyword, PhalcomCompletionKind.Keyword)
    })
}

/**
 * Builds the flat core-class selector completion list, filtered by the
 * selector's bare name starting with `currentWord`. Pass `""` to get every
 * selector unfiltered (e.g. right after a `.` trigger character).
 */
function getSelectorCompletions(currentWord: string): CompletionItem[] {
    const items: CompletionItem[] = []
    for (const flat of flatSelectors.values()) {
        const { name } = parseSelector(flat.selector)
        if (currentWord.length && !name.startsWith(currentWord)) {
            continue
        }
        const item = new CompletionItem(flat.selector, kindToCompletionKind[flat.kind])
        // Must be a SnippetString, not a plain string — a plain string
        // pastes `${1:_}` literally instead of becoming a live tab-stop.
        item.insertText = new SnippetString(getInsertTextFromSelector(flat.selector))
        item.detail = `${flat.kind} on ${flat.classes.join(", ")}`
        item.documentation = `Core selector ${flat.selector} (${Array.from(flat.sources).join(", ")}).`
        items.push(item)
    }
    return items
}
