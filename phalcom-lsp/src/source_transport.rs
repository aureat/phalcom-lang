//! Protocol-source identity conversion at compiler-owned source boundaries.

use phalcom_modules::{SourceId, SourceLocation};
use tower_lsp::lsp_types::Url;

/// Converts a file URI to the canonical source location used by module APIs.
pub(crate) fn source_location_for_uri(uri: &Url) -> Option<SourceLocation> {
    let path = uri.to_file_path().ok()?;
    let canonical_source = phalcom_modules::source::canonicalize_path(&path);
    Some(SourceLocation {
        source_id: SourceId(canonical_source.to_string_lossy().into()),
        display_path: path,
    })
}

/// Converts a file URI to its canonical source identity.
pub(crate) fn source_id_for_uri(uri: &Url) -> Option<SourceId> {
    source_location_for_uri(uri).map(|location| location.source_id)
}
