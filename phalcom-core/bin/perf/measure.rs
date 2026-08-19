use std::path::Path;
use std::process::Command;
use std::time::Instant;

use crate::model::{ResourceQuality, Sample, SampleOrder, SampleStatus};
use crate::suite::CaseVerification;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMeter {
    MacOsTime,
    LinuxGnuTime,
    WallOnly,
}

impl ResourceMeter {
    pub fn detect() -> Self {
        if cfg!(target_os = "macos") && Path::new("/usr/bin/time").exists() {
            ResourceMeter::MacOsTime
        } else if cfg!(target_os = "linux") && Path::new("/usr/bin/time").exists() {
            ResourceMeter::LinuxGnuTime
        } else {
            ResourceMeter::WallOnly
        }
    }

    pub fn quality(&self) -> ResourceQuality {
        match self {
            ResourceMeter::MacOsTime | ResourceMeter::LinuxGnuTime => ResourceQuality::Full,
            ResourceMeter::WallOnly => ResourceQuality::WallOnly,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParsedResources {
    pub user_ns: Option<u64>,
    pub sys_ns: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
}

pub fn parse_macos_time(stderr: &str) -> ParsedResources {
    let mut res = ParsedResources::default();
    for line in stderr.lines() {
        let line = line.trim();
        if line.contains("user") && line.contains("sys") {
            // Format: "0.05 real 0.02 user 0.02 sys"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for i in 0..parts.len() {
                if parts[i] == "user" && i > 0 {
                    if let Ok(sec) = parts[i - 1].parse::<f64>() {
                        res.user_ns = Some((sec * 1_000_000_000.0) as u64);
                    }
                } else if parts[i] == "sys" && i > 0 {
                    if let Ok(sec) = parts[i - 1].parse::<f64>() {
                        res.sys_ns = Some((sec * 1_000_000_000.0) as u64);
                    }
                }
            }
        } else if line.contains("maximum resident set size") {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(bytes) = num.parse::<u64>() {
                    res.peak_rss_bytes = Some(bytes);
                }
            }
        } else if line.contains("page reclaims") {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(val) = num.parse::<u64>() {
                    res.minor_page_faults = Some(val);
                }
            }
        } else if line.contains("page faults") && !line.contains("minor") && !line.contains("reclaims") {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(val) = num.parse::<u64>() {
                    res.major_page_faults = Some(val);
                }
            }
        } else if line.contains("voluntary context switches") && !line.contains("involuntary") {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(val) = num.parse::<u64>() {
                    res.voluntary_context_switches = Some(val);
                }
            }
        } else if line.contains("involuntary context switches") {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(val) = num.parse::<u64>() {
                    res.involuntary_context_switches = Some(val);
                }
            }
        }
    }
    res
}

pub fn parse_gnu_time(stderr: &str) -> ParsedResources {
    let mut res = ParsedResources::default();
    for line in stderr.lines() {
        let line = line.trim();
        if line.starts_with("User time (seconds):") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(sec) = val.trim().parse::<f64>() {
                    res.user_ns = Some((sec * 1_000_000_000.0) as u64);
                }
            }
        } else if line.starts_with("System time (seconds):") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(sec) = val.trim().parse::<f64>() {
                    res.sys_ns = Some((sec * 1_000_000_000.0) as u64);
                }
            }
        } else if line.contains("Maximum resident set size (kbytes):") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(kb) = val.trim().parse::<u64>() {
                    res.peak_rss_bytes = Some(kb * 1024);
                }
            }
        } else if line.contains("Minor (reclaiming a frame) page faults:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(v) = val.trim().parse::<u64>() {
                    res.minor_page_faults = Some(v);
                }
            }
        } else if line.contains("Major (requiring I/O) page faults:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(v) = val.trim().parse::<u64>() {
                    res.major_page_faults = Some(v);
                }
            }
        } else if line.starts_with("Voluntary context switches:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(v) = val.trim().parse::<u64>() {
                    res.voluntary_context_switches = Some(v);
                }
            }
        } else if line.starts_with("Involuntary context switches:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(v) = val.trim().parse::<u64>() {
                    res.involuntary_context_switches = Some(v);
                }
            }
        }
    }
    res
}

pub fn measure_sample(
    meter: ResourceMeter,
    binary: &Path,
    case_path: &Path,
    verification: &CaseVerification,
    index: usize,
    order: Option<SampleOrder>,
) -> (Sample, String, String) {
    let mut cmd = match meter {
        ResourceMeter::MacOsTime => {
            let mut c = Command::new("/usr/bin/time");
            c.arg("-l").arg(binary).arg(case_path);
            c
        }
        ResourceMeter::LinuxGnuTime => {
            let mut c = Command::new("/usr/bin/time");
            c.arg("-v").arg(binary).arg(case_path);
            c
        }
        ResourceMeter::WallOnly => {
            let mut c = Command::new(binary);
            c.arg(case_path);
            c
        }
    };

    let start = Instant::now();
    let output = cmd.output().unwrap_or_else(|e| {
        panic!("failed to execute process: {e}");
    });
    let elapsed = start.elapsed();
    let wall_ns = elapsed.as_nanos() as u64;

    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();

    let parsed = match meter {
        ResourceMeter::MacOsTime => parse_macos_time(&stderr_str),
        ResourceMeter::LinuxGnuTime => parse_gnu_time(&stderr_str),
        ResourceMeter::WallOnly => ParsedResources::default(),
    };

    let exit_code = output.status.code().unwrap_or(-1);
    let (status, note) = verify_outcome(&output, &stdout_str, &stderr_str, verification, case_path);

    let sample = Sample {
        index,
        order,
        wall_ns,
        user_ns: parsed.user_ns,
        sys_ns: parsed.sys_ns,
        peak_rss_bytes: parsed.peak_rss_bytes,
        minor_page_faults: parsed.minor_page_faults,
        major_page_faults: parsed.major_page_faults,
        voluntary_context_switches: parsed.voluntary_context_switches,
        involuntary_context_switches: parsed.involuntary_context_switches,
        exit_code,
        status,
    };

    (sample, note, stdout_str)
}

