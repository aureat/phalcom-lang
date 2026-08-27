//! Protocol-owned publication of the canonical compiler semantic snapshot.

use std::sync::{Arc, RwLock};

/// Immutable canonical semantic snapshot currently visible to LSP requests.
///
/// This cell deliberately contains publication state only. Semantic queries,
/// identity translation, invalidation, and mutation remain owned by the
/// compiler semantic workspace.
#[derive(Default)]
pub(crate) struct SemanticPublication {
    current: RwLock<Option<Arc<phalcom_semantic::SemanticSnapshot>>>,
}

/// Read-only handle used by protocol scheduling and tests to observe whether
/// an exact source document has reached the canonical semantic publication.
///
/// This handle exposes no semantic lookup or mutation operations. Feature
/// requests continue to query the immutable compiler snapshot through their
/// normal request context.
#[derive(Clone)]
pub struct SemanticPublicationHandle {
    publication: Arc<SemanticPublication>,
}

impl SemanticPublicationHandle {
    pub(crate) fn new(publication: Arc<SemanticPublication>) -> Self {
        Self { publication }
    }

    /// Returns whether the latest canonical publication contains `text` for
    /// the already-produced display path `path`.
    ///
    /// The comparison is exact and performs no filesystem reads or path
    /// canonicalization.
    pub fn contains_exact_source(&self, path: &std::path::Path, text: &str) -> bool {
        let Some(snapshot) = self.publication.load() else {
            return false;
        };
        let Some(module) = snapshot.module_for_display_path(path) else {
            return false;
        };
        snapshot.sources.get(module).is_some_and(|source| source.text.as_ref() == text)
    }
}

impl SemanticPublication {
    /// Creates an empty publication cell.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Loads the latest published snapshot, if the worker has published one.
    pub(crate) fn load(&self) -> Option<Arc<phalcom_semantic::SemanticSnapshot>> {
        self.current.read().expect("semantic publication lock poisoned").clone()
    }

    /// Publishes one immutable canonical snapshot for future requests.
    pub(crate) fn publish(&self, snapshot: Arc<phalcom_semantic::SemanticSnapshot>) {
        *self.current.write().expect("semantic publication lock poisoned") = Some(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::SemanticPublication;
    use std::sync::Arc;

    fn snapshot(generation: u64) -> Arc<phalcom_semantic::SemanticSnapshot> {
        Arc::new(phalcom_semantic::SemanticSnapshot::new(
            phalcom_semantic::WorkspaceId::from_raw(1),
            phalcom_semantic::SemanticRevision::from_raw(generation),
            generation,
            Arc::new(phalcom_semantic::TypeStore::new()),
            Arc::new(Default::default()),
            Arc::new(Default::default()),
            Arc::new(phalcom_semantic::SurfaceDispatchResolver::new()),
            Arc::new(phalcom_semantic::CallableSignatureTable::new()),
            Arc::new(phalcom_semantic::DeclarationTypeTable::new()),
            Arc::new(phalcom_semantic::MapTypeHierarchy::new()),
            Arc::new(Default::default()),
            Arc::new(phalcom_modules::graph::SemanticGraph::default()),
        ))
    }

    #[test]
    fn publication_cell_pins_exact_canonical_arc() {
        let publication = SemanticPublication::new();
        assert!(publication.load().is_none());

        let snapshot_a = snapshot(1);
        publication.publish(snapshot_a.clone());
        let loaded_a = publication.load().expect("snapshot A should be published");
        assert!(Arc::ptr_eq(&loaded_a, &snapshot_a));

        let snapshot_b = snapshot(2);
        publication.publish(snapshot_b.clone());

        // Existing request state remains pinned to A after B is published.
        assert!(Arc::ptr_eq(&loaded_a, &snapshot_a));
        let loaded_b = publication.load().expect("snapshot B should be published");
        assert!(Arc::ptr_eq(&loaded_b, &snapshot_b));
    }
}
