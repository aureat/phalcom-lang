//! Human and JSON traceback renderers (IS §5).
//!
//! The single public entry point is [`render_traceback`]. It dispatches to
//! [`render_human_traceback`] or [`render_json_traceback`] based on the caller's
//! `trace_format_json` flag.
//!
//! # Two source paths for frames
//!
//! * **Uncaught** errors: `VM::frames` is still live at `runtime_error` time
//!   ([PDR-0008](../../../../docs/decisions/0008-cell-boundary-diagnostics-and-state-hygiene.md) §2),
//!   so we build a `StackWalk` and stream `FrameView`s directly — no clone, no `rev()`.
//!
//! * **Caught-then-re-raised** (fiber cascade, `on(_)` handlers): frames were captured
//!   into a [`crate::error::FrameRecord`] vec at the `block_on` boundary before
//!   `unwind_to` destroyed them (T3). The renderer reads those records from
//!   `RuntimeError::Raise { traceback: Some(tb), .. }`.
//!
//! # Spec reference
//!
//! IS §5.1–§5.5, §11.  Output stable contract: JSON frame *sequence*, not human byte layout.

use std::sync::Arc;

use phalcom_common::range::SourceRange;

use crate::diagnostics::caret::{Label, LabelKind, Snippet};
use crate::diagnostics::style::{RenderConfig, Role, Styler};
use crate::error::{FrameRecord, PhError, RuntimeError};
use crate::interner::Symbol;
use crate::vm::VM;
use crate::vm::walk::{FrameName, FrameView};

// ──────────────────────────────────────────────────────────────────────────────
// Public entry point
// ──────────────────────────────────────────────────────────────────────────────

/// Renders a complete traceback for `err` and writes it to stderr.
///
/// For an **uncaught** error this walks the live VM frame stack; for a
/// **caught-and-captured** `RuntimeError::Raise` it reads the
/// [`FrameRecord`] vec stored at raise time.
///
/// `config` drives color and glyph selection; `trace_core` expands otherwise-elided
/// core frames; `trace_format_json` switches to the single-line JSON format (IS §5.4).
pub fn render_traceback(vm: &mut VM, err: &PhError, config: &RenderConfig, trace_core: bool, trace_format_json: bool) {
    let rendered = if trace_format_json {
        render_json_traceback(vm, err)
    } else {
        render_human_traceback(vm, err, config, trace_core)
    };
    eprint!("{rendered}");
}

// ──────────────────────────────────────────────────────────────────────────────
// Human renderer (IS §5.1)
// ──────────────────────────────────────────────────────────────────────────────

/// Renders a human-readable traceback string (IS §5.1).
///
/// Python order: oldest frame first, error message at the bottom.
/// Innermost frame gets the caret block; all other frames get a one-line echo.
fn render_human_traceback(vm: &mut VM, err: &PhError, config: &RenderConfig, trace_core: bool) -> String {
    let mut out = String::new();
    let styler = Styler::new(config);

    out.push_str(&styler.paint(Role::SeverityError, "Traceback (most recent call last):\n"));

    match err {
        PhError::Runtime(RuntimeError::Raise { traceback: Some(tb), .. }) => {
            // Caught path: render from the captured FrameRecord vec.
            render_records_human(&mut out, vm, tb, config, trace_core, &styler);
        }
        _ => {
            // Uncaught path: walk the live VM frame stack.
            render_live_human(&mut out, vm, config, trace_core, &styler);
        }
    }

    // Optional native frame synthesized from the in-flight send context (IS §5.5).
    if let Some(native_line) = maybe_native_frame_line(vm, &styler) {
        out.push_str(&native_line);
    }

    // Error message line with × marker.
    let msg = err.to_string();
    out.push_str(&format!("{} {}\n", styler.paint(Role::SeverityError, "×"), msg));

    // Innermost caret block, emitted after the × line so frames read top-to-bottom
    // with the error and source annotation together at the bottom (IS §5.1).
    if let Some(snippet) = innermost_caret_block(vm, err, config) {
        out.push_str(&snippet);
    }

    // Help suggestion line (IS §9).
    if let Some(help_line) = get_help_suggestion(vm, err, config, &styler) {
        out.push_str(&help_line);
    }

    out
}

