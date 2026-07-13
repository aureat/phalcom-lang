import { ExtensionContext, languages } from "vscode"
import { PhalcomCompletionProvider } from "./completions"
import { createContext, destroyContext } from "./context"
import { registerDiagnostics } from "./diagnostics"

export function activate(context: ExtensionContext) {

    let phalcomContext: PhalcomContext = createContext()
    let phalcomCompletionProvider = new PhalcomCompletionProvider(phalcomContext)

    context.subscriptions.push(languages.registerCompletionItemProvider('phalcom', phalcomCompletionProvider, '.'))

    registerDiagnostics(context)
}

export function deactivate() {
    destroyContext()
}
