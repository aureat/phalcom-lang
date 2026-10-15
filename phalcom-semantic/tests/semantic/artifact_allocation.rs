use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::error::QueryImageError;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::query::{DependencyEvidenceEntryV1, PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryRootEntryV1};
use phalcom_compiler_artifact::semantic::world::{CallableFlowEdgeKindV1, CallableFlowEdgeV1, CallableFlowGraphV1, CallableFlowNodeKindV1, CallableFlowNodeV1};
use phalcom_semantic::artifact::import::import_callable_flow_graph_with_allocation_budget;
use phalcom_semantic::checker::flow::graph::{FlowEdgeKind, FlowGraph, FlowNodeKind};
use phalcom_semantic::artifact::query::validate_query_image_closure_with_allocation_budget;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

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
        // SAFETY: `layout` is passed unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: `pointer` was returned by `System` through this wrapper and
        // `layout` is forwarded unchanged from the corresponding allocation.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_request(new_size);
        // SAFETY: `pointer` and `layout` preserve the system allocation pair;
        // `new_size` is forwarded unchanged to `System`.
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn valid_query_dag(count: usize) -> QueryImageV1 {
    let supported_roots = (0..count)
        .map(|index| QueryRootEntryV1 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::CallableSignature,
                stable_owner_key: format!("callable:{index}"),
            },
            dependency_indices: if index == 0 {
                Vec::new()
            } else {
                vec![(index - 1) as u32]
            },
        })
        .collect();
    QueryImageV1 {
        supported_roots,
        dependency_evidence: Vec::new(),
    }
}

