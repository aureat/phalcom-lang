import { ExtensionContext, languages, workspace } from "vscode"
import { PhalcomCompletionProvider } from "./completions"
import { createContext, destroyContext } from "./context"
import { registerDiagnostics } from "./diagnostics"
import { PhalcomHoverProvider } from "./hover"
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from "vscode-languageclient/node"

/**
 * The running `phalcom-lsp` client, if `phalcom.lsp.enabled` is `true`.
 *
 * `undefined` when the flag is off (the default) — the subprocess/static-table
 * providers registered by {@link activate} are the only intelligence running
 * in that case. Held here so {@link deactivate} can shut the client down.
 */
let lspClient: LanguageClient | undefined

/**
 * Resolves the `phalcom-lsp` server binary the same way {@link diagnostics.ts}
 * resolves the `phalcom` CLI: reads the `phalcom.lsp.serverPath` setting,
 * defaulting to `"phalcom-lsp"` resolved on `$PATH`. Mirrors the
 * `phalcom.executablePath` config-flag pattern so both binaries are
 * configured the same way.
 */
function getLspServerPath(): string {
    return workspace.getConfiguration("phalcom").get<string>("lsp.serverPath", "phalcom-lsp")
}

/**
 * Constructs and starts the `vscode-languageclient` `LanguageClient` that
 * spawns `phalcom-lsp` over stdio and registers it for `.ph` documents.
 *
 * Stage 1 of ADR-0056: this is purely a transport/launch shim — capability
 * behavior lives entirely server-side in the `phalcom-lsp` crate. Per §6
 * ("the two never both publish"), the caller must gate the superseded
 * TS provider (`diagnostics.ts`) off when this is on — see {@link activate}.
 */
function startLspClient(context: ExtensionContext): LanguageClient {
    const command = getLspServerPath()

    const serverOptions: ServerOptions = {
        run: { command, transport: TransportKind.stdio },
        debug: { command, transport: TransportKind.stdio }
    }

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "phalcom" }]
    }

    const client = new LanguageClient("phalcomLsp", "Phalcom Language Server", serverOptions, clientOptions)

    context.subscriptions.push(client)
    void client.start()

    return client
}

export function activate(context: ExtensionContext) {

    let phalcomContext: PhalcomContext = createContext()
    let phalcomCompletionProvider = new PhalcomCompletionProvider(phalcomContext)
    let phalcomHoverProvider = new PhalcomHoverProvider()

    context.subscriptions.push(languages.registerCompletionItemProvider('phalcom', phalcomCompletionProvider, '.'))
    context.subscriptions.push(languages.registerHoverProvider('phalcom', phalcomHoverProvider))

    if (workspace.getConfiguration("phalcom").get<boolean>("lsp.enabled", false)) {
        lspClient = startLspClient(context)
    } else {
        registerDiagnostics(context)
    }
}

export function deactivate(): Thenable<void> | undefined {
    destroyContext()
    return lspClient?.stop()
}
