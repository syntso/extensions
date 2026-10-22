mod checkout;
mod compilation;
mod component;
mod error;
mod index;
mod invocation;
mod listing;
mod manifest;
mod pin;
mod pins;
mod registry;
mod seal;
mod sha256;
mod signer;

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use error::RegistryError;
use index::Index;
use invocation::{Invocation, Places, Signing};
use registry::Registry;
use seal::Seal;
use signer::Signer;

const KEY_VARIABLE: &str = "REGISTRY_KEY";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("registry: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), RegistryError> {
    match Invocation::from_env()? {
        Invocation::Check(places) => check(&places),
        Invocation::Build(places) => build(&places),
        Invocation::Sign(places, signing) => sign(&places, &signing),
        Invocation::Help => {
            println!("{}", Invocation::USAGE);
            Ok(())
        }
        Invocation::Version => {
            println!("registry {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

fn check(places: &Places) -> Result<(), RegistryError> {
    let registry = Registry::open(&places.pins, &places.work)?;

    for (listing, component) in registry.check()? {
        println!(
            "checked {} {} sha256 {} ({} imports)",
            listing.manifest().id(),
            listing.manifest().version(),
            component.sha256(),
            component.imports().len()
        );
    }

    Ok(())
}

fn build(places: &Places) -> Result<(), RegistryError> {
    let registry = Registry::open(&places.pins, &places.work)?;
    let index = registry.build(&places.dist)?;

    report("built", &index, &places.dist);

    Ok(())
}

fn sign(places: &Places, signing: &Signing) -> Result<(), RegistryError> {
    let secret = match &signing.key {
        Some(path) => read(path)?,
        None => env::var(KEY_VARIABLE).map_err(|_| RegistryError::NoKey)?,
    };
    let signer = Signer::new(&secret, &read(&signing.public_key)?)?;
    let seal = Seal::new(signing.serial, signing.days)?;

    let registry = Registry::open(&places.pins, &places.work)?;
    let index = registry.sign(&places.dist, &signer, seal)?;

    report("signed", &index, &places.dist);
    println!("serial {} expires {}", seal.serial(), seal.expires());

    Ok(())
}

fn report(verb: &str, index: &Index, dist: &Path) {
    for listing in index.listings() {
        println!("{verb} {} sha256 {}", listing.file(), listing.sha256());
    }

    println!(
        "{verb} {} with {} extensions",
        dist.join(registry::INDEX).display(),
        index.listings().len()
    );
}

fn read(path: &Path) -> Result<String, RegistryError> {
    fs::read_to_string(path).map_err(|source| RegistryError::Read {
        path: path.to_owned(),
        source,
    })
}