#[test]
fn query_closure_debits_exact_allocator_requests_and_rolls_back_on_exhaustion() {
    let count = 512;
    let image = valid_query_dag(count);
    let limits = ValidationLimits::default();
    let expected = Layout::array::<&PortableQueryKeyV1>(count).unwrap().size()
        + Layout::array::<u8>(count).unwrap().size()
        + Layout::array::<(usize, usize)>(count).unwrap().size();
    let mut transaction = AllocationBudget::new(expected);

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let result = validate_query_image_closure_with_allocation_budget(&image, &limits, &mut transaction);
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    assert!(result.is_ok(), "valid dependency DAG should close: {result:?}");
    assert_eq!(observed, expected, "independent allocator must observe the charged layouts");
    assert_eq!(transaction.requested(), expected);

    let mut insufficient = AllocationBudget::new(expected - 1);
    let result = validate_query_image_closure_with_allocation_budget(&image, &limits, &mut insufficient);
    assert!(matches!(result, Err(ArtifactError::Allocation(AllocationError::Exhausted))));
    assert_eq!(insufficient.requested(), 0, "failed closure must not commit its child ledger");

    let section_limited = ValidationLimits {
        max_decoded_section_bytes: (expected - 1) as u64,
        ..limits
    };
    let mut transaction_with_headroom = AllocationBudget::new(expected * 2);
    let result = validate_query_image_closure_with_allocation_budget(&image, &section_limited, &mut transaction_with_headroom);
    assert!(matches!(result, Err(ArtifactError::Allocation(AllocationError::Exhausted))));
    assert_eq!(transaction_with_headroom.requested(), 0);

    let mut duplicate_root = valid_query_dag(2);
    duplicate_root.supported_roots[1].key = duplicate_root.supported_roots[0].key.clone();
    let mut duplicate_budget = AllocationBudget::new(1024);
    let result = validate_query_image_closure_with_allocation_budget(&duplicate_root, &limits, &mut duplicate_budget);
    assert!(matches!(result, Err(ArtifactError::QueryImage(QueryImageError::DuplicateRoot))));

    let mut cycle = valid_query_dag(2);
    cycle.supported_roots[0].dependency_indices.push(1);
    let mut cycle_budget = AllocationBudget::new(1024);
    let result = validate_query_image_closure_with_allocation_budget(&cycle, &limits, &mut cycle_budget);
    assert!(matches!(result, Err(ArtifactError::QueryImage(QueryImageError::DependencyCycle))));

    let mut self_dependency = valid_query_dag(1);
    self_dependency.supported_roots[0].dependency_indices.push(0);
    let mut self_budget = AllocationBudget::new(1024);
    let result = validate_query_image_closure_with_allocation_budget(&self_dependency, &limits, &mut self_budget);
    assert!(matches!(result, Err(ArtifactError::QueryImage(QueryImageError::InvalidDependencyIndex))));

    let mut missing_dependency = valid_query_dag(1);
    missing_dependency.supported_roots[0].dependency_indices.push(1);
    let mut missing_budget = AllocationBudget::new(1024);
    let result = validate_query_image_closure_with_allocation_budget(&missing_dependency, &limits, &mut missing_budget);
    assert!(matches!(result, Err(ArtifactError::QueryImage(QueryImageError::InvalidDependencyIndex))));

    let dependency_evidence = QueryImageV1 {
        supported_roots: Vec::new(),
        dependency_evidence: vec![DependencyEvidenceEntryV1 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::HierarchyEdge,
                stable_owner_key: "declaration:owner".into(),
            },
            stable_aggregate_owner: "declaration:owner".into(),
        }],
    };
    let mut evidence_budget = AllocationBudget::new(0);
    assert!(validate_query_image_closure_with_allocation_budget(&dependency_evidence, &limits, &mut evidence_budget,).is_ok());
    assert_eq!(evidence_budget.requested(), 0);

    // The high-degree entry node makes adjacency reservations visible to the
    // independent allocator measurement below.
    let graph_node_count = 128usize;
    let flow = CallableFlowGraphV1 {
        nodes: (0..graph_node_count)
            .map(|index| CallableFlowNodeV1 {
                kind: if index == 0 {
                    CallableFlowNodeKindV1::Entry
                } else if index + 1 == graph_node_count {
                    CallableFlowNodeKindV1::Exit
                } else {
                    CallableFlowNodeKindV1::Statement { index: index as u32 }
                },
                range: phalcom_compiler_artifact::module_world::SpanV1 { start: index as u64, end: index as u64 },
            })
            .collect(),
        edges: (1..graph_node_count)
            .map(|target| CallableFlowEdgeV1 {
                source: 0,
                target: target as u32,
                kind: CallableFlowEdgeKindV1::Normal,
            })
            .collect(),
        entry: Some(0),
        exits: vec![(graph_node_count - 1) as u32, 1],
    };
    let mut source_graph = FlowGraph::new();
    for node in &flow.nodes {
        let kind = match &node.kind {
            CallableFlowNodeKindV1::Entry => FlowNodeKind::Entry,
            CallableFlowNodeKindV1::Exit => FlowNodeKind::Exit,
            CallableFlowNodeKindV1::Statement { index } => FlowNodeKind::Statement(*index as usize),
            _ => unreachable!("fixture contains only entry, exit, and statement nodes"),
        };
        source_graph.add_node(kind, phalcom_common::range::SourceRange { start: node.range.start as usize, end: node.range.end as usize }).unwrap();
    }
    source_graph.try_set_entry(Some(phalcom_semantic::identity::FlowNodeId(0))).unwrap();
    source_graph.try_push_exit(phalcom_semantic::identity::FlowNodeId((graph_node_count - 1) as u32)).unwrap();
    source_graph.try_push_exit(phalcom_semantic::identity::FlowNodeId(1)).unwrap();
    for edge in &flow.edges {
        source_graph
            .add_edge_with_kind(
                phalcom_semantic::identity::FlowNodeId(edge.source),
                phalcom_semantic::identity::FlowNodeId(edge.target),
                FlowEdgeKind::Normal,
                None,
            )
            .unwrap();
    }

    let mut graph_budget = AllocationBudget::new(usize::MAX);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let imported = import_callable_flow_graph_with_allocation_budget(&flow, &limits, &mut graph_budget);
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);
    let imported = imported.expect("valid callable flow graph imports");
    let graph_bytes = graph_budget.requested();
    assert_eq!(observed, graph_bytes, "flow graph import must charge every requested buffer and Arc");
    assert_eq!(source_graph.node_entries().collect::<Vec<_>>(), imported.node_entries().collect::<Vec<_>>());
    assert_eq!(source_graph.edge_entries().collect::<Vec<_>>(), imported.edge_entries().collect::<Vec<_>>());
    assert_eq!(source_graph.entry(), imported.entry());
    assert_eq!(source_graph.exits(), imported.exits());

    let mut exhausted_before_first_request = AllocationBudget::new(0);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let exhausted = import_callable_flow_graph_with_allocation_budget(&flow, &limits, &mut exhausted_before_first_request);
    MEASURING.store(false, Ordering::Relaxed);
    let exhausted_error_bytes = REQUESTED_BYTES.load(Ordering::Relaxed);
    assert!(matches!(exhausted, Err(ArtifactError::Allocation(AllocationError::Exhausted))));
    assert_eq!(exhausted_error_bytes, 0, "reporting an exhausted budget must not allocate an error string");
    assert_eq!(exhausted_before_first_request.requested(), 0);

    let mut frozen_clone = Arc::clone(&imported);
    assert!(Arc::make_mut(&mut frozen_clone).add_node(FlowNodeKind::Entry, Default::default()).is_err());
    let mut source_clone = source_graph.clone();
    source_clone.add_node(FlowNodeKind::Entry, Default::default()).unwrap();
    assert_eq!(source_graph.node_count(), graph_node_count);
    assert_eq!(source_clone.node_count(), graph_node_count + 1);

    let mut exact_budget = AllocationBudget::new(graph_bytes);
    assert!(import_callable_flow_graph_with_allocation_budget(&flow, &limits, &mut exact_budget).is_ok());
    assert_eq!(exact_budget.requested(), graph_bytes);
    let mut one_byte_short = AllocationBudget::new(graph_bytes - 1);
    assert!(import_callable_flow_graph_with_allocation_budget(&flow, &limits, &mut one_byte_short).is_err());
    assert_eq!(one_byte_short.requested(), 0, "failed graph import must roll back its child budget");

    let mut malformed_graph = flow.clone();
    malformed_graph.edges[0].target = graph_node_count as u32;
    let mut malformed_budget = AllocationBudget::new(graph_bytes);
    assert!(import_callable_flow_graph_with_allocation_budget(&malformed_graph, &limits, &mut malformed_budget).is_err());
    assert_eq!(malformed_budget.requested(), 0, "malformed graph import must roll back its child budget");
}
