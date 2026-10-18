// test: expr-operators
// Lexed token source, typed selection, operator chains, defer recursion.
//
// Renders the parsed expression tree as a string during parsing so this fixture
// can focus on tokenization, recursive combinator composition, precedence, and
// backtracking without adding an evaluator to the grammar.
import parse_tests.support.test as Testing
from parse.core.rule import Rule
from parse.run import Parse
from parse.source.list_source import ListSource

data Token {
  Integer(_ value: Int)

  Name(_ text: String)

  Plus

  Star

  Caret

  LeftParen

  RightParen
}

let integerOperand = ListSource<Token>.select("integer") |token| {
  let picked: Option<String> = match token {
    Token::Integer(value) => { Some(value.toString) }
    _ => { None }
  }
  picked
}

let nameOperand = ListSource<Token>.select("name") |token| {
  let picked: Option<String> = match token {
    Token::Name(text) => { Some(text) }
    _ => { None }
  }
  picked
}

let plusOperator = ListSource<Token>.select("\"+\"") |token| {
  let picked: Option<(String, String) -> String> = match token {
    Token::Plus => { Some(|left: String, right: String| { "(" + left + "+" + right + ")" }) }
    _ => { None }
  }
  picked
}

let starOperator = ListSource<Token>.select("\"*\"") |token| {
  let picked: Option<(String, String) -> String> = match token {
    Token::Star => { Some(|left: String, right: String| { "(" + left + "*" + right + ")" }) }
    _ => { None }
  }
  picked
}

let caretOperator = ListSource<Token>.select("\"^\"") |token| {
  let picked: Option<(String, String) -> String> = match token {
    Token::Caret => { Some(|left: String, right: String| { "(" + left + "^" + right + ")" }) }
    _ => { None }
  }
  picked
}

let leftParen = ListSource<Token>.satisfy("\"(\"") |token| {
  match token {
    Token::LeftParen => { true }
    _ => { false }
  }
}

let rightParen = ListSource<Token>.satisfy("\")\"") |token| {
  match token {
    Token::RightParen => { true }
    _ => { false }
  }
}

// Recursive grammar with precedence:
//   expression := term ('+' term)*
//   term       := factor ('*' factor)*
//   factor     := base ('^' factor)?   (right-associative)
//   base       := integer | name | '(' expression ')'
let expression: Rule<ListSource<Token>, String> = Rule.defer

let parenthesized = expression.between(leftParen, rightParen)
let base = integerOperand.orElse(nameOperand).orElse(parenthesized)
let factor = base.chainRight1(caretOperator)
let term = factor.chainLeft1(starOperator)
let whole = term.chainLeft1(plusOperator)

expression.define(whole)

let tokenize = |text: String| {
  let mut index = 0
  let tokens = List<Token>.new()
  while index < text.size {
    let byte = text.bytes.at(index).unwrapOr(-1)
    if byte >= 48 and byte <= 57 {
      let mut value = 0
      while index < text.size {
        let b = text.bytes.at(index).unwrapOr(-1)
        if b >= 48 and b <= 57 {
          value = value * 10 + (b - 48)
          index = index + 1
        } else {
          break
        }
      }
      tokens.append(Token::Integer(value))
    } else if byte >= 97 and byte <= 122 {
      let mut name = ""
      while index < text.size {
        let b = text.bytes.at(index).unwrapOr(-1)
        if b >= 97 and b <= 122 {
          name = name + text.slice(index, index + 1).toString
          index = index + 1
        } else {
          break
        }
      }
      tokens.append(Token::Name(name))
    } else if byte == 43 {
      tokens.append(Token::Plus)
      index = index + 1
    } else if byte == 42 {
      tokens.append(Token::Star)
      index = index + 1
    } else if byte == 94 {
      tokens.append(Token::Caret)
      index = index + 1
    } else if byte == 40 {
      tokens.append(Token::LeftParen)
      index = index + 1
    } else if byte == 41 {
      tokens.append(Token::RightParen)
      index = index + 1
    } else {
      throw Error.new("tokenizer: unexpected byte \(byte)")
    }
  }
  tokens
}

let evaluate = |text: String| {
  Testing.Test.show(Parse.runAll(expression, ListSource<Token>.new(tokenize.call(text))))
}

Testing.Test.expectEqual(evaluate.call("1"), "ok:1")
Testing.Test.expectEqual(evaluate.call("abc"), "ok:abc")
Testing.Test.expectEqual(evaluate.call("1+2"), "ok:(1+2)")
Testing.Test.expectEqual(evaluate.call("1+2*3"), "ok:(1+(2*3))")
Testing.Test.expectEqual(evaluate.call("(1+2)*3"), "ok:((1+2)*3)")
Testing.Test.expectEqual(evaluate.call("2^3^2"), "ok:(2^(3^2))")
Testing.Test.expectEqual(evaluate.call("1+2+3"), "ok:((1+2)+3)")
Testing.Test.expectEqual(evaluate.call("n*(m+1)"), "ok:(n*(m+1))")
Testing.Test.expect(evaluate.call("1+").contains("err:"), "trailing operator fails")
Testing.Test.expect(evaluate.call("(1+2").contains("err:"), "unclosed paren fails")
Testing.Test.expect(evaluate.call("1+*3").contains("err:"), "dangling operator fails")

Testing.Test.pass("expr-operators")
