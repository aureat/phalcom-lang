# Manual VS Code smoke checklist

Run this after automated tests and after producing a VSIX.

## Build and package

```sh
cargo build -p phalcom-lsp

cd tools/vsphalcom
npm run vsix
code --install-extension ./vsphalcom-0.0.1.vsix --force
```

Restart/reload VS Code after replacing an already-installed build.

## Workspace

Create `smoke.ph`:

```phalcom
class Animal {
  move() {}
}

class Dog is Animal {
  bark() {}
}

const dog = Dog.new()
dog.
```

Check:

- syntax highlighting is active;
- `dog.` suggests `bark()` and inherited `move()`;
- class-side-only members do not appear as instance members;
- the inferred type hint for `dog` appears if hints are enabled;
- hover on `dog` identifies the inferred runtime class;
- go-to-definition from an inherited member reaches the correct declaration.

## `super`

Add:

```phalcom
class Parent {
  parentOnly() {}
}

class Child is Parent {
  childOnly() {}

  test() {
    super.
  }
}
```

Check:

- `super.` offers `parentOnly()`;
- `childOnly()` is not offered through `super`;
- definition navigation for a super-selected method lands in `Parent`.

## Live edit

Change:

```phalcom
const dog = Dog.new()
```

to a different class construction.

Without restarting the extension:

- inlay hint changes;
- hover changes;
- completion changes;
- stale members disappear.

## Module identity

Create:

`a.ph`
```phalcom
class User {
  aOnly() {}
}
```

`b.ph`
```phalcom
class User {
  bOnly() {}
}
```

`main.ph`
```phalcom
import "./a" as A
import "./b" as B

A.User.new().
B.User.new().
```

Check:

- first receiver offers `aOnly()` but not `bOnly()`;
- second receiver offers `bOnly()` but not `aOnly()`.

## Recovery

Temporarily leave:

```phalcom
dog.
```

at EOF and an unclosed class elsewhere. Completion should degrade gracefully rather than disappear globally or crash the server.

## Logs

If something fails, capture:
- Phalcom Language Server output channel;
- VS Code extension-host log;
- source file and cursor location;
- whether a reload changes the result.

A bug that disappears only after reload is usually an invalidation/revision bug, not a successful smoke result.
