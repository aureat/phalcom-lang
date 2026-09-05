//! Match result knowledge joining and compatibility definitions (Part 05.1).

use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
use crate::types::store::TypeStore;

/// Joins value results from reachable arms that complete normally. If no such
/// arm exists, the match expression cannot complete normally and has `Never`.
pub(crate) fn join_match_result_knowledge(store: &mut TypeStore, normal_branch_types: Vec<TypeKnowledge>) -> TypeKnowledge {
    if normal_branch_types.is_empty() {
        TypeKnowledge::established(store.never(), EvidenceOrigin::Flow)
    } else {
        crate::types::evidence::join_type_knowledge(store, normal_branch_types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::store::TypeData;

    #[test]
    fn match_result_without_normal_branch_is_never() {
        let mut store = TypeStore::new();
        let result = join_match_result_knowledge(&mut store, Vec::new());

        assert_eq!(result.ty(), Some(store.never()));
        assert!(matches!(store.get(store.never()), TypeData::Never));
    }

    #[test]
    fn match_result_with_normal_branches_joins_their_types() {
        let mut store = TypeStore::new();
        let unit = store.unit();
        let result = join_match_result_knowledge(
            &mut store,
            vec![
                TypeKnowledge::established(unit, EvidenceOrigin::Flow),
                TypeKnowledge::established(unit, EvidenceOrigin::Flow),
            ],
        );

        assert_eq!(result.ty(), Some(unit));
    }
}
