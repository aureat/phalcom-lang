import { commands, ExtensionContext, OutputChannel, TextDocumentContentProvider, Uri, window, workspace } from "vscode"
import { existsSync } from "node:fs"
import { join } from "node:path"
import { registerRunFile } from "./run"
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from "vscode-languageclient/node"
import { AnalysisStatusBarController } from "./analysisStatus"

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

export interface LspClientLifecycleHandle {
    stop(): Promise<void>
    dispose(): void
}

interface LspClientLifecycleOptions<T extends LspClientLifecycleHandle> {
    getClient(): T | undefined
    setClient(client: T | undefined): void
    isEnabled(): boolean
    start(): T
    log(message: string): void
}

export interface LspClientLifecycle<T extends LspClientLifecycleHandle> {
    startIfEnabled(): Promise<void>
    restart(): Promise<void>
    stop(): Promise<void>
    dispose(): void
}

/** Serializes client replacement and makes failed graceful shutdown recoverable. */
export function createLspClientLifecycle<T extends LspClientLifecycleHandle>(
    options: LspClientLifecycleOptions<T>
): LspClientLifecycle<T> {
    let transition: Promise<void> = Promise.resolve()

    const stopCurrent = async (): Promise<void> => {
        const previous = options.getClient()
        options.setClient(undefined)
        if (!previous) {
            return
        }

        try {
            await previous.stop()
        } catch (error) {
            options.log(`Language server stop failed; disposing client: ${String(error)}`)
        } finally {
            previous.dispose()
        }
    }

    const enqueue = (operation: () => Promise<void>): Promise<void> => {
        const result = transition.then(operation, operation)
        transition = result.catch(() => undefined)
        return result
    }

    return {
        startIfEnabled: () => enqueue(async () => {
            if (options.isEnabled() && !options.getClient()) {
                options.setClient(options.start())
            }
        }),
        restart: () => enqueue(async () => {
            await stopCurrent()
            if (options.isEnabled()) {
                options.setClient(options.start())
            }
        }),
        stop: () => enqueue(stopCurrent),
        dispose: () => {
            const previous = options.getClient()
            options.setClient(undefined)
            previous?.dispose()
        }
    }
}

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
            analysis: {
                mode: config.get<string>("analysis.mode", "local"),
                exclude: config.get<string[]>("analysis.exclude", []),
                logLevel: config.get<string>("analysis.logLevel", "info")
            },
            inlayHints: {
                types: config.get<string>("inlayHints.types", "stable"),
                suppressObvious: config.get<boolean>("inlayHints.suppressObvious", true)
            }
        }
    }
}

let statusBarController: AnalysisStatusBarController | undefined

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

    client.onNotification("phalcom/analysisLog", (event: AnalysisLogEvent) => {
        if (shouldShowAnalysisLog(event.level)) {
            lspOutput?.appendLine(formatAnalysisLog(event))
        }
    })

    if (statusBarController) {
        statusBarController.attach(client)
    }

    void client.start()

    return client
}

export type AnalysisLogLevel = "error" | "info" | "verbose"

export interface AnalysisLogEvent {
    session: number
    sequence: number
    level: AnalysisLogLevel
    phase: string
    event: string
    message?: string
    durationMs?: number
    generation?: number
}

function shouldShowAnalysisLog(level: AnalysisLogLevel): boolean {
    const configured = workspace.getConfiguration("phalcom").get<AnalysisLogLevel>("analysis.logLevel", "info")
    const rank: Record<AnalysisLogLevel, number> = { error: 0, info: 1, verbose: 2 }
    return rank[level] <= rank[configured]
}

export function formatAnalysisLog(event: AnalysisLogEvent): string {
    const details = [
        event.message,
        event.durationMs === undefined ? undefined : `${event.durationMs}ms`,
        event.generation === undefined ? undefined : `generation=${event.generation}`
    ].filter(Boolean).join(" · ")
    return `[${event.level}] ${event.event} (session=${event.session}, sequence=${event.sequence}, phase=${event.phase})${details ? ` — ${details}` : ""}`
}

function ensureLspClientLifecycle(context: ExtensionContext): LspClientLifecycle<LanguageClient> {
    if (!lspLifecycle) {
        lspLifecycle = createLspClientLifecycle({
            getClient: () => lspClient,
            setClient: client => { lspClient = client },
            isEnabled: () => workspace.getConfiguration("phalcom").get<boolean>("lsp.enabled", true),
            start: () => startLspClient(context),
            log: message => lspOutput?.appendLine(message)
        })
        context.subscriptions.push(lspLifecycle)
    }
    return lspLifecycle
}

let lspLifecycle: LspClientLifecycle<LanguageClient> | undefined

/** Stops the previous client even when graceful shutdown fails, then starts a replacement. */
export function restartLspClient(context: ExtensionContext): Promise<void> {
    return ensureLspClientLifecycle(context).restart()
}

export function activate(context: ExtensionContext) {
    registerRunFile(context)

    lspOutput = window.createOutputChannel("Phalcom Language Server")
    context.subscriptions.push(lspOutput)

    const virtualSourceProvider: TextDocumentContentProvider = {
        provideTextDocumentContent: async (uri: Uri): Promise<string> => {
            if (!lspClient) {
                return ""
            }
            try {
                return await lspClient.sendRequest<string | null>("phalcom/sourceText", { uri: uri.toString() }) ?? ""
            } catch (error) {
                lspOutput?.appendLine(`Virtual source request failed: ${String(error)}`)
                return ""
            }
        }
    }
    context.subscriptions.push(workspace.registerTextDocumentContentProvider("phalcom", virtualSourceProvider))

    statusBarController = new AnalysisStatusBarController()
    context.subscriptions.push(statusBarController)

    ensureLspClientLifecycle(context)

    context.subscriptions.push(commands.registerCommand("phalcom.restartLanguageServer", async () => {
        await restartLspClient(context)
    }))
    context.subscriptions.push(commands.registerCommand("phalcom.showLanguageServerOutput", () => lspOutput?.show()))
    context.subscriptions.push(commands.registerCommand("phalcom.showAnalysisStatus", async () => {
        await statusBarController?.showStatusPopover()
    }))
    context.subscriptions.push(workspace.onDidChangeConfiguration(async event => {
        if (event.affectsConfiguration("phalcom.lsp.enabled") || event.affectsConfiguration("phalcom.lsp.serverPath")) {
            await restartLspClient(context)
        }
    }))

    void ensureLspClientLifecycle(context).startIfEnabled()
}

export function deactivate(): Thenable<void> | undefined {
    return lspLifecycle?.stop()
}
