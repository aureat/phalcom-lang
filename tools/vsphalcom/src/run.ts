import { commands, ExtensionContext, Terminal, TextEditor, Uri, window, workspace } from "vscode"

/**
 * Resolves the `phalcom` CLI path: the `phalcom.executablePath` setting,
 * defaulting to `"phalcom"` on `$PATH`.
 */
function getExecutablePath(): string {
    return workspace.getConfiguration("phalcom").get<string>("executablePath", "phalcom")
}

/**
 * The single reused integrated terminal the run command sends to, created
 * lazily on first run and kept across invocations (like the Python/Go
 * extensions' "Run" terminal) so repeated runs append to one visible pane
 * instead of spawning a new terminal per click.
 */
let runTerminal: Terminal | undefined

/**
 * Returns the run terminal, creating it if it doesn't exist yet or was
 * closed by the user since the last run.
 */
function getRunTerminal(): Terminal {
    if (runTerminal === undefined || runTerminal.exitStatus !== undefined) {
        runTerminal = window.createTerminal("Phalcom")
    }
    return runTerminal
}

/**
 * Quotes `path` for the shell the integrated terminal runs, defensively
 * (spaces or other shell-special characters in a workspace path shouldn't
 * break the command line).
 */
function shellQuote(path: string): string {
    return `"${path.replace(/"/g, '\\"')}"`
}

/**
 * Registers `phalcom.runFile`: runs the active (or given) `.ph` file with
 * the configured `phalcom` CLI in a dedicated integrated terminal — the
 * editor/title "run" button's command.
 *
 * Uses the same `phalcom.executablePath` setting the CLI is configured
 * through elsewhere. Takes no LSP dependency — this is CLI-subprocess-based
 * regardless of whether
 * `phalcom.lsp.enabled` is on, since running a program is not an editor
 * intelligence capability `phalcom-lsp` provides (ADR-0056 §2, VM-free).
 */
export function registerRunFile(context: ExtensionContext) {
    context.subscriptions.push(
        commands.registerCommand("phalcom.runFile", (uri?: Uri) => {
            const editor: TextEditor | undefined = window.activeTextEditor
            const filePath = uri?.fsPath ?? editor?.document.uri.fsPath
            if (filePath === undefined) {
                window.showErrorMessage("Phalcom: no file to run — open a .ph file first.")
                return
            }

            const terminal = getRunTerminal()
            terminal.show(true)
            terminal.sendText(`${shellQuote(getExecutablePath())} ${shellQuote(filePath)}`)
        })
    )
}
