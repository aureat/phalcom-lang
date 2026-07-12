---
description: Formulate one well-specified background subagent for a task, spawn it (run_in_background), and return to foreground work without waiting. Carries the forge subagent contract.
argument-hint: <task>
---

Invoke the `forge` skill (Skill tool, `skill: "forge"`) and execute **§Offload** for this task:

$ARGUMENTS

Do not narrate the dispatch. Formulate with the subagent contract (deliverable, known entry points, graphify-first/no-survey clause, exact return shape), spawn via the Agent tool with `run_in_background: true`, add it to the `outstanding` ledger, then state your foreground task and continue — never end the turn on the spawn.
