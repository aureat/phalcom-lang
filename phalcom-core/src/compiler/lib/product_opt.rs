//! Representation-Aware Product Optimizer and Scalar Replacement (LANG005.C1.P2).
//!
//! Provides compile-time proof planning, candidate classification, and scalar replacement
//! of non-escaping `data` and unobserved exact `enum` values into `Value` frame slots.

use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::modules::semantic_lowering::{AssociatedLoweringSpec, DataConstructionLoweringSpec, LoweringSiteKind, ModuleLoweringSemantics};
use phalcom_ast::ast::{BindingKind, Expr, MatchExpr, PackItem, Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_semantic::identity::{BindingId, DataConstructorId, VariantId};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Maximum number of flattened scalar leaves allowed for a virtual product.
pub(crate) const MAX_VIRTUAL_PRODUCT_LEAVES: usize = 8;

/// Product optimization mode seam.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ProductOptimizationMode {
    #[default]
    Enabled,
    Disabled,
}

/// Identifies the kind of virtualized product.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VirtualProductKind {
    Data {
        constructor: DataConstructorId,
        construction: Arc<DataConstructionLoweringSpec>,
    },
    Tuple {
        spec: Arc<crate::modules::semantic_lowering::AnonymousProductConstructionLoweringSpec>,
    },
    Record {
        spec: Arc<crate::modules::semantic_lowering::AnonymousProductConstructionLoweringSpec>,
    },
    Variant {
        variant: VariantId,
    },
}

/// A component inside a virtual shape plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VirtualComponentPlan {
    Scalar {
        logical_component: u32,
        leaf_offset: u16,
    },
    NestedProduct {
        logical_component: u32,
        leaf_offset: u16,
        shape: Box<VirtualShapePlan>,
    },
}

/// Virtual shape layout describing how logical components map to scalar leaf slots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VirtualShapePlan {
    pub kind: VirtualProductKind,
    pub components: Box<[VirtualComponentPlan]>,
    pub leaf_count: u16,
}

impl VirtualShapePlan {
    pub fn scalar_offset_for_path(&self, path: &[u32]) -> Option<u16> {
        if path.is_empty() {
            return None;
        }
        let first = path[0];
        for comp in self.components.iter() {
            match comp {
                VirtualComponentPlan::Scalar {
                    logical_component,
                    leaf_offset,
                } if *logical_component == first => {
                    if path.len() == 1 {
                        return Some(*leaf_offset);
                    }
                    return None;
                }
                VirtualComponentPlan::NestedProduct {
                    logical_component,
                    leaf_offset,
                    shape,
                } if *logical_component == first => {
                    if path.len() == 1 {
                        return None;
                    }
                    return shape.scalar_offset_for_path(&path[1..]).map(|sub| leaf_offset + sub);
                }
                _ => {}
            }
        }
        None
    }

    pub fn subshape_for_path(&self, path: &[u32]) -> Option<&VirtualShapePlan> {
        if path.is_empty() {
            return Some(self);
        }
        let first = path[0];
        for comp in self.components.iter() {
            if let VirtualComponentPlan::NestedProduct { logical_component, shape, .. } = comp {
                if *logical_component == first {
                    return shape.subshape_for_path(&path[1..]);
                }
            }
        }
        None
    }

    pub fn subshape_offset_for_path(&self, path: &[u32]) -> Option<u16> {
        if path.is_empty() {
            return Some(0);
        }
        let first = path[0];
        for comp in self.components.iter() {
            if let VirtualComponentPlan::NestedProduct {
                logical_component,
                leaf_offset,
                shape,
            } = comp
            {
                if *logical_component == first {
                    return shape.subshape_offset_for_path(&path[1..]).map(|sub| leaf_offset + sub);
                }
            }
        }
        None
    }
}

/// Reasons why a product candidate is not virtualized.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationReason {
    OptimizationDisabled,
    NoResolvedConstruction,
    NotSimpleLocalBinding,
    MutableBinding,
    CapturedBinding,
    ReassignedBinding,
    ModuleOrGlobalBinding,
    DynamicOrUnprovenConstruction,
    ExpandedArguments,
    NullaryAlreadyImmediate,
    NoProfitableVirtualUse,
    MultipleWholeValueUses,
    RepeatedLoopWholeUse,
    EnumWholeValueObserved,
    NativeOptionAlreadySpecialized,
    LeafBudgetExceeded,
}

/// Optimization decision for a candidate local binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProductOptimizationDecision {
    Virtualize(VirtualShapePlan),
    Materialize(MaterializationReason),
}

/// Classified use of a candidate binding.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProductUseKind {
    DataProjection { component_path: Box<[u32]> },
    ExactVariantMatch,
    WholeValue { loop_depth: u16 },
}

/// Summarized usage profile for an analyzed binding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProductUseSummary {
    pub creation_loop_depth: u16,
    pub projections: usize,
    pub exact_variant_matches: usize,
    pub whole_values: Vec<u16>, // loop depth of each whole value use
    pub captured: bool,
    pub reassigned: bool,
    pub opaque_expansion: bool,
}

/// The complete product optimization plan for a single compiled body/function.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProductFunctionPlan {
    pub binding_decisions: BTreeMap<BindingId, ProductOptimizationDecision>,
    pub binding_decisions_by_range: BTreeMap<SourceRange, (BindingId, ProductOptimizationDecision)>,
    pub ephemeral_projections: BTreeMap<SourceRange, (VirtualShapePlan, Box<[u32]>)>,
}

/// Active compiler state representing a virtual product living in `leaf_count` frame slots starting at `head_slot`.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ActiveVirtualProduct {
    pub binding: Option<BindingId>,
    pub head_slot: u16,
    pub leaf_count: u16,
    pub shape: VirtualShapePlan,
    pub materialization_spec: Option<u16>,
}

impl ActiveVirtualProduct {
    #[allow(dead_code)]
    pub fn is_data(&self) -> bool {
        matches!(self.shape.kind, VirtualProductKind::Data { .. })
    }

    #[allow(dead_code)]
    pub fn is_tuple(&self) -> bool {
        matches!(self.shape.kind, VirtualProductKind::Tuple { .. })
    }

    #[allow(dead_code)]
    pub fn is_record(&self) -> bool {
        matches!(self.shape.kind, VirtualProductKind::Record { .. })
    }

    pub fn is_transparent_product(&self) -> bool {
        matches!(
            self.shape.kind,
            VirtualProductKind::Data { .. } | VirtualProductKind::Tuple { .. } | VirtualProductKind::Record { .. }
        )
    }

    #[allow(dead_code)]
    pub fn is_variant(&self) -> bool {
        matches!(self.shape.kind, VirtualProductKind::Variant { .. })
    }
}

