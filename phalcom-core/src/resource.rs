use phalcom_common::range::SourceRange;
use std::fmt;

/// A generation-tagged handle into `VM::resources` (PDR-0005 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle {
    pub index: u32,
    pub generation: u32,
}

pub const RESOURCE_MAX_INDEX: u32 = (1 << 21) - 1;
pub const RESOURCE_MAX_SLOTS: usize = (RESOURCE_MAX_INDEX as usize) + 1;
pub const RESOURCE_MAX_HANDLE: u64 = (1 << 53) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceHandleError {
    IndexOutOfRange,
    NonFinite,
    Negative,
    Fractional,
    OutOfRange,
}

impl ResourceHandle {
    /// Packs the bounded handle domain into an exactly representable binary64 integer.
    pub fn pack(self) -> Result<f64, ResourceHandleError> {
        if self.index > RESOURCE_MAX_INDEX {
            return Err(ResourceHandleError::IndexOutOfRange);
        }
        Ok((((self.index as u64) << 32) | self.generation as u64) as f64)
    }

    /// Decodes only exact, non-negative, bounded integer carriers.
    pub fn unpack(value: f64) -> Result<Self, ResourceHandleError> {
        if !value.is_finite() {
            return Err(ResourceHandleError::NonFinite);
        }
        if value < 0.0 {
            return Err(ResourceHandleError::Negative);
        }
        if value.fract() != 0.0 {
            return Err(ResourceHandleError::Fractional);
        }
        if value > RESOURCE_MAX_HANDLE as f64 {
            return Err(ResourceHandleError::OutOfRange);
        }
        let packed = value as u64;
        if packed as f64 != value {
            return Err(ResourceHandleError::OutOfRange);
        }
        let index = (packed >> 32) as u32;
        if index > RESOURCE_MAX_INDEX {
            return Err(ResourceHandleError::IndexOutOfRange);
        }
        Ok(Self {
            index,
            generation: packed as u32,
        })
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
    state: ResourceState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceState {
    Open,
    Closed,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    StaleHandle,
    AlreadyClosed,
    Capacity,
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

    pub fn open(&mut self, kind: ResourceKind, site: Option<SourceRange>) -> Result<ResourceHandle, ResourceError> {
        if let Some(index) = self.free_list.pop() {
            let entry = &mut self.entries[index as usize];
            let Some(generation) = entry.generation.checked_add(1) else {
                entry.state = ResourceState::Retired;
                return Err(ResourceError::Capacity);
            };
            entry.generation = generation;
            entry.kind = kind;
            entry.open_site = site;
            entry.state = ResourceState::Open;
            Ok(ResourceHandle { index, generation })
        } else {
            let index = u32::try_from(self.entries.len()).map_err(|_| ResourceError::Capacity)?;
            if self.entries.len() >= RESOURCE_MAX_SLOTS {
                return Err(ResourceError::Capacity);
            }
            let generation = 0;
            self.entries.push(ResourceEntry {
                generation,
                kind,
                open_site: site,
                state: ResourceState::Open,
            });
            Ok(ResourceHandle { index, generation })
        }
    }

    pub fn resolve(&mut self, handle: ResourceHandle) -> Result<&mut ResourceEntry, ResourceError> {
        let entry = self.entries.get_mut(handle.index as usize).ok_or(ResourceError::StaleHandle)?;
        if entry.generation != handle.generation {
            return Err(ResourceError::StaleHandle);
        }
        if entry.state != ResourceState::Open {
            return Err(ResourceError::AlreadyClosed);
        }
        Ok(entry)
    }

    pub fn is_closed(&self, handle: ResourceHandle) -> bool {
        match self.entries.get(handle.index as usize) {
            Some(entry) if entry.generation == handle.generation => entry.state != ResourceState::Open,
            _ => true,
        }
    }

    pub fn close(&mut self, handle: ResourceHandle) -> Result<(), ResourceError> {
        let entry = self.entries.get_mut(handle.index as usize).ok_or(ResourceError::StaleHandle)?;
        if entry.generation != handle.generation {
            return Err(ResourceError::StaleHandle);
        }
        if entry.state != ResourceState::Open {
            // Idempotent close per stream-protocol §3.1 law 4
            return Ok(());
        }
        entry.state = if entry.generation == u32::MAX {
            ResourceState::Retired
        } else {
            ResourceState::Closed
        };
        if entry.state == ResourceState::Closed {
            self.free_list.push(handle.index);
        }
        Ok(())
    }

    pub fn drain(&mut self) {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            if entry.state == ResourceState::Open {
                entry.state = if entry.generation == u32::MAX {
                    ResourceState::Retired
                } else {
                    ResourceState::Closed
                };
                if entry.state == ResourceState::Closed {
                    self.free_list.push(index as u32);
                }
            }
        }
    }

    pub fn leaks(&self) -> Vec<(&ResourceKind, Option<SourceRange>)> {
        self.entries
            .iter()
            .filter(|e| e.state == ResourceState::Open)
            .map(|e| (&e.kind, e.open_site))
            .collect()
    }

    pub fn leaks_detail(&self) -> Vec<(u32, &ResourceKind, Option<SourceRange>)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|e| e.1.state == ResourceState::Open)
            .map(|(idx, e)| (idx as u32, &e.kind, e.open_site))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_pack_unpack_exact_bounded_domain() {
        let cases = [
            ResourceHandle { index: 0, generation: 0 },
            ResourceHandle { index: 1, generation: 1 },
            ResourceHandle {
                index: (1 << 21) - 1,
                generation: u32::MAX,
            },
        ];

        for handle in cases {
            let packed = ResourceHandle::pack(handle).expect("valid handle must pack");
            assert_eq!(ResourceHandle::unpack(packed), Ok(handle));
        }

        assert_eq!(
            ResourceHandle {
                index: u32::MAX,
                generation: u32::MAX,
            }
            .pack(),
            Err(ResourceHandleError::IndexOutOfRange)
        );
    }

