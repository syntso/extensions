use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("unknown argument {0}; see registry --help")]
    UnknownArgument(String),

    #[error("{flag} needs a value")]
    MissingValue { flag: &'static str },

    #[error("{flag} needs a number, got {value}")]
    NotNumber { flag: &'static str, value: String },

    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("{path}: {message}")]
    Json { path: String, message: String },

    #[error("{0} is not a pin list: {1}")]
    Pins(PathBuf, String),

    #[error("{id} is pinned twice")]
    PinnedTwice { id: String },

    #[error(
        "{0} is not an extension id: lowercase letters, digits and dashes, starting with a letter, at most 32"
    )]
    ExtensionId(String),

    #[error("{0} is not a full commit: 40 lowercase hex digits")]
    Rev(String),

    #[error("{0} is not a repository: give an https URL or an absolute path")]
    Repository(String),

    #[error("cannot run {program}: {source}")]
    Spawn {
        program: &'static str,
        #[source]
        source: io::Error,
    },

    #[error("{command} failed in {path}")]
    Failed { command: String, path: PathBuf },

    #[error("{id}: the checkout is at {found}, not the pinned {pinned}")]
    WrongCommit {
        id: String,
        found: String,
        pinned: String,
    },

    #[error("{0}: cargo built no component, or more than one; build one cdylib for wasm32-wasip2")]
    NoComponent(PathBuf),

    #[error("{0}: the manifest is not an object")]
    NotObject(String),

    #[error("{path} does not follow the schema: {problems}")]
    Schema { path: String, problems: String },

    #[error("{path}: {field} {value} is not valid semver: {message}")]
    Semver {
        path: String,
        field: &'static str,
        value: String,
        message: String,
    },

    #[error("{path}: the manifest is pinned as {pinned} but says {found}")]
    WrongId {
        path: String,
        pinned: String,
        found: String,
    },

    #[error("not a webassembly component: {0}")]
    Component(String),

    #[error("{id}: the component imports {import}, which {reason}")]
    Import {
        id: String,
        import: String,
        reason: String,
    },

    #[error("{id}: the component does not export {export} of the synt:extension world")]
    Export { id: String, export: &'static str },

    #[error("{0}: the listing is missing {1}")]
    ListingField(String, &'static str),

    #[error("{0}: the manifest in the index is not in canonical form")]
    NotCanonical(String),

    #[error("{id}: the manifest in the index differs from extension.json at {rev}")]
    ManifestDiffers { id: String, rev: String },

    #[error("{id}: the index lists {found}, expected {expected}")]
    FileName {
        id: String,
        found: String,
        expected: String,
    },

    #[error("{file} has sha256 {found}, the index says {listed}")]
    Digest {
        file: String,
        found: String,
        listed: String,
    },

    #[error("the index lists {found:?}, but extensions.json pins {pinned:?}")]
    IndexDiffers {
        found: Vec<String>,
        pinned: Vec<String>,
    },

    #[error("{0} is in the dist folder but not in the index")]
    StrayFile(String),

    #[error("no signing key: pass --key <file> or set REGISTRY_KEY")]
    NoKey,

    #[error("the signing key is not usable: {0}")]
    Key(String),

    #[error("the signing key does not belong to registry.pub ({0})")]
    ForeignKey(String),

    #[error("signing failed: {0}")]
    Sign(String),

    #[error("the clock is before 1970")]
    Clock,
}