/// Context passed during AST traversal to classify uses.
#[derive(Clone, Copy, Debug)]
pub(crate) enum UseContext<'a> {
    Value,
    DataProjection(&'a [u32]),
    AssignmentTarget,
    MatchScrutinee,
}

/// Planner candidate recorded before eligibility decision.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct CandidateBinding {
    pub binding: BindingId,
    pub name: String,
    pub range: SourceRange,
    pub mutable: bool,
    pub is_global: bool,
    pub shape: Option<VirtualShapePlan>,
    pub rejection_reason: Option<MaterializationReason>,
    pub summary: ProductUseSummary,
}

/// Pre-pass planner that walks lexical AST structures against `ModuleLoweringSemantics`
/// and constructs a sound, conservative `ProductFunctionPlan`.
pub(crate) struct ProductPlanner<'a> {
    lowering: Option<&'a ModuleLoweringSemantics>,
    mode: ProductOptimizationMode,
    candidates: BTreeMap<BindingId, CandidateBinding>,
    module_body: bool,
    current_loop_depth: u16,
    nested_block_depth: usize,
    ephemeral_projections: BTreeMap<SourceRange, (VirtualShapePlan, Box<[u32]>)>,
}

impl<'a> ProductPlanner<'a> {
    pub fn new(lowering: Option<&'a ModuleLoweringSemantics>, mode: ProductOptimizationMode) -> Self {
        Self {
            lowering,
            mode,
            candidates: BTreeMap::new(),
            module_body: false,
            current_loop_depth: 0,
            nested_block_depth: 0,
            ephemeral_projections: BTreeMap::new(),
        }
    }

    fn binding_for_range(&self, range: SourceRange) -> Option<BindingId> {
        self.lowering?.bindings.get(&range).copied()
    }

    /// Recursively builds a `VirtualShapePlan` for a data or variant construction expression.
    pub fn build_virtual_shape(&self, expr: &Expr, current_leaf_offset: u16, depth: usize) -> Option<VirtualShapePlan> {
        if depth > 4 {
            return None;
        }

        let lowering = self.lowering?;
        let range = expr.range();

        // 1. Check for AssociatedInvoke
        if let Some(spec) = lowering
            .associated
            .iter()
            .find(|(site, _)| site.range == range && (site.kind == LoweringSiteKind::AssociatedInvoke || site.kind == LoweringSiteKind::AssociatedLookup))
            .map(|(_, s)| s)
        {
            match spec {
                AssociatedLoweringSpec::ConstructData {
                    constructor,
                    arity,
                    construction,
                } => {
                    if *arity == 0 {
                        return None; // Nullary already immediate
                    }
                    let args = match expr {
                        Expr::AssociatedInvoke(inv) => &inv.args,
                        Expr::MethodCall(mc) => &mc.args,
                        _ => return None,
                    };
                    if args.len() != *arity as usize {
                        return None;
                    }
                    // Check for dynamic pack/expand
                    for arg in args.iter() {
                        if matches!(arg, PackItem::Expand { .. }) {
                            return None;
                        }
                    }

                    // Build components
                    let mut components = Vec::new();
                    let mut leaf_offset = current_leaf_offset;
                    for (arg_idx, comp_idx) in construction.argument_to_component.iter().enumerate() {
                        let arg_expr = match &args[arg_idx] {
                            PackItem::Positional { expr, .. } => expr,
                            PackItem::Labeled { value, .. } => value,
                            _ => return None,
                        };

                        // Check if nested product construction
                        if let Some(nested_shape) = self.build_virtual_shape(arg_expr, leaf_offset, depth + 1) {
                            let leaf_count = nested_shape.leaf_count;
                            components.push(VirtualComponentPlan::NestedProduct {
                                logical_component: *comp_idx,
                                leaf_offset,
                                shape: Box::new(nested_shape),
                            });
                            leaf_offset += leaf_count;
                        } else {
                            components.push(VirtualComponentPlan::Scalar {
                                logical_component: *comp_idx,
                                leaf_offset,
                            });
                            leaf_offset += 1;
                        }
                    }
                    let leaf_count = leaf_offset - current_leaf_offset;
                    if leaf_count as usize > MAX_VIRTUAL_PRODUCT_LEAVES {
                        return None;
                    }
                    return Some(VirtualShapePlan {
                        kind: VirtualProductKind::Data {
                            constructor: constructor.clone(),
                            construction: construction.clone(),
                        },
                        components: components.into_boxed_slice(),
                        leaf_count,
                    });
                }
                AssociatedLoweringSpec::ConstructVariant { variant, arity } => {
                    if *arity == 0 {
                        return None;
                    }
                    // General enum case payload is treated as scalar leaves in P2
                    let mut components = Vec::new();
                    let mut leaf_offset = current_leaf_offset;
                    for logical_component in 0..(*arity as u32) {
                        components.push(VirtualComponentPlan::Scalar {
                            logical_component,
                            leaf_offset,
                        });
                        leaf_offset += 1;
                    }
                    let leaf_count = leaf_offset - current_leaf_offset;
                    if leaf_count as usize > MAX_VIRTUAL_PRODUCT_LEAVES {
                        return None;
                    }
                    return Some(VirtualShapePlan {
                        kind: VirtualProductKind::Variant { variant: variant.clone() },
                        components: components.into_boxed_slice(),
                        leaf_count,
                    });
                }
                _ => return None,
            }
        }

        // 2. Check for RecordConstruction
        if let Expr::RecordConstruction(rec) = expr {
            if let Some(AssociatedLoweringSpec::ConstructData {
                constructor,
                arity,
                construction,
            }) = lowering
                .associated
                .iter()
                .find(|(site, _)| site.range == rec.range && site.kind == LoweringSiteKind::AssociatedInvoke)
                .map(|(_, s)| s)
            {
                if *arity == 0 {
                    return None;
                }
                let mut components = Vec::new();
                let mut leaf_offset = current_leaf_offset;
                for (arg_idx, comp_idx) in construction.argument_to_component.iter().enumerate() {
                    let entry = &rec.entries[arg_idx];
                    if let Some(nested_shape) = self.build_virtual_shape(&entry.value, leaf_offset, depth + 1) {
                        let leaf_count = nested_shape.leaf_count;
                        components.push(VirtualComponentPlan::NestedProduct {
                            logical_component: *comp_idx,
                            leaf_offset,
                            shape: Box::new(nested_shape),
                        });
                        leaf_offset += leaf_count;
                    } else {
                        components.push(VirtualComponentPlan::Scalar {
                            logical_component: *comp_idx,
                            leaf_offset,
                        });
                        leaf_offset += 1;
                    }
                }
                let leaf_count = leaf_offset - current_leaf_offset;
                if leaf_count as usize > MAX_VIRTUAL_PRODUCT_LEAVES {
                    return None;
                }
                return Some(VirtualShapePlan {
                    kind: VirtualProductKind::Data {
                        constructor: constructor.clone(),
                        construction: construction.clone(),
                    },
                    components: components.into_boxed_slice(),
                    leaf_count,
                });
            }
        }

        // 3. Check for Static Tuple Literal
        if let Expr::TupleLiteral(t) = expr {
            if let Some(spec) = lowering.anonymous_products.get(&t.range).cloned() {
                if let crate::modules::semantic_lowering::AnonymousProductConstructionKind::Tuple { positional_len, labels } = &spec.kind {
                    let total_len = *positional_len as usize + labels.len();
                    if total_len == 0 {
                        return None; // Zero arity is Unit
                    }
                    let mut components = Vec::new();
                    let mut leaf_offset = current_leaf_offset;
                    for (comp_idx, entry) in t.entries.iter().enumerate() {
                        let comp_expr = match entry {
                            phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } => expr,
                            phalcom_ast::ast::TupleLiteralEntry::Labeled { value, .. } => value,
                            phalcom_ast::ast::TupleLiteralEntry::Expand { .. } => return None,
                        };
                        if let Some(nested_shape) = self.build_virtual_shape(comp_expr, leaf_offset, depth + 1) {
                            let leaf_count = nested_shape.leaf_count;
                            components.push(VirtualComponentPlan::NestedProduct {
                                logical_component: comp_idx as u32,
                                leaf_offset,
                                shape: Box::new(nested_shape),
                            });
                            leaf_offset += leaf_count;
                        } else {
                            components.push(VirtualComponentPlan::Scalar {
                                logical_component: comp_idx as u32,
                                leaf_offset,
                            });
                            leaf_offset += 1;
                        }
                    }
                    let leaf_count = leaf_offset - current_leaf_offset;
                    if leaf_count as usize > MAX_VIRTUAL_PRODUCT_LEAVES {
                        return None;
                    }
                    return Some(VirtualShapePlan {
                        kind: VirtualProductKind::Tuple { spec },
                        components: components.into_boxed_slice(),
                        leaf_count,
                    });
                }
            }
        }

