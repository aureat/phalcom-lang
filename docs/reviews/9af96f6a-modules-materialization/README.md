# Review: linked module program materialization

## Review target

This review covers commit [`9af96f6a0960470e280e2719ff49e1eb3ae135a8`](../../../.git/commit/9af96f6a0960470e280e2719ff49e1eb3ae135a8), `feat(modules): materialize linked module programs`, against parent [`bc05cf83445d4bc90b056c7378af78d95da313a3b`](../../../.git/commit/bc05cf83445d4bc90b056c7378af78d95da313a3b). No target commit was supplied, so the latest clean `main` commit was selected. The change spans 50 files, adds 1,263 lines, removes 396 lines, and introduces the program compiler/materializer/initializer path, module export dispatch, directory entry selection, and associated module fixtures and tests.

The review was organized into six requested areas and eight bounded reviewer passes, dispatched in four batches of two `phalcom-reviewer` agents. The primary evidence in this report is source-verified locally because several bounded subagent passes did not return before shutdown. The scope gives module behavior first priority, while also recording concrete CLI, runtime-dispatch, diagnostics, and API defects exposed by the same change.

## Findings summary

| Priority | Finding | Location |
| --- | --- | --- |
| P1 | Directory project/package entries fail before new entry selection runs. | [`phalcom-core/bin/phalcom/cli.rs:194`](../../../phalcom-core/bin/phalcom/cli.rs:194) |
| P1 | Whole-module re-exports cannot be linked, leaving `RuntimeExportRef::Module` unreachable. | [`phalcom-core/src/modules/materialize.rs:89`](../../../phalcom-core/src/modules/materialize.rs:89) |
| P1 | Invalid runtime dependency cycles panic while propagating failure instead of returning the specified invariant error. | [`phalcom-core/src/modules/initialize.rs:54`](../../../phalcom-core/src/modules/initialize.rs:54) |
| P2 | `EntrySelection::ModuleId` starts discovery with an empty project universe. | [`phalcom-core/src/modules/compile.rs:109`](../../../phalcom-core/src/modules/compile.rs:109) |
| P2 | Static module-export calls bypass rest-family dispatch, unlike dynamic module-export calls. | [`phalcom-core/src/vm/send.rs:1217`](../../../phalcom-core/src/vm/send.rs:1217) |
| P2 | Module-object export references ignore incoming arguments instead of applying the normal `call(...)` protocol. | [`phalcom-core/src/vm/send.rs:1198`](../../../phalcom-core/src/vm/send.rs:1198) |
| P2 | New `Interpreter::run_entry` erases structured program-compile errors into strings. | [`phalcom-core/src/interpret.rs:54`](../../../phalcom-core/src/interpret.rs:54) |

## Verification posture

The focused module runtime lane passed 5/5 tests, the `phalcom-modules` package tests passed, the REPL package lane passed 19/19 integration tests, the linked-import compile lane passed 2/2 tests, and the parent-to-target diff passed `git diff --check`. Those results validate the existing diamond, failure, inline-import rejection, export getter, and REPL compatibility paths; they do not cover the failing directory CLI path, whole-module re-export, invalid synthetic cycle, `ModuleId` discovery, rest-call parity, or argument-bearing module-object export.

The CLI directory failure was reproduced against `target/debug/phalcom`: a project directory that passes the module runtime fixture tests exits 74 with `Failed to read file .../diamond_app`. Full `phalcom-core` clippy with `-D warnings` was not treated as a clean gate because it reports the repository's existing large-`PhError`/`result-large-err` lint debt across baseline code as well as changed call sites. Detailed command output and scope classification are in [`verification.md`](verification.md), while missing coverage and recommended tests are in [`coverage-gaps.md`](coverage-gaps.md).

