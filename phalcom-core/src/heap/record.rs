//! Immutable positive-arity record storage.

use crate::interner::Symbol;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct RecordObject {
    labels: Box<[Symbol]>,
    values: Box<[Value]>,
}

impl RecordObject {
    pub(crate) fn new(labels: Box<[Symbol]>, values: Box<[Value]>) -> Self {
        assert!(!labels.is_empty(), "RecordObject must be positive-arity");
        assert_eq!(labels.len(), values.len(), "record labels and values must align");
        assert!(
            labels.iter().enumerate().all(|(i, label)| !labels[..i].contains(label)),
            "record labels must be unique"
        );
        Self { labels, values }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn labels(&self) -> &[Symbol] {
        &self.labels
    }
    pub fn values(&self) -> &[Value] {
        &self.values
    }
    pub fn get(&self, label: Symbol) -> Option<Value> {
        self.labels.iter().position(|candidate| *candidate == label).map(|i| self.values[i])
    }
    pub fn entries(&self) -> impl Iterator<Item = (Symbol, Value)> + '_ {
        self.labels.iter().copied().zip(self.values.iter().copied())
    }
}
