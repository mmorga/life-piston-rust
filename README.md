# life-piston-rust

Learning rust (and a little piston).

This isn't probably useful to anyone. Just me playing around.

## Building life

Install rust

```
brew install rustup-init
rustup update
```

Build/run the debug version

```
cargo run
```

Build the release/optimized version

```
cargo rustc --release -- -C target-cpu=native
```
