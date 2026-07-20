# T4 brief — dispatch AFTER T1 + T2 + T3 all land (an uncaught-only slice can run on T1+T2 alone if T3 stalls)

Implement traceback plan unit T4 — the traceback renderer, wired.
Repo main directly. READ FIRST: implementation-spec.md §5 (all subsections — normative),
§2 (FrameView it consumes), catalog §1/§2 (target renders), color.md §4/§5 (role application);
plan.md §T4. graphify first.

Deliverables:
1. diagnostics/traceback.rs — human renderer: Python order (walk is already oldest-first),
   two-line frame form (`  <module>:<line>   in <name>` + indented source line), caret block
   (diagnostics::caret) on INNERMOST frame only replacing its echo, core-frame elision with
   count + --trace-core expansion, source:None frames render header-only.
2. Budgets (IS §5.2): repeat-collapse first (>3 identical (module,name,line) → [previous frame
   repeated N more times]), then frame budget 40 (keep 15 oldest + 15 newest, elide middle);
   fiber chains budget 8 (keep 2+4).
3. Chains (IS §5.3): fiber link `⤷ raised inside fiber #N, spawned at file:line` (chain role);
   Error#cause via `⤷ caused by:`; displaced via titled secondary traceback note. Each side a
   complete traceback; only innermost side carries caret+message.
4. JSON traceback (IS §5.4): one line on stderr under --trace-format=json; frames
   oldest-first; fields module/file/line/name/core/fiber + error{message,kind}. This is the
   fixture contract.
5. Native frames (IS §5.5): dispatch send path stores in-flight selector + receiver class
   (two Symbols, native path only); on primitive Err, synthesize `  [native]    in
   Class.sel(_)` logical frame, never core-elided. anchor_of(selector, class) ->
   Option<(Symbol, SourceRange)> seam returning None today (registry is a later unit) —
   render bare form on None.
6. Rewire: runtime_error (dispatch.rs:121-169) walks via StackWalk and renders via
   traceback.rs; DELETE print_rt + print_frame (diagnostics/mod.rs:107-135). --trace-core flag
   on Cli.
7. help: line slot exists but emits nothing until T6's suggest engine lands (leave a seam,
   don't stub a fake suggestion).
Write-set: phalcom-core/src/diagnostics/traceback.rs (new) + mod.rs (deletions),
src/vm/dispatch.rs (runtime_error body + native send context), bin/phalcom/cli.rs
(--trace-core), tests/**.
Tests: JSON-fixture frame sequences — base case (shop.ph shape from catalog §1), recursion
collapse (DepthExceeded program), fiber chain (catalog §2 shape), native frame; two snapshot
canaries (catalog §1 + §6) marked churn-prone; color-off invariance on the canaries;
negative-control all. Existing negative-lane fixtures asserting old traceback text WILL move —
that is expected; update them to field asserts or new text deliberately, listing each in the
commit body.
Gate: cargo build && cargo test && cargo clippy --workspace. Rustdoc mandatory.
GIT: pathspec commits only, never add -a / checkout -b, stop if write-set dirty. End:
Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Return: renderer summary, deleted legacy list, fixture migrations, SHAs, empirical before/after
of the bogusSelector repro (class T { a { self.b } b { self.c } c { 1.bogusSelector } } +
T.new().a).
