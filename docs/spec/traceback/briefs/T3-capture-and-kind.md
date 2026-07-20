# T3 brief — dispatch AFTER E002 lands (dispatch.rs) AND G0 lands (error.rs); also needs T1's walk vocabulary (can start against IS §2's FrameView shape if T1 is in flight — coordinate)

Implement traceback plan unit T3 — capture record + surface error structure (PDR-0010 §1-§4,
ratified). Repo main directly.
READ FIRST: docs/decisions/0010-errors-carry-structure-and-cheap-origin.md (whole thing —
esp. §3 capture window, §3a per-hop cascade, §4 no-ObjRef rule);
implementation-spec.md §4 + §8.1 (kind table); plan.md §T3. graphify first (project rule).

Deliverables:
1. Capture record type: per-frame (module Symbol, method Symbol, line u32) — NEVER ObjRef,
   never source text. Stored beside RuntimeError::Raise's rendered field (error.rs:107-115).
2. Capture site: top of block_on's Err arm (primitive/block.rs:252-292) — after the surface
   Error is in hand, BEFORE vm.unwind_to (:282). Unconditional (matching AND non-matching
   branch — every on truncates since PDR-0007). Walk vm.frames[frames_len..] only (protected
   region depth, not whole stack).
3. Fiber cascade per-hop capture: inside the Call-mode cascade loop (dispatch.rs:341-370),
   capture each hop's link (fiber seq id + spawn site) BEFORE the :354-356 clears. Root-fiber
   exit path unchanged (frames stay live).
4. Error surface fields (zero floor delta, pure .ph + capture_error_value):
   core.ph Error gains _kind/_cause/_displaced + kind/cause getters + cause set on raise-from
   idiom if one exists (check core.ph Error class ~:54-61); capture_error_value
   (dispatch.rs:370-379 area) sets kind Symbol per IS §8.1 when wrapping native errors
   (incl. G0's new ConcurrentMutation → #concurrentMutation, DepthExceeded → #depthExceeded,
   Type → #type, DeadFrame → #deadFrame).
5. ensure sets displaced on the cleanup error when cleanup supersedes a raising body
   (block_ensure, block.rs:303-352 — the cleanup_err-wins arm).
6. Delete self.frames.clone() in runtime_error (dispatch.rs:123) if T1's walk hasn't already.
Write-set: phalcom-core/src/primitive/block.rs, src/vm/dispatch.rs, src/error.rs,
phalcom-core/core/core.ph, src/universe/primitives.rs (expect NO new binding — PDR-0010 says
zero floor delta; kind/cause getters are .ph fields), tests/**.
CAUTION: core.ph edits — new kernel-adjacent surface must not collide with install_core's
add_class! expectations; Error is bootstrapped in Rust with five floor primitives (core.ph
comment ~:633) — read that section before touching.
Tests: caught-error origin survives attempt() round-trip; e.kind == #range-shaped fixture;
displaced set on ensure-supersession; cross-fiber chain fields; negative-control all.
Gate: cargo build && cargo test && cargo clippy --workspace. Rustdoc mandatory.
GIT: pathspec commits only, never add -a / checkout -b, stop if write-set dirty. End:
Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Return: capture-record shape, kind mappings wired, SHAs, test evidence.

