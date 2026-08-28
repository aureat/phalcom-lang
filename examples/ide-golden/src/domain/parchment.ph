"""
Bird
├── Raptor
│   ├── Falcon
│   │   ├── Peregrine
│   │   ├── Saker
│   │   └── Gyrfalcon
|   │
│   ├── Eagle
│   └── Hawk
│
├── Songbird
│   ├── Wren
│   ├── Sparrow
│   ├── Robin
│   └── Canary
│
└── Apodiform Bird
    ├── Swift
    └── Hummingbird
        └── Colibri
"""

class Bird {}

class Raptor is Bird {}

class Falcon is Raptor {}

class Eagle is Raptor {}

class Hawk is Raptor {}

class Peregrine is Falcon {}

class Saker is Falcon {}

class Gyrfalcon is Falcon {}

class Songbird is Bird {}

class Wren is Songbird {}

class Sparrow is Songbird {}

class Robin is Songbird {}

class Canary is Songbird {}

class ApodiformBird is Bird {}

class Swift is ApodiformBird {}

class Hummingbird is ApodiformBird {}

class Colibri is Hummingbird {}

class Falcons {
  @class Peregrine {
    Peregrine.new()
  }

  @class Saker {
    Saker.new()
  }

  @class Gyrfalcon {
    Gyrfalcon.new()
  }

  @class doesBelong(_ bird: Bird) {
    if bird is Falcon {
      true
    } else {
      false
    }
  }
}

class Main {
  @class main {
    let peregrine = Falcons.Peregrine
    let saker = Falcons.Saker
    let gyrfalcon = Falcons.Gyrfalcon

    let wren = Wren.new()
    let sparrow = Sparrow.new()
    let robin = Robin.new()
    let canary = Canary.new()

    let r1 = Falcons.doesBelong(peregrine)
    let r2 = Falcons.doesBelong(wren)
    System.print(r1)
    System.print(r2)
  }
}
