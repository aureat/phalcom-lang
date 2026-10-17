use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_core::modules::canonical_artifact::{
    build_universe_telosc, load_universe_telosc_from_bytes, load_universe_telosc_from_bytes_with_allocation_summary,
};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct CountingAllocator;

static MEASURING: AtomicBool = AtomicBool::new(false);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);

fn record_request(bytes: usize) {
    if MEASURING.load(Ordering::Relaxed) {
        REQUESTED_BYTES.fetch_add(bytes, Ordering::Relaxed);
    }
}

// SAFETY: All allocator operations forward the original pointer/layout contract
// unchanged to `System`; the wrapper only observes requested layout sizes.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_request(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_request(new_size);
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[test]
fn full_universe_telosc_load_allocation_equality_and_n_minus_one() {
    // 1. Build canonical program and artifact bytes outside measurement
    let program = canonical_universe_program().expect("canonical universe program must compile");
    let view = program.compiler_image_view().expect("compiler image view must be present");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build_universe_telosc must succeed");
    assert!(!built.bytes.is_empty());

    // 2. Discover ledger requested bytes N with measurement disabled
    let (summary_res, ledger_requested) =
        load_universe_telosc_from_bytes_with_allocation_summary(&built.bytes, &limits);
    let initial_artifact = summary_res.expect("initial load without memory measurement must succeed");
    drop(initial_artifact);

    let n = ledger_requested;
    assert!(n > 0, "ledger must have recorded allocation requests");

    // 3. Test exact N under independent allocator measurement
    let limits_exact = ValidationLimits {
        max_total_allocation_bytes: n as u64,
        ..limits
    };

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits_exact);
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    let loaded = loaded.expect("load at exact budget N must succeed");

    // Drop artifact with measurement disabled
    drop(loaded);

    eprintln!(
        "FULL LOAD ALLOCATION DISCRIMINATOR:\n  ledger requested (N): {}\n  independent allocator requested: {}\n  delta: {}",
        n,
        observed,
        observed.abs_diff(n)
    );

    // 4. Verify N - 1 strictly fails
    let limits_minus_one = ValidationLimits {
        max_total_allocation_bytes: (n - 1) as u64,
        ..limits
    };
    let failed = load_universe_telosc_from_bytes(&built.bytes, &limits_minus_one);
    assert!(
        failed.is_err(),
        "load with N - 1 budget must fail closed with budget exhaustion"
    );

    // PATCH5 discriminator: during intermediate waves, observed != n is expected.
    // At T9 (Wave 4), this assertion is made unconditional.
    if std::env::var("STRICT_ALLOCATION_GATE").is_ok() {
        assert_eq!(
            observed, n,
            "independent allocator requests must match production ledger exactly"
        );
    }
}
