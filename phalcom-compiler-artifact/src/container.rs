//! `.telosc` binary container encoding, decoding, and structural validation.

use crate::codec::{BinaryReader, BinaryWriter};
use crate::digest::{ArtifactDigest, ArtifactHasher, DOMAIN_ARTIFACT, DOMAIN_SECTION};
use crate::error::{ArtifactError, CodecError, ContainerError};
use crate::header::{TeloscCapabilities, TeloscHeaderV1};
use crate::limits::ValidationLimits;
use crate::section::{
    SectionDirectoryEntryV1, SectionFlags, SectionKind, SECTION_DIRECTORY_ENTRY_SIZE,
};
use std::collections::HashSet;

/// Builder for constructing deterministic `.telosc` container artifacts.
pub struct TeloscContainerBuilder {
    header: TeloscHeaderV1,
    sections: Vec<PendingSection>,
}

struct PendingSection {
    kind: SectionKind,
    schema_version: u32,
    flags: SectionFlags,
    data: Vec<u8>,
}

impl TeloscContainerBuilder {
    pub fn new(header: TeloscHeaderV1) -> Self {
        Self {
            header,
            sections: Vec::new(),
        }
    }

    pub fn add_section(
        &mut self,
        kind: SectionKind,
        schema_version: u32,
        flags: SectionFlags,
        data: Vec<u8>,
    ) {
        self.sections.push(PendingSection {
            kind,
            schema_version,
            flags,
            data,
        });
    }

    /// Assembles and serializes the complete container artifact.
    pub fn build(mut self) -> Result<Vec<u8>, ArtifactError> {
        // Sort sections canonically by SectionKind
        self.sections.sort_by_key(|s| s.kind.tag());

        self.header.section_count = self.sections.len() as u32;

        // First pass: encode dummy header to compute exact header length and artifact_digest_offset
        let mut dummy_writer = BinaryWriter::new();
        let _artifact_digest_offset = self.header.encode(&mut dummy_writer);
        let header_len = dummy_writer.len();

        let dir_len = self.sections.len() * SECTION_DIRECTORY_ENTRY_SIZE;
        let mut current_offset = (header_len + dir_len) as u64;

        // Compute directory entries and section digests
        let mut directory = Vec::with_capacity(self.sections.len());
        for sec in &self.sections {
            let encoded_len = sec.data.len() as u64;
            let decoded_len = encoded_len; // Uncompressed in v1
            let digest = ArtifactDigest::digest_domain(DOMAIN_SECTION, &sec.data);

            directory.push(SectionDirectoryEntryV1 {
                kind: sec.kind,
                schema_version: sec.schema_version,
                flags: sec.flags,
                offset: current_offset,
                encoded_len,
                decoded_len,
                digest,
            });

            current_offset += encoded_len;
        }

        // Serialize directory
        let mut dir_writer = BinaryWriter::with_capacity(dir_len);
        for entry in &directory {
            dir_writer.write_u32(entry.kind.tag());
            dir_writer.write_u32(entry.schema_version);
            dir_writer.write_u32(entry.flags.bits());
            dir_writer.write_u64(entry.offset);
            dir_writer.write_u64(entry.encoded_len);
            dir_writer.write_u64(entry.decoded_len);
            dir_writer.write_fixed(entry.digest.as_bytes());
        }
        let dir_bytes = dir_writer.into_vec();

        // Compute whole artifact digest
        // artifact_digest = digest(header with zeroed artifact_digest + directory + sections)
        let mut header_zeroed_writer = BinaryWriter::with_capacity(header_len);
        let mut zeroed_header = self.header.clone();
        zeroed_header.artifact_digest = ArtifactDigest::ZERO;
        zeroed_header.encode(&mut header_zeroed_writer);
        let header_zeroed_bytes = header_zeroed_writer.into_vec();

        let mut artifact_hasher = ArtifactHasher::new_domain(DOMAIN_ARTIFACT);
        artifact_hasher.write(&header_zeroed_bytes);
        artifact_hasher.write(&dir_bytes);
        for sec in &self.sections {
            artifact_hasher.write(&sec.data);
        }
        let final_artifact_digest = artifact_hasher.finalize();

        // Encode final header with the computed artifact_digest
        self.header.artifact_digest = final_artifact_digest;
        let mut final_header_writer = BinaryWriter::with_capacity(header_len);
        self.header.encode(&mut final_header_writer);
        let final_header_bytes = final_header_writer.into_vec();

        // Assemble full payload
        let total_size = header_len + dir_len + self.sections.iter().map(|s| s.data.len()).sum::<usize>();
        let mut output = Vec::with_capacity(total_size);
        output.extend_from_slice(&final_header_bytes);
        output.extend_from_slice(&dir_bytes);
        for sec in self.sections {
            output.extend_from_slice(&sec.data);
        }

        Ok(output)
    }
}

