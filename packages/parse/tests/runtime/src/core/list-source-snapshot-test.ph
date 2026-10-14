// test: core-list-source-snapshot
// P5-owned ListSource snapshot, parser, and Slice-owner coverage.
import parse_tests.support.test as Testing
from parse.core.cursor import Cursor
from parse.run import Parse
from parse.source.list_source import ListSource

data Token {
  Integer(_ value: Int)

  Text(_ value: String)
}

// Only the returned source escapes; the caller's List is no longer reachable.
// The String payload makes the Array -> Slice -> element tracing path observable
// after a collection cycle as well as through parser execution.
class SourceFactory {
  @class
  sourceWithHeapToken -> ListSource<Token> {
    let caller = List<Token>.new()
    caller.append(Token::Text("retained through ListSource Slice owner"))
    ListSource<Token>.new(caller)
  }

  // Invoked by the native integration test after collecting the caller List.
  @class
  verifyStrings(_ source: ListSource<String>) -> Bool {
    let cursor = Cursor.new(source, 0)
    Testing.Test.expect(cursor.current == Some("first heap token"), "post-GC cursor reads the first token")
    Testing.Test.expect(cursor.advance.current == Some("second heap token"), "post-GC cursor reads the second token")
    let first = ListSource<String>.satisfy("first") |value| { value == "first heap token" }
    let second = ListSource<String>.satisfy("second") |value| { value == "second heap token" }
    Testing.Test.expectEqual(Testing.Test.show(Parse.runAll(first.then(second), source)), "ok:second heap token")
    true
  }
}

// Empty and singleton sources retain their boundary behavior.
let empty = ListSource<Int>.new(List<Int>.new())
let emptyCursor = Cursor.new(empty, 0)
Testing.Test.expect(empty.size == 0, "empty ListSource has zero size")
Testing.Test.expect(emptyCursor.atEnd, "empty ListSource cursor is at end")
Testing.Test.expect(emptyCursor.remaining == 0, "empty ListSource has no remaining tokens")
Testing.Test.expect(emptyCursor.current == None, "empty cursor has no current token")

let singletonItems = List<Int>.new()
singletonItems.append(7)
let singleton = ListSource<Int>.new(singletonItems)
let singletonCursor = Cursor.new(singleton, 0)
Testing.Test.expect(singleton.size == 1, "singleton ListSource has one token")
Testing.Test.expect(singletonCursor.current == Some(7), "singleton cursor reads its token")
Testing.Test.expect(singletonCursor.advance.atEnd, "singleton cursor reaches end")
Testing.Test.expect(singletonCursor.advance.offset == 1, "singleton final offset is one")
Testing.Test.expect(singletonCursor.advance.current == None, "singleton final cursor has no token")

// Each caller mutation category has an independent observable assertion.
let appendItems = List<Int>.new()
appendItems.append(10)
appendItems.append(20)
let appendSource = ListSource<Int>.new(appendItems)
appendItems.append(30)
Testing.Test.expect(appendSource.size == 2, "appending caller List does not extend snapshot")
Testing.Test.expect(appendSource.at(1) == 20, "appending caller List preserves snapshot order")
Testing.Test.expect(appendSource.at(0) == 10, "appending preserves the first element")

let replaceItems = List<Int>.new()
replaceItems.append(1)
replaceItems.append(2)
replaceItems.append(3)
let replaceSource = ListSource<Int>.new(replaceItems)
replaceItems.at(0, put: 99)
Testing.Test.expect(replaceSource.at(0) == 1, "replacing caller List does not alter snapshot")
Testing.Test.expect(replaceSource.at(2) == 3, "replacement preserves the remaining snapshot")
Testing.Test.expect(replaceSource.size == 3, "replacement preserves size")
Testing.Test.expect(replaceSource.at(1) == 2, "replacement preserves the middle element")

let removeItems = List<Int>.new()
removeItems.append(4)
removeItems.append(5)
removeItems.append(6)
let removeSource = ListSource<Int>.new(removeItems)
removeItems.remove(at: 1)
Testing.Test.expect(removeSource.size == 3, "removing caller List element does not shrink snapshot")
Testing.Test.expect(removeSource.at(1) == 5, "removing caller middle element preserves snapshot order")
Testing.Test.expect(removeSource.at(0) == 4, "removal preserves the preceding element")
Testing.Test.expect(removeSource.at(2) == 6, "removal preserves the following element")

let tokenCursor = Cursor.new(appendSource, 0)
Testing.Test.expect(tokenCursor.current == Some(10), "Cursor.current reads ListSource")
Testing.Test.expect(tokenCursor.advance.current == Some(20), "Cursor.advance reads the next snapshot token")
Testing.Test.expect(tokenCursor.advance.offset == 1, "Cursor.advance increments the offset")

let ten = ListSource<Int>.satisfy("ten") |value| { value == 10 }
let twenty = ListSource<Int>.select("twenty") |value| {
  if value == 20 { Some(value) } else { None }
}
let thirty = ListSource<Int>.select("thirty") |value| {
  if value == 30 { Some(value) } else { None }
}
let notTen = ListSource<Int>.satisfy("not ten") |value| { value == 99 }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(ten, appendSource)),
  "ok:10"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(notTen, appendSource)).contains("err:"),
  "satisfy rejects a predicate mismatch"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(twenty, appendSource)).contains("err:"),
  "select rejects None"
)

let completeItems = List<Int>.new()
completeItems.append(10)
completeItems.append(20)
completeItems.append(30)
let completeSource = ListSource<Int>.new(completeItems)
let completeRule = ten.then(twenty).then(thirty)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(completeRule, completeSource)),
  "ok:30"
)

let incompleteItems = List<Int>.new()
incompleteItems.append(10)
incompleteItems.append(20)
let incompleteSource = ListSource<Int>.new(incompleteItems)
Testing.Test.expect(
  Testing.Test.show(Parse.runAll(completeRule, incompleteSource)).contains("err:"),
  "runAll rejects incomplete input"
)

let trailingItems = List<Int>.new()
trailingItems.append(10)
trailingItems.append(20)
trailingItems.append(30)
trailingItems.append(40)
let trailingSource = ListSource<Int>.new(trailingItems)
Testing.Test.expect(
  Testing.Test.show(Parse.runAll(completeRule, trailingSource)).contains("end of input"),
  "runAll rejects trailing input"
)

let retained = SourceFactory.sourceWithHeapToken
System.gc
let retainedToken = retained.at(0)
match retainedToken {
  Token::Text(value) => {
    Testing.Test.expectEqual(value, "retained through ListSource Slice owner")
  }
  _ => {
    Testing.Test.expect(false, "ListSource retains its heap-backed token")
  }
}
let retainedRule = ListSource<Token>.satisfy("text") |token| {
  match token {
    Token::Text(_) => { true }
    _ => { false }
  }
}
Testing.Test.expect(
  Testing.Test.show(Parse.runAll(retainedRule, retained)).contains("ok:"),
  "parser consumes a source after caller lifetime and GC"
)

Testing.Test.pass("core-list-source-snapshot")
