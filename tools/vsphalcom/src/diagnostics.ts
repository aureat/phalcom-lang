import { execFile } from "child_process"
import {
    Diagnostic,
    DiagnosticCollection,
    DiagnosticSeverity,
    Disposable,
    ExtensionContext,
    languages,
    Position,
    Range,
    TextDocument,
    window,
    workspace
} from "vscode"

/** Shape of the single JSON diagnostic object `phalcom check --format json` prints to stdout on a syntax error. */
interface PhalcomCheckDiagnostic {
    severity: "error"
    message: string
    range: {
        start: { line: number; column: number }
        end: { line: number; column: number }
    }
}

/** Output channel used to log subprocess failures (missing binary, unexpected output) without popping error UI. */
let outputChannel = window.createOutputChannel("Phalcom")

/**
 * Maps the `phalcom check --format json` 1-based `{line, column}` pair to a
 * 0-based VS Code {@link Position}.
 */
function toPosition(pos: { line: number; column: number }): Position {
    return new Position(Math.max(0, pos.line - 1), Math.max(0, pos.column - 1))
}

/**
 * Runs `<executablePath> check <filePath> --format json` and resolves to the
 * parsed diagnostic, or `null` if the source is clean (exit 0, no output).
 *
 * Never rejects on subprocess failure (missing binary, unexpected output) —
 * callers get `null` and a message is logged to the output channel instead,
 * so a missing/broken binary degrades to "no diagnostics".
 */
function runCheck(executablePath: string, filePath: string): Promise<PhalcomCheckDiagnostic | null> {
    return new Promise((resolve) => {
        execFile(executablePath, ["check", filePath, "--format", "json"], (error, stdout) => {
            const trimmed = stdout.trim()

            if (!error) {
                // Exit 0: clean parse, no diagnostics.
                resolve(null)
                return
            }

            // execFile treats a nonzero exit as an "error" even though this is
            // the expected path for a syntax error. Distinguish "the binary ran
            // and reported a diagnostic" from "the binary could not be run".
            if ((error as NodeJS.ErrnoException).code === "ENOENT") {
                outputChannel.appendLine(
                    `[phalcom] executable not found at "${executablePath}" (configure "phalcom.executablePath")`
                )
                resolve(null)
                return
            }

            if (!trimmed) {
                outputChannel.appendLine(`[phalcom] check exited with an error and no output: ${error.message}`)
                resolve(null)
                return
            }

            try {
                const parsed = JSON.parse(trimmed) as PhalcomCheckDiagnostic
                resolve(parsed)
            } catch (parseError) {
                outputChannel.appendLine(
                    `[phalcom] failed to parse check output as JSON: ${String(parseError)}\noutput: ${trimmed}`
                )
                resolve(null)
            }
        })
    })
}

/**
 * Runs `phalcom check` on `document` and publishes the result (or clears any
 * stale entry) to `collection`. No-ops for documents that are not `.ph`
 * (language id `phalcom`) or have no on-disk path (untitled/unsaved).
 */
async function checkDocument(
    document: TextDocument,
    collection: DiagnosticCollection,
    getExecutablePath: () => string
): Promise<void> {
    if (document.languageId !== "phalcom") {
        return
    }
    if (!document.uri.fsPath || document.isUntitled) {
        return
    }

    const executablePath = getExecutablePath()
    const result = await runCheck(executablePath, document.uri.fsPath)

    if (!result) {
        collection.delete(document.uri)
        return
    }

    const range = new Range(toPosition(result.range.start), toPosition(result.range.end))
    const diagnostic = new Diagnostic(range, result.message, DiagnosticSeverity.Error)
    diagnostic.source = "phalcom"

    collection.set(document.uri, [diagnostic])
}

/**
 * Registers the diagnostics leg: spawns `phalcom check --format json` on
 * document save (and on open, to cover files already on disk when the editor
 * starts) and publishes results to a shared `DiagnosticCollection`.
 *
 * The binary path comes from the `phalcom.executablePath` setting (default
 * `"phalcom"`, resolved on `$PATH`).
 */
export function registerDiagnostics(context: ExtensionContext): Disposable {
    const collection = languages.createDiagnosticCollection("phalcom")

    const getExecutablePath = (): string =>
        workspace.getConfiguration("phalcom").get<string>("executablePath", "phalcom")

    const run = (document: TextDocument) => {
        void checkDocument(document, collection, getExecutablePath)
    }

    const disposables: Disposable[] = [
        collection,
        outputChannel,
        workspace.onDidSaveTextDocument(run),
        workspace.onDidOpenTextDocument(run),
        workspace.onDidCloseTextDocument((document) => collection.delete(document.uri))
    ]

    context.subscriptions.push(...disposables)

    return Disposable.from(...disposables)
}
