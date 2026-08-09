//! Private, GC-traced assembly state for dynamically shaped outgoing packs.

use crate::interner::Symbol;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackBuilderError {
    DuplicateLabel(Symbol),
    MissingPendingLabel,
    PendingLabelExists,
}

/// Compiler-owned, deliberately unobservable outgoing-argument accumulator.
#[derive(Default)]
pub struct ArgumentPackBuilderObject {
    positionals: Vec<Value>,
    labels: Vec<Symbol>,
    labeled_values: Vec<Value>,
    pending_label: Option<usize>,
}

impl ArgumentPackBuilderObject {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push_positional(&mut self, value: Value) {
        self.positionals.push(value);
    }

    pub fn reserve_label(&mut self, label: Symbol) -> Result<(), PackBuilderError> {
        if self.pending_label.is_some() {
            return Err(PackBuilderError::PendingLabelExists);
        }
        if self.labels.contains(&label) {
            return Err(PackBuilderError::DuplicateLabel(label));
        }
        self.labels.push(label);
        self.labeled_values.push(Value::Nil);
        self.pending_label = Some(self.labels.len() - 1);
        Ok(())
    }

    pub fn fill_reserved(&mut self, value: Value) -> Result<(), PackBuilderError> {
        let index = self.pending_label.take().ok_or(PackBuilderError::MissingPendingLabel)?;
        self.labeled_values[index] = value;
        Ok(())
    }

    pub fn append_labeled(&mut self, label: Symbol, value: Value) -> Result<(), PackBuilderError> {
        if self.pending_label.is_some() {
            return Err(PackBuilderError::PendingLabelExists);
        }
        if self.labels.contains(&label) {
            return Err(PackBuilderError::DuplicateLabel(label));
        }
        self.labels.push(label);
        self.labeled_values.push(value);
        Ok(())
    }

    pub fn has_pending(&self) -> bool {
        self.pending_label.is_some()
    }
    pub fn positionals(&self) -> &[Value] {
        &self.positionals
    }
    pub fn labels(&self) -> &[Symbol] {
        &self.labels
    }
    pub fn labeled_values(&self) -> &[Value] {
        &self.labeled_values
    }

    pub fn take_parts(&mut self) -> (Vec<Value>, Vec<Symbol>, Vec<Value>) {
        (
            std::mem::take(&mut self.positionals),
            std::mem::take(&mut self.labels),
            std::mem::take(&mut self.labeled_values),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reservation_targets_its_exact_placeholder() {
        let a = Symbol(1);
        let b = Symbol(2);
        let mut pack = ArgumentPackBuilderObject::new();
        pack.reserve_label(a).unwrap();
        pack.fill_reserved(Value::Int(1)).unwrap();
        pack.reserve_label(b).unwrap();
        pack.fill_reserved(Value::Int(2)).unwrap();
        assert_eq!(pack.labeled_values(), &[Value::Int(1), Value::Int(2)]);
        assert_eq!(pack.append_labeled(a, Value::Int(3)), Err(PackBuilderError::DuplicateLabel(a)));
    }
}
