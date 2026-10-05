type Base = List<{name: String} | List<String>>
type Producer<T> = (T) -> Unit

class Immer<T> {
  @class
  call(base: T, with producer: Producer<T>) -> T {
    let draft = base.clone()
    producer(draft)
    draft
  }
}

let userInfo = {
  firstName: "Nazim",
  lastName: "Hikmet",
  age: 61
}

type ServerConfig = {server: String, port: Int, paths: List<String>}

let serverConfig: ServerConfig = {
  server: "localhost",
  port: 8080,
  paths: ["a", "b", "c"]
}

let mut next: Base = Immer(base: serverConfig, with: |draft| {
    draft.push(["z"])
})
