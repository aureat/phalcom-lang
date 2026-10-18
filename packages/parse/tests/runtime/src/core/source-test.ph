// test: core-source
// TextSource and ListSource adapters, cursor utilities, select.
import parse_tests.support.test as Testing
from parse.core.cursor import Cursor
from parse.core.result import RunResult
from parse.run import Parse
from parse.source.list_source import ListSource
from parse.source.text_source import TextSource
from parse.text import Text

let source = TextSource.new("hi")

Testing.Test.expect(source.size == 2, "TextSource counts bytes")
Testing.Test.expect(source.at(0) == 104, "TextSource.at reads bytes")
Testing.Test.expect(source.at(99) == -1, "TextSource.at is -1 out of bounds")

let position = source.lineColAt(1)

Testing.Test.expect(position.first == 1, "lineColAt line")
Testing.Test.expect(position.second == 2, "lineColAt column")

let multiline = TextSource.new("ab\ncd")

let secondLine = multiline.lineColAt(4)

Testing.Test.expect(secondLine.first == 2, "lineColAt detects the line break")
Testing.Test.expect(secondLine.second == 2, "lineColAt column after line break")

// Cursor utilities.
let cursor = Cursor.new(source, 1)

Testing.Test.expect(cursor.offset == 1, "cursor offset")
Testing.Test.expect(cursor.advance.offset == 2, "cursor advance")
Testing.Test.expect(cursor.advanceBy(0).offset == 1, "cursor advanceBy zero")
Testing.Test.expect(cursor.atEnd == false, "cursor not at end")
Testing.Test.expect(cursor.advance.atEnd == true, "cursor at end after advance")
Testing.Test.expect(cursor.remaining == 1, "cursor remaining")
Testing.Test.expect(cursor.current == Some(105), "cursor current reads the byte")

// ListSource over a token stream.
data Token {
  Integer(_ value: Int)

  Word(_ text: String)

  Comma
}

data SelectedToken {
  Integer(_ value: Int)

  Word(_ text: String)
}

let tokens = List<Token>.new()
tokens.append(Token::Integer(3))
tokens.append(Token::Comma)
tokens.append(Token::Word("four"))

let tokenSource = ListSource<Token>.new(tokens)

// The caller's list may mutate afterwards; the source is a snapshot.
tokens.append(Token::Integer(99))
tokens.at(0, put: Token::Integer(7))
tokens.remove(at: 3)
Testing.Test.expect(tokenSource.size == 3, "ListSource snapshots its backing list")

let tokenCursor = Cursor.new(tokenSource, 0)
let expectedCurrent: Option<Token> = Option<Token>::Some(Token::Integer(3))
let expectedNext: Option<Token> = Option<Token>::Some(Token::Comma)
match tokenCursor.current {
  Option::Some(Token::Integer(value)) => {
    Testing.Test.expect(value == 3, "cursor current reads snapshot token")
  }
  _ => { throw Error.new("cursor current should be the snapshot integer") }
}
match tokenCursor.advance.current {
  Option::Some(Token::Comma) => {
    ()
  }
  _ => { throw Error.new("cursor advance should read the next snapshot token") }
}
match expectedCurrent {
  Option::Some(Token::Integer(value)) => {
    Testing.Test.expect(value == 3, "qualified expected cursor token is an integer")
  }
  _ => { throw Error.new("qualified expected cursor token should be present") }
}
match expectedNext {
  Option::Some(Token::Comma) => {
    ()
  }
  _ => { throw Error.new("qualified expected next token should be a comma") }
}

let integerToken = ListSource<Token>.select("integer") |token| {
  let picked: Option<Int> = match token {
    Token::Integer(value) => { Some(value) }
    _ => { None }
  }
  picked
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(integerToken, tokenSource)),
  "ok:3"
)

let commaToken = ListSource<Token>.satisfy("comma") |token| {
  match token {
    Token::Comma => { true }
    _ => { false }
  }
}

let wordToken = ListSource<Token>.select("word") |token| {
  let picked: Option<String> = match token {
    Token::Word(text) => { Some(text) }
    _ => { None }
  }
  picked
}

Testing.Test.expect(
  Testing.Test.show(Parse.run(commaToken, tokenSource)).contains("err:"),
  "comma does not match the first token"
)

let integerChoice = ListSource<Token>.select("integer or word") |token| {
  let picked: Option<SelectedToken> = match token {
    Token::Integer(value) => { Some(SelectedToken::Integer(value)) }
    _ => { None }
  }
  picked
}

let wordChoice = ListSource<Token>.select("integer or word") |token| {
  let picked: Option<SelectedToken> = match token {
    Token::Word(text) => { Some(SelectedToken::Word(text)) }
    _ => { None }
  }
  picked
}

let wordOrInteger = integerChoice.orElse(wordChoice)

match Parse.run(wordOrInteger, tokenSource) {
  RunResult::Ok(SelectedToken::Integer(value)) => {
    Testing.Test.expect(value == 3, "integer choice preserves typed payload")
  }
  _ => { throw Error.new("wordOrInteger should select an integer payload") }
}

let wordTokens = List<Token>.new()
wordTokens.append(Token::Word("four"))
let wordSource = ListSource<Token>.new(wordTokens)
match Parse.run(wordOrInteger, wordSource) {
  RunResult::Ok(SelectedToken::Word(text)) => {
    Testing.Test.expect(text == "four", "word choice preserves typed payload")
  }
  _ => { throw Error.new("wordOrInteger should select a word payload") }
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(integerToken.then(commaToken).then(wordToken), tokenSource)),
  "ok:four"
)

Testing.Test.pass("core-source")
