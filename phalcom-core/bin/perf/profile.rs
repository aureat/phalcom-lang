use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::Serialize;
use telos_observability::{PerfCounter, PerfPhase, PerfReport, PerfScope};

use crate::store;

#[derive(Parser, Debug)]
pub struct ProfileArgs {
    #[command(subcommand)]
    command: ProfileCommands,
}

#[derive(Subcommand, Debug)]
enum ProfileCommands {
    /// Attach a JSON report emitted by an instrumented process to a stored run
    Attach {
        run_ref: String,
        #[arg(long)]
        report: PathBuf,
    },
    /// Show phase, counter, query, and derived summaries for an attached profile
    Show {
        run_ref: String,
        #[arg(long, default_value_t = 20)]
        top: usize,
        #[arg(long)]
        json: bool,
    },
    /// Compare two attached profiles by phase time, counters, and query lifecycle
    Compare {
        baseline_ref: String,
        candidate_ref: String,
        #[arg(long, default_value_t = 20)]
        top: usize,
        #[arg(long)]
        json: bool,
    },
}

pub fn handle(workspace_root: &std::path::Path, args: ProfileArgs) {
    let result = match args.command {
        ProfileCommands::Attach { run_ref, report } => attach(workspace_root, &run_ref, &report),
        ProfileCommands::Show { run_ref, top, json } => show(workspace_root, &run_ref, top, json),
        ProfileCommands::Compare {
            baseline_ref,
            candidate_ref,
            top,
            json,
        } => compare(workspace_root, &baseline_ref, &candidate_ref, top, json),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn attach(workspace_root: &std::path::Path, run_ref: &str, report_path: &std::path::Path) -> Result<(), String> {
    let run = store::load_run(workspace_root, run_ref)?;
    let source = fs::read_to_string(report_path).map_err(|error| format!("failed to read {}: {error}", report_path.display()))?;
    let report = PerfReport::from_json_str(&source).map_err(|error| error.to_string())?;

    if let Some(revision) = report.metadata.source_revision.as_deref() {
        if revision != run.git.sha {
            return Err(format!("profile source revision {revision} does not match sample revision {}", run.git.sha));
        }
    }
    if let Some(dirty) = report.metadata.dirty {
        if dirty != run.git.dirty {
            return Err(format!("profile dirty state {dirty} does not match sample dirty state {}", run.git.dirty));
        }
    }

    let normalized = report.to_json_string().map_err(|error| error.to_string())?;
    let saved = store::save_internal_profile(workspace_root, &run.run_id, &normalized)?;
    println!("Attached profile '{}' to sample '{}' ({})", report.metadata.run_id, run.run_id, saved.display());
    Ok(())
}

fn show(workspace_root: &std::path::Path, run_ref: &str, top: usize, json: bool) -> Result<(), String> {
    let (run, profile_json) = store::load_internal_profile(workspace_root, run_ref)?;
    let report = PerfReport::from_json_str(&profile_json).map_err(|error| error.to_string())?;
    let summary = profile_summary(&run.run_id, &report, top);
    if json {
        println!("{}", serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?);
    } else {
        render_summary(&summary);
    }
    Ok(())
}

fn compare(workspace_root: &std::path::Path, baseline_ref: &str, candidate_ref: &str, top: usize, json: bool) -> Result<(), String> {
    let (baseline_run, baseline_json) = store::load_internal_profile(workspace_root, baseline_ref)?;
    let (candidate_run, candidate_json) = store::load_internal_profile(workspace_root, candidate_ref)?;
    let baseline = PerfReport::from_json_str(&baseline_json).map_err(|error| error.to_string())?;
    let candidate = PerfReport::from_json_str(&candidate_json).map_err(|error| error.to_string())?;
    let comparison = profile_comparison(&baseline_run.run_id, &baseline, &candidate_run.run_id, &candidate, top);
    if json {
        println!("{}", serde_json::to_string_pretty(&comparison).map_err(|error| error.to_string())?);
    } else {
        render_comparison(&comparison);
    }
    Ok(())
}

#[derive(Serialize)]
struct ProfileSummary {
    sample_run_id: String,
    profile_run_id: String,
    schema_version: u32,
    observability_level: String,
    lifecycle: Option<String>,
    baseline_head: Option<String>,
    execution_tree_head: Option<String>,
    source_revision: Option<String>,
    branch: Option<String>,
    dirty: Option<bool>,
    workload: Option<String>,
    elapsed_ns: u64,
    span_count: usize,
    outcomes: BTreeMap<String, usize>,
    top_phases: Vec<PhaseSummary>,
    top_counters: Vec<CounterSummary>,
    query_totals: QueryTotals,
    derived: BTreeMap<String, DerivedMetric>,
}

#[derive(Serialize)]
struct PhaseSummary {
    phase: String,
    total_ns: u64,
    max_span_ns: u64,
    spans: usize,
}

#[derive(Serialize)]
struct CounterSummary {
    counter: String,
    value: u64,
}

#[derive(Default, Serialize)]
struct QueryTotals {
    kinds: usize,
    requests: u64,
    direct_reuse: u64,
    revalidated_reuse: u64,
    recomputed: u64,
    invalidated: u64,
    blocked: u64,
    cancelled: u64,
    budget_exceeded: u64,
    failed: u64,
    recompute_duration_ns: u64,
}

#[derive(Serialize)]
struct DerivedMetric {
    numerator: Option<u64>,
    denominator: Option<u64>,
    ratio: Option<f64>,
    note: Option<String>,
}

fn profile_summary(sample_run_id: &str, report: &PerfReport, top: usize) -> ProfileSummary {
    let mut phase_totals = BTreeMap::<String, (u64, u64, usize)>::new();
    let mut outcomes = BTreeMap::<String, usize>::new();
    for span in &report.spans {
        let phase = phase_name(span.phase);
        let totals = phase_totals.entry(phase).or_default();
        totals.0 = totals.0.saturating_add(span.duration_ns);
        totals.1 = totals.1.max(span.duration_ns);
        totals.2 += 1;
        *outcomes.entry(format!("{:?}", span.outcome)).or_default() += 1;
    }
    let mut top_phases: Vec<_> = phase_totals
        .into_iter()
        .map(|(phase, (total_ns, max_span_ns, spans))| PhaseSummary {
            phase,
            total_ns,
            max_span_ns,
            spans,
        })
        .collect();
    top_phases.sort_by_key(|phase| std::cmp::Reverse(phase.total_ns));
    top_phases.truncate(top);

    let mut top_counters: Vec<_> = report
        .counters
        .iter()
        .map(|(counter, value)| CounterSummary {
            counter: counter_name(*counter),
            value: *value,
        })
        .collect();
    top_counters.sort_by_key(|counter| std::cmp::Reverse(counter.value));
    top_counters.truncate(top);

    let query_totals = query_totals(report);
    ProfileSummary {
        sample_run_id: sample_run_id.to_owned(),
        profile_run_id: report.metadata.run_id.clone(),
        schema_version: report.schema_version,
        observability_level: format!("{:?}", report.metadata.observability_level),
        lifecycle: report.metadata.lifecycle.map(|lifecycle| format!("{:?}", lifecycle)),
        baseline_head: report.metadata.baseline_head.clone(),
        execution_tree_head: report.metadata.execution_tree_head.clone(),
        source_revision: report.metadata.source_revision.clone(),
        branch: report.metadata.branch.clone(),
        dirty: report.metadata.dirty,
        workload: report.metadata.workload.clone(),
        elapsed_ns: report.elapsed_ns,
        span_count: report.spans.len(),
        outcomes,
        top_phases,
        top_counters,
        query_totals,
        derived: derived_metrics(report),
    }
}

fn query_totals(report: &PerfReport) -> QueryTotals {
    let mut totals = QueryTotals {
        kinds: report.query_metrics.len(),
        ..QueryTotals::default()
    };
    for metrics in &report.query_metrics {
        totals.requests = totals.requests.saturating_add(metrics.requests);
        totals.direct_reuse = totals.direct_reuse.saturating_add(metrics.direct_reuse);
        totals.revalidated_reuse = totals.revalidated_reuse.saturating_add(metrics.revalidated_reuse);
        totals.recomputed = totals.recomputed.saturating_add(metrics.recomputed);
        totals.invalidated = totals.invalidated.saturating_add(metrics.invalidated);
        totals.blocked = totals.blocked.saturating_add(metrics.blocked);
        totals.cancelled = totals.cancelled.saturating_add(metrics.cancelled);
        totals.budget_exceeded = totals.budget_exceeded.saturating_add(metrics.budget_exceeded);
        totals.failed = totals.failed.saturating_add(metrics.failed);
        totals.recompute_duration_ns = totals.recompute_duration_ns.saturating_add(metrics.recompute_duration_ns);
    }
    totals
}

fn derived_metrics(report: &PerfReport) -> BTreeMap<String, DerivedMetric> {
    let mut derived = BTreeMap::new();
    derived.insert(
        "source_traversal_amplification".into(),
        counter_ratio(
            report,
            PerfCounter::SemanticSourceTopLevelStatementVisits,
            PerfCounter::SemanticSourceUniqueTopLevelStatements,
            "total statement visits / unique top-level statements",
        ),
    );
    derived.insert(
        "dependency_fanout".into(),
        counter_ratio(
            report,
            PerfCounter::SemanticReverseClosureNodesVisited,
            PerfCounter::SemanticReverseClosureSeeds,
            "visited reverse-closure nodes / seed nodes",
        ),
    );
    derived.insert(
        "query_recomputation_ratio".into(),
        counter_ratio(
            report,
            PerfCounter::QueryRecomputed,
            PerfCounter::QueryRequests,
            "recomputed / requested queries",
        ),
    );
    derived.insert(
        "callable_body_analysis_amplification".into(),
        unavailable("unique callable-body identity count is not emitted by this report"),
    );
    derived.insert(
        "copy_amplification".into(),
        unavailable("physical copied entries and final retained-entry denominator are not measured"),
    );
    derived.insert("universe_work_ratio".into(), universe_work_ratio(report));
    derived
}

fn counter_ratio(report: &PerfReport, numerator: PerfCounter, denominator: PerfCounter, note: &str) -> DerivedMetric {
    let numerator = report.counters.get(&numerator).copied();
    let denominator = report.counters.get(&denominator).copied();
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator > 0 => DerivedMetric {
            numerator: Some(numerator),
            denominator: Some(denominator),
            ratio: Some(numerator as f64 / denominator as f64),
            note: Some(note.to_owned()),
        },
        (Some(numerator), Some(0)) => DerivedMetric {
            numerator: Some(numerator),
            denominator: Some(0),
            ratio: None,
            note: Some("denominator is zero".into()),
        },
        _ => unavailable("required numerator or denominator counter is absent"),
    }
}

fn universe_work_ratio(report: &PerfReport) -> DerivedMetric {
    let attributed_ns = report
        .spans
        .iter()
        .filter_map(|span| span.attribution.as_ref().map(|_| span.duration_ns))
        .fold(0u64, u64::saturating_add);
    let universe_ns = report
        .spans
        .iter()
        .filter(|span| span.attribution.as_ref().is_some_and(|attribution| attribution.scope == PerfScope::Universe))
        .map(|span| span.duration_ns)
        .fold(0u64, u64::saturating_add);
    if attributed_ns == 0 {
        return unavailable("profile has no scope-attributed phase durations");
    }
    DerivedMetric {
        numerator: Some(universe_ns),
        denominator: Some(attributed_ns),
        ratio: Some(universe_ns as f64 / attributed_ns as f64),
        note: Some("Universe time / time in scope-attributed spans; attribution coverage may be partial".into()),
    }
}

fn unavailable(note: &str) -> DerivedMetric {
    DerivedMetric {
        numerator: None,
        denominator: None,
        ratio: None,
        note: Some(note.to_owned()),
    }
}

#[derive(Serialize)]
struct ProfileComparison {
    baseline_sample_run_id: String,
    candidate_sample_run_id: String,
    baseline_profile_run_id: String,
    candidate_profile_run_id: String,
    compatible: bool,
    incompatibility_reasons: Vec<String>,
    baseline_elapsed_ns: u64,
    candidate_elapsed_ns: u64,
    elapsed_delta_ns: i128,
    elapsed_delta_pct: Option<f64>,
    phase_deltas: Vec<PhaseDelta>,
    counter_deltas: Vec<CounterDelta>,
    query_deltas: Vec<QueryDelta>,
    baseline_derived: BTreeMap<String, DerivedMetric>,
    candidate_derived: BTreeMap<String, DerivedMetric>,
}

#[derive(Serialize)]
struct PhaseDelta {
    phase: String,
    baseline_ns: u64,
    candidate_ns: u64,
    delta_ns: i128,
    delta_pct: Option<f64>,
}

#[derive(Serialize)]
struct CounterDelta {
    counter: String,
    baseline: u64,
    candidate: u64,
    delta: i128,
}

#[derive(Clone, Copy, Default)]
struct QueryCounts {
    requests: u64,
    direct_reuse: u64,
    revalidated_reuse: u64,
    recomputed: u64,
    invalidated: u64,
}

#[derive(Serialize)]
struct QueryDelta {
    query_kind: String,
    requests_delta: i128,
    direct_reuse_delta: i128,
    revalidated_reuse_delta: i128,
    recomputed_delta: i128,
    invalidated_delta: i128,
}

fn profile_comparison(baseline_sample_id: &str, baseline: &PerfReport, candidate_sample_id: &str, candidate: &PerfReport, top: usize) -> ProfileComparison {
    let mut reasons = Vec::new();
    if baseline.metadata.workload != candidate.metadata.workload {
        reasons.push(format!(
            "workload differs: {:?} vs {:?}",
            baseline.metadata.workload, candidate.metadata.workload
        ));
    }
    if baseline.metadata.observability_level != candidate.metadata.observability_level {
        reasons.push("observability level differs".into());
    }
    if baseline.metadata.lifecycle != candidate.metadata.lifecycle {
        reasons.push("process/session lifecycle classification differs".into());
    }
    if baseline.metadata.baseline_head != candidate.metadata.baseline_head {
        reasons.push("pinned baseline revision differs".into());
    }
    let phase_base = phase_totals(baseline);
    let phase_candidate = phase_totals(candidate);
    let phase_names: std::collections::BTreeSet<_> = phase_base.keys().chain(phase_candidate.keys()).cloned().collect();
    let mut phase_deltas: Vec<_> = phase_names
        .into_iter()
        .map(|phase| {
            let base = phase_base.get(&phase).copied().unwrap_or(0);
            let cand = phase_candidate.get(&phase).copied().unwrap_or(0);
            PhaseDelta {
                phase,
                baseline_ns: base,
                candidate_ns: cand,
                delta_ns: cand as i128 - base as i128,
                delta_pct: (base > 0).then_some((cand as f64 / base as f64 - 1.0) * 100.0),
            }
        })
        .collect();
    phase_deltas.sort_by_key(|delta| std::cmp::Reverse(delta.delta_ns.unsigned_abs()));
    phase_deltas.truncate(top);

    let counter_base = counter_values(baseline);
    let counter_candidate = counter_values(candidate);
    let counter_names: std::collections::BTreeSet<_> = counter_base.keys().chain(counter_candidate.keys()).cloned().collect();
    let mut counter_deltas: Vec<_> = counter_names
        .into_iter()
        .map(|counter| {
            let base = counter_base.get(&counter).copied().unwrap_or(0);
            let cand = counter_candidate.get(&counter).copied().unwrap_or(0);
            CounterDelta {
                counter,
                baseline: base,
                candidate: cand,
                delta: cand as i128 - base as i128,
            }
        })
        .collect();
    counter_deltas.sort_by_key(|delta| std::cmp::Reverse(delta.delta.unsigned_abs()));
    counter_deltas.truncate(top);

    let query_base = query_counts(baseline);
    let query_candidate = query_counts(candidate);
    let query_kinds: std::collections::BTreeSet<_> = query_base.keys().chain(query_candidate.keys()).cloned().collect();
    let mut query_deltas: Vec<_> = query_kinds
        .into_iter()
        .map(|query_kind| {
            let base = query_base.get(&query_kind).copied().unwrap_or_default();
            let cand = query_candidate.get(&query_kind).copied().unwrap_or_default();
            QueryDelta {
                query_kind,
                requests_delta: cand.requests as i128 - base.requests as i128,
                direct_reuse_delta: cand.direct_reuse as i128 - base.direct_reuse as i128,
                revalidated_reuse_delta: cand.revalidated_reuse as i128 - base.revalidated_reuse as i128,
                recomputed_delta: cand.recomputed as i128 - base.recomputed as i128,
                invalidated_delta: cand.invalidated as i128 - base.invalidated as i128,
            }
        })
        .collect();
    query_deltas.sort_by_key(|delta| {
        std::cmp::Reverse(
            delta.requests_delta.unsigned_abs()
                + delta.direct_reuse_delta.unsigned_abs()
                + delta.revalidated_reuse_delta.unsigned_abs()
                + delta.recomputed_delta.unsigned_abs()
                + delta.invalidated_delta.unsigned_abs(),
        )
    });
    query_deltas.truncate(top);

    let elapsed_delta_ns = candidate.elapsed_ns as i128 - baseline.elapsed_ns as i128;
    ProfileComparison {
        baseline_sample_run_id: baseline_sample_id.to_owned(),
        candidate_sample_run_id: candidate_sample_id.to_owned(),
        baseline_profile_run_id: baseline.metadata.run_id.clone(),
        candidate_profile_run_id: candidate.metadata.run_id.clone(),
        compatible: reasons.is_empty(),
        incompatibility_reasons: reasons,
        baseline_elapsed_ns: baseline.elapsed_ns,
        candidate_elapsed_ns: candidate.elapsed_ns,
        elapsed_delta_ns,
        elapsed_delta_pct: (baseline.elapsed_ns > 0).then_some((candidate.elapsed_ns as f64 / baseline.elapsed_ns as f64 - 1.0) * 100.0),
        phase_deltas,
        counter_deltas,
        query_deltas,
        baseline_derived: derived_metrics(baseline),
        candidate_derived: derived_metrics(candidate),
    }
}

fn phase_totals(report: &PerfReport) -> BTreeMap<String, u64> {
    let mut totals = BTreeMap::new();
    for span in &report.spans {
        let total = totals.entry(phase_name(span.phase)).or_insert(0u64);
        *total = total.saturating_add(span.duration_ns);
    }
    totals
}

fn counter_values(report: &PerfReport) -> BTreeMap<String, u64> {
    report.counters.iter().map(|(counter, value)| (counter_name(*counter), *value)).collect()
}

fn query_counts(report: &PerfReport) -> BTreeMap<String, QueryCounts> {
    let mut counts = BTreeMap::<String, QueryCounts>::new();
    for metrics in &report.query_metrics {
        let count = counts.entry(metrics.query_kind.clone()).or_default();
        count.requests = count.requests.saturating_add(metrics.requests);
        count.direct_reuse = count.direct_reuse.saturating_add(metrics.direct_reuse);
        count.revalidated_reuse = count.revalidated_reuse.saturating_add(metrics.revalidated_reuse);
        count.recomputed = count.recomputed.saturating_add(metrics.recomputed);
        count.invalidated = count.invalidated.saturating_add(metrics.invalidated);
    }
    counts
}

fn phase_name(phase: PerfPhase) -> String {
    serde_json::to_value(phase)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{:?}", phase))
}

fn counter_name(counter: PerfCounter) -> String {
    serde_json::to_value(counter)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{:?}", counter))
}

fn render_summary(summary: &ProfileSummary) {
    println!("Sample run: {}", summary.sample_run_id);
    println!(
        "Profile run: {}  schema v{}  level {}  lifecycle {}",
        summary.profile_run_id,
        summary.schema_version,
        summary.observability_level,
        summary.lifecycle.as_deref().unwrap_or("unspecified")
    );
    println!(
        "Workload: {}  elapsed: {:.3} s  spans: {}",
        summary.workload.as_deref().unwrap_or("unspecified"),
        summary.elapsed_ns as f64 / 1e9,
        summary.span_count
    );
    println!(
        "Baseline: {}  source: {}  branch: {}  dirty: {}",
        summary.baseline_head.as_deref().unwrap_or("unknown"),
        summary.source_revision.as_deref().unwrap_or("unknown"),
        summary.branch.as_deref().unwrap_or("unknown"),
        summary.dirty.map(|dirty| dirty.to_string()).unwrap_or_else(|| "unknown".into())
    );
    println!("Outcomes: {:?}", summary.outcomes);
    println!("\nTop phases:");
    for phase in &summary.top_phases {
        println!(
            "  {:<52} {:>10.3} ms  max {:>9.3} ms  spans {}",
            phase.phase,
            phase.total_ns as f64 / 1e6,
            phase.max_span_ns as f64 / 1e6,
            phase.spans
        );
    }
    println!(
        "\nQuery lifecycle: {} kinds, {} requests, {} reused, {} revalidated, {} recomputed, {} invalidated",
        summary.query_totals.kinds,
        summary.query_totals.requests,
        summary.query_totals.direct_reuse,
        summary.query_totals.revalidated_reuse,
        summary.query_totals.recomputed,
        summary.query_totals.invalidated
    );
    println!("\nTop counters:");
    for counter in &summary.top_counters {
        println!("  {:<54} {}", counter.counter, counter.value);
    }
    println!("\nDerived amplification:");
    for (name, metric) in &summary.derived {
        let value = metric.ratio.map(|ratio| format!("{ratio:.3}x")).unwrap_or_else(|| "unavailable".into());
        println!(
            "  {name}: {value}{}",
            metric.note.as_deref().map(|note| format!(" ({note})")).unwrap_or_default()
        );
    }
}

fn render_comparison(comparison: &ProfileComparison) {
    println!(
        "Internal profiles: {} -> {}",
        comparison.baseline_sample_run_id, comparison.candidate_sample_run_id
    );
    println!(
        "Compatible: {}{}",
        comparison.compatible,
        if comparison.incompatibility_reasons.is_empty() {
            String::new()
        } else {
            format!(" ({})", comparison.incompatibility_reasons.join("; "))
        }
    );
    let elapsed_pct = comparison
        .elapsed_delta_pct
        .map(|value| format!("{value:+.2}%"))
        .unwrap_or_else(|| "n/a".into());
    println!(
        "Elapsed: {:.3} s -> {:.3} s ({:+.3} s, {elapsed_pct})",
        comparison.baseline_elapsed_ns as f64 / 1e9,
        comparison.candidate_elapsed_ns as f64 / 1e9,
        comparison.elapsed_delta_ns as f64 / 1e9
    );
    println!("\nLargest phase deltas:");
    for phase in &comparison.phase_deltas {
        let pct = phase.delta_pct.map(|value| format!("{value:+.2}%")).unwrap_or_else(|| "n/a".into());
        println!("  {:<52} {:+.3} ms ({pct})", phase.phase, phase.delta_ns as f64 / 1e6);
    }
    println!("\nLargest counter deltas:");
    for counter in &comparison.counter_deltas {
        println!("  {:<54} {:+}", counter.counter, counter.delta);
    }
    println!("\nQuery lifecycle deltas:");
    for query in &comparison.query_deltas {
        println!(
            "  {:<32} requests {:+}, reused {:+}, revalidated {:+}, recomputed {:+}, invalidated {:+}",
            query.query_kind, query.requests_delta, query.direct_reuse_delta, query.revalidated_reuse_delta, query.recomputed_delta, query.invalidated_delta
        );
    }
}
