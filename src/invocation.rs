use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::RegistryError;
use crate::seal::Seal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    Check(Places),
    Build(Places),
    Sign(Places, Signing),
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Places {
    pub pins: PathBuf,
    pub work: PathBuf,
    pub dist: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signing {
    pub key: Option<PathBuf>,
    pub public_key: PathBuf,
    pub serial: Option<u64>,
    pub days: u64,
}

impl Invocation {
    pub const USAGE: &str = "\
registry: builds, checks and signs the Synt extensions registry

Usage: registry check [--pins <file>] [--work <dir>]
       registry build [--pins <file>] [--work <dir>] [--dist <dir>]
       registry sign  [--pins <file>] [--work <dir>] [--dist <dir>] [--key <file>]
                      [--public-key <file>] [--serial <n>] [--days <n>]

  check    clone every pinned extension at its commit, build it for wasm32-wasip2,
           check its manifest and the imports of its component
  build    check, then write the components and the unsigned index into the dist folder
  sign     check the dist folder again and sign the components and the index

  --pins <file>         the pinned extensions, default extensions.json
  --work <dir>          where extensions are cloned and built, default work
  --dist <dir>          the release files, default dist
  --key <file>          the minisign secret key; without it, the key text in REGISTRY_KEY
  --public-key <file>   the key the core trusts, default registry.pub
  --serial <n>          the serial of the index, default the current unix time
  --days <n>            days until the index expires, default 21
  --help, --version";

    pub fn from_env() -> Result<Self, RegistryError> {
        Self::parse(env::args_os().skip(1))
    }

    fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, RegistryError> {
        let mut arguments = arguments.into_iter();

        let command = match arguments.next() {
            None => return Ok(Self::Help),
            Some(command) => command.to_string_lossy().into_owned(),
        };

        let mut places = Places {
            pins: PathBuf::from("extensions.json"),
            work: PathBuf::from("work"),
            dist: PathBuf::from("dist"),
        };
        let mut signing = Signing {
            key: None,
            public_key: PathBuf::from("registry.pub"),
            serial: None,
            days: Seal::DAYS,
        };

        match command.as_str() {
            "--help" | "-h" | "help" => return Ok(Self::Help),
            "--version" | "-V" => return Ok(Self::Version),
            "check" | "build" | "sign" => {}
            _ => return Err(RegistryError::UnknownArgument(command)),
        }

        let signs = command == "sign";

        while let Some(argument) = arguments.next() {
            let flag = argument.to_string_lossy().into_owned();

            match flag.as_str() {
                "--pins" => places.pins = Self::path(&mut arguments, "--pins")?,
                "--work" => places.work = Self::path(&mut arguments, "--work")?,
                "--dist" if command != "check" => {
                    places.dist = Self::path(&mut arguments, "--dist")?;
                }
                "--key" if signs => signing.key = Some(Self::path(&mut arguments, "--key")?),
                "--public-key" if signs => {
                    signing.public_key = Self::path(&mut arguments, "--public-key")?;
                }
                "--serial" if signs => {
                    signing.serial = Some(Self::number(&mut arguments, "--serial")?);
                }
                "--days" if signs => signing.days = Self::number(&mut arguments, "--days")?,
                _ => return Err(RegistryError::UnknownArgument(flag)),
            }
        }

        Ok(match command.as_str() {
            "check" => Self::Check(places),
            "build" => Self::Build(places),
            _ => Self::Sign(places, signing),
        })
    }

    fn path(
        arguments: &mut impl Iterator<Item = OsString>,
        flag: &'static str,
    ) -> Result<PathBuf, RegistryError> {
        arguments
            .next()
            .map(PathBuf::from)
            .ok_or(RegistryError::MissingValue { flag })
    }

    fn number(
        arguments: &mut impl Iterator<Item = OsString>,
        flag: &'static str,
    ) -> Result<u64, RegistryError> {
        let value = arguments
            .next()
            .ok_or(RegistryError::MissingValue { flag })?
            .to_string_lossy()
            .into_owned();

        value
            .parse()
            .map_err(|_| RegistryError::NotNumber { flag, value })
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::{Invocation, Places};

    fn parse(line: &str) -> Result<Invocation, String> {
        Invocation::parse(line.split_whitespace().map(OsString::from))
            .map_err(|error| error.to_string())
    }

    #[test]
    fn reads_the_commands_with_their_defaults() {
        let places = Places {
            pins: PathBuf::from("extensions.json"),
            work: PathBuf::from("work"),
            dist: PathBuf::from("dist"),
        };

        assert_eq!(parse("check").unwrap(), Invocation::Check(places.clone()));
        assert_eq!(parse("").unwrap(), Invocation::Help);

        let Invocation::Sign(places, signing) =
            parse("sign --dist out --key k --serial 7 --days 3").unwrap()
        else {
            panic!("not sign");
        };

        assert_eq!(places.dist, PathBuf::from("out"));
        assert_eq!(signing.key, Some(PathBuf::from("k")));
        assert_eq!(signing.serial, Some(7));
        assert_eq!(signing.days, 3);
    }

    #[test]
    fn refuses_flags_of_other_commands_and_missing_values() {
        for line in [
            "check --key k",
            "check --dist d",
            "build --serial 1",
            "sign --days",
            "sign --days many",
            "publish",
        ] {
            assert!(parse(line).is_err(), "{line}");
        }
    }
}
