//! Assertion coverage for the compiler/LSP canonical parity evidence channel.

use phalcom_common::selector::SelectorBase;
use phalcom_lsp::parity::{CanonicalParityHarness, CanonicalParitySample, CanonicalParityStats, MAX_SAMPLES, ParitySurface};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath};
use phalcom_semantic::{FormalPresentation, SemanticTargetId};
use tower_lsp::lsp_types::Position;

mod support;

fn test_module() -> ModuleId {
    ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("test").unwrap()]))
}

#[test]
fn parity_harness_retains_canonical_identity_without_advisory_comparison() {
    let harness = CanonicalParityHarness::new();
    let module = test_module();
    let target = SemanticTargetId::Module(module.clone());
    let formal = FormalPresentation::Known("Int | String".into());
    harness.observe_hover(&module, &target, Some(&formal));

    assert_eq!(harness.stats().observations_total, 1);
    assert_eq!(
        harness.samples(),
        vec![CanonicalParitySample {
            surface: ParitySurface::Hover,
            module,
            target: Some(target),
            formal: Some(formal),
        }]
    );
}

#[test]
fn parity_harness_is_bounded_and_clears_as_one_snapshot() {
    let harness = CanonicalParityHarness::new();
    let module = test_module();
    let target = SemanticTargetId::Module(module.clone());
    for _ in 0..(MAX_SAMPLES + 8) {
        harness.observe_inlay_hint(&module, Some(&target), Some(&FormalPresentation::Unknown));
    }

    assert_eq!(harness.stats().retained_samples, MAX_SAMPLES);
    assert_eq!(harness.stats().observations_total, (MAX_SAMPLES + 8) as u64);
    harness.clear();
    assert_eq!(harness.stats(), CanonicalParityStats::default());
    assert!(harness.samples().is_empty());
}

#[tokio::test]
async fn live_hover_consumes_the_same_canonical_formal_product_it_publishes() {
    let root = std::env::temp_dir().join(format!("phalcom-lsp-live-parity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create parity workspace");
    std::fs::write(root.join("package.ph"), "").expect("write parity package marker");
    let root_uri = tower_lsp::lsp_types::Url::from_directory_path(&root).expect("parity workspace URI").to_string();
    let path = root.join("main.ph");
    let uri = tower_lsp::lsp_types::Url::from_file_path(&path).expect("parity source URI").to_string();
    let source = "class Point {\n  value() -> Int { 1 }\n}\n";
    let mut lsp = support::TestLsp::start().await;
    lsp.initialize(Some(&root_uri)).await;
    lsp.open_and_wait(&uri, source).await;

    let response = lsp.hover(&uri, Position::new(1, 3)).await;
    let rendered = response["result"]["contents"]["value"].as_str().unwrap_or_default();
    assert!(rendered.contains("Int"), "hover must publish the canonical formal type: {response:#?}");

    let snapshot = lsp.semantic_snapshot();
    let module = snapshot.module_for_display_path(&path).expect("main module should be canonicalized").clone();
    let sample = lsp
        .parity_observations()
        .into_iter()
        .find(|sample| sample.surface == ParitySurface::Hover)
        .expect("production hover path must feed canonical parity evidence");
    assert_eq!(sample.module, module);
    let Some(SemanticTargetId::Callable(callable)) = sample.target.as_ref() else {
        panic!("hover must retain the callable target identity: {sample:#?}");
    };
    assert_eq!(callable.selector.base, SelectorBase::Named("value".into()));
    let signature = snapshot
        .callable_signatures()
        .get(callable)
        .expect("the sampled callable must have a canonical signature");
    assert_eq!(&signature.callable, callable);
    let expected_formal = phalcom_semantic::TypePresenter::new(&snapshot.store).present_knowledge(&signature.published_return_knowledge());
    assert_eq!(sample.formal, Some(expected_formal));
    assert_eq!(lsp.parity_stats().observations_total, 1);

    lsp.finish().await;
    let _ = std::fs::remove_dir_all(root);
}
