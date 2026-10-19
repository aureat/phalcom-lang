//! Formal control-flow graph representation for callable bodies (Spec 04.5).

use crate::identity::{FlowEdgeId, FlowNodeId, PredicateId};
use phalcom_ast::ast::{BlockExpr, Expr, MethodCallExpr, PackItem, Statement};
use phalcom_common::range::SourceRange;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowNodeKind {
    Entry,
    Exit,
    Statement(usize),
    BranchCondition,
    LoopHeader,
    Join,
    Throw,
    Unreachable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlowEdgeKind {
    Normal,
    TrueBranch,
    FalseBranch,
    BackEdge,
    Break,
    Continue,
    Return,
    Throw,
    Unreachable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowNode {
    pub id: FlowNodeId,
    pub kind: FlowNodeKind,
    pub range: SourceRange,
    pub predecessors: Vec<FlowEdgeId>,
    pub successors: Vec<FlowEdgeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowEdge {
    pub id: FlowEdgeId,
    pub source: FlowNodeId,
    pub target: FlowNodeId,
    pub kind: FlowEdgeKind,
    pub predicate: Option<PredicateId>,
}

#[derive(Clone, Debug, Default)]
struct SourceFlowGraph {
    nodes: BTreeMap<FlowNodeId, FlowNode>,
    edges: BTreeMap<FlowEdgeId, FlowEdge>,
    entry: Option<FlowNodeId>,
    exits: Vec<FlowNodeId>,
    next_node_id: u32,
    next_edge_id: u32,
}

#[derive(Debug)]
struct FrozenFlowGraphData {
    nodes: BudgetedSortedMap<FlowNodeId, FlowNode>,
    edges: BudgetedSortedMap<FlowEdgeId, FlowEdge>,
    entry: Option<FlowNodeId>,
    exits: Vec<FlowNodeId>,
}

#[derive(Clone, Debug)]
enum FlowGraphStorage {
    Source(SourceFlowGraph),
    Frozen(Arc<FrozenFlowGraphData>),
}

/// Error returned when mutation is attempted on an imported frozen graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("an imported frozen flow graph is immutable")]
pub struct FrozenFlowGraphMutationError;

enum FlowGraphMapIter<'a, K, V> {
    Source(std::collections::btree_map::Iter<'a, K, V>),
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for FlowGraphMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(key, value)| (key, value)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Source(iter) => iter.size_hint(),
            Self::Frozen(iter) => iter.size_hint(),
        }
    }
}

impl<K, V> ExactSizeIterator for FlowGraphMapIter<'_, K, V> {}

struct FlowGraphMapValues<'a, K, V>(FlowGraphMapIter<'a, K, V>);

impl<'a, K, V> Iterator for FlowGraphMapValues<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> ExactSizeIterator for FlowGraphMapValues<'_, K, V> {}

struct FlowGraphMapKeys<'a, K, V>(FlowGraphMapIter<'a, K, V>);

impl<'a, K, V> Iterator for FlowGraphMapKeys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> ExactSizeIterator for FlowGraphMapKeys<'_, K, V> {}

/// Control-flow graph within a callable body. Imported graphs use immutable,
/// explicitly budgeted ordered buffers; source graphs retain mutable maps.
#[derive(Clone, Debug)]
pub struct FlowGraph {
    storage: FlowGraphStorage,
}

impl Default for FlowGraph {
    fn default() -> Self {
        Self {
            storage: FlowGraphStorage::Source(SourceFlowGraph::default()),
        }
    }
}

impl FlowGraph {
    /// Creates an empty mutable source graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstructs a frozen graph from maps whose backing buffers were charged
    /// before construction. The graph and its shared frozen backing are charged
    /// before their Arc allocations are requested.
    pub(crate) fn from_budgeted(
        allocations: &mut AllocationBudget,
        nodes: BudgetedSortedMap<FlowNodeId, FlowNode>,
        edges: BudgetedSortedMap<FlowEdgeId, FlowEdge>,
        entry: Option<FlowNodeId>,
        exits: Vec<FlowNodeId>,
    ) -> Result<Arc<Self>, AllocationError> {
        let frozen = allocations.arc(FrozenFlowGraphData { nodes, edges, entry, exits })?;
        allocations.arc(Self {
            storage: FlowGraphStorage::Frozen(frozen),
        })
    }

