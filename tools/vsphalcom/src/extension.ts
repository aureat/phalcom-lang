import { ExtensionContext, workspace } from "vscode"
import { registerRunFile } from "./run"
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from "vscode-languageclient/node"

/**
 * The running `phalcom-lsp` client, if `phalcom.lsp.enabled` is `true`
 * (the default — U-LSP's 5 stages have superseded every TS provider this
 * extension used to carry: `diagnostics.ts`, `completions.ts`, `context.ts`,
 * `hover.ts`, all deleted; the regex TextMate grammar is now a fallback-only
 * colorizer, layered under by `textDocument/semanticTokens/full` whenever
 * the client is running).
 *
 * `undefined` when the flag is set `false` — an escape hatch with no TS
 * fallback left to degrade to (ADR-0056 §6 end state): a user who disables
 * the flag gets no diagnostics/completion/hover/go-to-def, just syntax
 * highlighting from the grammar.
 */
let lspClient: LanguageClient | undefined

/**
 * Resolves the `phalcom-lsp` server binary the same way `run.ts` resolves
 * the `phalcom` CLI: reads the `phalcom.lsp.serverPath` setting, defaulting
 * to `"phalcom-lsp"` resolved on `$PATH`.
 */
function getLspServerPath(): string {
    return workspace.getConfiguration("phalcom").get<string>("lsp.serverPath", "phalcom-lsp")
}

/**
 * Constructs and starts the `vscode-languageclient` `LanguageClient` that
 * spawns `phalcom-lsp` over stdio and registers it for `.ph` documents.
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
    registerRunFile(context)

    if (workspace.getConfiguration("phalcom").get<boolean>("lsp.enabled", true)) {
        lspClient = startLspClient(context)
    }
}

export function deactivate(): Thenable<void> | undefined {
    return lspClient?.stop()
}
