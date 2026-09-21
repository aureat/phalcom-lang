from core.source import SourceText
from lex.lexer import Lexer
from parser.parser import Parser

class PhalcomSyntax {
    @class
    parse(_ text: String) -> ParseResult {
        let source = SourceText.new(text)
        let (tokens, lexicalDiagnostics) =
            Lexer.lex(source)

        let result = Parser.parse(tokens)

        // Initial package draft leaves lexical and parser diagnostic
        // concatenation to the final diagnostics API.
        result
    }
}
