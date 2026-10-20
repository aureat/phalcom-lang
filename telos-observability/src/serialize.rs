use crate::{PerfReport, SCHEMA_VERSION};

impl PerfReport {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json_str(input: &str) -> Result<Self, PerfReportDecodeError> {
        let report: Self = serde_json::from_str(input).map_err(PerfReportDecodeError::Json)?;
        // V2 adds query lifecycle aggregates and V3 adds bounded distributions.
        // Older reports remain readable because both new fields default empty.
        if !matches!(report.schema_version, 1..=SCHEMA_VERSION) {
            return Err(PerfReportDecodeError::UnsupportedSchema {
                found: report.schema_version,
                expected: SCHEMA_VERSION,
            });
        }
        Ok(report)
    }
}

#[derive(Debug)]
pub enum PerfReportDecodeError {
    Json(serde_json::Error),
    UnsupportedSchema { found: u32, expected: u32 },
}

impl std::fmt::Display for PerfReportDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(f, "invalid performance report JSON: {error}"),
            Self::UnsupportedSchema { found, expected } => {
                write!(f, "unsupported performance report schema {found}; expected {expected}")
            }
        }
    }
}

impl std::error::Error for PerfReportDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::UnsupportedSchema { .. } => None,
        }
    }
}
