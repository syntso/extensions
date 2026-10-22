use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::RegistryError;
use crate::checkout::Checkout;
use crate::compilation::Compilation;
use crate::component::Component;
use crate::index::Index;
use crate::listing::Listing;
use crate::manifest::Manifest;
use crate::pin::Pin;
use crate::pins::Pins;
use crate::seal::Seal;
use crate::sha256::Sha256;
use crate::signer::Signer;

pub const INDEX: &str = "index.json";
pub const INDEX_SIGNATURE: &str = "index.json.minisig";
const MANIFEST: &str = "extension.json";

pub struct Registry {
    pins: Pins,
    work: PathBuf,
}

impl Registry {
    pub fn open(pins: &Path, work: &Path) -> Result<Self, RegistryError> {
        let pins = Pins::load(pins)?;
        let work = Self::absolute(work)?;

        Ok(Self { pins, work })
    }

    pub fn check(&self) -> Result<Vec<(Listing, Component)>, RegistryError> {
        self.pins.iter().map(|pin| self.prepare(pin)).collect()
    }

    pub fn build(&self, dist: &Path) -> Result<Index, RegistryError> {
        let prepared = self.check()?;

        Self::fresh_folder(dist)?;

        for (listing, component) in &prepared {
            Self::write(&dist.join(listing.file()), component.bytes())?;
        }

        let index = Index::new(prepared.into_iter().map(|(listing, _)| listing).collect());
        Self::write(&dist.join(INDEX), &index.to_bytes())?;

        Ok(index)
    }

    pub fn sign(&self, dist: &Path, signer: &Signer, seal: Seal) -> Result<Index, RegistryError> {
        let index = Index::read(&dist.join(INDEX))?;

        self.recheck_folder(dist, &index)?;

        let mut listings = Vec::new();

        for listing in index.into_listings() {
            let wasm = self.recheck(dist, &listing)?;
            let signature = signer.sign(&wasm, &listing.seal().to_string())?;

            listings.push(listing.signed(signature));
        }

        let index = Index::new(listings);
        let bytes = index.to_bytes();
        let signature = signer.sign(&bytes, &seal.trusted_comment())?;

        Self::write(&dist.join(INDEX), &bytes)?;
        Self::write(&dist.join(INDEX_SIGNATURE), signature.as_bytes())?;

        Ok(index)
    }

    fn prepare(&self, pin: &Pin) -> Result<(Listing, Component), RegistryError> {
        let checkout = Checkout::fetch(pin, &self.work)?;
        let manifest = Self::pinned_manifest(pin, &checkout)?;

        let bytes = Compilation::run(&checkout)?;
        let component = Component::parse(bytes)?;
        component.check(&manifest)?;

        Ok((Listing::new(manifest, &component), component))
    }

    fn recheck_folder(&self, dist: &Path, index: &Index) -> Result<(), RegistryError> {
        let pinned = self.pins.ids();
        let found = index.ids();

        if pinned != found {
            return Err(RegistryError::IndexDiffers { found, pinned });
        }

        let mut expected: BTreeSet<String> = index
            .listings()
            .iter()
            .map(|listing| listing.file().to_owned())
            .collect();
        expected.insert(INDEX.to_owned());
        expected.insert(INDEX_SIGNATURE.to_owned());

        let entries = fs::read_dir(dist).map_err(|source| RegistryError::Read {
            path: dist.to_owned(),
            source,
        })?;

        for entry in entries {
            let entry = entry.map_err(|source| RegistryError::Read {
                path: dist.to_owned(),
                source,
            })?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let plain = entry.file_type().is_ok_and(|kind| kind.is_file());

            if !plain || !expected.contains(&name) {
                return Err(RegistryError::StrayFile(name));
            }
        }

        Ok(())
    }

    fn recheck(&self, dist: &Path, listing: &Listing) -> Result<Vec<u8>, RegistryError> {
        let manifest = listing.manifest();
        let id = manifest.id().to_string();
        let expected = Listing::file_name(manifest);

        if listing.file() != expected {
            return Err(RegistryError::FileName {
                id,
                found: listing.file().to_owned(),
                expected,
            });
        }

        let path = dist.join(listing.file());
        let wasm = fs::read(&path).map_err(|source| RegistryError::Read { path, source })?;
        let found = Sha256::of(&wasm);

        if found.as_str() != listing.sha256() {
            return Err(RegistryError::Digest {
                file: listing.file().to_owned(),
                found: found.to_string(),
                listed: listing.sha256().to_owned(),
            });
        }

        let component = Component::parse(wasm)?;
        component.check(manifest)?;

        let pin = self
            .pins
            .find(&id)
            .ok_or_else(|| RegistryError::IndexDiffers {
                found: vec![id.clone()],
                pinned: self.pins.ids(),
            })?;
        let checkout = Checkout::fetch(pin, &self.work)?;

        if Self::pinned_manifest(pin, &checkout)? != *manifest {
            return Err(RegistryError::ManifestDiffers {
                id,
                rev: pin.rev().to_string(),
            });
        }

        Ok(component.bytes().to_vec())
    }

    fn pinned_manifest(pin: &Pin, checkout: &Checkout) -> Result<Manifest, RegistryError> {
        let path = checkout.file(MANIFEST);
        let manifest = Manifest::read(&path)?;

        if manifest.id() != pin.id() {
            return Err(RegistryError::WrongId {
                path: path.display().to_string(),
                pinned: pin.id().to_string(),
                found: manifest.id().to_string(),
            });
        }

        Ok(manifest)
    }

    fn fresh_folder(path: &Path) -> Result<(), RegistryError> {
        let failed = |source| RegistryError::Write {
            path: path.to_owned(),
            source,
        };

        if path.exists() {
            fs::remove_dir_all(path).map_err(failed)?;
        }

        fs::create_dir_all(path).map_err(failed)
    }

    fn write(path: &Path, bytes: &[u8]) -> Result<(), RegistryError> {
        fs::write(path, bytes).map_err(|source| RegistryError::Write {
            path: path.to_owned(),
            source,
        })
    }

    fn absolute(path: &Path) -> Result<PathBuf, RegistryError> {
        std::path::absolute(path).map_err(|source| RegistryError::Read {
            path: path.to_owned(),
            source,
        })
    }
}
