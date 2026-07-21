let reader = BytesReader.new(Bytes.fromString_("hello world"))
let dst = Bytes.new(5)
let f1 = reader.read(dst)
let n1 = f1.await
System.print(n1.toString)
System.print(dst.utf8_)

let dst2 = Bytes.new(10)
let f2 = reader.read(dst2)
let n2 = f2.await
System.print(n2.toString)
System.print(dst2.slice(0, n2).utf8_)

let dst3 = Bytes.new(5)
let f3 = reader.read(dst3)
let n3 = f3.await
System.print(n3.toString)

reader.close