        // 4. Check for Static Record Literal
        if let Expr::RecordLiteral(r) = expr {
            if let Some(spec) = lowering.anonymous_products.get(&r.range).cloned() {
                if let crate::modules::semantic_lowering::AnonymousProductConstructionKind::Record { source_to_logical, .. } = &spec.kind {
                    if source_to_logical.is_empty() {
                        return None; // Zero arity is Unit
                    }
                    let mut components = Vec::new();
                    let mut leaf_offset = current_leaf_offset;
                    for (entry_idx, comp_idx) in source_to_logical.iter().enumerate() {
                        let entry = match &r.entries[entry_idx] {
                            phalcom_ast::ast::RecordLiteralEntry::Field(field) => field,
                            phalcom_ast::ast::RecordLiteralEntry::Expansion { .. } => return None,
                        };
                        if let Some(nested_shape) = self.build_virtual_shape(&entry.value, leaf_offset, depth + 1) {
                            let leaf_count = nested_shape.leaf_count;
                            components.push(VirtualComponentPlan::NestedProduct {
                                logical_component: *comp_idx,
                                leaf_offset,
                                shape: Box::new(nested_shape),
                            });
                            leaf_offset += leaf_count;
                        } else {
                            components.push(VirtualComponentPlan::Scalar {
                                logical_component: *comp_idx,
                                leaf_offset,
                            });
                            leaf_offset += 1;
                        }
                    }
                    let leaf_count = leaf_offset - current_leaf_offset;
                    if leaf_count as usize > MAX_VIRTUAL_PRODUCT_LEAVES {
                        return None;
                    }
                    return Some(VirtualShapePlan {
                        kind: VirtualProductKind::Record { spec },
                        components: components.into_boxed_slice(),
                        leaf_count,
                    });
                }
            }
        }

