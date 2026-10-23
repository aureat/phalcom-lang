//! Prints the digests of the Universe compiler artifact and the full Universe
//! artifact. Used to record and re-check the VER-R1 byte-identity baseline of
//! the general artifact compilation program (`docs/implementation-spec/artifact-compilation`).

use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_core::modules::canonical_artifact::{build_full_universe_telosc, build_universe_telosc};
use phalcom_core::modules::canonical_universe::canonical_universe_program;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn main() {
    let program = canonical_universe_program().expect("canonical universe program");
    let limits = ValidationLimits::default();
    let view = program.compiler_image_view().expect("compiler image view");
    let compiler = build_universe_telosc(&view, &limits).expect("compiler artifact");
    if let Some(dir) = std::env::args().nth(1) {
        std::fs::write(format!("{dir}/compiler.telosc"), &compiler.bytes).expect("write compiler artifact");
    }
    let container = phalcom_compiler_artifact::container::TeloscContainerView::parse(&compiler.bytes, &limits).expect("parse");
    let header = container.header();
    println!("compiler.interface_digest={}", hex(header.interface_digest.as_bytes()));
    println!("compiler.semantic_digest={}", hex(header.semantic_digest.as_bytes()));
    println!("compiler.query_digest={}", hex(header.query_digest.as_bytes()));
    println!("compiler.metadata_digest={}", hex(header.metadata_digest.as_bytes()));
    let sm = &compiler.summary;
    println!("sections module={} semantic={} query={} metadata={}", sm.module_world_bytes, sm.semantic_world_bytes, sm.query_image_bytes, sm.metadata_bytes);
    println!("compiler.len={}", compiler.bytes.len());
    println!("compiler.artifact_digest={}", hex(&compiler.summary.artifact_digest));
    let full = build_full_universe_telosc(&program, &limits).expect("full artifact");
    println!("full.len={}", full.bytes.len());
    println!("full.artifact_digest={}", hex(&full.summary.artifact_digest));
}
