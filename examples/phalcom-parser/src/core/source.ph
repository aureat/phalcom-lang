from core.domains import Natural, SourceOffset

data Span(
    start: SourceOffset,
    end: SourceOffset
)
where start <= end


impl Span {
    @class
    empty(at offset: SourceOffset) -> Span {
        Span(offset, offset)
    }

    length -> Natural {
        end - start
    }

    join(_ other: Span) -> Span {
        let joinedStart =
            if start <= other.start { start } else { other.start }

        let joinedEnd =
            if end >= other.end { end } else { other.end }

        Span(joinedStart, joinedEnd)
    }
}


data SourceText private(
    _ text: String
)

impl SourceText {
    @class
    new(_ text: String) -> SourceText {
        SourceText(text)
    }

    length -> Natural {
        _text.length
    }

    charAt(_ offset: SourceOffset) -> Option<Char> {
        if offset < length {
            Some(_text[offset])
        } else {
            None
        }
    }

    slice(_ span: Span) -> String
    where span.end <= self.length
    {
        _text.slice(from: span.start, to: span.end)
    }
}
