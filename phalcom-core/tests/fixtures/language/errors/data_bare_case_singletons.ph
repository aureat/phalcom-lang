// area: errors
// spec: objects/enums.md
// status: PASS

data Ordering {
  Less
  Greater
}

System.print(Ordering::Less)
System.print(Ordering::Greater)
System.print(Ordering::Less is Ordering)
System.print(Ordering::Greater is Ordering)