        None
    }

    pub fn plan_program(mut self, program: &Program) -> ProductFunctionPlan {
        self.module_body = true;
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
        self.finish_plan()
    }

    pub fn plan_statements(mut self, stmts: &[Statement]) -> ProductFunctionPlan {
        self.module_body = false;
        for stmt in stmts {
            self.visit_statement(stmt);
        }
        self.finish_plan()
    }

    fn finish_plan(self) -> ProductFunctionPlan {
        let mut binding_decisions = BTreeMap::new();
        let mut binding_decisions_by_range = BTreeMap::new();
        for (binding_id, candidate) in self.candidates {
            let decision = decide_product_optimization(&candidate, self.mode);
            binding_decisions_by_range.insert(candidate.range, (binding_id, decision.clone()));
            binding_decisions.insert(binding_id, decision);
        }
        ProductFunctionPlan {
            binding_decisions,
            binding_decisions_by_range,
            ephemeral_projections: self.ephemeral_projections,
        }
    }

    pub fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(binding) => {
                let is_global = self.module_body;
                let mutable = matches!(binding.kind, BindingKind::Let);

                // Analyze initializer BEFORE inserting into scope
                let shape = if let Some(init_expr) = &binding.value {
                    self.visit_expression(init_expr, UseContext::Value);
                    self.build_virtual_shape(init_expr, 0, 0)
                } else {
                    None
                };

                if let Pattern::Name { name, .. } = &binding.pattern {
                    if let Some(binding_id) = self.binding_for_range(binding.range) {
                        let mut candidate = CandidateBinding {
                            binding: binding_id,
                            name: name.clone(),
                            range: binding.range,
                            mutable,
                            is_global,
                            shape: shape.clone(),
                            rejection_reason: None,
                            summary: ProductUseSummary {
                                creation_loop_depth: self.current_loop_depth,
                                ..Default::default()
                            },
                        };
                        if is_global {
                            candidate.rejection_reason = Some(MaterializationReason::ModuleOrGlobalBinding);
                        } else if mutable {
                            candidate.rejection_reason = Some(MaterializationReason::MutableBinding);
                        } else if shape.is_none() {
                            candidate.rejection_reason = Some(MaterializationReason::NoResolvedConstruction);
                        }
                        self.candidates.insert(binding_id, candidate);
                    }
                } else {
                    // Refutable / destructuring patterns are ineligible for P2 product optimization
                }
            }
            Statement::Expr { expr, .. } => {
                self.visit_expression(expr, UseContext::Value);
            }
            Statement::Return(ret) => {
                if let Some(val) = &ret.value {
                    self.visit_expression(val, UseContext::Value);
                }
            }
            Statement::For(for_stmt) => {
                self.current_loop_depth += 1;
                for lane in &for_stmt.lanes {
                    self.visit_expression(&lane.iter, UseContext::Value);
                }
                for s in &for_stmt.body {
                    self.visit_statement(s);
                }
                self.current_loop_depth -= 1;
            }
            Statement::Break { .. } | Statement::Continue { .. } | Statement::TypeAlias(_) | Statement::Export(_) => {}
            Statement::Throw { expr, .. } => {
                self.visit_expression(expr, UseContext::Value);
            }
            Statement::Class(class_def) => {
                for member in &class_def.members {
                    match member {
                        phalcom_ast::ast::ClassMember::Method(m) => {
                            if let Some(stmts) = m.body.statements() {
                                self.nested_block_depth += 1;
                                for s in stmts {
                                    self.visit_statement(s);
                                }
                                self.nested_block_depth -= 1;
                            }
                        }
                        phalcom_ast::ast::ClassMember::Getter(g) => {
                            if let Some(stmts) = g.body.statements() {
                                self.nested_block_depth += 1;
                                for s in stmts {
                                    self.visit_statement(s);
                                }
                                self.nested_block_depth -= 1;
                            }
                        }
                        phalcom_ast::ast::ClassMember::Setter(s) => {
                            if let Some(stmts) = s.body.statements() {
                                self.nested_block_depth += 1;
                                for st in stmts {
                                    self.visit_statement(st);
                                }
                                self.nested_block_depth -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Statement::Impl(impl_def) => {
                for member in &impl_def.members {
                    self.nested_block_depth += 1;
                    match member {
                        phalcom_ast::ast::BehaviorMember::Method(method) => {
                            if let Some(body) = method.body.statements() {
                                for statement in body { self.visit_statement(statement); }
                            }
                        }
                        phalcom_ast::ast::BehaviorMember::Getter(getter) => {
                            if let Some(body) = getter.body.statements() {
                                for statement in body { self.visit_statement(statement); }
                            }
                        }
                        phalcom_ast::ast::BehaviorMember::Setter(setter) => {
                            if let Some(body) = setter.body.statements() {
                                for statement in body { self.visit_statement(statement); }
                            }
                        }
                        phalcom_ast::ast::BehaviorMember::Index(index) => {
                            for statement in &index.body { self.visit_statement(statement); }
                        }
                    }
                    self.nested_block_depth -= 1;
                }
            }
            Statement::Enum(_) | Statement::Data(_) => {}
        }
    }

    pub fn visit_expression(&mut self, expr: &Expr, ctx: UseContext) {
        match expr {
            Expr::Var { .. } => {
                if let Some(binding_id) = self.binding_for_range(expr.range()) {
                    if let Some(candidate) = self.candidates.get_mut(&binding_id) {
                        if self.nested_block_depth > 0 {
                            candidate.summary.captured = true;
                        }
                        match ctx {
                            UseContext::DataProjection(_path) => {
                                candidate.summary.projections += 1;
                            }
                            UseContext::AssignmentTarget => {
                                candidate.summary.reassigned = true;
                            }
                            UseContext::MatchScrutinee => {
                                candidate.summary.exact_variant_matches += 1;
                            }
                            UseContext::Value => {
                                candidate.summary.whole_values.push(self.current_loop_depth);
                            }
                        }
                    }
                }
            }
            Expr::GetProperty(get_prop) => {
                // Check if this property access is a resolved data component projection
                let mut path = Vec::new();
                let mut curr_expr: &Expr = expr;
                while let Expr::GetProperty(gp) = curr_expr {
                    if let Some(lowering) = self.lowering {
                        if let Some(AssociatedLoweringSpec::GetDataComponent { logical_index, .. }) = lowering
                            .associated
                            .iter()
                            .find(|(site, _)| site.range == gp.range && site.kind == LoweringSiteKind::AssociatedLookup)
                            .map(|(_, s)| s)
                        {
                            path.push(*logical_index);
                            curr_expr = &gp.object;
                            continue;
                        }
                    }
                    break;
                }

                if !path.is_empty() {
                    path.reverse();
                    // If root is a Var, visit it with DataProjection(path)
                    if let Expr::Var { .. } = curr_expr {
                        self.visit_expression(curr_expr, UseContext::DataProjection(&path));
                        return;
                    }
                    // If root is a direct constructor, record ephemeral projection
                    if let Some(shape) = self.build_virtual_shape(curr_expr, 0, 0) {
                        self.ephemeral_projections.insert(expr.range(), (shape, path.into_boxed_slice()));
                        self.visit_expression(curr_expr, UseContext::Value);
                        return;
                    }
                }

                // Ordinary GetProperty fallback
                self.visit_expression(&get_prop.object, UseContext::Value);
            }
            Expr::SetProperty(set_prop) => {
                self.visit_expression(&set_prop.object, UseContext::Value);
                self.visit_expression(&set_prop.value, UseContext::Value);
            }
            Expr::Assignment(assign) => {
                self.visit_expression(&assign.name, UseContext::AssignmentTarget);
                self.visit_expression(&assign.value, UseContext::Value);
            }
            Expr::Block(block) => {
                self.nested_block_depth += 1;
                for stmt in &block.body {
                    self.visit_statement(stmt);
                }
                self.nested_block_depth -= 1;
            }
            Expr::MethodCall(mc) => {
                self.visit_expression(&mc.object, UseContext::Value);
                for arg in &mc.args {
                    self.visit_pack_item(arg);
                }
            }
            Expr::AssociatedInvoke(inv) => {
                self.visit_expression(&inv.receiver, UseContext::Value);
                for arg in &inv.args {
                    self.visit_pack_item(arg);
                }
            }
            Expr::AssociatedLookup(lookup) => {
                self.visit_expression(&lookup.receiver, UseContext::Value);
            }
            Expr::CallableReference(c) => {
                if let phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. } = &c.target {
                    self.visit_expression(receiver, UseContext::Value);
                }
            }
            Expr::Binary(b) => {
                self.visit_expression(&b.left, UseContext::Value);
                self.visit_expression(&b.right, UseContext::Value);
            }
            Expr::Unary(u) => {
                self.visit_expression(&u.expr, UseContext::Value);
            }
            Expr::ComparisonChain(comp) => {
                for operand in &comp.operands {
                    self.visit_expression(operand, UseContext::Value);
                }
            }
            Expr::Match(m) => {
                self.visit_match_expr(m);
            }
            Expr::IfLet(if_let) => {
                self.visit_expression(&if_let.value, UseContext::Value);
                for stmt in &if_let.then_body.body {
                    self.visit_statement(stmt);
                }
                if let Some(else_branch) = &if_let.else_body {
                    for stmt in &else_branch.body {
                        self.visit_statement(stmt);
                    }
                }
            }
            Expr::WhileLet(while_let) => {
                self.current_loop_depth += 1;
                self.visit_expression(&while_let.value, UseContext::Value);
                for stmt in &while_let.body {
                    self.visit_statement(stmt);
                }
                self.current_loop_depth -= 1;
            }
            Expr::TupleLiteral(t) => {
                for entry in &t.entries {
                    match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } => self.visit_expression(expr, UseContext::Value),
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { value, .. } => self.visit_expression(value, UseContext::Value),
                        phalcom_ast::ast::TupleLiteralEntry::Expand { expr, .. } => self.visit_expression(expr, UseContext::Value),
                    }
                }
            }
            Expr::ListLiteral(l) => {
                for elem in &l.elements {
                    match elem {
                        phalcom_ast::ast::ListLiteralElement::Element { expr, .. } => self.visit_expression(expr, UseContext::Value),
                        phalcom_ast::ast::ListLiteralElement::Expansion { expr, .. } => self.visit_expression(expr, UseContext::Value),
                    }
                }
            }
            Expr::RecordLiteral(r) => {
                for entry in &r.entries {
                    match entry {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => self.visit_expression(&field.value, UseContext::Value),
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { expr, .. } => self.visit_expression(expr, UseContext::Value),
                    }
                }
            }
            Expr::RecordConstruction(rc) => {
                for entry in &rc.entries {
                    self.visit_expression(&entry.value, UseContext::Value);
                }
            }
            Expr::MapLiteral(m) => {
                for entry in &m.entries {
                    match entry {
                        phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                            if let phalcom_ast::ast::MapLiteralKey::Computed { expr, .. } = key {
                                self.visit_expression(expr, UseContext::Value);
                            }
                            self.visit_expression(value, UseContext::Value);
                        }
                        phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => self.visit_expression(expr, UseContext::Value),
                    }
                }
            }
            Expr::SetLiteral(s) => {
                for entry in &s.entries {
                    match entry {
                        phalcom_ast::ast::SetLiteralEntry::Element { expr, .. } => self.visit_expression(expr, UseContext::Value),
                        phalcom_ast::ast::SetLiteralEntry::Expansion { expr, .. } => self.visit_expression(expr, UseContext::Value),
                    }
                }
            }
            Expr::Index(i) => {
                self.visit_expression(&i.object, UseContext::Value);
                for arg in &i.args {
                    self.visit_pack_item(arg);
                }
            }
            Expr::SetIndex(si) => {
                self.visit_expression(&si.object, UseContext::Value);
                for arg in &si.args {
                    self.visit_pack_item(arg);
                }
                self.visit_expression(&si.value, UseContext::Value);
            }
            Expr::Range(r) => {
                if let Some(l) = &r.lower {
                    self.visit_expression(l, UseContext::Value);
                }
                if let Some(u) = &r.upper {
                    self.visit_expression(u, UseContext::Value);
                }
            }
            Expr::UnqualifiedCall(call) => {
                for arg in &call.args {
                    self.visit_pack_item(arg);
                }
            }
            Expr::Membership(m) => {
                self.visit_expression(&m.left, UseContext::Value);
                self.visit_expression(&m.right, UseContext::Value);
            }
            Expr::IsMembership(m) => {
                self.visit_expression(&m.left, UseContext::Value);
                self.visit_expression(&m.candidates, UseContext::Value);
            }
            Expr::SuperVar { .. }
            | Expr::SelfVar { .. }
            | Expr::TypeForm(_)
            | Expr::Ellipsis { .. }
            | Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::Symbol(_)
            | Expr::ImplementationSelector { .. }
            | Expr::Field { .. } => {}
        }
    }

    fn visit_pack_item(&mut self, item: &PackItem) {
        match item {
            PackItem::Positional { expr, .. } => self.visit_expression(expr, UseContext::Value),
            PackItem::Labeled { value, .. } => self.visit_expression(value, UseContext::Value),
            PackItem::Expand { expr, .. } => self.visit_expression(expr, UseContext::Value),
        }
    }

    fn visit_match_expr(&mut self, match_expr: &MatchExpr) {
        // Scrutinee can be checked for match candidate
        let is_candidate = if matches!(&*match_expr.value, Expr::Var { .. }) {
            self.binding_for_range(match_expr.value.range())
        } else {
            None
        };

        // Check if any arm has a root whole-value binding pattern
        let has_root_whole_binding = match_expr.arms.iter().any(|arm| matches!(arm.pattern, Pattern::Name { .. }));

        if is_candidate.is_some() {
            if has_root_whole_binding {
                self.visit_expression(&match_expr.value, UseContext::Value);
            } else {
                self.visit_expression(&match_expr.value, UseContext::MatchScrutinee);
            }
        } else {
            self.visit_expression(&match_expr.value, UseContext::Value);
        }

        for arm in &match_expr.arms {
            self.visit_expression(&arm.branch, UseContext::Value);
        }
    }
}