    /// Returns a node by its stable graph-local identifier.
    pub fn node(&self, id: &FlowNodeId) -> Option<&FlowNode> {
        match &self.storage {
            FlowGraphStorage::Source(graph) => graph.nodes.get(id),
            FlowGraphStorage::Frozen(graph) => graph.nodes.get(id),
        }
    }

    /// Returns an edge by its stable graph-local identifier.
    pub fn edge(&self, id: &FlowEdgeId) -> Option<&FlowEdge> {
        match &self.storage {
            FlowGraphStorage::Source(graph) => graph.edges.get(id),
            FlowGraphStorage::Frozen(graph) => graph.edges.get(id),
        }
    }

    /// Iterates over nodes in ascending graph-local identifier order.
    pub fn node_entries(&self) -> impl ExactSizeIterator<Item = (&FlowNodeId, &FlowNode)> + '_ {
        self.node_map_iter()
    }

    fn node_map_iter(&self) -> FlowGraphMapIter<'_, FlowNodeId, FlowNode> {
        match &self.storage {
            FlowGraphStorage::Source(graph) => FlowGraphMapIter::Source(graph.nodes.iter()),
            FlowGraphStorage::Frozen(graph) => FlowGraphMapIter::Frozen(graph.nodes.as_entries().iter()),
        }
    }

    /// Iterates over edges in ascending graph-local identifier order.
    pub fn edge_entries(&self) -> impl ExactSizeIterator<Item = (&FlowEdgeId, &FlowEdge)> + '_ {
        self.edge_map_iter()
    }

    fn edge_map_iter(&self) -> FlowGraphMapIter<'_, FlowEdgeId, FlowEdge> {
        match &self.storage {
            FlowGraphStorage::Source(graph) => FlowGraphMapIter::Source(graph.edges.iter()),
            FlowGraphStorage::Frozen(graph) => FlowGraphMapIter::Frozen(graph.edges.as_entries().iter()),
        }
    }

    /// Iterates over nodes in ascending graph-local identifier order.
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &FlowNode> + '_ {
        FlowGraphMapValues(self.node_map_iter())
    }

    /// Iterates over edges in ascending graph-local identifier order.
    pub fn edges(&self) -> impl ExactSizeIterator<Item = &FlowEdge> + '_ {
        FlowGraphMapValues(self.edge_map_iter())
    }

    /// Iterates over node identifiers in ascending order.
    pub fn node_ids(&self) -> impl ExactSizeIterator<Item = &FlowNodeId> + '_ {
        FlowGraphMapKeys(self.node_map_iter())
    }

    /// Iterates over edge identifiers in ascending order.
    pub fn edge_ids(&self) -> impl ExactSizeIterator<Item = &FlowEdgeId> + '_ {
        FlowGraphMapKeys(self.edge_map_iter())
    }

    /// Returns the number of graph nodes.
    pub fn node_count(&self) -> usize {
        match &self.storage {
            FlowGraphStorage::Source(graph) => graph.nodes.len(),
            FlowGraphStorage::Frozen(graph) => graph.nodes.len(),
        }
    }

    /// Returns the number of graph edges.
    pub fn edge_count(&self) -> usize {
        match &self.storage {
            FlowGraphStorage::Source(graph) => graph.edges.len(),
            FlowGraphStorage::Frozen(graph) => graph.edges.len(),
        }
    }

    /// Returns the graph entry node, if one is set.
    pub fn entry(&self) -> Option<FlowNodeId> {
        match &self.storage {
            FlowGraphStorage::Source(graph) => graph.entry,
            FlowGraphStorage::Frozen(graph) => graph.entry,
        }
    }

    /// Returns graph exits in semantic source order.
    pub fn exits(&self) -> &[FlowNodeId] {
        match &self.storage {
            FlowGraphStorage::Source(graph) => &graph.exits,
            FlowGraphStorage::Frozen(graph) => &graph.exits,
        }
    }

    /// Adds a node to a mutable source graph.
    pub fn try_add_node(&mut self, kind: FlowNodeKind, range: SourceRange) -> Result<FlowNodeId, FrozenFlowGraphMutationError> {
        let FlowGraphStorage::Source(graph) = &mut self.storage else {
            return Err(FrozenFlowGraphMutationError);
        };
        let id = FlowNodeId(graph.next_node_id);
        graph.next_node_id += 1;
        graph.nodes.insert(
            id,
            FlowNode {
                id,
                kind,
                range,
                predecessors: Vec::new(),
                successors: Vec::new(),
            },
        );
        Ok(id)
    }

    /// Adds an edge to a mutable source graph.
    pub fn try_add_edge(&mut self, source: FlowNodeId, target: FlowNodeId, predicate: Option<PredicateId>) -> Result<FlowEdgeId, FrozenFlowGraphMutationError> {
        self.add_edge_with_kind(source, target, FlowEdgeKind::Normal, predicate)
    }

    /// Adds a node, returning an error if this graph is frozen.
    pub fn add_node(&mut self, kind: FlowNodeKind, range: SourceRange) -> Result<FlowNodeId, FrozenFlowGraphMutationError> {
        self.try_add_node(kind, range)
    }

    /// Adds a normal edge, returning an error if this graph is frozen.
    pub fn add_edge(&mut self, source: FlowNodeId, target: FlowNodeId, predicate: Option<PredicateId>) -> Result<FlowEdgeId, FrozenFlowGraphMutationError> {
        self.try_add_edge(source, target, predicate)
    }

    /// Adds a typed edge, returning an error if this graph is frozen.
    pub fn add_edge_with_kind(
        &mut self,
        source: FlowNodeId,
        target: FlowNodeId,
        kind: FlowEdgeKind,
        predicate: Option<PredicateId>,
    ) -> Result<FlowEdgeId, FrozenFlowGraphMutationError> {
        let FlowGraphStorage::Source(graph) = &mut self.storage else {
            return Err(FrozenFlowGraphMutationError);
        };
        let id = FlowEdgeId(graph.next_edge_id);
        graph.next_edge_id += 1;
        graph.edges.insert(
            id,
            FlowEdge {
                id,
                source,
                target,
                kind,
                predicate,
            },
        );
        if let Some(s) = graph.nodes.get_mut(&source) {
            s.successors.push(id);
        }
        if let Some(t) = graph.nodes.get_mut(&target) {
            t.predecessors.push(id);
        }
        Ok(id)
    }

    /// Returns mutable access to a source node, or an error for a frozen graph.
    pub fn try_node_mut(&mut self, id: &FlowNodeId) -> Result<Option<&mut FlowNode>, FrozenFlowGraphMutationError> {
        let FlowGraphStorage::Source(graph) = &mut self.storage else {
            return Err(FrozenFlowGraphMutationError);
        };
        Ok(graph.nodes.get_mut(id))
    }

    /// Sets the entry node on a mutable source graph.
    pub fn try_set_entry(&mut self, entry: Option<FlowNodeId>) -> Result<(), FrozenFlowGraphMutationError> {
        let FlowGraphStorage::Source(graph) = &mut self.storage else {
            return Err(FrozenFlowGraphMutationError);
        };
        graph.entry = entry;
        Ok(())
    }

    /// Appends an exit in semantic source order on a mutable source graph.
    pub fn try_push_exit(&mut self, exit: FlowNodeId) -> Result<(), FrozenFlowGraphMutationError> {
        let FlowGraphStorage::Source(graph) = &mut self.storage else {
            return Err(FrozenFlowGraphMutationError);
        };
        graph.exits.push(exit);
        Ok(())
    }

    fn add_source_node(&mut self, kind: FlowNodeKind, range: SourceRange) -> FlowNodeId {
        self.try_add_node(kind, range).expect("source CFG construction uses mutable graph storage")
    }

    fn add_source_edge(&mut self, source: FlowNodeId, target: FlowNodeId, kind: FlowEdgeKind, predicate: Option<PredicateId>) {
        self.add_edge_with_kind(source, target, kind, predicate)
            .expect("source CFG construction uses mutable graph storage");
    }

    fn set_source_entry(&mut self, entry: Option<FlowNodeId>) {
        self.try_set_entry(entry).expect("source CFG construction uses mutable graph storage");
    }

    fn push_source_exit(&mut self, exit: FlowNodeId) {
        self.try_push_exit(exit).expect("source CFG construction uses mutable graph storage");
    }

    /// Checks if this graph contains cycle candidates or loop back-edges.
    pub fn is_acyclic(&self) -> bool {
        !self.nodes().any(|n| matches!(n.kind, FlowNodeKind::LoopHeader)) && !self.edges().any(|e| matches!(e.kind, FlowEdgeKind::BackEdge))
    }

    /// Returns the set of reachable nodes from the entry node.
    pub fn reachable_nodes(&self) -> BTreeSet<FlowNodeId> {
        let mut reachable = BTreeSet::new();
        let Some(entry_id) = self.entry() else {
            return reachable;
        };

        let mut worklist = vec![entry_id];
        reachable.insert(entry_id);

        while let Some(curr) = worklist.pop() {
            if let Some(node) = self.node(&curr) {
                for edge_id in &node.successors {
                    if let Some(edge) = self.edge(edge_id) {
                        if reachable.insert(edge.target) {
                            worklist.push(edge.target);
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Builds a canonical control-flow graph from a callable's body statements.
    pub fn from_statements(statements: &[Statement]) -> Self {
        let mut graph = FlowGraph::new();
        let r0 = SourceRange { start: 0, end: 0 };
        let entry = graph.add_source_node(FlowNodeKind::Entry, r0);
        graph.set_source_entry(Some(entry));

        let mut builder = CfgBuilder { graph, loop_stack: Vec::new() };

        let active_tails = builder.build_statements(statements, vec![entry]);

        if !active_tails.is_empty() {
            let exit_node = builder.graph.add_source_node(FlowNodeKind::Exit, r0);
            for tail in active_tails {
                builder.graph.add_source_edge(tail, exit_node, FlowEdgeKind::Return, None);
            }
            builder.graph.push_source_exit(exit_node);
        }

        builder.graph
    }
}

struct LoopTarget {
    header: FlowNodeId,
    exit_join: FlowNodeId,
}

struct CfgBuilder {
    graph: FlowGraph,
    loop_stack: Vec<LoopTarget>,
}

impl CfgBuilder {
    #[allow(deprecated)]
    fn build_statements(&mut self, statements: &[Statement], mut incoming: Vec<FlowNodeId>) -> Vec<FlowNodeId> {
        for (idx, stmt) in statements.iter().enumerate() {
            if incoming.is_empty() {
                // Statements following unreachable paths
                let unreach_node = self.graph.add_source_node(FlowNodeKind::Unreachable, stmt_range(stmt));
                incoming = vec![unreach_node];
                continue;
            }

            match stmt {
                Statement::Let(binding) => {
                    let node = self.graph.add_source_node(FlowNodeKind::Statement(idx), binding.range);
                    for prev in incoming {
                        self.graph.add_source_edge(prev, node, FlowEdgeKind::Normal, None);
                    }
                    incoming = vec![node];
                }
                Statement::Return(ret) => {
                    let ret_node = self.graph.add_source_node(FlowNodeKind::Exit, ret.range);
                    for prev in incoming {
                        self.graph.add_source_edge(prev, ret_node, FlowEdgeKind::Return, None);
                    }
                    self.graph.push_source_exit(ret_node);
                    incoming = Vec::new();
                }
                Statement::Throw { expr: _, range } => {
                    let throw_node = self.graph.add_source_node(FlowNodeKind::Throw, *range);
                    for prev in incoming {
                        self.graph.add_source_edge(prev, throw_node, FlowEdgeKind::Throw, None);
                    }
                    self.graph.push_source_exit(throw_node);
                    incoming = Vec::new();
                }
                Statement::Break { range } => {
                    if let Some(target) = self.loop_stack.last() {
                        let exit_join = target.exit_join;
                        for prev in incoming {
                            self.graph.add_source_edge(prev, exit_join, FlowEdgeKind::Break, None);
                        }
                    } else {
                        let err_node = self.graph.add_source_node(FlowNodeKind::Exit, *range);
                        for prev in incoming {
                            self.graph.add_source_edge(prev, err_node, FlowEdgeKind::Break, None);
                        }
                    }
                    incoming = Vec::new();
                }
                Statement::Continue { range } => {
                    if let Some(target) = self.loop_stack.last() {
                        let header = target.header;
                        for prev in incoming {
                            self.graph.add_source_edge(prev, header, FlowEdgeKind::Continue, None);
                        }
                    } else {
                        let err_node = self.graph.add_source_node(FlowNodeKind::Exit, *range);
                        for prev in incoming {
                            self.graph.add_source_edge(prev, err_node, FlowEdgeKind::Continue, None);
                        }
                    }
                    incoming = Vec::new();
                }
                Statement::For(for_stmt) => {
                    let header_node = self.graph.add_source_node(FlowNodeKind::LoopHeader, for_stmt.range);
                    for prev in incoming {
                        self.graph.add_source_edge(prev, header_node, FlowEdgeKind::Normal, None);
                    }

                    let loop_exit_join = self.graph.add_source_node(FlowNodeKind::Join, for_stmt.range);
                    self.loop_stack.push(LoopTarget {
                        header: header_node,
                        exit_join: loop_exit_join,
                    });

                    // Body statements
                    let body_tails = self.build_statements(&for_stmt.body, vec![header_node]);
                    for tail in body_tails {
                        self.graph.add_source_edge(tail, header_node, FlowEdgeKind::BackEdge, None);
                    }

                    // Loop exit false-branch
                    self.graph.add_source_edge(header_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);

                    self.loop_stack.pop();
                    incoming = vec![loop_exit_join];
                }
                Statement::Expr { expr, range } => {
                    incoming = self.build_expression_flow(expr, *range, idx, incoming);
                }
                #[allow(deprecated)]
                Statement::Class(_)
                | Statement::Data(_)
                | Statement::Trait(_)
                | Statement::Impl(_)
                | Statement::TypeAlias(_)
                | Statement::TypeDefinition(_)
                | Statement::Export(_) => {
                    let node = self.graph.add_source_node(FlowNodeKind::Statement(idx), stmt_range(stmt));
                    for prev in incoming {
                        self.graph.add_source_edge(prev, node, FlowEdgeKind::Normal, None);
                    }
                    incoming = vec![node];
                }
            }
        }
        incoming
    }

    fn build_expression_flow(&mut self, expr: &Expr, expr_range: SourceRange, stmt_idx: usize, incoming: Vec<FlowNodeId>) -> Vec<FlowNodeId> {
        match expr {
            Expr::MethodCall(call) => {
                if let Some(res) = self.build_method_call_flow(call, incoming.clone()) {
                    return res;
                }
            }
            Expr::IfLet(if_let) => {
                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, if_let.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, cond_node, FlowEdgeKind::Normal, None);
                }
                let then_tails = self.build_statements(&if_let.then_body.body, vec![cond_node]);
                let else_tails = if let Some(ref else_body) = if_let.else_body {
                    self.build_statements(&else_body.body, vec![cond_node])
                } else {
                    vec![cond_node]
                };

                let mut all_tails = Vec::new();
                all_tails.extend(then_tails);
                all_tails.extend(else_tails);

                if !all_tails.is_empty() {
                    let join_node = self.graph.add_source_node(FlowNodeKind::Join, if_let.range);
                    for tail in all_tails {
                        self.graph.add_source_edge(tail, join_node, FlowEdgeKind::Normal, None);
                    }
                    return vec![join_node];
                }
                return Vec::new();
            }
            Expr::If(if_expr) => {
                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, if_expr.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, cond_node, FlowEdgeKind::Normal, None);
                }
                let then_tails = self.build_statements(&if_expr.then_body.body, vec![cond_node]);
                let else_tails = match &if_expr.else_body {
                    Some(phalcom_ast::ast::IfElseBody::Block(block)) => self.build_statements(&block.body, vec![cond_node]),
                    Some(phalcom_ast::ast::IfElseBody::ElseIf(expr)) => self.build_expression_flow(expr, expr.range(), stmt_idx, vec![cond_node]),
                    None => vec![cond_node],
                };
                let mut all_tails = then_tails;
                all_tails.extend(else_tails);
                if all_tails.is_empty() {
                    return Vec::new();
                }
                let join_node = self.graph.add_source_node(FlowNodeKind::Join, if_expr.range);
                for tail in all_tails {
                    self.graph.add_source_edge(tail, join_node, FlowEdgeKind::Normal, None);
                }
                return vec![join_node];
            }
            Expr::While(while_expr) => {
                let header_node = self.graph.add_source_node(FlowNodeKind::LoopHeader, while_expr.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, header_node, FlowEdgeKind::Normal, None);
                }

                let condition_tails = self.build_expression_flow(&while_expr.condition, while_expr.condition.range(), stmt_idx, vec![header_node]);
                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, while_expr.condition.range());
                for tail in condition_tails {
                    self.graph.add_source_edge(tail, cond_node, FlowEdgeKind::Normal, None);
                }

                let loop_exit_join = self.graph.add_source_node(FlowNodeKind::Join, while_expr.range);
                self.loop_stack.push(LoopTarget {
                    header: header_node,
                    exit_join: loop_exit_join,
                });

                let body_tails = self.build_statements(&while_expr.body.body, vec![cond_node]);
                for tail in body_tails {
                    self.graph.add_source_edge(tail, header_node, FlowEdgeKind::BackEdge, None);
                }

                self.graph.add_source_edge(cond_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);
                self.loop_stack.pop();
                return vec![loop_exit_join];
            }
            Expr::WhileLet(while_let) => {
                let header_node = self.graph.add_source_node(FlowNodeKind::LoopHeader, while_let.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, header_node, FlowEdgeKind::Normal, None);
                }

                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, while_let.range);
                self.graph.add_source_edge(header_node, cond_node, FlowEdgeKind::Normal, None);

                let loop_exit_join = self.graph.add_source_node(FlowNodeKind::Join, while_let.range);
                self.loop_stack.push(LoopTarget {
                    header: header_node,
                    exit_join: loop_exit_join,
                });

                let body_tails = self.build_statements(&while_let.body, vec![cond_node]);
                for tail in body_tails {
                    self.graph.add_source_edge(tail, header_node, FlowEdgeKind::BackEdge, None);
                }

                self.graph.add_source_edge(cond_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);
                self.loop_stack.pop();
                return vec![loop_exit_join];
            }
            _ => {}
        }

        // Ordinary expression
        let node = self.graph.add_source_node(FlowNodeKind::Statement(stmt_idx), expr_range);
        for prev in incoming {
            self.graph.add_source_edge(prev, node, FlowEdgeKind::Normal, None);
        }
        vec![node]
    }

    fn build_method_call_flow(&mut self, call: &MethodCallExpr, incoming: Vec<FlowNodeId>) -> Option<Vec<FlowNodeId>> {
        match call.method.as_str() {
            "ifTrue:ifFalse:" | "ifTrue" if has_labeled_arg(&call.args, "ifFalse") => {
                let then_block = positional_block(&call.args, 0)?;
                let else_block = labeled_block(&call.args, "ifFalse")?;
                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, call.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, cond_node, FlowEdgeKind::Normal, None);
                }

                let then_tails = self.build_statements(&then_block.body, vec![cond_node]);
                let else_tails = self.build_statements(&else_block.body, vec![cond_node]);

                let mut all_tails = Vec::new();
                all_tails.extend(then_tails);
                all_tails.extend(else_tails);

                if !all_tails.is_empty() {
                    let join_node = self.graph.add_source_node(FlowNodeKind::Join, call.range);
                    for tail in all_tails {
                        self.graph.add_source_edge(tail, join_node, FlowEdgeKind::Normal, None);
                    }
                    Some(vec![join_node])
                } else {
                    Some(Vec::new())
                }
            }
            "ifTrue" => {
                let then_block = positional_block(&call.args, 0)?;
                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, call.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, cond_node, FlowEdgeKind::Normal, None);
                }

                let then_tails = self.build_statements(&then_block.body, vec![cond_node]);
                let join_node = self.graph.add_source_node(FlowNodeKind::Join, call.range);

                for tail in then_tails {
                    self.graph.add_source_edge(tail, join_node, FlowEdgeKind::Normal, None);
                }
                // False branch directly to join
                self.graph.add_source_edge(cond_node, join_node, FlowEdgeKind::FalseBranch, None);

                Some(vec![join_node])
            }
            "ifFalse" => {
                let else_block = positional_block(&call.args, 0)?;
                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, call.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, cond_node, FlowEdgeKind::Normal, None);
                }

                let else_tails = self.build_statements(&else_block.body, vec![cond_node]);
                let join_node = self.graph.add_source_node(FlowNodeKind::Join, call.range);

                for tail in else_tails {
                    self.graph.add_source_edge(tail, join_node, FlowEdgeKind::Normal, None);
                }
                // True branch directly to join
                self.graph.add_source_edge(cond_node, join_node, FlowEdgeKind::TrueBranch, None);

                Some(vec![join_node])
            }
            "whileTrue" => {
                let header_node = self.graph.add_source_node(FlowNodeKind::LoopHeader, call.range);
                for prev in incoming {
                    self.graph.add_source_edge(prev, header_node, FlowEdgeKind::Normal, None);
                }

                let cond_node = self.graph.add_source_node(FlowNodeKind::BranchCondition, call.range);
                self.graph.add_source_edge(header_node, cond_node, FlowEdgeKind::Normal, None);

                let loop_exit_join = self.graph.add_source_node(FlowNodeKind::Join, call.range);
                self.loop_stack.push(LoopTarget {
                    header: header_node,
                    exit_join: loop_exit_join,
                });

                if let Some(body_block) = positional_block(&call.args, 0) {
                    let body_tails = self.build_statements(&body_block.body, vec![cond_node]);
                    for tail in body_tails {
                        self.graph.add_source_edge(tail, header_node, FlowEdgeKind::BackEdge, None);
                    }
                }

                self.graph.add_source_edge(cond_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);
                self.loop_stack.pop();
                Some(vec![loop_exit_join])
            }
            _ => None,
        }
    }
}

