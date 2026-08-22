import {
    commands,
    Disposable,
    ExtensionContext,
    MarkdownString,
    StatusBarAlignment,
    StatusBarItem,
    window,
    workspace
} from "vscode"
import { LanguageClient } from "vscode-languageclient/node"

export type AnalysisPhase =
    | "starting"
    | "selectingCore"
    | "indexing"
    | "analyzing"
    | "publishing"
    | "ready"
    | "error"

export type AnalysisStep =
    | "discovering"
    | "parsing"
    | "shallowIndexing"
    | "flowAnalysis"
    | "solving"

export type AnalysisMode = "local" | "workspace"

export interface AnalysisStatus {
    session: number
    sequence: number
    phase: AnalysisPhase
    step?: AnalysisStep
    mode: AnalysisMode
    currentUri?: string
    discoveredFiles: number
    indexedFiles: number
    analyzedFiles: number
    generation?: number
    complete: boolean
    message?: string
}

export function formatStatusLabel(status: AnalysisStatus): string {
    switch (status.phase) {
        case "starting":
            return "$(sync~spin) Phalcom: Starting"
        case "selectingCore":
            return "$(sync~spin) Phalcom: Loading core"
        case "indexing": {
            const count = status.discoveredFiles > 0 ? status.discoveredFiles : status.indexedFiles
            return `$(sync~spin) Phalcom: Indexing · ${count} files`
        }
        case "analyzing":
            return status.currentUri
                ? "$(sync~spin) Phalcom: Analyzing current file"
                : "$(sync~spin) Phalcom: Analyzing workspace"
        case "publishing":
            return "$(sync~spin) Phalcom: Updating"
        case "ready":
            return "$(check) Phalcom: Ready"
        case "error":
            return "$(error) Phalcom: Error"
        default:
            return "$(info) Phalcom"
    }
}

export function formatStatusTooltip(status: AnalysisStatus): MarkdownString {
    const tooltip = new MarkdownString()
    tooltip.isTrusted = true
    tooltip.appendMarkdown("**Phalcom Analysis Status**\n\n")

    const stepInfo = status.step ? ` (${status.step})` : ""
    tooltip.appendMarkdown(`- **Phase**: \`${status.phase}\`${stepInfo}\n`)
    tooltip.appendMarkdown(`- **Mode**: \`${status.mode}\`\n`)
    tooltip.appendMarkdown(`- **Discovered Files**: \`${status.discoveredFiles}\`\n`)
    tooltip.appendMarkdown(`- **Indexed Files**: \`${status.indexedFiles}\`\n`)
    tooltip.appendMarkdown(`- **Analyzed Files**: \`${status.analyzedFiles}\`\n`)

    if (status.currentUri) {
        const relPath = workspace.asRelativePath(status.currentUri)
        tooltip.appendMarkdown(`- **Current File**: \`${relPath}\`\n`)
    }

    if (status.generation !== undefined) {
        tooltip.appendMarkdown(`- **Semantic Generation**: \`${status.generation}\`\n`)
    }

    if (status.message) {
        tooltip.appendMarkdown(`- **Message**: ${status.message}\n`)
    }

    tooltip.appendMarkdown("\n*Click status icon for details & actions*")
    return tooltip
}

export class AnalysisStatusBarController implements Disposable {
    private readonly statusBarItem: StatusBarItem
    private readonly disposables: Disposable[] = []
    private currentSession = 0
    private currentSequence = 0
    private lastStatus: AnalysisStatus | undefined

    constructor() {
        this.statusBarItem = window.createStatusBarItem("phalcom.analysisStatus", StatusBarAlignment.Right, 100)
        this.statusBarItem.name = "Phalcom Analysis Status"
        this.statusBarItem.command = "phalcom.showAnalysisStatus"
        this.updateVisibility()

        this.disposables.push(
            workspace.onDidChangeConfiguration(event => {
                if (event.affectsConfiguration("phalcom.analysis.statusBar")) {
                    this.updateVisibility()
                }
            })
        )
    }

    public attach(client: LanguageClient): void {
        this.reset()
        client.onNotification("phalcom/analysisStatus", (status: AnalysisStatus) => {
            this.handleStatus(status)
        })
    }

    public handleStatus(status: AnalysisStatus): boolean {
        // State reduction: reject stale updates from older sessions or out-of-order sequences
        if (status.session < this.currentSession) {
            return false
        }
        if (status.session === this.currentSession && status.sequence <= this.currentSequence) {
            return false
        }

        this.currentSession = status.session
        this.currentSequence = status.sequence
        this.lastStatus = status

        this.statusBarItem.text = formatStatusLabel(status)
        this.statusBarItem.tooltip = formatStatusTooltip(status)
        this.updateVisibility()
        return true
    }

    public async showStatusPopover(): Promise<void> {
        if (!this.lastStatus) {
            const pick = await window.showQuickPick(
                [
                    { label: "$(output) Show Language Server Output", action: "output" },
                    { label: "$(refresh) Restart Language Server", action: "restart" }
                ],
                { title: "Phalcom LSP: Starting..." }
            )
            if (pick?.action === "output") {
                await commands.executeCommand("phalcom.showLanguageServerOutput")
            } else if (pick?.action === "restart") {
                await commands.executeCommand("phalcom.restartLanguageServer")
            }
            return
        }

        const s = this.lastStatus
        const stepDetail = s.step ? ` (${s.step})` : ""
        const currentFile = s.currentUri ? workspace.asRelativePath(s.currentUri) : "None"
        const items = [
            {
                label: `Phase: ${s.phase}${stepDetail}`,
                description: `Mode: ${s.mode} | Gen: ${s.generation ?? "N/A"}`
            },
            {
                label: `Files: Discovered ${s.discoveredFiles} | Indexed ${s.indexedFiles} | Analyzed ${s.analyzedFiles}`,
                description: `Current: ${currentFile}`
            },
            {
                label: "$(output) Show Language Server Output",
                action: "output"
            },
            {
                label: "$(refresh) Restart Language Server",
                action: "restart"
            }
        ]

        if (s.message) {
            items.unshift({
                label: `Message: ${s.message}`,
                description: ""
            })
        }

        const selected = await window.showQuickPick(items, {
            title: `Phalcom LSP Status (Session ${s.session}, Seq ${s.sequence})`
        })

        if (selected?.action === "output") {
            await commands.executeCommand("phalcom.showLanguageServerOutput")
        } else if (selected?.action === "restart") {
            await commands.executeCommand("phalcom.restartLanguageServer")
        }
    }

    public updateVisibility(): void {
        const enabled = workspace.getConfiguration("phalcom").get<boolean>("analysis.statusBar", true)
        if (enabled) {
            this.statusBarItem.show()
        } else {
            this.statusBarItem.hide()
        }
    }

    public reset(): void {
        this.currentSession = 0
        this.currentSequence = 0
        this.lastStatus = undefined
        this.statusBarItem.text = "$(sync~spin) Phalcom: Starting"
        this.statusBarItem.tooltip = "Phalcom Language Server starting..."
        this.updateVisibility()
    }

    public getLastStatus(): AnalysisStatus | undefined {
        return this.lastStatus
    }

    public dispose(): void {
        this.statusBarItem.dispose()
        for (const d of this.disposables) {
            d.dispose()
        }
    }
}
