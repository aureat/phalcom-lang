data Person {
  Alice,
  Bob,
  Charlie,
  Diana,
}

//
// Parental relationship args
//

type ParentalRelationshipArgs = (Person, Person)

//
// Predicates
//

let parent =
  Predicate<*ParentalRelationshipArgs>
    .named(#parent)

let grandparent =
  Predicate<*ParentalRelationshipArgs>
    .named(#grandparent)

let ancestor =
  Predicate<*ParentalRelationshipArgs>
    .named(#ancestor)


//
// Queries — values only for now
//

let descendant =
  Variable<Person>.named(#descendant)

let queryGrandchildren =
  Query.of(
    grandparent(Alice, descendant)
  )

let queryDescendants =
  Query.of(
    ancestor(Alice, descendant)
  )


//
// Variables
//

let x =
  Variable<Person>.named(#x)

let y =
  Variable<Person>.named(#y)

let z =
  Variable<Person>.named(#z)


//
// Facts
//

let aliceParentOfBob =
  parent(Alice, Bob)

let bobParentOfCharlie =
  parent(Bob, Charlie)

let aliceParentOfDiana =
  parent(Alice, Diana)


//
// Rules
//

let grandparentIsParentOfParent =
  Rule {
    head: grandparent(x, z),
    body: ( parent(x, y), parent(y, z), ),
  }


// One-element tuples must remain one-element tuples.
// Assuming Phalcom's trailing-comma tuple rule:
let parentIsAncestor =
  Rule {
    head: ancestor(x, y),

    body: (
      parent(x, y),
    ),
  }


let parentOfAncestorIsAncestor =
  Rule {
    head: ancestor(x, z),

    body: (
      parent(x, y),
      ancestor(y, z),
    ),
  }


//
// Knowledge base
//

let family =
  KnowledgeBase.of(
    aliceParentOfBob,
    bobParentOfCharlie,
    aliceParentOfDiana,

    grandparentIsParentOfParent,
    parentIsAncestor,
    parentOfAncestorIsAncestor
  )


System.print(parent)
System.print(grandparent)

System.print(aliceParentOfBob)
System.print(grandparentIsParentOfParent)

System.print(family)