fn positional_block(args: &[PackItem], index: usize) -> Option<&BlockExpr> {
    match args.get(index)? {
        PackItem::Positional { expr: Expr::Block(block), .. } => Some(block),
        _ => None,
    }
}

fn labeled_block<'a>(args: &'a [PackItem], label_name: &str) -> Option<&'a BlockExpr> {
    for arg in args {
        if let PackItem::Labeled {
            label: phalcom_ast::ast::PackLabel::Static { text, .. },
            value: Expr::Block(block),
            ..
        } = arg
            && text == label_name
        {
            return Some(block);
        }
    }
    None
}

fn has_labeled_arg(args: &[PackItem], label_name: &str) -> bool {
    args.iter().any(|arg| {
        if let PackItem::Labeled {
            label: phalcom_ast::ast::PackLabel::Static { text, .. },
            ..
        } = arg
        {
            return text == label_name;
        }
        false
    })
}

#[allow(deprecated)]
fn stmt_range(stmt: &Statement) -> SourceRange {
    match stmt {
        Statement::Class(c) => c.range,
        Statement::Data(d) => d.range,
        Statement::Trait(t) => t.range,
        Statement::TypeDefinition(t) => t.range,
        Statement::TypeAlias(t) => t.range,
        Statement::Let(l) => l.range,
        Statement::Return(r) => r.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(f) => f.range,
        Statement::Break { range } => *range,
        Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Export(e) => e.range,
        Statement::Impl(i) => i.range,
    }
}

