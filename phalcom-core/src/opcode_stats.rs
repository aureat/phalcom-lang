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

use crate::bytecode::{Bytecode, BYTECODE_NAMES};
use std::cell::RefCell;

thread_local! {
    /// Execution count per opcode, indexed by [`Bytecode::index`].
    ///
    /// Thread-local rather than atomic: the VM is single-threaded (fibers are
    /// cooperative and share one OS thread — ADR-0030), so an atomic would buy
    /// nothing and cost a lock-prefixed instruction on the hottest path in the
    /// interpreter, distorting the very mix being counted.
    static COUNTS: RefCell<[u64; Bytecode::VARIANTS]> = const { RefCell::new([0; Bytecode::VARIANTS]) };
}

/// Records one execution of `opcode`.
///
/// Called once per dispatched instruction from
/// [`run_until_inner`](crate::vm::VM), behind `#[cfg(feature = "opcode-histogram")]`.
#[inline]
pub fn record(opcode: &Bytecode) {
    COUNTS.with(|c| c.borrow_mut()[opcode.index()] += 1);
}

/// Returns the current counts, indexed by [`Bytecode::index`].
pub fn snapshot() -> [u64; Bytecode::VARIANTS] {
    COUNTS.with(|c| *c.borrow())
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
}
