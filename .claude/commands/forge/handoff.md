---
description: Emit a copy-paste continuation prompt for a fresh agent, built only from current conversation context — no file reads, no graphify, no survey. One dense fenced block.
---

Invoke the `forge` skill (Skill tool, `skill: "forge"`) and execute **§Handoff**.

Do not narrate the dispatch. Build the continuation block only from what is already in context; mark any missing fact as `[verify: <what>]` rather than going to find it. Output exactly one fenced block, no preamble or postamble.
