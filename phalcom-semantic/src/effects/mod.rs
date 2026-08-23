//! Effect system foundation and inference (Spec 05).

pub mod atom;
pub mod infer;
pub mod scc;
pub mod summary;

pub use atom::{EffectAtom, EffectSet};
pub use infer::{adapt_effect_atom, adapt_effect_spec, infer_intraprocedural_effects};
pub use scc::infer_interprocedural_effects_scc;
pub use summary::{EffectKnowledge, EffectOpaqueReason};
