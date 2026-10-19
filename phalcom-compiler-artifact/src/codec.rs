//! Little-endian bounded binary serialization and deserialization primitives.

use crate::digest::{ArtifactDigest, ArtifactHasher};
use crate::error::CodecError;

#[derive(Clone, Debug)]
enum BinaryWriterSink {
    Buffer(Vec<u8>),
    Digest(ArtifactHasher),
}

/// Streaming little-endian serializer.
///
/// Normal writers retain encoded bytes. Digest writers consume the exact same
/// canonical wire writes directly into a domain-separated artifact hasher and
/// therefore require no intermediate allocation.
#[derive(Clone, Debug)]
pub struct BinaryWriter {
    sink: BinaryWriterSink,
    len: usize,
}

impl Default for BinaryWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self {
            sink: BinaryWriterSink::Buffer(Vec::new()),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sink: BinaryWriterSink::Buffer(Vec::with_capacity(capacity)),
            len: 0,
        }
    }

    /// Creates an allocation-free writer that hashes canonical wire bytes
    /// under the supplied artifact domain.
    pub fn digest(domain: &str) -> Self {
        Self {
            sink: BinaryWriterSink::Digest(ArtifactHasher::new_domain(domain)),
            len: 0,
        }
    }

    fn write_raw(&mut self, bytes: &[u8]) {
        match &mut self.sink {
            BinaryWriterSink::Buffer(buffer) => buffer.extend_from_slice(bytes),
            BinaryWriterSink::Digest(hasher) => hasher.write(bytes),
        }
        self.len = self.len.checked_add(bytes.len()).expect("binary writer length overflow");
    }

    pub fn write_u8(&mut self, val: u8) {
        self.write_raw(&[val]);
    }

    pub fn write_u16(&mut self, val: u16) {
        self.write_raw(&val.to_le_bytes());
    }

    pub fn write_u32(&mut self, val: u32) {
        self.write_raw(&val.to_le_bytes());
    }

    pub fn write_u64(&mut self, val: u64) {
        self.write_raw(&val.to_le_bytes());
    }

    pub fn write_bool(&mut self, val: bool) {
        self.write_u8(if val { 1 } else { 0 });
    }

    pub fn write_fixed<const N: usize>(&mut self, bytes: &[u8; N]) {
        self.write_raw(bytes);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_raw(bytes);
    }

    pub fn write_bytes_with_len(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_bytes_with_len(s.as_bytes());
    }

    pub fn write_optional<T>(&mut self, opt: Option<&T>, mut f: impl FnMut(&mut Self, &T)) {
        match opt {
            Some(val) => {
                self.write_u8(1);
                f(self, val);
            }
            None => {
                self.write_u8(0);
            }
        }
    }

    pub fn write_vec<T>(&mut self, items: &[T], mut f: impl FnMut(&mut Self, &T)) {
        self.write_u32(items.len() as u32);
        for item in items {
            f(self, item);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        match &self.sink {
            BinaryWriterSink::Buffer(buffer) => buffer,
            BinaryWriterSink::Digest(_) => panic!("digest BinaryWriter has no retained byte slice"),
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        match self.sink {
            BinaryWriterSink::Buffer(buffer) => buffer,
            BinaryWriterSink::Digest(_) => panic!("digest BinaryWriter cannot produce encoded bytes"),
        }
    }

    pub fn into_digest(self) -> ArtifactDigest {
        match self.sink {
            BinaryWriterSink::Digest(hasher) => hasher.finalize(),
            BinaryWriterSink::Buffer(_) => panic!("buffer BinaryWriter has no streaming digest"),
        }
    }
}

/// Zero-copy bounded binary reader with explicit validation budgets.
#[derive(Clone, Copy, Debug)]
pub struct BinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn has_remaining(&self) -> bool {
        self.pos < self.data.len()
    }

    pub fn read_u8(&mut self) -> Result<u8, CodecError> {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Ok(b)
        } else {
            Err(CodecError::UnexpectedEof)
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, CodecError> {
        let bytes = self.read_fixed::<2>()?;
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn read_u32(&mut self) -> Result<u32, CodecError> {
        let bytes = self.read_fixed::<4>()?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn read_u64(&mut self) -> Result<u64, CodecError> {
        let bytes = self.read_fixed::<8>()?;
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn read_bool(&mut self) -> Result<bool, CodecError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(CodecError::InvalidBool(other)),
        }
    }

    pub fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], CodecError> {
        if self.remaining() < N {
            return Err(CodecError::UnexpectedEof);
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(&self.data[self.pos..self.pos + N]);
        self.pos += N;
        Ok(arr)
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], CodecError> {
        if self.remaining() < len {
            return Err(CodecError::UnexpectedEof);
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    pub fn read_bytes_with_len(&mut self, max_len: usize) -> Result<&'a [u8], CodecError> {
        let len = self.read_u32()? as usize;
        if len > max_len {
            return Err(CodecError::LimitExceeded {
                item: "bytes",
                count: len,
                limit: max_len,
            });
        }
        self.read_slice(len)
    }

    pub fn read_str(&mut self, max_len: usize) -> Result<&'a str, CodecError> {
        let bytes = self.read_bytes_with_len(max_len)?;
        std::str::from_utf8(bytes).map_err(|_| CodecError::InvalidUtf8)
    }

    pub fn read_optional<T>(&mut self, mut f: impl FnMut(&mut Self) -> Result<T, CodecError>) -> Result<Option<T>, CodecError> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => f(self).map(Some),
            other => Err(CodecError::InvalidBool(other)),
        }
    }

    pub fn read_vec<T>(
        &mut self,
        max_items: usize,
        item_name: &'static str,
        mut f: impl FnMut(&mut Self) -> Result<T, CodecError>,
    ) -> Result<Vec<T>, CodecError> {
        let count = self.read_u32()? as usize;
        if count > max_items {
            return Err(CodecError::LimitExceeded {
                item: item_name,
                count,
                limit: max_items,
            });
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(f(self)?);
        }
        Ok(items)
    }
}
