# Synt extensions

The registry of extensions for [Synt](https://synt.so). Synt installs extensions from here, and only from here unless you build one yourself.

An extension is a WebAssembly component that follows the one extension API, `synt:extension@1.0.0`, and lives in its own public repository. This repository does not hold extension code. It holds `extensions.json`, a list of pinned commits, and the tool that turns those commits into signed builds. We build every extension from its source here, so the build you install is exactly the code that was reviewed.

The registry is empty for now. The first extensions come soon.

<br />

## How an extension gets listed

1. Put the extension in a public repository: `extension.json` and a `Cargo.toml` at the root, with `Cargo.lock` committed. It builds with `cargo build --target wasm32-wasip2 --release --locked`.
2. Open a pull request here that adds the extension to `extensions.json`:

   ```json
   [
     {
       "id": "docker",
       "repository": "https://github.com/someone/synt-docker",
       "rev": "0123456789abcdef0123456789abcdef01234567"
     }
   ]
   ```

   `id` is the id in the manifest, `rev` is a full commit. A new version is a pull request that moves `rev`.
3. CI clones the commit, builds it, checks the manifest against the schema and checks that the component imports only the `synt:extension` 1.x interfaces its manifest allows, plus the few WASI interfaces the core links (io, cli, the monotonic clock and random). It also checks that the component exports the whole world.
4. Synt reviews the source, the capabilities and the actions. Once the pull request is merged, CI builds again, a separate job signs the result, and a release named `index-<n>` becomes the latest.

The index is signed again every week and expires three weeks after it was signed, so an old copy cannot be passed off as current for long.

<br />

## How Synt installs from it

The core downloads from `https://github.com/syntso/extensions/releases/latest/download`:

| File | What it is |
|---|---|
| `index.json` | `{ "extensions": [...] }`: each listing is the extension's manifest in its canonical form, plus `file`, `sha256` and `signature` |
| `index.json.minisig` | The [minisign](https://jedisct1.github.io/minisign/) signature of `index.json`. Its trusted comment is `{"serial": n, "expires": unix seconds}` |
| `<id>-<version>.wasm` | The component |

Before it shows you anything, the core checks the index signature against the key built into it, refuses an index that has expired or whose serial is older than one it has already seen, then checks the component it downloads against the listed `sha256` and against the listing's own signature. That signature's trusted comment seals the id, name, version, description, repository, the sha256 of the component and the sha256 of the manifest, so a listing changed after signing is refused. Then it shows you the capabilities and actions and waits for your approval.

<br />

## Check a component yourself

The registry's public key:

```text
untrusted comment: minisign public key 42FAC173CE26C745
RWRFxybOc8H6Ql6aaTcNVx3n0SU72+dq2v+HkYaBj95SvaIxdf3BmsO9
```

It is also in [`registry.pub`](registry.pub). With [minisign](https://jedisct1.github.io/minisign/) and [jq](https://jqlang.org):

```bash
base=https://github.com/syntso/extensions/releases/latest/download
curl -sLO $base/index.json -sLO $base/index.json.minisig
minisign -Vm index.json -P RWRFxybOc8H6Ql6aaTcNVx3n0SU72+dq2v+HkYaBj95SvaIxdf3BmsO9

id=docker
file=$(jq -r ".extensions[] | select(.id == \"$id\") | .file" index.json)
curl -sLO $base/$file
jq -r ".extensions[] | select(.id == \"$id\") | .signature" index.json > $file.minisig
minisign -Vm $file -P RWRFxybOc8H6Ql6aaTcNVx3n0SU72+dq2v+HkYaBj95SvaIxdf3BmsO9
shasum -a 256 $file
```

To see that the build matches the source, clone the pinned commit and build it the same way: `cargo build --target wasm32-wasip2 --release --locked`, with the toolchain in [`rust-toolchain.toml`](rust-toolchain.toml).

<br />

## Build

```bash
git clone https://github.com/syntso/extensions.git
cd extensions
git config core.hooksPath .githooks
cargo test
cargo run --release -- check
```

`cargo run -- --help` lists the commands: `check`, `build` and `sign`. [CONTRIBUTING.md](CONTRIBUTING.md) explains them.

<br />

## License

[Apache-2.0](LICENSE).
