class Widget {
  @constructor new() {}
  @class
  make() { Widget.new() }

  render() {}
}

Widget./*@class*/make()

let widget = Widget.new()
widget./*@instance*/render()
