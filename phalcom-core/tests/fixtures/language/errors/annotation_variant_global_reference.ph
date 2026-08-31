// area: errors
// spec: annotations-data.md @variant / this implementation specification
// status: PASS

enum Ordering {
  @variant Less
  @variant Greater
}

System.print(Ordering::Less)
System.print(Ordering::Greater)
System.print(Ordering::Less is Ordering)
System.print(Ordering::Greater is Ordering)
