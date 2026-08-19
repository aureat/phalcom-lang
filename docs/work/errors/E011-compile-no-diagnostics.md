```sh
➜  phalcom git:(main) ✗ cargo run -p phalcom-repl                                                              11:53:46
   Compiling phalcom-modules v0.1.0 (/Users/altunhasanli/dev/phalcom/phalcom/phalcom-modules)
   Compiling phalcom-core v0.1.0 (/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core)
   Compiling phalcom-lsp v0.1.0 (/Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp)
   Compiling phalcom-repl v0.1.0 (/Users/altunhasanli/dev/phalcom/phalcom/phalcom-repl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.05s
     Running `/Users/altunhasanli/.cargo-target/debug/phalcom-repl`
Failed compiling universe module errors/unsupported: Compile(ConstFieldWrite("_instance"))

thread 'main' (11510326) panicked at phalcom-core/src/vm/bootstrap.rs:190:35:
universe modules must compile and run cleanly: Compile(ConstFieldWrite("_instance"))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

```sh
Failed compiling universe module errors/unimplemented: Compile(Message("attr.unknown: unknown attribute `@final`"))

thread 'main' (11516959) panicked at phalcom-core/src/vm/bootstrap.rs:190:35:
universe modules must compile and run cleanly: Compile(Message("attr.unknown: unknown attribute `@final`"))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

```sh
Failed compiling universe module errors/unimplemented: Compile(Parse(SyntaxError { kind: UnrecognizedEof { expected: ["newline"] }, range: 170..170 }))

thread 'main' (11512769) panicked at phalcom-core/src/vm/bootstrap.rs:190:35:
universe modules must compile and run cleanly: Compile(Parse(SyntaxError { kind: UnrecognizedEof { expected: ["newline"] }, range: 170..170 }))
```