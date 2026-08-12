class Client {
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
