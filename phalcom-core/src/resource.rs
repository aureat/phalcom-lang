use phalcom_common::range::SourceRange;
use std::fmt;

/// A generation-tagged handle into [`VM::resources`] (PDR-0005 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle {
    pub index: u32,
    pub generation: u32,
}

impl ResourceHandle {
    /// Packs a handle (index, generation) into a single f64 number.
    /// index is upper 32 bits, generation is lower 32 bits.
    pub fn pack(index: u32, generation: u32) -> f64 {
        let packed = ((index as u64) << 32) | (generation as u64);
        packed as f64
    }

    /// Unpacks an f64 number back into a ResourceHandle.
    pub fn unpack(val: f64) -> ResourceHandle {
        let packed = val as u64;
        let index = (packed >> 32) as u32;
        let generation = packed as u32;
        ResourceHandle { index, generation }
    }
}

/// Resource kinds tracked in the resource table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Custom(String),
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceKind::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// One live or closed table row in the resource table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry {
    pub generation: u32,
    pub kind: ResourceKind,
    pub open_site: Option<SourceRange>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    StaleHandle,
    AlreadyClosed,
}

/// A generation-tagged table of native resources on the VM.
///
/// THE TABLE IS A GC ROOT FOR NOTHING (PDR-0005 §4): It contains no `Value` or `ObjRef`.
#[derive(Debug, Default)]
pub struct ResourceTable {
    entries: Vec<ResourceEntry>,
    free_list: Vec<u32>,
}

impl ResourceTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn open(&mut self, kind: ResourceKind, site: Option<SourceRange>) -> ResourceHandle {
        if let Some(index) = self.free_list.pop() {
            let entry = &mut self.entries[index as usize];
            entry.generation = entry.generation.wrapping_add(1);
            entry.kind = kind;
            entry.open_site = site;
            entry.closed = false;
            ResourceHandle {
                index,
                generation: entry.generation,
            }
        } else {
            let index = self.entries.len() as u32;
            let generation = 1;
            self.entries.push(ResourceEntry {
                generation,
                kind,
                open_site: site,
                closed: false,
            });
            ResourceHandle { index, generation }
        }
    }

    pub fn resolve(&mut self, handle: ResourceHandle) -> Result<&mut ResourceEntry, ResourceError> {
        let entry = self.entries.get_mut(handle.index as usize).ok_or(ResourceError::StaleHandle)?;
        if entry.generation != handle.generation {
            return Err(ResourceError::StaleHandle);
        }
        if entry.closed {
            return Err(ResourceError::AlreadyClosed);
        }
        Ok(entry)
    }

    pub fn is_closed(&self, handle: ResourceHandle) -> bool {
        match self.entries.get(handle.index as usize) {
            Some(entry) if entry.generation == handle.generation => entry.closed,
            _ => true,
        }
    }

    pub fn close(&mut self, handle: ResourceHandle) -> Result<(), ResourceError> {
        let entry = self.entries.get_mut(handle.index as usize).ok_or(ResourceError::StaleHandle)?;
        if entry.generation != handle.generation {
            return Err(ResourceError::StaleHandle);
        }
        if entry.closed {
            // Idempotent close per stream-protocol §3.1 law 4
            return Ok(());
        }
        entry.closed = true;
        self.free_list.push(handle.index);
        Ok(())
    }

    pub fn drain(&mut self) {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            if !entry.closed {
                entry.closed = true;
                entry.generation = entry.generation.wrapping_add(1);
                self.free_list.push(index as u32);
            }
        }
    }

    pub fn leaks(&self) -> Vec<(&ResourceKind, Option<SourceRange>)> {
        self.entries
            .iter()
            .filter(|e| !e.closed)
            .map(|e| (&e.kind, e.open_site))
            .collect()
    }

    pub fn leaks_detail(&self) -> Vec<(u32, &ResourceKind, Option<SourceRange>)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|e| !e.1.closed)
            .map(|(idx, e)| (idx as u32, &e.kind, e.open_site))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_pack_unpack_boundary() {
        let h1 = ResourceHandle { index: 0, generation: 1 };
        let packed1 = ResourceHandle::pack(h1.index, h1.generation);
        let unpacked1 = ResourceHandle::unpack(packed1);
        assert_eq!(h1, unpacked1);

        let h2 = ResourceHandle { index: u32::MAX, generation: u32::MAX };
        let packed2 = ResourceHandle::pack(h2.index, h2.generation);
        let unpacked2 = ResourceHandle::unpack(packed2);
        assert_eq!(h2, unpacked2);
    }

    #[test]
    fn test_table_open_close_stale() {
        let mut table = ResourceTable::new();
        let h1 = table.open(ResourceKind::Custom("Test".to_string()), None);
        assert!(!table.is_closed(h1));

        assert!(table.close(h1).is_ok());
        assert!(table.is_closed(h1));
        // Idempotent close
        assert!(table.close(h1).is_ok());

        let h1_closed = ResourceHandle { index: h1.index, generation: h1.generation };
        assert_eq!(table.resolve(h1_closed), Err(ResourceError::AlreadyClosed));

        // Reuse slot
        let h2 = table.open(ResourceKind::Custom("Test2".to_string()), None);
        assert_eq!(h2.index, h1.index);
        assert_ne!(h2.generation, h1.generation);
        assert!(!table.is_closed(h2));
        assert_eq!(table.resolve(h1_closed), Err(ResourceError::StaleHandle));
    }
}
