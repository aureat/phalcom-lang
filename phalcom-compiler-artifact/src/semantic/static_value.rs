//! Portable static-value graph. Store lineages and native arena IDs never enter the wire format.

use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::semantic::type_store::ArtifactTypeId;
pub use crate::semantic::type_store::FieldKeyV1;
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::field_key::FieldKeyRef;
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};
use phalcom_type_meta::scalar::StaticInt;
use phalcom_type_meta::static_value::{StaticSequenceFamilyRef, StaticValueNode, StaticValueNodeEntry, StaticValueNodeId, compute_static_value_fingerprint};

pub const STATIC_VALUE_GRAPH_SCHEMA_V1: u32 = 1;

macro_rules! graph_id {
    ($($name:ident),* $(,)?) => { $(
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub u32);
        impl Wire for $name {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> { self.0.write(out) }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> { Ok(Self(u32::read(input)?)) }
        }
    )* };
}

graph_id!(
    ArtifactStaticValueId,
    ArtifactStaticStringId,
    ArtifactStaticSymbolId,
    ArtifactStaticTupleId,
    ArtifactStaticSequenceId,
    ArtifactStaticRecordId,
    ArtifactStaticDataId,
    ArtifactStaticVariantId,
    ArtifactStaticPackBackingId,
    ArtifactStaticPackId,
);

tagged!(SequenceFamilyV1 { 0 => Array, 1 => Vector });
record!(StaticSequenceV1 {
    family: SequenceFamilyV1,
    result_type: ArtifactTypeId,
    elements: Vec<ArtifactStaticValueId>,
});
record!(StaticRecordFieldV1 {
    key: FieldKeyV1,
    value: ArtifactStaticValueId,
});
record!(StaticDataComponentV1 {
    index: u32,
    value: ArtifactStaticValueId,
});
record!(StaticDataValueV1 {
    owner: StableDeclarationRef,
    constructor_owner: StableDeclarationRef,
    result_type: ArtifactTypeId,
    components: Vec<StaticDataComponentV1>,
});
record!(StaticVariantFieldV1 {
    index: u32,
    value: ArtifactStaticValueId,
});
record!(StaticVariantValueV1 {
    variant: StableVariantRef,
    result_type: ArtifactTypeId,
    fields: Vec<StaticVariantFieldV1>,
});
record!(StaticPackBackingV1 {
    positionals: Vec<ArtifactStaticValueId>,
    labels: Vec<ArtifactStaticSymbolId>,
    labeled_values: Vec<ArtifactStaticValueId>,
});
record!(StaticPackSliceV1 {
    backing: ArtifactStaticPackBackingId,
    positional_start: u32,
    positional_len: u32,
    labeled_start: u32,
    labeled_len: u32,
});

tagged!(StaticValueV1 limit max_static_values {
    0 => Unit,
    1 => Bool { value: bool },
    2 => Char { scalar: u32 },
    3 => Int { decimal: String },
    4 => Float { bits: u64 },
    5 => String { id: ArtifactStaticStringId },
    6 => Symbol { id: ArtifactStaticSymbolId },
    7 => Type { ty: ArtifactTypeId },
    8 => StaticTerm { ty: ArtifactTypeId },
    9 => Tuple { id: ArtifactStaticTupleId },
    10 => Sequence { id: ArtifactStaticSequenceId },
    11 => Record { id: ArtifactStaticRecordId },
    12 => Data { id: ArtifactStaticDataId },
    13 => Variant { id: ArtifactStaticVariantId },
    14 => Pack { id: ArtifactStaticPackId },
});

record!(StaticValueGraphV1 {
    values: Vec<StaticValueV1>,
    strings: Vec<String>,
    symbols: Vec<String>,
    tuples: Vec<Vec<ArtifactStaticValueId>>,
    sequences: Vec<StaticSequenceV1>,
    records: Vec<Vec<StaticRecordFieldV1>>,
    data_values: Vec<StaticDataValueV1>,
    variant_values: Vec<StaticVariantValueV1>,
    pack_backings: Vec<StaticPackBackingV1>,
    packs: Vec<StaticPackSliceV1>,
});

impl StaticValueGraphV1 {
    /// Encodes the graph after validating its local representation and edges.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(STATIC_VALUE_GRAPH_SCHEMA_V1);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes a bounded graph and rejects malformed local references and cycles.
    /// Type and declaration contracts are validated by the semantic importer.
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > limits.max_decoded_section_bytes {
            return Err(invalid("static-value graph byte budget exceeded"));
        }
        let allocation_limit =
            usize::try_from(limits.max_decoded_section_bytes).map_err(|_| invalid("static-value graph allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations: &mut allocations,
            items_left: limits.max_semantic_entries_per_family,
            depth: 0,
        };
        if input.reader.read_u32()? != STATIC_VALUE_GRAPH_SCHEMA_V1 {
            return Err(invalid("unsupported static-value graph schema"));
        }
        let graph = Self::read(&mut input)?;
        if input.reader.has_remaining() {
            return Err(invalid("trailing static-value graph bytes"));
        }
        graph.validate(limits)?;
        Ok(graph)
    }

