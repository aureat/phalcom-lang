# T1 brief — ready to dispatch AFTER E002 lands (write-set conflict: dispatch.rs, fiber.rs)

Implement traceback plan unit T1 — the stack walk primitive and spans accessor.
Repo: /Users/altunhasanli/dev/phalcom/phalcom, main directly.

READ FIRST (normative): docs/spec/current/traceback/implementation-spec.md §2 (FrameView, logical
expansion seam, span accessor), plan.md §T1. graphify before source reads (project rule).

Deliverables:
1. phalcom-core/src/vm/walk.rs — StackWalk<'vm> iterator, FrameView { module: Symbol, name:
   FrameName, line: u32, span: SourceRange, source: Option<Arc<String>>, is_core: bool,
   fiber: u32 }, FrameName { Main | Method(Symbol) | Block { enclosing: Symbol } |
   Native(Symbol) }. Iterates vm.frames OLDEST→NEWEST (no .rev()). Internal
   expand(physical) -> iterator seam, 1:1 today. is_core = module identity vs core module
   handle, NOT name compare. Selector-shape rendering helper (Cart.sum(_), <block in …>,
   <main>).
2. Chunk::span_at(ip) + line_at(ip, source) on chunk.rs — clamping (ip==0 → first, past-end →
   last), never panics. Migrate ALL existing spans[...] index sites to it (runtime_error at
   dispatch.rs ~:136-143 is the main one; grep for `.spans`).
3. FiberObject::seq: u32 display id — per-VM monotonic counter, root fiber #1, assigned at
   spawn (heap/fiber.rs + primitive/fiber.rs spawn site + wherever root fiber is created).
4. runtime_error may consume StackWalk for its SourceLoc construction if that's a clean
   mechanical swap, but do NOT change its output format (T4 owns the renderer rewrite).

Write-set: phalcom-core/src/vm/walk.rs (new), vm/mod.rs (module decl), chunk.rs, vm/dispatch.rs
(accessor migration only), heap/fiber.rs, primitive/fiber.rs, tests. STOP if unexpectedly dirty.

Tests: span_at clamping unit tests; 3-deep call walk test asserting order + selector-shaped
names; negative-control each. Gate: cargo build && cargo test && cargo clippy --workspace.
Rust docs mandatory on all new public items.

GIT: never add -a / checkout -b; pathspec commits only (`git commit -m … -- <paths>`); per
green checkpoint; conventional message ending
Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

Return: API summary, migrated span sites list, commit SHAs, test evidence.
