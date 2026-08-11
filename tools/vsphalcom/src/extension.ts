import { commands, ExtensionContext, OutputChannel, window, workspace } from "vscode"
import { existsSync } from "node:fs"
import { join } from "node:path"
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
let lspOutput: OutputChannel | undefined

/**
 * Resolves the `phalcom-lsp` server binary the same way `run.ts` resolves
 * the `phalcom` CLI: reads the `phalcom.lsp.serverPath` setting, defaulting
 * to `"phalcom-lsp"` resolved on `$PATH`.
 */
function getLspServerPath(context: ExtensionContext): string {
    const configured = workspace.getConfiguration("phalcom").get<string>("lsp.serverPath", "").trim()
    if (configured) {
        return configured
    }

    const executable = process.platform === "win32" ? "phalcom-lsp.exe" : "phalcom-lsp"
    const bundled = join(context.extensionPath, "server", `${process.platform}-${process.arch}`, executable)
    return existsSync(bundled) ? bundled : "phalcom-lsp"
}

function readInitializationOptions() {
    const config = workspace.getConfiguration("phalcom")
    return {
        phalcom: {
            lsp: {
                sysrootPath: config.get<string>("lsp.sysrootPath", "")
            },
            inlayHints: {
                types: config.get<string>("inlayHints.types", "stable"),
                suppressObvious: config.get<boolean>("inlayHints.suppressObvious", true)
            }
        }
    }
}

/**
 * Constructs and starts the `vscode-languageclient` `LanguageClient` that
 * spawns `phalcom-lsp` over stdio and registers it for `.ph` documents.
 */
function startLspClient(context: ExtensionContext): LanguageClient {
    const command = getLspServerPath(context)

    const serverOptions: ServerOptions = {
        run: { command, transport: TransportKind.stdio },
        debug: { command, transport: TransportKind.stdio }
    }

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "phalcom" }],
        initializationOptions: readInitializationOptions(),
        outputChannel: lspOutput,
        synchronize: {
            configurationSection: "phalcom",
            fileEvents: workspace.createFileSystemWatcher("**/*.ph")
        }
    }

    const client = new LanguageClient("phalcomLsp", "Phalcom Language Server", serverOptions, clientOptions)

    context.subscriptions.push(client)
    void client.start()

    return client
}

export function activate(context: ExtensionContext) {
    registerRunFile(context)

    lspOutput = window.createOutputChannel("Phalcom Language Server")
    context.subscriptions.push(lspOutput)

    context.subscriptions.push(commands.registerCommand("phalcom.restartLanguageServer", async () => {
        await lspClient?.stop()
        lspClient = startLspClient(context)
    }))
    context.subscriptions.push(commands.registerCommand("phalcom.showLanguageServerOutput", () => lspOutput?.show()))
    context.subscriptions.push(workspace.onDidChangeConfiguration(async event => {
        if (event.affectsConfiguration("phalcom.lsp.enabled") || event.affectsConfiguration("phalcom.lsp.serverPath")) {
            await lspClient?.stop()
            lspClient = workspace.getConfiguration("phalcom").get<boolean>("lsp.enabled", true) ? startLspClient(context) : undefined
        }
    }))

    if (workspace.getConfiguration("phalcom").get<boolean>("lsp.enabled", true)) {
        lspClient = startLspClient(context)
    }
}

export function deactivate(): Thenable<void> | undefined {
    return lspClient?.stop()
}
