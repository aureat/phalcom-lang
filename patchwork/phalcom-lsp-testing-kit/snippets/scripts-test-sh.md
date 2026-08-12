# `scripts/test.sh` changes

Change:

```sh
lsp)
  cargo test -p phalcom-lsp --test integration "$@"
  ;;
```

to:

```sh
lsp)
  cargo test -p phalcom-lsp "$@"
  ;;
```

Optional additions:

```sh
vsphalcom)
  cargo build -p phalcom-lsp
  npm --prefix tools/vsphalcom test
  ;;
editor)
  cargo test -p phalcom-lsp
  cargo build -p phalcom-lsp
  npm --prefix tools/vsphalcom test
  ;;
```

Add `vsphalcom` and `editor` to the usage text.

A standalone `scripts/editor.sh` is included if you prefer not to widen the central script immediately.