/// Pure eligibility decision function.
/// Takes candidate declaration facts, use summary, and mode, and returns a `ProductOptimizationDecision`.
pub(crate) fn decide_product_optimization(candidate: &CandidateBinding, mode: ProductOptimizationMode) -> ProductOptimizationDecision {
    if mode == ProductOptimizationMode::Disabled {
        return ProductOptimizationDecision::Materialize(MaterializationReason::OptimizationDisabled);
    }

    if let Some(reason) = &candidate.rejection_reason {
        return ProductOptimizationDecision::Materialize(reason.clone());
    }

    if candidate.is_global {
        return ProductOptimizationDecision::Materialize(MaterializationReason::ModuleOrGlobalBinding);
    }

    if candidate.mutable {
        return ProductOptimizationDecision::Materialize(MaterializationReason::MutableBinding);
    }

    if candidate.summary.captured {
        return ProductOptimizationDecision::Materialize(MaterializationReason::CapturedBinding);
    }

    if candidate.summary.reassigned {
        return ProductOptimizationDecision::Materialize(MaterializationReason::ReassignedBinding);
    }

    if candidate.summary.opaque_expansion {
        return ProductOptimizationDecision::Materialize(MaterializationReason::ExpandedArguments);
    }

    let Some(shape) = &candidate.shape else {
        return ProductOptimizationDecision::Materialize(MaterializationReason::NoResolvedConstruction);
    };

    if shape.leaf_count as usize > MAX_VIRTUAL_PRODUCT_LEAVES {
        return ProductOptimizationDecision::Materialize(MaterializationReason::LeafBudgetExceeded);
    }

    match &shape.kind {
        VirtualProductKind::Data { .. } | VirtualProductKind::Tuple { .. } | VirtualProductKind::Record { .. } => {
            // Check loop-repeat allocation sinking restriction
            for &whole_loop_depth in &candidate.summary.whole_values {
                if whole_loop_depth > candidate.summary.creation_loop_depth {
                    return ProductOptimizationDecision::Materialize(MaterializationReason::RepeatedLoopWholeUse);
                }
            }

            if candidate.summary.whole_values.len() > 1 {
                return ProductOptimizationDecision::Materialize(MaterializationReason::MultipleWholeValueUses);
            }

            if candidate.summary.projections == 0 && candidate.summary.whole_values.len() == 1 {
                return ProductOptimizationDecision::Materialize(MaterializationReason::NoProfitableVirtualUse);
            }

            if candidate.summary.projections == 0 && candidate.summary.whole_values.is_empty() {
                return ProductOptimizationDecision::Materialize(MaterializationReason::NoProfitableVirtualUse);
            }

            ProductOptimizationDecision::Virtualize(shape.clone())
        }
        VirtualProductKind::Variant { .. } => {
            // General enum case virtualization requires ZERO whole-value uses
            if !candidate.summary.whole_values.is_empty() {
                return ProductOptimizationDecision::Materialize(MaterializationReason::EnumWholeValueObserved);
            }

            if candidate.summary.exact_variant_matches == 0 {
                return ProductOptimizationDecision::Materialize(MaterializationReason::NoProfitableVirtualUse);
            }

            ProductOptimizationDecision::Virtualize(shape.clone())
        }
    }
}

