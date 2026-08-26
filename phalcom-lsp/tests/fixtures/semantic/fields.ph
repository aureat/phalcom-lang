class Client {
  @constructor new() {}
  request() {}
}

class Service {
  @constructor
  new() {
    _client = Client.new()
  }

  run() {
    _client./*@field*/request()
  }
}
