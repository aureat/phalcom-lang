//! Interned registry of physical product layouts.

use super::layout::{ProductLayout, ProductLayoutId};
use std::collections::HashMap;
use std::sync::Arc;

/// Heap-owned registry interning physical product layouts.
#[derive(Debug, Clone, Default)]
pub struct ProductLayoutRegistry {
    layouts: Vec<Arc<ProductLayout>>,
    interner: HashMap<ProductLayout, ProductLayoutId>,
}

impl ProductLayoutRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            layouts: Vec::new(),
            interner: HashMap::new(),
        };
        // Register default empty layout at id 0
        let empty = ProductLayout::empty();
        reg.register(empty);
        reg
    }

    /// Registers/interns a physical layout and returns its unique ID.
    pub fn register(&mut self, layout: ProductLayout) -> ProductLayoutId {
        if let Some(&id) = self.interner.get(&layout) {
            return id;
        }
        let id = ProductLayoutId(self.layouts.len() as u32);
        let arc = Arc::new(layout.clone());
        self.layouts.push(arc);
        self.interner.insert(layout, id);
        id
    }

    /// Looks up an interned layout by its ID.
    pub fn get(&self, id: ProductLayoutId) -> Option<&ProductLayout> {
        self.layouts.get(id.0 as usize).map(|a| a.as_ref())
    }
}