impl<'vm> Compiler<'vm> {
    /// Returns the active virtual product anchored at `head_slot`, if any.
    pub(crate) fn active_virtual_product(&self, head_slot: usize) -> Option<&ActiveVirtualProduct> {
        let func = self.functions.last()?;
        let slot = u16::try_from(head_slot).ok()?;
        func.active_virtual_products.get(&slot)
    }

    /// Prunes active virtual products whose leaf slots are no longer within live locals.
    pub(crate) fn remove_virtual_products_outside_live_locals(&mut self) {
        if let Some(func) = self.functions.last_mut() {
            let num_locals = func.num_locals as u16;
            func.active_virtual_products
                .retain(|&head_slot, product| head_slot + product.leaf_count <= num_locals);
        }
    }

    /// Reserves contiguous persistent frame slots for a virtual product.
    pub(crate) fn reserve_virtual_binding_slots(
        &mut self,
        source_symbol: crate::interner::Symbol,
        leaf_count: u16,
        range: SourceRange,
    ) -> Result<u16, CompilerError> {
        if leaf_count == 0 {
            return Err(CompilerError::Message("cannot reserve zero virtual leaves".to_string()));
        }
        // 1. Head slot gets the user-visible source symbol
        self.add_local(source_symbol, false)?;
        let head_slot = (self.functions.last().unwrap().num_locals - 1) as u16;
        self.emit(Bytecode::ReserveScratchLocal(head_slot), range);

        // 2. Trailing leaves get fresh scratch symbols
        for _ in 1..leaf_count {
            let scratch_sym = self.fresh_scratch_symbol("$product");
            self.add_local(scratch_sym, false)?;
            let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
            self.emit(Bytecode::ReserveScratchLocal(slot), range);
        }

        Ok(head_slot)
    }

    /// Reserves contiguous ephemeral scratch slots on the operand stack for constructor->projection.
    pub(crate) fn reserve_ephemeral_virtual_slots(&mut self, leaf_count: u16, range: SourceRange) -> Result<u16, CompilerError> {
        if leaf_count == 0 {
            return Err(CompilerError::Message("cannot reserve zero virtual leaves".to_string()));
        }
        let first_slot = self.reserve_pack_scratch("$ephemeral_head", range)?;
        for _ in 1..leaf_count {
            self.reserve_pack_scratch("$ephemeral_leaf", range)?;
        }
        Ok(first_slot)
    }

