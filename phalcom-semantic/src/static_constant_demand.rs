//! Session-owned, depth-first forcing for constants selected by static keys.

use crate::checker::context::CheckingContext;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{InputFingerprint, ProductFingerprint, QueryKey};
use crate::db::query::FormalQueryInputs;
use crate::db::{QueryOutcome, SemanticDb};
use crate::declarations::DeclarationTypeTable;
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::ModuleId;
use crate::module_bindings::{ModuleStaticBindingCatalog, find_constant_declaration};
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::source::ParsedModuleUnit;
use crate::static_constant::{CheckedStaticExpression, StaticConstantId, StaticConstantProduct, StaticConstantTable};
use crate::static_definition::StaticDefinitionTable;
use crate::static_intrinsic::StaticIntrinsicRegistry;
use crate::static_key::{ProjectedRecordKeySet, project_checked_static_key, type_record_key_expressions};
use crate::static_value::StaticValueStore;
use crate::type_alias::TypeAliasTable;
use crate::types::annotation::{ProjectedRecordKeyResolver, TypeFormationSite, TypeResolver, resolve_type_annotation};
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::TypeAnnotation;
use phalcom_modules::identity::ImportSiteId;
use phalcom_modules::linker::LinkedProgram;
use phalcom_modules::resolver::ImportResolutionProduct;
use phalcom_modules::topology::ModuleTopology;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Control result for constant/key work that may suspend on an exact semantic owner.
pub(crate) enum StaticDemandOutcome<T> {
    Ready(T),
    Awaiting(BTreeSet<QueryKey>),
    Blocked(crate::types::outcome::BlockReason),
    Cancelled,
    BudgetExceeded(crate::types::outcome::BudgetReport),
    Failed(String),
}

impl<T> From<QueryOutcome<T>> for StaticDemandOutcome<T> {
    fn from(outcome: QueryOutcome<T>) -> Self {
        match outcome {
            QueryOutcome::Ready(value) => Self::Ready(value),
            QueryOutcome::Blocked(reason) => Self::Blocked(reason),
            QueryOutcome::Cancelled => Self::Cancelled,
            QueryOutcome::BudgetExceeded(report) => Self::BudgetExceeded(report),
            QueryOutcome::Failed(error) => Self::Failed(error),
        }
    }
}

/// Current session inputs used while a type owner is suspended on exact key demands.
pub(crate) struct StaticConstantDemandDriver<'a> {
    pub db: &'a mut SemanticDb,
    pub store: &'a mut TypeStore,
    pub values: &'a mut StaticValueStore,
    pub constants: &'a mut StaticConstantTable,
    pub definitions: &'a StaticDefinitionTable,
    pub catalogs: &'a BTreeMap<ModuleId, Arc<ModuleStaticBindingCatalog>>,
    pub sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub linked: &'a LinkedProgram,
    pub topology: &'a ModuleTopology,
    pub import_products: &'a BTreeMap<ImportSiteId, Arc<ImportResolutionProduct>>,
    pub source_resolution_input: InputFingerprint,
    pub linked_component_product: ProductFingerprint,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub type_aliases: &'a TypeAliasTable,
    pub dispatch: &'a SurfaceDispatchResolver,
    pub field_signatures: &'a FieldSignatureTable,
    pub callable_signatures: &'a CallableSignatureTable,
    pub data_semantics: &'a crate::data_semantics::DataSemanticTable,
    pub enum_semantics: &'a crate::enum_semantics::EnumSemanticTable,
    pub associated_families: &'a crate::associated::AssociatedFamilyTable,
    pub intrinsics: &'a StaticIntrinsicRegistry,
    pub budget: QueryBudget,
    pub cancel: &'a CancellationToken,
    pub(crate) active_constants: Vec<StaticConstantId>,
    pub(crate) deferred_static_definitions: &'a BTreeSet<crate::identity::DeclarationId>,
}

