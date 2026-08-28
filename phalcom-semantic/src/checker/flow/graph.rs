//! Formal control-flow graph representation for callable bodies (Spec 04.5).

use crate::identity::{FlowEdgeId, FlowNodeId, PredicateId};
use phalcom_ast::ast::{BlockExpr, Expr, MethodCallExpr, PackItem, Statement};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

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

/// Control-flow graph within a callable body.
#[derive(Clone, Debug, Default)]
pub struct FlowGraph {
    pub nodes: BTreeMap<FlowNodeId, FlowNode>,
    pub edges: BTreeMap<FlowEdgeId, FlowEdge>,
    pub entry: Option<FlowNodeId>,
    pub exits: Vec<FlowNodeId>,
    next_node_id: u32,
    next_edge_id: u32,
}

impl FlowGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: FlowNodeKind, range: SourceRange) -> FlowNodeId {
        let id = FlowNodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.insert(
            id,
            FlowNode {
                id,
                kind,
                range,
                predecessors: Vec::new(),
                successors: Vec::new(),
            },
        );
        id
    }

    pub fn add_edge(&mut self, source: FlowNodeId, target: FlowNodeId, predicate: Option<PredicateId>) -> FlowEdgeId {
        self.add_edge_with_kind(source, target, FlowEdgeKind::Normal, predicate)
    }

    pub fn add_edge_with_kind(&mut self, source: FlowNodeId, target: FlowNodeId, kind: FlowEdgeKind, predicate: Option<PredicateId>) -> FlowEdgeId {
        let id = FlowEdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        self.edges.insert(
            id,
            FlowEdge {
                id,
                source,
                target,
                kind,
                predicate,
            },
        );
        if let Some(s) = self.nodes.get_mut(&source) {
            s.successors.push(id);
        }
        if let Some(t) = self.nodes.get_mut(&target) {
            t.predecessors.push(id);
        }
        id
    }

    /// Checks if this graph contains cycle candidates or loop back-edges.
    pub fn is_acyclic(&self) -> bool {
        !self.nodes.values().any(|n| matches!(n.kind, FlowNodeKind::LoopHeader)) && !self.edges.values().any(|e| matches!(e.kind, FlowEdgeKind::BackEdge))
    }

    /// Returns the set of reachable nodes from the entry node.
    pub fn reachable_nodes(&self) -> BTreeSet<FlowNodeId> {
        let mut reachable = BTreeSet::new();
        let Some(entry_id) = self.entry else {
            return reachable;
        };

        let mut worklist = vec![entry_id];
        reachable.insert(entry_id);

        while let Some(curr) = worklist.pop() {
            if let Some(node) = self.nodes.get(&curr) {
                for edge_id in &node.successors {
                    if let Some(edge) = self.edges.get(edge_id) {
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
        let entry = graph.add_node(FlowNodeKind::Entry, r0);
        graph.entry = Some(entry);

        let mut builder = CfgBuilder { graph, loop_stack: Vec::new() };

        let active_tails = builder.build_statements(statements, vec![entry]);

        if !active_tails.is_empty() {
            let exit_node = builder.graph.add_node(FlowNodeKind::Exit, r0);
            for tail in active_tails {
                builder.graph.add_edge_with_kind(tail, exit_node, FlowEdgeKind::Return, None);
            }
            builder.graph.exits.push(exit_node);
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
    fn build_statements(&mut self, statements: &[Statement], mut incoming: Vec<FlowNodeId>) -> Vec<FlowNodeId> {
        for (idx, stmt) in statements.iter().enumerate() {
            if incoming.is_empty() {
                // Statements following unreachable paths
                let unreach_node = self.graph.add_node(FlowNodeKind::Unreachable, stmt_range(stmt));
                incoming = vec![unreach_node];
                continue;
            }

            match stmt {
                Statement::Let(binding) => {
                    let node = self.graph.add_node(FlowNodeKind::Statement(idx), binding.range);
                    for prev in incoming {
                        self.graph.add_edge_with_kind(prev, node, FlowEdgeKind::Normal, None);
                    }
                    incoming = vec![node];
                }
                Statement::Return(ret) => {
                    let ret_node = self.graph.add_node(FlowNodeKind::Exit, ret.range);
                    for prev in incoming {
                        self.graph.add_edge_with_kind(prev, ret_node, FlowEdgeKind::Return, None);
                    }
                    self.graph.exits.push(ret_node);
                    incoming = Vec::new();
                }
                Statement::Throw { expr: _, range } => {
                    let throw_node = self.graph.add_node(FlowNodeKind::Throw, *range);
                    for prev in incoming {
                        self.graph.add_edge_with_kind(prev, throw_node, FlowEdgeKind::Throw, None);
                    }
                    self.graph.exits.push(throw_node);
                    incoming = Vec::new();
                }
                Statement::Break { range } => {
                    if let Some(target) = self.loop_stack.last() {
                        let exit_join = target.exit_join;
                        for prev in incoming {
                            self.graph.add_edge_with_kind(prev, exit_join, FlowEdgeKind::Break, None);
                        }
                    } else {
                        let err_node = self.graph.add_node(FlowNodeKind::Exit, *range);
                        for prev in incoming {
                            self.graph.add_edge_with_kind(prev, err_node, FlowEdgeKind::Break, None);
                        }
                    }
                    incoming = Vec::new();
                }
                Statement::Continue { range } => {
                    if let Some(target) = self.loop_stack.last() {
                        let header = target.header;
                        for prev in incoming {
                            self.graph.add_edge_with_kind(prev, header, FlowEdgeKind::Continue, None);
                        }
                    } else {
                        let err_node = self.graph.add_node(FlowNodeKind::Exit, *range);
                        for prev in incoming {
                            self.graph.add_edge_with_kind(prev, err_node, FlowEdgeKind::Continue, None);
                        }
                    }
                    incoming = Vec::new();
                }
                Statement::For(for_stmt) => {
                    let header_node = self.graph.add_node(FlowNodeKind::LoopHeader, for_stmt.range);
                    for prev in incoming {
                        self.graph.add_edge_with_kind(prev, header_node, FlowEdgeKind::Normal, None);
                    }

                    let loop_exit_join = self.graph.add_node(FlowNodeKind::Join, for_stmt.range);
                    self.loop_stack.push(LoopTarget {
                        header: header_node,
                        exit_join: loop_exit_join,
                    });

                    // Body statements
                    let body_tails = self.build_statements(&for_stmt.body, vec![header_node]);
                    for tail in body_tails {
                        self.graph.add_edge_with_kind(tail, header_node, FlowEdgeKind::BackEdge, None);
                    }

                    // Loop exit false-branch
                    self.graph.add_edge_with_kind(header_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);

                    self.loop_stack.pop();
                    incoming = vec![loop_exit_join];
                }
                Statement::Expr { expr, range } => {
                    incoming = self.build_expression_flow(expr, *range, idx, incoming);
                }
                Statement::Class(_) | Statement::TypeAlias(_) | Statement::Export(_) => {
                    let node = self.graph.add_node(FlowNodeKind::Statement(idx), stmt_range(stmt));
                    for prev in incoming {
                        self.graph.add_edge_with_kind(prev, node, FlowEdgeKind::Normal, None);
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
                let cond_node = self.graph.add_node(FlowNodeKind::BranchCondition, if_let.range);
                for prev in incoming {
                    self.graph.add_edge_with_kind(prev, cond_node, FlowEdgeKind::Normal, None);
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
                    let join_node = self.graph.add_node(FlowNodeKind::Join, if_let.range);
                    for tail in all_tails {
                        self.graph.add_edge_with_kind(tail, join_node, FlowEdgeKind::Normal, None);
                    }
                    return vec![join_node];
                }
                return Vec::new();
            }
            Expr::WhileLet(while_let) => {
                let header_node = self.graph.add_node(FlowNodeKind::LoopHeader, while_let.range);
                for prev in incoming {
                    self.graph.add_edge_with_kind(prev, header_node, FlowEdgeKind::Normal, None);
                }

                let cond_node = self.graph.add_node(FlowNodeKind::BranchCondition, while_let.range);
                self.graph.add_edge_with_kind(header_node, cond_node, FlowEdgeKind::Normal, None);

                let loop_exit_join = self.graph.add_node(FlowNodeKind::Join, while_let.range);
                self.loop_stack.push(LoopTarget {
                    header: header_node,
                    exit_join: loop_exit_join,
                });

                let body_tails = self.build_statements(&while_let.body, vec![cond_node]);
                for tail in body_tails {
                    self.graph.add_edge_with_kind(tail, header_node, FlowEdgeKind::BackEdge, None);
                }

                self.graph.add_edge_with_kind(cond_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);
                self.loop_stack.pop();
                return vec![loop_exit_join];
            }
            _ => {}
        }

        // Ordinary expression
        let node = self.graph.add_node(FlowNodeKind::Statement(stmt_idx), expr_range);
        for prev in incoming {
            self.graph.add_edge_with_kind(prev, node, FlowEdgeKind::Normal, None);
        }
        vec![node]
    }

    fn build_method_call_flow(&mut self, call: &MethodCallExpr, incoming: Vec<FlowNodeId>) -> Option<Vec<FlowNodeId>> {
        match call.method.as_str() {
            "ifTrue:ifFalse:" | "ifTrue" if has_labeled_arg(&call.args, "ifFalse") => {
                let then_block = positional_block(&call.args, 0)?;
                let else_block = labeled_block(&call.args, "ifFalse")?;
                let cond_node = self.graph.add_node(FlowNodeKind::BranchCondition, call.range);
                for prev in incoming {
                    self.graph.add_edge_with_kind(prev, cond_node, FlowEdgeKind::Normal, None);
                }

                let then_tails = self.build_statements(&then_block.body, vec![cond_node]);
                let else_tails = self.build_statements(&else_block.body, vec![cond_node]);

                let mut all_tails = Vec::new();
                all_tails.extend(then_tails);
                all_tails.extend(else_tails);

                if !all_tails.is_empty() {
                    let join_node = self.graph.add_node(FlowNodeKind::Join, call.range);
                    for tail in all_tails {
                        self.graph.add_edge_with_kind(tail, join_node, FlowEdgeKind::Normal, None);
                    }
                    Some(vec![join_node])
                } else {
                    Some(Vec::new())
                }
            }
            "ifTrue" => {
                let then_block = positional_block(&call.args, 0)?;
                let cond_node = self.graph.add_node(FlowNodeKind::BranchCondition, call.range);
                for prev in incoming {
                    self.graph.add_edge_with_kind(prev, cond_node, FlowEdgeKind::Normal, None);
                }

                let then_tails = self.build_statements(&then_block.body, vec![cond_node]);
                let join_node = self.graph.add_node(FlowNodeKind::Join, call.range);

                for tail in then_tails {
                    self.graph.add_edge_with_kind(tail, join_node, FlowEdgeKind::Normal, None);
                }
                // False branch directly to join
                self.graph.add_edge_with_kind(cond_node, join_node, FlowEdgeKind::FalseBranch, None);

                Some(vec![join_node])
            }
            "ifFalse" => {
                let else_block = positional_block(&call.args, 0)?;
                let cond_node = self.graph.add_node(FlowNodeKind::BranchCondition, call.range);
                for prev in incoming {
                    self.graph.add_edge_with_kind(prev, cond_node, FlowEdgeKind::Normal, None);
                }

                let else_tails = self.build_statements(&else_block.body, vec![cond_node]);
                let join_node = self.graph.add_node(FlowNodeKind::Join, call.range);

                for tail in else_tails {
                    self.graph.add_edge_with_kind(tail, join_node, FlowEdgeKind::Normal, None);
                }
                // True branch directly to join
                self.graph.add_edge_with_kind(cond_node, join_node, FlowEdgeKind::TrueBranch, None);

                Some(vec![join_node])
            }
            "whileTrue" => {
                let header_node = self.graph.add_node(FlowNodeKind::LoopHeader, call.range);
                for prev in incoming {
                    self.graph.add_edge_with_kind(prev, header_node, FlowEdgeKind::Normal, None);
                }

                let cond_node = self.graph.add_node(FlowNodeKind::BranchCondition, call.range);
                self.graph.add_edge_with_kind(header_node, cond_node, FlowEdgeKind::Normal, None);

                let loop_exit_join = self.graph.add_node(FlowNodeKind::Join, call.range);
                self.loop_stack.push(LoopTarget {
                    header: header_node,
                    exit_join: loop_exit_join,
                });

                if let Some(body_block) = positional_block(&call.args, 0) {
                    let body_tails = self.build_statements(&body_block.body, vec![cond_node]);
                    for tail in body_tails {
                        self.graph.add_edge_with_kind(tail, header_node, FlowEdgeKind::BackEdge, None);
                    }
                }

                self.graph.add_edge_with_kind(cond_node, loop_exit_join, FlowEdgeKind::FalseBranch, None);
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

fn stmt_range(stmt: &Statement) -> SourceRange {
    match stmt {
        Statement::Class(c) => c.range,
        Statement::TypeAlias(t) => t.range,
        Statement::Let(l) => l.range,
        Statement::Return(r) => r.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(f) => f.range,
        Statement::Break { range } => *range,
        Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Export(e) => e.range,
    }
}
