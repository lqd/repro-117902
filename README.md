#### repro-117902

Minimal reproducer for [issue #117902](https://github.com/rust-lang/rust/issues/117902) for `wgpu`, with crucial help from [@saethlin](https://github.com/saethlin).

`cargo run --release` will segfault on `1.75.0-beta.5` or the equivalent recent nightly, at least on `aarch64-apple-darwin`.
