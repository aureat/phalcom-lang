use phalcom_type_meta::allocation::AllocationBudget;
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(align(16))]
struct NontrivialItem {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

#[test]
fn test_allocation_helpers() {
    let mut budget = AllocationBudget::new(1024);
    let values: Vec<u32> = Vec::new();

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let arc = budget.arc_slice(values).expect("empty arc slice");
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    assert_eq!(arc.len(), 0);
    assert_eq!(observed, budget.requested(), "observed must equal ledger debit for empty slice");

    let mut budget = AllocationBudget::new(1024);
    let mut values = budget.vector::<u32>(4).unwrap();
    values.extend([10, 20, 30, 40]);
    let before_arc_debit = budget.requested();

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let arc = budget.arc_slice(values).expect("exact capacity arc slice");
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    let arc_debit = budget.requested() - before_arc_debit;
    assert_eq!(observed, arc_debit, "observed must equal ledger debit for exact capacity slice");
    assert_eq!(&*arc, &[10, 20, 30, 40]);

    let mut budget = AllocationBudget::new(1024);
    let mut values = budget.vector::<u32>(8).unwrap();
    values.extend([1, 2, 3]);
    let before_arc_debit = budget.requested();

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let arc = budget.arc_slice(values).expect("over capacity arc slice");
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    let arc_debit = budget.requested() - before_arc_debit;
    assert_eq!(observed, arc_debit, "observed must equal ledger debit for over-capacity slice");
    assert_eq!(&*arc, &[1, 2, 3]);

    let mut budget = AllocationBudget::new(4096);
    let mut values = budget.vector::<NontrivialItem>(2).unwrap();
    values.push(NontrivialItem { a: 1, b: 2, c: 3, d: 4 });
    values.push(NontrivialItem { a: 5, b: 6, c: 7, d: 8 });
    let before_arc_debit = budget.requested();

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let arc = budget.arc_slice(values).expect("nontrivial arc slice");
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    let arc_debit = budget.requested() - before_arc_debit;
    assert_eq!(observed, arc_debit, "observed must equal ledger debit for nontrivial item slice");
    assert_eq!(arc.len(), 2);

    let mut budget = AllocationBudget::new(1024);

    // Empty
    let v_empty: Vec<u64> = Vec::new();
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let b_empty = budget.boxed_slice(v_empty).unwrap();
    MEASURING.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), 0);
    assert_eq!(b_empty.len(), 0);

    // Exact
    let mut v_exact = budget.vector::<u64>(3).unwrap();
    v_exact.extend([1, 2, 3]);
    let before = budget.requested();
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let b_exact = budget.boxed_slice(v_exact).unwrap();
    MEASURING.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), 0);
    assert_eq!(budget.requested() - before, 0);
    assert_eq!(&*b_exact, &[1, 2, 3]);

    // Over-capacity
    let mut v_over = budget.vector::<u64>(5).unwrap();
    v_over.extend([10, 20]);
    let before = budget.requested();
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let b_over = budget.boxed_slice(v_over).unwrap();
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);
    let debit = budget.requested() - before;
    assert_eq!(observed, debit, "observed must equal debit on shrink");
    assert_eq!(&*b_over, &[10, 20]);

    // Arc str
    let mut budget = AllocationBudget::new(1024);
    let s = "hello world";

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let arc = budget.arc_str(s).expect("arc str");
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    assert_eq!(&*arc, s);
    assert_eq!(observed, budget.requested(), "observed must equal ledger debit for arc_str");

    // N-1 test
    let expected = budget.requested();
    let mut short_budget = AllocationBudget::new(expected - 1);
    assert!(short_budget.arc_str(s).is_err());
    assert_eq!(short_budget.requested(), 0);
}