    /// Validates scalar representations, all local edges, and the static DAG.
    /// Returns dependency-first value order for factory-based reconstruction.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<Vec<ArtifactStaticValueId>, ArtifactError> {
        self.validate_inner(limits, ValidationScratch::Untracked)
    }

    /// Validates the graph while charging traversal scratch to the enclosing
    /// semantic import transaction. The ordinary validator is retained for
    /// standalone wire validation, while semantic loading must account for the
    /// actual vectors used by the cycle/dependency traversal.
    pub fn validate_with_allocation_budget(
        &self,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<Vec<ArtifactStaticValueId>, ArtifactError> {
        self.validate_inner(limits, ValidationScratch::Budgeted(allocations))
    }

    fn validate_inner(&self, limits: &ValidationLimits, mut scratch: ValidationScratch<'_>) -> Result<Vec<ArtifactStaticValueId>, ArtifactError> {
        let mut budget = GraphBudget::new(limits);
        budget.collection::<StaticValueV1>(self.values.len(), limits.max_static_values)?;
        budget.collection::<String>(self.strings.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<String>(self.symbols.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<Vec<ArtifactStaticValueId>>(self.tuples.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<StaticSequenceV1>(self.sequences.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<Vec<StaticRecordFieldV1>>(self.records.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<StaticDataValueV1>(self.data_values.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<StaticVariantValueV1>(self.variant_values.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<StaticPackBackingV1>(self.pack_backings.len(), limits.max_semantic_entries_per_family)?;
        budget.collection::<StaticPackSliceV1>(self.packs.len(), limits.max_semantic_entries_per_family)?;
        for text in self.strings.iter().chain(&self.symbols) {
            budget.text(text)?;
        }
        for tuple in &self.tuples {
            // Empty native tuples canonicalize to Unit; accepting an empty row
            // would lose its graph identity during ordinary reconstruction.
            if tuple.is_empty() {
                return Err(malformed("empty static tuple must be represented as Unit"));
            }
            self.check_children(tuple, &mut budget)?;
        }
        for sequence in &self.sequences {
            self.check_children(&sequence.elements, &mut budget)?;
        }
        for record in &self.records {
            budget.collection::<StaticRecordFieldV1>(record.len(), limits.max_semantic_entries_per_family)?;
            let mut previous = None;
            for field in record {
                check_index(field.value.0, self.values.len(), "record child")?;
                validate_key(&field.key, &mut budget)?;
                if let Some(previous) = previous
                    && compare_keys(previous, &field.key) != std::cmp::Ordering::Less
                {
                    return Err(malformed("static record keys must be unique and canonically ordered"));
                }
                previous = Some(&field.key);
            }
        }
        for data in &self.data_values {
            budget.owner(&data.owner)?;
            budget.owner(&data.constructor_owner)?;
            if data.owner != data.constructor_owner {
                return Err(malformed("static data constructor has a foreign owner"));
            }
            budget.collection::<StaticDataComponentV1>(data.components.len(), limits.max_semantic_entries_per_family)?;
            // Index order is declaration order, not input-map encounter order.
            for (index, component) in data.components.iter().enumerate() {
                if component.index as usize != index {
                    return Err(malformed("static data component indexes are not contiguous"));
                }
                check_index(component.value.0, self.values.len(), "data child")?;
            }
        }
        for variant in &self.variant_values {
            budget.owner(&variant.variant.owner)?;
            budget.text(&variant.variant.selector)?;
            budget.collection::<StaticVariantFieldV1>(variant.fields.len(), limits.max_semantic_entries_per_family)?;
            for (index, field) in variant.fields.iter().enumerate() {
                if field.index as usize != index {
                    return Err(malformed("static variant field indexes are not contiguous"));
                }
                check_index(field.value.0, self.values.len(), "variant child")?;
            }
        }
        for backing in &self.pack_backings {
            self.check_children(&backing.positionals, &mut budget)?;
            self.check_children(&backing.labeled_values, &mut budget)?;
            budget.collection::<ArtifactStaticSymbolId>(backing.labels.len(), limits.max_semantic_entries_per_family)?;
            if backing.labels.len() != backing.labeled_values.len() {
                return Err(malformed("static pack labels and values have different lengths"));
            }
            let mut labels = scratch.vector::<&str>(backing.labels.len())?;
            for label in &backing.labels {
                check_index(label.0, self.symbols.len(), "pack label")?;
                budget.scratch(std::mem::size_of::<usize>() * 4)?;
                labels.push(self.symbols[label.0 as usize].as_str());
            }
            labels.sort_unstable();
            if labels.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(malformed("duplicate static pack label"));
            }
        }
        for pack in &self.packs {
            check_index(pack.backing.0, self.pack_backings.len(), "pack backing")?;
            let backing = &self.pack_backings[pack.backing.0 as usize];
            check_range(pack.positional_start, pack.positional_len, backing.positionals.len())?;
            check_range(pack.labeled_start, pack.labeled_len, backing.labels.len())?;
        }
        for value in &self.values {
            match value {
                StaticValueV1::Unit
                | StaticValueV1::Bool { .. }
                | StaticValueV1::Float { .. }
                | StaticValueV1::Type { .. }
                | StaticValueV1::StaticTerm { .. } => {}
                StaticValueV1::Char { scalar } => {
                    if char::from_u32(*scalar).is_none() {
                        return Err(malformed("invalid static character scalar"));
                    }
                }
                StaticValueV1::Int { decimal } => {
                    budget.text(decimal)?;
                    validate_decimal(decimal)?;
                }
                StaticValueV1::String { id } => check_index(id.0, self.strings.len(), "string")?,
                StaticValueV1::Symbol { id } => check_index(id.0, self.symbols.len(), "symbol")?,
                StaticValueV1::Tuple { id } => check_index(id.0, self.tuples.len(), "tuple")?,
                StaticValueV1::Sequence { id } => check_index(id.0, self.sequences.len(), "sequence")?,
                StaticValueV1::Record { id } => check_index(id.0, self.records.len(), "record")?,
                StaticValueV1::Data { id } => check_index(id.0, self.data_values.len(), "data")?,
                StaticValueV1::Variant { id } => check_index(id.0, self.variant_values.len(), "variant")?,
                StaticValueV1::Pack { id } => check_index(id.0, self.packs.len(), "pack")?,
            }
        }
        // Traverse structural arenas as graph nodes too. Many slices may share
        // a large backing; following its children once avoids quadratic work.
        let mut offsets = [0usize; 8];
        let extents = [
            self.values.len(),
            self.tuples.len(),
            self.sequences.len(),
            self.records.len(),
            self.data_values.len(),
            self.variant_values.len(),
            self.packs.len(),
            self.pack_backings.len(),
        ];
        let mut total = 0usize;
        for (index, extent) in extents.into_iter().enumerate() {
            offsets[index] = total;
            total = total.checked_add(extent).ok_or_else(|| invalid("static graph extent overflow"))?;
        }
        budget.scratch(
            total
                .checked_mul(std::mem::size_of::<usize>() + 1)
                .ok_or_else(|| invalid("static graph scratch overflow"))?,
        )?;
        budget.scratch(
            self.values
                .len()
                .checked_mul(std::mem::size_of::<ArtifactStaticValueId>())
                .ok_or_else(|| invalid("static order overflow"))?,
        )?;
        budget.scratch(
            total
                .checked_mul(std::mem::size_of::<(usize, StaticChildren<'_>)>())
                .ok_or_else(|| invalid("static stack overflow"))?,
        )?;
        let mut nodes = scratch.vector::<(u8, usize)>(total)?;
        nodes.resize(total, (0u8, 0usize));
        let mut order = scratch.vector::<ArtifactStaticValueId>(self.values.len())?;
        let mut stack = scratch.vector::<(usize, StaticChildren<'_>)>(total)?;
        for root in 0..total {
            if nodes[root].0 != 0 {
                continue;
            }
            nodes[root].0 = 1;
            stack.push((root, self.children(root, &offsets)));
            while let Some((node, children)) = stack.last_mut() {
                if let Some(child) = children.next() {
                    match nodes[child].0 {
                        1 => return Err(malformed("cyclic static-value graph")),
                        2 => {
                            nodes[*node].1 = nodes[*node].1.max(nodes[child].1 + 1);
                        }
                        _ => {
                            if stack.len() >= limits.max_recursion_depth {
                                return Err(invalid("static-value graph depth budget exceeded"));
                            }
                            nodes[child].0 = 1;
                            stack.push((child, self.children(child, &offsets)));
                        }
                    }
                } else {
                    let (node, _) = stack.pop().ok_or_else(|| malformed("missing static traversal frame"))?;
                    nodes[node].0 = 2;
                    nodes[node].1 = nodes[node].1.max(1);
                    if nodes[node].1 > limits.max_recursion_depth {
                        return Err(invalid("static-value graph depth budget exceeded"));
                    }
                    if node < self.values.len() {
                        order.push(ArtifactStaticValueId(node as u32));
                    }
                    if let Some((parent, _)) = stack.last() {
                        nodes[*parent].1 = nodes[*parent].1.max(nodes[node].1 + 1);
                    }
                }
            }
        }
        Ok(order)
    }

    /// Checks every type edge against the shared type graph before materialization.
    pub fn validate_type_references(&self, type_count: usize) -> Result<(), ArtifactError> {
        for value in &self.values {
            if let StaticValueV1::Type { ty } | StaticValueV1::StaticTerm { ty } = value {
                check_index(ty.0, type_count, "static type")?;
            }
        }
        for ty in self
            .sequences
            .iter()
            .map(|row| row.result_type)
            .chain(self.data_values.iter().map(|row| row.result_type))
            .chain(self.variant_values.iter().map(|row| row.result_type))
        {
            check_index(ty.0, type_count, "static result type")?;
        }
        Ok(())
    }

    fn check_children(&self, children: &[ArtifactStaticValueId], budget: &mut GraphBudget<'_>) -> Result<(), ArtifactError> {
        budget.collection::<ArtifactStaticValueId>(children.len(), budget.limits.max_semantic_entries_per_family)?;
        for child in children {
            check_index(child.0, self.values.len(), "static child")?;
        }
        Ok(())
    }

    fn children<'a>(&'a self, node: usize, offsets: &[usize; 8]) -> StaticChildren<'a> {
        if node < offsets[1] {
            let edge = match self.values[node] {
                StaticValueV1::Tuple { id } => Some(offsets[1] + id.0 as usize),
                StaticValueV1::Sequence { id } => Some(offsets[2] + id.0 as usize),
                StaticValueV1::Record { id } => Some(offsets[3] + id.0 as usize),
                StaticValueV1::Data { id } => Some(offsets[4] + id.0 as usize),
                StaticValueV1::Variant { id } => Some(offsets[5] + id.0 as usize),
                StaticValueV1::Pack { id } => Some(offsets[6] + id.0 as usize),
                _ => None,
            };
            StaticChildren::One(edge)
        } else if node < offsets[2] {
            StaticChildren::Values(self.tuples[node - offsets[1]].iter())
        } else if node < offsets[3] {
            StaticChildren::Values(self.sequences[node - offsets[2]].elements.iter())
        } else if node < offsets[4] {
            StaticChildren::Record(self.records[node - offsets[3]].iter())
        } else if node < offsets[5] {
            StaticChildren::Data(self.data_values[node - offsets[4]].components.iter())
        } else if node < offsets[6] {
            StaticChildren::Variant(self.variant_values[node - offsets[5]].fields.iter())
        } else if node < offsets[7] {
            StaticChildren::One(Some(offsets[7] + self.packs[node - offsets[6]].backing.0 as usize))
        } else {
            let backing = &self.pack_backings[node - offsets[7]];
            StaticChildren::Backing(backing.positionals.iter().chain(backing.labeled_values.iter()))
        }
    }
}

enum StaticChildren<'a> {
    One(Option<usize>),
    Values(std::slice::Iter<'a, ArtifactStaticValueId>),
    Record(std::slice::Iter<'a, StaticRecordFieldV1>),
    Data(std::slice::Iter<'a, StaticDataComponentV1>),
    Variant(std::slice::Iter<'a, StaticVariantFieldV1>),
    Backing(std::iter::Chain<std::slice::Iter<'a, ArtifactStaticValueId>, std::slice::Iter<'a, ArtifactStaticValueId>>),
}
impl Iterator for StaticChildren<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        match self {
            Self::One(edge) => edge.take(),
            Self::Values(values) => values.next().map(|id| id.0 as usize),
            Self::Record(fields) => fields.next().map(|field| field.value.0 as usize),
            Self::Data(fields) => fields.next().map(|field| field.value.0 as usize),
            Self::Variant(fields) => fields.next().map(|field| field.value.0 as usize),
            Self::Backing(values) => values.next().map(|id| id.0 as usize),
        }
    }
}

fn malformed(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
fn check_index(raw: u32, extent: usize, family: &str) -> Result<(), ArtifactError> {
    if raw as usize >= extent {
        return Err(malformed(format!("invalid {family} reference {raw}")));
    }
    Ok(())
}
fn check_range(start: u32, len: u32, extent: usize) -> Result<(), ArtifactError> {
    if start.checked_add(len).is_none_or(|end| end as usize > extent) {
        return Err(malformed("static pack slice exceeds backing"));
    }
    Ok(())
}
/// Validates exact canonical decimal spelling before any arbitrary-precision parse.
pub(crate) fn validate_decimal(decimal: &str) -> Result<(), ArtifactError> {
    let digits = decimal.strip_prefix('-').unwrap_or(decimal);
    if digits.is_empty() || !digits.bytes().all(|digit| digit.is_ascii_digit()) || (digits.len() > 1 && digits.starts_with('0')) || decimal == "-0" {
        return Err(malformed("noncanonical static integer decimal"));
    }
    Ok(())
}
fn validate_key(key: &FieldKeyV1, budget: &mut GraphBudget<'_>) -> Result<(), ArtifactError> {
    match key {
        FieldKeyV1::Symbol { value } | FieldKeyV1::String { value } => budget.text(value),
        FieldKeyV1::Int { decimal } => {
            budget.text(decimal)?;
            validate_decimal(decimal)
        }
        FieldKeyV1::Bool { .. } => Ok(()),
    }
}
fn compare_keys(left: &FieldKeyV1, right: &FieldKeyV1) -> std::cmp::Ordering {
    fn rank(key: &FieldKeyV1) -> u8 {
        match key {
            FieldKeyV1::Symbol { .. } => 0,
            FieldKeyV1::String { .. } => 1,
            FieldKeyV1::Int { .. } => 2,
            FieldKeyV1::Bool { .. } => 3,
        }
    }
    rank(left).cmp(&rank(right)).then_with(|| match (left, right) {
        (FieldKeyV1::Symbol { value: left }, FieldKeyV1::Symbol { value: right })
        | (FieldKeyV1::String { value: left }, FieldKeyV1::String { value: right }) => left.cmp(right),
        (FieldKeyV1::Bool { value: left }, FieldKeyV1::Bool { value: right }) => left.cmp(right),
        (FieldKeyV1::Int { decimal: left }, FieldKeyV1::Int { decimal: right }) => {
            let left_negative = left.starts_with('-');
            let right_negative = right.starts_with('-');
            right_negative.cmp(&left_negative).then_with(|| {
                let magnitude = left.len().cmp(&right.len()).then_with(|| left.cmp(right));
                if left_negative { magnitude.reverse() } else { magnitude }
            })
        }
        _ => std::cmp::Ordering::Equal,
    })
}

struct GraphBudget<'a> {
    limits: &'a ValidationLimits,
    bytes: u64,
    items: usize,
}
impl<'a> GraphBudget<'a> {
    fn new(limits: &'a ValidationLimits) -> Self {
        Self {
            limits,
            bytes: limits.max_decoded_section_bytes,
            items: limits.max_semantic_entries_per_family,
        }
    }
    fn scratch(&mut self, bytes: usize) -> Result<(), ArtifactError> {
        self.bytes = self
            .bytes
            .checked_sub(bytes as u64)
            .ok_or_else(|| invalid("static graph allocation budget exceeded"))?;
        Ok(())
    }
    fn collection<T>(&mut self, count: usize, limit: usize) -> Result<(), ArtifactError> {
        if count > limit || u32::try_from(count).is_err() {
            return Err(invalid("static graph collection budget exceeded"));
        }
        self.items = self.items.checked_sub(count).ok_or_else(|| invalid("static graph item budget exceeded"))?;
        self.scratch(
            count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or_else(|| invalid("static graph allocation overflow"))?,
        )
    }
    fn text(&mut self, text: &str) -> Result<(), ArtifactError> {
        if text.len() > self.limits.max_string_bytes {
            return Err(invalid("static graph string budget exceeded"));
        }
        self.scratch(text.len())
    }
    fn owner(&mut self, owner: &StableDeclarationRef) -> Result<(), ArtifactError> {
        use phalcom_type_meta::identity::StableProjectRef;
        match &owner.module.project {
            StableProjectRef::Builtin { namespace, version } if namespace.as_ref() == "universe" => {
                self.text(namespace)?;
                self.text(version)?;
            }
            _ => return Err(malformed("static owner is not a canonical Universe identity")),
        }
        self.collection::<Box<str>>(owner.module.path.len(), self.limits.max_semantic_entries_per_family)?;
        self.collection::<Box<str>>(owner.path.len(), self.limits.max_semantic_entries_per_family)?;
        if owner.path.is_empty() {
            return Err(malformed("empty static declaration owner"));
        }
        for segment in owner.module.path.iter().chain(owner.path.iter()) {
            self.text(segment)?;
            if segment.is_empty() {
                return Err(malformed("empty static owner path segment"));
            }
        }
        Ok(())
    }
}

enum ValidationScratch<'a> {
    Untracked,
    Budgeted(&'a mut AllocationBudget),
}

impl ValidationScratch<'_> {
    fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, ArtifactError> {
        match self {
            Self::Untracked => {
                let mut values = Vec::new();
                values.try_reserve_exact(count).map_err(|_| invalid("static graph scratch allocation failed"))?;
                Ok(values)
            }
            Self::Budgeted(allocations) => allocations.vector(count).map_err(ArtifactError::Allocation),
        }
    }
}

impl From<&FieldKeyRef> for FieldKeyV1 {
    fn from(k: &FieldKeyRef) -> Self {
        match k {
            FieldKeyRef::Symbol(s) => FieldKeyV1::Symbol { value: s.to_string() },
            FieldKeyRef::String(s) => FieldKeyV1::String { value: s.to_string() },
            FieldKeyRef::Int(i) => FieldKeyV1::Int { decimal: i.to_string() },
            FieldKeyRef::Bool(b) => FieldKeyV1::Bool { value: *b },
        }
    }
}

impl TryFrom<&FieldKeyV1> for FieldKeyRef {
    type Error = ArtifactError;
    fn try_from(k: &FieldKeyV1) -> Result<Self, Self::Error> {
        Ok(match k {
            FieldKeyV1::Symbol { value } => FieldKeyRef::Symbol(value.as_str().into()),
            FieldKeyV1::String { value } => FieldKeyRef::String(value.as_str().into()),
            FieldKeyV1::Int { decimal } => {
                let si = decimal.parse::<StaticInt>().map_err(|_| invalid("invalid static integer"))?;
                FieldKeyRef::Int(si)
            }
            FieldKeyV1::Bool { value } => FieldKeyRef::Bool(*value),
        })
    }
}

impl StaticValueGraphV1 {
    /// Projects a slice of canonical metadata static value entries into an artifact graph.
    pub fn from_metadata_nodes(entries: &[StaticValueNodeEntry]) -> Result<Self, ArtifactError> {
        let mut values = Vec::with_capacity(entries.len());
        let mut strings = Vec::new();
        let mut symbols = Vec::new();
        let mut tuples = Vec::new();
        let mut sequences = Vec::new();
        let mut records = Vec::new();
        let mut data_values = Vec::new();
        let mut variant_values = Vec::new();
        let mut pack_backings = Vec::new();
        let mut packs = Vec::new();

        for entry in entries {
            let v = match &entry.node {
                StaticValueNode::Unit => StaticValueV1::Unit,
                StaticValueNode::Bool(b) => StaticValueV1::Bool { value: *b },
                StaticValueNode::Char(c) => StaticValueV1::Char { scalar: *c as u32 },
                StaticValueNode::Int(i) => StaticValueV1::Int { decimal: i.to_string() },
                StaticValueNode::FloatBits(bits) => StaticValueV1::Float { bits: *bits },
                StaticValueNode::String(s) => {
                    let id = ArtifactStaticStringId(strings.len() as u32);
                    strings.push(s.to_string());
                    StaticValueV1::String { id }
                }
                StaticValueNode::Symbol(s) => {
                    let id = ArtifactStaticSymbolId(symbols.len() as u32);
                    symbols.push(s.to_string());
                    StaticValueV1::Symbol { id }
                }
                StaticValueNode::Type(ty) => StaticValueV1::Type { ty: ArtifactTypeId(ty.0) },
                StaticValueNode::StaticTerm(ty) => StaticValueV1::StaticTerm { ty: ArtifactTypeId(ty.0) },
                StaticValueNode::Tuple(elems) => {
                    let id = ArtifactStaticTupleId(tuples.len() as u32);
                    tuples.push(elems.iter().map(|child| ArtifactStaticValueId(child.0)).collect());
                    StaticValueV1::Tuple { id }
                }
                StaticValueNode::Sequence { family, result_type, elements } => {
                    let id = ArtifactStaticSequenceId(sequences.len() as u32);
                    sequences.push(StaticSequenceV1 {
                        family: match family {
                            StaticSequenceFamilyRef::Array => SequenceFamilyV1::Array,
                            StaticSequenceFamilyRef::Vector => SequenceFamilyV1::Vector,
                        },
                        result_type: ArtifactTypeId(result_type.0),
                        elements: elements.iter().map(|child| ArtifactStaticValueId(child.0)).collect(),
                    });
                    StaticValueV1::Sequence { id }
                }
                StaticValueNode::Record(fields) => {
                    let id = ArtifactStaticRecordId(records.len() as u32);
                    let mut rec_fields = Vec::with_capacity(fields.len());
                    for (k, val) in fields.iter() {
                        rec_fields.push(StaticRecordFieldV1 {
                            key: FieldKeyV1::from(k),
                            value: ArtifactStaticValueId(val.0),
                        });
                    }
                    records.push(rec_fields);
                    StaticValueV1::Record { id }
                }
                StaticValueNode::Data {
                    owner,
                    result_type,
                    components,
                } => {
                    let id = ArtifactStaticDataId(data_values.len() as u32);
                    let comp_vec = components
                        .iter()
                        .map(|(cid, val)| StaticDataComponentV1 {
                            index: cid.index,
                            value: ArtifactStaticValueId(val.0),
                        })
                        .collect();
                    data_values.push(StaticDataValueV1 {
                        owner: owner.clone(),
                        constructor_owner: owner.clone(),
                        result_type: ArtifactTypeId(result_type.0),
                        components: comp_vec,
                    });
                    StaticValueV1::Data { id }
                }
                StaticValueNode::Variant { variant, result_type, fields } => {
                    let id = ArtifactStaticVariantId(variant_values.len() as u32);
                    let field_vec = fields
                        .iter()
                        .map(|(fid, val)| StaticVariantFieldV1 {
                            index: fid.index,
                            value: ArtifactStaticValueId(val.0),
                        })
                        .collect();
                    variant_values.push(StaticVariantValueV1 {
                        variant: variant.clone(),
                        result_type: ArtifactTypeId(result_type.0),
                        fields: field_vec,
                    });
                    StaticValueV1::Variant { id }
                }
                StaticValueNode::Pack { positionals, labeled } => {
                    let backing_id = ArtifactStaticPackBackingId(pack_backings.len() as u32);
                    let pos_vec: Vec<ArtifactStaticValueId> = positionals.iter().map(|p| ArtifactStaticValueId(p.0)).collect();
                    let mut label_ids = Vec::with_capacity(labeled.len());
                    let mut labeled_vals = Vec::with_capacity(labeled.len());
                    for (lbl, val) in labeled.iter() {
                        let sym_id = ArtifactStaticSymbolId(symbols.len() as u32);
                        symbols.push(lbl.to_string());
                        label_ids.push(sym_id);
                        labeled_vals.push(ArtifactStaticValueId(val.0));
                    }
                    let pos_len = pos_vec.len() as u32;
                    let labeled_len = labeled_vals.len() as u32;
                    pack_backings.push(StaticPackBackingV1 {
                        positionals: pos_vec,
                        labels: label_ids,
                        labeled_values: labeled_vals,
                    });
                    let pack_id = ArtifactStaticPackId(packs.len() as u32);
                    packs.push(StaticPackSliceV1 {
                        backing: backing_id,
                        positional_start: 0,
                        positional_len: pos_len,
                        labeled_start: 0,
                        labeled_len,
                    });
                    StaticValueV1::Pack { id: pack_id }
                }
            };
            values.push(v);
        }

        let graph = Self {
            values,
            strings,
            symbols,
            tuples,
            sequences,
            records,
            data_values,
            variant_values,
            pack_backings,
            packs,
        };
        Ok(graph)
    }

    /// Converts an artifact static value graph back into canonical metadata entries.
    pub fn to_metadata_nodes(&self) -> Result<Vec<StaticValueNodeEntry>, ArtifactError> {
        let mut entries = Vec::with_capacity(self.values.len());
        for val in &self.values {
            let node = match val {
                StaticValueV1::Unit => StaticValueNode::Unit,
                StaticValueV1::Bool { value } => StaticValueNode::Bool(*value),
                StaticValueV1::Char { scalar } => {
                    let c = char::from_u32(*scalar).ok_or_else(|| invalid("invalid char scalar"))?;
                    StaticValueNode::Char(c)
                }
                StaticValueV1::Int { decimal } => {
                    let i = decimal.parse::<StaticInt>().map_err(|_| invalid("invalid integer"))?;
                    StaticValueNode::Int(i)
                }
                StaticValueV1::Float { bits } => StaticValueNode::FloatBits(*bits),
                StaticValueV1::String { id } => {
                    let s = self.strings.get(id.0 as usize).ok_or_else(|| invalid("invalid string id"))?;
                    StaticValueNode::String(s.as_str().into())
                }
                StaticValueV1::Symbol { id } => {
                    let s = self.symbols.get(id.0 as usize).ok_or_else(|| invalid("invalid symbol id"))?;
                    StaticValueNode::Symbol(s.as_str().into())
                }
                StaticValueV1::Type { ty } => StaticValueNode::Type(phalcom_type_meta::type_node::TypeNodeId(ty.0)),
                StaticValueV1::StaticTerm { ty } => StaticValueNode::StaticTerm(phalcom_type_meta::type_node::TypeNodeId(ty.0)),
                StaticValueV1::Tuple { id } => {
                    let t = self.tuples.get(id.0 as usize).ok_or_else(|| invalid("invalid tuple id"))?;
                    let nodes = t.iter().map(|child| StaticValueNodeId(child.0)).collect::<Vec<_>>().into_boxed_slice();
                    StaticValueNode::Tuple(nodes)
                }
                StaticValueV1::Sequence { id } => {
                    let s = self.sequences.get(id.0 as usize).ok_or_else(|| invalid("invalid sequence id"))?;
                    let elements = s.elements.iter().map(|child| StaticValueNodeId(child.0)).collect::<Vec<_>>().into_boxed_slice();
                    StaticValueNode::Sequence {
                        family: match s.family {
                            SequenceFamilyV1::Array => StaticSequenceFamilyRef::Array,
                            SequenceFamilyV1::Vector => StaticSequenceFamilyRef::Vector,
                        },
                        result_type: phalcom_type_meta::type_node::TypeNodeId(s.result_type.0),
                        elements,
                    }
                }
                StaticValueV1::Record { id } => {
                    let r = self.records.get(id.0 as usize).ok_or_else(|| invalid("invalid record id"))?;
                    let mut fields = Vec::with_capacity(r.len());
                    for f in r {
                        let fk = FieldKeyRef::try_from(&f.key)?;
                        fields.push((fk, StaticValueNodeId(f.value.0)));
                    }
                    StaticValueNode::Record(fields.into_boxed_slice())
                }
                StaticValueV1::Data { id } => {
                    let d = self.data_values.get(id.0 as usize).ok_or_else(|| invalid("invalid data id"))?;
                    let components = d
                        .components
                        .iter()
                        .map(|c| {
                            (
                                phalcom_type_meta::identity::StableDataComponentRef {
                                    owner: d.owner.clone(),
                                    index: c.index,
                                },
                                StaticValueNodeId(c.value.0),
                            )
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    StaticValueNode::Data {
                        owner: d.owner.clone(),
                        result_type: phalcom_type_meta::type_node::TypeNodeId(d.result_type.0),
                        components,
                    }
                }
                StaticValueV1::Variant { id } => {
                    let v = self.variant_values.get(id.0 as usize).ok_or_else(|| invalid("invalid variant id"))?;
                    let fields = v
                        .fields
                        .iter()
                        .map(|f| {
                            (
                                phalcom_type_meta::identity::StableVariantFieldRef {
                                    variant: v.variant.clone(),
                                    index: f.index,
                                },
                                StaticValueNodeId(f.value.0),
                            )
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    StaticValueNode::Variant {
                        variant: v.variant.clone(),
                        result_type: phalcom_type_meta::type_node::TypeNodeId(v.result_type.0),
                        fields,
                    }
                }
                StaticValueV1::Pack { id } => {
                    let p = self.packs.get(id.0 as usize).ok_or_else(|| invalid("invalid pack id"))?;
                    let b = self.pack_backings.get(p.backing.0 as usize).ok_or_else(|| invalid("invalid backing id"))?;
                    let p_start = p.positional_start as usize;
                    let p_end = p_start + p.positional_len as usize;
                    let positionals = b
                        .positionals
                        .get(p_start..p_end)
                        .ok_or_else(|| invalid("pack positional slice out of bounds"))?
                        .iter()
                        .map(|val| StaticValueNodeId(val.0))
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let l_start = p.labeled_start as usize;
                    let l_end = l_start + p.labeled_len as usize;
                    let labels = b.labels.get(l_start..l_end).ok_or_else(|| invalid("pack label slice out of bounds"))?;
                    let l_vals = b
                        .labeled_values
                        .get(l_start..l_end)
                        .ok_or_else(|| invalid("pack labeled value slice out of bounds"))?;
                    let mut labeled = Vec::with_capacity(labels.len());
                    for (sym_id, val) in labels.iter().zip(l_vals.iter()) {
                        let sym = self.symbols.get(sym_id.0 as usize).ok_or_else(|| invalid("invalid symbol id"))?;
                        labeled.push((sym.as_str().into(), StaticValueNodeId(val.0)));
                    }
                    StaticValueNode::Pack {
                        positionals,
                        labeled: labeled.into_boxed_slice(),
                    }
                }
            };
            entries.push(StaticValueNodeEntry {
                node,
                structural_fingerprint: phalcom_type_meta::fingerprint::Fingerprint128::ZERO,
            });
        }

        for i in 0..entries.len() {
            let fp = compute_static_value_fingerprint(
                &entries[i].node,
                |child_id| entries[child_id.0 as usize].structural_fingerprint,
                |type_id| phalcom_type_meta::fingerprint::Fingerprint128::from_u128(type_id.0 as u128),
            );
            entries[i].structural_fingerprint = fp;
        }

        Ok(entries)
    }
}
