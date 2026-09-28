# shx

An open-source, minimal Warp alternative for developers.

`shx` is a terminal assistant. You type what you want in plain language; it works
out whether you meant a shell command or a request, and either runs the command
or asks your model provider what to do. Nothing is executed without showing you
first.

![A real shx session: a shell command runs, `rm -rf /` is refused outright, a command left as a template is held back for editing, a question with no API key says exactly what to set, and `/history` lists what ran.](docs/session.png)

That screenshot is a real session, not a mockup. One prompt took all five kinds of
input: a command that ran, a command that was refused, a command the model left as
a template, a question, and a built-in.

## Why it exists

Warp is good, and closed source. `shx` is a small, readable Rust program you can
audit: the entire path from your keystrokes to a spawned process is a few hundred
lines, and every command that touches your disk goes through a risk classifier
first.

## Features

- **One prompt for everything.** Shell commands run directly; natural language
  goes to a model. You do not pick a mode.
- **Refuses the dangerous ones.** Commands are classified safe, low, medium, high
  or critical. Critical commands are refused outright, high risk needs capitals,
  medium needs a lowercase `yes`. `rm -rf /` is not confirmable by any answer.
- **Catches what the model left as a template.** `brew install <package>` is held
  back for editing rather than run literally.
- **Knows your machine.** Package-manager commands are generated for your actual
  distribution, so it suggests `pacman -S` and not `apt install`.
- **Any provider.** Gemini, Anthropic, OpenAI, Ollama and the rest, through one
  interface.
- **Works without a key.** The shell path needs no provider at all.
- **Persistent history and completion** for everything you have run.

## Install

```sh
git clone https://github.com/krey-yon/shx.git
cd shx
cargo install --path .
```

Or grab a release tarball from the releases page and run `install.sh` inside it.

## Usage

```
shx                        start the assistant
shx "install ripgrep"      run one request and exit
shx --provider ollama      choose a provider
shx --model <name>         override the model for this run
shx --dry-run "clear disk" explain what it would do, run nothing
shx --verbose              debug logging to stderr
shx config show            print the current settings
shx config set model <name>
shx config path            where the config file lives
```

Inside the REPL, `cd`, `clear`/`cls` and `exit`/`quit` are handled locally. Prefix
anything with `/` for a built-in:

| Command | Effect |
| --- | --- |
| `/help` | list commands and keys |
| `/clear` | clear the screen |
| `/model` | show or change the model |
| `/provider` | show or change the provider |
| `/history` | what you have run this session |
| `/forget` | clear the session history |
| `/exit` | quit |

## Configure

On first run `shx` asks for a provider and an API key, then writes
`~/.shx/config.json` and `~/.shx/credentials.json` with owner-only permissions.
To keep the key out of the config entirely, export the provider's variable and
`shx` will use that:

```sh
export GEMINI_API_KEY=...      # or ANTHROPIC_API_KEY, OPENAI_API_KEY
```

Every setting can also come from the environment, which wins over the file:
`SHX_PROVIDER`, `SHX_MODEL`, `SHX_MAX_TOKENS`, `SHX_TEMPERATURE`,
`SHX_COMMAND_TIMEOUT`. Flags win over the environment.

## Development

The gate is exact, and CI runs these same commands with `-D warnings`, so nothing
passes there and fails locally:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps   # must also be warning-free
```

Agent tests use mock completion models and a fixed prompter, so the whole suite
runs offline with no API key and no terminal. Run a single test:

```sh
cargo test --lib safety::safety_analyzer
```

Conventions that are not Rust defaults live in [`AGENTS.md`](AGENTS.md).

## License

MIT. See [LICENSE](LICENSE).
