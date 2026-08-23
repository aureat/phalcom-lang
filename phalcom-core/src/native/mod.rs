//! Runtime descriptor, registry, and installer for native Phalcom primitives.

pub mod descriptor;
pub mod install;
pub mod registry;
pub mod source;
pub mod verify;

pub use descriptor::*;
pub use install::*;
pub use registry::*;
pub use source::*;
pub use verify::*;
