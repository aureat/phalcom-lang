from prolog.predicate import Predicate
from prolog.variable import Variable
from prolog.query import Query
from prolog.rule import Rule
from prolog.relationship import Relationship
from prolog.base import KnowledgeBase

data Person {
  Alice
  Bob
  Charlie
  Diana
}

let parent = Predicate<*(Person, Person)>.named(#parent)
let grandparent = Predicate<*(Person, Person)>.named(#grandparent)

let ancestor = Variable<Person>.named(#ancestor)
let middle = Variable<Person>.named(#middle)
let descendant = Variable<Person>.named(#descendant)

let query = Query.of(
  grandparent(Person::Alice, descendant)
)

let query2 = Query.of(
  parent(Person::Alice, middle),
  parent(middle, descendant)
)

let gpRule = Rule {
  head: grandparent(ancestor, descendant),
  body: (
    parent(ancestor, middle),
    parent(middle, descendant),
  ),
}

let directParent = Rule {
  head: parent(Person::Alice, Person::Bob),
  body: (
    parent(Person::Alice, Person::Bob),
  ),
}

let kb = KnowledgeBase.of(
  gpRule,
  directParent
)

System.print("query goals: \(query.size)")
System.print("query2 goals: \(query2.size)")
System.print("rule body: \(gpRule.size)")
System.print("kb clauses: \(kb.size)")