/// Renders the live VM call stack as human-readable frame lines.
fn render_live_human(out: &mut String, vm: &mut VM, config: &RenderConfig, trace_core: bool, styler: &Styler) {
    // Collect the live walk into an owned vec so we can post-process it
    // (budget + collapse + elision) without keeping a borrow on `vm`.
    let views: Vec<FrameView> = vm.walk().collect();
    let items = process_frame_views(views, trace_core);
    emit_human_items(out, vm, &items, config, styler, true);
}

/// Renders a captured `FrameRecord` list (from a caught-and-re-raised error) as
/// human-readable frame lines, including fiber-boundary chain links (IS §5.3).
fn render_records_human(out: &mut String, vm: &mut VM, records: &[FrameRecord], config: &RenderConfig, trace_core: bool, styler: &Styler) {
    // Partition into fiber groups separated by FiberBoundary records.
    let groups = partition_records(records, vm);
    let groups_len = groups.len();

    for (group_idx, (fiber_seq, boundary, group_frames)) in groups.iter().enumerate() {
        // Fiber boundary link (IS §5.3) — emitted *before* the group that
        // raised inside the fiber whose floor it marks.
        if let Some((spawn_file_sym, spawn_line)) = boundary {
            let file_str = spawn_file_sym
                .map(|s| vm.resolve_symbol(s).to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            out.push_str(&format!(
                "{} raised inside fiber #{}, spawned at {}:{}\n",
                styler.paint(Role::Chain, "⤷"),
                fiber_seq,
                styler.paint(Role::Location, &file_str),
                spawn_line,
            ));
        }

        // Build synthetic FrameViews from the compact records.
        let views = records_to_views(vm, group_frames, *fiber_seq);
        let items = process_frame_views(views, trace_core);
        // Only the innermost group's final frame is the innermost overall.
        let is_last_group = group_idx == groups_len - 1;
        emit_human_items(out, vm, &items, config, styler, is_last_group);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// JSON renderer (IS §5.4)
// ──────────────────────────────────────────────────────────────────────────────

/// Renders a single-line JSON traceback (IS §5.4).
///
/// Format: `{"ev":"traceback","error":{"message":"…","kind":"…"},"frames":[…]}`
///
/// Frames appear oldest-first, matching the human render.
fn render_json_traceback(vm: &mut VM, err: &PhError) -> String {
    let message = err.to_string();
    let kind = extract_kind_str(vm, err);

    let mut json_frames: Vec<String> = Vec::new();

    match err {
        PhError::Runtime(RuntimeError::Raise { traceback: Some(tb), .. }) => {
            let groups = partition_records(tb, vm);
            for (_fiber_seq, _boundary, frames) in &groups {
                for rec in *frames {
                    if let FrameRecord::Normal { module, method, line } = rec {
                        let module_name = vm.resolve_symbol(*module).to_string();
                        let file = format!("{}.ph", module_name);
                        let name = vm.resolve_symbol(*method).to_string();
                        let is_core = is_core_module(vm, *module);
                        json_frames.push(json_frame(&module_name, &file, *line, &name, is_core, 1));
                    }
                }
            }
        }
        _ => {
            let views: Vec<FrameView> = vm.walk().collect();
            for v in &views {
                let module_name = vm.resolve_symbol(v.module).to_string();
                let file = format!("{}.ph", module_name);
                let name = frame_name_to_string(vm, &v.name);
                json_frames.push(json_frame(&module_name, &file, v.line, &name, v.is_core, v.fiber));
            }
        }
    }

    // Optional native frame (IS §5.5).
    if let (Some(sel), Some(cls)) = (vm.native_selector, vm.native_class) {
        let sel_str = vm.resolve_symbol(sel).to_string();
        let cls_str = vm.resolve_symbol(cls).to_string();
        let name = format!("{}.{}", cls_str, sel_str);
        json_frames.push(json_frame("[native]", "[native]", 0, &name, false, 1));
    }

    let frames_json = json_frames.join(",");
    format!(
        "{{\"ev\":\"traceback\",\"error\":{{\"message\":{},\"kind\":{}}},\"frames\":[{}]}}\n",
        json_str(&message),
        json_str(&kind),
        frames_json,
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal rendering helpers
// ──────────────────────────────────────────────────────────────────────────────

/// One unit of human output after collapse/budget/elision.
enum HumanItem {
    /// A normal resolved frame.
    Frame(FrameView),
    /// A run of >3 identical consecutive frames collapsed to one entry + repeat notice.
    Repeated { first: FrameView, count: usize },
    /// N consecutive core frames elided (IS §5.1).
    ElidedCore { count: usize },
    /// The middle slice of an over-budget stack (IS §5.2).
    ElidedMiddle { count: usize },
}

/// Applies repeat-collapse then core-elision and budget-trimming to a `FrameView` vec (IS §5.2).
fn process_frame_views(views: Vec<FrameView>, trace_core: bool) -> Vec<HumanItem> {
    let collapsed = collapse_repeats(views);
    let elided = elide_core(collapsed, trace_core);
    apply_budget(elided)
}

/// Collapses runs of >3 consecutive identical (module, name, line) frames (IS §5.2).
fn collapse_repeats(views: Vec<FrameView>) -> Vec<HumanItem> {
    let mut out: Vec<HumanItem> = Vec::new();
    let mut idx = 0;
    while idx < views.len() {
        let f = &views[idx];
        let mut run = 0usize;
        while idx + 1 + run < views.len() {
            let next = &views[idx + 1 + run];
            if same_frame_identity(f, next) {
                run += 1;
            } else {
                break;
            }
        }
        if run >= 3 {
            out.push(HumanItem::Repeated { first: f.clone(), count: run });
            idx += 1 + run;
        } else {
            out.push(HumanItem::Frame(f.clone()));
            idx += 1;
        }
    }
    out
}

/// Returns `true` when two `FrameView`s represent the same logical source location.
fn same_frame_identity(a: &FrameView, b: &FrameView) -> bool {
    a.module == b.module && a.name == b.name && a.line == b.line
}

/// Replaces runs of core frames with [`HumanItem::ElidedCore`] unless `trace_core` is set.
fn elide_core(items: Vec<HumanItem>, trace_core: bool) -> Vec<HumanItem> {
    if trace_core {
        return items;
    }
    let mut out = Vec::with_capacity(items.len());
    let mut pending_core = 0usize;

    for item in items {
        let is_core = match &item {
            HumanItem::Frame(v) => v.is_core,
            HumanItem::Repeated { first, .. } => first.is_core,
            _ => false,
        };
        if is_core {
            pending_core += 1;
        } else {
            if pending_core > 0 {
                out.push(HumanItem::ElidedCore { count: pending_core });
                pending_core = 0;
            }
            out.push(item);
        }
    }
    if pending_core > 0 {
        out.push(HumanItem::ElidedCore { count: pending_core });
    }
    out
}

/// Applies the 40-frame budget: keeps oldest 15 + newest 15, elides the middle (IS §5.2).
fn apply_budget(items: Vec<HumanItem>) -> Vec<HumanItem> {
    // Count displayable (non-boundary) frames.
    let normal_count: usize = items.iter().filter(|i| matches!(i, HumanItem::Frame(_) | HumanItem::Repeated { .. })).count();
    if normal_count <= 40 {
        return items;
    }
    let mut out = Vec::new();
    let mut seen = 0usize;
    let mut elided = 0usize;
    for item in items {
        match item {
            HumanItem::Frame(_) | HumanItem::Repeated { .. } => {
                seen += 1;
                if seen <= 15 || seen > normal_count - 15 {
                    out.push(item);
                } else {
                    elided += 1;
                    if elided == 1 {
                        // Placeholder; count patched after the loop.
                        out.push(HumanItem::ElidedMiddle { count: 0 });
                    }
                }
            }
            other => out.push(other),
        }
    }
    // Patch in the real elided count.
    for item in &mut out {
        if let HumanItem::ElidedMiddle { count } = item {
            *count = elided;
        }
    }
    out
}

/// Emits `items` into `out` as human-readable lines.
///
/// `last_group` indicates whether this is the innermost group — only the
/// very last frame in the very last group skips the source echo, because the
/// caret block is emitted after the × line by the top-level caller.
fn emit_human_items(out: &mut String, vm: &mut VM, items: &[HumanItem], _config: &RenderConfig, styler: &Styler, last_group: bool) {
    let item_count = items.len();
    for (i, item) in items.iter().enumerate() {
        let is_innermost = last_group && i + 1 == item_count;
        match item {
            HumanItem::Frame(v) => {
                emit_frame_line(out, vm, v, styler);
                if !is_innermost {
                    emit_source_echo(out, v);
                }
                // Innermost frame: caret block emitted after × by render_human_traceback.
            }
            HumanItem::Repeated { first, count } => {
                emit_frame_line(out, vm, first, styler);
                emit_source_echo(out, first);
                out.push_str(&format!(
                    "  {}\n",
                    styler.paint(Role::Elision, &format!("[previous frame repeated {} more times]", count),)
                ));
            }
            HumanItem::ElidedCore { count } => {
                out.push_str(&format!(
                    "  {}\n",
                    styler.paint(
                        Role::Elision,
                        &format!(
                            "[{} core frame{} elided — pass --trace-core to expand]",
                            count,
                            if *count == 1 { "" } else { "s" }
                        ),
                    )
                ));
            }
            HumanItem::ElidedMiddle { count } => {
                out.push_str(&format!("  {}\n", styler.paint(Role::Elision, &format!("[… {} frames elided …]", count))));
            }
        }
    }
}

/// Emits one `  file:line   in name` header line.
fn emit_frame_line(out: &mut String, vm: &VM, view: &FrameView, styler: &Styler) {
    let module_name = vm.resolve_symbol(view.module);
    let file = format!("{}.ph", module_name);
    let name = frame_name_to_string(vm, &view.name);
    out.push_str(&format!(
        "  {}   in {}\n",
        styler.paint(Role::Location, &format!("{}:{}", file, view.line)),
        styler.paint(Role::Identifier, &name),
    ));
}

/// Emits one echoed source line (ordinary frames), trimming trailing whitespace.
fn emit_source_echo(out: &mut String, view: &FrameView) {
    if let Some(src) = &view.source {
        let text = get_line(src, view.line as usize);
        if !text.trim().is_empty() {
            out.push_str(&format!("      {}\n", text.trim_end()));
        }
    }
}

/// Builds the caret block for the innermost live frame (IS §5.1).
///
/// Returns `None` when the innermost frame has no source text or there are no
/// frames at all.
fn innermost_caret_block(vm: &VM, err: &PhError, config: &RenderConfig) -> Option<String> {
    let frame = vm.frames.last()?;
    let closure = vm.heap.closure(frame.closure);
    let module = vm.heap.module(closure.module);
    let source_id = closure.callable.chunk.source_id;
    let source: Arc<String> = module.source_at(source_id)?.clone();
    let span_index = frame.ip.saturating_sub(1);
    let span: SourceRange = closure.callable.chunk.span_at(span_index);
    let module_name = vm.resolve_symbol(module.name_sym);
    let file = format!("{}.ph", module_name);
    let msg = err.to_string();
    let label = Label {
        span,
        text: &msg,
        kind: LabelKind::Primary,
    };
    let snippet = Snippet::with_file(file);
    Some(snippet.render(&source, &[label], config))
}

// ──────────────────────────────────────────────────────────────────────────────
// Native frame synthesis (IS §5.5)
// ──────────────────────────────────────────────────────────────────────────────

/// Builds a native-frame line when the VM has an in-flight selector+class (IS §5.5).
///
/// Returns `None` when no native context is set.
fn maybe_native_frame_line(vm: &VM, styler: &Styler) -> Option<String> {
    let sel = vm.native_selector?;
    let cls = vm.native_class?;
    let sel_str = vm.resolve_symbol(sel);
    let cls_str = vm.resolve_symbol(cls);
    Some(format!(
        "  {}   in {}\n",
        styler.paint(Role::Location, "[native]"),
        styler.paint(Role::Identifier, &format!("{}.{}", cls_str, sel_str)),
    ))
}

// ──────────────────────────────────────────────────────────────────────────────
// Record-based frame helpers (caught-error path)
// ──────────────────────────────────────────────────────────────────────────────

/// One fiber group from a `FrameRecord` vec.
///
/// Fields: `(fiber_seq, boundary_info, frame_slice)`.
///
/// `boundary_info` carries the fiber link info `(spawn_file, spawn_line)` that
/// precedes this group; `None` for the first group (no prior fiber crossing).
type FiberGroup<'a> = (u32, Option<(Option<Symbol>, u32)>, &'a [FrameRecord]);

/// Partitions a flat `FrameRecord` slice into fiber groups.
///
/// Each `FiberBoundary` record ends the current group and starts a new one.
fn partition_records<'a>(records: &'a [FrameRecord], vm: &VM) -> Vec<FiberGroup<'a>> {
    let mut groups: Vec<FiberGroup<'a>> = Vec::new();
    let current_fiber_seq = vm.heap.fiber(vm.current).seq;

    let mut group_start = 0usize;
    let mut current_seq = current_fiber_seq;
    let mut pending_boundary: Option<(Option<Symbol>, u32)> = None;

    for (i, rec) in records.iter().enumerate() {
        if let FrameRecord::FiberBoundary { seq, spawn_file, spawn_line } = rec {
            // Close the current group up to (not including) this boundary.
            groups.push((current_seq, pending_boundary.take(), &records[group_start..i]));
            group_start = i + 1;
            current_seq = *seq;
            pending_boundary = Some((*spawn_file, *spawn_line));
        }
    }
    // Push the last (or only) group.
    groups.push((current_seq, pending_boundary, &records[group_start..]));
    groups
}

/// Converts a slice of `FrameRecord::Normal` entries into synthetic `FrameView`s.
///
/// Boundary records within the slice are skipped (already consumed by
/// [`partition_records`]). Records carry only `line` (not byte `span`), so the
/// caret block falls back to the live-walk path instead.
fn records_to_views(vm: &mut VM, records: &[FrameRecord], fiber_seq: u32) -> Vec<FrameView> {
    let mut views = Vec::with_capacity(records.len());
    for rec in records {
        if let FrameRecord::Normal { module, method, line } = rec {
            // Best-effort source resolution via source_id 0.
            let source = vm
                .find_module_by_symbol(*module)
                .and_then(|mod_ref| vm.heap.module(mod_ref).source_at(0).cloned());
            let is_core = is_core_module(vm, *module);
            views.push(FrameView {
                module: *module,
                name: FrameName::Method(*method),
                line: *line,
                // Records store line numbers only; byte span is unavailable.
                span: SourceRange::new(0, 0),
                source,
                is_core,
                fiber: fiber_seq,
                class_name: None,
            });
        }
    }
    views
}

// ──────────────────────────────────────────────────────────────────────────────
// Formatting utilities
// ──────────────────────────────────────────────────────────────────────────────

/// Renders a [`FrameName`] to a human-readable string.
fn frame_name_to_string(vm: &VM, name: &FrameName) -> String {
    match name {
        FrameName::Main => "<main>".to_string(),
        FrameName::Method(sym) => vm.resolve_symbol(*sym).to_string(),
        FrameName::Block { enclosing } => {
            let enc = vm.resolve_symbol(*enclosing);
            format!("<closure in {}>", enc)
        }
        FrameName::Native(sym) => {
            let s = vm.resolve_symbol(*sym);
            format!("[native {}]", s)
        }
    }
}

/// Returns `true` when `module_sym` resolves to the bootstrap core module.
///
/// Uses handle identity (IS §2.1: "not a name check").
fn is_core_module(vm: &VM, module_sym: Symbol) -> bool {
    let Some(core_ref) = vm.core_module() else {
        return false;
    };
    vm.find_module_by_symbol(module_sym).is_some_and(|m| m == core_ref)
}

/// Returns the `kind` string for an error, or `""` when none applies.
fn extract_kind_str(vm: &mut VM, err: &PhError) -> String {
    match err {
        PhError::Runtime(RuntimeError::Raise {
            error: crate::value::Value::Obj(id),
            ..
        }) => vm
            .heap
            .as_instance(*id)
            .and_then(|instance| instance.slots.get(1))
            .and_then(|kind| match kind {
                crate::value::Value::Symbol(sym) => Some(vm.resolve_symbol(*sym).to_string()),
                _ => None,
            })
            .unwrap_or_default(),
        PhError::Runtime(rt) => {
            if let Some(crate::value::Value::Symbol(s)) = vm.error_kind_symbol(rt) {
                return vm.resolve_symbol(s).to_string();
            }
            String::new()
        }
        _ => String::new(),
    }
}

/// Returns the text of 1-based `line_num` from `source`, or `""` when out of range.
fn get_line(source: &str, line_num: usize) -> &str {
    source.lines().nth(line_num.saturating_sub(1)).unwrap_or("")
}

// ──────────────────────────────────────────────────────────────────────────────
// JSON helpers (no serde_json dependency)
// ──────────────────────────────────────────────────────────────────────────────

/// Serializes one frame as a JSON object string (IS §5.4).
fn json_frame(module: &str, file: &str, line: u32, name: &str, core: bool, fiber: u32) -> String {
    format!(
        "{{\"module\":{},\"file\":{},\"line\":{},\"name\":{},\"core\":{},\"fiber\":{}}}",
        json_str(module),
        json_str(file),
        line,
        json_str(name),
        core,
        fiber,
    )
}

/// Escapes `s` as a JSON string literal (double-quoted).
pub(crate) fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Computes the optional `help:` suggestion line for human traceback output (IS §9).
fn get_help_suggestion(vm: &mut VM, err: &PhError, config: &RenderConfig, styler: &Styler) -> Option<String> {
    match err {
        PhError::Runtime(RuntimeError::Raise { help: Some(h), .. }) => Some(format!("  {} {}\n", styler.paint(Role::SeverityHelp, "help:"), h)),
        PhError::Runtime(RuntimeError::UndefinedVariable { name }) => {
            let mut candidates = Vec::new();
            // 1. Locals from innermost frame (if present)
            if let Some(frame) = vm.frames.last() {
                let closure = vm.heap.closure(frame.closure);
                for &sym in &closure.callable.local_names {
                    candidates.push(vm.resolve_symbol(sym).to_string());
                }

                // 2. Module globals
                let module = vm.heap.module(closure.module);
                for &sym in module.name_to_slot.keys() {
                    candidates.push(vm.resolve_symbol(sym).to_string());
                }
            }

            // 3. Core globals
            if let Some(core_module_ref) = vm.core_module() {
                let core_module = vm.heap.module(core_module_ref);
                for &sym in core_module.name_to_slot.keys() {
                    candidates.push(vm.resolve_symbol(sym).to_string());
                }
            }

            candidates.sort();
            candidates.dedup();

            let cand_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
            crate::diagnostics::suggest::best_match(name, cand_refs.into_iter())
                .map(|sug| format!("  {} did you mean '{}'?\n", styler.paint(Role::SeverityHelp, "help:"), sug))
        }
        _ => None,
    }
}
