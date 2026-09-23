//! Declaration-owned identities and structural catalogs for static cases.

use crate::identity::{StaticCaseId, StaticDomainDeclarationId};
use phalcom_ast::ast::{AlgebraicDef, AlgebraicShapeSyntax, CasePayloadSyntax};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use crate::types::id::{StaticDomainId, TypeId};
use crate::types::store::{StaticArgument, TypeStore};

/// Source-known case shape, independent of checked payload kinds.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticCaseShape {
    Singleton,
    Constructor { slots: Box<[SelectorSlot]> },
}

/// One stable case declaration published before payload validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticCaseHeader {
    pub id: StaticCaseId,
    pub shape: StaticCaseShape,
    pub source_range: SourceRange,
}

/// The closed case set for a nominal or associated static domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticDomainCaseCatalogProduct {
    pub domain: StaticDomainDeclarationId,
    pub cases: Box<[StaticCaseHeader]>,
}

impl StaticDomainCaseCatalogProduct {
    /// Selects a case by canonical selector, including getter versus constructor shape.
    pub fn select(&self, selector: &Selector) -> Option<&StaticCaseHeader> {
        self.cases.iter().find(|case| &case.id.selector == selector)
    }
}

/// A case name absent from the domain, or present only with another selector shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticCaseSelectionError {
    Missing,
    WrongShape,
    Ambiguous,
}

/// Selects through declaration membership rather than store interning history.
pub fn select_static_case<'a>(
    catalog: &'a StaticDomainCaseCatalogProduct,
    selector: &Selector,
) -> Result<&'a StaticCaseHeader, StaticCaseSelectionError> {
    let mut matches = catalog.cases.iter().filter(|case| &case.id.selector == selector);
    if let Some(case) = matches.next() {
        return if matches.next().is_some() { Err(StaticCaseSelectionError::Ambiguous) } else { Ok(case) };
    }
    if catalog.cases.iter().any(|case| case.id.selector.base == selector.base) {
        Err(StaticCaseSelectionError::WrongShape)
    } else {
        Err(StaticCaseSelectionError::Missing)
    }
}

/// Interns an already selected singleton static term.
pub fn materialize_static_singleton(store: &mut TypeStore, domain: StaticDomainId, case: &StaticCaseHeader) -> Option<TypeId> {
    if case.shape != StaticCaseShape::Singleton {
        return None;
    }
    let owner = match &case.id.domain {
        StaticDomainDeclarationId::Nominal(owner) => owner.clone(),
        StaticDomainDeclarationId::Associated(declaration) => declaration.owner.clone(),
    };
    let constructor = store.intern_static_constructor(domain, owner, case.id.selector.encode(), Box::new([]));
    store.static_data(domain, constructor, Box::<[StaticArgument]>::default()).ok()
}

/// Builds declaration identity and membership using only structural source facts.
pub fn catalog_from_declaration(domain: StaticDomainDeclarationId, def: &AlgebraicDef) -> StaticDomainCaseCatalogProduct {
    let cases = match &def.shape {
        AlgebraicShapeSyntax::Sum(sum) => sum
            .cases
            .iter()
            .map(|case| {
                let selector = phalcom_ast::selector::selector_from_variant(case);
                let shape = match &case.payload {
                    CasePayloadSyntax::Singleton => StaticCaseShape::Singleton,
                    CasePayloadSyntax::Tuple { .. } | CasePayloadSyntax::Record { .. } => {
                        StaticCaseShape::Constructor { slots: selector.slots.clone() }
                    }
                };
                StaticCaseHeader {
                    id: StaticCaseId::with_selector(domain.clone(), selector),
                    shape,
                    source_range: case.name_range,
                }
            })
            .collect(),
        AlgebraicShapeSyntax::Product(product) => {
            let slots = product.components().iter().map(|component| match &component.external_label {
                Some(label) => SelectorSlot::Label(label.clone()),
                None => SelectorSlot::Positional,
            }).collect::<Vec<_>>();
            let selector = Selector::method(&def.name, slots).unwrap_or_else(|_| Selector {
                base: phalcom_common::selector::SelectorBase::Named(def.name.clone()),
                kind: phalcom_common::selector::SelectorKind::Method,
                slots: Box::new([]),
            });
            let shape = StaticCaseShape::Constructor { slots: selector.slots.clone() };
            vec![StaticCaseHeader { id: StaticCaseId::with_selector(domain.clone(), selector), shape, source_range: def.name_range }]
        }
    };
    StaticDomainCaseCatalogProduct { domain, cases: cases.into_boxed_slice() }
}
