import { ExtensionContext, languages } from "vscode"
import { PhalcomCompletionProvider } from "./completions"
import { createContext, destroyContext } from "./context"

export function activate(context: ExtensionContext) {

    let phalcomContext: PhalcomContext = createContext()
    let phalcomCompletionProvider = new PhalcomCompletionProvider(phalcomContext)

    context.subscriptions.push(languages.registerCompletionItemProvider('phalcom', phalcomCompletionProvider, '.'))
}

export function deactivate() {
    destroyContext()
}
