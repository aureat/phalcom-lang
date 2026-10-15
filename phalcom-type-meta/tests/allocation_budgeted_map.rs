use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, DuplicateKey};

struct CountingSystem;

static TRACK_REQUESTS: AtomicBool = AtomicBool::new(false);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);
static FAIL_NEXT_REQUEST: AtomicBool = AtomicBool::new(false);

// SAFETY: This allocator forwards every operation unchanged to `System` and
// only records requested layout sizes using atomics while the test enables it.
unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACK_REQUESTS.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        if FAIL_NEXT_REQUEST.swap(false, Ordering::Relaxed) {
            return std::ptr::null_mut();
        }
        // SAFETY: `layout` is passed through unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if TRACK_REQUESTS.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        if FAIL_NEXT_REQUEST.swap(false, Ordering::Relaxed) {
            return std::ptr::null_mut();
        }
        // SAFETY: `layout` is passed through unchanged to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` and `layout` came from the forwarded system allocation.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if TRACK_REQUESTS.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(new_size, Ordering::Relaxed);
        }
        if FAIL_NEXT_REQUEST.swap(false, Ordering::Relaxed) {
            return std::ptr::null_mut();
        }
        // SAFETY: `ptr`, `layout`, and `new_size` are forwarded unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingSystem = CountingSystem;

#[test]
fn budgeted_collections_and_allocation_helpers_match_requested_bytes() {
    let expected = Layout::array::<(u64, u64)>(3).unwrap().size();
    let mut budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let mut builder = budget.sorted_map_builder::<u64, u64>(3).unwrap();
    builder.push(8, 80).unwrap();
    builder.push(2, 20).unwrap();
    builder.push(5, 50).unwrap();
    let map = builder.finish().unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);

    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), expected);
    assert_eq!(budget.requested(), expected);
    assert_eq!(map.iter().map(|(key, value)| (*key, *value)).collect::<Vec<_>>(), [(2, 20), (5, 50), (8, 80)]);
    assert_eq!(map.get(&5), Some(&50));
    assert!(map.contains_key(&8));
    assert_eq!(map.range(3..=8).map(|(key, _)| *key).collect::<Vec<_>>(), [5, 8]);

    sorted_map_growth_charges_each_replacement_request();
    sorted_map_rejects_duplicates_and_budget_failures_do_not_charge_unmade_requests();
    sorted_map_supports_borrowed_string_lookup_and_bounds();
    sorted_set_uses_budgeted_growth_and_rejects_duplicates();
    allocation_helpers_match_observed_requests();
    allocator_failures_restore_uncommitted_budget_charges();
}

fn sorted_map_growth_charges_each_replacement_request() {
    let expected = (1..=3).map(|count| Layout::array::<(u64, u64)>(count).unwrap().size()).sum::<usize>();
    let mut budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let mut builder = budget.sorted_map_builder::<u64, u64>(0).unwrap();
    builder.push(1, 10).unwrap();
    builder.push(2, 20).unwrap();
    builder.push(3, 30).unwrap();
    let map = builder.finish().unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);

    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), expected);
    assert_eq!(budget.requested(), expected);
    assert_eq!(map.len(), 3);
}

fn sorted_map_rejects_duplicates_and_budget_failures_do_not_charge_unmade_requests() {
    let mut budget = AllocationBudget::new(1024);
    let mut duplicate = budget.sorted_map_builder::<u64, u64>(2).unwrap();
    duplicate.push(3, 1).unwrap();
    duplicate.push(3, 2).unwrap();
    assert_eq!(duplicate.finish(), Err(DuplicateKey));

    let mut short = AllocationBudget::new(Layout::array::<u64>(2).unwrap().size());
    let mut values = short.vector::<u64>(2).unwrap();
    values.extend([1, 2]);
    let before = (short.remaining(), short.requested());
    assert_eq!(short.grow_vector_exact(&mut values, 1), Err(AllocationError::Exhausted));
    assert_eq!((short.remaining(), short.requested()), before);
    assert_eq!(values, [1, 2]);

    let mut too_small = AllocationBudget::new(1);
    assert_eq!(too_small.vector::<u64>(1), Err(AllocationError::Exhausted));
    assert_eq!((too_small.remaining(), too_small.requested()), (1, 0));
}

