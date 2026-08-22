//! Kind models distinguishing value types from type constructors.

use super::id::KindId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum KindData {
    /// Fully saturated value type (`*` or `Type`).
    Type,
    /// Type constructor (`Kind -> Kind` or `(Kind, ...) -> Kind`).
    Arrow {
        parameters: Box<[KindId]>,
        result: KindId,
    },
}
