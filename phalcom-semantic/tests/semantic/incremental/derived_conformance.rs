//! Incremental invalidation and cross-module publication for guarded derived conformance rules (Spec 03.11 / LANG005.C6.P4 / CV-53..60).

use phalcom_common::selector::SelectorBase;
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::checker::analysis::CallableAnalysisStatus;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::snapshot::SemanticSnapshot;
use std::sync::Arc;

fn location(path: &std::path::Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

#[test]
fn generic_conformance_stable_callable_is_source_owned_and_matches_cold_reconstruction() {
    use phalcom_semantic::identity::CallableOwnerId;
    use phalcom_semantic::metadata::stable_identity::{StableIdentityContext, to_stable_callable_with_context};
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let path = root.path().join("provider.ph");
    let source = location(&path);
    let text = r#"
trait Value { value -> Int }
class Box<T> {}
impl<T> Value for Box<T> { value -> Int { 7 } }
class Consumer {
  @class integers(_ box: Box<Int>) -> Int { box.value }
  @class strings(_ box: Box<String>) -> Int { box.value }
}
"#;
    let mutation = |revision, text: &str| WorkspaceSourceBatchMutation::SetOverlay {
        source: source.clone(),
        text: Arc::from(text),
        revision: SourceRevision(revision),
        recovered_program: None,
    };
    let witnesses = |snapshot: &SemanticSnapshot| {
        snapshot
            .callable_signatures
            .iter()
            .map(|(callable, _)| callable)
            .filter(|callable| {
                matches!(callable.owner, CallableOwnerId::Conformance(_)) && callable.owner.module() == snapshot.module_for_display_path(&path).unwrap()
            })
            .map(|callable| to_stable_callable_with_context(callable, &StableIdentityContext::new(&snapshot.module_products.universe)))
            .collect::<Vec<_>>()
    };
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.apply_module_mutations([mutation(1, text)]).unwrap();
    assert!(!initial.snapshot.has_errors(), "{:?}", initial.snapshot.diagnostics);
    let before = witnesses(&initial.snapshot);
    assert_eq!(before.len(), 1, "both exact Box applications share one source witness");
    let edited = format!("{text}\n// source-only edit\n");
    let incremental = session.apply_module_mutations([mutation(2, &edited)]).unwrap();
    let cold = SemanticWorkspaceSession::new().apply_module_mutations([mutation(2, &edited)]).unwrap();
    assert!(!incremental.snapshot.has_errors(), "{:?}", incremental.snapshot.diagnostics);
    assert!(!cold.snapshot.has_errors(), "{:?}", cold.snapshot.diagnostics);
    assert_eq!(before, witnesses(&incremental.snapshot));
    assert_eq!(witnesses(&incremental.snapshot), witnesses(&cold.snapshot));
}

fn callable_id(snapshot: &SemanticSnapshot, module: &phalcom_modules::ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId {
    let owner = DeclarationId::new(module.clone(), owner.into());
    snapshot
        .callable_signatures
        .iter()
        .map(|(callable, _)| callable)
        .find(|callable| {
            callable.owner == owner && callable.side == side && matches!(&callable.selector.base, SelectorBase::Named(base) if base.as_str() == name)
        })
        .cloned()
        .unwrap_or_else(|| panic!("missing callable {owner:?}.{name} on {side:?}"))
}

#[test]
fn cross_module_guarded_rule_proves_downstream_goal() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let provider_loc = location(&provider_path);
    let consumer_loc = location(&consumer_path);

    let provider_src = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Stream
export Counter
"#;

    let consumer_src = r#"
from .provider import Stream, Counter
class Consumer {
  @class consume<S>(_ s: S) -> Int where S |= Stream {
    s.next_item()
  }
  @class run(_ c: Counter) -> Int {
    Consumer.consume(c)
  }
}
export Consumer
"#;

    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial session");

    assert!(!result.snapshot.has_errors(), "diagnostics: {:?}", result.snapshot.diagnostics);

    let provider_module = result.snapshot.module_for_display_path(&provider_path).expect("provider module").clone();
    let consumer_module = result.snapshot.module_for_display_path(&consumer_path).expect("consumer module").clone();

    let stream_decl = DeclarationId::new(provider_module, "Stream".into());
    let rules = result.snapshot.derived_conformance_rules.rules_for_trait(&stream_decl);
    assert_eq!(rules.len(), 1, "provider derived rule must be published in snapshot");

    let run_fn = callable_id(&result.snapshot, &consumer_module, "Consumer", "run", DispatchSide::Class);
    let run_analysis = result.snapshot.callable_analyses.get(&run_fn).expect("run analysis");
    assert_eq!(run_analysis.status, CallableAnalysisStatus::Complete);
}

#[test]
fn cross_module_guarded_rule_runtime_products_are_publishable() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let provider_loc = location(&provider_path);

    let provider_src = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
export Stepper
export Stream
"#;

    let mut session = SemanticWorkspaceSession::new();
    let result = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("initial session");

    let provider_module = result.snapshot.module_for_display_path(&provider_path).expect("provider module").clone();
    let stream_decl = DeclarationId::new(provider_module.clone(), "Stream".into());
    let rules = result.snapshot.derived_conformance_rules.rules_for_trait(&stream_decl);
    assert_eq!(rules.len(), 1);
    let rule = &rules[0];
    assert_eq!(rule.source_module, provider_module);
    assert_eq!(rule.generic_signature.parameters.len(), 1);
    assert_eq!(rule.associated_type_plan.bindings.len(), 1);
}