/// Zero-copy read-only view of a validated `.telosc` container.
#[derive(Clone, Debug)]
pub struct TeloscContainerView<'a> {
    pub header: TeloscHeaderV1,
    pub directory: Vec<SectionDirectoryEntryV1>,
    raw_data: &'a [u8],
}

impl<'a> TeloscContainerView<'a> {
    /// Parses and strictly validates a container binary buffer against resource limits.
    pub fn parse(data: &'a [u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if data.len() as u64 > limits.max_artifact_bytes {
            return Err(ArtifactError::LimitExceeded(format!(
                "artifact size ({} bytes) exceeds limit of {} bytes",
                data.len(),
                limits.max_artifact_bytes
            )));
        }

        let mut reader = BinaryReader::new(data);
        let (header, artifact_digest_offset) = TeloscHeaderV1::decode(&mut reader, limits)?;

        let dir_entry_count = header.section_count as usize;
        let dir_bytes_len = dir_entry_count * SECTION_DIRECTORY_ENTRY_SIZE;
        if reader.remaining() < dir_bytes_len {
            return Err(CodecError::UnexpectedEof.into());
        }

        let dir_start_pos = reader.position();
        let mut directory = Vec::with_capacity(dir_entry_count);
        let mut seen_kinds = HashSet::new();

        for _ in 0..dir_entry_count {
            let kind_tag = reader.read_u32()?;
            let kind = SectionKind::from_tag(kind_tag);
            let schema_version = reader.read_u32()?;
            let flags_bits = reader.read_u32()?;
            let flags = SectionFlags::from_bits(flags_bits).ok_or(ContainerError::UnsupportedSectionFlags(flags_bits))?;
            if flags.contains(SectionFlags::COMPRESSED) {
                return Err(ContainerError::UnsupportedSectionFlags(flags_bits).into());
            }
            if kind.is_known() && schema_version != 1 {
                return Err(ContainerError::UnsupportedSectionSchema { section: kind, schema: schema_version }.into());
            }
            let offset = reader.read_u64()?;
            let encoded_len = reader.read_u64()?;
            let decoded_len = reader.read_u64()?;
            let digest = ArtifactDigest(reader.read_fixed::<32>()?);

            // Duplicate section detection
            if !seen_kinds.insert(kind) {
                return Err(ContainerError::DuplicateSection(kind).into());
            }

            // Unknown required section rejection
            if matches!(kind, SectionKind::Unknown(_)) && flags.contains(SectionFlags::REQUIRED) {
                return Err(ContainerError::UnknownRequiredSection {
                    tag: kind.tag(),
                }
                .into());
            }

            // Section budget violation
            if encoded_len > limits.max_encoded_section_bytes {
                return Err(ContainerError::SectionBudgetViolation {
                    section: kind,
                    size: encoded_len,
                    budget: limits.max_encoded_section_bytes,
                }
                .into());
            }

            // Offset + length overflow check
            let end = offset
                .checked_add(encoded_len)
                .ok_or(ContainerError::OffsetOverflow {
                    section: kind,
                    offset,
                    len: encoded_len,
                })?;

            if end > data.len() as u64 {
                return Err(ContainerError::OutOfFileSection {
                    section: kind,
                    end,
                    file_len: data.len() as u64,
                }
                .into());
            }

            directory.push(SectionDirectoryEntryV1 {
                kind,
                schema_version,
                flags,
                offset,
                encoded_len,
                decoded_len,
                digest,
            });
        }
        let dir_end_pos = reader.position();

