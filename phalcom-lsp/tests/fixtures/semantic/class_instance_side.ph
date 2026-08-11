class Widget {
  @class
  make() { Widget.new() }

  render() {}
}

Widget./*@class*/make()

const widget = Widget.new()
widget./*@instance*/render()
