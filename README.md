# shx

An open-source, minimal Warp alternative for developers.

`shx` is a terminal assistant. You type what you want in plain language; it works
out whether you meant a shell command or a request, and either runs the command
or asks your model provider what to do. Nothing is executed without showing you
first.

Status: early. The crate builds and the module map below is being filled in
commit by commit — see [`AGENTS.md`](AGENTS.md) for the layout.

## Development

```sh
git clone https://github.com/krey-yon/shx.git
cd shx
cargo install --path .
```

The gate is exact, and CI runs these same commands with `-D warnings`, so
nothing passes there and fails locally:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps   # must also be warning-free
```

Run a single test:

```sh
cargo test --lib safety::analyzer::tests::classifies_rm_rf_slash_as_critical
```

## License

MIT. See [LICENSE](LICENSE).
