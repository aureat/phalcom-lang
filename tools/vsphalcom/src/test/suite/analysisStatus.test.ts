import * as assert from "node:assert"
import {
    AnalysisStatus,
    AnalysisStatusBarController,
    formatStatusLabel,
    formatStatusTooltip
} from "../../analysisStatus"

suite("AnalysisStatus Controller Unit Tests", () => {
    test("formatStatusLabel maps phases correctly", () => {
        const base: AnalysisStatus = {
            session: 1,
            sequence: 1,
            phase: "starting",
            mode: "local",
            discoveredFiles: 0,
            indexedFiles: 0,
            analyzedFiles: 0,
            complete: false
        }

        assert.strictEqual(formatStatusLabel({ ...base, phase: "starting" }), "$(sync~spin) Phalcom: Starting")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "selectingCore" }), "$(sync~spin) Phalcom: Loading core")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "indexing", discoveredFiles: 10 }), "$(sync~spin) Phalcom: Indexing · 10 files")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "analyzing", currentUri: "file:///workspace/main.ph" }), "$(sync~spin) Phalcom: Analyzing current file")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "analyzing" }), "$(sync~spin) Phalcom: Analyzing workspace")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "publishing" }), "$(sync~spin) Phalcom: Updating")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "ready" }), "$(check) Phalcom: Ready")
        assert.strictEqual(formatStatusLabel({ ...base, phase: "error" }), "$(error) Phalcom: Error")
    })

    test("formatStatusTooltip includes counts and details", () => {
        const status: AnalysisStatus = {
            session: 1,
            sequence: 2,
            phase: "indexing",
            step: "discovering",
            mode: "workspace",
            currentUri: "file:///workspace/lib.ph",
            discoveredFiles: 15,
            indexedFiles: 10,
            analyzedFiles: 2,
            generation: 5,
            complete: false,
            message: "indexing in progress"
        }

        const tooltip = formatStatusTooltip(status)
        const value = tooltip.value

        assert.ok(value.includes("**Phalcom Analysis Status**"))
        assert.ok(value.includes("- **Phase**: `indexing` (discovering)"))
        assert.ok(value.includes("- **Mode**: `workspace`"))
        assert.ok(value.includes("- **Discovered Files**: `15`"))
        assert.ok(value.includes("- **Indexed Files**: `10`"))
        assert.ok(value.includes("- **Analyzed Files**: `2`"))
        assert.ok(value.includes("- **Semantic Generation**: `5`"))
        assert.ok(value.includes("- **Message**: indexing in progress"))
    })

    test("handleStatus enforces session and sequence monotonic reduction", () => {
        const controller = new AnalysisStatusBarController()

        const status1: AnalysisStatus = {
            session: 1,
            sequence: 1,
            phase: "starting",
            mode: "local",
            discoveredFiles: 0,
            indexedFiles: 0,
            analyzedFiles: 0,
            complete: false
        }

        assert.strictEqual(controller.handleStatus(status1), true)
        assert.strictEqual(controller.getLastStatus()?.sequence, 1)

        // Stale sequence in same session -> rejected
        const staleSeq: AnalysisStatus = { ...status1, sequence: 1, phase: "ready" }
        assert.strictEqual(controller.handleStatus(staleSeq), false)
        assert.strictEqual(controller.getLastStatus()?.phase, "starting")

        // Newer sequence in same session -> accepted
        const newerSeq: AnalysisStatus = { ...status1, sequence: 2, phase: "indexing" }
        assert.strictEqual(controller.handleStatus(newerSeq), true)
        assert.strictEqual(controller.getLastStatus()?.phase, "indexing")

        // Older session -> rejected
        const olderSession: AnalysisStatus = { ...status1, session: 0, sequence: 10, phase: "ready" }
        assert.strictEqual(controller.handleStatus(olderSession), false)

        // Newer session with lower sequence -> accepted
        const newerSession: AnalysisStatus = { ...status1, session: 2, sequence: 1, phase: "selectingCore" }
        assert.strictEqual(controller.handleStatus(newerSession), true)
        assert.strictEqual(controller.getLastStatus()?.session, 2)

        controller.dispose()
    })
})
