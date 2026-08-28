from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text)


# The session import is rustfmt-wrapped differently from the phase-B anchor.
path = "phalcom-semantic/src/session.rs"
text = read(path)
text = text.replace(
    "    query_source_structure, query_unlinked_interface, semantic_signature_from_surface,\n",
    "    query_source_structure, query_unlinked_interface,\n",
)
write(path, text)

# Make the getter's empty parameter slice infer the canonical element type.
path = "phalcom-semantic/src/checker/declaration_signature.rs"
text = read(path)
text = text.replace(
    "            Box::new([]),\n",
    "            Vec::<CallableParameterSemantic>::new().into_boxed_slice(),\n",
    1,
)
write(path, text)

# `ensure_linked_interface` already accepts the canonical LinkedProgram.
path = "phalcom-semantic/src/db/query.rs"
text = read(path)
text = text.replace(
    "    match ensure_linked_interface(db, &callable.owner.module, formal_inputs) {\n",
    "    match ensure_linked_interface(db, &callable.owner.module, formal_inputs.linked) {\n",
    1,
)
write(path, text)

# The first phase-B signature anchor matched the wrapper. Restore the wrapper
# and add the canonical declaration signature to the actual field-aware body API.
path = "phalcom-semantic/src/checker/body.rs"
text = read(path)
wrapper_pattern = re.compile(
    r"(pub fn analyze_callable_body\(.*?    dispatch: &SurfaceDispatchResolver,\n)"
    r"    declared_signature: Option<\(&CallableId, &crate::signature::CallableSemanticSignature\)>,\n"
    r"(    module: ModuleId,)",
    re.S,
)
text, count = wrapper_pattern.subn(r"\1\2", text, count=1)
if count != 1:
    raise SystemExit(f"expected to restore one analyze_callable_body wrapper signature, found {count}")

fields_pattern = re.compile(
    r"(pub fn analyze_callable_body_with_fields\(.*?    dispatch: &SurfaceDispatchResolver,\n)(    module: ModuleId,)",
    re.S,
)
text, count = fields_pattern.subn(
    r"\1    declared_signature: Option<(&CallableId, &crate::signature::CallableSemanticSignature)>,\n\2",
    text,
    count=1,
)
if count != 1:
    raise SystemExit(f"expected one analyze_callable_body_with_fields signature, found {count}")
write(path, text)

print("phase B compile-anchor fixes applied")