#[cfg(test)]
mod frozen_storage_tests {
    use super::*;

    #[test]
    fn frozen_clones_share_storage_and_reject_mutation_while_source_clones_are_isolated() {
        let mut allocations = AllocationBudget::new(4096);
        let nodes = BudgetedSortedMap::from_sorted_entries(Vec::new()).expect("empty node map is ordered");
        let edges = BudgetedSortedMap::from_sorted_entries(Vec::new()).expect("empty edge map is ordered");
        let frozen = FlowGraph::from_budgeted(&mut allocations, nodes, edges, None, Vec::new()).expect("frozen graph allocation");
        let mut frozen_clone = frozen.as_ref().clone();
        let (FlowGraphStorage::Frozen(left), FlowGraphStorage::Frozen(right)) = (&frozen.storage, &frozen_clone.storage) else {
            panic!("imported graph must retain frozen storage after clone");
        };
        assert!(Arc::ptr_eq(left, right));
        assert_eq!(
            frozen_clone.add_node(FlowNodeKind::Entry, SourceRange::default()),
            Err(FrozenFlowGraphMutationError)
        );
        assert_eq!(frozen_clone.try_set_entry(Some(FlowNodeId(0))), Err(FrozenFlowGraphMutationError));
        assert_eq!(frozen_clone.try_push_exit(FlowNodeId(0)), Err(FrozenFlowGraphMutationError));

        let mut source = FlowGraph::new();
        source.add_node(FlowNodeKind::Entry, SourceRange::default()).expect("source graph is mutable");
        let mut source_clone = source.clone();
        source_clone
            .add_node(FlowNodeKind::Exit, SourceRange::default())
            .expect("source clone is mutable");
        assert_eq!(source.node_count(), 1);
        assert_eq!(source_clone.node_count(), 2);
    }
}
