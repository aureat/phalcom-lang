import { ExtensionContext, languages } from "vscode"
import { PhalcomCompletionProvider } from "./completions"
import { createContext, destroyContext } from "./context"
import { registerDiagnostics } from "./diagnostics"
import { PhalcomHoverProvider } from "./hover"

export function activate(context: ExtensionContext) {

    let phalcomContext: PhalcomContext = createContext()
    let phalcomCompletionProvider = new PhalcomCompletionProvider(phalcomContext)
    let phalcomHoverProvider = new PhalcomHoverProvider()

    context.subscriptions.push(languages.registerCompletionItemProvider('phalcom', phalcomCompletionProvider, '.'))
    context.subscriptions.push(languages.registerHoverProvider('phalcom', phalcomHoverProvider))

    registerDiagnostics(context)
}

export function deactivate() {
    destroyContext()
}
