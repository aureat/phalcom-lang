# Deferred: multiline string literals

PDR-0029 keeps ordinary double-quoted strings single-line. A physical LF or
CRLF is invalid inside the literal rather than string content; use `\n` or
`\r\n` escapes today.

Do not introduce triple quotes, heredocs, indentation trimming, or a raw
multiline form as an implementation convenience. A future design must choose
the delimiter, closing-delimiter indentation rule, source-value newline
normalization, escape behavior, interpolation behavior, diagnostics, and how
the form interacts with comments and statement newlines. That is a separate
literal-design unit, not residue from string interpolation completion.