impl StaticConstantDemandDriver<'_> {
    /// Forces one checker-selected constant and its transitive constant reads.
    pub(crate) fn force(&mut self, id: &StaticConstantId) -> StaticDemandOutcome<Arc<StaticConstantProduct>> {
        let key = QueryKey::StaticConstant(id.clone());
        if self.db.validate_ready(&key) {
            if let Some(product) = self.db.product(&key).and_then(|product| product.as_static_constant()) {
                self.constants.insert(product.clone());
                return StaticDemandOutcome::Ready(product.clone());
            }
        }

        if let Some(cycle_start) = self.active_constants.iter().position(|active| active == id) {
            let cycle = self.active_constants[cycle_start..].iter().cloned().collect::<BTreeSet<_>>();
            match self.publish_recursive_cycle(&cycle) {
                QueryOutcome::Ready(()) => {
                    return self.constants.get(id).cloned().map_or_else(
                        || StaticDemandOutcome::Failed(format!("recursive constant cycle omitted {id:?}")),
                        StaticDemandOutcome::Ready,
                    );
                }
                QueryOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
                QueryOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
                QueryOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
            }
        }

        self.active_constants.push(id.clone());
        let outcome = self.compute(id);
        let popped = self.active_constants.pop();
        debug_assert_eq!(popped.as_ref(), Some(id), "static-constant demand stack must unwind in LIFO order");
        outcome
    }

    /// Checks, recursively forces, then projects each computed key in one owner annotation.
    pub(crate) fn project_annotation<'a>(
        &mut self,
        owner: QueryKey,
        module: &ModuleId,
        annotation: &'a TypeAnnotation,
    ) -> StaticDemandOutcome<ProjectedRecordKeySet<'a>> {
        self.project_annotations(owner, module, std::iter::once(annotation))
    }

    /// Projects computed keys across a deterministic set of annotations owned
    /// by one signature query, assigning ordinals across the complete owner.
    pub(crate) fn project_annotations<'a>(
        &mut self,
        owner: QueryKey,
        module: &ModuleId,
        annotations: impl IntoIterator<Item = &'a TypeAnnotation>,
    ) -> StaticDemandOutcome<ProjectedRecordKeySet<'a>> {
        let Some(catalog) = self.catalogs.get(module).cloned() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let runtime_states = BTreeMap::new();
        let discovery_states = crate::session::static_constant_binding_states(&catalog, &runtime_states, None);
        let mut sites = Vec::new();
        let mut next_ordinal = 0;
        for annotation in annotations {
            let annotation_sites = type_record_key_expressions(annotation, next_ordinal);
            if let Some(last) = annotation_sites.last() {
                next_ordinal = last.ordinal.saturating_add(1);
            }
            sites.extend(annotation_sites);
        }

        // The first ordinary-checker pass only discovers checker-selected stable
        // constant identities. No unresolved constant is given a speculative type.
        let mut demands = BTreeSet::new();
        for site in &sites {
            let checked = {
                let intrinsics = self.intrinsics;
                let context = demand_checking_context(
                    &mut *self.store,
                    self.hierarchy,
                    self.resolver,
                    self.declarations,
                    self.dispatch,
                    self.topology,
                    self.field_signatures,
                    self.data_semantics,
                    self.enum_semantics,
                    self.associated_families,
                    self.deferred_static_definitions,
                    module,
                );
                let checked = crate::checker::static_body::check_static_expression(context, site.expression, None, &catalog, &discovery_states, intrinsics);
                match checked {
                    crate::checker::static_body::StaticExpressionCheckOutcome::Ready(checked)
                    | crate::checker::static_body::StaticExpressionCheckOutcome::Invalid(checked) => checked,
                    crate::checker::static_body::StaticExpressionCheckOutcome::Awaiting(demands) => {
                        return StaticDemandOutcome::Awaiting(demands);
                    }
                }
            };
            demands.extend(checked.static_constant_reads.iter().cloned().map(StaticConstantId));
        }
        for demand in demands {
            match self.force(&demand) {
                StaticDemandOutcome::Ready(_) => {}
                StaticDemandOutcome::Awaiting(demands) => return StaticDemandOutcome::Awaiting(demands),
                StaticDemandOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
                StaticDemandOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
                StaticDemandOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
                StaticDemandOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
            }
        }

        let checked_states = crate::session::static_constant_binding_states(&catalog, &runtime_states, Some(self.constants));
        let mut projections = ProjectedRecordKeySet::new(owner);
        for site in sites {
            let checked = {
                let intrinsics = self.intrinsics;
                let context = demand_checking_context(
                    &mut *self.store,
                    self.hierarchy,
                    self.resolver,
                    self.declarations,
                    self.dispatch,
                    self.topology,
                    self.field_signatures,
                    self.data_semantics,
                    self.enum_semantics,
                    self.associated_families,
                    self.deferred_static_definitions,
                    module,
                );
                let checked = crate::checker::static_body::check_static_expression(context, site.expression, None, &catalog, &checked_states, intrinsics);
                match checked {
                    crate::checker::static_body::StaticExpressionCheckOutcome::Ready(checked)
                    | crate::checker::static_body::StaticExpressionCheckOutcome::Invalid(checked) => Arc::new(checked),
                    crate::checker::static_body::StaticExpressionCheckOutcome::Awaiting(demands) => {
                        return StaticDemandOutcome::Awaiting(demands);
                    }
                }
            };
            let mut budget = self.budget;
            let projection = match project_checked_static_key(
                &checked,
                self.constants,
                self.definitions,
                self.store,
                self.data_semantics,
                self.enum_semantics,
                self.values,
                &mut budget,
                self.cancel,
            ) {
                QueryOutcome::Ready(projection) => projection,
                QueryOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
                QueryOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
                QueryOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
            };
            if projections.insert(site.syntax, site.ordinal, checked, projection).is_err() {
                return StaticDemandOutcome::Failed("duplicate computed Record key identity".to_owned());
            }
        }
        StaticDemandOutcome::Ready(projections)
    }

    fn compute(&mut self, id: &StaticConstantId) -> StaticDemandOutcome<Arc<StaticConstantProduct>> {
        let Some(catalog) = self.catalogs.get(id.module()).cloned() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let Some(seed) = catalog.get(id.symbol()).cloned() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let Some(source) = self.sources.get(id.module()).cloned() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let Some(binding) = find_constant_declaration(&source.program, id.symbol(), seed.declaration_range, seed.initializer_range) else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let Some(initializer) = binding.value.as_ref() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let runtime_states = BTreeMap::new();
        let discovery_states = crate::session::static_constant_binding_states(&catalog, &runtime_states, None);

        // Discover initializer reads before checking again against current product types.
        let discovered = {
            let intrinsics = self.intrinsics;
            let context = demand_checking_context(
                &mut *self.store,
                self.hierarchy,
                self.resolver,
                self.declarations,
                self.dispatch,
                self.topology,
                self.field_signatures,
                self.data_semantics,
                self.enum_semantics,
                self.associated_families,
                self.deferred_static_definitions,
                id.module(),
            );
            crate::checker::static_body::check_static_expression(context, initializer, None, &catalog, &discovery_states, intrinsics)
        };
        let discovered = match discovered {
            crate::checker::static_body::StaticExpressionCheckOutcome::Ready(checked)
            | crate::checker::static_body::StaticExpressionCheckOutcome::Invalid(checked) => checked,
            crate::checker::static_body::StaticExpressionCheckOutcome::Awaiting(demands) => {
                return StaticDemandOutcome::Awaiting(demands);
            }
        };
        for symbol in discovered.static_constant_reads.iter() {
            let dependency = StaticConstantId(symbol.clone());
            if dependency == *id {
                return self.recursive_cycle_outcome(id).into();
            }
            match self.force(&dependency) {
                StaticDemandOutcome::Ready(_) => {}
                StaticDemandOutcome::Awaiting(demands) => return StaticDemandOutcome::Awaiting(demands),
                StaticDemandOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
                StaticDemandOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
                StaticDemandOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
                StaticDemandOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
            }
        }

        let (expected_type, key_expressions, annotation_diagnostics) = if let Some(annotation) = binding.annotation.as_ref() {
            let key_sites = type_record_key_expressions(annotation, 0);
            if key_sites.is_empty() {
                let mut diagnostics = Vec::new();
                if crate::checker::annotation_elaboration::annotation_contains_inference_hole(annotation) {
                    return self.check_evaluate_publish_with_expected(id, binding, crate::checker::expected::ExpectedType::None, Vec::new(), diagnostics);
                }
                let knowledge = resolve_type_annotation(
                    self.store,
                    self.declarations,
                    self.resolver,
                    &TypeFormationSite::module(id.module().clone()),
                    annotation,
                    &mut diagnostics,
                );
                (knowledge.ty(), Vec::<Arc<CheckedStaticExpression>>::new(), diagnostics)
            } else {
                let projections = match self.project_annotation(QueryKey::StaticConstant(id.clone()), id.module(), annotation) {
                    StaticDemandOutcome::Ready(projections) => projections,
                    StaticDemandOutcome::Awaiting(demands) => return StaticDemandOutcome::Awaiting(demands),
                    StaticDemandOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
                    StaticDemandOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
                    StaticDemandOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
                    StaticDemandOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
                };
                let mut diagnostics = Vec::new();
                let projected_resolver = ProjectedRecordKeyResolver {
                    parent: self.resolver,
                    projections: &projections,
                };
                let knowledge = resolve_type_annotation(
                    self.store,
                    self.declarations,
                    &projected_resolver,
                    &TypeFormationSite::module(id.module().clone()),
                    annotation,
                    &mut diagnostics,
                );
                let checked = projections.facts().map(|fact| fact.checked.clone()).collect::<Vec<_>>();
                let expected = knowledge.ty();
                // Reuse the checked owner facts when capturing exact key reads.
                return self.compute_with_projected_keys(id, binding, expected, checked, diagnostics);
            }
        } else {
            (None, Vec::<Arc<CheckedStaticExpression>>::new(), Vec::new())
        };
        let _ = &key_expressions;
        self.check_evaluate_publish(id, binding, expected_type, Vec::new(), annotation_diagnostics)
    }

    fn compute_with_projected_keys(
        &mut self,
        id: &StaticConstantId,
        binding: &phalcom_ast::ast::LetBinding,
        expected_type: Option<crate::types::id::TypeId>,
        key_expressions: Vec<Arc<crate::static_constant::CheckedStaticExpression>>,
        annotation_diagnostics: Vec<crate::diagnostic::SemanticDiagnostic>,
    ) -> StaticDemandOutcome<Arc<StaticConstantProduct>> {
        self.check_evaluate_publish(id, binding, expected_type, key_expressions, annotation_diagnostics)
    }

    fn check_evaluate_publish(
        &mut self,
        id: &StaticConstantId,
        binding: &phalcom_ast::ast::LetBinding,
        expected_type: Option<crate::types::id::TypeId>,
        key_expressions: Vec<Arc<crate::static_constant::CheckedStaticExpression>>,
        annotation_diagnostics: Vec<crate::diagnostic::SemanticDiagnostic>,
    ) -> StaticDemandOutcome<Arc<StaticConstantProduct>> {
        let expected = match expected_type {
            Some(ty) => crate::checker::expected::ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::DeclarationContract),
            None => crate::checker::expected::ExpectedType::None,
        };
        self.check_evaluate_publish_with_expected(id, binding, expected, key_expressions, annotation_diagnostics)
    }

    fn check_evaluate_publish_with_expected(
        &mut self,
        id: &StaticConstantId,
        binding: &phalcom_ast::ast::LetBinding,
        expected: crate::checker::expected::ExpectedType,
        key_expressions: Vec<Arc<crate::static_constant::CheckedStaticExpression>>,
        annotation_diagnostics: Vec<crate::diagnostic::SemanticDiagnostic>,
    ) -> StaticDemandOutcome<Arc<StaticConstantProduct>> {
        let Some(catalog) = self.catalogs.get(id.module()).cloned() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let Some(initializer) = binding.value.as_ref() else {
            return StaticDemandOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        let binding_states = crate::session::static_constant_binding_states(&catalog, &BTreeMap::new(), Some(self.constants));
        let expression_check = {
            let intrinsics = self.intrinsics;
            let mut context = demand_checking_context(
                &mut *self.store,
                self.hierarchy,
                self.resolver,
                self.declarations,
                self.dispatch,
                self.topology,
                self.field_signatures,
                self.data_semantics,
                self.enum_semantics,
                self.associated_families,
                self.deferred_static_definitions,
                id.module(),
            );
            let diagnostic_start = context.diagnostics.len();
            let expected = if let Some(annotation) = binding
                .annotation
                .as_ref()
                .filter(|annotation| crate::checker::annotation_elaboration::annotation_contains_inference_hole(annotation))
            {
                let (elaboration, _) = context.elaborate_type_annotation(self.resolver, annotation);
                match elaboration {
                    Some(crate::checker::annotation_elaboration::AnnotationElaboration::Inference { context: inf_ctx, term, .. }) => {
                        crate::checker::expected::ExpectedType::inference_from(inf_ctx, term, crate::checker::expected::ExpectationOrigin::DeclarationContract)
                    }
                    Some(crate::checker::annotation_elaboration::AnnotationElaboration::Canonical(ty)) => {
                        crate::checker::expected::ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::DeclarationContract)
                    }
                    None => crate::checker::expected::ExpectedType::None,
                }
            } else {
                expected
            };
            crate::checker::static_body::check_static_expression_with_expected_and_diagnostic_start(
                context,
                initializer,
                expected,
                &catalog,
                &binding_states,
                intrinsics,
                diagnostic_start,
            )
        };
        let expression = match expression_check {
            crate::checker::static_body::StaticExpressionCheckOutcome::Ready(checked)
            | crate::checker::static_body::StaticExpressionCheckOutcome::Invalid(checked) => checked,
            crate::checker::static_body::StaticExpressionCheckOutcome::Awaiting(demands) => {
                return StaticDemandOutcome::Awaiting(demands);
            }
        };
        let expected_type = expression.result_type;
        let input_fingerprint = crate::db::fingerprint::static_constant_input_fingerprint(id, initializer, binding.annotation.as_ref(), expected_type);
        let checked = crate::static_constant::CheckedStaticConstantInitializer {
            id: id.clone(),
            declaration_range: binding.range,
            initializer_range: initializer.range(),
            expression,
            record_key_static_constant_reads: Arc::from([]),
            input_fingerprint,
        };
        let mut checked = checked;
        if !annotation_diagnostics.is_empty() {
            let mut diagnostics = checked.diagnostics.to_vec();
            for diagnostic in annotation_diagnostics {
                if !diagnostics.contains(&diagnostic) {
                    diagnostics.push(diagnostic);
                }
            }
            checked.expression.diagnostics = Arc::from(diagnostics.into_boxed_slice());
        }
        if !key_expressions.is_empty() {
            let mut dependencies = checked.dependencies.to_vec();
            let mut reads = checked.static_constant_reads.to_vec();
            let mut record_key_reads = Vec::new();
            for key_expression in key_expressions {
                dependencies.extend(key_expression.dependencies.iter().cloned());
                reads.extend(key_expression.static_constant_reads.iter().cloned());
                record_key_reads.extend(key_expression.static_constant_reads.iter().cloned());
            }
            dependencies.sort();
            dependencies.dedup();
            reads.sort();
            reads.dedup();
            record_key_reads.sort();
            record_key_reads.dedup();
            checked.expression.dependencies = Arc::from(dependencies.into_boxed_slice());
            checked.expression.static_constant_reads = Arc::from(reads.into_boxed_slice());
            checked.record_key_static_constant_reads = Arc::from(record_key_reads.into_boxed_slice());
        }
        let checked = Arc::new(checked);
        for symbol in checked.static_constant_reads.iter() {
            let dependency = StaticConstantId(symbol.clone());
            if dependency == *id {
                return self.recursive_cycle_outcome(id).into();
            }
            match self.force(&dependency) {
                StaticDemandOutcome::Ready(_) => {}
                StaticDemandOutcome::Awaiting(demands) => return StaticDemandOutcome::Awaiting(demands),
                StaticDemandOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
                StaticDemandOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
                StaticDemandOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
                StaticDemandOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
            }
        }

        let outcome = {
            let mut budget = self.budget;
            crate::static_eval::StaticEvaluator::for_constant(&checked, self.values, &mut budget, self.cancel)
                .with_static_constants(self.constants)
                .with_static_definitions(self.definitions)
                .with_type_store(self.store)
                .with_semantic_tables(self.store, self.data_semantics, self.enum_semantics)
                .evaluate()
        };
        let value = match &outcome {
            crate::static_eval::StaticEvalOutcome::Reduced(value) => Some(*value),
            _ => None,
        };
        let product = Arc::new(StaticConstantProduct {
            id: id.clone(),
            declaration_range: binding.range,
            initializer_range: initializer.range(),
            checked: checked.clone(),
            outcome,
            value,
            lkg_value: value,
            diagnostics: checked.diagnostics.clone(),
        });
        let inputs = FormalQueryInputs {
            sources: self.sources,
            source_resolution_input: self.source_resolution_input,
            linked_component_product: self.linked_component_product,
            linked: self.linked,
            topology: self.topology,
            import_products: self.import_products,
            hierarchy: self.hierarchy,
            base_resolver: self.resolver,
            declarations: self.declarations,
            type_aliases: self.type_aliases,
            field_signatures: Some(self.field_signatures),
            field_lifecycle: None,
            constructor_initializations: None,
            callable_signatures: Some(self.callable_signatures),
            data_semantics: Some(self.data_semantics),
            enum_semantics: Some(self.enum_semantics),
            associated_families: Some(self.associated_families),
            type_member_declarations: None,
            conformance_semantics: None,
            module_values: None,
        };
        let published = match crate::db::query::query_static_constant(self.db, product, self.values, &inputs, self.store) {
            QueryOutcome::Ready(product) => product,
            QueryOutcome::Blocked(reason) => return StaticDemandOutcome::Blocked(reason),
            QueryOutcome::Cancelled => return StaticDemandOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return StaticDemandOutcome::BudgetExceeded(report),
            QueryOutcome::Failed(error) => return StaticDemandOutcome::Failed(error),
        };
        self.constants.insert(published.clone());
        StaticDemandOutcome::Ready(published)
    }

    fn recursive_cycle_outcome(&mut self, id: &StaticConstantId) -> QueryOutcome<Arc<StaticConstantProduct>> {
        let cycle = self.active_constants.iter().cloned().collect::<BTreeSet<_>>();
        match self.publish_recursive_cycle(&cycle) {
            QueryOutcome::Ready(()) => self
                .constants
                .get(id)
                .cloned()
                .map_or_else(|| QueryOutcome::Failed(format!("recursive constant cycle omitted {id:?}")), QueryOutcome::Ready),
            QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
            QueryOutcome::Cancelled => QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Failed(error) => QueryOutcome::Failed(error),
        }
    }

    fn publish_recursive_cycle(&mut self, cycle: &BTreeSet<StaticConstantId>) -> QueryOutcome<()> {
        for id in cycle {
            if self
                .constants
                .get(id)
                .is_some_and(|product| matches!(product.outcome, crate::static_eval::StaticEvalOutcome::Recursive(_)))
            {
                continue;
            }
            let Some(catalog) = self.catalogs.get(id.module()).cloned() else {
                return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
            };
            let Some(seed) = catalog.get(id.symbol()).cloned() else {
                return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
            };
            let Some(source) = self.sources.get(id.module()).cloned() else {
                return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
            };
            let Some(binding) = find_constant_declaration(&source.program, id.symbol(), seed.declaration_range, seed.initializer_range) else {
                return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
            };
            let Some(initializer) = binding.value.as_ref() else {
                return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
            };
            let checked = {
                let intrinsics = self.intrinsics;
                let context = demand_checking_context(
                    &mut *self.store,
                    self.hierarchy,
                    self.resolver,
                    self.declarations,
                    self.dispatch,
                    self.topology,
                    self.field_signatures,
                    self.data_semantics,
                    self.enum_semantics,
                    self.associated_families,
                    self.deferred_static_definitions,
                    id.module(),
                );
                crate::checker::static_body::analyze_static_constant_initializer(
                    context,
                    id,
                    seed.declaration_range,
                    seed.initializer_range.unwrap_or_else(|| initializer.range()),
                    initializer,
                    binding.annotation.as_ref(),
                    None,
                    &catalog,
                    &crate::session::static_constant_binding_states(&catalog, &BTreeMap::new(), None),
                    intrinsics,
                )
            };
            let mut diagnostics = checked.diagnostics.to_vec();
            diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                id.module().clone(),
                crate::diagnostic::DiagnosticCode::StaticConstantCycle,
                format!("static constant `{}` participates in a cycle through computed Record-key demand", id.name()),
                seed.declaration_range,
            ));
            let product = Arc::new(StaticConstantProduct {
                id: id.clone(),
                declaration_range: seed.declaration_range,
                initializer_range: seed.initializer_range.unwrap_or_else(|| initializer.range()),
                checked: Arc::new(checked),
                outcome: crate::static_eval::StaticEvalOutcome::Recursive(crate::static_eval::StaticCycle::empty()),
                value: None,
                lkg_value: None,
                diagnostics: Arc::from(diagnostics.into_boxed_slice()),
            });
            let inputs = FormalQueryInputs {
                sources: self.sources,
                source_resolution_input: self.source_resolution_input,
                linked_component_product: self.linked_component_product,
                linked: self.linked,
                topology: self.topology,
                import_products: self.import_products,
                hierarchy: self.hierarchy,
                base_resolver: self.resolver,
                declarations: self.declarations,
                type_aliases: self.type_aliases,
                field_signatures: Some(self.field_signatures),
                field_lifecycle: None,
                constructor_initializations: None,
                callable_signatures: Some(self.callable_signatures),
                data_semantics: Some(self.data_semantics),
                enum_semantics: Some(self.enum_semantics),
                associated_families: Some(self.associated_families),
                type_member_declarations: None,
                conformance_semantics: None,
                module_values: None,
            };
            let published = match crate::db::query::query_static_constant(self.db, product, self.values, &inputs, self.store) {
                QueryOutcome::Ready(product) => product,
                QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Failed(error) => return QueryOutcome::Failed(error),
            };
            self.constants.insert(published);
        }
        QueryOutcome::Ready(())
    }
}

