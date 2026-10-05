let mut reader = BytesReader.new(Bytes.fromString("hello world"))
let mut dst = Bytes.new(5)
let mut f1 = reader.read(dst)
let mut n1 = f1.await
System.print(n1.toString)
System.print(dst.utf8)

let mut dst2 = Bytes.new(10)
let mut f2 = reader.read(dst2)
let mut n2 = f2.await
System.print(n2.toString)
System.print(dst2.slice(0, n2).utf8)

let mut dst3 = Bytes.new(5)
let mut f3 = reader.read(dst3)
let mut n3 = f3.await
System.print(n3.toString)

reader.close
