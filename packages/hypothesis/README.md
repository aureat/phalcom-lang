# Hypothesis

Hypothesis is a standalone property-testing library package for Phalcom.

The package is being reconstructed around the current language model. The
active source tree contains scripted, replay, and seeded random choice
providers; integer and boolean strategies; property evaluation; bounded
search; integer shrinking; and repeated replay of minimized failures through
the explicit `Property.check` API. The earlier implementation is retained
under `legacy-src/` as a design reference; it is not part of the active package
surface.

The modernization follows the requirements analysis supplied with this task:
algebraic runtime state uses `data` and `match`, strategies keep their generated
type, examples defensively copy their choice tape, and the engine core avoids
reflection-driven `Any` transport. Composite strategies, semantic spans,
automatic derivation, reporting, persistence, and stateful testing remain later
work.

The package manifest is `project.toml`. The executable entry currently only
identifies the package. The executable property corpus will be added after the
core contracts stabilize.

`legacy-src/`, `tests/`, `scripts/`, and the existing design documents are
retained as historical material. The Python scripts check source shape and are
not executable language or library verification.
