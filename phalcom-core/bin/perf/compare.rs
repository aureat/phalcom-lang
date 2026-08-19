use std::collections::BTreeMap;

use crate::model::{BenchmarkRun, CaseAggregate, CaseComparison, ComparisonRun, MetricSummary, PairedStats, SampleStatus, Verdict};

pub fn calculate_metric_summary(vals: &[f64]) -> Option<MetricSummary> {
    if vals.is_empty() {
        return None;
    }
    let mut sorted = vals.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let min = sorted[0];
    let max = sorted[n - 1];

    let median = calc_median(&sorted);
    let mad = calc_mad(&sorted, median);
    let p90 = calc_p90(&sorted);

    Some(MetricSummary { n, min, median, max, mad, p90 })
}

pub fn calc_median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

pub fn calc_mad(sorted: &[f64], median: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let mut devs: Vec<f64> = sorted.iter().map(|v| (v - median).abs()).collect();
    devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    calc_median(&devs)
}

pub fn calc_p90(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = (0.90 * (sorted.len() - 1) as f64).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

pub fn compute_case_aggregate(case_id: &str, tag: &str, samples: &[crate::model::Sample]) -> Option<CaseAggregate> {
    let ok_samples: Vec<&crate::model::Sample> = samples.iter().filter(|s| s.status == SampleStatus::Ok).collect();

    if ok_samples.is_empty() {
        return None;
    }

    let wall_vals: Vec<f64> = ok_samples.iter().map(|s| s.wall_ns as f64 / 1_000_000.0).collect();
    let user_vals: Vec<f64> = ok_samples.iter().filter_map(|s| s.user_ns.map(|v| v as f64 / 1_000_000.0)).collect();
    let sys_vals: Vec<f64> = ok_samples.iter().filter_map(|s| s.sys_ns.map(|v| v as f64 / 1_000_000.0)).collect();
    let rss_vals: Vec<f64> = ok_samples.iter().filter_map(|s| s.peak_rss_bytes.map(|v| v as f64)).collect();

    let wall = calculate_metric_summary(&wall_vals)?;
    let user_ns = calculate_metric_summary(&user_vals);
    let sys_ns = calculate_metric_summary(&sys_vals);
    let peak_rss_bytes = calculate_metric_summary(&rss_vals);

    Some(CaseAggregate {
        id: case_id.to_string(),
        tag: tag.to_string(),
        wall,
        user_ns,
        sys_ns,
        peak_rss_bytes,
    })
}

pub fn check_compatibility(baseline: &BenchmarkRun, candidate: &BenchmarkRun, allow_host_mismatch: bool) -> (bool, Vec<String>) {
    let mut reasons = Vec::new();

    if baseline.host.arch != candidate.host.arch {
        reasons.push(format!("architecture mismatch: {} vs {}", baseline.host.arch, candidate.host.arch));
    }
    if baseline.host.os != candidate.host.os {
        reasons.push(format!("OS mismatch: {} vs {}", baseline.host.os, candidate.host.os));
    }
    if baseline.build.profile != candidate.build.profile {
        reasons.push(format!("profile mismatch: {} vs {}", baseline.build.profile, candidate.build.profile));
    }
    if baseline.host.cpu_model != candidate.host.cpu_model {
        let msg = format!("CPU model mismatch: '{}' vs '{}'", baseline.host.cpu_model, candidate.host.cpu_model);
        if allow_host_mismatch {
            eprintln!("warning: {msg}");
        } else {
            reasons.push(msg);
        }
    }

    let compatible = reasons.is_empty();
    (compatible, reasons)
}

pub fn compare_runs(baseline: &BenchmarkRun, candidate: &BenchmarkRun, allow_host_mismatch: bool, gate: bool) -> ComparisonRun {
    let (compatible, incompatibility_reasons) = check_compatibility(baseline, candidate, allow_host_mismatch);

    let mut layout_delta = BTreeMap::new();
    for (type_name, base_layout) in &baseline.layouts {
        if let Some(cand_layout) = candidate.layouts.get(type_name) {
            if base_layout != cand_layout {
                layout_delta.insert(type_name.clone(), (*base_layout, *cand_layout));
            }
        }
    }

    let cand_cases_map: BTreeMap<&str, &crate::model::CaseResult> = candidate.cases.iter().map(|c| (c.id.as_str(), c)).collect();

    let mut case_comparisons = Vec::new();
    let mut has_regressions = false;
    let mut has_improvements = false;

    for base_case in &baseline.cases {
        let base_agg = base_case.aggregate.clone();
        let cand_case = cand_cases_map.get(base_case.id.as_str());
        let cand_agg = cand_case.and_then(|c| c.aggregate.clone());

        let (delta_abs, delta_pct, gate_result, paired_stats) = analyze_case_pair(base_case, cand_case.copied(), base_agg.as_ref(), cand_agg.as_ref(), gate);

        if gate_result == "FAIL" {
            has_regressions = true;
        } else if gate_result == "IMPROVED" {
            has_improvements = true;
        }

        case_comparisons.push(CaseComparison {
            id: base_case.id.clone(),
            baseline_agg: base_agg,
            candidate_agg: cand_agg,
            delta_abs,
            delta_pct,
            gate_threshold: Some(0.05), // 5% wall threshold
            gate_result,
            paired_stats,
        });
    }

    let verdict = if has_regressions {
        Verdict::Regressions
    } else if has_improvements {
        Verdict::Improvements
    } else {
        Verdict::Neutral
    };

    ComparisonRun {
        schema_version: 1,
        comparison_id: format!("{}-vs-{}", baseline.run_id, candidate.run_id),
        baseline_run_id: baseline.run_id.clone(),
        candidate_run_id: candidate.run_id.clone(),
        compatible,
        incompatibility_reasons,
        layout_delta,
        cases: case_comparisons,
        verdict,
    }
}

fn analyze_case_pair(
    base_case: &crate::model::CaseResult,
    cand_case: Option<&crate::model::CaseResult>,
    base_agg: Option<&CaseAggregate>,
    cand_agg: Option<&CaseAggregate>,
    gate: bool,
) -> (Option<f64>, Option<f64>, String, Option<PairedStats>) {
    let (base_agg, cand_agg) = match (base_agg, cand_agg) {
        (Some(b), Some(c)) => (b, c),
        _ => return (None, None, "MISSING".into(), None),
    };

    let base_wall = base_agg.wall.median;
    let cand_wall = cand_agg.wall.median;

    let delta_abs = cand_wall - base_wall;
    let delta_pct = (cand_wall / base_wall - 1.0) * 100.0;

    let mut paired_stats = None;
    if let Some(cand_case) = cand_case {
        if !base_case.samples.is_empty() && base_case.samples.len() == cand_case.samples.len() {
            paired_stats = compute_paired_stats(&base_case.samples, &cand_case.samples);
        }
    }

    let mut gate_result = "PASS".to_string();

    if gate {
        // Wall regression > 5% AND > 1.0 ms
        let wall_regression = delta_pct > 5.0 && delta_abs > 1.0;

        // RSS regression > 10% AND > 8 MiB (8388608 bytes)
        let rss_regression = match (&base_agg.peak_rss_bytes, &cand_agg.peak_rss_bytes) {
            (Some(b_rss), Some(c_rss)) => {
                let abs_rss_delta = c_rss.median - b_rss.median;
                let pct_rss_delta = (c_rss.median / b_rss.median - 1.0) * 100.0;
                pct_rss_delta > 10.0 && abs_rss_delta > 8_388_608.0
            }
            _ => false,
        };

        if let Some(ref pstats) = paired_stats {
            let two_thirds = (pstats.pairs as f64 * 2.0 / 3.0).ceil() as usize;
            if wall_regression && pstats.slower >= two_thirds {
                gate_result = "FAIL".into();
            } else if delta_pct < -5.0 && delta_abs < -1.0 && pstats.faster >= two_thirds {
                gate_result = "IMPROVED".into();
            } else if wall_regression || delta_pct < -5.0 {
                gate_result = "NOISY".into();
            }
        } else if wall_regression || rss_regression {
            gate_result = "FAIL".into();
        } else if delta_pct < -5.0 && delta_abs < -1.0 {
            gate_result = "IMPROVED".into();
        }
    }

    (Some(delta_abs), Some(delta_pct), gate_result, paired_stats)
}

fn compute_paired_stats(base_samples: &[crate::model::Sample], cand_samples: &[crate::model::Sample]) -> Option<PairedStats> {
    let mut ratios = Vec::new();
    let mut faster = 0;
    let mut slower = 0;
    let mut ties = 0;

    for (b, c) in base_samples.iter().zip(cand_samples.iter()) {
        if b.status == SampleStatus::Ok && c.status == SampleStatus::Ok {
            let ratio = c.wall_ns as f64 / b.wall_ns as f64;
            ratios.push(ratio);
            if (ratio - 1.0).abs() < 0.01 {
                ties += 1;
            } else if ratio < 1.0 {
                faster += 1;
            } else {
                slower += 1;
            }
        }
    }

    if ratios.is_empty() {
        return None;
    }

    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_ratio = calc_median(&ratios);
    let pairs = ratios.len();
    let two_thirds = (pairs as f64 * 2.0 / 3.0).ceil() as usize;

    let inconclusive = faster < two_thirds && slower < two_thirds;

    Some(PairedStats {
        pairs,
        faster,
        slower,
        ties,
        median_ratio,
        inconclusive,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_and_mad() {
        let vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = calculate_metric_summary(&vals).unwrap();
        assert_eq!(summary.median, 3.0);
        assert_eq!(summary.min, 1.0);
        assert_eq!(summary.max, 5.0);
        assert_eq!(summary.mad, 1.0);
    }

    #[test]
    fn paired_stats_2_3_rule() {
        let mut base_samples = Vec::new();
        let mut cand_samples = Vec::new();

        for i in 0..6 {
            base_samples.push(crate::model::Sample {
                index: i,
                order: None,
                wall_ns: 100_000_000,
                user_ns: None,
                sys_ns: None,
                peak_rss_bytes: None,
                minor_page_faults: None,
                major_page_faults: None,
                voluntary_context_switches: None,
                involuntary_context_switches: None,
                exit_code: 0,
                status: SampleStatus::Ok,
            });
            // Candidate is slower 5 out of 6 times
            let cand_ns = if i == 0 { 95_000_000 } else { 120_000_000 };
            cand_samples.push(crate::model::Sample {
                index: i,
                order: None,
                wall_ns: cand_ns,
                user_ns: None,
                sys_ns: None,
                peak_rss_bytes: None,
                minor_page_faults: None,
                major_page_faults: None,
                voluntary_context_switches: None,
                involuntary_context_switches: None,
                exit_code: 0,
                status: SampleStatus::Ok,
            });
        }

        let stats = compute_paired_stats(&base_samples, &cand_samples).unwrap();
        assert_eq!(stats.pairs, 6);
        assert_eq!(stats.slower, 5);
        assert_eq!(stats.faster, 1);
        assert!(!stats.inconclusive);
    }
}
