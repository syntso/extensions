use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::error::RegistryError;
use crate::index::{Index, Listing, Seal};
use crate::process;
use crate::registry::Extension;
use crate::signer::Signer;

const PUBLIC_KEY: &str = "registry.pub";
const INDEX: &str = "index.json";

pub fn publish(root: &Path, extensions: &[Extension], out: &Path) -> Result<(), RegistryError> {
    let signer = Signer::from_environment(&root.join(PUBLIC_KEY))?;

    let listings = extensions
        .iter()
        .map(|extension| list(extension, &signer, out))
        .collect::<Result<Vec<_>, _>>()?;

    let index = Index::new(listings);
    let path = out.join(INDEX);

    write(&path, &index.to_json()?)?;

    println!("wrote {}", path.display());
    Ok(())
}

fn list(extension: &Extension, signer: &Signer, out: &Path) -> Result<Listing, RegistryError> {
    let manifest = extension.manifest();
    let file = format!("{}.wasm", manifest.id);
    let path = out.join(&file);

    let bytes = fs::read(&path).map_err(|source| RegistryError::Read {
        path: path.clone(),
        source,
    })?;

    let sha256 = hex(&Sha256::digest(&bytes));
    let comment = Seal::new(manifest, &sha256).to_json()?;
    let signature = signer.sign(&bytes, &comment)?;

    write(&out.join(format!("{file}.minisig")), &signature)?;

    let commit = commit(extension)?;

    println!("signed {} {} at {commit}", manifest.id, manifest.version);
    Ok(Listing::new(manifest, commit, file, sha256, signature))
}

fn commit(extension: &Extension) -> Result<String, RegistryError> {
    let mut git = Command::new("git");
    git.args(["rev-parse", "HEAD"])
        .current_dir(extension.folder());

    Ok(process::run(&mut git)?.trim().to_owned())
}

fn write(path: &Path, contents: &str) -> Result<(), RegistryError> {
    fs::write(path, contents).map_err(|source| RegistryError::Write {
        path: path.to_owned(),
        source,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_as_lowercase_hex() {
        let digest = hex(&Sha256::digest(b"abc"));

        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
