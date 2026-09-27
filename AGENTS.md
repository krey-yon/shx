# AGENTS.md

Instructions for agents working in this repository.

## What this is

`shx` is a Rust rewrite of a Python terminal assistant. It is a line-oriented
REPL: it classifies what you type, decides whether it is a shell command or a
natural-language request, and for the latter asks a model provider via
[`rig`](https://rig.rs). It is **not** a terminal emulator — there is no PTY and
no multiplexer.

## The gate

Every commit must leave this green. CI runs the identical commands.

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps
```

`cargo doc` matters: the crate has `missing_docs` and broken intra-doc links as
warnings, so a new public item without a doc comment fails the build.

## Conventions that differ from the default

- **Bindings are `snake_case`, types and modules are `PascalCase`.** Do not
  "fix" a `PascalCase` variable to make it consistent — Rust requires the
  opposite, and the rustc `non_snake_case` lint is the arbiter.
- **File and module names are full words.** `shell/command_classifier.rs`, not
  `shell/classify.rs`. `safety/safety_analyzer.rs`, not `safety/analyser.rs`.
- **One concern per module.** A module that both classifies and executes
  commands is two modules. When in doubt, split and pass the value across.
- **Keep files small: 300 lines of code, excluding the `#[cfg(test)]` module.**
  A `#[cfg(test)] mod tests` may be as long as the tests need. If the code
  exceeds the cap, that is the module wanting to be two modules — split it
  rather than compressing it.
- **Comment sparingly.** A comment that restates the line below it is noise. Keep
  one only where the *why* is not derivable from the code: a rule the code
  cannot state, a constraint inherited from a platform or a protocol, a decision
  that looks wrong until you know the reason. Never narrate the control flow.
- **Tests live next to the code they test** in a `#[cfg(test)] mod tests`. Only
  end-to-end REPL flows go in `tests/`.
- **Doc comments (`///`) are required on public items** — `missing_docs` is a
  build failure. Inline `//` comments are held to the standard above.
- **No new dependencies** without a reason written in the commit body. The
  manifest is deliberately small.
- **No `unwrap` or `expect` outside `#[cfg(test)]`.** `expect_used` and
  `unwrap_used` are warn-level, which CI promotes to an error.
- **`#![forbid(unsafe_code)]` is in the manifest.** It is not negotiable.

## Commands worth knowing

```sh
cargo test --lib <module path>          # one module's tests
cargo test --lib <filter> -- --nocapture # see printed output
cargo clippy --fix                      # auto-fix what clippy can
cargo doc --open                        # read the rendered docs
```

## Tooling

`rust-skills` is vendored for agents but is **gitignored** — it is a tool, not
project source. If it is missing:

```sh
git clone --depth 1 https://github.com/leonardomso/rust-skills.git .opencode/skills/rust-skills
```

Read `SKILL.md` there as an index and open only the `rules/*.md` files relevant
to the code in front of you. The prefixes map to categories: `own-` (ownership
and borrowing), `err-` (error handling), `test-`, `serde-`, `name-`, `lint-`,
`anti-`, `type-`, `mem-`, `proj-`.

## Agent parallelism

Multiple agents share one working tree and one `.git` index. If you were spawned
as a subagent:

- **Only touch the files you were assigned.** The partition is the module map in
  `src/lib.rs`.
- **Never run `git add`, `git commit`, or `git checkout`.** The orchestrating
  thread owns the index; parallel commits corrupt it.
- **Do not add dependencies or change `Cargo.toml`.** Report the need instead.

## Environment

- Rust 1.92.0, edition 2024, pinned in `rust-toolchain.toml`. The MSRV is real:
  `reedline` is held at 0.49 because 0.50 requires rustc 1.95.
- Target platforms are Linux, macOS, and Windows. Platform detection is a
  runtime branch, not a `cfg` — it reads `/etc/os-release` at runtime, so the
  same binary reports the right distribution on every machine.
