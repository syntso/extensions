# Contributing

## Branches

`main` is what gets published. Branch from it and open pull requests into it. Pull requests are squash-merged, so the pull request title becomes the commit.

## Commits

[Conventional Commits](https://www.conventionalcommits.org), one line, no body: `feat: list the docker extension`. The whole line is at most 72 characters. `git config core.hooksPath .githooks` turns on the local check.

## Listing an extension

Add or move a pin in `extensions.json`: `{ "id", "repository", "rev" }`, with the full commit as `rev` and an `https` URL as `repository`. Run `cargo run --release -- check` before you push; CI runs `build` on the pull request.

## The tool

`registry` has three commands:

- `check` clones every pinned extension at its commit into `work/`, builds it with `cargo build --release --locked --target wasm32-wasip2`, reads `extension.json` (comments and trailing commas allowed, as in Synt), checks it against `schema/extension.schema.json`, and checks the imports and exports of the component.
- `build` does the same and writes `dist/<id>-<version>.wasm` and an unsigned `dist/index.json`.
- `sign` checks `dist/` again, then signs each component and the index. It reads the key from `--key <file>` or from `REGISTRY_KEY`, an unencrypted minisign key (`minisign -G -W`), and refuses a key that does not match `registry.pub`. It picks the serial (the current unix time unless `--serial` says otherwise) and the expiry (21 days unless `--days` says otherwise), since both live only in the signature's trusted comment.

The listing a component is signed with carries the manifest in the core's canonical form: defaults left out, keys sorted, pretty-printed, as `Manifest::to_json` in the core writes it. The core hashes that text to compare the listing with its seal, so the form has to match byte for byte. `tests/fixtures` holds manifests with the text the core produced for each of them; the tests compare against it. When the core changes how it writes a manifest, regenerate those files from the core.

## The schema and the WIT

`schema/extension.schema.json` and `wit/synt-extension.wit` are copied from the [SDK](https://github.com/syntso/sdk), which is where they are changed. Copy them again when the SDK releases a new version of `synt:extension` 1.x, and update the list of interfaces and their capabilities in `src/component/import.rs` with them.

## Publishing

`publish.yml` runs on every push to `main`, every Monday and by hand. Two jobs, so the signing key never meets extension code:

1. `build` has no secrets and only read access. It clones and builds every pinned extension (their build scripts and macros run here) and uploads `dist/`.
2. `publish` runs in the protected `registry` environment, the only place `REGISTRY_KEY` exists. It builds nothing but this tool, from this repository's own commit and without a shared cache, downloads `dist/`, and runs `sign`. Before signing, `sign` checks again that the index lists exactly the pinned ids, that the folder holds nothing else, that every file has the listed sha256, that every manifest is in canonical form and equals `extension.json` at the pinned commit (fetched again, not built), and that every component imports only what its manifest allows and exports the world. Then it publishes a release `index-<run number>`, marked latest, with `index.json`, `index.json.minisig` and the components.

## Code

The tool follows the rules of the [core](https://github.com/syntso/synt/blob/dev/CONTRIBUTING.md): one main type per file, the folder's main type in the file beside the folder, errors with `thiserror`, no `unwrap()` outside tests, no comments.

## Before you push

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check
```
