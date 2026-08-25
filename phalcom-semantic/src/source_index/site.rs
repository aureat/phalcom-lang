//! Snapshot-local source-site records.

use crate::identity::{CallableId, DeclarationId, FieldId, SourceOwner, SourceSiteId};
use phalcom_common::range::SourceRange;

/// Kind of source location recorded in a compiler source index.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceSiteKind {
    /// Module source boundary or module-level synthetic site.
    Module,
    /// Declaration site with canonical declaration identity.
    Declaration(DeclarationId),
    /// Callable declaration/body site with canonical callable identity.
    Callable(CallableId),
    /// Field declaration site with canonical field identity.
    Field(FieldId),
    /// Binding declaration site; target attachment is published separately.
    BindingDeclaration,
    /// Expression site attached to a formal expression product.
    Expression,
    /// Token occurrence site; exact target attachment is optional.
    Occurrence,
}

/// Source location identity and range metadata within one semantic snapshot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSite {
    /// Snapshot-local owner-qualified identity.
    pub id: SourceSiteId,
    /// Current source range. This is metadata, not identity.
    pub range: SourceRange,
    /// Structural kind of this site.
    pub kind: SourceSiteKind,
}

impl SourceSite {
    /// Creates a source site from an owner, local ordinal, range, and kind.
    pub fn new(owner: SourceOwner, local: crate::identity::SourceSiteLocalId, range: SourceRange, kind: SourceSiteKind) -> Self {
        Self {
            id: SourceSiteId { owner, local },
            range,
            kind,
        }
    }
}
