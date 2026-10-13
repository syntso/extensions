<div align="center">

<img src="docs/images/logo.png" alt="Synt" width="96" />

# Synt Extensions

**The registry of extensions that Synt can install.**

</div>

---

Every extension lives in its own public repository and is pinned here as a git submodule. CI builds each one, signs it and publishes the result as a GitHub release that [Synt](https://github.com/syntso/synt) installs from.

- **Pinned.** Each extension is one exact commit. What was reviewed is what gets built.
- **Checked.** The manifest has to match, the API version has to be supported, and the component may only ask for what the host provides.
- **Signed.** Every `.wasm` is signed with the registry's [minisign](https://jedisct1.github.io/minisign/) key. Synt checks the signature and the sha256 before it installs anything.
- **You approve.** Installing shows what an extension may call, and every call it makes still asks you.

| Extension | What it does |
|---|---|
| [uptime](https://github.com/syntso/extension-uptime) | How long a host has been up, and how loaded it is |

## Add an extension

Write it first: see [extension-api](https://github.com/syntso/extension-api). Its repository needs `extension.toml` and `Cargo.toml` at the root and must build with `cargo build --release --target wasm32-wasip2`.

Then, in a fork of this repository:

```bash
git submodule add https://github.com/you/extension-disk.git extensions/disk
git -C extensions/disk checkout <commit>
```

Add it to `extensions.toml`, with the same id and version as its `extension.toml`:

```toml
[[extension]]
id = "disk"
path = "extensions/disk"
version = "0.1.0"
```

Check and build it the way CI does:

```bash
cargo run --release -- build
```

Open a pull request into `dev` titled `feat: add disk`.

## Update an extension

Move the submodule to the new commit and bump `version` in `extensions.toml` to match the new `extension.toml`:

```bash
git -C extensions/disk fetch && git -C extensions/disk checkout <commit>
```

Open a pull request into `dev` titled `feat: update disk to 0.2.0`.

## What CI checks

- `extensions.toml` and every `extension.toml` it lists: id and version match, `api` is supported (now `0.1`), and every entry in `capabilities.call` is a core action extensions may call.
- The built component: it exports `describe` and `run`, it is built against the declared API, it imports storage only with `storage = true`, and it imports nothing the host does not provide (no files, no network, no wall clock).

Pull requests are only checked and built. When `dev` is merged into `main`, and once a week, CI builds without secrets, then signs every component and `index.json` in a separate protected job, and publishes them with their signatures as the release `index-<run>`, marked latest.

## Verify a download

```bash
minisign -V -p registry.pub -m index.json
minisign -V -p registry.pub -m uptime.wasm
```

The trusted comment of `index.json.minisig` holds a serial that only grows and an expiry, so Synt refuses an older or stale index. The trusted comment of each component's signature holds its id, name, version, description, repository, sha256 and capabilities from `index.json`.

## License

[MIT](LICENSE).