fn demand_checking_context<'a>(
    store: &'a mut TypeStore,
    hierarchy: &'a dyn TypeHierarchy,
    resolver: &'a dyn TypeResolver,
    declarations: &'a DeclarationTypeTable,
    dispatch: &'a SurfaceDispatchResolver,
    topology: &'a ModuleTopology,
    field_signatures: &'a FieldSignatureTable,
    data_semantics: &'a crate::data_semantics::DataSemanticTable,
    enum_semantics: &'a crate::enum_semantics::EnumSemanticTable,
    associated_families: &'a crate::associated::AssociatedFamilyTable,
    deferred_static_definitions: &'a BTreeSet<crate::identity::DeclarationId>,
    module: &ModuleId,
) -> CheckingContext<'a> {
    let mut context = CheckingContext::new_with_dispatch_ref(store, hierarchy, resolver, declarations, dispatch, module.clone());
    context.set_caller_package(topology.owning_package(module));
    context.set_access_topology(topology);
    context.attach_field_signatures(field_signatures);
    context.attach_data_semantics(data_semantics);
    context.attach_enum_semantics(enum_semantics);
    context.attach_associated_families(associated_families);
    for declaration in deferred_static_definitions {
        context.resolver.track_deferred_static_definition(declaration.clone());
    }
    context.phase = crate::checker::context::EvaluationPhase::Static;
    context
}
