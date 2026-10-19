use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::query::{LegacyPortableQueryKeyV1, PortableQueryKindV1, QueryImageV2, QueryProductRefV2, QueryRootEntryV2};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATED.fetch_add(new_size, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn encoded_query(count: usize) -> Vec<u8> {
    QueryImageV2 {
        supported_roots: (0..count)
            .map(|index| QueryRootEntryV2 {
                key: LegacyPortableQueryKeyV1 {
                    kind: PortableQueryKindV1::CallableSignature,
                    stable_owner_key: format!("{index:05}"),
                },
                product: QueryProductRefV2 {
                    kind: PortableQueryKindV1::CallableSignature,
                    row_index: 0,
                },
                input_fingerprint: index as u64,
                product_fingerprint: index as u64 + 1,
                dependency_indices: Vec::new(),
            })
            .collect(),
        dependency_evidence: Vec::new(),
    }
    .encode()
    .unwrap()
}

fn query_decode_rejects_before_vector_allocation_when_transaction_is_too_small() {
    let bytes = encoded_query(50_000);
    let mut transaction = AllocationBudget::new(bytes.len());
    let before = ALLOCATED.load(Ordering::Relaxed);
    let result = QueryImageV2::decode_with_allocation_budget(&bytes, &ValidationLimits::default(), &mut transaction);
    let allocated = ALLOCATED.load(Ordering::Relaxed) - before;

    assert!(result.is_err());
    assert!(matches!(result, Err(ArtifactError::Allocation(AllocationError::Exhausted))));
    assert_eq!(transaction.requested(), 0, "failed child decode must not commit");
    assert!(allocated < 1024, "rejected decode allocated {allocated} bytes");
}

fn query_decode_enforces_section_limit_independently_of_transaction_budget() {
    let bytes = encoded_query(128);
    let limits = ValidationLimits {
        max_decoded_section_bytes: 1,
        ..ValidationLimits::default()
    };
    let mut transaction = AllocationBudget::new(1024 * 1024);
    let result = QueryImageV2::decode_with_allocation_budget(&bytes, &limits, &mut transaction);

    assert!(matches!(result, Err(ArtifactError::Allocation(AllocationError::Exhausted))));
    assert_eq!(transaction.requested(), 0, "failed child decode must not commit");
}

fn query_decode_debit_matches_allocator_requested_capacity() {
    let count = 50_000;
    let bytes = encoded_query(count);
    let expected = Layout::array::<QueryRootEntryV2>(count).unwrap().size() + count * 5;
    let mut transaction = AllocationBudget::new(expected);
    let before = ALLOCATED.load(Ordering::Relaxed);
    let decoded = QueryImageV2::decode_with_allocation_budget(&bytes, &ValidationLimits::default(), &mut transaction).unwrap();
    let allocated = ALLOCATED.load(Ordering::Relaxed) - before;

    assert_eq!(decoded.supported_roots.len(), count);
    assert_eq!(transaction.requested(), expected);
    assert_eq!(allocated, expected, "ledger must debit the allocator request, not encoded bytes");
}

#[test]
fn query_decode_enforces_measured_transaction_and_section_allocation_bounds() {
    // Keep all counting-allocator observations in one harness test so Cargo's
    // parallel test workers cannot contaminate operation-local deltas.
    query_decode_rejects_before_vector_allocation_when_transaction_is_too_small();
    query_decode_enforces_section_limit_independently_of_transaction_budget();
    query_decode_debit_matches_allocator_requested_capacity();
}
