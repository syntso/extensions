# Contributing

## Branches

- `main` holds what is published. Every push to it builds, signs and releases the index.
- `dev` is where changes come together. Branch from it, and open pull requests into it.
- Name branches after the change: `feat/add-disk`, `feat/update-uptime`, `fix/index-format`.

Pull requests into `dev` are squash-merged, so the pull request title becomes the commit.

## Commits

[Conventional Commits](https://www.conventionalcommits.org), one line, no body.

```text
feat: add disk
feat: update uptime to 0.2.0
fix(registry): reject an empty capability list
```

- Types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`, `style`, `revert`.
- Scope: `registry` for the tool, or nothing.
- Subject: imperative, starts lowercase, no period, the whole line at most 72 characters.

`git config core.hooksPath .githooks` turns on the local check.

## Reviewing an extension

- Read the code at the pinned commit, not at the tip of its repository.
- The capabilities are the smallest set the code needs. A new capability in an update gets a second look.
- The extension's own dependencies are permissively licensed, and nothing is fetched at build time beyond crates.

## Code

The `registry` tool in `src` follows the rules of the core:

- One main type per file, named after it. A concept with several files gets its own folder.
- Errors with `thiserror`. No `unwrap()` outside tests.
- No comments and no doc comments. Names, types and small functions carry the meaning.
- A blank line between logical steps; guards first, the main path flat below.

## Before you push

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --release -- build
```

CI runs the tests on macOS, Windows and Linux, plus `cargo deny check` for dependency licenses and the build of every extension.
