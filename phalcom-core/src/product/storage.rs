//! Word-packed product storage for data objects and enum payloads.

use super::layout::{ProductLayout, ProductLayoutId, ProductSlotRepr};
use crate::interner::Symbol;
use crate::value::Value;

/// Word-packed storage buffer for one product instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductStorage {
    pub layout: ProductLayoutId,
    pub words: Box<[u64]>,
}

impl ProductStorage {
    /// Encodes exactly one complete product before publication to the heap.
    pub fn from_values(layout_id: ProductLayoutId, layout: &ProductLayout, values: &[Value]) -> Result<Self, &'static str> {
        if values.len() != layout.components.len() {
            return Err("product component count does not match layout");
        }
        let mut storage = Self::new(layout_id, layout);
        for (index, value) in values.iter().copied().enumerate() {
            storage.store_component(layout, index as u32, value)?;
        }
        Ok(storage)
    }

    /// Allocates zeroed product storage matching the specified layout.
    pub fn new(layout_id: ProductLayoutId, layout: &ProductLayout) -> Self {
        Self {
            layout: layout_id,
            words: vec![0u64; layout.word_len as usize].into_boxed_slice(),
        }
    }

    /// Stores a component into the storage buffer according to its physical layout.
    pub fn store_component(&mut self, layout: &ProductLayout, logical_index: u32, value: Value) -> Result<(), &'static str> {
        let comp = layout.component(logical_index).ok_or("component index out of bounds")?;
        let offset = comp.word_offset as usize;

        match comp.repr {
            ProductSlotRepr::Int64 => {
                let int_val = value.as_int().ok_or("expected Int for Int64 component slot")?;
                self.words[offset] = int_val as u64;
            }
            ProductSlotRepr::Float64 => {
                let float_val = value.as_float().ok_or("expected Float for Float64 component slot")?;
                self.words[offset] = float_val.to_bits();
            }
            ProductSlotRepr::Bool => {
                let bool_val = value.as_bool().ok_or("expected Bool for Bool component slot")?;
                self.words[offset] = if bool_val { 1 } else { 0 };
            }
            ProductSlotRepr::Symbol => {
                let sym = value.symbol_value().ok_or("expected Symbol for Symbol component slot")?;
                self.words[offset] = sym.0 as u64;
            }
            ProductSlotRepr::Value => {
                let (payload, meta) = value.raw_words();
                self.words[offset] = payload;
                self.words[offset + 1] = meta;
            }
        }
        Ok(())
    }

    /// Loads a component from the storage buffer as a uniform `Value`.
    pub fn load_component(&self, layout: &ProductLayout, logical_index: u32) -> Result<Value, &'static str> {
        let comp = layout.component(logical_index).ok_or("component index out of bounds")?;
        let offset = comp.word_offset as usize;

        match comp.repr {
            ProductSlotRepr::Int64 => {
                let raw = self.words[offset] as i64;
                Ok(Value::int(raw))
            }
            ProductSlotRepr::Float64 => {
                let raw = f64::from_bits(self.words[offset]);
                Ok(Value::float(raw))
            }
            ProductSlotRepr::Bool => {
                let raw = self.words[offset] != 0;
                Ok(Value::bool(raw))
            }
            ProductSlotRepr::Symbol => {
                let raw = Symbol(self.words[offset] as u32);
                Ok(Value::symbol(raw))
            }
            ProductSlotRepr::Value => {
                let payload = self.words[offset];
                let meta = self.words[offset + 1];
                Ok(Value::from_raw_words(payload, meta))
            }
        }
    }
}