        // Header and directory cannot overlap with any section
        for entry in &directory {
            if entry.offset < dir_end_pos as u64 {
                return Err(ContainerError::OffsetOverflow {
                    section: entry.kind,
                    offset: entry.offset,
                    len: entry.encoded_len,
                }
                .into());
            }
        }

        // Overlapping sections validation: sort entries by offset
        let mut sorted_entries = directory.clone();
        sorted_entries.sort_by_key(|e| e.offset);
        for i in 0..sorted_entries.len().saturating_sub(1) {
            let current = &sorted_entries[i];
            let next = &sorted_entries[i + 1];
            let current_end = current.offset + current.encoded_len;
            if current_end > next.offset {
                return Err(ContainerError::OverlappingSections {
                    first: current.kind,
                    second: next.kind,
                }
                .into());
            }
        }

        // Section payload digest verification
        for entry in &directory {
            let sec_bytes = &data[entry.offset as usize..(entry.offset + entry.encoded_len) as usize];
            let actual_digest = ArtifactDigest::digest_domain(DOMAIN_SECTION, sec_bytes);
            if actual_digest != entry.digest {
                return Err(ContainerError::SectionDigestMismatch {
                    section: entry.kind,
                    expected: entry.digest,
                    actual: actual_digest,
                }
                .into());
            }
        }

        // Verify required sections are present
        if !seen_kinds.contains(&SectionKind::ModuleWorld) {
            return Err(ContainerError::MissingRequiredSection(SectionKind::ModuleWorld).into());
        }
        if !seen_kinds.contains(&SectionKind::SemanticWorld) {
            return Err(ContainerError::MissingRequiredSection(SectionKind::SemanticWorld).into());
        }
        if header
            .capabilities
            .contains(TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1)
            && !seen_kinds.contains(&SectionKind::QueryImage)
        {
            return Err(ContainerError::MissingRequiredSection(SectionKind::QueryImage).into());
        }
        if header
            .capabilities
            .contains(TeloscCapabilities::RUNTIME_PUBLIC_METADATA)
            && !seen_kinds.contains(&SectionKind::Metadata)
        {
            return Err(ContainerError::MissingRequiredSection(SectionKind::Metadata).into());
        }

        if header.capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE) && !seen_kinds.contains(&SectionKind::Executable) {
            return Err(ContainerError::MissingRequiredSection(SectionKind::Executable).into());
        }

        // Verify whole-artifact digest
        let mut artifact_hasher = ArtifactHasher::new_domain(DOMAIN_ARTIFACT);
        // Header with artifact_digest bytes zeroed
        artifact_hasher.write(&data[0..artifact_digest_offset]);
        artifact_hasher.write(&[0u8; 32]);
        artifact_hasher.write(&data[artifact_digest_offset + 32..dir_start_pos]);
        // Directory bytes
        artifact_hasher.write(&data[dir_start_pos..dir_end_pos]);
        // Section bytes in directory order
        for entry in &directory {
            let sec_bytes = &data[entry.offset as usize..(entry.offset + entry.encoded_len) as usize];
            artifact_hasher.write(sec_bytes);
        }
        let computed_artifact_digest = artifact_hasher.finalize();
        if computed_artifact_digest != header.artifact_digest {
            return Err(ContainerError::ArtifactDigestMismatch {
                expected: header.artifact_digest,
                actual: computed_artifact_digest,
            }
            .into());
        }

        Ok(Self {
            header,
            directory,
            raw_data: data,
        })
    }

    pub fn header(&self) -> &TeloscHeaderV1 {
        &self.header
    }

    pub fn directory(&self) -> &[SectionDirectoryEntryV1] {
        &self.directory
    }

    pub fn get_section(&self, kind: SectionKind) -> Option<&'a [u8]> {
        let entry = self.directory.iter().find(|e| e.kind == kind)?;
        let start = entry.offset as usize;
        let end = start + entry.encoded_len as usize;
        Some(&self.raw_data[start..end])
    }

    pub fn require_section(&self, kind: SectionKind) -> Result<&'a [u8], ContainerError> {
        self.get_section(kind)
            .ok_or(ContainerError::MissingRequiredSection(kind))
    }
}
