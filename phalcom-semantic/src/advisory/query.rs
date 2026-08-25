//! Read-only advisory query facade over one immutable workspace product.

use crate::identity::{CallableId, FieldId, SourceSiteId};

use super::{AdvisoryCallableSummary, AdvisoryFact, AdvisoryParameterSlot, AdvisoryTargetResolution, AdvisoryWorkspace};

#[derive(Clone, Copy, Debug)]
pub struct AdvisoryQuery<'a> {
    workspace: &'a AdvisoryWorkspace,
}

impl<'a> AdvisoryQuery<'a> {
    pub fn new(workspace: &'a AdvisoryWorkspace) -> Self {
        Self { workspace }
    }

    pub fn expression(&self, site: &SourceSiteId) -> Option<&'a AdvisoryFact> {
        self.workspace.expression(site)
    }

    pub fn binding(&self, site: &SourceSiteId) -> Option<&'a AdvisoryFact> {
        self.workspace.binding(site)
    }

    pub fn field(&self, field: &FieldId) -> Option<&'a AdvisoryFact> {
        self.workspace.field(field)
    }

    pub fn parameter(&self, slot: &AdvisoryParameterSlot) -> Option<&'a AdvisoryFact> {
        self.workspace.parameter(slot)
    }

    pub fn callable(&self, callable: &CallableId) -> Option<&'a AdvisoryCallableSummary> {
        self.workspace.callable(callable)
    }

    pub fn target(&self, site: &SourceSiteId) -> Option<&'a AdvisoryTargetResolution> {
        self.workspace.target(site)
    }
}
