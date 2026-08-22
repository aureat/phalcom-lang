use phalcom_common::selector::Selector;
use crate::interner::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorObject {
    pub selector: Selector,
    pub symbol: Symbol,
}
