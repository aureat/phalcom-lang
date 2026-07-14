//! Per-opcode execution counters for the VM dispatch loop (perf-log hole H3).
//!
//! Compiled out entirely unless the `opcode-histogram` feature is on. Answers the
//! one question the rest of the harness cannot: **what does a single instruction
//! cost?** Profiling cannot — the dispatch loop is one `match` in one function, so
//! `sample` attributes every opcode arm to `run_until_inner` and prices none of
//! them (perf-log §4: that frame is 27–35% of ticks, unattributed).
//!
//! # Measuring without the observer effect
//!
//! An increment per instruction is not free, and this crate has already measured
//! what per-opcode work costs in the same loop: `vm-trace`'s span was 18.2% of arith
//! wall-clock (perf-log 003). So **a timing read from a build with this feature on
//! is wrong**, and this module deliberately reports no times.
//!
//! What makes that a non-problem: **counts are deterministic**. The same program
//! over the same inputs retires exactly the same instruction mix in both builds. So
//! the protocol is two runs —
//!
//! 1. counts from a `--features opcode-histogram` build,
//! 2. wall-clock from a **default** build,
//!
//! divided. The counter never touches the number it produces.
//! `benchmarks/vm/opcode-cost.py` mechanizes this; do not hand-roll it, and do not
//! read the times printed by a histogram build.
//!
//! # What the average does and does not buy
//!
//! `wall / total` is a true mean over the *executed mix*, not a per-opcode price:
//! a `Loop` and an `Invoke` land in the same average. Comparing that mean across
//! benchmarks with different mixes is what makes it informative — a program that is
//! 60% `Invoke` and one that is 60% `GetLocal` do not have the same mean, and the
//! spread is the signal. Pricing an individual opcode needs a differential (two
//! programs differing by a known count of one opcode), which the histogram makes
//! constructible but does not itself perform.
//!
//! # Pairs, and why "adjacent" is narrower than it sounds (H13)
//!
//! The single-opcode histogram cannot select superinstructions: it says `Invoke` is
//! 13–25% of every hot mix but never what `Invoke` *follows* (perf-log F16 reason 2).
//! [`PAIRS`] fills that in — but only counts a pair the compiler could actually fuse.
//!
//! A fusion is a compile-time rewrite of two opcodes **adjacent in one chunk's code
//! array** into one. So a dynamic "previous instruction executed" is the wrong
//! predicate: the opcode before a callee's first instruction is the caller's
//! `Invoke`, and the opcode before a loop body's first is a `Loop` from the bottom.
//! Those pairs are execution-adjacent and **not fusible** — counting them would
//! inflate exactly the pairs nobody can act on, which is the failure mode that makes
//! a histogram look like an answer.
//!
//! A pair is therefore counted only when the current instruction is the **static
//! successor** of the previous one: same closure, and `ip == prev_ip + 1`. Every
//! call, return, jump, loop back-edge and fiber switch breaks the chain and is
//! counted as no pair at all. `sum(pairs) < total - 1` is expected, and the deficit
//! is a real quantity: it is the control-flow-transfer count.

use crate::bytecode::{Bytecode, BYTECODE_NAMES};
use crate::heap::ObjRef;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Execution count per opcode, indexed by [`Bytecode::index`].
    ///
    /// Thread-local rather than atomic: the VM is single-threaded (fibers are
    /// cooperative and share one OS thread — ADR-0030), so an atomic would buy
    /// nothing and cost a lock-prefixed instruction on the hottest path in the
    /// interpreter, distorting the very mix being counted.
    static COUNTS: RefCell<[u64; Bytecode::VARIANTS]> = const { RefCell::new([0; Bytecode::VARIANTS]) };

    /// Execution count per **statically adjacent** opcode pair, indexed
    /// `[prev][cur]` by [`Bytecode::index`]. See the module docs: a pair is only
    /// counted when `cur` is `prev`'s static successor, because only such a pair
    /// is fusible into a superinstruction.
    static PAIRS: RefCell<[[u64; Bytecode::VARIANTS]; Bytecode::VARIANTS]> =
        const { RefCell::new([[0; Bytecode::VARIANTS]; Bytecode::VARIANTS]) };

    /// The previous instruction's `(closure, ip, opcode index)`, or `None` at the
    /// start of a run. Read only to decide static adjacency.
    static PREV: Cell<Option<(ObjRef, usize, usize)>> = const { Cell::new(None) };
}