    #[test]
    fn test_table_open_close_stale() {
        let mut table = ResourceTable::new();
        let h1 = table.open(ResourceKind::Custom("Test".to_string()), None).unwrap();
        assert!(!table.is_closed(h1));

        assert!(table.close(h1).is_ok());
        assert!(table.is_closed(h1));
        // Idempotent close
        assert!(table.close(h1).is_ok());

        let h1_closed = ResourceHandle {
            index: h1.index,
            generation: h1.generation,
        };
        assert_eq!(table.resolve(h1_closed), Err(ResourceError::AlreadyClosed));

        // Reuse slot
        let h2 = table.open(ResourceKind::Custom("Test2".to_string()), None).unwrap();
        assert_eq!(h2.index, h1.index);
        assert_ne!(h2.generation, h1.generation);
        assert!(!table.is_closed(h2));
        assert_eq!(table.resolve(h1_closed), Err(ResourceError::StaleHandle));
        assert_eq!(table.close(h1_closed), Err(ResourceError::StaleHandle));
        assert!(!table.is_closed(h2), "stale A must never close reused B");
    }

    #[test]
    fn max_generation_retires_without_reuse_or_wrap() {
        let mut table = ResourceTable {
            entries: vec![ResourceEntry {
                generation: u32::MAX,
                kind: ResourceKind::Custom("max".to_string()),
                open_site: None,
                state: ResourceState::Open,
            }],
            free_list: Vec::new(),
        };
        let handle = ResourceHandle {
            index: 0,
            generation: u32::MAX,
        };
        assert!(table.close(handle).is_ok());
        assert!(table.is_closed(handle));
        assert!(table.close(handle).is_ok());
        let next = table.open(ResourceKind::Custom("next".to_string()), None).unwrap();
        assert_ne!(next.index, handle.index);
        table.close(next).unwrap();
        table.free_list.push(handle.index);
        assert_eq!(table.open(ResourceKind::Custom("retired".to_string()), None), Err(ResourceError::Capacity));
    }
}
