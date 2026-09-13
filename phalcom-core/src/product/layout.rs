//! Physical product layout authority and component placement.

/// The physical representation kind of one product component slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductSlotRepr {
    Int64,
    Float64,
    Bool,
    Symbol,
    Value,
}

impl ProductSlotRepr {
    /// Number of 64-bit words this representation occupies in [`super::ProductStorage`].
    #[inline]
    pub const fn word_size(self) -> u32 {
        match self {
            Self::Int64 | Self::Float64 | Self::Bool | Self::Symbol => 1,
            Self::Value => 2,
        }
    }
}

/// Placement and representation of one logical component in product storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProductComponentLayout {
    pub logical_index: u32,
    pub word_offset: u32,
    pub repr: ProductSlotRepr,
}

/// Identifies an interned [`ProductLayout`] in [`super::ProductLayoutRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProductLayoutId(pub u32);

/// Physical layout specification for a packed product (data instance or enum payload).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductLayout {
    pub word_len: u32,
    pub components: Box<[ProductComponentLayout]>,
    value_slot_offsets: Box<[u32]>,
}

impl ProductLayout {
    /// Builds and validates a new `ProductLayout` from component descriptions.
    pub fn new(components: Vec<ProductComponentLayout>) -> Result<Self, &'static str> {
        let mut sorted = components;
        sorted.sort_by_key(|c| c.logical_index);

        for (i, c) in sorted.iter().enumerate() {
            if c.logical_index != i as u32 {
                return Err("product layout logical indexes must be dense 0..N");
            }
        }

        // Validate non-overlapping word offsets and compute word_len
        let mut max_word = 0u32;
        let mut value_offsets = Vec::new();
        for c in &sorted {
            let end_offset = c.word_offset.checked_add(c.repr.word_size()).ok_or("word offset overflow")?;
            if end_offset > max_word {
                max_word = end_offset;
            }
            if c.repr == ProductSlotRepr::Value {
                value_offsets.push(c.word_offset);
            }
        }

        // Check for overlapping intervals
        for i in 0..sorted.len() {
            let a = &sorted[i];
            let a_end = a.word_offset + a.repr.word_size();
            for j in (i + 1)..sorted.len() {
                let b = &sorted[j];
                let b_end = b.word_offset + b.repr.word_size();
                if a.word_offset < b_end && b.word_offset < a_end {
                    return Err("overlapping word offsets in product layout");
                }
            }
        }

        Ok(Self {
            word_len: max_word,
            components: sorted.into_boxed_slice(),
            value_slot_offsets: value_offsets.into_boxed_slice(),
        })
    }

    /// Returns a layout for an empty/nullary product.
    pub fn empty() -> Self {
        Self {
            word_len: 0,
            components: Box::new([]),
            value_slot_offsets: Box::new([]),
        }
    }

    /// Word offsets of all two-word `Value` slots for precise GC tracing.
    #[inline]
    pub fn value_slot_offsets(&self) -> &[u32] {
        &self.value_slot_offsets
    }

    /// Returns true if this product layout has zero components (nullary).
    pub fn is_nullary(&self) -> bool {
        self.components.is_empty()
    }

    /// Finds a component by its logical index.
    pub fn component(&self, logical_index: u32) -> Option<&ProductComponentLayout> {
        self.components.get(logical_index as usize)
    }
}

/// Description of one logical component for building a [`ProductLayout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProductComponentSpec {
    pub logical_index: u32,
    pub repr: ProductSlotRepr,
}

/// Physical layout specification for a packed product before offset calculation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductLayoutSpec {
    pub components: Box<[ProductComponentSpec]>,
}

impl ProductLayoutSpec {
    pub fn new(components: Vec<ProductComponentSpec>) -> Self {
        Self {
            components: components.into_boxed_slice(),
        }
    }

    pub fn empty() -> Self {
        Self {
            components: Box::new([]),
        }
    }

    /// Computes dense word offsets and builds the concrete [`ProductLayout`].
    pub fn build_layout(&self) -> Result<ProductLayout, &'static str> {
        let mut word_offset = 0u32;
        let mut comp_layouts = Vec::with_capacity(self.components.len());
        for comp in self.components.iter() {
            comp_layouts.push(ProductComponentLayout {
                logical_index: comp.logical_index,
                word_offset,
                repr: comp.repr,
            });
            word_offset = word_offset.checked_add(comp.repr.word_size()).ok_or("word offset overflow")?;
        }
        ProductLayout::new(comp_layouts)
    }
}