fn verify_outcome(output: &std::process::Output, stdout: &str, stderr: &str, verification: &CaseVerification, case_path: &Path) -> (SampleStatus, String) {
    let panicked = output.status.code() == Some(101) || stderr.contains("panicked at");
    if panicked {
        return (SampleStatus::ProcessError, "panicked".into());
    }

    match verification {
        CaseVerification::StdoutExact { expected } => {
            if output.status.code() != Some(0) {
                return (SampleStatus::CorrectnessFailure, format!("non-zero exit {}", output.status));
            }
            let actual_trimmed = stdout.trim();
            let expected_trimmed = expected.trim();
            if actual_trimmed == expected_trimmed {
                (SampleStatus::Ok, String::new())
            } else {
                (
                    SampleStatus::CorrectnessFailure,
                    format!("stdout mismatch: expected `{expected_trimmed}`, got `{actual_trimmed}`"),
                )
            }
        }
        CaseVerification::SidecarExpected => {
            let expected_path = case_path.with_extension("expected");
            let expected_bytes = std::fs::read(&expected_path).unwrap_or_default();
            let mut expected_str = String::from_utf8_lossy(&expected_bytes).to_string();
            if expected_str.ends_with('\n') {
                expected_str.pop();
                if expected_str.ends_with('\r') {
                    expected_str.pop();
                }
            }
            let mut actual_str = stdout.to_string();
            if actual_str.ends_with('\n') {
                actual_str.pop();
                if actual_str.ends_with('\r') {
                    actual_str.pop();
                }
            }
            if actual_str == expected_str {
                (SampleStatus::Ok, String::new())
            } else {
                (SampleStatus::CorrectnessFailure, "stdout mismatch".into())
            }
        }
        CaseVerification::NegativeDiagnostic { substring } => {
            if output.status.code() == Some(0) {
                return (SampleStatus::CorrectnessFailure, "unexpectedly succeeded".into());
            }
            let combined = format!("{stdout}\n{stderr}");
            if combined.contains(substring) {
                (SampleStatus::Ok, String::new())
            } else {
                (SampleStatus::CorrectnessFailure, format!("missing diagnostic `{substring}`"))
            }
        }
        CaseVerification::ExitZeroOnly => {
            if output.status.code() == Some(0) {
                (SampleStatus::Ok, String::new())
            } else {
                (SampleStatus::CorrectnessFailure, format!("non-zero exit {}", output.status))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_macos_fixture() {
        let stderr = r#"
        0.05 real         0.02 user         0.03 sys
   3293184  maximum resident set size
         0  average shared memory size
      1283  page reclaims
         5  page faults
         3  voluntary context switches
        32  involuntary context switches
"#;
        let parsed = parse_macos_time(stderr);
        assert_eq!(parsed.user_ns, Some(20_000_000));
        assert_eq!(parsed.sys_ns, Some(30_000_000));
        assert_eq!(parsed.peak_rss_bytes, Some(3293184));
        assert_eq!(parsed.minor_page_faults, Some(1283));
        assert_eq!(parsed.major_page_faults, Some(5));
        assert_eq!(parsed.voluntary_context_switches, Some(3));
        assert_eq!(parsed.involuntary_context_switches, Some(32));
    }

    #[test]
    fn parse_gnu_fixture() {
        let stderr = r#"
	User time (seconds): 0.04
	System time (seconds): 0.01
	Percent of CPU this job got: 100%
	Elapsed (wall clock) time (h:mm:ss or m:ss): 0:00.05
	Maximum resident set size (kbytes): 4096
	Minor (reclaiming a frame) page faults: 2000
	Major (requiring I/O) page faults: 2
	Voluntary context switches: 10
	Involuntary context switches: 50
"#;
        let parsed = parse_gnu_time(stderr);
        assert_eq!(parsed.user_ns, Some(40_000_000));
        assert_eq!(parsed.sys_ns, Some(10_000_000));
        assert_eq!(parsed.peak_rss_bytes, Some(4096 * 1024));
        assert_eq!(parsed.minor_page_faults, Some(2000));
        assert_eq!(parsed.major_page_faults, Some(2));
        assert_eq!(parsed.voluntary_context_switches, Some(10));
        assert_eq!(parsed.involuntary_context_switches, Some(50));
    }

    #[test]
    fn parse_malformed_returns_default() {
        let stderr = "some random log output";
        let parsed_mac = parse_macos_time(stderr);
        let parsed_gnu = parse_gnu_time(stderr);
        assert_eq!(parsed_mac, ParsedResources::default());
        assert_eq!(parsed_gnu, ParsedResources::default());
    }
}
