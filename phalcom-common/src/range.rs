use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CopyRange<T> {
    pub start: T,
    pub end: T,
}

impl From<Range<usize>> for CopyRange<usize> {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

pub type SourceRange = CopyRange<usize>;

pub const EmptySourceRange: SourceRange = SourceRange { start: 0, end: 0 };