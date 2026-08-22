use crate::interner::Symbol;
use phalcom_common::selector::Selector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorObject {
    pub selector: Selector,
    pub symbol: Symbol,
}