#[test]
fn antecedent_conformance_removal_invalidates_derived_consumer() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let provider_loc = location(&provider_path);
    let consumer_loc = location(&consumer_path);

    let provider_src_a = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Stream
export Counter
"#;

    let consumer_src = r#"
from .provider import Stream, Counter
class Consumer {
  @class consume<S>(_ s: S) -> Int where S |= Stream {
    s.next_item()
  }
  @class run(_ c: Counter) -> Int {
    Consumer.consume(c)
  }
}
export Consumer
"#;

    let mut session = SemanticWorkspaceSession::new();
    session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src_a),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("session 1");
    // Remove `impl Stepper for Counter`
    let provider_src_b = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
export Stepper
export Stream
export Counter
"#;

    let out_b = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_b),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("session 2");
    assert!(
        out_b.snapshot.has_errors(),
        "consumer should fail when antecedent conformance is removed; diags: {:?}",
        out_b.snapshot.diagnostics
    );
}

#[test]
fn antecedent_conformance_readd_restores_derived_consumer() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let provider_loc = location(&provider_path);
    let consumer_loc = location(&consumer_path);

    let provider_src_a = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
export Stepper
export Stream
export Counter
"#;

    let consumer_src = r#"
from .provider import Stream, Counter
class Consumer {
  @class consume<S>(_ s: S) -> Int where S |= Stream {
    s.next_item()
  }
  @class run(_ c: Counter) -> Int {
    Consumer.consume(c)
  }
}
export Consumer
"#;

    let mut session = SemanticWorkspaceSession::new();
    let out_a = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src_a),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("session 1");
    assert!(out_a.snapshot.has_errors());

    // Re-add `impl Stepper for Counter`
    let provider_src_b = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Stream
export Counter
"#;

    let out_b = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_b),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("session 2");
    assert!(
        !out_b.snapshot.has_errors(),
        "re-adding antecedent should restore consumer: {:?}",
        out_b.snapshot.diagnostics
    );
}

#[test]
fn derived_rule_condition_edit_invalidates_consumer() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let provider_loc = location(&provider_path);
    let consumer_loc = location(&consumer_path);

    let provider_src_a = r#"
trait Stepper {
  step() -> Int
}
trait Extra {}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Extra
export Stream
export Counter
"#;

    let consumer_src = r#"
from .provider import Stream, Counter
class Consumer {
  @class consume<S>(_ s: S) -> Int where S |= Stream {
    s.next_item()
  }
  @class run(_ c: Counter) -> Int {
    Consumer.consume(c)
  }
}
export Consumer
"#;

    let mut session = SemanticWorkspaceSession::new();
    let out_a = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src_a),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("session 1");
    assert!(!out_a.snapshot.has_errors(), "init diagnostics: {:?}", out_a.snapshot.diagnostics);

    // Tighten rule to require Extra
    let provider_src_b = r#"
trait Stepper {
  step() -> Int
}
trait Extra {}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper, I |= Extra {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Extra
export Stream
export Counter
"#;

    let out_b = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_b),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("session 2");
    assert!(
        out_b.snapshot.has_errors(),
        "consumer should fail when rule condition is tightened to require un-satisfied Extra"
    );
}

#[test]
fn derived_rule_constraint_reorder_is_semantic_noop() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let provider_loc = location(&provider_path);

    let provider_src_a = r#"