/// Records one execution of `opcode`, dispatched at `ip` in `closure`'s chunk.
///
/// Called once per dispatched instruction from
/// [`run_until_inner`](crate::vm::VM), behind `#[cfg(feature = "opcode-histogram")]`.
///
/// `closure` and `ip` are what let this distinguish a fusible pair from a mere
/// execution-order neighbour (module docs, H13). Pass the **pre-increment** `ip` —
/// the index the opcode was actually read from.
#[inline]
pub fn record(opcode: &Bytecode, closure: ObjRef, ip: usize) {
    let cur = opcode.index();
    COUNTS.with(|c| c.borrow_mut()[cur] += 1);

    // Count the pair only across a straight-line step within one chunk. A call,
    // return, jump, back-edge or fiber switch lands here with a different closure
    // or a non-successor ip, and is deliberately counted as no pair.
    if let Some((prev_closure, prev_ip, prev)) = PREV.with(Cell::get)
        && prev_closure == closure
        && prev_ip + 1 == ip
    {
        PAIRS.with(|p| p.borrow_mut()[prev][cur] += 1);
    }
    PREV.with(|p| p.set(Some((closure, ip, cur))));
}

/// Returns the current counts, indexed by [`Bytecode::index`].
pub fn snapshot() -> [u64; Bytecode::VARIANTS] {
    COUNTS.with(|c| *c.borrow())
}

/// Returns the current statically-adjacent pair counts, indexed `[prev][cur]`.
pub fn pair_snapshot() -> [[u64; Bytecode::VARIANTS]; Bytecode::VARIANTS] {
    PAIRS.with(|p| *p.borrow())
}

/// Writes the histogram to stderr, descending by count.
///
/// Deliberately stderr, not stdout: every golden fixture in `tests/lang/` asserts
/// exact stdout, and the wren-suite comparison diffs stdout byte-for-byte against
/// Wren's. Printing the histogram to stdout would fail every one of them and make
/// the feature unusable on the corpus it most needs to measure.
pub fn dump() {
    let counts = snapshot();
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return;
    }

    let mut rows: Vec<(usize, u64)> = counts.iter().copied().enumerate().filter(|(_, n)| *n > 0).collect();
    rows.sort_by_key(|(_, n)| std::cmp::Reverse(*n));

    eprintln!("opcode histogram: {total} instructions retired");
    for (idx, n) in rows {
        let share = (n as f64 / total as f64) * 100.0;
        eprintln!("{:>16}  {:>14}  {:>5.1}%", BYTECODE_NAMES[idx], n, share);
    }
    eprintln!("{:>16}  {:>14}", "TOTAL", total);

    dump_pairs(total);
}

/// Writes the top statically-adjacent opcode pairs to stderr, descending by count.
///
/// These are the superinstruction candidates (H13): each row is a `(prev, cur)` the
/// compiler could fuse, with the share of all retired instructions the fusion would
/// remove. Shown as a share of `total` rather than of the pair count, because that
/// is the quantity a fusion actually buys — one dispatch removed per occurrence.
fn dump_pairs(total: u64) {
    let pairs = pair_snapshot();
    let mut rows: Vec<(usize, usize, u64)> = pairs
        .iter()
        .enumerate()
        .flat_map(|(prev, row)| row.iter().copied().enumerate().map(move |(cur, n)| (prev, cur, n)))
        .filter(|(_, _, n)| *n > 0)
        .collect();
    if rows.is_empty() {
        return;
    }
    rows.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));

    let paired: u64 = rows.iter().map(|(_, _, n)| *n).sum();
    eprintln!();
    eprintln!("statically-adjacent opcode pairs (superinstruction candidates):");
    for (prev, cur, n) in rows.iter().take(20) {
        let share = (*n as f64 / total as f64) * 100.0;
        eprintln!(
            "{:>16} -> {:<16} {:>14}  {:>5.1}% of all instrs",
            BYTECODE_NAMES[*prev], BYTECODE_NAMES[*cur], n, share
        );
    }
    // The deficit is not slack: every instruction that is not the static successor
    // of its predecessor was reached by a control-flow transfer. Reporting it keeps
    // the pair shares honest — they are shares of `total`, not of `paired`.
    let transfers = total.saturating_sub(paired).saturating_sub(1);
    eprintln!(
        "{:>16}  {paired} pairs over {total} instructions ({transfers} control-flow transfers)",
        "TOTAL"
    );
}
