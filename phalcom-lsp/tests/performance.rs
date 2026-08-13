use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde_json::json;
use tower_lsp::lsp_types::{Position, Url};

use crate::support::TestLsp;

static NEXT_PERF_ROOT: AtomicU64 = AtomicU64::new(1);

fn perf_root(label: &str) -> PathBuf {
    let id = NEXT_PERF_ROOT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("phalcom-lsp-perf-{label}-{}-{id}", std::process::id()))
}

fn populate_workspace(root: &Path, files: usize) -> String {
    fs::create_dir_all(root).expect("create perf workspace");
    fs::write(root.join("leaf.ph"), "class Leaf { value() { 1 } }\n").expect("write leaf");
    fs::write(root.join("dependent.ph"), "class Dependent { value() { 2 } }\n").expect("write dependent");
    fs::write(root.join("surface.ph"), "class Surface { old() { 3 } }\n").expect("write surface");
    for index in 0..files {
        fs::write(
            root.join(format!("scan-{index}.ph")),
            format!("class Scan{index} {{ marker() {{ {index} }} }}\n"),
        )
        .expect("write scan file");
    }
    Url::from_directory_path(root).expect("perf workspace URL").to_string()
}

fn elapsed_ms(start: Instant) -> u128 {
    start.elapsed().as_millis()
}

#[tokio::test]
#[ignore = "performance measurement harness"]
async fn perf_local_and_workspace_convergence() {
    let root = perf_root("modes");
    let root_uri = populate_workspace(&root, 32);
    let leaf_uri = Url::from_file_path(root.join("leaf.ph")).unwrap().to_string();
    let surface_uri = Url::from_file_path(root.join("surface.ph")).unwrap().to_string();

    let construction_start = Instant::now();
    let mut local = TestLsp::start().await;
    let construction_ms = elapsed_ms(construction_start);
    let initialize_start = Instant::now();
    let init = local.initialize(Some(&root_uri)).await;
    let initialize_ms = elapsed_ms(initialize_start);
    assert!(init["result"]["capabilities"].is_object());

    let open_start = Instant::now();
    let before_open = local.counter_snapshot();
    local.open(&leaf_uri, "class Leaf { value() { 1 } }\n").await;
    let shallow_open_ms = elapsed_ms(open_start);
    let local_inlay_start = Instant::now();
    let _ = local.inlay_hints(&leaf_uri, 20).await;
    local.wait_for_publication_after(before_open).await;
    let local_convergence_ms = elapsed_ms(local_inlay_start);

    let leaf_edit_start = Instant::now();
    local.change(&leaf_uri, "class Leaf { value() { 2 } }\n").await;
    let leaf_edit_ms = elapsed_ms(leaf_edit_start);
    let dependent_edit_start = Instant::now();
    local.change(&leaf_uri, "class Leaf { value() { \"changed\" } }\n").await;
    let dependent_edit_ms = elapsed_ms(dependent_edit_start);
    let class_edit_start = Instant::now();
    local.change(&surface_uri, "class Surface { newer() { 4 } }\n").await;
    let class_edit_ms = elapsed_ms(class_edit_start);
    let before_rapid_edits = local.counter_snapshot();
    for edit in 0..20 {
        local.change(&leaf_uri, &format!("class Leaf {{ value() {{ {edit} }} }}\n")).await;
    }
    local.wait_for_publication_after(before_rapid_edits).await;
    let hover = local.hover(&leaf_uri, Position::new(0, 7)).await;
    assert!(hover.get("result").is_some());
    println!(
        "perf_local construction_ms={construction_ms} initialize_ms={initialize_ms} shallow_open_ms={shallow_open_ms} convergence_ms={local_convergence_ms} leaf_edit_ms={leaf_edit_ms} dependent_edit_ms={dependent_edit_ms} class_edit_ms={class_edit_ms} counters={:?}",
        local.counter_snapshot()
    );
    local.finish().await;

    let workspace_start = Instant::now();
    let mut workspace = TestLsp::start().await;
    workspace
        .initialize_with_options(Some(&root_uri), json!({ "phalcom": { "analysis": { "mode": "workspace" } } }))
        .await;
    let before_workspace_open = workspace.counter_snapshot();
    workspace.open(&leaf_uri, "class Leaf { value() { 1 } }\n").await;
    let _ = workspace.inlay_hints(&leaf_uri, 20).await;
    workspace.wait_for_publication_after(before_workspace_open).await;
    println!(
        "perf_workspace convergence_ms={} counters={:?}",
        elapsed_ms(workspace_start),
        workspace.counter_snapshot()
    );
    workspace.finish().await;

    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
#[ignore = "performance measurement harness"]
async fn perf_hover_during_progressive_scan() {
    let root = perf_root("busy");
    let root_uri = populate_workspace(&root, 256);
    let leaf_uri = Url::from_file_path(root.join("leaf.ph")).unwrap().to_string();

    let mut lsp = TestLsp::start().await;
    let initialize_start = Instant::now();
    lsp.initialize(Some(&root_uri)).await;
    lsp.open(&leaf_uri, "class Leaf { value() { 1 } }\n").await;
    let hover_start = Instant::now();
    let response = lsp.hover(&leaf_uri, Position::new(0, 7)).await;
    assert!(response.get("result").is_some());
    println!(
        "perf_busy initialize_to_hover_ms={} hover_ms={} counters={:?}",
        elapsed_ms(initialize_start),
        elapsed_ms(hover_start),
        lsp.counter_snapshot()
    );
    lsp.finish().await;
    let _ = fs::remove_dir_all(root);
}