fn sorted_map_supports_borrowed_string_lookup_and_bounds() {
    let mut budget = AllocationBudget::new(4096);
    let mut builder = budget.sorted_map_builder::<String, u8>(3).unwrap();
    builder.push("zeta".into(), 3).unwrap();
    builder.push("alpha".into(), 1).unwrap();
    builder.push("beta".into(), 2).unwrap();
    let map = builder.finish().unwrap();
    assert_eq!(map.get("beta"), Some(&2));
    assert_eq!(
        map.range(String::from("alpha")..String::from("zeta"))
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
}

fn sorted_set_uses_budgeted_growth_and_rejects_duplicates() {
    let mut budget = AllocationBudget::new(24);
    let mut builder = budget.sorted_set_builder::<u64>(1).unwrap();
    builder.push(9).unwrap();
    builder.push(3).unwrap();
    assert!(matches!(builder.push(7), Err(AllocationError::Exhausted)));
    let set = builder.finish().unwrap();
    assert_eq!(set.iter().copied().collect::<Vec<_>>(), [3, 9]);
    assert!(set.contains(&9));
    assert_eq!(set.range(3..9).copied().collect::<Vec<_>>(), [3]);

    let mut duplicate_budget = AllocationBudget::new(1024);
    let mut duplicate = duplicate_budget.sorted_set_builder::<u64>(2).unwrap();
    duplicate.push(5).unwrap();
    duplicate.push(5).unwrap();
    assert_eq!(duplicate.finish(), Err(DuplicateKey));
}

fn allocation_helpers_match_observed_requests() {
    let mut vector_budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let vector = vector_budget.vector::<u64>(3).unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), vector_budget.requested());
    assert_eq!(vector.capacity(), 3);

    let mut string_budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let owned = string_budget.string("allocation").unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), string_budget.requested());
    assert_eq!(owned, "allocation");

    let mut boxed_str_budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let boxed_str = boxed_str_budget.boxed_str("portable").unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), boxed_str_budget.requested());
    assert_eq!(boxed_str.as_ref(), "portable");

    let mut boxed_budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let boxed = boxed_budget.boxed(17_u64).unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), boxed_budget.requested());
    assert_eq!(*boxed, 17);

    let mut arc_budget = AllocationBudget::new(1024);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let arc = arc_budget.arc([1_u8, 2, 3]).unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), arc_budget.requested());
    assert_eq!(*arc, [1, 2, 3]);
}

fn allocator_failures_restore_uncommitted_budget_charges() {
    let mut vector_budget = AllocationBudget::new(1024);
    FAIL_NEXT_REQUEST.store(true, Ordering::Relaxed);
    assert_eq!(vector_budget.vector::<u64>(3), Err(AllocationError::Failed));
    assert_eq!((vector_budget.remaining(), vector_budget.requested()), (1024, 0));

    let mut string_budget = AllocationBudget::new(1024);
    FAIL_NEXT_REQUEST.store(true, Ordering::Relaxed);
    assert_eq!(string_budget.boxed_str("failure"), Err(AllocationError::Failed));
    assert_eq!((string_budget.remaining(), string_budget.requested()), (1024, 0));

    let mut grow_budget = AllocationBudget::new(1024);
    let mut values = grow_budget.vector::<u64>(1).unwrap();
    values.push(7);
    let before = (grow_budget.remaining(), grow_budget.requested());
    FAIL_NEXT_REQUEST.store(true, Ordering::Relaxed);
    assert_eq!(grow_budget.grow_vector_exact(&mut values, 2), Err(AllocationError::Failed));
    assert_eq!((grow_budget.remaining(), grow_budget.requested()), before);
    assert_eq!(values, [7]);
}
