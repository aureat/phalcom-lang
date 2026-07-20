# Implementation specs

Dispatch-ready implementation plans for ratified (or ratification-pending) surface specs in
[`../core/`](../core/). One file per unit. Each names every file it touches, the tree
patterns it copies, and its test gates — so an implementer starts from `file:line` anchors,
not a survey.

An impl spec whose governing record is still **Proposed** is blocked by
[`decisions/README.md`](../../../decisions/README.md) rule 5 until that record is Accepted;
the spec says so in its header.

| Impl spec | Surface spec | Governing record | Buildable? |
|---|---|---|---|
| [`bytes.md`](bytes.md) | [`core/bytes.md`](../core/bytes.md) | [PDR-0011](../../../decisions/0011-admit-bytes-native-octet-buffer.md) | ✅ unblocked — PDR-0011 Accepted 2026-07-20 |
