import coreTable from "./generated/core-table.json"
import { parseSignatureString } from "./util"

/**
 * `PhalcomContext` predates the harvested-table completion rewrite
 * (U-VSPHALCOM-2 leg 2) and is retained only because
 * `PhalcomCompletionProvider`'s constructor (called from `extension.ts`)
 * still takes one. The current `completions.ts` is table-driven and does
 * not read receiver types out of this context, so this file now builds a
 * minimal, non-stale context straight from `core-table.json` instead of
 * the deleted `src/language.ts` type table.
 */
export const phalcomGlobal: PhalcomGlobal = {
    context: null
}

export function createContext(): PhalcomContext {
    let context: PhalcomContext = {
        globals: {},
        locals: {},
        types: {},
    }

    // Register every harvested core class as a known type/global, so the
    // context shape stays meaningful even though nothing narrows
    // completions by it yet.
    Object.keys(coreTable.classes).forEach(className => {
        const entries = (coreTable.classes as { [name: string]: { selector: string }[] })[className]
        context.types[className] = {
            super: null,
            type: null,
            methods: entries.map(entry => ({
                signatureString: entry.selector,
                signature: parseSignatureString(entry.selector)
            }))
        }
        context.globals[className] = className
    })

    // `self`/`super` remain meaningful pseudo-locals inside a method body.
    context.locals["self"] = "Object"
    context.locals["super"] = "Object"

    phalcomGlobal.context = context

    return context
}

export function destroyContext(): void {
    phalcomGlobal.context = null
}
