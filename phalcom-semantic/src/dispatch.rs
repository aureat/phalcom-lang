//! Dispatch models and selector resolution helpers.

pub use crate::identity::DispatchSide;
use phalcom_common::selector::Selector;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchTarget {
    pub selector: Selector,
    pub side: DispatchSide,
}
