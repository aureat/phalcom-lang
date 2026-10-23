//! Deterministic dependency ordering for an immutable set of package artifacts.
//!
//! This is a prepublication admission step, not native semantic composition.
//! A successful order still requires every portable type, static value, owner,
//! query and module product to relocate into ONE receiving generation.

use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::StableProjectRef;

/// Borrowed package dependency evidence from validated artifact module worlds.
/// Every dependency is an exact persistent project identity, never a name.
pub(crate) struct PackageDependencyNode<'a> {
    pub owner: &'a StableProjectRef,
    pub external_projects: &'a [StableProjectRef],
}

fn invalid(message: &'static str) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

/// Stable Kahn-style topological order. Ties are broken by exact stable owner
/// identity rather than by input index. All scratch allocations are charged.
pub(crate) fn plan_package_dependencies(
    nodes: &[PackageDependencyNode<'_>],
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<Vec<usize>, ArtifactError> {
    if nodes.len() > limits.max_modules {
        return Err(invalid("external artifact dependency set exceeds module limit"));
    }
    for (index, node) in nodes.iter().enumerate() {
        if !matches!(node.owner, StableProjectRef::Package { .. }) {
            return Err(invalid("artifact dependency set contains a non-package owner"));
        }
        for previous in &nodes[..index] {
            if previous.owner == node.owner {
                return Err(invalid("duplicate exact package owner in artifact dependency set"));
            }
            if let (
                StableProjectRef::Package { package: a, version: av, .. },
                StableProjectRef::Package { package: b, version: bv, .. },
            ) = (previous.owner, node.owner)
            {
                if a == b && av == bv {
                    return Err(invalid("conflicting artifact fingerprints for one package name and version"));
                }
            }
        }
        for (dep_index, dependency) in node.external_projects.iter().enumerate() {
            if node.external_projects[..dep_index].contains(dependency) {
                return Err(invalid("duplicate external project dependency in package artifact"));
            }
            match dependency {
                StableProjectRef::Package { .. } => {
                    if !nodes.iter().any(|other| other.owner == dependency) {
                        return Err(invalid("package dependency is missing its exact artifact authority"));
                    }
                }
                StableProjectRef::Builtin { namespace, version }
                    if namespace.as_ref() == "universe" && version.as_ref() == "0.1.0" => {}
                StableProjectRef::Session { .. } | StableProjectRef::SourceArtifact { .. } | StableProjectRef::Builtin { .. } => {
                    return Err(invalid("unsupported external project authority in artifact-only dependency plan"));
                }
            }
        }
    }

    let mut completed = allocations.vector::<bool>(nodes.len()).map_err(ArtifactError::Allocation)?;
    completed.resize(nodes.len(), false);
    let mut order = allocations.vector::<usize>(nodes.len()).map_err(ArtifactError::Allocation)?;
    for _ in 0..nodes.len() {
        let next = (0..nodes.len())
            .filter(|&candidate| !completed[candidate])
            .filter(|&candidate| {
                nodes[candidate].external_projects.iter().all(|dependency| {
                    match dependency {
                        StableProjectRef::Package { .. } => nodes
                            .iter()
                            .enumerate()
                            .any(|(index, node)| node.owner == dependency && completed[index]),
                        StableProjectRef::Builtin { namespace, version }
                            if namespace.as_ref() == "universe" && version.as_ref() == "0.1.0" => true,
                        _ => false,
                    }
                })
            })
            .min_by(|&a, &b| nodes[a].owner.cmp(nodes[b].owner));
        let index = next.ok_or_else(|| invalid("cyclic package artifact dependencies"))?;
        completed[index] = true;
        order.push(index);
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_type_meta::fingerprint::Fingerprint128;

    fn package(name: &str, seed: u8) -> StableProjectRef {
        StableProjectRef::Package {
            package: name.into(),
            version: "1.0.0".into(),
            artifact_fingerprint: Fingerprint128([seed; 16]),
        }
    }

    #[test]
    fn dependencies_first_with_stable_owner_tiebreak() {
        let a = package("a", 1);
        let b = package("b", 2);
        let c = package("c", 3);
        let a_deps = [b.clone()];
        let b_deps = [c.clone()];
        let nodes = [
            PackageDependencyNode { owner: &a, external_projects: &a_deps },
            PackageDependencyNode { owner: &b, external_projects: &b_deps },
            PackageDependencyNode { owner: &c, external_projects: &[] },
        ];
        let mut budget = AllocationBudget::new(64 * 1024);
        assert_eq!(plan_package_dependencies(&nodes, &ValidationLimits::default(), &mut budget).unwrap(), vec![2, 1, 0]);
        let independent = [
            PackageDependencyNode { owner: &b, external_projects: &[] },
            PackageDependencyNode { owner: &a, external_projects: &[] },
        ];
        assert_eq!(plan_package_dependencies(&independent, &ValidationLimits::default(), &mut budget).unwrap(), vec![1, 0]);
    }

    #[test]
    fn rejects_missing_exact_dependencies_duplicates_and_cycles() {
        let a = package("a", 1);
        let b = package("b", 2);
        let a_deps = [b.clone()];
        let b_deps = [a.clone()];
        let mut budget = AllocationBudget::new(64 * 1024);
        let limits = ValidationLimits::default();
        let missing = [PackageDependencyNode { owner: &a, external_projects: &a_deps }];
        assert!(plan_package_dependencies(&missing, &limits, &mut budget).is_err());
        let duplicate = [
            PackageDependencyNode { owner: &a, external_projects: &[] },
            PackageDependencyNode { owner: &a, external_projects: &[] },
        ];
        assert!(plan_package_dependencies(&duplicate, &limits, &mut budget).is_err());
        let cycle = [
            PackageDependencyNode { owner: &a, external_projects: &a_deps },
            PackageDependencyNode { owner: &b, external_projects: &b_deps },
        ];
        assert!(plan_package_dependencies(&cycle, &limits, &mut budget).is_err());
    }

    #[test]
    fn rejects_untrusted_source_only_dependency() {
        let a = package("a", 1);
        let source = StableProjectRef::SourceArtifact {
            logical_uri: "file:///untrusted".into(),
            source_fingerprint: Fingerprint128([9; 16]),
        };
        let deps = [source];
        let nodes = [PackageDependencyNode { owner: &a, external_projects: &deps }];
        let mut budget = AllocationBudget::new(64 * 1024);
        assert!(plan_package_dependencies(&nodes, &ValidationLimits::default(), &mut budget).is_err());
    }
}
