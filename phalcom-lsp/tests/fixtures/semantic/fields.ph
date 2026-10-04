class Client {
  @constructor new() {}
  request() {}
}

class Service {
  mut _client
  @constructor
  new() {
    _client = Client.new()
  }

  run() {
    _client./*@field*/request()
  }
}