    /// Compiles constructor arguments in source order into virtual leaf slots.
    pub(crate) fn compile_virtual_constructor_into_slots(
        &mut self,
        expr: &Expr,
        shape: &VirtualShapePlan,
        head_slot: u16,
        range: SourceRange,
    ) -> Result<(), CompilerError> {
        match expr {
            Expr::TupleLiteral(t) => {
                for (comp_idx, entry) in t.entries.iter().enumerate() {
                    let arg_expr = match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } => expr,
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { value, .. } => value,
                        phalcom_ast::ast::TupleLiteralEntry::Expand { .. } => return Err(CompilerError::InvalidExecutablePattern(range)),
                    };
                    let comp_plan = shape
                        .components
                        .iter()
                        .find(|c| match c {
                            VirtualComponentPlan::Scalar { logical_component, .. } => *logical_component == comp_idx as u32,
                            VirtualComponentPlan::NestedProduct { logical_component, .. } => *logical_component == comp_idx as u32,
                        })
                        .ok_or_else(|| CompilerError::Message(format!("missing component plan for tuple {comp_idx}")))?;

                    match comp_plan {
                        VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                            self.compile_expr(arg_expr.clone())?;
                            let target_slot = head_slot + leaf_offset;
                            self.emit(Bytecode::SetLocal(target_slot), range);
                            self.emit(Bytecode::Pop, range);
                        }
                        VirtualComponentPlan::NestedProduct {
                            leaf_offset,
                            shape: nested_shape,
                            ..
                        } => {
                            let nested_head = head_slot + leaf_offset;
                            self.compile_virtual_constructor_into_slots(arg_expr, nested_shape, nested_head, range)?;
                        }
                    }
                }
            }
            Expr::RecordLiteral(r) => {
                let VirtualProductKind::Record { spec } = &shape.kind else {
                    return Err(CompilerError::MissingAssociatedResolution(range));
                };
                let crate::modules::semantic_lowering::AnonymousProductConstructionKind::Record { source_to_logical, .. } = &spec.kind else {
                    return Err(CompilerError::MissingAssociatedResolution(range));
                };

                for (entry_idx, comp_idx) in source_to_logical.iter().enumerate() {
                    let entry = match &r.entries[entry_idx] {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => field,
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { .. } => return Err(CompilerError::InvalidExecutablePattern(range)),
                    };
                    let comp_plan = shape
                        .components
                        .iter()
                        .find(|c| match c {
                            VirtualComponentPlan::Scalar { logical_component, .. } => logical_component == comp_idx,
                            VirtualComponentPlan::NestedProduct { logical_component, .. } => logical_component == comp_idx,
                        })
                        .ok_or_else(|| CompilerError::Message(format!("missing component plan for record {comp_idx}")))?;

                    match comp_plan {
                        VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                            self.compile_expr(entry.value.clone())?;
                            let target_slot = head_slot + leaf_offset;
                            self.emit(Bytecode::SetLocal(target_slot), range);
                            self.emit(Bytecode::Pop, range);
                        }
                        VirtualComponentPlan::NestedProduct {
                            leaf_offset,
                            shape: nested_shape,
                            ..
                        } => {
                            let nested_head = head_slot + leaf_offset;
                            self.compile_virtual_constructor_into_slots(&entry.value, nested_shape, nested_head, range)?;
                        }
                    }
                }
            }
            Expr::AssociatedInvoke(inv) => {
                let spec = {
                    let lowering = self.lowering().ok_or(CompilerError::MissingAssociatedResolution(range))?;
                    lowering
                        .associated
                        .iter()
                        .find(|(site, _)| {
                            site.range == inv.range && (site.kind == LoweringSiteKind::AssociatedInvoke || site.kind == LoweringSiteKind::AssociatedLookup)
                        })
                        .map(|(_, s)| s.clone())
                        .ok_or(CompilerError::MissingAssociatedResolution(range))?
                };

                match spec {
                    AssociatedLoweringSpec::ConstructData { construction, .. } => {
                        for (arg_idx, comp_idx) in construction.argument_to_component.iter().enumerate() {
                            let arg_item = &inv.args[arg_idx];
                            let arg_expr = match arg_item {
                                PackItem::Positional { expr, .. } => expr,
                                PackItem::Labeled { value, .. } => value,
                                _ => return Err(CompilerError::InvalidExecutablePattern(range)),
                            };

                            let comp_plan = shape
                                .components
                                .iter()
                                .find(|c| match c {
                                    VirtualComponentPlan::Scalar { logical_component, .. } => logical_component == comp_idx,
                                    VirtualComponentPlan::NestedProduct { logical_component, .. } => logical_component == comp_idx,
                                })
                                .ok_or_else(|| CompilerError::Message(format!("missing component plan for {comp_idx}")))?;

                            match comp_plan {
                                VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                                    self.compile_expr(arg_expr.clone())?;
                                    let target_slot = head_slot + leaf_offset;
                                    self.emit(Bytecode::SetLocal(target_slot), range);
                                    self.emit(Bytecode::Pop, range);
                                }
                                VirtualComponentPlan::NestedProduct {
                                    leaf_offset,
                                    shape: nested_shape,
                                    ..
                                } => {
                                    let nested_head = head_slot + leaf_offset;
                                    self.compile_virtual_constructor_into_slots(arg_expr, nested_shape, nested_head, range)?;
                                }
                            }
                        }
                    }
                    AssociatedLoweringSpec::ConstructVariant { arity, .. } => {
                        for (idx, arg_item) in inv.args.iter().enumerate() {
                            let arg_expr = match arg_item {
                                PackItem::Positional { expr, .. } => expr,
                                PackItem::Labeled { value, .. } => value,
                                _ => return Err(CompilerError::InvalidExecutablePattern(range)),
                            };
                            let target_slot = head_slot + idx as u16;
                            self.compile_expr(arg_expr.clone())?;
                            self.emit(Bytecode::SetLocal(target_slot), range);
                            self.emit(Bytecode::Pop, range);
                        }
                    }
                    _ => return Err(CompilerError::MissingAssociatedResolution(range)),
                }
            }
            Expr::MethodCall(mc) => {
                let spec = {
                    let lowering = self.lowering().ok_or(CompilerError::MissingAssociatedResolution(range))?;
                    lowering
                        .associated
                        .iter()
                        .find(|(site, _)| {
                            site.range == mc.range && (site.kind == LoweringSiteKind::AssociatedInvoke || site.kind == LoweringSiteKind::AssociatedLookup)
                        })
                        .map(|(_, s)| s.clone())
                        .ok_or(CompilerError::MissingAssociatedResolution(range))?
                };

                match spec {
                    AssociatedLoweringSpec::ConstructData { construction, .. } => {
                        for (arg_idx, comp_idx) in construction.argument_to_component.iter().enumerate() {
                            let arg_item = &mc.args[arg_idx];
                            let arg_expr = match arg_item {
                                PackItem::Positional { expr, .. } => expr,
                                PackItem::Labeled { value, .. } => value,
                                _ => return Err(CompilerError::InvalidExecutablePattern(range)),
                            };

                            let comp_plan = shape
                                .components
                                .iter()
                                .find(|c| match c {
                                    VirtualComponentPlan::Scalar { logical_component, .. } => logical_component == comp_idx,
                                    VirtualComponentPlan::NestedProduct { logical_component, .. } => logical_component == comp_idx,
                                })
                                .ok_or_else(|| CompilerError::Message(format!("missing component plan for {comp_idx}")))?;

                            match comp_plan {
                                VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                                    self.compile_expr(arg_expr.clone())?;
                                    let target_slot = head_slot + leaf_offset;
                                    self.emit(Bytecode::SetLocal(target_slot), range);
                                    self.emit(Bytecode::Pop, range);
                                }
                                VirtualComponentPlan::NestedProduct {
                                    leaf_offset,
                                    shape: nested_shape,
                                    ..
                                } => {
                                    let nested_head = head_slot + leaf_offset;
                                    self.compile_virtual_constructor_into_slots(arg_expr, nested_shape, nested_head, range)?;
                                }
                            }
                        }
                    }
                    AssociatedLoweringSpec::ConstructVariant { arity, .. } => {
                        for (idx, arg_item) in mc.args.iter().enumerate() {
                            let arg_expr = match arg_item {
                                PackItem::Positional { expr, .. } => expr,
                                PackItem::Labeled { value, .. } => value,
                                _ => return Err(CompilerError::InvalidExecutablePattern(range)),
                            };
                            let target_slot = head_slot + idx as u16;
                            self.compile_expr(arg_expr.clone())?;
                            self.emit(Bytecode::SetLocal(target_slot), range);
                            self.emit(Bytecode::Pop, range);
                        }
                    }
                    _ => return Err(CompilerError::MissingAssociatedResolution(range)),
                }
            }
            Expr::RecordConstruction(rec) => {
                let spec = {
                    let lowering = self.lowering().ok_or(CompilerError::MissingAssociatedResolution(range))?;
                    lowering
                        .associated
                        .iter()
                        .find(|(site, _)| site.range == rec.range && site.kind == LoweringSiteKind::AssociatedInvoke)
                        .map(|(_, s)| s.clone())
                        .ok_or(CompilerError::MissingAssociatedResolution(range))?
                };

                let AssociatedLoweringSpec::ConstructData { construction, .. } = spec else {
                    return Err(CompilerError::MissingAssociatedResolution(range));
                };

                for (arg_idx, comp_idx) in construction.argument_to_component.iter().enumerate() {
                    let entry = &rec.entries[arg_idx];
                    let comp_plan = shape
                        .components
                        .iter()
                        .find(|c| match c {
                            VirtualComponentPlan::Scalar { logical_component, .. } => logical_component == comp_idx,
                            VirtualComponentPlan::NestedProduct { logical_component, .. } => logical_component == comp_idx,
                        })
                        .ok_or_else(|| CompilerError::Message(format!("missing component plan for {comp_idx}")))?;

                    match comp_plan {
                        VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                            self.compile_expr(entry.value.clone())?;
                            let target_slot = head_slot + leaf_offset;
                            self.emit(Bytecode::SetLocal(target_slot), range);
                            self.emit(Bytecode::Pop, range);
                        }
                        VirtualComponentPlan::NestedProduct {
                            leaf_offset,
                            shape: nested_shape,
                            ..
                        } => {
                            let nested_head = head_slot + leaf_offset;
                            self.compile_virtual_constructor_into_slots(&entry.value, nested_shape, nested_head, range)?;
                        }
                    }
                }
            }
            _ => return Err(CompilerError::MissingAssociatedResolution(range)),
        }
        Ok(())
    }

    /// Emits projection of a single scalar leaf or a subshape from an active virtual product.
    pub(crate) fn emit_virtual_projection(&mut self, product: &ActiveVirtualProduct, component_path: &[u32], range: SourceRange) -> Result<(), CompilerError> {
        if let Some(leaf_offset) = product.shape.scalar_offset_for_path(component_path) {
            let slot = product.head_slot + leaf_offset;
            self.emit(Bytecode::GetLocal(slot), range);
            return Ok(());
        }

        // Check if path denotes a nested virtual product subshape
        if let Some(subshape) = product.shape.subshape_for_path(component_path) {
            let offset = product.shape.subshape_offset_for_path(component_path).unwrap_or(0);
            let sub_product = ActiveVirtualProduct {
                binding: None,
                head_slot: product.head_slot + offset,
                leaf_count: subshape.leaf_count,
                shape: subshape.clone(),
                materialization_spec: None,
            };
            return self.emit_virtual_materialization(&sub_product, range);
        }

        Err(CompilerError::Message("invalid component projection path on virtual product".to_string()))
    }

    /// Rematerializes a virtual transparent product (Data, Tuple, or Record).
    pub(crate) fn emit_virtual_materialization(&mut self, product: &ActiveVirtualProduct, range: SourceRange) -> Result<(), CompilerError> {
        match &product.shape.kind {
            VirtualProductKind::Data { construction, .. } => {
                // Load or recursively materialize components in logical argument order required by the recipe
                for (arg_idx, comp_idx) in construction.argument_to_component.iter().enumerate() {
                    let comp_plan = product
                        .shape
                        .components
                        .iter()
                        .find(|c| match c {
                            VirtualComponentPlan::Scalar { logical_component, .. } => logical_component == comp_idx,
                            VirtualComponentPlan::NestedProduct { logical_component, .. } => logical_component == comp_idx,
                        })
                        .ok_or_else(|| CompilerError::Message(format!("missing component {comp_idx} during materialization")))?;

                    match comp_plan {
                        VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                            let slot = product.head_slot + leaf_offset;
                            self.emit(Bytecode::GetLocal(slot), range);
                        }
                        VirtualComponentPlan::NestedProduct {
                            leaf_offset,
                            shape: nested_shape,
                            ..
                        } => {
                            let nested_product = ActiveVirtualProduct {
                                binding: None,
                                head_slot: product.head_slot + leaf_offset,
                                leaf_count: nested_shape.leaf_count,
                                shape: (**nested_shape).clone(),
                                materialization_spec: None,
                            };
                            self.emit_virtual_materialization(&nested_product, range)?;
                        }
                    }
                }

                let ctor_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_data_construction(construction.clone(), range)?;

                let arity = construction.argument_to_component.len() as u8;
                if arity == 0 {
                    self.emit(Bytecode::LoadDataSingleton(ctor_idx), range);
                } else {
                    self.emit(Bytecode::ConstructData { constructor: ctor_idx, arity }, range);
                }
                Ok(())
            }
            VirtualProductKind::Tuple { spec } => {
                let spec_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_anonymous_product_spec(spec.clone(), range)?;

                let total_len = product.shape.components.len();
                for comp_idx in 0..total_len {
                    let comp_plan = product
                        .shape
                        .components
                        .iter()
                        .find(|c| match c {
                            VirtualComponentPlan::Scalar { logical_component, .. } => *logical_component == comp_idx as u32,
                            VirtualComponentPlan::NestedProduct { logical_component, .. } => *logical_component == comp_idx as u32,
                        })
                        .ok_or_else(|| CompilerError::Message(format!("missing component {comp_idx} during tuple materialization")))?;

                    match comp_plan {
                        VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                            let slot = product.head_slot + leaf_offset;
                            self.emit(Bytecode::GetLocal(slot), range);
                        }
                        VirtualComponentPlan::NestedProduct {
                            leaf_offset,
                            shape: nested_shape,
                            ..
                        } => {
                            let nested_product = ActiveVirtualProduct {
                                binding: None,
                                head_slot: product.head_slot + leaf_offset,
                                leaf_count: nested_shape.leaf_count,
                                shape: (**nested_shape).clone(),
                                materialization_spec: None,
                            };
                            self.emit_virtual_materialization(&nested_product, range)?;
                        }
                    }
                }

                self.emit(Bytecode::BuildStaticTuple { spec: spec_idx }, range);
                Ok(())
            }
            VirtualProductKind::Record { spec } => {
                let spec_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_anonymous_product_spec(spec.clone(), range)?;

                let crate::modules::semantic_lowering::AnonymousProductConstructionKind::Record { source_to_logical, .. } = &spec.kind else {
                    return Err(CompilerError::Message("invalid record lowering spec".into()));
                };

                for comp_idx in source_to_logical.iter() {
                    let comp_plan = product
                        .shape
                        .components
                        .iter()
                        .find(|c| match c {
                            VirtualComponentPlan::Scalar { logical_component, .. } => logical_component == comp_idx,
                            VirtualComponentPlan::NestedProduct { logical_component, .. } => logical_component == comp_idx,
                        })
                        .ok_or_else(|| CompilerError::Message(format!("missing component {comp_idx} during record materialization")))?;

                    match comp_plan {
                        VirtualComponentPlan::Scalar { leaf_offset, .. } => {
                            let slot = product.head_slot + leaf_offset;
                            self.emit(Bytecode::GetLocal(slot), range);
                        }
                        VirtualComponentPlan::NestedProduct {
                            leaf_offset,
                            shape: nested_shape,
                            ..
                        } => {
                            let nested_product = ActiveVirtualProduct {
                                binding: None,
                                head_slot: product.head_slot + leaf_offset,
                                leaf_count: nested_shape.leaf_count,
                                shape: (**nested_shape).clone(),
                                materialization_spec: None,
                            };
                            self.emit_virtual_materialization(&nested_product, range)?;
                        }
                    }
                }

                self.emit(Bytecode::BuildStaticRecord { spec: spec_idx }, range);
                Ok(())
            }
            VirtualProductKind::Variant { .. } => {
                Err(CompilerError::Message("cannot rematerialize unobserved variant payload".to_string()))
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn emit_virtual_data_materialization(&mut self, product: &ActiveVirtualProduct, range: SourceRange) -> Result<(), CompilerError> {
        self.emit_virtual_materialization(product, range)
    }
}

#[cfg(test)]
mod tests;
