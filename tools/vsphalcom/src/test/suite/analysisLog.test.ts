import * as assert from "node:assert"
import { AnalysisLogEvent, formatAnalysisLog } from "../../extension"

suite("AnalysisLog formatting tests", () => {
    test("formats structured lifecycle context and optional details", () => {
        const event: AnalysisLogEvent = {
            session: 3,
            sequence: 8,
            level: "info",
            phase: "publishing",
            event: "snapshot.published",
            message: "formal snapshot ready",
            durationMs: 17,
            generation: 12
        }

        assert.strictEqual(
            formatAnalysisLog(event),
            "[info] snapshot.published (session=3, sequence=8, phase=publishing) — formal snapshot ready · 17ms · generation=12"
        )
    })

    test("omits absent optional details", () => {
        const event: AnalysisLogEvent = {
            session: 1,
            sequence: 2,
            level: "error",
            phase: "error",
            event: "semantic.update.failed"
        }

        assert.strictEqual(formatAnalysisLog(event), "[error] semantic.update.failed (session=1, sequence=2, phase=error)")
    })
})