trait Stepper {
  step() -> Int
}
trait Extra {}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper, I |= Extra {
  type Element = Int
  next_item() -> Int { self.step() }
}
export Stepper
export Extra
export Stream
"#;

    let provider_src_b = r#"
trait Stepper {
  step() -> Int
}
trait Extra {}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Extra, I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
export Stepper
export Extra
export Stream
"#;

    let mut session_a = SemanticWorkspaceSession::new();
    let out_a = session_a
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_a),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("session A");

    let mut session_b = SemanticWorkspaceSession::new();
    let out_b = session_b
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_b),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("session B");

    let provider_module = out_a.snapshot.module_for_display_path(&provider_path).expect("provider module").clone();
    let stream_decl = DeclarationId::new(provider_module, "Stream".into());
    let rule_a = &out_a.snapshot.derived_conformance_rules.rules_for_trait(&stream_decl)[0];
    let rule_b = &out_b.snapshot.derived_conformance_rules.rules_for_trait(&stream_decl)[0];

    assert_eq!(
        rule_a.fingerprint, rule_b.fingerprint,
        "derived rule fingerprint must be identical across constraint reorders"
    );
}

#[test]
fn trait_contract_edit_invalidates_derived_evidence() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let provider_loc = location(&provider_path);
    let consumer_loc = location(&consumer_path);

    let provider_src_a = r#"
trait Stepper {
  step() -> Int
}
trait Bound {}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Bound
export Stream
export Counter
"#;

    let consumer_src = r#"
from .provider import Stream, Counter
class Consumer {
  @class consume<S>(_ s: S) -> Int where S |= Stream {
    s.next_item()
  }
  @class run(_ c: Counter) -> Int {
    Consumer.consume(c)
  }
}
export Consumer
"#;

    let mut session = SemanticWorkspaceSession::new();
    let out_a = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src_a),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("session 1");
    assert!(!out_a.snapshot.has_errors(), "init diagnostics: {:?}", out_a.snapshot.diagnostics);

    // Edit Stream trait contract to require `where Self::Element |= Bound`
    let provider_src_b = r#"
trait Stepper {
  step() -> Int
}
trait Bound {}
trait Stream where Self::Element |= Bound {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Bound
export Stream
export Counter
"#;

    let out_b = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_b),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("session 2");
    assert!(
        out_b.snapshot.has_errors(),
        "editing Stream trait contract to require Bound on Element should invalidate derived conformance when Int does not conform to Bound; diags: {:?}",
        out_b.snapshot.diagnostics
    );
}

#[test]
fn derived_rule_incremental_matches_cold() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let provider_loc = location(&provider_path);
    let consumer_loc = location(&consumer_path);

    let provider_src_1 = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 42 }
}
impl Stepper for Counter {}
export Stepper
export Stream
export Counter
"#;

    let provider_src_2 = r#"
trait Stepper {
  step() -> Int
}
trait Stream {
  type Element
  next_item() -> Int
}
impl<I> Stream for I where I |= Stepper {
  type Element = Int
  next_item() -> Int { self.step() }
}
class Counter {
  step() -> Int { 100 }
}
impl Stepper for Counter {}
export Stepper
export Stream
export Counter
"#;

    let consumer_src = r#"
from .provider import Stream, Counter
class Consumer {
  @class consume<S>(_ s: S) -> Int where S |= Stream {
    s.next_item()
  }
  @class run(_ c: Counter) -> Int {
    Consumer.consume(c)
  }
}
export Consumer
"#;

    let mut session = SemanticWorkspaceSession::new();
    let _ = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src_1),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("session 1");

    let inc_out = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider_loc.clone(),
            text: Arc::from(provider_src_2),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("incremental session 2");

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold_out = cold_session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider_loc.clone(),
                text: Arc::from(provider_src_2),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer_loc.clone(),
                text: Arc::from(consumer_src),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("cold session");

    assert_eq!(inc_out.snapshot.has_errors(), cold_out.snapshot.has_errors());
    let provider_module = inc_out.snapshot.module_for_display_path(&provider_path).expect("provider module").clone();
    let stream_decl = DeclarationId::new(provider_module, "Stream".into());
    let inc_rules = inc_out.snapshot.derived_conformance_rules.rules_for_trait(&stream_decl);
    let cold_rules = cold_out.snapshot.derived_conformance_rules.rules_for_trait(&stream_decl);
    assert_eq!(inc_rules.len(), cold_rules.len());
    assert_eq!(inc_rules[0].fingerprint, cold_rules[0].fingerprint);
}
