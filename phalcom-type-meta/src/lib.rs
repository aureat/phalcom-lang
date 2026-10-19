#![feature(allocator_api)]
//! Standalone, store-independent, versioned semantic metadata models, indexed graphs, and artifact schema.

pub mod allocation;
pub mod budgeted;
pub mod bundle;
pub mod declaration;
pub mod encode;
pub mod field_key;
pub mod fingerprint;
pub mod generic;
pub mod header;
pub mod identity;
pub mod kind;
pub mod scalar;
pub mod scoped_type;
pub mod static_definition;
pub mod static_value;
pub mod type_node;
pub mod validate;

pub use allocation::*;
pub use budgeted::*;
pub use bundle::*;
pub use declaration::*;
pub use encode::*;
pub use field_key::*;
pub use fingerprint::*;
pub use generic::*;
pub use header::*;
pub use identity::*;
pub use kind::*;
pub use scalar::*;
pub use scoped_type::*;
pub use static_definition::*;
pub use static_value::*;
pub use type_node::*;
pub use validate::*;
